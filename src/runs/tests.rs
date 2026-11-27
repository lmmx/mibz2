//! The records' path against the column's: `front` (both kernels) against
//! `mtf::mtf_scalar`, results and errors; the records against the column's runs; `fill`
//! (both kernels) against the column's LF table, word for word; the backward walks (plain,
//! and the room walk at several lane and segment counts) against `bwt::walk` on the
//! column's T-vector; every broken tiling refused; and `decode_block` by records against
//! the column's kernels. For a team's fill: the cuts and the shares between them (both
//! kernels, any number of shares) against `fill`'s table, each share writing its own rows
//! alone; the cuts refused exactly where `fill` refuses; and cuts that don't tile refused.

use super::*;
use crate::bwt::{self, Cooperative, Plain, Segmented};
use crate::{decode_block, mtf, Scratch};

/// The kernels to check: the scalar ones, and this CPU's where it has more
fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    v
}

/// A column's runs cut at CAP, and each byte's count of them: the plainest scan
fn runs_of_column(ll: &[u8]) -> (Vec<u16>, [u32; 256]) {
    let (mut runs, mut per) = (Vec::new(), [0u32; 256]);
    let mut i = 0;
    while i < ll.len() {
        let c = ll[i];
        let mut j = i;
        while j < ll.len() && ll[j] == c {
            j += 1;
        }
        let mut left = j - i;
        while left > 0 {
            let len = left.min(CAP as usize);
            runs.push((c as u16) << 8 | len as u16);
            per[c as usize] += 1;
            left -= len;
        }
        i = j;
    }
    (runs, per)
}

/// xorshift64*
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 11) % n
    }
}

/// The run symbols (RUNA 0, RUNB 1) that code a run of `len` rows (bijective base 2)
fn run_syms(mut len: u32, out: &mut Vec<u16>) {
    while len > 0 {
        if len & 1 == 1 {
            out.push(0);
            len = (len - 1) / 2;
        } else {
            out.push(1);
            len = (len - 2) / 2;
        }
    }
}

/// A made-up block (the Pathfinder's generator, `examples/run_front.rs`, with two shapes
/// added): `in_use` bytes, symbols that are run symbols with probability `run_pct`%, literals
/// near the front or anywhere (indices past 31 to the list's end), runs coded to exact
/// adversarial lengths around the record's cap (126-128, 254-256, ...) and long ones, run
/// symbols 21 and 22 long (`RunTooLong`'s edge), and an end-of-block symbol at the end, in
/// the middle as well, or nowhere
fn synthetic(rng: &mut Rng, syms_max: u64) -> Symbols {
    let in_use = match rng.below(4) {
        0 => 1 + rng.below(4) as usize,
        1 => 256,
        _ => 1 + rng.below(256) as usize,
    };
    let mut bytes: [u8; 256] = std::array::from_fn(|i| i as u8);
    for i in (1..256).rev() {
        bytes.swap(i, rng.below(i as u64 + 1) as usize);
    }
    let mut seq_to_unseq = [0u8; 256];
    seq_to_unseq[..in_use].copy_from_slice(&bytes[..in_use]);
    let eob = in_use as u16 + 1;
    let run_pct = [5, 30, 60, 90, 97][rng.below(5) as usize];
    let wide = rng.below(3) == 0;
    let shaped = rng.below(3) == 0;
    let len = match rng.below(4) {
        0 => rng.below(40) as usize,
        _ => rng.below(syms_max) as usize,
    };
    let mut syms = Vec::with_capacity(len + 64);
    while syms.len() < len {
        if shaped && rng.below(4) == 0 {
            // a run of an exact length: about the cap, a multiple of it, or long
            let l = match rng.below(100) {
                0..=24 => 125 + rng.below(5) as u32,
                25..=49 => 253 + rng.below(5) as u32,
                50..=74 => CAP * (1 + rng.below(8) as u32) + rng.below(3) as u32,
                75..=97 => 1 + rng.below(2000) as u32,
                // (Miri: runs of at most a few thousand rows, or it takes hours)
                _ if cfg!(miri) => 1 + rng.below(200) as u32,
                98 => (1 << 20) + rng.below(1 << 20) as u32,
                // 21 or 22 run symbols: 2^21 - 1 .. 2^23 - 2 rows
                _ => (1 << 21) - 1 + rng.below(1 << 22) as u32,
            };
            run_syms(l, &mut syms);
        } else if in_use == 1 || rng.below(100) < run_pct {
            syms.push(rng.below(2) as u16);
        } else {
            // an index in 1..in_use: any, or mostly the list's first few
            let nn = if wide || rng.below(16) == 0 {
                1 + rng.below(in_use as u64 - 1)
            } else {
                1 + rng.below((in_use as u64 - 1).min(6))
            };
            syms.push(nn as u16 + 1);
        }
    }
    match rng.below(8) {
        0 => {}
        1 if !syms.is_empty() => {
            let at = rng.below(syms.len() as u64) as usize;
            syms[at] = eob;
            syms.push(eob);
        }
        _ => syms.push(eob),
    }
    Symbols {
        crc: 0,
        orig_ptr: rng.next() as u32 & 0xff_ffff,
        seq_to_unseq,
        syms,
        eob,
        max_len: 1 << 23,
    }
}

/// What the checks have seen
#[derive(Default, Debug)]
struct Seen {
    ok: usize,
    too_long: usize,
    run_too_long: usize,
    rows: usize,
    records: usize,
    /// records cut from a run over CAP, and literals past the list's first 32
    cut: usize,
    deep: usize,
    tables: usize,
    walks: usize,
}

/// The column's LF table, by its definition: `L[i] | LF(i) << 8`, LF(i) = C(L[i]) + the
/// copies of L[i] before row i
fn lf_of(ll: &[u8]) -> Vec<u32> {
    let counts = crate::run_fed::counts_of(ll);
    let mut next = [0u32; 256];
    let mut cum = 0;
    for c in 0..256 {
        next[c] = cum;
        cum += counts[c];
    }
    ll.iter()
        .map(|&c| {
            let e = c as u32 | next[c as usize] << 8;
            next[c as usize] += 1;
            e
        })
        .collect()
}

/// The backward walks of `p` from `orig` against `bwt::walk` of the column's T-vector
fn check_walks(p: &[u32], tt: &bwt::TVector, orig: u32, seg: &mut bwt::SegScratch, what: &str) {
    let mut want = Vec::new();
    bwt::walk(tt, orig, &mut want);
    let mut got = vec![9u8; 3];
    bwt::walk_back(&bwt::TVector(p.to_vec()), orig, &mut got);
    assert!(got == want, "{what}: the plain backward walk");
    for (lanes, segments) in [(1, 2), (4, 8), (32, 64)] {
        let mut t = bwt::TVector(p.to_vec());
        let mut got = vec![9u8; 3];
        match lanes {
            1 => bwt::walk_back_room_with::<1>(&mut t, orig, &mut got, seg, segments),
            4 => bwt::walk_back_room_with::<4>(&mut t, orig, &mut got, seg, segments),
            _ => bwt::walk_back_room_with::<32>(&mut t, orig, &mut got, seg, segments),
        }
        assert!(got == want, "{what}: the backward room walk, {lanes} lanes");
        // the starts' flags cleared again: the table as it was
        assert!(t.0 == p, "{what}: the table after the room walk");
    }
}

