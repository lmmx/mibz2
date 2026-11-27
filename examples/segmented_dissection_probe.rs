//! Smooth-zone probe (`segmented-fallback-dissection`, zero `src/` edits):
//! dissects `Segmented`'s exact phase breakdown (front/MTF, T-vector build/fill, segmented walk,
//! segment assembly, RLE1+CRC) across non-wikidata corpora (84% of off-wikidata full blocks) and
//! measures concrete, 100% byte-exact candidate levers on `Segmented` (7 interleaved repeats).
//!
//! Run on baseline x86-64 and `-C target-cpu=native`:
//!     cargo run --release --features moonshot --example segmented_dissection_probe -- /tmp/segmented_corpora/*.bz2
//!     RUSTFLAGS="-C target-cpu=native" cargo run --release --features moonshot --target-dir target/native --example segmented_dissection_probe -- /tmp/segmented_corpora/*.bz2

#[cfg(feature = "moonshot")]
mod imp {
    use mibz2::bwt::{self, InverseBwt, OpenCells, SegScratch, Segmented, TVector, PF_L1, PF_OFF};
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::{crc, mtf, rle, Kernels};
    use std::path::Path;
    use std::time::Instant;

    const MIN_SEGMENTED: usize = bwt::MIN_SEGMENTED;
    const TILE: usize = 256;
    const START32: u32 = 1 << 31;
    const LINK32: u32 = !START32;
    const START64_0: u64 = 1 << 63;
    const START64_1: u64 = 1 << 62;
    const STOP64: u64 = START64_0 | START64_1;

    #[cfg(target_os = "linux")]
    fn thread_cpu_ns() -> u64 {
        #[repr(C)]
        struct Timespec {
            tv_sec: i64,
            tv_nsec: i64,
        }
        extern "C" {
            fn clock_gettime(clk_id: i32, tp: *mut Timespec) -> i32;
        }
        const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
        let mut ts = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: valid pointer to initialized Timespec.
        unsafe {
            clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut ts);
        }
        (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
    }

