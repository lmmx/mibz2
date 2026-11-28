# B5 review aid: differential fuzz of `front_avx2` against `front_scalar`

Local-only test written for the independent B5 review of `src/runs.rs` at `791ee415` (PR #1238, claim `runs-front-i9-rescue`); verdict in `frontier/deltas/2026-11-28-runs-front-i9-rescue-b5-review.md`. Not merged into `src/`. Apply on top of `791ee415` with `git apply`; it appends two tests to `src/runs/tests.rs`.

- `b5_review_front_avx2_differential_fuzz`: xorshift streams (seed `B5_SEED`, count `B5_ITERS`, 60 under Miri) mixing RUNA/RUNB bursts of 0..22 (incl. 18..23 RUNB-only, to hit `RunTooLong` and 2^22-row runs), literals in every `STEPS`/`second()` band and `0x7fff..0xffff`, EOB of 0/1/2..32/0x8000/0xffff or absent, `max_len` from 0 to 2^31, and 1-in-8 non-bijective `seq_to_unseq`. Compares result, records, counts, `per`, `trusted`, `len` and the fill's TT; reuses dirty `Runs`; counts the cases `front_avx2` itself serves (~44%).
- `b5_review_front_avx2_max_len`: m = 2^24 - 65, 2^24 - 64, 2^24 - 1 (served, equal to scalar) and 2^24 (refused to scalar).

Miri: `RUSTFLAGS="-C target-feature=+avx2,+bmi1,+bmi2,+lzcnt,+popcnt" MIRIFLAGS="-Zmiri-env-forward=B5_SEED" cargo +nightly miri test --lib -- runs::tests::b5_review_front_avx2_differential_fuzz`. Without those RUSTFLAGS Miri detects no AVX2 and never enters `front_avx2`.

```diff
diff --git a/src/runs/tests.rs b/src/runs/tests.rs
index f75b482..4cacf79 100644
--- a/src/runs/tests.rs
+++ b/src/runs/tests.rs
@@ -1584,3 +1584,83 @@ mod fold_tests {
         assert!(tried > 50, "{tried} blocks for the fold");
     }
 }
+
+/// B5 review (local only): randomized differential of `front_avx2` against `front_scalar`
+/// on adversarial streams, with dirty `Runs` reuse and the trusted fill.
+#[test]
+fn b5_review_front_avx2_differential_fuzz() {
+    let k = Kernels::detect();
+    assert!(front_fast(k), "this review needs the AVX2 front");
+    let mut st: u64 = std::env::var("B5_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(0x9e3779b97f4a7c15);
+    let mut rnd = move || { st ^= st << 13; st ^= st >> 7; st ^= st << 17; st };
+    let iters: usize = std::env::var("B5_ITERS").ok().and_then(|s| s.parse().ok()).unwrap_or(if cfg!(miri) { 60 } else { 20_000 });
+    let (mut r, mut rs) = (Runs::default(), Runs::default());
+    let (mut avx_some, mut ok_cases, mut err_cases) = (0usize, 0usize, 0usize);
+    for it in 0..iters {
+        let mode = rnd() % 8;
+        let len = match rnd() % 6 { 0 => (rnd() % 8) as usize, 1 => (rnd() % 70) as usize, 2 => (rnd() % 300) as usize,
+            3 => 60 + (rnd() % 10) as usize, 4 => (rnd() % 5000) as usize, _ => 127 + (rnd() % 140) as usize };
+        let eob: u16 = match rnd() % 10 { 0 => (rnd() % 2) as u16, 1 => 0xffff, 2 => 2 + (rnd() % 31) as u16, 3 => 0x8000, _ => 2 + (rnd() % 257) as u16 };
+        let mut syms = Vec::with_capacity(len + 8);
+        while syms.len() < len {
+            let burst = match mode { 0 => (rnd() % 23) as usize, 1 => 18 + (rnd() % 6) as usize, 2 => 0, _ => (rnd() % 4) as usize };
+            for _ in 0..burst { syms.push(if mode == 1 { 1 } else { (rnd() % 2) as u16 }); }
+            let lit = match rnd() % 9 { 0 => 2 + (rnd() % 15) as u16, 1 => 17 + (rnd() % 16) as u16, 2 => 32 + (rnd() % 3) as u16,
+                3 => 33 + (rnd() % 223) as u16, 4 => 255 + (rnd() % 4) as u16, 5 => 0x7fff + (rnd() % 3) as u16, 6 => 0xfffe + (rnd() % 2) as u16,
+                7 => 2, _ => 2 + (rnd() % 300) as u16 };
+            syms.push(lit);
+        }
+        syms.truncate(len);
+        if rnd() % 4 != 0 && len > 0 { let p = (rnd() as usize) % len; syms[p] = eob; }
+        let max_len = match rnd() % 6 { 0 => (rnd() % 50) as usize, 1 => (rnd() % 1000) as usize, 2 => 900_000, 3 => (1usize << 31) - 1, 4 => 1 << 31, _ => (rnd() % 5_000_000) as usize };
+        let mut s2u = [0u8; 256];
+        let dup = rnd() % 8 == 0;
+        let (mul, add) = ((rnd() as u8) | 1, rnd() as u8);
+        for (i, x) in s2u.iter_mut().enumerate() { *x = if dup { (rnd() % 7) as u8 } else { (i as u8).wrapping_mul(mul).wrapping_add(add) }; }
+        let b = Symbols { syms, eob, seq_to_unseq: s2u, orig_ptr: 0, crc: 0, max_len };
+        let want = front_scalar(&b, &mut rs);
+        // also exercise the AVX2 kernel directly, to count where it does not fall back
+        let mut probe = Runs::default();
+        if unsafe { front_avx2(&b, &b.syms, &mut probe) }.is_some() { avx_some += 1; }
+        let got = front(&b, &mut r, k);
+        assert_eq!(got, want, "it {it}: result (len {len}, eob {eob}, max_len {max_len}, dup {dup})");
+        if let Ok(n) = want {
+            ok_cases += 1;
+            assert_eq!(r.runs, rs.runs, "it {it}: runs");
+            assert_eq!(r.counts, rs.counts, "it {it}: counts");
+            assert_eq!(r.per, rs.per, "it {it}: per");
+            assert_eq!((r.trusted, r.len), (rs.trusted, rs.len), "it {it}: trusted/len");
+            if probe.trusted { assert_eq!(probe.runs, rs.runs, "it {it}: direct runs"); assert_eq!(probe.counts, rs.counts); }
+            if n <= 200_000 {
+                let (mut a, mut c) = (bwt::TVector::new(), bwt::TVector::new());
+                assert_eq!(fill(&r, n, &mut a, k), fill(&rs, n, &mut c, Kernels::scalar()), "it {it}: fill ok");
+                assert_eq!(a.0, c.0, "it {it}: TT");
+            }
+        } else { err_cases += 1; assert!(!r.trusted); }
+    }
+    eprintln!("b5 fuzz: {iters} iters, {ok_cases} ok, {err_cases} err, {avx_some} served by front_avx2");
+    assert!(avx_some > iters / 10 || cfg!(miri));
+}
+
+/// B5 review (local only): the largest blocks `front_avx2` accepts, and the first it refuses
+#[test]
+fn b5_review_front_avx2_max_len() {
+    let k = Kernels::detect();
+    for &m in &[(1usize << 24) - 1, 1 << 24, (1 << 24) - 64, (1 << 24) - 65] {
+        let mut st = 12345u64 ^ m as u64;
+        let mut syms: Vec<u16> = (0..m).map(|_| { st ^= st << 13; st ^= st >> 7; st ^= st << 17;
+            match st % 4 { 0 => 0, 1 => 1, _ => 2 + (st >> 8) as u16 % 300 } }).collect();
+        // break any run of 22 run symbols
+        let mut z = 0; for s in syms.iter_mut() { if *s < 2 { z += 1; if z == 20 { *s = 5; z = 0; } } else { z = 0; } }
+        *syms.last_mut().unwrap() = 400;
+        let b = Symbols { syms, eob: 400, seq_to_unseq: std::array::from_fn(|i| i as u8), orig_ptr: 0, crc: 0, max_len: (1 << 31) - 1 };
+        let (mut r, mut rs) = (Runs::default(), Runs::default());
+        let mut probe = Runs::default();
+        let direct = unsafe { front_avx2(&b, &b.syms, &mut probe) };
+        assert_eq!(direct.is_some(), m < 1 << 24, "m {m}");
+        let want = front_scalar(&b, &mut rs);
+        assert_eq!(front(&b, &mut r, k), want);
+        if want.is_ok() { assert_eq!(r.runs, rs.runs); assert_eq!(r.counts, rs.counts); assert_eq!(r.trusted, rs.trusted); }
+        eprintln!("m {m}: direct {direct:?}, scalar {want:?}");
+    }
+}
```