/// One block's symbols through both fronts and, where it decodes, both fills and the
/// backward walks, everything compared with the column's path
fn check_block(b: &Symbols, r: &mut Runs, seen: &mut Seen, what: &str) {
    let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
    let want = mtf::mtf_scalar(b, &mut ll, &mut counts);
    match want {
        Ok(_) => seen.ok += 1,
        Err(ErrorKind::BlockTooLong) => seen.too_long += 1,
        Err(ErrorKind::RunTooLong) => seen.run_too_long += 1,
        Err(ref e) => panic!("{what}: mtf_scalar's {e:?}"),
    }
    let (runs, per) = match want {
        Ok(_) => runs_of_column(&ll),
        Err(_) => (Vec::new(), [0; 256]),
    };
    for k in kernels() {
        let got = front(b, r, k);
        assert!(got == want, "{what}, {k:?}: {got:?}, mtf {want:?}");
        let Ok(n) = want else { continue };
        assert!(r.runs == runs, "{what}, {k:?}: the records");
        assert!(r.counts == counts, "{what}, {k:?}: the row counts");
        assert!(r.per == per, "{what}, {k:?}: the record counts");
        let want_p = lf_of(&ll);
        for kf in kernels() {
            let mut tt = bwt::TVector(vec![7; 5]);
            assert!(
                fill(r, n, &mut tt, kf),
                "{what}, {k:?}, fill {kf:?}: refused"
            );
            assert!(tt.0 == want_p, "{what}, {k:?}, fill {kf:?}: the table");
        }
        seen.tables += 1;
    }
    if let Ok(n) = want {
        // the fill by shares, from the records the last front left (every front's are the
        // same, above): one block in four
        if seen.ok % 4 == 1 {
            check_shares(r, n, &lf_of(&ll), what);
        }
        // the walks, once a block (they don't depend on the front's kernel)
        if n > 0 && (cfg!(miri) || n < 1 << 16) {
            let mut tt = bwt::TVector::new();
            bwt::build(&ll, &counts, &mut tt);
            let orig = b.orig_ptr % n as u32;
            check_walks(
                &lf_of(&ll),
                &tt,
                orig,
                &mut bwt::SegScratch::default(),
                what,
            );
            seen.walks += 1;
        }
        seen.rows += n;
        seen.records += runs.len();
        seen.cut += runs.windows(2).filter(|w| w[0] >> 8 == w[1] >> 8).count();
        seen.deep += b.syms.iter().filter(|&&s| s > 32 && s != b.eob).count();
    }
}

/// A slot no share wrote (no entry: a link is below 2^24)
const GUARD: u32 = u32::MAX;

/// The fill by shares against `fill`'s table `want`: the cuts made and tiling; the table
/// filled by 1 to 13 shares of whole pieces, by each kernel; and each piece filled alone
/// into a guarded table, writing its own rows and no other slot
fn check_shares(r: &Runs, n: usize, want: &[u32], what: &str) {
    let mut cv = vec![];
    assert!(cuts(r, n, &mut cv), "{what}: the cuts refused");
    assert_eq!(cv.len(), PIECES + 1, "{what}: the cuts");
    assert!(cuts_tile(r, n, &cv), "{what}: the cuts don't tile");
    for k in kernels() {
        let counts: &[usize] = if cfg!(miri) {
            &[1, 3, 13]
        } else {
            &[1, 2, 3, 4, 5, 12, 13]
        };
        for &shares in counts {
            let mut t = vec![GUARD; n + 9];
            for j in 0..shares {
                let (from, to) = (&cv[PIECES * j / shares], &cv[PIECES * (j + 1) / shares]);
                // SAFETY: the table holds n slots and more, and nothing else touches it
                let ok = unsafe { fill_share(r, from, to, t.as_mut_ptr(), k) };
                assert!(ok, "{what}, {k:?}: share {j} of {shares} failed its check");
            }
            assert!(
                t[..n] == *want,
                "{what}, {k:?}: the table by {shares} shares"
            );
            assert!(
                t[n..] == [GUARD; 9],
                "{what}, {k:?}: a store past the block"
            );
        }
        for (j, w) in cv.windows(2).enumerate() {
            let mut t = vec![GUARD; n + 9];
            // SAFETY: as above
            assert!(unsafe { fill_share(r, &w[0], &w[1], t.as_mut_ptr(), k) });
            let (lo, hi) = (w[0].row as usize, w[1].row as usize);
            assert!(t[lo..hi] == want[lo..hi], "{what}, {k:?}: piece {j}'s rows");
            assert!(
                t[..lo].iter().chain(&t[hi..]).all(|&x| x == GUARD),
                "{what}, {k:?}: piece {j} wrote outside its rows {lo}..{hi}"
            );
        }
    }
}

/// The equivalence on made-up blocks: `front`'s result (length or error) is
/// `mtf_scalar`'s, its records the column's runs cut at 127, its counts the column's, and
/// the table `bwt::build`'s, by each kernel, on every block and at lengths around its own
#[test]
fn miri_runs_synthetic_blocks() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut r = Runs::default();
    let mut seen = Seen::default();
    let cases = if cfg!(miri) { 6 } else { 12_000 };
    let syms_max = if cfg!(miri) { 60 } else { 3000 };
    for i in 0..cases {
        let mut b = synthetic(&mut rng, syms_max);
        check_block(&b, &mut r, &mut seen, &format!("synthetic {i}"));
        // and at lengths around its own, and one nothing reaches
        let mut lens = vec![rng.below(1 << 12) as usize];
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        if let Ok(n) = mtf::mtf_scalar(&b, &mut ll, &mut counts) {
            lens.extend([n, n + 1]);
            lens.extend(n.checked_sub(1));
        }
        for max in lens {
            b.max_len = max;
            check_block(&b, &mut r, &mut seen, &format!("synthetic {i} at {max}"));
        }
    }
    eprintln!("{seen:?}");
    if !cfg!(miri) {
        // every shape was reached: blocks, both errors, cut runs, deep literals
        assert!(seen.ok > 1000 && seen.too_long > 100 && seen.run_too_long > 100);
        assert!(seen.cut > 1000 && seen.deep > 1000, "{seen:?}");
    }
}

/// Each case of the AVX2 front's hand-off and record-writing, on a block built to reach it:
/// a run of exactly 126, 127, 128, 254 and 255 rows, between literals and at the block's
/// start and end; literals of indices 31, 32, 33 and 255; 21 and 22 run symbols together
#[test]
fn miri_runs_front_edges() {
    let mut s2u = [0u8; 256];
    for (i, x) in s2u.iter_mut().enumerate() {
        *x = (255 - i) as u8;
    }
    let mut r = Runs::default();
    let mut seen = Seen::default();
    let lens = [1, 2, 126, 127, 128, 254, 255, 256, 381, 1000];
    for (i, &first) in lens.iter().enumerate() {
        let mut syms = Vec::new();
        run_syms(first, &mut syms);
        for (j, &l) in lens.iter().enumerate() {
            let idx = [1u16, 31, 32, 33, 200, 255][(i + j) % 6];
            syms.push(idx + 1);
            run_syms(l - 1, &mut syms);
        }
        syms.push(257);
        let b = Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: s2u,
            syms,
            eob: 257,
            max_len: 900_000,
        };
        check_block(&b, &mut r, &mut seen, &format!("edges {i}"));
    }
    // 21 run symbols (2^21 - 1 rows and more) and 22 (RunTooLong), at the start and later
    // (21 run symbols are 2^21 - 1 rows at least: too many for Miri, which checks 22's error)
    let ones = if cfg!(miri) {
        &[][..]
    } else {
        &[(21, false), (21, true)][..]
    };
    for &(count, later) in ones.iter().chain(&[(22, false), (22, true)]) {
        let mut syms = Vec::new();
        if later {
            syms.extend([5, 0, 7]);
        }
        syms.extend(std::iter::repeat(0).take(count));
        syms.extend([3, 257]);
        let b = Symbols {
            crc: 0,
            orig_ptr: 0,
            seq_to_unseq: s2u,
            syms,
            eob: 257,
            max_len: 1 << 23,
        };
        check_block(&b, &mut r, &mut seen, &format!("{count} run symbols"));
    }
    assert!(
        seen.run_too_long == 2 && seen.ok == lens.len() + ones.len(),
        "{seen:?}"
    );
}