    #[cfg(not(target_os = "linux"))]
    fn thread_cpu_ns() -> u64 {
        0
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ProdRoute {
        Open,
        RunsFallback,
        Column,
    }

    struct BlockData {
        corpus: String,
        route: ProdRoute,
        n: usize,
        orig_ptr: u32,
        crc_expected: u32,
        ll: Vec<u8>,
        counts: [u32; 256],
        runs: Vec<u16>,
        syms: mibz2::block::Symbols,
        want_inv: Vec<u8>,
    }

    fn random_rows(n: usize, m: usize, rows: &mut Vec<u32>) {
        rows.clear();
        let mut x = 0x2545_f491_4f6c_dd1du64 ^ n as u64;
        rows.extend((0..m).map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x % n as u64) as u32
        }));
    }

    fn repeat_cycle<T: Copy>(out: &mut [T], m: usize) {
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

    #[inline(never)]
    fn assemble_reversed_scalar(
        bufs: &[Vec<u8>],
        next: &[usize],
        first: usize,
        n: usize,
        out: &mut Vec<u8>,
    ) {
        out.clear();
        out.reserve(n);
        let dst = &mut out.spare_capacity_mut()[..n];
        let mut at = n;
        let mut seg = first;
        while at > 0 {
            let b = &bufs[seg];
            let len = b.len().min(at);
            for (d, &x) in dst[at - len..at].iter_mut().rev().zip(&b[..len]) {
                d.write(x);
            }
            at -= len;
            seg = next[seg];
            if seg == first {
                break;
            }
        }
        assert!(at < n || n == 0);
        repeat_cycle(dst, n - at);
        // SAFETY: every byte of dst[..n] was initialized.
        unsafe { out.set_len(n) };
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn reverse_copy_avx2(src: *const u8, dst: *mut u8, len: usize) {
        use std::arch::x86_64::*;
        let rev = _mm256_setr_epi8(
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 15, 14, 13, 12, 11, 10, 9, 8, 7,
            6, 5, 4, 3, 2, 1, 0,
        );
        let mut i = 0usize;
        while i + 32 <= len {
            let v = _mm256_loadu_si256(src.add(i) as *const __m256i);
            let s = _mm256_shuffle_epi8(v, rev);
            let r = _mm256_permute2x128_si256::<0x01>(s, s);
            _mm256_storeu_si256(dst.add(len - i - 32) as *mut __m256i, r);
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

    #[inline(never)]
    fn assemble_reversed_vec(
        bufs: &[Vec<u8>],
        next: &[usize],
        first: usize,
        n: usize,
        out: &mut Vec<u8>,
        k: Kernels,
    ) {
        out.clear();
        out.reserve(n);
        let dst = out.as_mut_ptr();
        let mut at = n;
        let mut seg = first;
        let use_avx2 = k.avx2();
        while at > 0 {
            let b = &bufs[seg];
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
            seg = next[seg];
            if seg == first {
                break;
            }
        }
        assert!(at < n || n == 0);
        // SAFETY: dst[at..n] initialized; repeat_cycle fills dst[..at] when at > 0.
        unsafe {
            let slice = std::slice::from_raw_parts_mut(dst, n);
            repeat_cycle(slice, n - at);
            out.set_len(n);
        }
    }

    // ---- Lever A: 100% sequential LF build on column-route blocks (replaces Ψ scatter) ----

    #[inline(never)]
    fn build_lf_sequential(ll: &[u8], counts: &[u32; 256], tt: &mut Vec<u32>) {
        let n = ll.len();
        tt.clear();
        tt.reserve(n);
        let mut ftab = [0u32; 256];
        let mut cum = 0u32;
        for b in 0..256 {
            ftab[b] = cum;
            cum += counts[b];
        }
        let dst = tt.as_mut_ptr();
        let src = ll.as_ptr();
        let mut i = 0usize;
        // SAFETY: all pointer reads/writes stay within `n <= ll.len()` and `tt.capacity()`.
        unsafe {
            while i + 8 <= n {
                let w = (src.add(i) as *const u64).read_unaligned();
                if w == w.rotate_left(8) {
                    let c = (w & 0xff) as usize;
                    let s = ftab[c];
                    ftab[c] = s + 8;
                    let base = (c as u32) | (s << 8);
                    *dst.add(i) = base;
                    *dst.add(i + 1) = base + 256;
                    *dst.add(i + 2) = base + 512;
                    *dst.add(i + 3) = base + 768;
                    *dst.add(i + 4) = base + 1024;
                    *dst.add(i + 5) = base + 1280;
                    *dst.add(i + 6) = base + 1536;
                    *dst.add(i + 7) = base + 1792;
                    i += 8;
                    continue;
                }
                for off in 0..8 {
                    let c = *src.add(i + off) as usize;
                    let s = ftab[c];
                    ftab[c] = s + 1;
                    *dst.add(i + off) = (c as u32) | (s << 8);
                }
                i += 8;
            }
            while i < n {
                let c = *src.add(i) as usize;
                let s = ftab[c];
                ftab[c] = s + 1;
                *dst.add(i) = (c as u32) | (s << 8);
                i += 1;
            }
            tt.set_len(n);
        }
    }

    // ---- Lever C: `runs::fill_avx2` without redundant per-run tiling checks ----
    // (`Runs` is an opaque struct produced only by `runs::front`, which already enforces exact
    // tiling: 1 <= len <= 127, sum(len) == n, sum_c(len) == counts[c].)

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_runs_checked_avx2(
        runs: &[u16],
        counts: &[u32; 256],
        n: usize,
        tt: &mut Vec<u32>,
    ) {
        use std::arch::x86_64::*;
        tt.clear();
        tt.reserve(n + 8);
        let (mut next, mut lim) = ([0u32; 256], [0u32; 256]);
        let mut cum = 0u32;
        for c in 0..256 {
            next[c] = cum;
            cum += counts[c];
            lim[c] = cum;
        }
        let out = tt.as_mut_ptr();
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut q = 0u32;
        let n32 = n as u32;
        for &x in runs {
            let c = (x >> 8) as usize;
            let len = (x & 0xff) as u32;
            if len == 0 || len > n32 - q || len > lim[c] - next[c] {
                tt.clear();
                return;
            }
            let e = (c as u32) | (next[c] << 8);
            let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
            let (mut w, end) = (q, q + len);
            loop {
                _mm256_storeu_si256(out.add(w as usize) as *mut __m256i, v);
                w += 8;
                if w >= end {
                    break;
                }
                v = _mm256_add_epi32(v, step);
            }
            next[c] += len;
            q = end;
        }
        if q == n32 {
            tt.set_len(n);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fill_runs_trusted_avx2(
        runs: &[u16],
        counts: &[u32; 256],
        n: usize,
        tt: &mut Vec<u32>,
    ) {
        use std::arch::x86_64::*;
        tt.clear();
        tt.reserve(n + 8);
        let mut next = [0u32; 256];
        let mut cum = 0u32;
        for c in 0..256 {
            next[c] = cum;
            cum += counts[c];
        }
        let out = tt.as_mut_ptr();
        let lanes = _mm256_setr_epi32(0, 256, 512, 768, 1024, 1280, 1536, 1792);
        let step = _mm256_set1_epi32(2048);
        let mut q = 0usize;
        for &x in runs {
            let c = (x >> 8) as usize;
            let len = (x & 0xff) as usize;
            let e = (c as u32) | (next[c] << 8);
            let mut v = _mm256_add_epi32(_mm256_set1_epi32(e as i32), lanes);
            let (mut w, end) = (q, q + len);
            loop {
                _mm256_storeu_si256(out.add(w) as *mut __m256i, v);
                w += 8;
                if w >= end {
                    break;
                }
                v = _mm256_add_epi32(v, step);
            }
            next[c] += len as u32;
            q = end;
        }
        tt.set_len(n);
    }

    #[inline(never)]
    fn build_lf_checked(blk: &BlockData, tt: &mut Vec<u32>, k: Kernels) {
        if blk.route == ProdRoute::Column {
            build_lf_sequential(&blk.ll, &blk.counts, tt);
        } else {
            #[cfg(target_arch = "x86_64")]
            if k.avx2() {
                unsafe { fill_runs_checked_avx2(&blk.runs, &blk.counts, blk.n, tt) };
                return;
            }
            let _ = k;
            build_lf_sequential(&blk.ll, &blk.counts, tt);
        }
    }

    #[inline(never)]
    fn build_lf_trusted(blk: &BlockData, tt: &mut Vec<u32>, k: Kernels) {
        if blk.route == ProdRoute::Column {
            build_lf_sequential(&blk.ll, &blk.counts, tt);
        } else {
            #[cfg(target_arch = "x86_64")]
            if k.avx2() {
                unsafe { fill_runs_trusted_avx2(&blk.runs, &blk.counts, blk.n, tt) };
                return;
            }
            let _ = k;
            build_lf_sequential(&blk.ll, &blk.counts, tt);
        }
    }

    #[inline(always)]
    fn prefetch_ptr<const H: u8, T>(t: *const T, row: u32) {
        let p = t.wrapping_add(row as usize);
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            if H != PF_OFF {
                _mm_prefetch::<_MM_HINT_T0>(p as *const i8);
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        let _ = p;
    }

    #[derive(Default)]
    struct WalkScratch {
        starts: Vec<u32>,
        bufs: Vec<Vec<u8>>,
        next: Vec<usize>,
        first: usize,
        n: usize,
    }

    #[derive(Clone, Copy)]
    struct Lane {
        row: u32,
        seg: usize,
        p: *mut u8,
        end: *mut u8,
    }

    #[inline(never)]
    fn room_walk_tiled<const L: usize, const PF: u8>(
        tt: &mut [u32],
        orig_ptr: u32,
        s: &mut WalkScratch,
        segments: usize,
    ) {
        let n = tt.len();
        let r0 = orig_ptr;
        random_rows(n, segments - 1, &mut s.starts);
        s.starts.push(r0);
        s.starts.sort_unstable();
        s.starts.dedup();
        let ns = s.starts.len();
        for &r in &s.starts {
            tt[r as usize] |= START32;
        }
        let cap = 2 * n / ns + 64;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = move |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let t = tt.as_ptr();
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        let take = move |seg: usize, bufs: &mut Vec<Vec<u8>>| -> Lane {
            let b = &mut bufs[seg];
            let base = b.as_mut_ptr();
            let q = unsafe { *t.add(starts[seg] as usize) } & LINK32;
            unsafe { *base = q as u8 };
            Lane {
                row: q >> 8,
                seg,
                p: unsafe { base.add(1) },
                end: unsafe { base.add(b.capacity()) },
            }
        };
        let finish = move |l: &Lane, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
            let b = &mut bufs[l.seg];
            unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
            next[l.seg] = seg_of(l.row);
        };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };

        let mut queued = 0usize;
        let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0usize;
        while active < L && queued < ns {
            lanes[active] = take(queued, bufs);
            if PF != PF_OFF {
                prefetch_ptr::<PF, u32>(t, lanes[active].row);
            }
            queued += 1;
            active += 1;
        }
        if active == L {
            let mut rows = [0u32; L];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; TILE]; L];
            let flush = |l: &mut Lane, row: &[u8; TILE], from: usize, to: usize| {
                let len = to - from;
                unsafe {
                    std::ptr::copy_nonoverlapping(row.as_ptr().add(from), l.p, len);
                    l.p = l.p.add(len);
                }
            };
            'full: loop {
                let mut room = cap - 1;
                for l in lanes.iter_mut() {
                    if l.p == l.end {
                        grow(l, bufs);
                    }
                    room = room.min(unsafe { l.end.offset_from(l.p) } as usize);
                }
                let mut done = 0usize;
                while done < room {
                    let w = (room - done).min(TILE);
                    let mut st = [0usize; L];
                    for j in 0..w {
                        let rs = &mut rows;
                        let mut marked: u64 = 0;
                        let tp = tile.as_mut_ptr().cast::<u8>();
                        macro_rules! sstep {
                            ($k:expr) => {{
                                let k = $k;
                                unsafe {
                                    let r = rs.get_unchecked_mut(k);
                                    let q = *t.add(*r as usize);
                                    if q & START32 != 0 {
                                        marked |= 1 << k;
                                    } else {
                                        *tp.add(k * TILE + j) = q as u8;
                                        *r = q >> 8;
                                        if PF != PF_OFF {
                                            prefetch_ptr::<PF, u32>(t, q >> 8);
                                        }
                                    }
                                }
                            }};
                        }
                        let mut c = 0;
                        while c + 8 <= L {
                            sstep!(c);
                            sstep!(c + 1);
                            sstep!(c + 2);
                            sstep!(c + 3);
                            sstep!(c + 4);
                            sstep!(c + 5);
                            sstep!(c + 6);
                            sstep!(c + 7);
                            c += 8;
                        }
                        while c < L {
                            sstep!(c);
                            c += 1;
                        }
                        while marked != 0 {
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            l.row = rs[k];
                            flush(l, &tile[k], st[k], j);
                            finish(l, bufs, next);
                            if queued == ns {
                                for (i, li) in lanes.iter_mut().enumerate() {
                                    li.row = rs[i];
                                    if i != k {
                                        let waiting = (marked >> i) & 1 != 0;
                                        flush(li, &tile[i], st[i], j + usize::from(!waiting));
                                    }
                                }
                                break 'full;
                            }
                            *l = take(queued, bufs);
                            rs[k] = l.row;
                            if PF != PF_OFF {
                                prefetch_ptr::<PF, u32>(t, l.row);
                            }
                            queued += 1;
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
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| l.seg != usize::MAX && next[l.seg] == usize::MAX)
            .collect();
        for l in live.iter_mut() {
            if l.p == l.end {
                grow(l, bufs);
            }
        }
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                let q = unsafe { *t.add(l.row as usize) };
                if q & START32 != 0 {
                    finish(l, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                l.row = q >> 8;
                if PF != PF_OFF {
                    prefetch_ptr::<PF, u32>(t, l.row);
                }
                unsafe {
                    *l.p = q as u8;
                    l.p = l.p.add(1);
                }
                if l.p == l.end {
                    grow(l, bufs);
                }
                i += 1;
            }
        }
        for &r in starts {
            tt[r as usize] &= LINK32;
        }
        s.first = seg_of(r0);
        s.n = n;
    }

    // ---- Lever D: KKP 2-step fused LF² in u64 (negative-result check) ----

    struct Lf2Scratch {
        ftab2: Box<[[u32; 256]; 256]>,
        ll_tmp: Vec<u8>,
        t2: Vec<u64>,
    }

    impl Default for Lf2Scratch {
        fn default() -> Self {
            Self {
                ftab2: vec![[0u32; 256]; 256]
                    .into_boxed_slice()
                    .try_into()
                    .unwrap(),
                ll_tmp: Vec::new(),
                t2: Vec::new(),
            }
        }
    }

    #[inline(never)]
    fn expand_runs_to_ll(runs: &[u16], n: usize, ll: &mut Vec<u8>) {
        ll.clear();
        ll.reserve(n);
        let dst = ll.as_mut_ptr();
        let mut q = 0usize;
        unsafe {
            for &x in runs {
                let c = (x >> 8) as u8;
                let len = (x & 0xff) as usize;
                if len == 1 {
                    *dst.add(q) = c;
                } else {
                    std::ptr::write_bytes(dst.add(q), c, len);
                }
                q += len;
            }
            ll.set_len(n);
        }
    }

    #[inline(never)]
    fn build_lf2_from_ll(
        ll: &[u8],
        counts: &[u32; 256],
        starts: &[u32],
        ftab2: &mut [[u32; 256]; 256],
        t2: &mut Vec<u64>,
    ) {
        let n = ll.len();
        let mut f_start = [0u32; 257];
        let mut cum = 0u32;
        for c in 0..256 {
            f_start[c] = cum;
            cum += counts[c];
        }
        f_start[256] = cum;
        let mut c2 = [[0u32; 256]; 256];
        for c in 0..256 {
            let s0 = f_start[c] as usize;
            let s1 = f_start[c + 1] as usize;
            let row = &mut c2[c];
            for &b in &ll[s0..s1] {
                row[b as usize] += 1;
            }
        }
        let mut acc = [0u32; 256];
        acc.copy_from_slice(&f_start[..256]);
        for c in 0..256 {
            let src = &c2[c];
            ftab2[c].copy_from_slice(&acc);
            for (a, &s) in acc.iter_mut().zip(src) {
                *a += s;
            }
        }
        let mut next_st_idx = [0usize; 256];
        let mut next_st_val = [u32::MAX; 256];
        let mut si = 0usize;
        for c in 0..256 {
            let (lo, hi) = (f_start[c], f_start[c + 1]);
            while si < starts.len() && starts[si] < lo {
                si += 1;
            }
            next_st_idx[c] = si;
            if si < starts.len() && starts[si] < hi {
                next_st_val[c] = starts[si];
            }
            while si < starts.len() && starts[si] < hi {
                si += 1;
            }
        }
        let mut ftab = [0u32; 256];
        ftab.copy_from_slice(&f_start[..256]);
        t2.clear();
        t2.reserve(n);
        let dst = t2.as_mut_ptr();
        let llp = ll.as_ptr();
        unsafe {
            for i in 0..n {
                let c = *llp.add(i) as usize;
                let s = ftab[c];
                ftab[c] = s + 1;
                let b = *llp.add(s as usize) as usize;
                let up = ftab2.get_unchecked_mut(c).get_unchecked_mut(b);
                let u = *up;
                *up = u + 1;
                *dst.add(i) = if s == next_st_val[c] {
                    let idx = next_st_idx[c] + 1;
                    next_st_idx[c] = idx;
                    next_st_val[c] = if idx < starts.len() && starts[idx] < f_start[c + 1] {
                        starts[idx]
                    } else {
                        u32::MAX
                    };
                    START64_1 | (c as u64) | ((s as u64) << 16)
                } else {
                    (c as u64) | ((b as u64) << 8) | ((u as u64) << 16)
                };
            }
            t2.set_len(n);
        }
        for &r in starts {
            t2[r as usize] |= START64_0;
        }
    }

    #[inline(never)]
    fn room_walk_2step<const L: usize, const PF: u8>(
        t2: &[u64],
        orig_ptr: u32,
        s: &mut WalkScratch,
    ) {
        let n = t2.len();
        let ns = s.starts.len();
        let cap = 2 * n / ns + 128;
        s.bufs.resize_with(ns.max(s.bufs.len()), Vec::new);
        for b in &mut s.bufs[..ns] {
            b.clear();
            b.reserve(cap);
        }
        s.next.clear();
        s.next.resize(ns, usize::MAX);
        let starts = &s.starts;
        let seg_of = move |row: u32| starts.binary_search(&row).unwrap_or(usize::MAX);
        let t = t2.as_ptr();
        let bufs = &mut s.bufs;
        let next = &mut s.next;

        let take_next =
            |queued: &mut usize, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| -> Option<Lane> {
                while *queued < ns {
                    let seg = *queued;
                    *queued += 1;
                    let b = &mut bufs[seg];
                    let base = b.as_mut_ptr();
                    let q = unsafe { *t.add(starts[seg] as usize) };
                    if q & START64_1 != 0 {
                        unsafe {
                            *base = q as u8;
                            b.set_len(1);
                        }
                        next[seg] = seg_of(((q >> 16) & 0xff_ffff) as u32);
                        continue;
                    }
                    unsafe { (base as *mut u16).write_unaligned(q as u16) };
                    return Some(Lane {
                        row: ((q >> 16) & 0xff_ffff) as u32,
                        seg,
                        p: unsafe { base.add(2) },
                        end: unsafe { base.add(b.capacity()) },
                    });
                }
                None
            };
        let finish_at =
            |l: &Lane, stop_row: u32, bufs: &mut Vec<Vec<u8>>, next: &mut Vec<usize>| {
                let b = &mut bufs[l.seg];
                unsafe { b.set_len(l.p.offset_from(b.as_ptr()) as usize) };
                next[l.seg] = seg_of(stop_row);
            };
        let grow = |l: &mut Lane, bufs: &mut Vec<Vec<u8>>| {
            let b = &mut bufs[l.seg];
            let len = unsafe { l.p.offset_from(b.as_ptr()) } as usize;
            unsafe { b.set_len(len) };
            b.reserve(b.capacity());
            l.p = unsafe { b.as_mut_ptr().add(len) };
            l.end = unsafe { b.as_mut_ptr().add(b.capacity()) };
        };
        let mut queued = 0usize;
        let mut lanes: [Lane; L] = std::array::from_fn(|_| Lane {
            row: 0,
            seg: usize::MAX,
            p: std::ptr::null_mut(),
            end: std::ptr::null_mut(),
        });
        let mut active = 0usize;
        while active < L {
            if let Some(l) = take_next(&mut queued, bufs, next) {
                if PF != PF_OFF {
                    prefetch_ptr::<PF, u64>(t, l.row);
                }
                lanes[active] = l;
                active += 1;
            } else {
                break;
            }
        }
        const TILE2: usize = TILE * 2;
        if active == L {
            let mut rows = [0u32; L];
            for (k, l) in lanes.iter().enumerate() {
                rows[k] = l.row;
            }
            let mut tile = [[0u8; TILE2]; L];
            let flush = |l: &mut Lane, row: &[u8; TILE2], from_b: usize, to_b: usize| {
                let len = to_b - from_b;
                unsafe {
                    std::ptr::copy_nonoverlapping(row.as_ptr().add(from_b), l.p, len);
                    l.p = l.p.add(len);
                }
            };
            'full: loop {
                let mut room_steps = (cap - 4) / 2;
                for l in lanes.iter_mut() {
                    if unsafe { l.end.offset_from(l.p) } < 8 {
                        grow(l, bufs);
                    }
                    room_steps =
                        room_steps.min((unsafe { l.end.offset_from(l.p) } as usize - 2) / 2);
                }
                let mut done = 0usize;
                while done < room_steps {
                    let w = (room_steps - done).min(TILE);
                    let mut st_b = [0usize; L];
                    for j in 0..w {
                        let rs = &mut rows;
                        let tp = tile.as_mut_ptr().cast::<u8>();
                        let mut marked: u64 = 0;
                        macro_rules! step2 {
                            ($k:expr) => {{
                                let k = $k;
                                unsafe {
                                    let r = rs.get_unchecked_mut(k);
                                    let q = *t.add(*r as usize);
                                    if q & STOP64 != 0 {
                                        marked |= 1 << k;
                                    } else {
                                        (tp.add(k * TILE2 + j * 2) as *mut u16)
                                            .write_unaligned(q as u16);
                                        let nr = ((q >> 16) & 0xff_ffff) as u32;
                                        *r = nr;
                                        if PF != PF_OFF {
                                            prefetch_ptr::<PF, u64>(t, nr);
                                        }
                                    }
                                }
                            }};
                        }
                        let mut c = 0;
                        while c + 8 <= L {
                            step2!(c);
                            step2!(c + 1);
                            step2!(c + 2);
                            step2!(c + 3);
                            step2!(c + 4);
                            step2!(c + 5);
                            step2!(c + 6);
                            step2!(c + 7);
                            c += 8;
                        }
                        while c < L {
                            step2!(c);
                            c += 1;
                        }
                        while marked != 0 {
                            let k = marked.trailing_zeros() as usize;
                            marked &= marked - 1;
                            let l = &mut lanes[k];
                            let r_cur = rs[k];
                            let q = unsafe { *t.add(r_cur as usize) };
                            if q & START64_0 != 0 {
                                flush(l, &tile[k], st_b[k], j * 2);
                                finish_at(l, r_cur, bufs, next);
                            } else {
                                flush(l, &tile[k], st_b[k], j * 2);
                                unsafe {
                                    *l.p = q as u8;
                                    l.p = l.p.add(1);
                                }
                                finish_at(l, ((q >> 16) & 0xff_ffff) as u32, bufs, next);
                            }
                            if let Some(nl) = take_next(&mut queued, bufs, next) {
                                *l = nl;
                                rs[k] = l.row;
                                if PF != PF_OFF {
                                    prefetch_ptr::<PF, u64>(t, l.row);
                                }
                                st_b[k] = (j + 1) * 2;
                            } else {
                                for (i, li) in lanes.iter_mut().enumerate() {
                                    li.row = rs[i];
                                    if i != k {
                                        let waiting = (marked >> i) & 1 != 0;
                                        flush(
                                            li,
                                            &tile[i],
                                            st_b[i],
                                            if waiting { j * 2 } else { (j + 1) * 2 },
                                        );
                                    }
                                }
                                break 'full;
                            }
                        }
                    }
                    for (k, l) in lanes.iter_mut().enumerate() {
                        flush(l, &tile[k], st_b[k], w * 2);
                    }
                    done += w;
                }
            }
        }
        let mut live: Vec<Lane> = lanes[..active]
            .iter()
            .copied()
            .filter(|l| l.seg != usize::MAX && next[l.seg] == usize::MAX)
            .collect();
        while !live.is_empty() {
            let mut i = 0;
            while i < live.len() {
                let l = &mut live[i];
                if unsafe { l.end.offset_from(l.p) } < 4 {
                    grow(l, bufs);
                }
                let q = unsafe { *t.add(l.row as usize) };
                if q & START64_0 != 0 {
                    finish_at(l, l.row, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                if q & START64_1 != 0 {
                    unsafe {
                        *l.p = q as u8;
                        l.p = l.p.add(1);
                    }
                    finish_at(l, ((q >> 16) & 0xff_ffff) as u32, bufs, next);
                    live.swap_remove(i);
                    continue;
                }
                unsafe {
                    (l.p as *mut u16).write_unaligned(q as u16);
                    l.p = l.p.add(2);
                }
                l.row = ((q >> 16) & 0xff_ffff) as u32;
                if PF != PF_OFF {
                    prefetch_ptr::<PF, u64>(t, l.row);
                }
                i += 1;
            }
        }
        s.first = seg_of(orig_ptr);
        s.n = n;
    }

    fn extract_blocks(path: &Path, k: Kernels) -> (Vec<BlockData>, usize, usize, usize) {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let data = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let want_whole = mibz2::decode(&data).expect("shipped mibz2::decode failed");

        let mut sc = Scanner::new(SliceSource(&data));
        let mut syms = Vec::new();
        let mut open = OpenCells::default();
        let mut seg = Segmented::default();
        let mut runs_buf = Runs::default();
        let mut out_blocks = Vec::new();
        let mut assembled_whole = Vec::new();
        let (mut n_open, mut n_rfall, mut n_col) = (0usize, 0usize, 0usize);

        loop {
            match sc.next(syms).expect("scan failed") {
                Event::Done => break,
                Event::Block { symbols, .. } => {
                    let orig_ptr = symbols.orig_ptr;
                    let crc_expected = symbols.crc;
                    let syms_clone = mibz2::block::Symbols {
                        syms: symbols.syms.clone(),
                        seq_to_unseq: symbols.seq_to_unseq,
                        eob: symbols.eob,
                        orig_ptr: symbols.orig_ptr,
                        crc: symbols.crc,
                        max_len: symbols.max_len,
                    };
                    let mut inv_out = Vec::new();
                    let route;
                    let mut ll = Vec::new();
                    let mut counts = [0u32; 256];
                    let mut runs_vec = Vec::new();
                    let n;

                    if runs::takes_runs(&symbols) {
                        n = runs::front(&symbols, &mut runs_buf, k).expect("runs::front");
                        let rec = runs_buf.records();
                        let open_admits = if n <= 900_000
                            && (orig_ptr as usize) < n
                            && rec.len() * 64 >= n
                            && rec.len() * 25 < n * 7
                        {
                            if rec.len() * 20 < n * 3 {
                                true
                            } else {
                                let (mut m, mut cur, mut len) = (0usize, u16::MAX, 0usize);
                                for &x in rec {
                                    let (b, l) = (x >> 8, (x & 0xff) as usize);
                                    if b == cur {
                                        len += l;
                                    } else {
                                        if len > 0 && len <= 16 {
                                            m += len;
                                        }
                                        cur = b;
                                        len = l;
                                    }
                                }
                                if len > 0 && len <= 16 {
                                    m += len;
                                }
                                m < 256_000usize.min(n * 3 / 10)
                            }
                        } else {
                            false
                        };
                        assert!(seg.invert_runs(&runs_buf, n, orig_ptr, &mut inv_out, k));
                        if open_admits {
                            let mut open_out = Vec::new();
                            assert!(open.invert_runs(&runs_buf, n, orig_ptr, &mut open_out, k));
                            assert_eq!(open_out, inv_out);
                            route = ProdRoute::Open;
                            if n >= 100_000 {
                                n_open += 1;
                            }
                        } else {
                            route = ProdRoute::RunsFallback;
                            if n >= 100_000 {
                                n_rfall += 1;
                            }
                            runs_vec = rec.to_vec();
                            for &x in &runs_vec {
                                counts[(x >> 8) as usize] += (x & 0xff) as u32;
                            }
                        }
                    } else {
                        n = mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf");
                        seg.invert(&ll, &counts, orig_ptr, &mut inv_out);
                        route = ProdRoute::Column;
                        if n >= 100_000 {
                            n_col += 1;
                        }
                    }

                    let mut dec = Vec::new();
                    rle::unrle(&inv_out, &mut dec, k).expect("unrle");
                    assert_eq!(crc::crc32(&dec, k), crc_expected, "{name}: CRC mismatch");
                    assembled_whole.extend_from_slice(&dec);

                    if route != ProdRoute::Open && n >= MIN_SEGMENTED {
                        out_blocks.push(BlockData {
                            corpus: name.clone(),
                            route,
                            n,
                            orig_ptr,
                            crc_expected,
                            ll,
                            counts,
                            runs: runs_vec,
                            syms: syms_clone,
                            want_inv: inv_out,
                        });
                    }
                    syms = symbols.syms;
                }
            }
        }
        assert_eq!(assembled_whole, want_whole, "{name}: whole decode mismatch");
        (out_blocks, n_open, n_rfall, n_col)
    }

    #[derive(Clone, Copy)]
    enum Arm {
        Shipped,
        AColLfOnly,
        BColLfVecRev,
        CTrustedFillVecRev,
        DLf2U64,
    }

    impl Arm {
        const ALL: [Arm; 5] = [
            Arm::Shipped,
            Arm::AColLfOnly,
            Arm::BColLfVecRev,
            Arm::CTrustedFillVecRev,
            Arm::DLf2U64,
        ];
        fn name(self) -> &'static str {
            match self {
                Arm::Shipped => "shipped",
                Arm::AColLfOnly => "A_col_lf",
                Arm::BColLfVecRev => "B_col_lf+avx2rev",
                Arm::CTrustedFillVecRev => "C_trustfill+rev",
                Arm::DLf2U64 => "D_lf2_u64",
            }
        }
    }

    #[derive(Default)]
    struct SharedScratch {
        tt: TVector,
        seg: SegScratch,
        tt_raw: Vec<u32>,
        walk: WalkScratch,
        lf2: Lf2Scratch,
        out: Vec<u8>,
    }

    #[derive(Default, Clone, Copy)]
    struct PhaseTotals {
        build_ns: u128,
        walk_ns: u128,
        asm_ns: u128,
        cpu_ns: u64,
    }
    impl PhaseTotals {
        fn total_ms(self) -> f64 {
            (self.build_ns + self.walk_ns + self.asm_ns) as f64 / 1e6
        }
        fn cpu_ms(self) -> f64 {
            self.cpu_ns as f64 / 1e6
        }
    }

    fn run_arm(
        arm: Arm,
        blk: &BlockData,
        prebuilt_runs: Option<&Runs>,
        st: &mut SharedScratch,
        k: Kernels,
        verify: bool,
    ) -> PhaseTotals {
        let c0 = thread_cpu_ns();
        let (tb, tw, ta) = match arm {
            Arm::Shipped => {
                if blk.route == ProdRoute::Column {
                    let t0 = Instant::now();
                    bwt::build(&blk.ll, &blk.counts, &mut st.tt);
                    let tb = t0.elapsed().as_nanos();
                    let t1 = Instant::now();
                    let segd = bwt::walk_segments_room_parts(
                        &mut st.tt,
                        blk.orig_ptr,
                        &mut st.out,
                        &mut st.seg,
                    );
                    let tw = t1.elapsed().as_nanos();
                    let t2 = Instant::now();
                    if segd {
                        st.seg.assemble(&mut st.out);
                    }
                    let ta = t2.elapsed().as_nanos();
                    (tb, tw, ta)
                } else {
                    let r = prebuilt_runs.expect("prebuilt runs");
                    let t0 = Instant::now();
                    assert!(runs::fill(r, blk.n, &mut st.tt, k));
                    let tb = t0.elapsed().as_nanos();
                    let t1 = Instant::now();
                    bwt::walk_back_room(&mut st.tt, blk.orig_ptr, &mut st.out, &mut st.seg);
                    let tw = t1.elapsed().as_nanos();
                    (tb, tw, 0)
                }
            }
            Arm::AColLfOnly => {
                let t0 = Instant::now();
                build_lf_checked(blk, &mut st.tt_raw, k);
                let tb = t0.elapsed().as_nanos();
                let t1 = Instant::now();
                room_walk_tiled::<32, PF_L1>(&mut st.tt_raw, blk.orig_ptr, &mut st.walk, 512);
                let tw = t1.elapsed().as_nanos();
                let t2 = Instant::now();
                assemble_reversed_scalar(
                    &st.walk.bufs,
                    &st.walk.next,
                    st.walk.first,
                    st.walk.n,
                    &mut st.out,
                );
                let ta = t2.elapsed().as_nanos();
                (tb, tw, ta)
            }
            Arm::BColLfVecRev => {
                let t0 = Instant::now();
                build_lf_checked(blk, &mut st.tt_raw, k);
                let tb = t0.elapsed().as_nanos();
                let t1 = Instant::now();
                room_walk_tiled::<32, PF_L1>(&mut st.tt_raw, blk.orig_ptr, &mut st.walk, 512);
                let tw = t1.elapsed().as_nanos();
                let t2 = Instant::now();
                assemble_reversed_vec(
                    &st.walk.bufs,
                    &st.walk.next,
                    st.walk.first,
                    st.walk.n,
                    &mut st.out,
                    k,
                );
                let ta = t2.elapsed().as_nanos();
                (tb, tw, ta)
            }
            Arm::CTrustedFillVecRev => {
                let t0 = Instant::now();
                build_lf_trusted(blk, &mut st.tt_raw, k);
                let tb = t0.elapsed().as_nanos();
                let t1 = Instant::now();
                room_walk_tiled::<32, PF_L1>(&mut st.tt_raw, blk.orig_ptr, &mut st.walk, 512);
                let tw = t1.elapsed().as_nanos();
                let t2 = Instant::now();
                assemble_reversed_vec(
                    &st.walk.bufs,
                    &st.walk.next,
                    st.walk.first,
                    st.walk.n,
                    &mut st.out,
                    k,
                );
                let ta = t2.elapsed().as_nanos();
                (tb, tw, ta)
            }
            Arm::DLf2U64 => {
                let t0 = Instant::now();
                random_rows(blk.n, 511, &mut st.walk.starts);
                st.walk.starts.push(blk.orig_ptr);
                st.walk.starts.sort_unstable();
                st.walk.starts.dedup();
                if blk.route == ProdRoute::Column {
                    build_lf2_from_ll(
                        &blk.ll,
                        &blk.counts,
                        &st.walk.starts,
                        &mut st.lf2.ftab2,
                        &mut st.lf2.t2,
                    );
                } else {
                    expand_runs_to_ll(&blk.runs, blk.n, &mut st.lf2.ll_tmp);
                    build_lf2_from_ll(
                        &st.lf2.ll_tmp,
                        &blk.counts,
                        &st.walk.starts,
                        &mut st.lf2.ftab2,
                        &mut st.lf2.t2,
                    );
                }
                let tb = t0.elapsed().as_nanos();
                let t1 = Instant::now();
                room_walk_2step::<32, PF_L1>(&st.lf2.t2, blk.orig_ptr, &mut st.walk);
                let tw = t1.elapsed().as_nanos();
                let t2 = Instant::now();
                assemble_reversed_vec(
                    &st.walk.bufs,
                    &st.walk.next,
                    st.walk.first,
                    st.walk.n,
                    &mut st.out,
                    k,
                );
                let ta = t2.elapsed().as_nanos();
                (tb, tw, ta)
            }
        };
        let cpu_ns = thread_cpu_ns().saturating_sub(c0);
        if verify {
            assert_eq!(
                st.out,
                blk.want_inv,
                "byte mismatch: arm={} corpus={} route={:?} n={}",
                arm.name(),
                blk.corpus,
                blk.route,
                blk.n
            );
        }
        PhaseTotals {
            build_ns: tb,
            walk_ns: tw,
            asm_ns: ta,
            cpu_ns,
        }
    }

    pub fn main() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        assert!(
            !args.is_empty(),
            "usage: segmented_dissection_probe FILE.bz2..."
        );
        let k = Kernels::detect();
        println!(
            "segmented_dissection_probe: kernels={k:?} target_avx2={} repeats=7",
            cfg!(target_feature = "avx2")
        );

        let mut all_blocks = Vec::new();
        println!("\n## 1. Route census (full blocks n >= 100,000)");
        println!(
            "{:<10} {:>6} {:>6} {:>10} {:>8} {:>10}",
            "corpus", "open", "r_fall", "column", "seg_tot", "seg_share"
        );
        for arg in &args {
            let p = Path::new(arg);
            let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or(arg);
            let (blks, n_open, n_rfall, n_col) = extract_blocks(p, k);
            let tot = n_open + n_rfall + n_col;
            let seg_tot = n_rfall + n_col;
            println!(
                "{:<10} {:>6} {:>6} {:>10} {:>8} {:>9.1}%",
                name,
                n_open,
                n_rfall,
                n_col,
                seg_tot,
                if tot > 0 {
                    100.0 * seg_tot as f64 / tot as f64
                } else {
                    0.0
                }
            );
            all_blocks.extend(blks);
        }

        let prebuilt_runs: Vec<Option<Runs>> = all_blocks
            .iter()
            .map(|blk| {
                if blk.route == ProdRoute::RunsFallback {
                    let mut r = Runs::default();
                    let n = runs::front(&blk.syms, &mut r, k).unwrap();
                    assert_eq!(n, blk.n);
                    Some(r)
                } else {
                    None
                }
            })
            .collect();

        // Worker phase dissection (best of 3 thread-CPU ms, verifying CRC on every block).
        println!("\n## 2. Full worker phase breakdown on Segmented blocks (best of 3 CPU ms)");
        let mut front_ns = std::collections::BTreeMap::<String, u64>::new();
        let mut inv_build_ns = std::collections::BTreeMap::<String, u64>::new();
        let mut inv_walk_ns = std::collections::BTreeMap::<String, u64>::new();
        let mut rle_crc_ns = std::collections::BTreeMap::<String, u64>::new();
        {
            let mut st = SharedScratch::default();
            let mut tmp_ll = Vec::new();
            let mut tmp_counts = [0u32; 256];
            let mut tmp_runs = Runs::default();
            let mut rle_buf = Vec::new();
            for (bi, blk) in all_blocks.iter().enumerate() {
                let (mut bf, mut bb, mut bw, mut br) = (u64::MAX, u64::MAX, u64::MAX, u64::MAX);
                for _ in 0..3 {
                    let t0 = thread_cpu_ns();
                    if blk.route == ProdRoute::Column {
                        let _ = mtf::mtf(&blk.syms, &mut tmp_ll, &mut tmp_counts, k).unwrap();
                    } else {
                        let _ = runs::front(&blk.syms, &mut tmp_runs, k).unwrap();
                    }
                    bf = bf.min(thread_cpu_ns().saturating_sub(t0));

                    let t1 = thread_cpu_ns();
                    if blk.route == ProdRoute::Column {
                        bwt::build(&blk.ll, &blk.counts, &mut st.tt);
                    } else {
                        assert!(runs::fill(
                            prebuilt_runs[bi].as_ref().unwrap(),
                            blk.n,
                            &mut st.tt,
                            k
                        ));
                    }
                    let t2 = thread_cpu_ns();
                    bb = bb.min(t2.saturating_sub(t1));

                    if blk.route == ProdRoute::Column {
                        bwt::walk_segments_room(&mut st.tt, blk.orig_ptr, &mut st.out, &mut st.seg);
                    } else {
                        bwt::walk_back_room(&mut st.tt, blk.orig_ptr, &mut st.out, &mut st.seg);
                    }
                    bw = bw.min(thread_cpu_ns().saturating_sub(t2));

                    rle_buf.clear();
                    let t3 = thread_cpu_ns();
                    rle::unrle(&blk.want_inv, &mut rle_buf, k).unwrap();
                    assert_eq!(crc::crc32(&rle_buf, k), blk.crc_expected);
                    br = br.min(thread_cpu_ns().saturating_sub(t3));
                }
                *front_ns.entry(blk.corpus.clone()).or_default() += bf;
                *inv_build_ns.entry(blk.corpus.clone()).or_default() += bb;
                *inv_walk_ns.entry(blk.corpus.clone()).or_default() += bw;
                *rle_crc_ns.entry(blk.corpus.clone()).or_default() += br;
            }
        }
        println!(
            "{:<10} {:>9} {:>9} {:>10} {:>9} {:>10} {:>9}",
            "corpus", "front_ms", "build_ms", "walk+a_ms", "rle+c_ms", "inv_share", "bld/inv"
        );
        for c in front_ns.keys() {
            let f = front_ns[c] as f64 / 1e6;
            let b = inv_build_ns[c] as f64 / 1e6;
            let w = inv_walk_ns[c] as f64 / 1e6;
            let r = rle_crc_ns[c] as f64 / 1e6;
            let tot = f + b + w + r;
            println!(
                "{:<10} {:>9.2} {:>9.2} {:>10.2} {:>9.2} {:>9.1}% {:>8.1}%",
                c,
                f,
                b,
                w,
                r,
                100.0 * (b + w) / tot,
                100.0 * b / (b + w)
            );
        }

        // Exactness verification pass + warmup across ALL arms on EVERY Segmented block.
        let mut scratch = SharedScratch::default();
        for &arm in &Arm::ALL {
            for (bi, blk) in all_blocks.iter().enumerate() {
                run_arm(arm, blk, prebuilt_runs[bi].as_ref(), &mut scratch, k, true);
            }
        }
        println!(
            "\nExactness verified: {}/{} Segmented blocks x {} arms = 100% byte-identical!",
            all_blocks.len(),
            all_blocks.len(),
            Arm::ALL.len()
        );

        const REPEATS: usize = 7;
        let corpora: Vec<String> = front_ns.keys().cloned().collect();

        let mut rep_pts_wall = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_pts_cpu = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_pts_bld = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_pts_wlk = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_pts_asm = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_rfall_wall = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_rfall_cpu = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_col_wall = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_col_cpu = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_all_wall = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_all_cpu = vec![vec![0.0f64; REPEATS]; Arm::ALL.len()];
        let mut rep_corp_cpu = vec![vec![vec![0.0f64; REPEATS]; corpora.len()]; Arm::ALL.len()];

        for r in 0..REPEATS {
            for offset in 0..Arm::ALL.len() {
                let ai = (offset + r) % Arm::ALL.len();
                let arm = Arm::ALL[ai];
                for (bi, blk) in all_blocks.iter().enumerate() {
                    let pt = run_arm(arm, blk, prebuilt_runs[bi].as_ref(), &mut scratch, k, false);
                    let w = pt.total_ms();
                    let cpu = pt.cpu_ms();
                    rep_all_wall[ai][r] += w;
                    rep_all_cpu[ai][r] += cpu;
                    if blk.route == ProdRoute::RunsFallback {
                        rep_rfall_wall[ai][r] += w;
                        rep_rfall_cpu[ai][r] += cpu;
                    } else {
                        rep_col_wall[ai][r] += w;
                        rep_col_cpu[ai][r] += cpu;
                    }
                    if matches!(blk.corpus.as_str(), "text" | "prose" | "pysrc" | "rustsrc") {
                        rep_pts_wall[ai][r] += w;
                        rep_pts_cpu[ai][r] += cpu;
                        rep_pts_bld[ai][r] += pt.build_ns as f64 / 1e6;
                        rep_pts_wlk[ai][r] += pt.walk_ns as f64 / 1e6;
                        rep_pts_asm[ai][r] += pt.asm_ns as f64 / 1e6;
                    }
                    let ci = corpora.iter().position(|x| x == &blk.corpus).unwrap();
                    rep_corp_cpu[ai][ci][r] += cpu;
                }
            }
        }

        let stats = |v: &[f64]| -> (f64, f64, f64) {
            let mut s = v.to_vec();
            s.sort_by(|a, b| a.total_cmp(b));
            (s[s.len() / 2], s[0], s[s.len() - 1])
        };
        let dtag = |ai: usize, lo: f64, hi: f64, blo: f64, bhi: f64| {
            if ai == 0 {
                "-"
            } else if hi < blo {
                "FASTER"
            } else if lo > bhi {
                "SLOWER"
            } else {
                "OVERLAP"
            }
        };

        let (bw_med, bw_lo, bw_hi) = stats(&rep_pts_wall[0]);
        let (bc_med, bc_lo, bc_hi) = stats(&rep_pts_cpu[0]);

        println!("\n## 3. Primary gate (WALL ms): prose/text + source (7 interleaved repeats)");
        println!(
            "{:<17} {:>8} {:>19} {:>8} {:>8} {:>7} {:>8} {:>8}",
            "arm", "med_ms", "[min..max]", "bld_ms", "wlk_ms", "asm_ms", "delta%", "disjoint"
        );
        for (ai, &arm) in Arm::ALL.iter().enumerate() {
            let (m, lo, hi) = stats(&rep_pts_wall[ai]);
            let (bm, _, _) = stats(&rep_pts_bld[ai]);
            let (wm, _, _) = stats(&rep_pts_wlk[ai]);
            let (am, _, _) = stats(&rep_pts_asm[ai]);
            println!(
                "{:<17} {:>8.2} [{:>7.2}..{:>7.2}] {:>8.2} {:>8.2} {:>7.2} {:>+7.2}% {:>8}",
                arm.name(),
                m,
                lo,
                hi,
                bm,
                wm,
                am,
                (m / bw_med - 1.0) * 100.0,
                dtag(ai, lo, hi, bw_lo, bw_hi)
            );
        }

        println!(
            "\n## 4. Primary gate (THREAD CPU ms, steal-free diagnostic): prose/text + source"
        );
        println!(
            "{:<17} {:>8} {:>19} {:>8} {:>8}",
            "arm", "cpu_ms", "[min..max]", "delta%", "disjoint"
        );
        for (ai, &arm) in Arm::ALL.iter().enumerate() {
            let (m, lo, hi) = stats(&rep_pts_cpu[ai]);
            println!(
                "{:<17} {:>8.2} [{:>7.2}..{:>7.2}] {:>+7.2}% {:>8}",
                arm.name(),
                m,
                lo,
                hi,
                (m / bc_med - 1.0) * 100.0,
                dtag(ai, lo, hi, bc_lo, bc_hi)
            );
        }

        let (br_c, br_lo, br_hi) = stats(&rep_rfall_cpu[0]);
        let (bk_c, bk_lo, bk_hi) = stats(&rep_col_cpu[0]);
        let (ba_c, ba_lo, ba_hi) = stats(&rep_all_cpu[0]);
        let (br_w, _, _) = stats(&rep_rfall_wall[0]);
        let (bk_w, _, _) = stats(&rep_col_wall[0]);
        let (ba_w, _, _) = stats(&rep_all_wall[0]);

        println!("\n## 5. Route split & all-6 summary (THREAD CPU ms [disjoint] / WALL delta%)");
        println!(
            "{:<17} {:>8} {:>8} {:>7} {:>8} {:>8} {:>7} {:>8} {:>8} {:>7}  (wall rfall/col/all6)",
            "arm", "rfall_c", "delta%", "d", "col_c", "delta%", "d", "all6_c", "delta%", "d"
        );
        for (ai, &arm) in Arm::ALL.iter().enumerate() {
            let (rm, rlo, rhi) = stats(&rep_rfall_cpu[ai]);
            let (cm, clo, chi) = stats(&rep_col_cpu[ai]);
            let (am, alo, ahi) = stats(&rep_all_cpu[ai]);
            let (rmw, _, _) = stats(&rep_rfall_wall[ai]);
            let (cmw, _, _) = stats(&rep_col_wall[ai]);
            let (amw, _, _) = stats(&rep_all_wall[ai]);
            println!(
                "{:<17} {:>8.2} {:>+7.2}% {:>7} {:>8.2} {:>+7.2}% {:>7} {:>8.2} {:>+7.2}% {:>7}  ({:+.1}%/{:+.1}%/{:+.1}%)",
                arm.name(),
                rm, (rm / br_c - 1.0) * 100.0, dtag(ai, rlo, rhi, br_lo, br_hi),
                cm, (cm / bk_c - 1.0) * 100.0, dtag(ai, clo, chi, bk_lo, bk_hi),
                am, (am / ba_c - 1.0) * 100.0, dtag(ai, alo, ahi, ba_lo, ba_hi),
                (rmw / br_w - 1.0) * 100.0,
                (cmw / bk_w - 1.0) * 100.0,
                (amw / ba_w - 1.0) * 100.0,
            );
        }

        println!("\n## 6. Per-corpus breakdown (THREAD CPU median ms and delta% vs shipped)");
        print!("{:<17}", "arm");
        for c in &corpora {
            print!(" {:>15}", c);
        }
        println!();
        let base_c: Vec<f64> = (0..corpora.len())
            .map(|ci| stats(&rep_corp_cpu[0][ci]).0)
            .collect();
        for (ai, &arm) in Arm::ALL.iter().enumerate() {
            print!("{:<17}", arm.name());
            for ci in 0..corpora.len() {
                let (m, _, _) = stats(&rep_corp_cpu[ai][ci]);
                let p = (m / base_c[ci] - 1.0) * 100.0;
                print!(" {:>7.2}({:>+5.1}%)", m, p);
            }
            println!();
        }
    }
}

#[cfg(feature = "moonshot")]
fn main() {
    imp::main();
}

#[cfg(not(feature = "moonshot"))]
fn main() {
    eprintln!("segmented_dissection_probe requires --features moonshot");
}
