//! Smooth-zone probe (`explore`: cooperative decoder dissection, zero `src/` edits):
//! 1. Audits where `Cooperative` actually triggers in `StreamDecoder::new` / `Plan::for_threads`.
//! 2. Benchmarks shipped `StreamDecoder::planned` (`Workers(1/2/4)` vs `Teams(1x2, 1x4, 2x2)`) on
//!    exact 1-, 2-, and 4-block streams across `wikidata`, `text`, `prose_src`, `records`, and `random`,
//!    plus cold `Pool::new(K)` thread-spawn vs warm steady-state latency.
//! 3. Measures exact phase breakdown of `Cooperative<K>` (`k = 1, 2, 4`) on both `Runs` and `Column`
//!    paths (Huffman, front+cuts / mtf_checkpoints, build, walk setup, walk, finish+assemble, RLE1+CRC,
//!    and pool barrier overhead) and computes Amdahl serial fractions and `1t -> 2t -> 4t` scaling.
//! 4. Benchmarks concrete, 100% byte-exact candidate levers on `src/coop.rs` (`L1` staged TILE=256
//!    8-way unrolled walk, `L2` parallel sequential LF build on Column blocks, `L3` register-resident
//!    `mtf_checkpoints`, `L4` trusted `fill_share` + AVX2 `assemble_reversed` + single-barrier pool)
//!    against shipped `Cooperative<K>` and 1-thread `OpenCells` (7 interleaved repeats).
//!
//! Run on baseline x86-64 and `-C target-cpu=native`:
//!   cargo run --release --features moonshot --example coop_dissection_probe -- --repeats 7 /tmp/wikidata.bz2 /tmp/coop_corpora/text.bz2 /tmp/coop_corpora/records.bz2 /tmp/coop_corpora/random.bz2
//!   RUSTFLAGS="-C target-cpu=native" cargo run --release --features moonshot --target-dir target/native --example coop_dissection_probe -- --repeats 7 /tmp/wikidata.bz2 /tmp/coop_corpora/text.bz2 /tmp/coop_corpora/records.bz2 /tmp/coop_corpora/random.bz2

#[cfg(feature = "moonshot")]
mod imp {
    use mibz2::block::Symbols;
    use mibz2::bwt::{self, Cooperative, InverseBwt, OpenCells};
    use mibz2::parallel::{Plan, StreamDecoder};
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::{crc, mtf, rle, Kernels};

    #[inline(always)]
    fn prefetch_l1(t: *const u32, row: u32) {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: prefetch is a non-faulting hint on valid table address.
        unsafe {
            std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                t.add(row as usize) as *const i8,
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        let _ = (t, row);
    }
    use std::any::Any;
    use std::fs;
    use std::hint::black_box;
    use std::io::Read;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, MutexGuard};
    use std::thread::{self, JoinHandle, Thread};
    use std::time::Instant;

    const TILE: usize = 256;
    const LANES: usize = 32;
    const BASE_SEGMENTS: usize = 512;
    const START: u32 = 1 << 31;
    const LINK: u32 = !START;
    const SHUTDOWN: usize = usize::MAX;
    const SPINS: u32 = 2_000;
    const PIECES: usize = 12;
    const CHECKPOINT: usize = mtf::CHECKPOINT;

    // ---- Linux CPU clock helpers ----