/// The column's own records filled by each kernel, for a short column: the table, or None
/// if every kernel refused (and left the table empty); kernels that disagree fail
fn table_of(ll: &[u8], r: &mut Runs, brk: impl Fn(&mut Runs) -> usize) -> Option<Vec<u32>> {
    let mut got = Vec::new();
    for k in kernels() {
        let (runs, per) = runs_of_column(ll);
        r.runs = runs;
        r.per = per;
        r.counts = crate::run_fed::counts_of(ll);
        let n = brk(r);
        let mut tt = bwt::TVector(vec![3; 4]);
        let ok = fill(r, n, &mut tt, k);
        assert!(ok || tt.0.is_empty(), "{k:?}: a refused table kept");
        // the cuts are refused exactly where the fill is, and then left empty
        let mut cv = vec![];
        assert_eq!(
            cuts(r, n, &mut cv),
            ok,
            "{k:?}: the cuts and the fill disagree"
        );
        assert_eq!(cv.len(), if ok { PIECES + 1 } else { 0 });
        got.push(ok.then_some(tt.0));
    }
    assert!(got.windows(2).all(|w| w[0] == w[1]), "the kernels disagree");
    got.pop().unwrap()
}

/// Every tiling broken by one record or count is refused by both kernels, the table left
/// empty; the empty block's table is empty
#[test]
fn miri_runs_broken_tilings_refused() {
    let mut r = Runs::default();
    assert_eq!(table_of(&[], &mut r, |_| 0), Some(vec![]));
    let ll: Vec<u8> = b"aaabbbbbbcab"
        .iter()
        .chain(std::iter::repeat(&b'z').take(300))
        .chain(b"qa")
        .copied()
        .collect();
    let n = ll.len();
    assert_eq!(table_of(&ll, &mut r, |_| n), Some(lf_of(&ll)));
    // a record a row longer or shorter, of length 0, of another byte; a count off by one,
    // or moved to another byte; the wrong n either way; a record dropped or added (of
    // length 0, or 1)
    type Break<'a> = Box<dyn Fn(&mut Runs) -> usize + 'a>;
    let breaks: Vec<Break> = vec![
        Box::new(|r| {
            r.runs[1] += 1;
            n
        }),
        Box::new(|r| {
            r.runs[1] -= 1;
            n
        }),
        Box::new(|r| {
            r.runs[2] &= 0xff00;
            n
        }),
        Box::new(|r| {
            r.runs[2] += 1 << 8;
            n
        }),
        Box::new(|r| {
            r.counts[b'a' as usize] += 1;
            n
        }),
        Box::new(|r| {
            r.counts[b'a' as usize] -= 1;
            r.counts[b'b' as usize] += 1;
            n
        }),
        Box::new(|_| n + 1),
        Box::new(|_| n - 1),
        Box::new(|r| {
            r.runs.pop();
            n
        }),
        Box::new(|r| {
            r.runs.push((b'a' as u16) << 8);
            n
        }),
        Box::new(|r| {
            r.runs.push((b'a' as u16) << 8 | 1);
            n
        }),
    ];
    for (i, brk) in breaks.iter().enumerate() {
        assert!(table_of(&ll, &mut r, brk).is_none(), "break {i} accepted");
    }
}

/// `decode_block` by records (`Segmented`, which reads runs) gives what the column's kernel
/// (`Plain`, which doesn't) gives, on blocks of few symbols and of many, errors included
#[test]
fn miri_runs_decode_block_equals_the_column() {
    let mut rng = Rng(0x0123_4567_89ab_cdef);
    let (mut sr, mut sp) = (Scratch::<Segmented>::default(), Scratch::<Plain>::default());
    let cases = if cfg!(miri) { 12 } else { 3000 };
    let mut by_runs = 0;
    for i in 0..cases {
        let mut b = synthetic(&mut rng, if cfg!(miri) { 120 } else { 3000 });
        // a bound nothing reaches, and the selector's edge on either side
        b.max_len = match rng.below(3) {
            0 => 1 << 20,
            1 => 2 * b.syms.len(),
            _ => (2 * b.syms.len()).saturating_sub(1),
        };
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        if let Ok(n) = mtf::mtf_scalar(&b, &mut ll, &mut counts) {
            if n > 0 {
                b.orig_ptr = rng.below(n as u64 + 1) as u32;
                // the CRC of what the column decodes to, so the block checks
                let mut out = Vec::new();
                let mut tt = bwt::TVector::new();
                bwt::build(&ll, &counts, &mut tt);
                if (b.orig_ptr as usize) < n {
                    bwt::walk(&tt, b.orig_ptr, &mut out);
                    let mut text = Vec::new();
                    if crate::rle::unrle(&out, &mut text, Kernels::scalar()).is_ok() {
                        b.crc = crate::crc::crc32(&text, Kernels::scalar());
                    }
                }
            }
        }
        by_runs += takes_runs(&b) as usize;
        for k in kernels() {
            let (mut a, mut c) = (vec![1u8], vec![1u8]);
            let ra = decode_block(&b, k, &mut sr, &mut a);
            let rc = decode_block(&b, k, &mut sp, &mut c);
            assert!(ra == rc, "case {i}, {k:?}: {ra:?} against {rc:?}");
            assert!(a == c, "case {i}, {k:?}: the bytes");
        }
    }
    assert!(by_runs > cases / 4, "{by_runs} of {cases} by records");
}

/// Built with AVX2, BMI1, BMI2, LZCNT and POPCNT enabled (as Miri's AVX2 pass is), the
/// tests above check the AVX2 front and fill, not only the scalar ones
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "bmi1",
    target_feature = "bmi2",
    target_feature = "lzcnt",
    target_feature = "popcnt"
))]
#[test]
fn miri_runs_avx2_kernels_selected() {
    let k = Kernels::detect();
    assert!(k.avx2 && front_fast(k));
    assert_eq!(kernels().len(), 2);
}

// ---- Which path, and which kernel, runs: tests added from the mutation testing (§3.2 of
// docs/journal/2026-11-10-run-front-production.md). Each choice below gives the same bytes
// either way, so the equivalence tests above can't see a wrong one; these see it.

/// A record the front never writes (its length is over CAP): left in place, it shows that
/// `front` didn't run
const UNTOUCHED: u16 = 0xffff;

/// `decode_block` takes the records' path exactly where it should: with `Segmented` and
/// `Cooperative` (which read runs) on a block `takes_runs` accepts, and never with `Plain`
/// or on any other block
#[test]
fn runs_path_taken_exactly_where_selected() {
    let mut rng = Rng(0x5eed_0f9a_7e57_0001);
    let (mut sr, mut sp) = (Scratch::<Segmented>::default(), Scratch::<Plain>::default());
    let mut sc = Scratch::<Cooperative<2>>::default();
    let (mut by_runs, mut by_column) = (0, 0);
    for i in 0..400 {
        let mut b = synthetic(&mut rng, 3000);
        b.max_len = match rng.below(3) {
            0 => 1 << 20,
            1 => (4 * b.syms.len()).div_ceil(3),
            _ => (4 * b.syms.len() / 3).saturating_sub(1),
        };
        let want = takes_runs(&b);
        for k in kernels() {
            sr.runs.runs = vec![UNTOUCHED];
            sp.runs.runs = vec![UNTOUCHED];
            sc.runs.runs = vec![UNTOUCHED];
            let mut out = Vec::new();
            let _ = decode_block(&b, k, &mut sr, &mut out);
            let _ = decode_block(&b, k, &mut sp, &mut out);
            let _ = decode_block(&b, k, &mut sc, &mut out);
            let took = sr.runs.runs != [UNTOUCHED];
            assert_eq!(took, want, "case {i}, {k:?}: Segmented by records");
            let took = sc.runs.runs != [UNTOUCHED];
            assert_eq!(took, want, "case {i}, {k:?}: Cooperative by records");
            assert_eq!(
                sp.runs.runs,
                [UNTOUCHED],
                "case {i}, {k:?}: Plain by records"
            );
        }
        if want {
            by_runs += 1;
        } else {
            by_column += 1;
        }
    }
    assert!(
        by_runs > 50 && by_column > 50,
        "{by_runs} by records, {by_column} not"
    );
}

/// The selector's bounds: below 2^24 rows, at most 3/4 symbols per row of the bound
/// (`0.50`, `0.74`, `0.75` accepted; `0.76` rejected), and `RUN_SHARE` still rejects
/// high-entropy blocks within the widened band
#[test]
fn runs_takes_runs_bounds() {
    let make = |max_len: usize, syms: Vec<u16>| Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq: [0; 256],
        syms,
        eob: 2,
        max_len,
    };
    let mut b = make((1 << 24) - 1, vec![0; 10]);
    assert!(takes_runs(&b));
    b.max_len = 1 << 24;
    assert!(!takes_runs(&b), "a bound of 2^24 rows");

    // Exact boundary ratios: 0.50, 0.74, 0.75 accepted; 0.76 rejected
    assert!(takes_runs(&make(100, vec![0; 50])), "0.50 accepted");
    assert!(takes_runs(&make(100, vec![0; 74])), "0.74 accepted");
    assert!(takes_runs(&make(100, vec![0; 75])), "0.75 accepted");
    assert!(!takes_runs(&make(100, vec![0; 76])), "0.76 rejected");

    // Non-divisible edge: 4 * 10 = 40 <= 3 * 14 = 42, while 4 * 10 = 40 > 3 * 13 = 39
    assert!(takes_runs(&make(14, vec![0; 10])));
    assert!(!takes_runs(&make(13, vec![0; 10])));

    // High-entropy and sparse-run blocks (< 1/8 runs) within 0.50 < len/max_len <= 0.75 stay on Column
    assert!(
        !takes_runs(&make(100, vec![2; 64])),
        "literal-only block at 0.64 rejected"
    );
    let mut sparse_runs = vec![2; 64];
    sparse_runs[0] = 0;
    sparse_runs[1] = 0;
    sparse_runs[2] = 0; // 3/64 = 4.7% (like 26-letter random noise: > 1/32, < 1/8)
    assert!(
        !takes_runs(&make(100, sparse_runs)),
        "sparse-run noise (< 1/8) in widened band stays on Column"
    );
    let mut dense_runs = vec![2; 64];
    for slot in &mut dense_runs[..8] {
        *slot = 0; // 8/64 = 12.5% = 1/8
    }
    assert!(
        takes_runs(&make(100, dense_runs)),
        "12.5% run share in widened band accepted"
    );
}

/// Exact decode parity across `0.50 < len/max_len <= 0.75`: `Segmented` takes the run-records
/// path and produces byte-identical output and errors to `Plain` (column path)
#[test]
fn runs_widened_band_decode_parity() {
    let mut rng = Rng(0x75_b4ad_1080_0001);
    let (mut sr, mut sp) = (Scratch::<Segmented>::default(), Scratch::<Plain>::default());
    let mut checked = 0usize;
    for i in 0..80 {
        let mut b = synthetic(&mut rng, 1500);
        if b.syms.len() < 40 {
            continue;
        }
        // Place len / max_len squarely in (0.50, 0.75] (ratios ~0.55..0.75)
        let num = 4 * b.syms.len();
        b.max_len = match i % 4 {
            0 => num.div_ceil(3),           // ~0.75
            1 => (100 * b.syms.len()) / 74, // ~0.74
            2 => (100 * b.syms.len()) / 62, // ~0.62
            _ => (100 * b.syms.len()) / 52, // ~0.52
        };
        if !takes_runs(&b) {
            continue;
        }
        assert!(2 * b.syms.len() > b.max_len, "must lie above old 0.50 cut");
        checked += 1;
        for k in kernels() {
            let (mut a, mut c) = (Vec::new(), Vec::new());
            let ra = decode_block(&b, k, &mut sr, &mut a);
            let rc = decode_block(&b, k, &mut sp, &mut c);
            assert_eq!(ra, rc, "case {i}, {k:?}");
            assert_eq!(a, c, "case {i}, {k:?}: output mismatch");
        }
    }
    assert!(
        checked >= 20,
        "expected >= 20 widened-band blocks, got {checked}"
    );
}

/// A small block that decodes: "abc"'s indices, a run, and the end of block
fn small_block() -> Symbols {
    let mut seq_to_unseq = [0u8; 256];
    seq_to_unseq[..3].copy_from_slice(b"abc");
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq,
        syms: vec![2, 0, 1, 3, 0, 2, 4],
        eob: 4,
        max_len: 1000,
    }
}

/// `front` runs the AVX2 kernel exactly when the kernels and the CPU allow it (it alone
/// writes the bitmaps), and the scalar one otherwise; both give the same records
#[test]
fn runs_front_kernel_selected() {
    let b = small_block();
    let mut want = Runs::default();
    let n = front_scalar(&b, &mut want).unwrap();
    assert!(want.lit.is_empty() && want.one.is_empty());
    let mut r = Runs::default();
    assert_eq!(front(&b, &mut r, Kernels::scalar()), Ok(n));
    assert!(r.lit.is_empty(), "the scalar kernels took the AVX2 front");
    assert_eq!(
        (&r.runs, r.counts, r.per),
        (&want.runs, want.counts, want.per)
    );
    #[cfg(target_arch = "x86_64")]
    {
        let k = Kernels::detect();
        let fast = k.avx2
            && std::is_x86_feature_detected!("bmi2")
            && std::is_x86_feature_detected!("lzcnt")
            && std::is_x86_feature_detected!("popcnt");
        let mut r = Runs::default();
        assert_eq!(front(&b, &mut r, k), Ok(n));
        assert_eq!(!r.lit.is_empty(), fast, "{k:?}: the AVX2 front taken");
        assert_eq!(
            (&r.runs, r.counts, r.per),
            (&want.runs, want.counts, want.per)
        );
    }
}

/// The footprint ledger sees the records' buffers
#[test]
fn runs_buffers_listed() {
    let mut r = Runs::default();
    front(&small_block(), &mut r, Kernels::detect()).unwrap();
    let names: Vec<_> = r.buffers().iter().map(|b| b.name).collect();
    assert_eq!(names, ["runs.runs", "runs.lit", "runs.one"]);
    assert!(r.buffers()[0].cap >= r.runs.len() * 2 && !r.runs.is_empty());
}