    #[cfg(target_os = "linux")]
    fn process_cpu_ns() -> u64 {
        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }
        extern "C" {
            fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
        }
        const CLOCK_PROCESS_CPUTIME_ID: i32 = 2;
        let mut ts = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: valid pointer to initialized Timespec.
        unsafe {
            clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts);
        }
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    }

    #[cfg(not(target_os = "linux"))]
    fn process_cpu_ns() -> u64 {
        0
    }

    fn median_range(vals: &[f64]) -> (f64, f64, f64) {
        let mut v = vals.to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = v[v.len() / 2];
        (med, v[0], v[v.len() - 1])
    }

    // ---- Faithful replica of src/coop.rs::Pool (for per-phase & lever instrumentation) ----

    #[derive(Clone, Copy)]
    struct Erased(*const (dyn Fn(usize) + Sync + 'static));
    // SAFETY: points at Sync closure only dereferenced during active `run`.
    unsafe impl Send for Erased {}

    struct Shared {
        epoch: AtomicUsize,
        done: AtomicUsize,
        task: Mutex<Option<Erased>>,
        panic: Mutex<Option<Box<dyn Any + Send>>>,
        caller: Mutex<Option<Thread>>,
    }

    fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
        m.lock().unwrap_or_else(|e| e.into_inner())
    }

    struct CoopPool {
        k: usize,
        shared: Arc<Shared>,
        helpers: Vec<JoinHandle<()>>,
    }

    impl CoopPool {
        fn new(k: usize) -> Self {
            let k = k.max(1);
            let shared = Arc::new(Shared {
                epoch: AtomicUsize::new(0),
                done: AtomicUsize::new(0),
                task: Mutex::new(None),
                panic: Mutex::new(None),
                caller: Mutex::new(None),
            });
            let mut helpers = Vec::with_capacity(k - 1);
            for idx in 1..k {
                let s = Arc::clone(&shared);
                let h = thread::Builder::new()
                    .name(format!("probe-coop-{idx}"))
                    .spawn(move || pool_helper(&s, idx))
                    .unwrap();
                helpers.push(h);
            }
            CoopPool { k, shared, helpers }
        }

        fn threads(&self) -> usize {
            self.k
        }

        fn run(&self, f: &(dyn Fn(usize) + Sync)) {
            if self.k == 1 {
                f(0);
                return;
            }
            let s = &*self.shared;
            let raw: *const (dyn Fn(usize) + Sync + '_) = f;
            // SAFETY: lifetime erased only for duration of `run`; `Wait` joins all helpers.
            let erased = Erased(unsafe {
                std::mem::transmute::<
                    *const (dyn Fn(usize) + Sync + '_),
                    *const (dyn Fn(usize) + Sync + 'static),
                >(raw)
            });
            *lock(&s.caller) = Some(thread::current());
            *lock(&s.task) = Some(erased);
            s.done.store(0, Ordering::Relaxed);
            s.epoch.fetch_add(1, Ordering::Release);
            for h in &self.helpers {
                h.thread().unpark();
            }
            let _ = catch_unwind(AssertUnwindSafe(|| f(0)));
            let mut spins = 0u32;
            while s.done.load(Ordering::Acquire) != self.k - 1 {
                spins += 1;
                if spins < SPINS {
                    std::hint::spin_loop();
                } else {
                    thread::park();
                }
            }
            *lock(&s.task) = None;
            let _ = lock(&s.panic).take();
        }
    }

    fn pool_helper(s: &Shared, idx: usize) {
        let mut seen = 0usize;
        loop {
            let mut spins = 0u32;
            let e = loop {
                let e = s.epoch.load(Ordering::Acquire);
                if e != seen {
                    break e;
                }
                spins += 1;
                if spins < SPINS {
                    std::hint::spin_loop();
                } else {
                    thread::park();
                }
            };
            if e == SHUTDOWN {
                return;
            }
            seen = e;
            let task = *lock(&s.task);
            if let Some(Erased(f)) = task {
                // SAFETY: caller waits for all helpers before `f` ends.
                let r = catch_unwind(AssertUnwindSafe(|| unsafe { (*f)(idx) }));
                if let Err(p) = r {
                    let mut slot = lock(&s.panic);
                    if slot.is_none() {
                        *slot = Some(p);
                    }
                }
            }
            let caller = lock(&s.caller).clone();
            s.done.fetch_add(1, Ordering::AcqRel);
            if let Some(c) = caller {
                c.unpark();
            }
        }
    }

    impl Drop for CoopPool {
        fn drop(&mut self) {
            self.shared.epoch.store(SHUTDOWN, Ordering::Release);
            for h in &self.helpers {
                h.thread().unpark();
            }
            for h in self.helpers.drain(..) {
                let _ = h.join();
            }
        }
    }

    // ---- Faithful runs::cuts + RunsBuild + FusedBuild + candidate LF build ----

    #[derive(Clone)]
    struct Cut {
        rec: usize,
        row: u32,
        next: [u32; 256],
    }

    fn make_cuts(records: &[u16], counts: &[u32; 256], n: usize, out: &mut Vec<Cut>) -> bool {
        out.clear();
        let mut next = [0u32; 256];
        let mut lim = [0u32; 256];
        let mut cum = 0u64;
        for c in 0..256 {
            next[c] = cum as u32;
            cum += counts[c] as u64;
            lim[c] = cum as u32;
        }
        if cum != n as u64 {
            return false;
        }
        let total = n as u64 + 8 * records.len() as u64;
        let at = |j: usize| (total * j as u64).div_ceil(PIECES as u64);
        out.push(Cut {
            rec: 0,
            row: 0,
            next,
        });
        let (mut q, mut work) = (0u32, 0u64);
        let mut due = at(1);
        let mut empty = false;
        for (i, &x) in records.iter().enumerate() {
            if work >= due {
                while out.len() < PIECES && work >= at(out.len()) {
                    out.push(Cut {
                        rec: i,
                        row: q,
                        next,
                    });
                }
                due = if out.len() < PIECES {
                    at(out.len())
                } else {
                    u64::MAX
                };
            }
            let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
            empty |= len == 0;
            next[c] += len;
            q += len;
            work += len as u64 + 8;
        }
        if empty || next != lim {
            out.clear();
            return false;
        }
        let end = Cut {
            rec: records.len(),
            row: q,
            next,
        };
        out.resize(PIECES + 1, end);
        true
    }

    #[derive(Clone, Copy)]
    struct Slots(*mut u32);
    // SAFETY: shares write strictly disjoint ranges of the reserved table.
    unsafe impl Sync for Slots {}

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_share_avx2_checked(
        runs: &[u16],
        row: u32,
        row_end: u32,
        mut next: [u32; 256],
        lim: &[u32; 256],
        out: *mut u32,
    ) -> bool {
        use std::arch::x86_64::*;
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut q = row;
        for &x in runs {
            let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
            if len == 0 || len > row_end - q || len > lim[c] - next[c] {
                return false;
            }
            let e = c as u32 | (next[c] << 8);
            let end = q + len;
            if row_end - end >= 7 {
                let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
                let mut w = q;
                loop {
                    _mm256_storeu_si256(out.add(w as usize) as *mut __m256i, v);
                    w += 8;
                    if w >= end {
                        break;
                    }
                    v = _mm256_add_epi32(v, step);
                }
            } else {
                for j in 0..len {
                    *out.add((q + j) as usize) = e + (j << 8);
                }
            }
            next[c] += len;
            q = end;
        }
        q == row_end && next == *lim
    }

    /// Lever 4a: trusted `fill_share_avx2` — `make_cuts` already validated every record in `runs`
    /// tiles `[from.row..to.row]` and `next == lim`. Split the last `<= 7` rows into a clean tail
    /// outside the hot loop so the main loop has ZERO per-record validation or end-distance branches!
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_share_avx2_trusted(
        runs: &[u16],
        row: u32,
        row_end: u32,
        mut next: [u32; 256],
        out: *mut u32,
    ) {
        use std::arch::x86_64::*;
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let safe_end = row_end.saturating_sub(7);
        let mut q = row;
        let mut idx = 0usize;
        let nrec = runs.len();
        while idx < nrec {
            let x = *runs.get_unchecked(idx);
            let c = (x >> 8) as usize;
            let len = (x & 0xff) as u32;
            let end = q + len;
            if end > safe_end {
                break;
            }
            let nc = *next.get_unchecked(c);
            let e = (c as u32) | (nc << 8);
            *next.get_unchecked_mut(c) = nc + len;
            let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
            let mut w = q;
            loop {
                _mm256_storeu_si256(out.add(w as usize) as *mut __m256i, v);
                w += 8;
                if w >= end {
                    break;
                }
                v = _mm256_add_epi32(v, step);
            }
            q = end;
            idx += 1;
        }
        while idx < nrec {
            let x = *runs.get_unchecked(idx);
            let c = (x >> 8) as usize;
            let len = (x & 0xff) as u32;
            let nc = *next.get_unchecked(c);
            let e = (c as u32) | (nc << 8);
            *next.get_unchecked_mut(c) = nc + len;
            for j in 0..len {
                *out.add((q + j) as usize) = e + (j << 8);
            }
            q += len;
            idx += 1;
        }
    }

    fn edge(n: usize, k: usize, stride: usize, j: usize) -> usize {
        if j >= k {
            n
        } else {
            ((n * j / k + stride / 2) / stride * stride).min(n)
        }
    }

    #[inline(always)]
    fn read_u64(s: &[u8], i: usize) -> u64 {
        u64::from_ne_bytes(s[i..i + 8].try_into().unwrap())
    }

    /// Exact replica of shipped `src/coop.rs::FusedBuild::share` (Ψ random scatter + random `ll[s]`).
    fn fused_build_psi_share(
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        starts: &[u32; 256],
        slots: Slots,
        j: usize,
        shares: usize,
    ) {
        let n = ll.len();
        let at = |e: usize| {
            if e == n {
                counts
            } else {
                &checkpoints[e / CHECKPOINT]
            }
        };
        let (lo, hi) = (
            edge(n, shares, CHECKPOINT, j),
            edge(n, shares, CHECKPOINT, j + 1),
        );
        let (before, after) = (at(lo), at(hi));
        let mut ftab = [0u32; 256];
        let mut lim = [0u32; 256];
        for b in 0..256 {
            ftab[b] = starts[b] + before[b];
            lim[b] = starts[b] + after[b];
        }
        let one = |i: usize, ftab: &mut [u32; 256]| -> bool {
            let c = ll[i] as usize;
            let s = ftab[c];
            if s >= lim[c] {
                return false;
            }
            // SAFETY: disjoint bucket windows within reserved `tt`.
            unsafe { *slots.0.add(s as usize) = ll[s as usize] as u32 | ((i as u32) << 8) };
            ftab[c] = s + 1;
            true
        };
        let mut i = lo;
        let mut ok = true;
        'rows: while i + 16 <= hi {
            let (a, b) = (read_u64(ll, i), read_u64(ll, i + 8));
            if a == b && a == a.rotate_left(8) {
                let c = ll[i] as usize;
                let s = ftab[c] as usize;
                if s + 16 > lim[c] as usize {
                    ok = false;
                    break 'rows;
                }
                let base = (i as u32) << 8;
                for (jj, &l) in ll[s..s + 16].iter().enumerate() {
                    // SAFETY: disjoint bucket window within reserved `tt`.
                    unsafe { *slots.0.add(s + jj) = l as u32 | (base + ((jj as u32) << 8)) };
                }
                ftab[c] += 16;
            } else {
                for ii in i..i + 16 {
                    if !one(ii, &mut ftab) {
                        ok = false;
                        break 'rows;
                    }
                }
            }
            i += 16;
        }
        while ok && i < hi {
            ok = one(i, &mut ftab);
            i += 1;
        }
        assert!(ok && ftab == lim);
    }

    /// Lever 2: Parallel 100% SEQUENTIAL `LF` build (`P[i] = ll[i] | ftab[c] << 8`) using existing
    /// MTF checkpoints! Zero random writes, zero random `ll[s]` reads, zero cross-core cache-line
    /// RFO bouncing (share `j` writes strictly contiguous `tt[lo..hi]`), with 8B equal-run fast path.
    fn fused_build_lf_share(
        ll: &[u8],
        counts: &[u32; 256],
        checkpoints: &[[u32; 256]],
        starts: &[u32; 256],
        slots: Slots,
        j: usize,
        shares: usize,
    ) {
        let n = ll.len();
        let at = |e: usize| {
            if e == n {
                counts
            } else {
                &checkpoints[e / CHECKPOINT]
            }
        };
        let (lo, hi) = (
            edge(n, shares, CHECKPOINT, j),
            edge(n, shares, CHECKPOINT, j + 1),
        );
        let (before, after) = (at(lo), at(hi));
        let mut ftab = [0u32; 256];
        let mut lim = [0u32; 256];
        for b in 0..256 {
            ftab[b] = starts[b] + before[b];
            lim[b] = starts[b] + after[b];
        }
        let mut i = lo;
        while i + 8 <= hi {
            let w = read_u64(ll, i);
            if w == w.rotate_left(8) {
                let c = (w & 0xff) as usize;
                let base = (c as u32) | (ftab[c] << 8);
                // SAFETY: share `j` exclusively owns contiguous `slots[lo..hi]`.
                unsafe {
                    let dst = slots.0.add(i);
                    *dst = base;
                    *dst.add(1) = base + 256;
                    *dst.add(2) = base + 512;
                    *dst.add(3) = base + 768;
                    *dst.add(4) = base + 1024;
                    *dst.add(5) = base + 1280;
                    *dst.add(6) = base + 1536;
                    *dst.add(7) = base + 1792;
                }
                ftab[c] += 8;
                i += 8;
            } else {
                // SAFETY: share `j` exclusively owns contiguous `slots[lo..hi]`.
                unsafe {
                    let dst = slots.0.add(i);
                    for k in 0..8 {
                        let c = *ll.get_unchecked(i + k) as usize;
                        let s = *ftab.get_unchecked(c);
                        *ftab.get_unchecked_mut(c) = s + 1;
                        *dst.add(k) = (c as u32) | (s << 8);
                    }
                }
                i += 8;
            }
        }
        while i < hi {
            let c = ll[i] as usize;
            let s = ftab[c];
            ftab[c] = s + 1;
            // SAFETY: share `j` exclusively owns contiguous `slots[lo..hi]`.
            unsafe { *slots.0.add(i) = (c as u32) | (s << 8) };
            i += 1;
        }
        assert!(ftab == lim);
    }

    // ---- Lever 3: Register-resident `(f0, f1)` MTF with checkpoints (replaces `PagedList`) ----

    #[cfg(target_arch = "x86_64")]
    #[repr(C, align(64))]
    #[derive(Clone, Copy)]
    struct Step {
        a: [u8; 16],
        b: [u8; 16],
        m: [u8; 16],
        _pad: [u8; 16],
    }

    #[cfg(target_arch = "x86_64")]
    const fn make_steps() -> [Step; 32] {
        const Z: u8 = 0x80;
        let mut t = [Step {
            a: [0; 16],
            b: [Z; 16],
            m: [0; 16],
            _pad: [0; 16],
        }; 32];
        let mut nn = 0;
        while nn < 32 {
            let mut i = 0;
            while i < 16 {
                if nn < 16 {
                    t[nn].a[i] = if i == 0 {
                        nn as u8
                    } else if i <= nn {
                        (i - 1) as u8
                    } else {
                        i as u8
                    };
                } else {
                    t[nn].a[i] = if i == 0 { Z } else { (i - 1) as u8 };
                    if i == 0 {
                        t[nn].b[i] = (nn - 16) as u8;
                    }
                    if i <= nn - 16 {
                        t[nn].m[i] = 0xff;
                    }
                }
                i += 1;
            }
            nn += 1;
        }
        t
    }

    #[cfg(target_arch = "x86_64")]
    static STEPS: [Step; 32] = make_steps();

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    unsafe fn mtf_second(
        tail: *mut u8,
        t: usize,
        f1: std::arch::x86_64::__m128i,
        idx: std::arch::x86_64::__m256i,
    ) -> u8 {
        use std::arch::x86_64::*;
        let v = *tail.add(t);
        let row = |j: usize| _mm256_load_si256(tail.add(32 * j) as *const __m256i);
        let below0 = _mm256_inserti128_si256::<1>(_mm256_castsi128_si256(f1), f1);
        let on = |r: __m256i, below: __m256i| {
            _mm256_alignr_epi8::<15>(r, _mm256_permute2x128_si256::<0x03>(r, below))
        };
        if t < 32 {
            let r0 = row(0);
            let m = _mm256_cmpgt_epi8(_mm256_set1_epi8(t as i8 + 1), idx);
            _mm256_store_si256(
                tail as *mut __m256i,
                _mm256_blendv_epi8(r0, on(r0, below0), m),
            );
            return v;
        }
        let tv = _mm256_set1_epi32(t as i32);
        let i0 = _mm256_cvtepu8_epi32(_mm256_castsi256_si128(idx));
        let mut prev = below0;
        let mut base = _mm256_setzero_si256();
        let step32 = _mm256_set1_epi32(32);
        let eight = _mm256_set1_epi32(8);
        for j in 0..7 {
            let r = row(j);
            let moved = on(r, prev);
            let mut b = _mm256_add_epi32(base, i0);
            let w0 = _mm256_cmpgt_epi32(tv, _mm256_sub_epi32(b, _mm256_set1_epi32(1)));
            b = _mm256_add_epi32(b, eight);
            let w1 = _mm256_cmpgt_epi32(tv, _mm256_sub_epi32(b, _mm256_set1_epi32(1)));
            b = _mm256_add_epi32(b, eight);
            let w2 = _mm256_cmpgt_epi32(tv, _mm256_sub_epi32(b, _mm256_set1_epi32(1)));
            b = _mm256_add_epi32(b, eight);
            let w3 = _mm256_cmpgt_epi32(tv, _mm256_sub_epi32(b, _mm256_set1_epi32(1)));
            let p01 = _mm256_packs_epi32(w0, w1);
            let p23 = _mm256_packs_epi32(w2, w3);
            let m = _mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16(p01, p23));
            let m = _mm256_shuffle_epi32::<0xd8>(m);
            _mm256_store_si256(
                tail.add(32 * j) as *mut __m256i,
                _mm256_blendv_epi8(r, moved, m),
            );
            prev = r;
            base = _mm256_add_epi32(base, step32);
        }
        v
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn mtf_checkpoints_reg_avx2(
        b: &Symbols,
        ll: &mut Vec<u8>,
        counts: &mut [u32; 256],
        checkpoints: &mut Vec<[u32; 256]>,
    ) -> usize {
        use std::arch::x86_64::*;
        let max = b.max_len;
        ll.clear();
        ll.reserve(max);
        *counts = [0; 256];
        checkpoints.clear();
        checkpoints.push([0; 256]);
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        #[repr(align(64))]
        struct Rows([u8; 224]);
        let mut rows = Rows([0u8; 224]);
        rows.0.copy_from_slice(&s2u[32..]);
        let tail = rows.0.as_mut_ptr();
        let idx = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        );
        let steps = STEPS.as_ptr();
        let (mut run, mut weight) = (0u32, 1u32);
        let out = ll.as_mut_ptr();
        let mut n = 0usize;
        macro_rules! step_byte {
            ($s:expr) => {{
                let nn = (($s - 1) as usize).min(255);
                if nn < 32 {
                    let t = steps.add(nn);
                    let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
                    let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
                    let m = _mm_load_si128((*t).m.as_ptr() as *const __m128i);
                    let g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, bb));
                    f1 = _mm_blendv_epi8(f1, _mm_alignr_epi8::<15>(f1, f0), m);
                    f0 = g0;
                } else {
                    let v = mtf_second(tail, nn - 32, f1, idx);
                    f1 = _mm_alignr_epi8::<15>(f1, f0);
                    f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
                }
                let c = _mm_cvtsi128_si32(f0) as u8;
                counts[c as usize] += 1;
                *out.add(n) = c;
                n += 1;
            }};
        }
        let mut next = CHECKPOINT;
        let mut syms = b.syms.iter();
        'block: loop {
            let limit = max.min(next);
            let mut at = None;
            for &s in syms.by_ref() {
                if s <= 1 {
                    run += (s as u32 + 1) * weight;
                    weight <<= 1;
                    continue;
                }
                if run != 0 {
                    if n + run as usize > limit {
                        at = Some(s);
                        break;
                    }
                    let c = _mm_cvtsi128_si32(f0) as u8;
                    counts[c as usize] += run;
                    if run <= 64 && n + 64 <= max {
                        let v = _mm256_broadcastb_epi8(f0);
                        _mm256_storeu_si256(out.add(n) as *mut __m256i, v);
                        _mm256_storeu_si256(out.add(n + 32) as *mut __m256i, v);
                    } else {
                        std::ptr::write_bytes(out.add(n), c, run as usize);
                    }
                    n += run as usize;
                    (run, weight) = (0, 1);
                }
                if s == b.eob {
                    break 'block;
                }
                if n >= limit {
                    at = Some(s);
                    break;
                }
                step_byte!(s);
            }
            let Some(s) = at else { break 'block };
            if run != 0 {
                let c = _mm_cvtsi128_si32(f0) as u8;
                let mut left = run as usize;
                while n + left >= next {
                    let part = next - n;
                    counts[c as usize] += part as u32;
                    std::ptr::write_bytes(out.add(n), c, part);
                    (n, left) = (next, left - part);
                    checkpoints.push(*counts);
                    next += CHECKPOINT;
                }
                counts[c as usize] += left as u32;
                std::ptr::write_bytes(out.add(n), c, left);
                n += left;
                (run, weight) = (0, 1);
                if s == b.eob {
                    break 'block;
                }
            } else {
                checkpoints.push(*counts);
                next += CHECKPOINT;
            }
            step_byte!(s);
        }
        if n == next {
            checkpoints.push(*counts);
        }
        ll.set_len(n);
        n
    }

    // ---- Shared Walk: Shipped Unstaged (`src/coop.rs::traverse`) vs Lever 1 Staged (`TILE=256`) ----

    #[derive(Clone, Copy)]
    struct Lane {
        row: u32,
        seg: usize,
        p: *mut u8,
        end: *mut u8,
    }

    #[derive(Clone, Copy)]
    struct Segs {
        bufs: *mut Vec<u8>,
        next: *mut usize,
    }
    // SAFETY: each segment index is claimed once via `taken.fetch_add(1)`.
    unsafe impl Sync for Segs {}

    #[derive(Clone, Copy)]
    struct WalkShared {
        t: *const u32,
        starts_ptr: *const u32,
        starts_len: usize,
        segs: Segs,
        cap: usize,
    }
    // SAFETY: `t` and `starts` are read-only during walk; each segment has a single owner.
    unsafe impl Sync for WalkShared {}

    #[derive(Default)]
    struct LocalSegScratch {
        starts: Vec<u32>,
        bufs: Vec<Vec<u8>>,
        next: Vec<usize>,
        n: usize,
        first: usize,
    }

    /// Deterministic PRNG matching `src/bwt.rs::random_rows`.
    fn pick_starts(n: usize, count: usize, out: &mut Vec<u32>) {
        out.clear();
        let mut x: u64 = 0x9e37_79b9_7f4a_7c15 ^ (n as u64);
        for _ in 0..count {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            out.push((x % (n as u64)) as u32);
        }
    }

    fn walk_setup(tt: &mut [u32], r0: u32, s: &mut LocalSegScratch, segments: usize) -> WalkShared {
        let n = tt.len();
        pick_starts(n, segments.max(1) - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            tt[r as usize] |= START;
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        WalkShared {
            t: tt.as_ptr(),
            starts_ptr: s.starts.as_ptr(),
            starts_len: ns,
            segs: Segs {
                bufs: s.bufs.as_mut_ptr(),
                next: s.next.as_mut_ptr(),
            },
            cap,
        }
    }

    fn walk_finish(tt: &mut [u32], r0: u32, s: &mut LocalSegScratch) {
        for &r in &s.starts {
            tt[r as usize] &= LINK;
        }
        s.first = s.starts.binary_search(&r0).unwrap_or(usize::MAX);
        s.n = tt.len();
    }

    /// Faithful replica of shipped `src/coop.rs::traverse::<32>` (unstaged direct heap stores).
    unsafe fn traverse_unstaged(ws: WalkShared, taken: &AtomicUsize) {
        let t = ws.t;
        let starts = std::slice::from_raw_parts(ws.starts_ptr, ws.starts_len);
        let ns = starts.len();
        let segs = ws.segs;
        let cap = ws.cap;
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let take = |seg: usize| -> Lane {
            let b = &mut *segs.bufs.add(seg);
            let base = b.as_mut_ptr();
            let q = *t.add(starts[seg] as usize) & LINK;
            *base = q as u8;
            prefetch_l1(t, q >> 8);
            Lane {
                row: q >> 8,
                seg,
                p: base.add(1),
                end: base.add(b.capacity()),
            }
        };
        let finish = |l: &Lane| {
            let b = &mut *segs.bufs.add(l.seg);
            let len = l.p.offset_from(b.as_ptr()) as usize;
            b.set_len(len);
            *segs.next.add(l.seg) = seg_of(l.row);
        };
        let grow = |l: &mut Lane| {
            let b = &mut *segs.bufs.add(l.seg);
            let len = l.p.offset_from(b.as_ptr()) as usize;
            b.set_len(len);
            b.reserve(b.capacity());
            l.p = b.as_mut_ptr().add(len);
            l.end = b.as_mut_ptr().add(b.capacity());
        };
        let next_seg = || {
            let s = taken.fetch_add(1, Ordering::Relaxed);
            (s < ns).then_some(s)
        };

        let mut lanes = [Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        }; LANES];
        let mut active = 0;
        while active < LANES {
            let Some(seg) = next_seg() else { break };
            lanes[active] = take(seg);
            active += 1;
        }
        if active == LANES {
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l);
                    }
                    room = room.min(l.end.offset_from(l.p) as usize);
                }
                let mut base: [*mut u8; LANES] = std::array::from_fn(|k| lanes[k].p);
                for off in 0..room {
                    for k in 0..LANES {
                        let l = &mut lanes[k];
                        let q = *t.add(l.row as usize);
                        if q & START != 0 {
                            l.p = base[k].wrapping_add(off);
                            finish(l);
                            let Some(seg) = next_seg() else {
                                for (j, lj) in lanes.iter_mut().enumerate() {
                                    if j < k {
                                        lj.p = base[j].wrapping_add(off + 1);
                                    } else if j > k {
                                        lj.p = base[j].wrapping_add(off);
                                    }
                                }
                                break 'full;
                            };
                            *l = take(seg);
                            base[k] = l.p.wrapping_sub(off + 1);
                            continue;
                        }
                        l.row = q >> 8;
                        prefetch_l1(t, l.row);
                        *base[k].wrapping_add(off) = q as u8;
                    }
                }
                for (l, b) in lanes.iter_mut().zip(base) {
                    l.p = b.wrapping_add(room);
                }
            }
        }
        let unfinished = |l: &Lane| l.seg != usize::MAX && *segs.next.add(l.seg) == usize::MAX;
        let mut live: Vec<Lane> = lanes[..active].iter().copied().filter(unfinished).collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = *t.add(l.row as usize);
                if q & START != 0 {
                    finish(l);
                    live.swap_remove(i);
                    continue;
                }
                l.row = q >> 8;
                prefetch_l1(t, l.row);
                *l.p = q as u8;
                l.p = l.p.add(1);
                if l.p == l.end {
                    grow(l);
                }
                i += 1;
            }
        }
    }

    /// Lever 1: `TILE = 256` L1-staged, 8-way unrolled deferred-flag cooperative walk
    /// (ports `#171`'s `bwt::room_segments::<32, PF_L1, true>` to `coop::traverse` with atomic `taken`).
    unsafe fn traverse_staged(ws: WalkShared, taken: &AtomicUsize) {
        let t = ws.t;
        let starts = std::slice::from_raw_parts(ws.starts_ptr, ws.starts_len);
        let ns = starts.len();
        let segs = ws.segs;
        let cap = ws.cap;
        let seg_of = |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let take = |seg: usize| -> Lane {
            let b = &mut *segs.bufs.add(seg);
            let base = b.as_mut_ptr();
            let q = *t.add(starts[seg] as usize) & LINK;
            *base = q as u8;
            prefetch_l1(t, q >> 8);
            Lane {
                row: q >> 8,
                seg,
                p: base.add(1),
                end: base.add(b.capacity()),
            }
        };
        let finish = |l: &Lane| {
            let b = &mut *segs.bufs.add(l.seg);
            let len = l.p.offset_from(b.as_ptr()) as usize;
            b.set_len(len);
            *segs.next.add(l.seg) = seg_of(l.row);
        };
        let grow = |l: &mut Lane| {
            let b = &mut *segs.bufs.add(l.seg);
            let len = l.p.offset_from(b.as_ptr()) as usize;
            b.set_len(len);
            b.reserve(b.capacity());
            l.p = b.as_mut_ptr().add(len);
            l.end = b.as_mut_ptr().add(b.capacity());
        };
        let flush = |l: &mut Lane, row: &[u8; TILE], from: usize, to: usize| {
            let count = to - from;
            if count > 0 {
                std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, count);
                l.p = l.p.add(count);
            }
        };
        let next_seg = || {
            let s = taken.fetch_add(1, Ordering::Relaxed);
            (s < ns).then_some(s)
        };

        let mut lanes = [Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        }; LANES];
        let mut active = 0;
        while active < LANES {
            let Some(seg) = next_seg() else { break };
            lanes[active] = take(seg);
            active += 1;
        }
        if active == LANES {
            let mut tile = [[0u8; TILE]; LANES];
            let mut rs: [u32; LANES] = std::array::from_fn(|k| lanes[k].row);
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l);
                    }
                    room = room.min(l.end.offset_from(l.p) as usize);
                }
                let mut done = 0usize;
                while done < room {
                    let w = (room - done).min(TILE);
                    let mut st = [0usize; LANES];
                    let tp = tile.as_mut_ptr().cast::<u8>();
                    for j in 0..w {
                        let mut marked = 0u64;
                        macro_rules! sstep {
                            ($k:expr) => {{
                                let k = $k;
                                let r = rs.get_unchecked_mut(k);
                                let q = *t.add(*r as usize);
                                if q & START != 0 {
                                    marked |= 1u64 << k;
                                } else {
                                    *tp.add(k * TILE + j) = q as u8;
                                    *r = q >> 8;
                                    prefetch_l1(t, q >> 8);
                                }
                            }};
                        }
                        sstep!(0);
                        sstep!(1);
                        sstep!(2);
                        sstep!(3);
                        sstep!(4);
                        sstep!(5);
                        sstep!(6);
                        sstep!(7);
                        sstep!(8);
                        sstep!(9);
                        sstep!(10);
                        sstep!(11);
                        sstep!(12);
                        sstep!(13);
                        sstep!(14);
                        sstep!(15);
                        sstep!(16);
                        sstep!(17);
                        sstep!(18);
                        sstep!(19);
                        sstep!(20);
                        sstep!(21);
                        sstep!(22);
                        sstep!(23);
                        sstep!(24);
                        sstep!(25);
                        sstep!(26);
                        sstep!(27);
                        sstep!(28);
                        sstep!(29);
                        sstep!(30);
                        sstep!(31);

                        while marked != 0 {
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            l.row = rs[k];
                            flush(l, &tile[k], st[k], j);
                            finish(l);
                            let Some(seg) = next_seg() else {
                                for (i, li) in lanes.iter_mut().enumerate() {
                                    li.row = rs[i];
                                    if i != k {
                                        let waiting = (marked >> i) & 1 != 0;
                                        flush(li, &tile[i], st[i], j + usize::from(!waiting));
                                    }
                                }
                                break 'full;
                            };
                            *l = take(seg);
                            rs[k] = l.row;
                            st[k] = j + 1;
                        }
                    }
                    for (k, l) in lanes.iter_mut().enumerate() {
                        flush(l, &tile[k], st[k], w);
                    }
                    done += w;
                }
            }
        }
        let unfinished = |l: &Lane| l.seg != usize::MAX && *segs.next.add(l.seg) == usize::MAX;
        let mut live: Vec<Lane> = lanes[..active].iter().copied().filter(unfinished).collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = *t.add(l.row as usize);
                if q & START != 0 {
                    finish(l);
                    live.swap_remove(i);
                    continue;
                }
                l.row = q >> 8;
                prefetch_l1(t, l.row);
                *l.p = q as u8;
                l.p = l.p.add(1);
                if l.p == l.end {
                    grow(l);
                }
                i += 1;
            }
        }
    }

    // ---- Assembly: scalar vs AVX2/u64 vectorized reverse ----

    fn repeat_cycle(out: &mut [u8], m: usize) {
        let n = out.len();
        if m == 0 || m >= n {
            return;
        }
        out.copy_within(n - m.., 0);
        let mut have = m;
        while have < n {
            let take = have.min(n - have);
            out.copy_within(..take, have);
            have += take;
        }
    }

    fn assemble_forward(s: &LocalSegScratch, out: &mut Vec<u8>) {
        let n = s.n;
        out.clear();
        out.reserve(n);
        let mut seg = s.first;
        while out.len() < n {
            let b = &s.bufs[seg];
            out.extend_from_slice(&b[..b.len().min(n - out.len())]);
            seg = s.next[seg];
        }
    }

    fn assemble_reversed_scalar(s: &LocalSegScratch, out: &mut Vec<u8>) {
        let n = s.n;
        out.clear();
        out.reserve(n);
        let dst = &mut out.spare_capacity_mut()[..n];
        let mut at = n;
        let mut seg = s.first;
        while at > 0 {
            let b = &s.bufs[seg];
            let len = b.len().min(at);
            for (d, &x) in dst[at - len..at].iter_mut().rev().zip(&b[..len]) {
                d.write(x);
            }
            at -= len;
            seg = s.next[seg];
            if seg == s.first {
                break;
            }
        }
        // SAFETY: dst[at..n] initialized; repeat_cycle fills dst[..at] when at > 0.
        unsafe {
            let sl = std::slice::from_raw_parts_mut(out.as_mut_ptr(), n);
            repeat_cycle(sl, n - at);
            out.set_len(n);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn reverse_copy_avx2(src: *const u8, dst: *mut u8, len: usize) {
        use std::arch::x86_64::*;
        let shuf = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let mut i = 0usize;
        while i + 32 <= len {
            let v = _mm256_loadu_si256(src.add(i) as *const __m256i);
            let r = _mm256_shuffle_epi8(v, shuf);
            let s = _mm256_permute2x128_si256::<0x01>(r, r);
            _mm256_storeu_si256(dst.add(len - i - 32) as *mut __m256i, s);
            i += 32;
        }
        while i + 8 <= len {
            let w = (src.add(i) as *const u64).read_unaligned().swap_bytes();
            (dst.add(len - i - 8) as *mut u64).write_unaligned(w);
            i += 8;
        }
        while i < len {
            *dst.add(len - 1 - i) = *src.add(i);
            i += 1;
        }
    }

    fn assemble_reversed_vec(s: &LocalSegScratch, out: &mut Vec<u8>, k: Kernels) {
        let n = s.n;
        out.clear();
        out.reserve(n);
        let dst = out.as_mut_ptr();
        let mut at = n;
        let mut seg = s.first;
        let use_avx2 = k.avx2();
        while at > 0 {
            let b = &s.bufs[seg];
            let len = b.len().min(at);
            let src = b.as_ptr();
            let base = at - len;
            // SAFETY: base..at <= n <= out.capacity(); src[..len] valid.
            unsafe {
                #[cfg(target_arch = "x86_64")]
                if use_avx2 && len >= 32 {
                    reverse_copy_avx2(src, dst.add(base), len);
                } else {
                    let mut i = 0usize;
                    while i + 8 <= len {
                        let w = (src.add(i) as *const u64).read_unaligned().swap_bytes();
                        (dst.add(at - i - 8) as *mut u64).write_unaligned(w);
                        i += 8;
                    }
                    while i < len {
                        *dst.add(at - 1 - i) = *src.add(i);
                        i += 1;
                    }
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    let _ = use_avx2;
                    let mut i = 0usize;
                    while i + 8 <= len {
                        let w = (src.add(i) as *const u64).read_unaligned().swap_bytes();
                        (dst.add(at - i - 8) as *mut u64).write_unaligned(w);
                        i += 8;
                    }
                    while i < len {
                        *dst.add(at - 1 - i) = *src.add(i);
                        i += 1;
                    }
                }
            }
            at = base;
            seg = s.next[seg];
            if seg == s.first {
                break;
            }
        }
        // SAFETY: dst[at..n] initialized; repeat_cycle fills dst[..at] when at > 0.
        unsafe {
            let sl = std::slice::from_raw_parts_mut(dst, n);
            repeat_cycle(sl, n - at);
            out.set_len(n);
        }
    }

    // ---- Extracted block & bzip2 sub-stream builder ----

    struct BlockInfo {
        syms: Symbols,
        orig_ptr: u32,
        takes_runs: bool,
        open_admits: bool,
        n: usize,
        // Pre-decoded reference output bytes (post-RLE1) and pre-RLE1 bytes
        ref_pre_rle: Vec<u8>,
        ref_decoded: Vec<u8>,
        crc: u32,
        // Cached column & runs data for isolated inversion lever tests
        ll: Vec<u8>,
        counts: [u32; 256],
        checkpoints: Vec<[u32; 256]>,
        runs_records: Vec<u16>,
        runs_counts: [u32; 256],
    }

    fn extract_blocks(bz2_bytes: &[u8], max_blocks: usize, k: Kernels) -> Vec<BlockInfo> {
        let mut sc = Scanner::new(SliceSource(bz2_bytes));
        let mut syms_buf = Vec::new();
        let mut out_blocks = Vec::new();
        let mut open = OpenCells::default();
        loop {
            match sc.next(syms_buf).expect("scan error") {
                Event::Done => break,
                Event::Block { symbols, .. } => {
                    let syms = Symbols {
                        syms: symbols.syms.clone(),
                        seq_to_unseq: symbols.seq_to_unseq,
                        eob: symbols.eob,
                        orig_ptr: symbols.orig_ptr,
                        crc: symbols.crc,
                        max_len: symbols.max_len,
                    };
                    syms_buf = symbols.syms;
                    let orig_ptr = syms.orig_ptr;
                    let takes_runs = runs::takes_runs(&syms);
                    let mut ll = Vec::new();
                    let mut counts = [0u32; 256];
                    let mut checkpoints = Vec::new();
                    let n = mtf::mtf_checkpoints(&syms, &mut ll, &mut counts, &mut checkpoints, k)
                        .unwrap();
                    if n < bwt::MIN_COOP {
                        continue;
                    }
                    let mut r = Runs::default();
                    let mut runs_records = Vec::new();
                    let mut runs_counts = [0u32; 256];
                    let mut open_admits = false;
                    if takes_runs {
                        let rn = runs::front(&syms, &mut r, k).unwrap();
                        assert_eq!(rn, n);
                        runs_records = r.records().to_vec();
                        runs_counts = counts;
                        let mut tmp = Vec::new();
                        open_admits = open.invert_runs(&r, n, orig_ptr, &mut tmp, k);
                    }
                    // Reference pre-RLE1 via shipped Cooperative<2>
                    let mut coop = Cooperative::<2>::default();
                    let mut ref_pre = Vec::new();
                    if takes_runs {
                        assert!(coop.invert_runs(&r, n, orig_ptr, &mut ref_pre, k));
                    } else {
                        coop.invert_checkpointed(
                            &ll,
                            &counts,
                            &checkpoints,
                            orig_ptr,
                            &mut ref_pre,
                        );
                    }
                    let mut ref_dec = Vec::new();
                    rle::unrle(&ref_pre, &mut ref_dec, k).unwrap();
                    let crc_val = crc::update_with(0, &ref_dec, k);
                    out_blocks.push(BlockInfo {
                        syms,
                        orig_ptr,
                        takes_runs,
                        open_admits,
                        n,
                        ref_pre_rle: ref_pre,
                        ref_decoded: ref_dec,
                        crc: crc_val,
                        ll,
                        counts,
                        checkpoints,
                        runs_records,
                        runs_counts,
                    });
                    if out_blocks.len() >= max_blocks {
                        break;
                    }
                }
            }
        }
        out_blocks
    }

    /// Build valid 1-, 2-, 4-block `.bz2` streams from raw decoded payloads using system `bzip2 -9`.
    fn compress_blocks(raw: &[u8]) -> Vec<u8> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("bzip2")
            .arg("-9")
            .arg("-c")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("bzip2 binary required");
        let mut stdin = child.stdin.take().unwrap();
        let out = thread::scope(|s| {
            s.spawn(move || {
                stdin.write_all(raw).unwrap();
                drop(stdin);
            });
            child.wait_with_output().unwrap()
        });
        assert!(out.status.success(), "bzip2 failed");
        out.stdout
    }

    fn run_stream_plan(bz2: &[u8], plan: Plan, k: Kernels) -> (f64, f64, usize) {
        let slots = (plan.workers() * 4).max(4);
        let t0 = Instant::now();
        let c0 = process_cpu_ns();
        let dec = StreamDecoder::planned(plan, slots, k);
        let total = thread::scope(|s| {
            let dec_ref = &dec;
            s.spawn(move || {
                let _ = dec_ref.write(bz2.to_vec());
                dec_ref.close();
            });
            let mut sum = 0usize;
            let mut spent = Vec::new();
            while let Some(b) = dec.read_block(spent).unwrap() {
                sum += b.len();
                black_box(b.first().copied().unwrap_or(0));
                spent = b;
            }
            sum
        });
        let wall_ms = t0.elapsed().as_secs_f64() * 1e3;
        let cpu_ms = (process_cpu_ns().saturating_sub(c0)) as f64 / 1e6;
        (wall_ms, cpu_ms, total)
    }

    // ---- Build realistic synthetic prose+source and JSON corpora if needed ----

    fn build_prose_src_raw(target_bytes: usize) -> Vec<u8> {
        let mut raw = Vec::with_capacity(target_bytes + 65536);
        let mut files = Vec::new();
        for dir in ["src", "docs/journal"] {
            if let Ok(rd) = fs::read_dir(dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_file() {
                        files.push(p);
                    }
                }
            }
        }
        files.sort();
        let mut pass = 0usize;
        while raw.len() < target_bytes && !files.is_empty() {
            for p in &files {
                if let Ok(mut f) = fs::File::open(p) {
                    let mut buf = Vec::new();
                    if f.read_to_end(&mut buf).is_ok() {
                        // Perturb pass index slightly so blocks aren't identical duplicates
                        if pass > 0 {
                            raw.extend_from_slice(format!("\n--- pass {pass} ---\n").as_bytes());
                        }
                        raw.extend_from_slice(&buf);
                        if raw.len() >= target_bytes {
                            break;
                        }
                    }
                }
            }
            pass += 1;
        }
        raw.truncate(target_bytes);
        raw
    }

    // ---- Lever runner: single-barrier vs double-barrier cooperative inversion ----

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Variant {
        Shipped,
        L1StagedWalk,
        L2LfBuild,
        L4TrustAsm1Bar,
        AllCombined,
    }

    impl Variant {
        fn name(self) -> &'static str {
            match self {
                Variant::Shipped => "shipped_coop",
                Variant::L1StagedWalk => "L1_staged_walk",
                Variant::L2LfBuild => "L2_lf_build",
                Variant::L4TrustAsm1Bar => "L4_trust_asm_1bar",
                Variant::AllCombined => "C_all_combined",
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_coop_invert_variant(
        blk: &BlockInfo,
        pool: &CoopPool,
        k: Kernels,
        variant: Variant,
        tt: &mut Vec<u32>,
        seg: &mut LocalSegScratch,
        cuts: &mut Vec<Cut>,
        out: &mut Vec<u8>,
    ) {
        let threads = pool.threads();
        let segments = BASE_SEGMENTS * threads;
        let use_staged = matches!(variant, Variant::L1StagedWalk | Variant::AllCombined);
        let use_lf_col = matches!(variant, Variant::L2LfBuild | Variant::AllCombined);
        let use_trusted_and_vec = matches!(variant, Variant::L4TrustAsm1Bar | Variant::AllCombined);
        let use_single_bar = matches!(variant, Variant::L4TrustAsm1Bar | Variant::AllCombined);

        let is_lf = blk.takes_runs || use_lf_col;
        let n = blk.n;
        tt.clear();
        tt.reserve(n + 8);
        let slots = Slots(tt.as_mut_ptr());

        if blk.takes_runs {
            assert!(make_cuts(&blk.runs_records, &blk.runs_counts, n, cuts));
        }

        let mut starts = [0u32; 256];
        if !blk.takes_runs {
            let mut cum = 0u32;
            for (s, &c) in starts.iter_mut().zip(&blk.counts) {
                *s = cum;
                cum += c;
            }
        }

        let run_build_share = |j: usize| {
            if blk.takes_runs {
                let (from, to) = (
                    &cuts[PIECES * j / threads],
                    &cuts[PIECES * (j + 1) / threads],
                );
                let slice = &blk.runs_records[from.rec..to.rec];
                // SAFETY: share owns disjoint `from.row..to.row` inside reserved `tt`.
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    if k.avx2() {
                        if use_trusted_and_vec {
                            fill_share_avx2_trusted(slice, from.row, to.row, from.next, slots.0);
                        } else {
                            assert!(fill_share_avx2_checked(
                                slice, from.row, to.row, from.next, &to.next, slots.0
                            ));
                        }
                    } else {
                        let mut next = from.next;
                        let mut q = from.row;
                        for &x in slice {
                            let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
                            let e = (c as u32) | (next[c] << 8);
                            for jj in 0..len {
                                *slots.0.add((q + jj) as usize) = e + (jj << 8);
                            }
                            next[c] += len;
                            q += len;
                        }
                    }
                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        let mut next = from.next;
                        let mut q = from.row;
                        for &x in slice {
                            let (c, len) = ((x >> 8) as usize, (x & 0xff) as u32);
                            let e = (c as u32) | (next[c] << 8);
                            for jj in 0..len {
                                *slots.0.add((q + jj) as usize) = e + (jj << 8);
                            }
                            next[c] += len;
                            q += len;
                        }
                    }
                }
            } else if use_lf_col {
                fused_build_lf_share(
                    &blk.ll,
                    &blk.counts,
                    &blk.checkpoints,
                    &starts,
                    slots,
                    j,
                    threads,
                );
            } else {
                fused_build_psi_share(
                    &blk.ll,
                    &blk.counts,
                    &blk.checkpoints,
                    &starts,
                    slots,
                    j,
                    threads,
                );
            }
        };

        if use_single_bar && threads > 1 {
            // Single `Pool::run`: build -> lightweight atomic sense barrier -> thread 0 sets up walk -> walk!
            let built_done = AtomicUsize::new(0);
            let walk_ready = AtomicBool::new(false);
            let taken = AtomicUsize::new(0);
            struct SyncCell<'a>(
                std::cell::UnsafeCell<WalkShared>,
                std::cell::UnsafeCell<&'a mut Vec<u32>>,
                std::cell::UnsafeCell<&'a mut LocalSegScratch>,
            );
            // SAFETY: only thread 0 writes inside `SyncCell` between `built_done == threads` and `walk_ready = true`.
            unsafe impl Sync for SyncCell<'_> {}
            let sc = SyncCell(
                std::cell::UnsafeCell::new(WalkShared {
                    t: std::ptr::null(),
                    starts_ptr: std::ptr::null(),
                    starts_len: 0,
                    segs: Segs {
                        bufs: std::ptr::null_mut(),
                        next: std::ptr::null_mut(),
                    },
                    cap: 0,
                }),
                std::cell::UnsafeCell::new(tt),
                std::cell::UnsafeCell::new(seg),
            );
            let sc = &sc;

            pool.run(&|j| {
                run_build_share(j);
                built_done.fetch_add(1, Ordering::AcqRel);
                if j == 0 {
                    let mut sp = 0u32;
                    while built_done.load(Ordering::Acquire) != threads {
                        if sp < 4096 {
                            sp += 1;
                            std::hint::spin_loop();
                        } else {
                            thread::yield_now();
                        }
                    }
                    // SAFETY: all build shares finished and released.
                    unsafe {
                        let tt_ref = &mut **sc.1.get();
                        tt_ref.set_len(n);
                        let r0 = if is_lf {
                            blk.orig_ptr
                        } else {
                            tt_ref[blk.orig_ptr as usize] >> 8
                        };
                        let seg_ref = &mut **sc.2.get();
                        let ws = walk_setup(tt_ref, r0, seg_ref, segments);
                        *sc.0.get() = ws;
                    }
                    walk_ready.store(true, Ordering::Release);
                } else {
                    let mut sp = 0u32;
                    while !walk_ready.load(Ordering::Acquire) {
                        if sp < 4096 {
                            sp += 1;
                            std::hint::spin_loop();
                        } else {
                            thread::yield_now();
                        }
                    }
                }
                // SAFETY: `walk_ready` acquired; `ws` is read-only during walk.
                let ws = unsafe { *sc.0.get() };
                unsafe {
                    if use_staged {
                        traverse_staged(ws, &taken);
                    } else {
                        traverse_unstaged(ws, &taken);
                    }
                }
            });
            // SAFETY: pool.run returned; caller exclusively owns tt & seg.
            let tt_ref = unsafe { &mut **sc.1.get() };
            let seg_ref = unsafe { &mut **sc.2.get() };
            let r0 = if is_lf {
                blk.orig_ptr
            } else {
                (tt_ref[blk.orig_ptr as usize] & LINK) >> 8
            };
            walk_finish(tt_ref, r0, seg_ref);
            if is_lf {
                if use_trusted_and_vec {
                    assemble_reversed_vec(seg_ref, out, k);
                } else {
                    assemble_reversed_scalar(seg_ref, out);
                }
            } else {
                assemble_forward(seg_ref, out);
            }
        } else {
            pool.run(&run_build_share);
            // SAFETY: all shares wrote `[0..n)`.
            unsafe { tt.set_len(n) };
            let r0 = if is_lf {
                blk.orig_ptr
            } else {
                tt[blk.orig_ptr as usize] >> 8
            };
            let ws = walk_setup(tt, r0, seg, segments);
            let taken = AtomicUsize::new(0);
            pool.run(&|_| unsafe {
                if use_staged {
                    traverse_staged(ws, &taken);
                } else {
                    traverse_unstaged(ws, &taken);
                }
            });
            walk_finish(tt, r0, seg);
            if is_lf {
                if use_trusted_and_vec {
                    assemble_reversed_vec(seg, out, k);
                } else {
                    assemble_reversed_scalar(seg, out);
                }
            } else {
                assemble_forward(seg, out);
            }
        }
    }

    // ---- Main probe ----

    pub fn run() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut repeats = 7usize;
        let mut paths = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if args[i] == "--repeats" && i + 1 < args.len() {
                repeats = args[i + 1].parse().unwrap();
                i += 2;
            } else {
                paths.push(args[i].clone());
                i += 1;
            }
        }
        let k = Kernels::detect();
        println!("=== Cooperative Decoder Dissection Probe ===");
        println!(
            "Kernels: avx2={}, clmul={} | repeats={} | cfg(target_feature=avx2)={}",
            k.avx2(),
            k.clmul(),
            repeats,
            cfg!(target_feature = "avx2")
        );

        // 1. Production trigger audit
        println!(
            "\n--- 1. Production Trigger Audit (`Plan::for_threads` / `StreamDecoder::new`) ---"
        );
        println!(
            "Current host Plan::for_threads: 1t={:?}, 2t={:?}, 4t={:?}, 8t={:?}, 12t={:?}",
            Plan::for_threads(1),
            Plan::for_threads(2),
            Plan::for_threads(4),
            Plan::for_threads(8),
            Plan::for_threads(12)
        );
        println!("i9-10900K (phys=10, L3=20MiB) via Machine::service_target():");
        let i9 = mibz2::parallel::Machine::service_target();
        mibz2::parallel::with_machine(i9, || {
            for t in [1, 4, 8, 10, 12, 16, 20] {
                let p = Plan::for_threads(t);
                let dec = StreamDecoder::new(t, t * 4, k);
                println!(
                    "  t={t:2}: plan={p:?}, is_service={}, is_pipelined={}",
                    dec.is_service(),
                    dec.is_pipelined()
                );
            }
        });

        // 2. Pool spawn & barrier micro-benchmark
        println!("\n--- 2. Pool Spawn & Barrier Micro-Benchmark (us) ---");
        for threads in [2usize, 4] {
            let mut spawn_us = Vec::new();
            let mut bar_us = Vec::new();
            for _ in 0..repeats {
                let t0 = Instant::now();
                let pool = CoopPool::new(threads);
                pool.run(&|j| {
                    black_box(j);
                });
                spawn_us.push(t0.elapsed().as_secs_f64() * 1e6);

                // Warm 200 barriers
                for _ in 0..20 {
                    pool.run(&|j| {
                        black_box(j);
                    });
                }
                let t1 = Instant::now();
                for _ in 0..200 {
                    pool.run(&|j| {
                        black_box(j);
                    });
                }
                bar_us.push(t1.elapsed().as_secs_f64() * 1e6 / 200.0);
            }
            let (sm, s0, s1) = median_range(&spawn_us);
            let (bm, b0, b1) = median_range(&bar_us);
            println!(
                "  k={threads}: cold Pool::new + 1st run = {sm:6.1} us [{s0:.1}..{s1:.1}], warm Pool::run barrier = {bm:5.2} us [{b0:.2}..{b1:.2}] (2 barriers/blk = {:.2} us)",
                bm * 2.0
            );
        }

        // Load corpora (4 full blocks each)
        let mut corpora: Vec<(String, Vec<BlockInfo>)> = Vec::new();
        for p in &paths {
            let name = Path::new(p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("corpus")
                .to_string();
            let data = fs::read(p).unwrap_or_else(|e| panic!("read {p}: {e}"));
            let blks = extract_blocks(&data, 4, k);
            if !blks.is_empty() {
                corpora.push((name, blks));
            }
        }
        // Add real local prose+src 4-block corpus
        let prose_raw = build_prose_src_raw(4 * 900_000);
        let prose_bz2 = compress_blocks(&prose_raw);
        let prose_blks = extract_blocks(&prose_bz2, 4, k);
        if !prose_blks.is_empty() {
            corpora.push(("prose_src".to_string(), prose_blks));
        }

        println!("\n--- Corpus Block Census (up to 4 full blocks >= MIN_COOP each) ---");
        for (name, blks) in &corpora {
            let runs_ct = blks.iter().filter(|b| b.takes_runs).count();
            let open_ct = blks.iter().filter(|b| b.open_admits).count();
            let tot_n: usize = blks.iter().map(|b| b.n).sum();
            let tot_syms: usize = blks.iter().map(|b| b.syms.syms.len()).sum();
            println!(
                "  {name:<12}: {} blks, total_rows={tot_n}, syms/row={:.3}, takes_runs={runs_ct}/{}, open_admits={open_ct}/{}",
                blks.len(),
                tot_syms as f64 / tot_n as f64,
                blks.len(),
                blks.len()
            );
        }

        // 3. End-to-End 1-, 2-, 4-Block Stream Comparison (Shipped StreamDecoder::planned)
        println!("\n--- 3. End-to-End Shipped StreamDecoder::planned on 1-, 2-, 4-Block Streams (wall ms / CPU ms, {repeats} repeats) ---");
        let plans = [
            ("W(1)", Plan::Workers(1)),
            ("W(2)", Plan::Workers(2)),
            ("W(4)", Plan::Workers(4)),
            ("T(1x2)", Plan::Teams { teams: 1, k: 2 }),
            ("T(1x4)", Plan::Teams { teams: 1, k: 4 }),
            ("T(2x2)", Plan::Teams { teams: 2, k: 2 }),
        ];
        for (name, blks) in &corpora {
            for &nb in &[1usize, 2, 4] {
                if blks.len() < nb {
                    continue;
                }
                let mut raw = Vec::new();
                for b in &blks[..nb] {
                    raw.extend_from_slice(&b.ref_decoded);
                }
                let bz2 = compress_blocks(&raw);
                let mut wall_map: Vec<Vec<f64>> = vec![Vec::new(); plans.len()];
                let mut cpu_map: Vec<Vec<f64>> = vec![Vec::new(); plans.len()];
                for _ in 0..repeats {
                    for (pi, &(_, plan)) in plans.iter().enumerate() {
                        let (w, c, got) = run_stream_plan(&bz2, plan, k);
                        assert_eq!(got, raw.len());
                        wall_map[pi].push(w);
                        cpu_map[pi].push(c);
                    }
                }
                let (w1, _, _) = median_range(&wall_map[0]);
                print!("  {name:<10} {nb}blk ({:.2}MB):", raw.len() as f64 / 1e6);
                for (pi, &(pname, _)) in plans.iter().enumerate() {
                    let (wm, _, _) = median_range(&wall_map[pi]);
                    let (cm, _, _) = median_range(&cpu_map[pi]);
                    print!("  {pname}={wm:5.2}ms({:.2}x,cpu={cm:5.1})", w1 / wm);
                }
                println!();
            }
        }

        // 4. Exact Phase Breakdown of Cooperative<K> (k = 1, 2, 4) across corpora (4 blocks)
        println!("\n--- 4. Exact Cooperative<K> Phase Breakdown (4 blks, median wall ms, {repeats} repeats) ---");
        for (name, blks) in &corpora {
            let nb = blks.len();
            // Build 4-block bz2 for exact Huffman timing
            let mut raw = Vec::new();
            for b in blks {
                raw.extend_from_slice(&b.ref_decoded);
            }
            let bz2 = compress_blocks(&raw);
            let mut huff_ms = Vec::new();
            let mut syms_recycle = Vec::new();
            for _ in 0..repeats {
                let t0 = Instant::now();
                let mut sc = Scanner::new(SliceSource(&bz2));
                let mut cnt = 0usize;
                loop {
                    match sc.next(syms_recycle).unwrap() {
                        Event::Done => {
                            syms_recycle = Vec::new();
                            break;
                        }
                        Event::Block { symbols, .. } => {
                            black_box(symbols.syms.len());
                            syms_recycle = symbols.syms;
                            cnt += 1;
                        }
                    }
                }
                black_box(cnt);
                huff_ms.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            let (huff_med, _, _) = median_range(&huff_ms);

            // Also measure 1-thread OpenCells inversion+front+rle+crc on the same blocks!
            let mut open_inv_ms = Vec::new();
            let mut open = OpenCells::default();
            let mut r_tmp = Runs::default();
            let mut out_tmp = Vec::new();
            let mut dec_tmp = Vec::new();
            let mut ll_tmp = Vec::new();
            let mut ct_tmp = [0u32; 256];
            for _ in 0..repeats {
                let t0 = Instant::now();
                for b in blks {
                    if b.takes_runs {
                        let n = runs::front(&b.syms, &mut r_tmp, k).unwrap();
                        if !open.invert_runs(&r_tmp, n, b.orig_ptr, &mut out_tmp, k) {
                            let _ = mtf::mtf(&b.syms, &mut ll_tmp, &mut ct_tmp, k).unwrap();
                            open.invert(&ll_tmp, &ct_tmp, b.orig_ptr, &mut out_tmp);
                        }
                    } else {
                        let _ = mtf::mtf(&b.syms, &mut ll_tmp, &mut ct_tmp, k).unwrap();
                        open.invert(&ll_tmp, &ct_tmp, b.orig_ptr, &mut out_tmp);
                    }
                    assert_eq!(out_tmp, b.ref_pre_rle);
                    dec_tmp.clear();
                    rle::unrle(&out_tmp, &mut dec_tmp, k).unwrap();
                    let c = crc::update_with(0, &dec_tmp, k);
                    assert_eq!(c, b.crc);
                }
                open_inv_ms.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            let (open_med, _, _) = median_range(&open_inv_ms);
            println!(
                "\n  Corpus `{name}` ({nb} blks, path={}): Huffman={huff_med:.2}ms | 1t OpenCells(no-huff)={open_med:.2}ms (total={:.2}ms)",
                if blks[0].takes_runs { "Runs" } else { "Column" },
                huff_med + open_med
            );

            for &threads in &[1usize, 2, 4] {
                let pool = CoopPool::new(threads);
                let mut p_front = Vec::new();
                let mut p_cuts = Vec::new();
                let mut p_build = Vec::new();
                let mut p_wsetup = Vec::new();
                let mut p_walk = Vec::new();
                let mut p_asm = Vec::new();
                let mut p_rle_crc = Vec::new();

                let mut tt = Vec::new();
                let mut seg = LocalSegScratch::default();
                let mut cuts = Vec::new();
                let mut out = Vec::new();
                let mut r_obj = Runs::default();
                let mut ll = Vec::new();
                let mut counts = [0u32; 256];
                let mut cps = Vec::new();

                for _ in 0..repeats {
                    let (mut tf, mut tc, mut tb, mut tws, mut tw, mut ta, mut tr) =
                        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
                    for blk in blks {
                        let t0 = Instant::now();
                        if blk.takes_runs {
                            let _ = runs::front(&blk.syms, &mut r_obj, k).unwrap();
                        } else {
                            let _ =
                                mtf::mtf_checkpoints(&blk.syms, &mut ll, &mut counts, &mut cps, k)
                                    .unwrap();
                        }
                        tf += t0.elapsed().as_secs_f64() * 1e3;

                        let t1 = Instant::now();
                        if blk.takes_runs {
                            assert!(make_cuts(
                                r_obj.records(),
                                &blk.runs_counts,
                                blk.n,
                                &mut cuts
                            ));
                        }
                        tc += t1.elapsed().as_secs_f64() * 1e3;

                        let t2 = Instant::now();
                        tt.clear();
                        tt.reserve(blk.n + 8);
                        let slots = Slots(tt.as_mut_ptr());
                        if blk.takes_runs {
                            let recs = r_obj.records();
                            let slots_ref = &slots;
                            pool.run(&|j| {
                                let ptr = slots_ref.0;
                                let (from, to) = (
                                    &cuts[PIECES * j / threads],
                                    &cuts[PIECES * (j + 1) / threads],
                                );
                                unsafe {
                                    #[cfg(target_arch = "x86_64")]
                                    if k.avx2() {
                                        assert!(fill_share_avx2_checked(
                                            &recs[from.rec..to.rec],
                                            from.row,
                                            to.row,
                                            from.next,
                                            &to.next,
                                            ptr
                                        ));
                                    }
                                }
                            });
                        } else {
                            let mut starts = [0u32; 256];
                            let mut cum = 0u32;
                            for (s, &c) in starts.iter_mut().zip(&counts) {
                                *s = cum;
                                cum += c;
                            }
                            pool.run(&|j| {
                                fused_build_psi_share(
                                    &ll, &counts, &cps, &starts, slots, j, threads,
                                );
                            });
                        }
                        unsafe { tt.set_len(blk.n) };
                        tb += t2.elapsed().as_secs_f64() * 1e3;

                        let t3 = Instant::now();
                        let r0 = if blk.takes_runs {
                            blk.orig_ptr
                        } else {
                            tt[blk.orig_ptr as usize] >> 8
                        };
                        let ws = walk_setup(&mut tt, r0, &mut seg, BASE_SEGMENTS * threads);
                        tws += t3.elapsed().as_secs_f64() * 1e3;

                        let t4 = Instant::now();
                        let taken = AtomicUsize::new(0);
                        pool.run(&|_| unsafe { traverse_unstaged(ws, &taken) });
                        tw += t4.elapsed().as_secs_f64() * 1e3;

                        let t5 = Instant::now();
                        walk_finish(&mut tt, r0, &mut seg);
                        if blk.takes_runs {
                            assemble_reversed_scalar(&seg, &mut out);
                        } else {
                            assemble_forward(&seg, &mut out);
                        }
                        ta += t5.elapsed().as_secs_f64() * 1e3;
                        assert_eq!(out, blk.ref_pre_rle);

                        let t6 = Instant::now();
                        dec_tmp.clear();
                        rle::unrle(&out, &mut dec_tmp, k).unwrap();
                        let c = crc::update_with(0, &dec_tmp, k);
                        assert_eq!(c, blk.crc);
                        tr += t6.elapsed().as_secs_f64() * 1e3;
                    }
                    p_front.push(tf);
                    p_cuts.push(tc);
                    p_build.push(tb);
                    p_wsetup.push(tws);
                    p_walk.push(tw);
                    p_asm.push(ta);
                    p_rle_crc.push(tr);
                }
                let mf = median_range(&p_front).0;
                let mc = median_range(&p_cuts).0;
                let mb = median_range(&p_build).0;
                let mws = median_range(&p_wsetup).0;
                let mw = median_range(&p_walk).0;
                let ma = median_range(&p_asm).0;
                let mr = median_range(&p_rle_crc).0;
                let tot = huff_med + mf + mc + mb + mws + mw + ma + mr;
                let serial = huff_med + mf + mc + mws + ma + mr;
                println!(
                    "    k={threads}: tot={tot:5.2}ms | huff={huff_med:4.2}({:4.1}%) front={mf:4.2}({:4.1}%) cuts={mc:4.2}({:4.1}%) build={mb:4.2}({:4.1}%) wset={mws:4.2}({:4.1}%) walk={mw:5.2}({:4.1}%) asm={ma:4.2}({:4.1}%) rle+crc={mr:4.2}({:4.1}%) | serial={:4.1}%",
                    100.0 * huff_med / tot,
                    100.0 * mf / tot,
                    100.0 * mc / tot,
                    100.0 * mb / tot,
                    100.0 * mws / tot,
                    100.0 * mw / tot,
                    100.0 * ma / tot,
                    100.0 * mr / tot,
                    100.0 * serial / tot
                );
            }
        }

        // 5. Candidate Levers on Cooperative<K> (k = 1, 2, 4, 7 interleaved repeats, 100% byte-exact)
        println!("\n--- 5. Candidate Lever Benchmarks on Cooperative<K> (Inversion = cuts+build+walk+asm; plus Column MTF Lever 3) ---");
        let variants = [
            Variant::Shipped,
            Variant::L1StagedWalk,
            Variant::L2LfBuild,
            Variant::L4TrustAsm1Bar,
            Variant::AllCombined,
        ];
        for (name, blks) in &corpora {
            let runs_ct = blks.iter().filter(|b| b.takes_runs).count();
            let is_col = runs_ct < blks.len();
            let route_label = if runs_ct == blks.len() {
                "Runs"
            } else if runs_ct == 0 {
                "Column"
            } else {
                "Mixed(Runs+Col)"
            };
            println!(
                "\n  Corpus `{name}` ({} blks, route={route_label}):",
                blks.len()
            );
            // Measure Lever 3 (register-resident mtf_checkpoints) on Column blocks
            if is_col {
                let mut ship_mtf = Vec::new();
                let mut reg_mtf = Vec::new();
                let mut ll_a = Vec::new();
                let mut ll_b = Vec::new();
                let mut ct_a = [0u32; 256];
                let mut ct_b = [0u32; 256];
                let mut cp_a = Vec::new();
                let mut cp_b = Vec::new();
                for _ in 0..repeats {
                    let t0 = Instant::now();
                    for blk in blks {
                        let na =
                            mtf::mtf_checkpoints(&blk.syms, &mut ll_a, &mut ct_a, &mut cp_a, k)
                                .unwrap();
                        assert_eq!(na, blk.n);
                    }
                    ship_mtf.push(t0.elapsed().as_secs_f64() * 1e3);

                    let t1 = Instant::now();
                    for blk in blks {
                        #[cfg(target_arch = "x86_64")]
                        let nb = unsafe {
                            mtf_checkpoints_reg_avx2(&blk.syms, &mut ll_b, &mut ct_b, &mut cp_b)
                        };
                        #[cfg(not(target_arch = "x86_64"))]
                        let nb =
                            mtf::mtf_checkpoints(&blk.syms, &mut ll_b, &mut ct_b, &mut cp_b, k)
                                .unwrap();
                        assert_eq!(nb, blk.n);
                        assert_eq!(ll_b, blk.ll);
                        assert_eq!(ct_b, blk.counts);
                        assert_eq!(cp_b, blk.checkpoints);
                    }
                    reg_mtf.push(t1.elapsed().as_secs_f64() * 1e3);
                }
                let (sm, s0, s1) = median_range(&ship_mtf);
                let (rm, r0, r1) = median_range(&reg_mtf);
                println!(
                    "    Lever 3 (mtf_checkpoints_reg_avx2 vs shipped PagedList): {rm:.2}ms [{r0:.2}..{r1:.2}] vs {sm:.2}ms [{s0:.2}..{s1:.2}] ({:+.1}%)",
                    100.0 * (rm - sm) / sm
                );
            }

            for &threads in &[1usize, 2, 4] {
                let pool = CoopPool::new(threads);
                let mut times: Vec<Vec<f64>> = vec![Vec::new(); variants.len()];
                let mut cpus: Vec<Vec<f64>> = vec![Vec::new(); variants.len()];
                let mut tt = Vec::new();
                let mut seg = LocalSegScratch::default();
                let mut cuts = Vec::new();
                let mut out = Vec::new();
                for _ in 0..repeats {
                    for (vi, &var) in variants.iter().enumerate() {
                        if !is_col && var == Variant::L2LfBuild {
                            times[vi].push(0.0);
                            cpus[vi].push(0.0);
                            continue;
                        }
                        let t0 = Instant::now();
                        let c0 = process_cpu_ns();
                        for blk in blks {
                            run_coop_invert_variant(
                                blk, &pool, k, var, &mut tt, &mut seg, &mut cuts, &mut out,
                            );
                            assert_eq!(out, blk.ref_pre_rle, "mismatch on {name} {:?}", var.name());
                        }
                        times[vi].push(t0.elapsed().as_secs_f64() * 1e3);
                        cpus[vi].push((process_cpu_ns().saturating_sub(c0)) as f64 / 1e6);
                    }
                }
                let (base_m, base_0, base_1) = median_range(&times[0]);
                let (base_c, _, _) = median_range(&cpus[0]);
                println!(
                    "    k={threads}: shipped_coop={base_m:5.2}ms [{base_0:.2}..{base_1:.2}] (cpu={base_c:5.1}ms)"
                );
                for (vi, &var) in variants.iter().enumerate().skip(1) {
                    if !is_col && var == Variant::L2LfBuild {
                        continue;
                    }
                    let (m, lo, hi) = median_range(&times[vi]);
                    let (cm, _, _) = median_range(&cpus[vi]);
                    let pct = 100.0 * (m - base_m) / base_m;
                    let cpct = 100.0 * (cm - base_c) / base_c;
                    let disj = if hi < base_0 {
                        "DISJOINT-FAST"
                    } else if lo > base_1 {
                        "DISJOINT-SLOW"
                    } else {
                        "overlap"
                    };
                    println!(
                        "         {:<18} = {m:5.2}ms [{lo:.2}..{hi:.2}] ({pct:+5.1}%, {disj}) | cpu={cm:5.1}ms ({cpct:+5.1}%)",
                        var.name()
                    );
                }
            }
        }
    }
}

#[cfg(feature = "moonshot")]
fn main() {
    imp::run();
}

#[cfg(not(feature = "moonshot"))]
fn main() {
    eprintln!("Run with `--features moonshot` to execute coop_dissection_probe.");
}