/// `walk_back_room` segments from `MIN_SEGMENTED` rows and walks plainly below it (the room
/// walk records its block in the scratch; the plain walk leaves it), with `walk`'s bytes
/// either side
#[test]
fn runs_walk_back_room_segments_from_its_bound() {
    let mut rng = Rng(0x00c0_ffee_1234_5678);
    for n in [bwt::MIN_SEGMENTED - 1, bwt::MIN_SEGMENTED] {
        let ll: Vec<u8> = (0..n).map(|_| b"abcd"[rng.below(4) as usize]).collect();
        let mut tt = bwt::TVector::new();
        bwt::build(&ll, &crate::run_fed::counts_of(&ll), &mut tt);
        let orig = rng.below(n as u64) as u32;
        let mut want = Vec::new();
        bwt::walk(&tt, orig, &mut want);
        let mut seg = bwt::SegScratch::default();
        let mut got = Vec::new();
        bwt::walk_back_room(&mut bwt::TVector(lf_of(&ll)), orig, &mut got, &mut seg);
        assert!(got == want, "{n} rows: the bytes");
        let segmented = n >= bwt::MIN_SEGMENTED;
        assert_eq!(seg.n, if segmented { n } else { 0 }, "{n} rows: segmented");
    }
}

/// Cuts that don't tile the block are refused (`cuts_tile`), each broken one way; and a
/// share between cuts out of order, or past the records, writes nothing and fails
#[test]
fn miri_runs_cuts_that_dont_tile_refused() {
    let mut rng = Rng(0x7e57_c0de_0bad_cafe);
    let ll: Vec<u8> = (0..700)
        .flat_map(|_| {
            let c = b"abcd"[rng.below(4) as usize];
            std::iter::repeat(c).take(1 + rng.below(9) as usize)
        })
        .collect();
    let n = ll.len();
    let r = Runs::of_column(&ll);
    let mut cv = vec![];
    assert!(cuts(&r, n, &mut cv) && cuts_tile(&r, n, &cv));
    // (the pieces are not empty: the breaks below move real cuts)
    assert!(cv.windows(2).all(|w| w[0].row < w[1].row));
    type Break = fn(&mut Vec<Cut>);
    let breaks: [Break; 12] = [
        |c| c.clear(),
        |c| c.truncate(1),
        |c| c[0].rec = 1,
        |c| c[0].row = 1,
        |c| c[0].next[b'b' as usize] += 1,
        |c| c[PIECES].rec -= 1,
        |c| c[PIECES].row -= 1,
        |c| c[PIECES].row += 1,
        |c| c[5].rec = c[6].rec + 1,
        |c| c[5].row = c[6].row + 1,
        |c| c[5].next[b'c' as usize] = c[6].next[b'c' as usize] + 1,
        |c| {
            for x in &mut c[1..] {
                x.next[b'd' as usize] += 1;
            }
        },
    ];
    for (i, brk) in breaks.iter().enumerate() {
        let mut bad = cv.clone();
        brk(&mut bad);
        assert!(!cuts_tile(&r, n, &bad), "break {i} accepted");
    }
    // another block's length, either way; and counts that are not these records'
    assert!(!cuts_tile(&r, n + 1, &cv) && !cuts_tile(&r, n - 1, &cv));
    let mut other = Runs::of_column(&ll);
    other.counts[b'a' as usize] -= 1;
    other.counts[b'b' as usize] += 1;
    assert!(!cuts_tile(&other, n, &cv));
    // (a count more for the last byte: every bucket's start is the same, the sum is not)
    let mut more = Runs::of_column(&ll);
    more.counts[255] += 1;
    assert!(!cuts_tile(&more, n, &cv));
    for k in kernels() {
        let mut t = vec![GUARD; n + 9];
        // cuts out of order: by the row, by a byte's value, by the records
        let mut late = cv[4].clone();
        late.next[b'a' as usize] = cv[3].next[b'a' as usize] - 1;
        let mut short = cv[4].clone();
        short.rec = r.runs.len() + 1;
        for (from, to) in [(&cv[4], &cv[3]), (&cv[3], &late), (&cv[3], &short)] {
            // SAFETY: the table holds n slots and more, and nothing else touches it
            assert!(!unsafe { fill_share(&r, from, to, t.as_mut_ptr(), k) });
            assert!(
                t.iter().all(|&x| x == GUARD),
                "{k:?}: a refused share wrote"
            );
        }
        // rows out of order alone (the records and the values in order)
        let mut far = cv[3].clone();
        far.row = cv[4].row + 1;
        // SAFETY: as above
        assert!(!unsafe { fill_share(&r, &far, &cv[4], t.as_mut_ptr(), k) });
        assert!(
            t.iter().all(|&x| x == GUARD),
            "{k:?}: a refused share wrote"
        );
        // a share whose records are not its cuts': a piece's first record gone, and the
        // record before the piece added; a cut a row late (the rows end short of it), a cut
        // with a byte's value one more (the values end short of it), and a cut a row early
        // (below). Each fails, having written only inside its own rows
        let (mut late_start, mut early_start) = (cv[3].clone(), cv[3].clone());
        late_start.rec += 1;
        early_start.rec -= 1;
        let (mut row_more, mut value_more) = (cv[4].clone(), cv[4].clone());
        row_more.row += 1;
        value_more.next[b'a' as usize] += 1;
        // and a cut a row early, its values as they were: the share's last record would
        // pass it by a row, with every byte's values still inside the cut's
        let mut row_less = cv[4].clone();
        row_less.row -= 1;
        for (i, (from, to)) in [
            (&late_start, &cv[4]),
            (&early_start, &cv[4]),
            (&cv[3], &row_more),
            (&cv[3], &value_more),
            (&cv[3], &row_less),
        ]
        .into_iter()
        .enumerate()
        {
            let mut t = vec![GUARD; n + 9];
            // SAFETY: as above
            assert!(
                !unsafe { fill_share(&r, from, to, t.as_mut_ptr(), k) },
                "{k:?}: wrong share {i} passed"
            );
            let (lo, hi) = (from.row as usize, to.row as usize);
            assert!(
                t[..lo].iter().chain(&t[hi..]).all(|&x| x == GUARD),
                "{k:?}: wrong share {i} wrote outside its rows"
            );
        }
        // a record of another byte (the same rows): refused at the record, where its byte's
        // values would pass the cut's, not only by the end check, so the rows after it are
        // never written
        let col = [b"b".repeat(5), b"a".repeat(20), b"c".repeat(20)].concat();
        let mut whole = vec![];
        assert!(cuts(&Runs::of_column(&col), 45, &mut whole));
        let mut other = Runs::of_column(&col);
        other.runs[1] = (b'b' as u16) << 8 | 20;
        let mut t = vec![GUARD; 45 + 9];
        // SAFETY: as above
        assert!(!unsafe { fill_share(&other, &whole[0], &whole[PIECES], t.as_mut_ptr(), k) });
        assert!(
            t[8..] == [GUARD; 46],
            "{k:?}: filled on past a record over its bound"
        );
        // an empty record, 7 rows before the share's end (where an 8-slot store from it
        // would pass the end): refused, nothing written past the share's rows
        let mut gap = Runs::of_column(&[b"a".repeat(20), b"b".repeat(7)].concat());
        gap.runs.insert(1, (b'c' as u16) << 8);
        let mut ends = vec![];
        assert!(cuts(
            &Runs::of_column(&[b"a".repeat(20), b"b".repeat(7)].concat()),
            27,
            &mut ends
        ));
        let mut end = ends[PIECES].clone();
        end.rec = 3;
        let mut t = vec![GUARD; 27 + 9];
        // SAFETY: as above
        assert!(!unsafe { fill_share(&gap, &ends[0], &end, t.as_mut_ptr(), k) });
        assert!(
            t[27..] == [GUARD; 9],
            "{k:?}: an empty record's store past the block"
        );
    }
}

/// Every length of a record against every distance from its end to the share's: the 8-slot
/// stores never pass the share's rows (the slots after them stay as they were), and the
/// table is `fill`'s
#[test]
fn miri_runs_fill_share_stores_end_inside_its_rows() {
    // (under Miri, the lengths either side of a store's 8 slots and the distances either
    // side of its 7-slot margin)
    let (lens, afters): (Vec<usize>, Vec<usize>) = if cfg!(miri) {
        (vec![1, 8, 9, 17], vec![1, 6, 7, 8])
    } else {
        ((1..=19).collect(), (1..=9).collect())
    };
    for &len in &lens {
        for &after in &afters {
            let ll = [b"a".repeat(len), b"b".repeat(after)].concat();
            let r = Runs::of_column(&ll);
            let mut tt = bwt::TVector::new();
            assert!(fill(&r, ll.len(), &mut tt, Kernels::scalar()));
            check_shares(&r, ll.len(), &tt.0, &format!("{len} then {after}"));
        }
    }
}

/// The pieces are of about equal work: each within a record's of a twelfth of the block's
/// (a store for each 8 rows, and one for each record)
#[test]
fn runs_cuts_balance_the_work() {
    let mut rng = Rng(0xba1a_9ce0_0f00_d5ed);
    let ll: Vec<u8> = (0..40_000)
        .flat_map(|_| {
            let c = b"abcdefg"[rng.below(7) as usize];
            std::iter::repeat(c).take(1 + rng.below(40) as usize)
        })
        .collect();
    let (n, r) = (ll.len(), Runs::of_column(&ll));
    let mut cv = vec![];
    assert!(cuts(&r, n, &mut cv));
    let work = |a: &Cut, b: &Cut| (b.row - a.row) as u64 + 8 * (b.rec - a.rec) as u64;
    let total = work(&cv[0], &cv[PIECES]);
    assert_eq!(total, n as u64 + 8 * r.runs.len() as u64);
    for (j, w) in cv.windows(2).enumerate() {
        let got = work(&w[0], &w[1]);
        assert!(
            got.abs_diff(total / PIECES as u64) <= CAP as u64 + 8,
            "piece {j}: {got} of {total}"
        );
    }
}

/// The cuts' bounds: a block of 2^24 rows, or records, is refused (the sums are 32-bit)
#[test]
fn runs_cuts_bounds() {
    let mut cv = vec![];
    // 2^24 rows of one byte (the last, so that no bucket starts after its own), in records
    // of 127 rows and one of the rest
    let n = 1usize << 24;
    let z = 0xffusize;
    let mut r = Runs {
        runs: vec![(z as u16) << 8 | 127; n / 127],
        ..Runs::default()
    };
    r.runs.push((z as u16) << 8 | (n % 127) as u16);
    r.counts[z] = n as u32;
    assert!(!cuts(&r, n, &mut cv), "2^24 rows");
    // a row fewer: made, and tiling. The same cuts with the row back, whole in every other
    // way, don't tile 2^24 rows
    *r.runs.last_mut().unwrap() -= 1;
    r.counts[z] -= 1;
    assert!(cuts(&r, n - 1, &mut cv) && cuts_tile(&r, n - 1, &cv));
    r.runs.push((z as u16) << 8 | 1);
    r.counts[z] += 1;
    let mut big = cv.clone();
    big[PIECES].rec += 1;
    big[PIECES].row += 1;
    big[PIECES].next[z] += 1;
    assert!(!cuts_tile(&r, n, &big), "2^24 rows' cuts");
    // (whole: one row fewer, with its record and count gone, they tile)
    r.runs.pop();
    r.counts[z] -= 1;
    assert!(cuts_tile(&r, n - 1, &cv));
    // 2^24 records, of a row each, in a block of 2^24 - 1 rows: refused for their number
    // (and for their rows; one fewer is made)
    let mut r = Runs {
        runs: vec![(b'a' as u16) << 8 | 1; n],
        ..Runs::default()
    };
    r.counts[b'a' as usize] = n as u32 - 1;
    assert!(!cuts(&r, n - 1, &mut cv), "2^24 records");
    r.runs.pop();
    assert!(cuts(&r, n - 1, &mut cv));
    // records whose rows wrap 32 bits back to the block's 254: refused for their number,
    // where their sums alone would pass
    let wrap = (1usize << 32).div_ceil(255);
    assert_eq!((wrap * 255) as u32, 254);
    let mut r = Runs {
        runs: vec![(b'a' as u16) << 8 | 255; wrap],
        ..Runs::default()
    };
    r.counts[b'a' as usize] = 254;
    assert!(!cuts(&r, 254, &mut cv), "rows that wrap to the block's");
}

/// A block of fewer records than pieces, and the empty block: the cuts made, the pieces left
/// over empty at the end, the table `fill`'s
#[test]
fn miri_runs_cuts_of_short_blocks() {
    for ll in [&b""[..], b"a", b"aab", b"abcabcabca"] {
        let r = Runs::of_column(ll);
        let mut tt = bwt::TVector::new();
        assert!(fill(&r, ll.len(), &mut tt, Kernels::scalar()));
        check_shares(&r, ll.len(), &tt.0, &format!("{ll:?}"));
    }
}

/// `Runs::of_column` is `front`'s records for the column (the tests' other maker of them)
#[test]
fn miri_runs_of_column_is_the_scan() {
    let ll = [b"ab".repeat(40), vec![b'q'; 300], b"xyzzy".to_vec()].concat();
    let r = Runs::of_column(&ll);
    let (runs, per) = runs_of_column(&ll);
    assert!(r.runs == runs && r.per == per && r.counts == crate::run_fed::counts_of(&ll));
    assert!(!r.trusted);
}

/// `front` sets `trusted` only on valid completion (`n < 2^24`), `fill` via `fill_avx2_trusted`
/// matches `fill_scalar` and `lf_of` identically, wrong `n` is refused, and both `records_mut`
/// and a subsequent failing `front` clear `trusted`.
#[test]
fn miri_runs_trusted_fill_parity_and_invalidation() {
    let mut rng = Rng(0x7257_ed11_c0de_5eed);
    let cases = if cfg!(miri) { 8 } else { 200 };
    for i in 0..cases {
        let mut b = synthetic(&mut rng, if cfg!(miri) { 120 } else { 2000 });
        b.max_len = 1 << 20;
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        let Ok(want_n) = mtf::mtf_scalar(&b, &mut ll, &mut counts) else {
            continue;
        };
        let want_lf = lf_of(&ll);
        for k in kernels() {
            let mut r = Runs::default();
            assert!(!r.trusted);
            let n = front(&b, &mut r, k).expect("mtf_scalar succeeded");
            assert_eq!(n, want_n);
            assert!(
                r.trusted,
                "case {i}, {k:?}: front did not mark Runs trusted"
            );
            assert_eq!(r.len, n);

            let mut tt = bwt::TVector::new();
            assert!(fill(&r, n, &mut tt, k));
            assert_eq!(tt.0, want_lf, "case {i}, {k:?}: trusted fill mismatch");

            // Wrong `n` must be rejected and leave `tt` empty
            let mut bad_tt = bwt::TVector(vec![7; 3]);
            assert!(!fill(&r, n + 1, &mut bad_tt, k));
            assert!(bad_tt.0.is_empty());
            if n > 0 {
                let mut bad_tt2 = bwt::TVector(vec![7; 3]);
                assert!(!fill(&r, n - 1, &mut bad_tt2, k));
                assert!(bad_tt2.0.is_empty());
            }

            // Mutating records via `records_mut` must clear `trusted` so tampering is caught
            if !r.runs.is_empty() {
                r.records_mut()[0] &= 0xff00; // invalid zero-length record
                assert!(!r.trusted);
                let mut tampered_tt = bwt::TVector(vec![9; 2]);
                assert!(!fill(&r, n, &mut tampered_tt, k));
                assert!(tampered_tt.0.is_empty());
            }
        }
    }

    // A failing `front` after a trusted `front` must clear `trusted`
    let mut rng2 = Rng(0xbadf_00d5_1234_5678);
    let mut ok_block = synthetic(&mut rng2, 200);
    ok_block.max_len = 1 << 20;
    let mut r = Runs::default();
    let n = front(&ok_block, &mut r, Kernels::detect()).expect("valid block");
    assert!(r.trusted && r.len == n);
    let mut fail_block = ok_block;
    fail_block.max_len = 0;
    if n > 0 {
        assert!(front(&fail_block, &mut r, Kernels::detect()).is_err());
        assert!(!r.trusted);
    }
}

/// Research only (`mtf-fold`): the fold (src/runs/fold.rs and the open kernel's sink) against
/// the shipped path, block for block: the runs it sends are the records' maximal runs and its
/// record count theirs; it gives up only where the shipped front errs or its AVX2 kernel leaves
/// the block to the scalar one (or the marked row is past the block); and where it finishes,
/// the open kernel's bytes are the shipped path's, and it finishes exactly where the shipped
/// path takes the open kernel
#[cfg(feature = "mtf-fold")]
mod fold_tests {
    use super::*;
    use crate::bwt::{ForcedOpenCells, InverseBwt, OpenCellsOf};
    use crate::runs::fold::{self, Outcome, Sink};

    /// A sink that keeps what it is sent
    #[derive(Default)]
    struct Kept(Vec<(u8, u32)>);

    impl Sink for Kept {
        fn run(&mut self, c: u8, total: u32) {
            self.0.push((c, total));
        }
    }

    /// Adjacent runs of one byte merged
    fn merged(runs: impl IntoIterator<Item = (u8, u32)>) -> Vec<(u8, u32)> {
        let mut v: Vec<(u8, u32)> = Vec::new();
        for (c, w) in runs {
            match v.last_mut() {
                Some(last) if last.0 == c => last.1 += w,
                _ => v.push((c, w)),
            }
        }
        v
    }

    /// Whether the shipped AVX2 front gives the block up to the scalar one, for kernels `k`
    fn avx2_gives_up(b: &Symbols, k: Kernels) -> bool {
        #[cfg(target_arch = "x86_64")]
        if front_fast(k) {
            // SAFETY: `front_fast` holds only where the features it needs were detected
            return unsafe { front_avx2(b, &b.syms, &mut Runs::default()) }.is_none();
        }
        let _ = (b, k);
        false
    }

    /// The fold's front against the shipped front on one block
    fn check_front(b: &Symbols, r: &mut Runs, what: &str) -> bool {
        let mut folded = false;
        for k in kernels() {
            let mut kept = Kept::default();
            let got = fold::front(b, &mut Runs::default(), k, &mut kept, usize::MAX);
            let want = front(b, r, k);
            // and with the admission's refusal by literals: refused only where the shipped
            // records are refused by the dense line (or the shipped front errs), else the same
            let refused = fold::refused(b.max_len);
            let got_r = fold::front(b, &mut Runs::default(), k, &mut Kept::default(), refused);
            match (&got_r, &want) {
                (Err(Outcome::NotOpen), Ok(n)) => assert!(
                    25 * r.runs.len() >= 7 * n,
                    "{what}, {k:?}: refused by its literals, admissible by its records"
                ),
                (Err(Outcome::NotOpen), Err(_)) => {}
                _ => assert!(got_r == got, "{what}, {k:?}: {got_r:?} against {got:?}"),
            }
            if let Err(o) = got {
                assert!(o == Outcome::Bailed, "{what}, {k:?}: {o:?}");
            }
            match got.ok() {
                Some((n, recs)) => {
                    assert!(want == Ok(n), "{what}, {k:?}: {n} rows, shipped {want:?}");
                    assert!(recs == r.runs.len(), "{what}, {k:?}: the record count");
                    assert!(kept.0.iter().all(|&(_, w)| w > 0), "{what}: an empty run");
                    let records = r.runs.iter().map(|&x| ((x >> 8) as u8, (x & 0xff) as u32));
                    assert!(
                        merged(kept.0) == merged(records),
                        "{what}, {k:?}: the maximal runs"
                    );
                    folded = true;
                }
                None => assert!(
                    want.is_err() || avx2_gives_up(b, k),
                    "{what}, {k:?}: given up on a block the shipped front takes"
                ),
            }
        }
        folded
    }

    /// The fold through the open kernel against the shipped path on one block: the outcomes
    /// counted in `seen` (done, not open, given up)
    fn check_invert<const F: bool>(b: &Symbols, k: Kernels, seen: &mut [usize; 3], what: &str) {
        let (mut ship, mut cells) = (OpenCellsOf::<F>::default(), OpenCellsOf::<F>::default());
        let (mut r, mut scratch) = (Runs::default(), Runs::default());
        let mut got = vec![0x5a; 37];
        // (on a copy: the fold hands its symbols over once it admits the block)
        let mut bf = copy(b);
        let mut handed = None;
        let outcome = cells.fold(&mut bf, &mut scratch, &mut got, k, &mut |n, syms| {
            assert!(handed.is_none(), "{what}: handed over twice");
            handed = Some((n, syms));
        });
        match (outcome, handed) {
            (Outcome::Done(n), Some((m, syms))) => {
                assert!(
                    n == m && syms == b.syms,
                    "{what}, {k:?}: what was handed over"
                );
                assert!(bf.syms.is_empty(), "{what}, {k:?}: the symbols kept");
            }
            (Outcome::Done(_), None) => panic!("{what}, {k:?}: folded, nothing handed over"),
            (_, h) => assert!(
                h.is_none() && bf.syms == b.syms,
                "{what}, {k:?}: handed over, not folded"
            ),
        }
        let shipped = front(b, &mut r, k);
        let decodes = matches!(shipped, Ok(n) if (b.orig_ptr as usize) < n);
        match outcome {
            Outcome::Done(n) => {
                assert!(
                    shipped == Ok(n),
                    "{what}, {k:?}: {n} rows, shipped {shipped:?}"
                );
                let mut want = vec![0xa5; 11];
                assert!(ship.invert_runs(&r, n, b.orig_ptr, &mut want, k));
                assert!(
                    ship.opened == 1,
                    "{what}, {k:?}: shipped, not the open kernel"
                );
                assert!(got == want, "{what}, {k:?}: the bytes");
                seen[0] += 1;
            }
            Outcome::NotOpen => {
                // (refused by its literals before its rows are known, it may also be a block
                // the shipped front refuses: either way the shipped path decides)
                if decodes {
                    let n = shipped.unwrap();
                    let mut want = Vec::new();
                    assert!(ship.invert_runs(&r, n, b.orig_ptr, &mut want, k));
                    assert!(
                        ship.opened == 0,
                        "{what}, {k:?}: the shipped path opened it"
                    );
                }
                seen[1] += 1;
            }
            Outcome::Bailed => {
                assert!(
                    !decodes || avx2_gives_up(b, k),
                    "{what}, {k:?}: given up on a block the shipped path decodes"
                );
                seen[2] += 1;
            }
        }
    }

    /// A bound the open kernel takes: the block's own rows, a row short, or 900,000
    fn bound(b: &mut Symbols, rng: &mut Rng) {
        let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
        let n = mtf::mtf_scalar(b, &mut ll, &mut counts).ok();
        b.max_len = match (rng.below(4), n) {
            (0, Some(n)) if n <= 900_000 => n,
            (1, Some(n)) if (1..=900_000).contains(&n) => n - 1,
            // (Miri: a bound of a few thousand rows, as the fold sizes its buffer by it)
            _ if cfg!(miri) => 1 << 12,
            _ => 900_000,
        };
        if let Some(n) = n.filter(|&n| n > 0) {
            if rng.below(8) != 0 {
                b.orig_ptr = rng.below(n as u64) as u32;
            }
        }
    }

    #[test]
    fn miri_fold_front_is_the_records_maximal_runs() {
        let mut rng = Rng(0x51ed_270b_27a5_1ce1);
        let mut r = Runs::default();
        let cases = if cfg!(miri) { 8 } else { 6000 };
        let syms_max = if cfg!(miri) { 80 } else { 3000 };
        let mut folded = 0;
        for i in 0..cases {
            let mut b = synthetic(&mut rng, syms_max);
            folded += check_front(&b, &mut r, &format!("synthetic {i}")) as usize;
            bound(&mut b, &mut rng);
            check_front(&b, &mut r, &format!("synthetic {i} at {}", b.max_len));
        }
        assert!(folded > cases / 4, "{folded} of {cases} folded");
    }

    #[test]
    fn miri_fold_front_edges() {
        // runs about the cap, at the block's start, between literals and at its end; deep
        // literals; 21 and 22 run symbols together
        let mut r = Runs::default();
        let mut seq_to_unseq = [0u8; 256];
        for (i, s) in seq_to_unseq.iter_mut().enumerate() {
            *s = 255 - i as u8;
        }
        for len in [1, 126, 127, 128, 254, 255, 381, (1 << 21) - 1, 1 << 21] {
            for lit in [2u16, 32, 33, 34, 256] {
                let mut syms = Vec::new();
                run_syms(len, &mut syms);
                syms.push(lit);
                run_syms(len, &mut syms);
                syms.extend([3, 2, lit]);
                run_syms(len, &mut syms);
                syms.push(257);
                let b = Symbols {
                    crc: 0,
                    orig_ptr: 0,
                    seq_to_unseq,
                    syms,
                    eob: 257,
                    max_len: 900_000,
                };
                check_front(&b, &mut r, &format!("run {len}, literal {lit}"));
            }
        }
    }

    #[test]
    fn miri_fold_open_kernel_is_the_shipped_path() {
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        let cases = if cfg!(miri) { 6 } else { 2000 };
        let syms_max = if cfg!(miri) { 60 } else { 3000 };
        let (mut forced, mut prod) = ([0usize; 3], [0usize; 3]);
        for i in 0..cases {
            let mut b = synthetic(&mut rng, syms_max);
            bound(&mut b, &mut rng);
            for k in kernels() {
                let what = format!("synthetic {i}");
                check_invert::<true>(&b, k, &mut forced, &what);
                check_invert::<false>(&b, k, &mut prod, &what);
            }
        }
        eprintln!("forced {forced:?}, production {prod:?}");
        if !cfg!(miri) {
            assert!(forced[0] > cases / 4, "forced: {forced:?}");
            assert!(prod[0] > 0 && prod[1] > 0, "production: {prod:?}");
        }
    }

    /// Blocks made from columns of long runs (the open kernel's own kind), through
    /// production's admission: the fold finishes them all
    #[test]
    fn fold_open_kernel_on_columns_of_runs() {
        let mut rng = Rng(0x0dd_ba11_cafe_f00d);
        let mut done = [0usize; 3];
        for i in 0..60 {
            let n = 1 + rng.below(300_000) as usize;
            let mut ll = Vec::with_capacity(n);
            while ll.len() < n {
                let c = b"abcdefgh"[rng.below(8) as usize];
                let w = 1 + rng.below([4, 40, 400][i % 3]) as usize;
                ll.extend(std::iter::repeat_n(c, w.min(n - ll.len())));
            }
            let b = symbols_of(&ll, rng.below(n as u64) as u32);
            for k in kernels() {
                check_invert::<false>(&b, k, &mut done, &format!("column {i}"));
            }
        }
        assert!(done[2] == 0 && done[0] > 0, "{done:?}");
    }

    /// A column's MTF/RLE2 symbols (libbzip2's encoder's, its bytes in use in order)
    fn symbols_of(ll: &[u8], orig_ptr: u32) -> Symbols {
        let mut used: Vec<u8> = ll.to_vec();
        used.sort_unstable();
        used.dedup();
        let mut seq_to_unseq = [0u8; 256];
        seq_to_unseq[..used.len()].copy_from_slice(&used);
        let eob = used.len() as u16 + 1;
        let mut list = used.clone();
        let mut syms = Vec::new();
        let mut zeros = 0u32;
        for &c in ll {
            let j = list.iter().position(|&x| x == c).unwrap();
            if j == 0 {
                zeros += 1;
                continue;
            }
            run_syms(zeros, &mut syms);
            zeros = 0;
            list.remove(j);
            list.insert(0, c);
            syms.push(j as u16 + 1);
        }
        run_syms(zeros, &mut syms);
        syms.push(eob);
        Symbols {
            crc: 0,
            orig_ptr,
            seq_to_unseq,
            syms,
            eob,
            max_len: 900_000,
        }
    }

    fn copy(b: &Symbols) -> Symbols {
        Symbols {
            syms: b.syms.clone(),
            ..*b
        }
    }

    /// `decode_owned` with the fold (the open kernel's) against `decode_owned` without
    /// (`Segmented`'s): one output and one error, on blocks of enough symbols to be folded
    #[test]
    fn fold_decode_owned_is_the_shipped_decode() {
        use crate::bwt::Segmented;
        use crate::{decode_owned, Scratch};
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15 ^ 0xf01d);
        let (mut sf, mut ss) = (
            Scratch::<ForcedOpenCells>::default(),
            Scratch::<Segmented>::default(),
        );
        let mut tried = 0;
        for i in 0..400 {
            let mut b = synthetic(&mut rng, 40_000);
            bound(&mut b, &mut rng);
            let (mut ll, mut counts) = (Vec::new(), [0u32; 256]);
            if let Ok(n) = mtf::mtf_scalar(&b, &mut ll, &mut counts) {
                if (b.orig_ptr as usize) < n && rng.below(4) != 0 {
                    let mut tt = bwt::TVector::new();
                    bwt::build(&ll, &counts, &mut tt);
                    let mut out = Vec::new();
                    bwt::walk(&tt, b.orig_ptr, &mut out);
                    let mut text = Vec::new();
                    if crate::rle::unrle(&out, &mut text, Kernels::scalar()).is_ok() {
                        b.crc = crate::crc::crc32(&text, Kernels::scalar());
                    }
                }
            }
            tried += (fold::wants(&b) && takes_runs(&b)) as usize;
            for k in kernels() {
                let (mut bf, mut bs) = (copy(&b), copy(&b));
                let got = decode_owned(&mut bf, k, &mut sf, |n, _| Vec::with_capacity(n));
                let want = decode_owned(&mut bs, k, &mut ss, |n, _| Vec::with_capacity(n));
                assert!(got == want, "case {i}, {k:?}: {got:?} against {want:?}");
            }
        }
        assert!(tried > 50, "{tried} blocks for the fold");
    }
}
