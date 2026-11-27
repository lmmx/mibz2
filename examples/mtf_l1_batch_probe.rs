//! (research only, smooth-zone validation probe, zero `src/` edits)
//!
//! Validates PR #1048's L1d-batched rescue hypothesis for `mtf-fold` (`#1010`):
//! decoupling `front_avx2` from `Pass::take + OutQ16` via a 1 KiB L1d-resident
//! `[u32; 256]` packed maximal-run batch `(c << 24) | total` plus an `OutQ16`
//! power-of-2 guard zone (`CHUNK=496/STRIDE=512` and `CHUNK=1008/STRIDE=1024`).
//!
//! Usage:
//!   cargo run --release --example mtf_l1_batch_probe -- corpora/wikidata.bz2 [REPEATS]

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("mtf_l1_batch_probe requires x86_64 with AVX2/BMI1/BMI2/LZCNT/POPCNT");
}

#[cfg(target_arch = "x86_64")]
fn main() {
    x86_probe::run();
}

#[cfg(target_arch = "x86_64")]
mod x86_probe {
    use mibz2::block::Symbols;
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::Kernels;
    use std::arch::x86_64::*;
    use std::fs::File;
    use std::hint::black_box;
    use std::os::unix::fs::FileExt;
    use std::time::Instant;

    const CAP: u32 = 127;
    const LIMIT: u32 = 16;
    const INLINE: usize = 16;
    const NONE_CHUNK: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;
    const MAX_PACKED_RUN: u64 = (1 << 24) - 1;

    const LAZY_DENSITY: (usize, usize) = (3, 20);
    const DENSE: (usize, usize) = (7, 25);
    const DENSE_M: u32 = 256_000;
    const DENSE_STRIPS: usize = 16_384;

    // Reference i9-10900K numbers from `frontier/surface.tsv` & `mtf-fold-l1-batch-redesign.toml`:
    // A5 + A6a = 332.7 ms at 1W (1,306 ms wall); 47.5 ms/w at 8W (184.8 ms/w busy).
    // Whole-worker gate: >= 2.5% of 1W wall = >= 32.65 ms => >= 9.81% of A5 + A6a.
    const I9_A5_A6A_1W_MS: f64 = 332.7;
    const I9_WALL_1W_MS: f64 = 1306.0;
    const I9_A5_A6A_8W_MS: f64 = 47.5;
    const I9_BUSY_8W_MS: f64 = 184.8;

    // ---- Safe Linux on-CPU thread clock via /proc/thread-self/schedstat ----

    struct CpuClock {
        file: Option<File>,
        buf: [u8; 128],
    }

    impl CpuClock {
        fn new() -> Self {
            CpuClock {
                file: File::open("/proc/thread-self/schedstat").ok(),
                buf: [0u8; 128],
            }
        }

        fn now_ns(&mut self) -> Option<u64> {
            let f = self.file.as_ref()?;
            let n = f.read_at(&mut self.buf, 0).ok()?;
            let s = std::str::from_utf8(&self.buf[..n]).ok()?;
            s.split_whitespace().next()?.parse::<u64>().ok()
        }
    }

    // ---- Production admission mirror (src/cells_lazy.rs + src/open.rs) ----

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Cap {
        m: u32,
        strips: usize,
    }

    impl Cap {
        const NONE: Cap = Cap {
            m: u32::MAX,
            strips: usize::MAX,
        };
        fn dense(n: usize) -> Cap {
            Cap {
                m: DENSE_M.min((n as u64 * 3 / 10) as u32),
                strips: DENSE_STRIPS,
            }
        }
    }

    fn lazy_counts_admit(records: usize, n: usize) -> bool {
        n >= 65_536 && records * 64 >= n && records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0
    }

    fn open_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if lazy_counts_admit(records, n) {
            Some(Cap::NONE)
        } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }

    fn within_cap(records: &[u16], p: u32, limit: u32, cap: Cap) -> bool {
        let (mut pos, mut m, mut ns) = (0u32, 0u32, 0usize);
        let mut i = 0;
        while i < records.len() {
            let rec = records[i];
            let c = rec >> 8;
            let mut width = u32::from(rec & 0xFF);
            i += 1;
            while i < records.len() && (records[i] >> 8) == c {
                width += u32::from(records[i] & 0xFF);
                i += 1;
            }
            if width <= limit {
                m += width;
            } else if p.wrapping_sub(pos) >= width {
                ns += 1;
            } else {
                m += 1;
                ns += usize::from(pos < p) + usize::from(p + 1 < pos + width);
            }
            if m > cap.m || ns > cap.strips {
                return false;
            }
            pos += width;
        }
        true
    }

    // ---- Safe OutQ16 parameterized by (CHUNK, STRIDE) — zero `unsafe` ----

    struct Spill<const STRIDE: usize> {
        next: Vec<u32>,
        ovf: Vec<Box<[u8]>>,
        cuts: Vec<[u32; 256]>,
        push_slow_calls: u64,
        tail_guard_calls: u64,
        chunks_taken: u64,
        ovf_chunks: u64,
    }

    impl<const STRIDE: usize> Default for Spill<STRIDE> {
        fn default() -> Self {
            Spill {
                next: Vec::new(),
                ovf: Vec::new(),
                cuts: Vec::new(),
                push_slow_calls: 0,
                tail_guard_calls: 0,
                chunks_taken: 0,
                ovf_chunks: 0,
            }
        }
    }

    struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    struct Queues<'a, const CHUNK: usize, const STRIDE: usize> {
        out: &'a mut [u8],
        spill: &'a mut Spill<STRIDE>,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        tails: [u32; 256],
        fill: [u32; 256],
        limit: u32,
        over: bool,
    }

    impl<'a, const CHUNK: usize, const STRIDE: usize> Queues<'a, CHUNK, STRIDE> {
        const BYTES: usize = 2 * STRIDE;
        const HAS_GUARD: bool = CHUNK + 16 <= STRIDE;

        fn new(out: &'a mut [u8], spill: &'a mut Spill<STRIDE>, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            Queues {
                in_out: (out.len() / Self::BYTES) as u32,
                out,
                spill,
                chunks: 0,
                heads: [NONE_CHUNK; 256],
                tails: [NONE_CHUNK; 256],
                fill: [CHUNK as u32; 256],
                limit,
                over: false,
            }
        }

        fn heads(&self) -> Heads {
            Heads {
                heads: self.heads,
                in_out: self.in_out,
            }
        }

        #[inline(always)]
        fn push(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            let f = self.fill[a] as usize;
            let lo = m & (EPOCH - 1);
            let tail = self.tails[a];
            let room_ok = if Self::HAS_GUARD {
                f + (w as usize) <= CHUNK
            } else {
                f + 16 <= CHUNK
            };
            if w <= 16 && room_ok && lo + w < EPOCH && tail < self.in_out {
                let at = tail as usize * Self::BYTES + 2 * f;
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                self.fill[a] = (f as u32) + w;
            } else {
                self.probe_outq_push_slow(a, m, w, thin);
            }
        }

        #[cold]
        #[inline(never)]
        fn probe_outq_push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
            self.spill.push_slow_calls += 1;
            let f0 = self.fill[a] as usize;
            let lo0 = m & (EPOCH - 1);
            if !Self::HAS_GUARD
                && w <= 16
                && f0 < CHUNK
                && f0 + (w as usize) <= CHUNK
                && f0 + 16 > CHUNK
                && lo0 + w < EPOCH
                && self.tails[a] < self.in_out
            {
                self.spill.tail_guard_calls += 1;
            }
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                if self.fill[a] as usize == CHUNK {
                    self.take_chunk(a);
                }
                let f = self.fill[a] as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(self.tails[a]);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                self.fill[a] += take;
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let mut cut = *thin;
                    cut[a] += w - left;
                    self.spill.cuts.push(cut);
                }
            }
        }

        fn take_chunk(&mut self, a: usize) {
            let id = self.chunks;
            self.chunks += 1;
            self.spill.chunks_taken += 1;
            if id >= self.in_out {
                self.spill.ovf_chunks += 1;
                if self.spill.ovf.len() <= (id - self.in_out) as usize {
                    self.spill
                        .ovf
                        .push(vec![0u8; Self::BYTES].into_boxed_slice());
                }
            }
            if self.spill.next.len() <= id as usize {
                self.spill.next.resize(id as usize + 1, NONE_CHUNK);
            }
            self.spill.next[id as usize] = NONE_CHUNK;
            if self.heads[a] == NONE_CHUNK {
                self.heads[a] = id;
            } else {
                self.spill.next[self.tails[a] as usize] = id;
            }
            self.tails[a] = id;
            self.fill[a] = 0;
        }

        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * Self::BYTES;
                &mut self.out[at..at + Self::BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    struct Reader<'a, const CHUNK: usize, const STRIDE: usize> {
        out: &'a [u8],
        spill: &'a Spill<STRIDE>,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read: u32,
        epoch: u32,
        cut: u32,
    }

    impl<'a, const CHUNK: usize, const STRIDE: usize> Reader<'a, CHUNK, STRIDE> {
        const BYTES: usize = 2 * STRIDE;

        fn new(out: &'a [u8], spill: &'a Spill<STRIDE>, heads: &Heads, a: usize) -> Self {
            Reader {
                out,
                spill,
                in_out: heads.in_out,
                a,
                id: heads.heads[a],
                at: 0,
                read: 0,
                epoch: 0,
                cut: spill.cuts.first().map_or(u32::MAX, |c| c[a]),
            }
        }

        fn read_all(&mut self, total: u32, dst: &mut Vec<u32>) {
            dst.clear();
            let mut left = total as usize;
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * Self::BYTES;
                    &self.out[at..at + Self::BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for s in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2) {
                    dst.push(hi | u32::from(u16::from_le_bytes([s[0], s[1]])));
                }
                self.at += take;
                self.read += take as u32;
                left -= take;
            }
        }

        /// Mirrors A6c sequential queue readback (`wire` / `wire_xor` read side) to check
        /// whether `CHUNK/STRIDE` affects A6c queue-read cost.
        #[inline(never)]
        fn checksum_all(&mut self, total: u32) -> u64 {
            let mut acc = 0u64;
            let mut left = total as usize;
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == CHUNK {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * Self::BYTES;
                    &self.out[at..at + Self::BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for s in chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2) {
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    acc = acc.wrapping_add(u64::from(v));
                }
                self.at += take;
                self.read += take as u32;
                left -= take;
            }
            acc
        }
    }

    // ---- Safe First + Pass (exact mirror of src/open.rs, zero `unsafe`) ----

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    struct First {
        thick_rows: [u32; 256],
        thin: [u32; 256],
        thick: [u32; 256],
        m: u32,
        mark: u32,
        runs: usize,
    }

    impl Default for First {
        fn default() -> Self {
            First {
                thick_rows: [0; 256],
                thin: [0; 256],
                thick: [0; 256],
                m: 0,
                mark: 0,
                runs: 0,
            }
        }
    }

    #[inline(always)]
    fn split(
        c: u8,
        pos: u32,
        width: u32,
        p: u32,
        limit: u32,
        mut visit: impl FnMut(u8, u32, u32, bool),
    ) {
        let end = pos + width;
        if width > limit && (pos..end).contains(&p) {
            if pos < p {
                visit(c, pos, p - pos, false);
            }
            visit(c, p, 1, true);
            if p + 1 < end {
                visit(c, p + 1, end - p - 1, false);
            }
        } else {
            visit(c, pos, width, width <= limit);
        }
    }

    struct Pass<'a, const CHUNK: usize, const STRIDE: usize> {
        queues: Queues<'a, CHUNK, STRIDE>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl<'a, const CHUNK: usize, const STRIDE: usize> Pass<'a, CHUNK, STRIDE> {
        fn new(
            out: &'a mut [u8],
            spill: &'a mut Spill<STRIDE>,
            edges: &'a mut Vec<u32>,
            limit: u32,
            cap_strips: usize,
        ) -> Self {
            Pass {
                queues: Queues::new(out, spill, limit),
                edges,
                ns: 0,
                cap_strips,
                f: First::default(),
            }
        }

        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
            self.f.runs += 1;
            if width <= limit {
                self.piece(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece(c, pos, width, false, p);
            } else {
                self.take_marked(c, pos, width, p, limit);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32, limit: u32) {
            split(c, pos, width, p, limit, |c, x, w, keep| {
                self.piece(c, x, w, keep, p)
            });
        }

        #[inline(always)]
        fn piece(&mut self, c: u8, x: u32, w: u32, keep: bool, p: u32) {
            let a = usize::from(c);
            let f = &mut self.f;
            if keep {
                if p.wrapping_sub(x) < w {
                    f.mark = f.m + (p - x);
                }
                self.queues.push(a, f.m, w, &f.thin);
                f.thin[a] += w;
                f.m += w;
            } else {
                if self.ns < self.cap_strips {
                    let at = 3 * self.ns;
                    if self.edges.len() < at + 3 {
                        let need = (at + 3)
                            .max(2 * self.edges.len())
                            .max(1024)
                            .min(self.cap_strips.saturating_mul(3));
                        self.edges.reserve_exact(need - self.edges.len());
                        self.edges.resize(need, 0);
                    }
                    self.edges[at..at + 3].copy_from_slice(&[
                        x,
                        f.thin[a] + f.thick_rows[a],
                        (w << 8) | u32::from(c),
                    ]);
                }
                self.ns += 1;
                f.thick[a] += 1;
                f.thick_rows[a] += w;
            }
        }
    }

    // ---- AVX2 MTF lookup tables & second() (faithful copy of src/runs.rs) ----

    #[repr(align(64))]
    #[derive(Clone, Copy)]
    struct Step {
        a: [u8; 16],
        b: [u8; 16],
        c: [u8; 16],
        d: [u8; 16],
    }

    const fn steps() -> [Step; 32] {
        const Z: u8 = 0x80;
        let mut t = [Step {
            a: [0; 16],
            b: [Z; 16],
            c: [0; 16],
            d: [Z; 16],
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
                    t[nn].c[i] = i as u8;
                } else {
                    t[nn].a[i] = if i == 0 { Z } else { (i - 1) as u8 };
                    t[nn].c[i] = if i == 0 {
                        Z
                    } else if i <= nn - 16 {
                        (i - 1) as u8
                    } else {
                        i as u8
                    };
                    if i == 0 {
                        t[nn].b[i] = (nn - 16) as u8;
                        t[nn].d[i] = 15;
                    }
                }
                i += 1;
            }
            nn += 1;
        }
        t
    }

    static STEPS: [Step; 32] = steps();

    #[inline(always)]
    unsafe fn second(tail: *mut u8, t: usize, f1: __m128i, idx: __m256i) -> u8 {
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
        } else {
            let tv = _mm256_set1_epi8(t as u8 as i8);
            let r: [__m256i; 7] = std::array::from_fn(row);
            for j in 0..7 {
                let p = _mm256_add_epi8(idx, _mm256_set1_epi8((32 * j) as u8 as i8));
                let m = _mm256_cmpeq_epi8(_mm256_min_epu8(p, tv), p);
                let below = if j == 0 { below0 } else { r[j - 1] };
                let new = _mm256_blendv_epi8(r[j], on(r[j], below), m);
                _mm256_store_si256(tail.add(32 * j) as *mut __m256i, new);
            }
        }
        v
    }

    #[cold]
    #[inline(never)]
    #[target_feature(enable = "avx2")]
    unsafe fn second_cold(tail: *mut u8, t: usize, f1: __m128i) -> u8 {
        let idx = _mm256_setr_epi8(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        );
        second(tail, t, f1, idx)
    }

    struct FrontScratch {
        lit: Vec<u64>,
        one: Vec<u64>,
        runs: Vec<u16>,
        counts: [u32; 256],
        per: [u32; 256],
    }

    impl Default for FrontScratch {
        fn default() -> Self {
            FrontScratch {
                lit: Vec::new(),
                one: Vec::new(),
                runs: Vec::new(),
                counts: [0; 256],
                per: [0; 256],
            }
        }
    }

    // ---- Arm 1: Shipped split (`src/runs.rs::front_avx2` + `src/open.rs::maximal` + OutQ16 512/512) ----

    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn probe_front_avx2_shipped(b: &Symbols, f: &mut FrontScratch) -> Option<usize> {
        let m = b.syms.len();
        let sp = b.syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return None;
        }
        let words = m / 64 + 3;
        f.lit.clear();
        f.lit.reserve(words);
        f.one.clear();
        f.one.reserve(words);
        let lit32 = f.lit.as_mut_ptr() as *mut u32;
        let one32 = f.one.as_mut_ptr() as *mut u32;
        let v1 = _mm256_set1_epi16(1);
        let v0 = _mm256_setzero_si256();
        let ve = _mm256_set1_epi16(b.eob as i16);
        macro_rules! bits {
            ($x:expr, $y:expr) => {
                _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                    as u32
            };
        }
        let mut end = usize::MAX;
        let mut ci = 0usize;
        while end == usize::MAX && ci * 32 + 32 <= m {
            let x = _mm256_loadu_si256(sp.add(ci * 32) as *const __m256i);
            let y = _mm256_loadu_si256(sp.add(ci * 32 + 16) as *const __m256i);
            let run = bits!(
                _mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
                _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y)
            );
            let runa = bits!(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
            let eob = bits!(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & !run;
            let (mut l, mut o) = (!run, !runa);
            if eob != 0 {
                let t = eob.trailing_zeros();
                end = ci * 32 + t as usize;
                let keep = ((2u64 << t) - 1) as u32;
                l &= keep >> 1;
                o &= keep;
            }
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        if end == usize::MAX {
            let (mut l, mut o) = (0u32, 0u32);
            for (t, &s) in b.syms[ci * 32..].iter().enumerate() {
                o |= ((s != 0) as u32) << t;
                if s > 1 {
                    if s == b.eob {
                        end = ci * 32 + t;
                        break;
                    }
                    l |= 1 << t;
                }
            }
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        for i in ci..2 * words {
            *lit32.add(i) = 0;
            *one32.add(i) = 0;
        }
        f.lit.set_len(words);
        f.one.set_len(words);
        if end == usize::MAX {
            return None;
        }

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
        let mut slack = b.max_len / CAP as usize + 1;
        f.runs.clear();
        f.runs.reserve(m + slack + 2);
        let out0 = f.runs.as_mut_ptr();
        let mut out = out0;
        let mut cnt = [0u64; 256];
        let mut big = 0u64;
        let litp = f.lit.as_ptr();
        let onep = f.one.as_ptr();
        macro_rules! long {
            ($c:expr, $total:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                let recs = total.div_ceil(CAP as u64);
                let more = recs.saturating_sub(1) as usize;
                big += total;
                if more > slack || big > b.max_len as u64 {
                    return None;
                }
                slack -= more;
                cnt[c as usize] += total | (recs << 32);
                let mut left = total;
                while left > 0 {
                    let len = left.min(CAP as u64);
                    *out = (c << 8 | len) as u16;
                    out = out.add(1);
                    left -= len;
                }
            }};
        }
        macro_rules! emit {
            ($c:expr, $total:expr, $span:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                if total > CAP as u64 {
                    if $span > 22 {
                        return None;
                    }
                    long!(c, total);
                } else {
                    *(out as *mut u8) = total as u8;
                    *(out as *mut u8).add(1) = c as u8;
                    out = out.add(1);
                    cnt[c as usize] += total | (1 << 32);
                }
            }};
        }
        let last = end / 64;
        let (mut wi, mut w) = (0usize, *litp);
        macro_rules! seek {
            () => {{
                loop {
                    if w != 0 {
                        let t = w.trailing_zeros() as usize;
                        w &= w - 1;
                        break Some(t);
                    }
                    if wi == last {
                        break None;
                    }
                    wi += 1;
                    w = *litp.add(wi);
                }
            }};
        }
        macro_rules! count {
            ($q:expr, $q2:expr) => {{
                let from = $q + 1;
                let bits = ((onep as *const u8).add(from >> 3) as *const u64).read_unaligned()
                    >> (from & 7);
                _bzhi_u64(bits, ($q2 - $q) as u32)
            }};
        }
        let first = seek!();
        let q = first.map_or(end, |t| wi * 64 + t);
        if q > 21 {
            return None;
        }
        long!(*s2u.as_ptr() as u64, _bzhi_u64(*onep, q as u32 + 1) - 1);
        if let Some(mut tz) = first {
            let mut after = *onep.add(wi) >> 1;
            let mut spw = sp.add(wi * 64);
            loop {
                let s = *spw.add(tz) as usize;
                if s < 33 {
                    let t = steps.add(s - 1);
                    let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
                    let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
                    let cc = _mm_load_si128((*t).c.as_ptr() as *const __m128i);
                    let dd = _mm_load_si128((*t).d.as_ptr() as *const __m128i);
                    let g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, bb));
                    f1 = _mm_or_si128(_mm_shuffle_epi8(f1, cc), _mm_shuffle_epi8(f0, dd));
                    f0 = g0;
                } else {
                    let v = second(tail, (s - 1).min(255) - 32, f1, idx);
                    f1 = _mm_alignr_epi8::<15>(f1, f0);
                    f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
                }
                let c = _mm_cvtsi128_si32(f0) as u8 as u64;
                if w != 0 {
                    let t2 = w.trailing_zeros() as usize;
                    w &= w - 1;
                    emit!(c, _bzhi_u64(after, t2 as u32) >> tz, t2 - tz);
                    tz = t2;
                } else {
                    let q = wi * 64 + tz;
                    let next = seek!();
                    let q2 = next.map_or(end, |t| wi * 64 + t);
                    if q2 - q > 22 {
                        return None;
                    }
                    emit!(c, count!(q, q2), q2 - q);
                    match next {
                        Some(t) => (tz, after, spw) = (t, *onep.add(wi) >> 1, sp.add(wi * 64)),
                        None => break,
                    }
                }
            }
        }
        let n: u64 = cnt.iter().map(|&x| x & 0xffff_ffff).sum();
        if n > b.max_len as u64 {
            return None;
        }
        f.runs.set_len(out.offset_from(out0) as usize);
        for (c, &x) in cnt.iter().enumerate() {
            f.counts[c] = x as u32;
            f.per[c] = (x >> 32) as u32;
        }
        Some(n as usize)
    }

    #[inline(never)]
    fn probe_a6a_from_records<const CHUNK: usize, const STRIDE: usize>(
        records: &[u16],
        p: u32,
        out: &mut [u8],
        spill: &mut Spill<STRIDE>,
        edges: &mut Vec<u32>,
        limit_m: u32,
        cap_strips: usize,
    ) -> (First, Heads, usize, bool) {
        let mut pass = Pass::<CHUNK, STRIDE>::new(out, spill, edges, limit_m, cap_strips);
        let mut pos = 0u32;
        let mut i = 0usize;
        while i < records.len() {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 0xFF);
            i += 1;
            while i < records.len() && (records[i] >> 8) as u8 == c {
                width += u32::from(records[i] & 0xFF);
                i += 1;
            }
            pass.take(c, pos, width, p, LIMIT);
            pos += width;
        }
        let heads = pass.queues.heads();
        let over = pass.queues.over;
        (pass.f, heads, pass.ns, over)
    }

    // ---- Arm 2: Direct fold `#1010` (`src/runs/fold.rs::front_avx2` + `FoldSink`) ----

    struct FoldSink<'a> {
        pass: Pass<'a, 512, 512>,
        p: u32,
        c: u8,
        pos: u32,
        width: u32,
    }

    impl FoldSink<'_> {
        #[inline(always)]
        fn flush(&mut self) {
            if self.width != 0 {
                self.pass.take(self.c, self.pos, self.width, self.p, LIMIT);
                self.pos += self.width;
                self.width = 0;
            }
        }

        #[inline(always)]
        fn run(&mut self, c: u8, total: u32) {
            if c == self.c && self.width != 0 {
                self.width += total;
                return;
            }
            self.flush();
            self.c = c;
            self.width = total;
        }
    }

    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn probe_front_avx2_fold_1010(
        b: &Symbols,
        f: &mut FrontScratch,
        sink: &mut FoldSink<'_>,
        lits_refused: usize,
    ) -> Option<(usize, usize)> {
        let m = b.syms.len();
        let sp = b.syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return None;
        }
        let words = m / 64 + 3;
        f.lit.clear();
        f.lit.reserve(words);
        f.one.clear();
        f.one.reserve(words);
        let lit32 = f.lit.as_mut_ptr() as *mut u32;
        let one32 = f.one.as_mut_ptr() as *mut u32;
        let v1 = _mm256_set1_epi16(1);
        let v0 = _mm256_setzero_si256();
        let ve = _mm256_set1_epi16(b.eob as i16);
        macro_rules! bits {
            ($x:expr, $y:expr) => {
                _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                    as u32
            };
        }
        let mut end = usize::MAX;
        let mut ci = 0usize;
        while end == usize::MAX && ci * 32 + 32 <= m {
            let x = _mm256_loadu_si256(sp.add(ci * 32) as *const __m256i);
            let y = _mm256_loadu_si256(sp.add(ci * 32 + 16) as *const __m256i);
            let run = bits!(
                _mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
                _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y)
            );
            let runa = bits!(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
            let eob = bits!(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & !run;
            let (mut l, mut o) = (!run, !runa);
            if eob != 0 {
                let t = eob.trailing_zeros();
                end = ci * 32 + t as usize;
                let keep = ((2u64 << t) - 1) as u32;
                l &= keep >> 1;
                o &= keep;
            }
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        if end == usize::MAX {
            let (mut l, mut o) = (0u32, 0u32);
            for (t, &s) in b.syms[ci * 32..].iter().enumerate() {
                o |= ((s != 0) as u32) << t;
                if s > 1 {
                    if s == b.eob {
                        end = ci * 32 + t;
                        break;
                    }
                    l |= 1 << t;
                }
            }
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        for i in ci..2 * words {
            *lit32.add(i) = 0;
            *one32.add(i) = 0;
        }
        f.lit.set_len(words);
        f.one.set_len(words);
        if end == usize::MAX {
            return None;
        }
        if lits_refused < m {
            let lits: usize = f.lit.iter().map(|w| w.count_ones() as usize).sum();
            if lits >= lits_refused {
                return None;
            }
        }

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
        let mut slack = b.max_len / CAP as usize + 1;
        let (mut n, mut recs, mut big) = (0u64, 0usize, 0u64);
        let litp = f.lit.as_ptr();
        let onep = f.one.as_ptr();
        macro_rules! long {
            ($c:expr, $total:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                let r = total.div_ceil(CAP as u64);
                let more = r.saturating_sub(1) as usize;
                big += total;
                if more > slack || big > b.max_len as u64 {
                    return None;
                }
                slack -= more;
                recs += r as usize;
                n += total;
                if total != 0 {
                    sink.run(c as u8, total as u32);
                }
            }};
        }
        macro_rules! emit {
            ($c:expr, $total:expr, $span:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                if total > CAP as u64 {
                    if $span > 22 {
                        return None;
                    }
                    long!(c, total);
                } else {
                    recs += 1;
                    n += total;
                    sink.run(c as u8, total as u32);
                }
            }};
        }
        let last = end / 64;
        let (mut wi, mut w) = (0usize, *litp);
        macro_rules! seek {
            () => {{
                loop {
                    if w != 0 {
                        let t = w.trailing_zeros() as usize;
                        w &= w - 1;
                        break Some(t);
                    }
                    if wi == last {
                        break None;
                    }
                    wi += 1;
                    w = *litp.add(wi);
                }
            }};
        }
        macro_rules! count {
            ($q:expr, $q2:expr) => {{
                let from = $q + 1;
                let bits = ((onep as *const u8).add(from >> 3) as *const u64).read_unaligned()
                    >> (from & 7);
                _bzhi_u64(bits, ($q2 - $q) as u32)
            }};
        }
        let first = seek!();
        let q = first.map_or(end, |t| wi * 64 + t);
        if q > 21 {
            return None;
        }
        long!(*s2u.as_ptr() as u64, _bzhi_u64(*onep, q as u32 + 1) - 1);
        if let Some(mut tz) = first {
            let mut after = *onep.add(wi) >> 1;
            let mut spw = sp.add(wi * 64);
            loop {
                let s = *spw.add(tz) as usize;
                if s < 33 {
                    let t = steps.add(s - 1);
                    let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
                    let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
                    let cc = _mm_load_si128((*t).c.as_ptr() as *const __m128i);
                    let dd = _mm_load_si128((*t).d.as_ptr() as *const __m128i);
                    let g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, bb));
                    f1 = _mm_or_si128(_mm_shuffle_epi8(f1, cc), _mm_shuffle_epi8(f0, dd));
                    f0 = g0;
                } else {
                    let v = second(tail, (s - 1).min(255) - 32, f1, idx);
                    f1 = _mm_alignr_epi8::<15>(f1, f0);
                    f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
                }
                let c = _mm_cvtsi128_si32(f0) as u8 as u64;
                if w != 0 {
                    let t2 = w.trailing_zeros() as usize;
                    w &= w - 1;
                    emit!(c, _bzhi_u64(after, t2 as u32) >> tz, t2 - tz);
                    tz = t2;
                } else {
                    let q = wi * 64 + tz;
                    let next = seek!();
                    let q2 = next.map_or(end, |t| wi * 64 + t);
                    if q2 - q > 22 {
                        return None;
                    }
                    emit!(c, count!(q, q2), q2 - q);
                    match next {
                        Some(t) => (tz, after, spw) = (t, *onep.add(wi) >> 1, sp.add(wi * 64)),
                        None => break,
                    }
                }
            }
        }
        if n > b.max_len as u64 {
            return None;
        }
        Some((n as usize, recs))
    }

    // ---- Arms 3-5 (+ batch sweep): L1d-batched packed maximal MTF fold + OutQ16 guard zone ----

    #[repr(align(64))]
    struct MtfRows([u8; 224]);

    #[repr(align(64))]
    struct BatchState {
        front: [u8; 32],
        rows: MtfRows,
        wi: usize,
        w: u64,
        tz: usize,
        last: usize,
        end: usize,
        slack: usize,
        big: u64,
        extra_recs: usize,
        done: bool,
        huge: Option<(u8, u64)>,
    }

    fn s2u_injective(s2u: &[u8; 256], used: usize) -> bool {
        let mut seen = [0u64; 4];
        for &b in &s2u[..used] {
            let word = (b >> 6) as usize;
            let bit = 1u64 << (b & 63);
            if seen[word] & bit != 0 {
                return false;
            }
            seen[word] |= bit;
        }
        true
    }

    /// Pass 1: build `lit` and `one` bitmaps, count `lits` in-flight (free `base_recs` + early
    /// `lits_refused` check), and track `max_sym` before `eob` so `s2u_injective` is exact even
    /// for synthetic `Symbols` with arbitrary symbols.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn probe_build_bitmaps_avx2(
        b: &Symbols,
        f: &mut FrontScratch,
        lits_refused: usize,
    ) -> Option<(usize, usize, u16)> {
        let m = b.syms.len();
        let sp = b.syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return None;
        }
        let words = m / 64 + 3;
        f.lit.clear();
        f.lit.reserve(words);
        f.one.clear();
        f.one.reserve(words);
        let lit32 = f.lit.as_mut_ptr() as *mut u32;
        let one32 = f.one.as_mut_ptr() as *mut u32;
        let v1 = _mm256_set1_epi16(1);
        let v0 = _mm256_setzero_si256();
        let ve = _mm256_set1_epi16(b.eob as i16);
        let mut vmax = v0;
        macro_rules! bits {
            ($x:expr, $y:expr) => {
                _mm256_movemask_epi8(_mm256_permute4x64_epi64::<0xd8>(_mm256_packs_epi16($x, $y)))
                    as u32
            };
        }
        let mut end = usize::MAX;
        let mut lits = 0usize;
        let mut max_sym = 0u16;
        let mut ci = 0usize;
        while end == usize::MAX && ci * 32 + 32 <= m {
            let x = _mm256_loadu_si256(sp.add(ci * 32) as *const __m256i);
            let y = _mm256_loadu_si256(sp.add(ci * 32 + 16) as *const __m256i);
            let run = bits!(
                _mm256_cmpeq_epi16(_mm256_min_epu16(x, v1), x),
                _mm256_cmpeq_epi16(_mm256_min_epu16(y, v1), y)
            );
            let runa = bits!(_mm256_cmpeq_epi16(x, v0), _mm256_cmpeq_epi16(y, v0));
            let eob = bits!(_mm256_cmpeq_epi16(x, ve), _mm256_cmpeq_epi16(y, ve)) & !run;
            let (mut l, mut o) = (!run, !runa);
            if eob != 0 {
                let t = eob.trailing_zeros();
                end = ci * 32 + t as usize;
                let keep = ((2u64 << t) - 1) as u32;
                l &= keep >> 1;
                o &= keep;
                for &s in &b.syms[ci * 32..end] {
                    max_sym = max_sym.max(s);
                }
            } else {
                vmax = _mm256_max_epu16(vmax, _mm256_max_epu16(x, y));
            }
            lits += l.count_ones() as usize;
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        if end == usize::MAX {
            let (mut l, mut o) = (0u32, 0u32);
            for (t, &s) in b.syms[ci * 32..].iter().enumerate() {
                o |= ((s != 0) as u32) << t;
                if s > 1 {
                    if s == b.eob {
                        end = ci * 32 + t;
                        break;
                    }
                    l |= 1 << t;
                }
                max_sym = max_sym.max(s);
            }
            lits += l.count_ones() as usize;
            *lit32.add(ci) = l;
            *one32.add(ci) = o;
            ci += 1;
        }
        for i in ci..2 * words {
            *lit32.add(i) = 0;
            *one32.add(i) = 0;
        }
        f.lit.set_len(words);
        f.one.set_len(words);
        if end == usize::MAX {
            return None;
        }
        if lits_refused < m && lits >= lits_refused {
            return None;
        }
        // Horizontal max of vmax (16 lanes of u16)
        let hi = _mm256_extracti128_si256::<1>(vmax);
        let lo = _mm256_castsi256_si128(vmax);
        let m8 = _mm_max_epu16(lo, hi);
        let m4 = _mm_max_epu16(m8, _mm_srli_si128::<8>(m8));
        let m2 = _mm_max_epu16(m4, _mm_srli_si128::<4>(m4));
        let m1 = _mm_max_epu16(m2, _mm_srli_si128::<2>(m2));
        max_sym = max_sym.max(_mm_cvtsi128_si32(m1) as u16);
        Some((end, lits, max_sym))
    }

    /// Pass 2 (pure 128-bit XMM hot loop!): emits up to `BATCH` packed `(c << 24) | total`
    /// runs into stack `[u32; BATCH]`. `f0`/`f1` live permanently in XMM registers on the 98.5%
    /// `s < 33` fast path; zero `FoldSink`, zero `cnt[256]`, zero `CAP=127` writes, zero YMM
    /// loads or `vzeroupper` in the hot loop.
    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn probe_fill_batch_avx2<const BATCH: usize>(
        st: &mut BatchState,
        sp: *const u16,
        litp: *const u64,
        onep: *const u64,
        max_len: u64,
        batch: &mut [u32; BATCH],
    ) -> Option<usize> {
        let mut f0 = _mm_load_si128(st.front.as_ptr() as *const __m128i);
        let mut f1 = _mm_load_si128(st.front.as_ptr().add(16) as *const __m128i);
        let tail = st.rows.0.as_mut_ptr();
        let steps = STEPS.as_ptr();

        let mut wi = st.wi;
        let mut w = st.w;
        let mut tz = st.tz;
        let last = st.last;
        let end = st.end;
        let mut after = *onep.add(wi) >> 1;
        let mut spw = sp.add(wi * 64);

        let out0 = batch.as_mut_ptr();
        let mut out = out0;
        let out_end = out0.add(BATCH);

        macro_rules! seek {
            () => {{
                loop {
                    if w != 0 {
                        let t = w.trailing_zeros() as usize;
                        w &= w - 1;
                        break Some(t);
                    }
                    if wi == last {
                        break None;
                    }
                    wi += 1;
                    w = *litp.add(wi);
                }
            }};
        }
        macro_rules! count {
            ($q:expr, $q2:expr) => {{
                let from = $q + 1;
                let bits = ((onep as *const u8).add(from >> 3) as *const u64).read_unaligned()
                    >> (from & 7);
                _bzhi_u64(bits, ($q2 - $q) as u32)
            }};
        }
        macro_rules! emit_long {
            ($c:expr, $total:expr, $span:expr, $finished:expr) => {{
                if $span > 22 {
                    return None;
                }
                let r = $total.div_ceil(CAP as u64);
                let more = r.saturating_sub(1) as usize;
                st.big += $total;
                if more > st.slack || st.big > max_len {
                    return None;
                }
                st.slack -= more;
                st.extra_recs += more;
                if $total <= MAX_PACKED_RUN {
                    *out = (($c << 24) | $total) as u32;
                    out = out.add(1);
                } else {
                    st.huge = Some(($c as u8, $total));
                    st.done = $finished;
                    break;
                }
            }};
        }

        loop {
            let s = *spw.add(tz) as usize;
            if s < 33 {
                let t = steps.add(s - 1);
                let a = _mm_load_si128((*t).a.as_ptr() as *const __m128i);
                let bb = _mm_load_si128((*t).b.as_ptr() as *const __m128i);
                let cc = _mm_load_si128((*t).c.as_ptr() as *const __m128i);
                let dd = _mm_load_si128((*t).d.as_ptr() as *const __m128i);
                let g0 = _mm_or_si128(_mm_shuffle_epi8(f0, a), _mm_shuffle_epi8(f1, bb));
                f1 = _mm_or_si128(_mm_shuffle_epi8(f1, cc), _mm_shuffle_epi8(f0, dd));
                f0 = g0;
            } else {
                let v = second_cold(tail, (s - 1).min(255) - 32, f1);
                f1 = _mm_alignr_epi8::<15>(f1, f0);
                f0 = _mm_or_si128(_mm_slli_si128::<1>(f0), _mm_cvtsi32_si128(v as i32));
            }
            let c = _mm_cvtsi128_si32(f0) as u8 as u64;

            if w != 0 {
                let t2 = w.trailing_zeros() as usize;
                w &= w - 1;
                let total = _bzhi_u64(after, t2 as u32) >> tz;
                let span = t2 - tz;
                tz = t2;
                if total <= CAP as u64 {
                    *out = ((c << 24) | total) as u32;
                    out = out.add(1);
                } else {
                    emit_long!(c, total, span, false);
                }
                if out == out_end {
                    break;
                }
            } else {
                let q = wi * 64 + tz;
                let next = seek!();
                let q2 = next.map_or(end, |t| wi * 64 + t);
                let span = q2 - q;
                if span > 22 {
                    return None;
                }
                let total = count!(q, q2);
                let finished = match next {
                    Some(t) => {
                        tz = t;
                        after = *onep.add(wi) >> 1;
                        spw = sp.add(wi * 64);
                        false
                    }
                    None => true,
                };
                if total <= CAP as u64 {
                    *out = ((c << 24) | total) as u32;
                    out = out.add(1);
                } else {
                    emit_long!(c, total, span, finished);
                }
                if finished {
                    st.done = true;
                    break;
                }
                if out == out_end {
                    break;
                }
            }
        }

        _mm_store_si128(st.front.as_mut_ptr() as *mut __m128i, f0);
        _mm_store_si128(st.front.as_mut_ptr().add(16) as *mut __m128i, f1);
        st.wi = wi;
        st.w = w;
        st.tz = tz;
        Some(out.offset_from(out0) as usize)
    }

    /// 100% safe batch consumer (injective `seq_to_unseq`, 100% of real bzip2 blocks):
    /// all 15 GPRs dedicated to `Pass::take + OutQ16`, zero `c == prev_c` check, `m`/`in_out`
    /// kept in registers and `f.runs` incremented once per batch instead of per run.
    #[inline(never)]
    fn probe_flush_batch_direct<const CHUNK: usize, const STRIDE: usize>(
        batch: &[u32],
        pass: &mut Pass<'_, CHUNK, STRIDE>,
        pos: &mut u32,
        p: u32,
    ) {
        let mut cur = *pos;
        let mut m = pass.f.m;
        let in_out = pass.queues.in_out;
        pass.f.runs += batch.len();
        for &packed in batch {
            let c = (packed >> 24) as u8;
            let width = packed & 0x00FF_FFFF;
            if width <= LIMIT {
                let a = usize::from(c);
                if p.wrapping_sub(cur) < width {
                    pass.f.mark = m + (p - cur);
                }
                let f = pass.queues.fill[a] as usize;
                let lo = m & (EPOCH - 1);
                let tail = pass.queues.tails[a];
                let room_ok = if CHUNK + 16 <= STRIDE {
                    f + (width as usize) <= CHUNK
                } else {
                    f + 16 <= CHUNK
                };
                if room_ok && lo + width < EPOCH && tail < in_out {
                    let at = tail as usize * (2 * STRIDE) + 2 * f;
                    let iota: [u16; 16] =
                        std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                    for (d, v) in pass.queues.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                        d.copy_from_slice(&v.to_le_bytes());
                    }
                    pass.queues.fill[a] = (f as u32) + width;
                } else {
                    pass.queues.probe_outq_push_slow(a, m, width, &pass.f.thin);
                }
                pass.f.thin[a] += width;
                m += width;
            } else {
                pass.f.m = m;
                if p.wrapping_sub(cur) >= width {
                    pass.piece(c, cur, width, false, p);
                } else {
                    pass.take_marked(c, cur, width, p, LIMIT);
                    m = pass.f.m;
                }
            }
            cur += width;
        }
        pass.f.m = m;
        *pos = cur;
    }

    /// 100% safe fallback batch consumer when `seq_to_unseq` is non-injective: merges adjacent
    /// identical output bytes seamlessly across batch boundaries (supports merged runs > 16M!).
    #[inline(never)]
    fn probe_flush_batch_merging<const CHUNK: usize, const STRIDE: usize>(
        batch: &[u32],
        pass: &mut Pass<'_, CHUNK, STRIDE>,
        pos: &mut u32,
        pending_c: &mut u8,
        pending_w: &mut u32,
        p: u32,
    ) {
        let mut cur = *pos;
        let mut pc = *pending_c;
        let mut pw = *pending_w;
        for &packed in batch {
            let c = (packed >> 24) as u8;
            let width = packed & 0x00FF_FFFF;
            if c == pc && pw != 0 {
                pw += width;
            } else {
                if pw != 0 {
                    pass.take(pc, cur, pw, p, LIMIT);
                    cur += pw;
                }
                pc = c;
                pw = width;
            }
        }
        *pos = cur;
        *pending_c = pc;
        *pending_w = pw;
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn consume_single_run<const CHUNK: usize, const STRIDE: usize>(
        injective: bool,
        c: u8,
        width: u32,
        pass: &mut Pass<'_, CHUNK, STRIDE>,
        pos: &mut u32,
        pending_c: &mut u8,
        pending_w: &mut u32,
        p: u32,
    ) {
        if width == 0 {
            return;
        }
        if injective {
            pass.take(c, *pos, width, p, LIMIT);
            *pos += width;
        } else if c == *pending_c && *pending_w != 0 {
            *pending_w += width;
        } else {
            if *pending_w != 0 {
                pass.take(*pending_c, *pos, *pending_w, p, LIMIT);
                *pos += *pending_w;
            }
            *pending_c = c;
            *pending_w = width;
        }
    }

    /// Full L1d-batched A5 + A6a driver for any `(BATCH, CHUNK, STRIDE)`.
    #[allow(clippy::too_many_arguments)]
    fn run_l1_batch<const BATCH: usize, const CHUNK: usize, const STRIDE: usize>(
        b: &Symbols,
        scratch: &mut FrontScratch,
        out: &mut [u8],
        spill: &mut Spill<STRIDE>,
        edges: &mut Vec<u32>,
        limit_m: u32,
        cap_strips: usize,
        lits_refused: usize,
    ) -> Option<(usize, usize, First, Heads, usize, bool)> {
        let (end, lits, max_sym) = unsafe { probe_build_bitmaps_avx2(b, scratch, lits_refused)? };

        let used = usize::from(max_sym).clamp(1, 256);
        let injective = s2u_injective(&b.seq_to_unseq, used);

        let last = end / 64;
        let litp = scratch.lit.as_ptr();
        let onep = scratch.one.as_ptr();
        let mut wi = 0usize;
        let mut w = scratch.lit[0];
        let first = loop {
            if w != 0 {
                let t = w.trailing_zeros() as usize;
                w &= w - 1;
                break Some(t);
            }
            if wi == last {
                break None;
            }
            wi += 1;
            w = scratch.lit[wi];
        };

        let q = first.map_or(end, |t| wi * 64 + t);
        if q > 21 {
            return None;
        }
        let open_len = unsafe { _bzhi_u64(*onep, q as u32 + 1) } - 1;
        let mut slack = b.max_len / CAP as usize + 1;
        let open_recs = open_len.div_ceil(CAP as u64);
        let open_more = open_recs.saturating_sub(1) as usize;
        if open_more > slack || open_len > b.max_len as u64 {
            return None;
        }
        slack -= open_more;

        let p = b.orig_ptr;
        let mut pass = Pass::<CHUNK, STRIDE>::new(out, spill, edges, limit_m, cap_strips);
        let mut pos = 0u32;
        let mut pending_c = 0u8;
        let mut pending_w = 0u32;

        if open_len != 0 {
            consume_single_run(
                injective,
                b.seq_to_unseq[0],
                open_len as u32,
                &mut pass,
                &mut pos,
                &mut pending_c,
                &mut pending_w,
                p,
            );
        }

        let mut extra_recs = 0usize;
        if let Some(tz) = first {
            let mut front = [0u8; 32];
            front.copy_from_slice(&b.seq_to_unseq[..32]);
            let mut rows = MtfRows([0u8; 224]);
            rows.0.copy_from_slice(&b.seq_to_unseq[32..]);
            let mut st = BatchState {
                front,
                rows,
                wi,
                w,
                tz,
                last,
                end,
                slack,
                big: open_len,
                extra_recs: 0,
                done: false,
                huge: None,
            };
            let mut batch = [0u32; BATCH];
            let sp = b.syms.as_ptr();
            let max_len = b.max_len as u64;
            while !st.done {
                let blen = unsafe {
                    probe_fill_batch_avx2::<BATCH>(&mut st, sp, litp, onep, max_len, &mut batch)?
                };
                if blen != 0 {
                    if injective {
                        probe_flush_batch_direct(&batch[..blen], &mut pass, &mut pos, p);
                    } else {
                        probe_flush_batch_merging(
                            &batch[..blen],
                            &mut pass,
                            &mut pos,
                            &mut pending_c,
                            &mut pending_w,
                            p,
                        );
                    }
                }
                if let Some((hc, htot)) = st.huge.take() {
                    consume_single_run(
                        injective,
                        hc,
                        htot as u32,
                        &mut pass,
                        &mut pos,
                        &mut pending_c,
                        &mut pending_w,
                        p,
                    );
                }
            }
            extra_recs = st.extra_recs;
        }

        if !injective && pending_w != 0 {
            pass.take(pending_c, pos, pending_w, p, LIMIT);
            pos += pending_w;
        }

        let n = pos as usize;
        if n > b.max_len {
            return None;
        }
        let recs = open_recs as usize + lits + extra_recs;
        let heads = pass.queues.heads();
        let over = pass.queues.over;
        Some((n, recs, pass.f, heads, pass.ns, over))
    }

    // ---- Verification helpers ----

    #[derive(PartialEq, Eq, Debug)]
    struct BlockSnapshot {
        n: usize,
        recs: usize,
        first: First,
        ns: usize,
        edges: Vec<u32>,
        ranks: Vec<Vec<u32>>,
        over: bool,
    }

    #[allow(clippy::too_many_arguments)]
    fn snapshot_from<const CHUNK: usize, const STRIDE: usize>(
        n: usize,
        recs: usize,
        first: First,
        heads: &Heads,
        ns: usize,
        edges: &[u32],
        out: &[u8],
        spill: &Spill<STRIDE>,
        over: bool,
    ) -> BlockSnapshot {
        let mut ranks = Vec::with_capacity(256);
        for a in 0..256 {
            let mut r = Reader::<CHUNK, STRIDE>::new(out, spill, heads, a);
            let mut v = Vec::new();
            r.read_all(first.thin[a], &mut v);
            ranks.push(v);
        }
        BlockSnapshot {
            n,
            recs,
            first,
            ns,
            edges: edges[..3 * ns].to_vec(),
            ranks,
            over,
        }
    }

    fn run_all_arms_snapshot(b: &Symbols) -> (BlockSnapshot, Vec<(&'static str, BlockSnapshot)>) {
        let bound = b.max_len;
        let strips = bound / (LIMIT as usize + 1) + 2;
        let buf_len = bound.min(1_000_000) + INLINE;

        // Reference: Arm 1 (`shipped_split`)
        let mut scratch = FrontScratch::default();
        let mut out = vec![0u8; buf_len];
        let mut spill512 = Spill::<512>::default();
        let mut edges = Vec::new();
        let n1 = unsafe { probe_front_avx2_shipped(b, &mut scratch) }.expect("shipped_split");
        let recs1 = scratch.runs.len();
        let (f1, h1, ns1, ov1) = probe_a6a_from_records::<512, 512>(
            &scratch.runs,
            b.orig_ptr,
            &mut out,
            &mut spill512,
            &mut edges,
            bound as u32,
            strips,
        );
        let ref_snap =
            snapshot_from::<512, 512>(n1, recs1, f1, &h1, ns1, &edges, &out, &spill512, ov1);

        let mut others = Vec::new();

        // Arm 2: `direct_fold_1010`
        {
            let mut sc = FrontScratch::default();
            let mut out2 = vec![0u8; buf_len];
            let mut sp2 = Spill::<512>::default();
            let mut ed2 = Vec::new();
            let (n2, recs2, f2, h2, ns2, ov2) = {
                let mut sink = FoldSink {
                    pass: Pass::<512, 512>::new(
                        &mut out2,
                        &mut sp2,
                        &mut ed2,
                        bound as u32,
                        strips,
                    ),
                    p: b.orig_ptr,
                    c: 0,
                    pos: 0,
                    width: 0,
                };
                let (n2, recs2) =
                    unsafe { probe_front_avx2_fold_1010(b, &mut sc, &mut sink, usize::MAX) }
                        .expect("direct_fold_1010");
                sink.flush();
                let h2 = sink.pass.queues.heads();
                let ov2 = sink.pass.queues.over;
                let f2 = sink.pass.f;
                let ns2 = sink.pass.ns;
                (n2, recs2, f2, h2, ns2, ov2)
            };
            others.push((
                "direct_fold_1010",
                snapshot_from::<512, 512>(n2, recs2, f2, &h2, ns2, &ed2, &out2, &sp2, ov2),
            ));
        }

        // Arm 3: `l1_batch_256_shipped_outq`
        {
            let mut sc = FrontScratch::default();
            let mut out3 = vec![0u8; buf_len];
            let mut sp3 = Spill::<512>::default();
            let mut ed3 = Vec::new();
            let (n3, recs3, f3, h3, ns3, ov3) = run_l1_batch::<256, 512, 512>(
                b,
                &mut sc,
                &mut out3,
                &mut sp3,
                &mut ed3,
                bound as u32,
                strips,
                usize::MAX,
            )
            .expect("l1_batch_256_shipped_outq");
            others.push((
                "l1_batch_256_shipped_outq",
                snapshot_from::<512, 512>(n3, recs3, f3, &h3, ns3, &ed3, &out3, &sp3, ov3),
            ));
        }

        // Arm 4: `l1_batch_256_guard496`
        {
            let mut sc = FrontScratch::default();
            let mut out4 = vec![0u8; buf_len];
            let mut sp4 = Spill::<512>::default();
            let mut ed4 = Vec::new();
            let (n4, recs4, f4, h4, ns4, ov4) = run_l1_batch::<256, 496, 512>(
                b,
                &mut sc,
                &mut out4,
                &mut sp4,
                &mut ed4,
                bound as u32,
                strips,
                usize::MAX,
            )
            .expect("l1_batch_256_guard496");
            others.push((
                "l1_batch_256_guard496",
                snapshot_from::<496, 512>(n4, recs4, f4, &h4, ns4, &ed4, &out4, &sp4, ov4),
            ));
        }

        // Arm 5: `l1_batch_256_guard1008`
        {
            let mut sc = FrontScratch::default();
            let mut out5 = vec![0u8; buf_len];
            let mut sp5 = Spill::<1024>::default();
            let mut ed5 = Vec::new();
            let (n5, recs5, f5, h5, ns5, ov5) = run_l1_batch::<256, 1008, 1024>(
                b,
                &mut sc,
                &mut out5,
                &mut sp5,
                &mut ed5,
                bound as u32,
                strips,
                usize::MAX,
            )
            .expect("l1_batch_256_guard1008");
            others.push((
                "l1_batch_256_guard1008",
                snapshot_from::<1008, 1024>(n5, recs5, f5, &h5, ns5, &ed5, &out5, &sp5, ov5),
            ));
        }

        (ref_snap, others)
    }

    fn verify_synthetic_edge_cases() {
        // 1. Non-injective `seq_to_unseq` crossing batch boundaries (BATCH=256)
        {
            let mut s2u = [0u8; 256];
            s2u[0] = b'A';
            s2u[1] = b'A';
            s2u[2] = b'B';
            s2u[3] = b'A';
            s2u[4] = b'C';
            let mut syms = Vec::new();
            // Opening run of 3 of 'A', then 600 literals alternating 'A'/'A' and 'B'/'C'
            syms.push(0); // +1
            syms.push(0); // +2 => open_len = 3
            for i in 0..600u16 {
                let s = match i % 5 {
                    0 => 2, // 'A' (merges with previous 'A'!)
                    1 => 4, // 'A' (merges again across batch boundary!)
                    2 => 3, // 'B'
                    3 => 5, // 'C'
                    _ => 2,
                };
                syms.push(s);
                if i % 17 == 0 {
                    // Add a thick run (>16)
                    syms.push(1);
                    syms.push(1);
                    syms.push(1);
                    syms.push(1); // +30
                }
            }
            syms.push(6); // eob
            let b = Symbols {
                crc: 0,
                orig_ptr: 15,
                seq_to_unseq: s2u,
                syms,
                eob: 6,
                max_len: 100_000,
            };
            let (ref_snap, others) = run_all_arms_snapshot(&b);
            for (name, snap) in others {
                assert_eq!(snap, ref_snap, "synthetic non-injective mismatch in {name}");
            }
        }

        // 2. Merged run > 16M (`> 1 << 24 = 16,777,216`) via non-injective `seq_to_unseq`
        {
            let mut s2u = [0u8; 256];
            s2u[0] = b'X';
            s2u[1] = b'X';
            s2u[2] = b'Y';
            let mut syms = Vec::new();
            // 5 consecutive literals all mapping to 'X', each followed by 21 RUNB (1) symbols
            // => each literal has total = 2^22 - 1 = 4,194,303; 5 merged = 20,971,515 > 16,777,216!
            for _ in 0..5 {
                syms.push(2); // literal swapping s2u[0]/s2u[1], both 'X'
                syms.extend_from_slice(&[1u16; 21]);
            }
            // Followed by a small thin run of 'Y'
            syms.push(3);
            syms.push(4); // eob
            let b = Symbols {
                crc: 0,
                orig_ptr: 10_000_000, // marked row right inside the 20.97M thick run!
                seq_to_unseq: s2u,
                syms,
                eob: 4,
                max_len: 25_000_000,
            };
            let (ref_snap, others) = run_all_arms_snapshot(&b);
            assert_eq!(ref_snap.n, 5 * 4_194_303 + 1);
            assert_eq!(ref_snap.first.runs, 2); // 1 huge merged 'X' run (>16M) + 1 thin 'Y' run
            assert_eq!(ref_snap.first.thick_rows[b'X' as usize], 5 * 4_194_303 - 1);
            assert_eq!(ref_snap.ns, 2); // split around orig_ptr = 10_000_000!
            for (name, snap) in others {
                assert_eq!(snap, ref_snap, "synthetic >16M run mismatch in {name}");
            }
        }

        // 3. Direct `consume_single_run` check for a raw >16M run (20,000,000 rows)
        {
            let mut out = vec![0u8; 4096];
            let mut sp = Spill::<512>::default();
            let mut ed = Vec::new();
            let mut pass = Pass::<496, 512>::new(&mut out, &mut sp, &mut ed, 25_000_000, 100);
            let (mut pos, mut pc, mut pw) = (0u32, 0u8, 0u32);
            consume_single_run(
                true, b'Z', 20_000_000, &mut pass, &mut pos, &mut pc, &mut pw, 123_456,
            );
            assert_eq!(pos, 20_000_000);
            assert_eq!(pass.ns, 2);
            assert_eq!(pass.f.m, 1);
            assert_eq!(pass.f.mark, 0);
            assert_eq!(pass.f.thick_rows[b'Z' as usize], 19_999_999);
        }

        // 4. Zero-literal block (only opening run) + small block forcing `ovf` chunks (`in_out == 0`)
        {
            let s2u: [u8; 256] = std::array::from_fn(|i| i as u8);
            // Zero literals, opening run of 12
            let b0 = Symbols {
                crc: 0,
                orig_ptr: 5,
                seq_to_unseq: s2u,
                syms: vec![1, 0, 1, 2], // RUNB(2) + RUNA(2) + RUNB(8) = 12, EOB=2
                eob: 2,
                max_len: 500,
            };
            let (ref0, others0) = run_all_arms_snapshot(&b0);
            for (name, snap) in others0 {
                assert_eq!(snap, ref0, "synthetic zero-literal mismatch in {name}");
            }

            // Small block with deep MTF (`s >= 33` and `s >= 65`) + `ovf` chunks
            let mut syms = Vec::new();
            for s in [2u16, 17, 33, 45, 70, 120, 200, 255, 2, 3, 4, 5, 65, 32, 33] {
                syms.push(s);
            }
            syms.push(257); // eob
            let b1 = Symbols {
                crc: 0,
                orig_ptr: 7,
                seq_to_unseq: s2u,
                syms,
                eob: 257,
                max_len: 400, // < 1024 B => `in_out == 0`, forces `ovf` chunks!
            };
            let (ref1, others1) = run_all_arms_snapshot(&b1);
            for (name, snap) in others1 {
                assert_eq!(snap, ref1, "synthetic small/deep-mtf mismatch in {name}");
            }
        }

        // 5. Epoch crossing (`m` > 65,536) + chunk boundary crossings (`f = 495..496`, `511..512`, `1007..1008`)
        {
            let s2u: [u8; 256] = std::array::from_fn(|i| i as u8);
            let mut syms = Vec::with_capacity(45_000);
            for i in 0..22_000u32 {
                // Alternate 2..=5 so every literal is distinct and spans 3 bytes (thin, w=3)
                // Total retained rows m = 66,000 > 65,536 (crosses EPOCH boundary!)
                syms.push(((i % 4) + 2) as u16);
                syms.push(1); // +2 => total = 3
            }
            syms.push(10); // eob
            let b = Symbols {
                crc: 0,
                orig_ptr: 65_535,
                seq_to_unseq: s2u,
                syms,
                eob: 10,
                max_len: 100_000,
            };
            let (ref_snap, others) = run_all_arms_snapshot(&b);
            assert!(ref_snap.first.m > EPOCH);
            for (name, snap) in others {
                assert_eq!(
                    snap, ref_snap,
                    "synthetic epoch-crossing mismatch in {name}"
                );
            }
        }
    }

    // ---- Benchmark harness ----

    #[derive(Clone, Copy)]
    enum Arm {
        ShippedSplit,
        DirectFold1010,
        L1Batch256ShippedOutQ,
        L1Batch256Guard496,
        L1Batch256Guard1008,
        L1Batch128Guard496,
        L1Batch512Guard496,
        L1Batch1024Guard496,
    }

    impl Arm {
        const ALL: [Arm; 8] = [
            Arm::ShippedSplit,
            Arm::DirectFold1010,
            Arm::L1Batch256ShippedOutQ,
            Arm::L1Batch256Guard496,
            Arm::L1Batch256Guard1008,
            Arm::L1Batch128Guard496,
            Arm::L1Batch512Guard496,
            Arm::L1Batch1024Guard496,
        ];

        fn name(self) -> &'static str {
            match self {
                Arm::ShippedSplit => "shipped_split",
                Arm::DirectFold1010 => "direct_fold_1010",
                Arm::L1Batch256ShippedOutQ => "l1_batch_256_shipped_outq",
                Arm::L1Batch256Guard496 => "l1_batch_256_guard496",
                Arm::L1Batch256Guard1008 => "l1_batch_256_guard1008",
                Arm::L1Batch128Guard496 => "l1_batch_128_guard496",
                Arm::L1Batch512Guard496 => "l1_batch_512_guard496",
                Arm::L1Batch1024Guard496 => "l1_batch_1024_guard496",
            }
        }
    }

    struct BenchScratch {
        front: FrontScratch,
        out: Vec<u8>,
        edges: Vec<u32>,
        spill512: Spill<512>,
        spill1024: Spill<1024>,
    }

    impl BenchScratch {
        fn new() -> Self {
            BenchScratch {
                front: FrontScratch::default(),
                out: vec![0u8; 900_000 + INLINE],
                edges: Vec::with_capacity(65_536),
                spill512: Spill::default(),
                spill1024: Spill::default(),
            }
        }
    }

    #[inline(never)]
    fn run_arm_once(arm: Arm, blocks: &[Symbols], sc: &mut BenchScratch) -> u64 {
        let mut checksum = 0u64;
        for b in blocks {
            let bound = b.max_len;
            let strips = bound / (LIMIT as usize + 1) + 2;
            let need = bound + INLINE;
            if sc.out.len() < need {
                sc.out.resize(need, 0);
            }
            let out_slice = &mut sc.out[..need];
            match arm {
                Arm::ShippedSplit => {
                    let n = unsafe { probe_front_avx2_shipped(b, &mut sc.front) }.unwrap();
                    let recs = sc.front.runs.len();
                    let (f, _h, ns, _ov) = probe_a6a_from_records::<512, 512>(
                        &sc.front.runs,
                        b.orig_ptr,
                        out_slice,
                        &mut sc.spill512,
                        &mut sc.edges,
                        bound as u32,
                        strips,
                    );
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
                Arm::DirectFold1010 => {
                    let mut sink = FoldSink {
                        pass: Pass::<512, 512>::new(
                            out_slice,
                            &mut sc.spill512,
                            &mut sc.edges,
                            bound as u32,
                            strips,
                        ),
                        p: b.orig_ptr,
                        c: 0,
                        pos: 0,
                        width: 0,
                    };
                    let (n, recs) = unsafe {
                        probe_front_avx2_fold_1010(b, &mut sc.front, &mut sink, usize::MAX)
                    }
                    .unwrap();
                    sink.flush();
                    let f = sink.pass.f;
                    let ns = sink.pass.ns;
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
                Arm::L1Batch256ShippedOutQ => {
                    let (n, recs, f, _h, ns, _ov) = run_l1_batch::<256, 512, 512>(
                        b,
                        &mut sc.front,
                        out_slice,
                        &mut sc.spill512,
                        &mut sc.edges,
                        bound as u32,
                        strips,
                        usize::MAX,
                    )
                    .unwrap();
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
                Arm::L1Batch256Guard496 => {
                    let (n, recs, f, _h, ns, _ov) = run_l1_batch::<256, 496, 512>(
                        b,
                        &mut sc.front,
                        out_slice,
                        &mut sc.spill512,
                        &mut sc.edges,
                        bound as u32,
                        strips,
                        usize::MAX,
                    )
                    .unwrap();
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
                Arm::L1Batch256Guard1008 => {
                    let (n, recs, f, _h, ns, _ov) = run_l1_batch::<256, 1008, 1024>(
                        b,
                        &mut sc.front,
                        out_slice,
                        &mut sc.spill1024,
                        &mut sc.edges,
                        bound as u32,
                        strips,
                        usize::MAX,
                    )
                    .unwrap();
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
                Arm::L1Batch128Guard496 => {
                    let (n, recs, f, _h, ns, _ov) = run_l1_batch::<128, 496, 512>(
                        b,
                        &mut sc.front,
                        out_slice,
                        &mut sc.spill512,
                        &mut sc.edges,
                        bound as u32,
                        strips,
                        usize::MAX,
                    )
                    .unwrap();
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
                Arm::L1Batch512Guard496 => {
                    let (n, recs, f, _h, ns, _ov) = run_l1_batch::<512, 496, 512>(
                        b,
                        &mut sc.front,
                        out_slice,
                        &mut sc.spill512,
                        &mut sc.edges,
                        bound as u32,
                        strips,
                        usize::MAX,
                    )
                    .unwrap();
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
                Arm::L1Batch1024Guard496 => {
                    let (n, recs, f, _h, ns, _ov) = run_l1_batch::<1024, 496, 512>(
                        b,
                        &mut sc.front,
                        out_slice,
                        &mut sc.spill512,
                        &mut sc.edges,
                        bound as u32,
                        strips,
                        usize::MAX,
                    )
                    .unwrap();
                    checksum = checksum
                        .wrapping_add(n as u64)
                        .wrapping_add(recs as u64)
                        .wrapping_add(u64::from(f.m))
                        .wrapping_add(u64::from(f.mark))
                        .wrapping_add(ns as u64);
                }
            }
        }
        black_box(checksum)
    }

    #[inline(never)]
    fn run_arm_with_a6c_readback<const BATCH: usize, const CHUNK: usize, const STRIDE: usize>(
        blocks: &[Symbols],
        sc: &mut FrontScratch,
        out: &mut Vec<u8>,
        spill: &mut Spill<STRIDE>,
        edges: &mut Vec<u32>,
    ) -> u64 {
        let mut acc = 0u64;
        for b in blocks {
            let bound = b.max_len;
            let strips = bound / (LIMIT as usize + 1) + 2;
            let need = bound + INLINE;
            if out.len() < need {
                out.resize(need, 0);
            }
            let out_slice = &mut out[..need];
            let (_n, _recs, f, h, _ns, _ov) = run_l1_batch::<BATCH, CHUNK, STRIDE>(
                b,
                sc,
                out_slice,
                spill,
                edges,
                bound as u32,
                strips,
                usize::MAX,
            )
            .unwrap();
            for a in 0..256 {
                if f.thin[a] != 0 {
                    let mut r = Reader::<CHUNK, STRIDE>::new(out_slice, spill, &h, a);
                    acc = acc.wrapping_add(r.checksum_all(f.thin[a]));
                }
            }
        }
        black_box(acc)
    }

    #[inline(never)]
    fn run_shipped_with_a6c_readback(
        blocks: &[Symbols],
        sc: &mut FrontScratch,
        out: &mut Vec<u8>,
        spill: &mut Spill<512>,
        edges: &mut Vec<u32>,
    ) -> u64 {
        let mut acc = 0u64;
        for b in blocks {
            let bound = b.max_len;
            let strips = bound / (LIMIT as usize + 1) + 2;
            let need = bound + INLINE;
            if out.len() < need {
                out.resize(need, 0);
            }
            let out_slice = &mut out[..need];
            let _n = unsafe { probe_front_avx2_shipped(b, sc) }.unwrap();
            let (f, h, _ns, _ov) = probe_a6a_from_records::<512, 512>(
                &sc.runs,
                b.orig_ptr,
                out_slice,
                spill,
                edges,
                bound as u32,
                strips,
            );
            for a in 0..256 {
                if f.thin[a] != 0 {
                    let mut r = Reader::<512, 512>::new(out_slice, spill, &h, a);
                    acc = acc.wrapping_add(r.checksum_all(f.thin[a]));
                }
            }
        }
        black_box(acc)
    }

    fn stats(vals: &[f64]) -> (f64, f64, f64) {
        let mut s = vals.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        let med = s[s.len() / 2];
        (med, s[0], s[s.len() - 1])
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        let path = args
            .get(1)
            .map(String::as_str)
            .unwrap_or("corpora/wikidata.bz2");
        let repeats: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(7);

        let kernels = Kernels::detect();
        assert!(
            std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("bmi1")
                && std::is_x86_feature_detected!("bmi2")
                && std::is_x86_feature_detected!("lzcnt")
                && std::is_x86_feature_detected!("popcnt"),
            "CPU missing required AVX2/BMI/LZCNT/POPCNT features"
        );

        println!("=== Phase 0: Synthetic edge-case verification ===");
        verify_synthetic_edge_cases();
        println!(
            "PASS: non-injective seq_to_unseq, merged >16M run (20,971,515 rows), raw 20M run, zero-literal, ovf chunks, deep MTF, epoch crossings"
        );

        println!("\n=== Phase 1: Loading corpus `{path}` and selecting Open blocks ===");
        let data = std::fs::read(path).expect("read corpus");
        let mut scanner = Scanner::new(SliceSource(&data));
        let mut all_blocks = Vec::new();
        while let Event::Block { symbols, .. } = scanner.next(Vec::new()).expect("scan") {
            all_blocks.push(symbols);
        }

        let mut open_blocks = Vec::new();
        let mut prod_runs = Runs::default();
        let mut sc_check = FrontScratch::default();
        for b in all_blocks {
            let n_prod = runs::front(&b, &mut prod_runs, kernels).expect("prod runs::front");
            let n_our =
                unsafe { probe_front_avx2_shipped(&b, &mut sc_check) }.expect("our front_avx2");
            assert_eq!(n_our, n_prod, "front_avx2 n mismatch vs production");
            if let Some(cap) = open_cap(sc_check.runs.len(), n_our, b.orig_ptr) {
                if within_cap(&sc_check.runs, b.orig_ptr, LIMIT, cap) {
                    open_blocks.push(b);
                }
            }
        }
        println!(
            "Loaded {} Open blocks (total symbols = {})",
            open_blocks.len(),
            open_blocks.iter().map(|b| b.syms.len()).sum::<usize>()
        );

        println!("\n=== Phase 2: 100% bit-for-bit state-exact verification across all {} Open blocks ===", open_blocks.len());
        let mut total_recs = 0usize;
        let mut total_thin_m = 0u64;
        let mut total_thick_ns = 0usize;
        let mut total_runs = 0usize;
        for (bi, b) in open_blocks.iter().enumerate() {
            let (ref_snap, others) = run_all_arms_snapshot(b);
            total_recs += ref_snap.recs;
            total_thin_m += u64::from(ref_snap.first.m);
            total_thick_ns += ref_snap.ns;
            total_runs += ref_snap.first.runs;
            for (name, snap) in others {
                assert_eq!(
                    snap, ref_snap,
                    "block {bi}: arm `{name}` differed from `shipped_split`"
                );
            }
        }
        println!(
            "PASS: all 5 core arms 100% bit-for-bit state-identical on all {} Open blocks (recs={}, runs={}, retained_m={}, thick_strips={})",
            open_blocks.len(),
            total_recs,
            total_runs,
            total_thin_m,
            total_thick_ns
        );

        // OutQ16 slow-call census across all 233 Open blocks
        println!("\n=== Phase 3: OutQ16 push_slow & overflow census ===");
        let mut sc = BenchScratch::new();
        for (label, arm) in [
            ("shipped_512_512", Arm::L1Batch256ShippedOutQ),
            ("guard_496_512", Arm::L1Batch256Guard496),
            ("guard_1008_1024", Arm::L1Batch256Guard1008),
        ] {
            sc.spill512 = Spill::default();
            sc.spill1024 = Spill::default();
            let _ = run_arm_once(arm, &open_blocks, &mut sc);
            let (ps, tg, ch, ovf) = match arm {
                Arm::L1Batch256Guard1008 => (
                    sc.spill1024.push_slow_calls,
                    sc.spill1024.tail_guard_calls,
                    sc.spill1024.chunks_taken,
                    sc.spill1024.ovf_chunks,
                ),
                _ => (
                    sc.spill512.push_slow_calls,
                    sc.spill512.tail_guard_calls,
                    sc.spill512.chunks_taken,
                    sc.spill512.ovf_chunks,
                ),
            };
            println!(
                "  {label:<18} push_slow={ps:>7}  tail_guard={tg:>7}  chunks={ch:>6}  ovf_chunks={ovf}"
            );
            assert_eq!(ovf, 0, "{label} unexpectedly spilled overflow chunks!");
        }

        // Warmup
        let mut expected_cksum = None;
        for &arm in &Arm::ALL {
            let ck = run_arm_once(arm, &open_blocks, &mut sc);
            if let Some(exp) = expected_cksum {
                assert_eq!(ck, exp, "checksum mismatch for {}", arm.name());
            } else {
                expected_cksum = Some(ck);
            }
        }

        println!(
            "\n=== Phase 4: {repeats} interleaved repeats (A5 + A6a, wall + schedstat CPU ms) ==="
        );
        let n_arms = Arm::ALL.len();
        let mut wall_ms = vec![vec![0.0f64; repeats]; n_arms];
        let mut cpu_ms = vec![vec![0.0f64; repeats]; n_arms];
        let mut clock = CpuClock::new();

        for r in 0..repeats {
            // Rotate arm order each repeat to cancel drift/order bias
            for step in 0..n_arms {
                let idx = (step + r) % n_arms;
                let arm = Arm::ALL[idx];
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                let ck = run_arm_once(arm, &open_blocks, &mut sc);
                let dt_wall = t0.elapsed().as_secs_f64() * 1e3;
                let c1 = clock.now_ns();
                assert_eq!(Some(ck), expected_cksum);
                wall_ms[idx][r] = dt_wall;
                cpu_ms[idx][r] = match (c0, c1) {
                    (Some(a), Some(b)) => (b.saturating_sub(a)) as f64 / 1e6,
                    _ => dt_wall,
                };
            }
        }

        let (base_wall_med, base_wall_min, base_wall_max) = stats(&wall_ms[0]);
        let (base_cpu_med, base_cpu_min, base_cpu_max) = stats(&cpu_ms[0]);
        println!(
            "{:<26} | {:>23} | {:>23} | {:>18} | {:>18} | {:>8} | {:>8}",
            "arm",
            "wall_ms med [min..max]",
            "cpu_ms med [min..max]",
            "paired wall ratio",
            "paired cpu ratio",
            "1W ms",
            "1W %wall"
        );
        println!("{}", "-".repeat(136));
        for (i, &arm) in Arm::ALL.iter().enumerate() {
            let (wm, wlo, whi) = stats(&wall_ms[i]);
            let (cm, clo, chi) = stats(&cpu_ms[i]);
            let pr_wall: Vec<f64> = (0..repeats)
                .map(|r| wall_ms[i][r] / wall_ms[0][r])
                .collect();
            let pr_cpu: Vec<f64> = (0..repeats).map(|r| cpu_ms[i][r] / cpu_ms[0][r]).collect();
            let (pwm, pwlo, pwhi) = stats(&pr_wall);
            let (pcm, pclo, pchi) = stats(&pr_cpu);
            // Project onto i9 A5+A6a using paired CPU ratio (and wall ratio noted below)
            let save_frac = 1.0 - pcm;
            let save_1w_ms = save_frac * I9_A5_A6A_1W_MS;
            let save_1w_pct = save_1w_ms / I9_WALL_1W_MS * 100.0;
            println!(
                "{:<26} | {:6.2} [{:6.2}..{:6.2}] | {:6.2} [{:6.2}..{:6.2}] | {:.4}x [{:.3}..{:.3}] | {:.4}x [{:.3}..{:.3}] | {:+7.2} | {:+7.2}%",
                arm.name(),
                wm, wlo, whi,
                cm, clo, chi,
                pwm, pwlo, pwhi,
                pcm, pclo, pchi,
                save_1w_ms,
                save_1w_pct
            );
        }
        println!(
            "\nBaseline `shipped_split`: wall={base_wall_med:.2} [{base_wall_min:.2}..{base_wall_max:.2}] ms, cpu={base_cpu_med:.2} [{base_cpu_min:.2}..{base_cpu_max:.2}] ms"
        );
        println!(
            "8W projection (A5+A6a={I9_A5_A6A_8W_MS:.1} ms/w of {I9_BUSY_8W_MS:.1} ms/w worker busy):"
        );
        for (i, &arm) in Arm::ALL.iter().enumerate().skip(1) {
            let pr_cpu: Vec<f64> = (0..repeats).map(|r| cpu_ms[i][r] / cpu_ms[0][r]).collect();
            let pr_wall: Vec<f64> = (0..repeats)
                .map(|r| wall_ms[i][r] / wall_ms[0][r])
                .collect();
            let (pcm, _, _) = stats(&pr_cpu);
            let (pwm, _, _) = stats(&pr_wall);
            let save_8w_cpu = (1.0 - pcm) * I9_A5_A6A_8W_MS;
            let save_8w_pct = save_8w_cpu / I9_BUSY_8W_MS * 100.0;
            let save_1w_wall = (1.0 - pwm) * I9_A5_A6A_1W_MS;
            let save_1w_wall_pct = save_1w_wall / I9_WALL_1W_MS * 100.0;
            println!(
                "  {:<26} cpu_8W={:+6.2} ms/w ({:+5.2}% busy) | wall_1W={:+6.2} ms ({:+5.2}% wall)",
                arm.name(),
                save_8w_cpu,
                save_8w_pct,
                save_1w_wall,
                save_1w_wall_pct
            );
        }

        // Phase 5: A5+A6a + A6c queue-readback check (checks if 496/512 or 1008/1024 moves A6c)
        println!("\n=== Phase 5: A5 + A6a + A6c queue-readback check ({repeats} repeats) ===");
        let mut rb_wall = vec![vec![0.0f64; repeats]; 4];
        let mut rb_cpu = vec![vec![0.0f64; repeats]; 4];
        let rb_names = [
            "shipped_split+read512",
            "l1_256_shipped+read512",
            "l1_256_guard496+read496",
            "l1_256_guard1008+read1008",
        ];
        let mut rb_ck = None;
        for r in 0..repeats {
            for step in 0..4 {
                let idx = (step + r) % 4;
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                let ck = match idx {
                    0 => run_shipped_with_a6c_readback(
                        &open_blocks,
                        &mut sc.front,
                        &mut sc.out,
                        &mut sc.spill512,
                        &mut sc.edges,
                    ),
                    1 => run_arm_with_a6c_readback::<256, 512, 512>(
                        &open_blocks,
                        &mut sc.front,
                        &mut sc.out,
                        &mut sc.spill512,
                        &mut sc.edges,
                    ),
                    2 => run_arm_with_a6c_readback::<256, 496, 512>(
                        &open_blocks,
                        &mut sc.front,
                        &mut sc.out,
                        &mut sc.spill512,
                        &mut sc.edges,
                    ),
                    _ => run_arm_with_a6c_readback::<256, 1008, 1024>(
                        &open_blocks,
                        &mut sc.front,
                        &mut sc.out,
                        &mut sc.spill1024,
                        &mut sc.edges,
                    ),
                };
                let dt_w = t0.elapsed().as_secs_f64() * 1e3;
                let c1 = clock.now_ns();
                if let Some(exp) = rb_ck {
                    assert_eq!(ck, exp, "A6c readback checksum mismatch");
                } else {
                    rb_ck = Some(ck);
                }
                rb_wall[idx][r] = dt_w;
                rb_cpu[idx][r] = match (c0, c1) {
                    (Some(a), Some(b)) => (b.saturating_sub(a)) as f64 / 1e6,
                    _ => dt_w,
                };
            }
        }
        for i in 0..4 {
            let (wm, wlo, whi) = stats(&rb_wall[i]);
            let (cm, clo, chi) = stats(&rb_cpu[i]);
            let pr_cpu: Vec<f64> = (0..repeats).map(|r| rb_cpu[i][r] / rb_cpu[0][r]).collect();
            let (pcm, pclo, pchi) = stats(&pr_cpu);
            println!(
                "  {:<26} wall={:6.2} [{:6.2}..{:6.2}] | cpu={:6.2} [{:6.2}..{:6.2}] | paired_cpu={:.4}x [{:.3}..{:.3}]",
                rb_names[i], wm, wlo, whi, cm, clo, chi, pcm, pclo, pchi
            );
        }

        // Phase 6: Isolated A5 vs A6a sub-phase decomposition (5 repeats)
        println!("\n=== Phase 6: Isolated A5 vs A6a sub-phase decomposition ===");
        let mut pre_runs: Vec<Vec<u16>> = Vec::with_capacity(open_blocks.len());
        let mut pre_packed: Vec<Vec<u32>> = Vec::with_capacity(open_blocks.len());
        for b in &open_blocks {
            let _ = unsafe { probe_front_avx2_shipped(b, &mut sc.front) }.unwrap();
            pre_runs.push(sc.front.runs.clone());
            let mut pvec = Vec::with_capacity(sc.front.runs.len());
            let mut i = 0;
            while i < sc.front.runs.len() {
                let rec = sc.front.runs[i];
                let c = (rec >> 8) as u8;
                let mut w = u32::from(rec & 0xFF);
                i += 1;
                while i < sc.front.runs.len() && (sc.front.runs[i] >> 8) as u8 == c {
                    w += u32::from(sc.front.runs[i] & 0xFF);
                    i += 1;
                }
                pvec.push((u32::from(c) << 24) | w);
            }
            pre_packed.push(pvec);
        }

        let sub_names = [
            "shipped_A5_only",
            "l1_A5_batch256_only",
            "shipped_A6a_512_only",
            "l1_flush_512_only",
            "l1_flush_496_only",
            "l1_flush_1008_only",
        ];
        let mut sub_cpu = vec![vec![0.0f64; repeats]; sub_names.len()];
        #[allow(clippy::needless_range_loop)]
        for r in 0..repeats {
            for step in 0..sub_names.len() {
                let idx = (step + r) % sub_names.len();
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                let mut ck = 0u64;
                match idx {
                    0 => {
                        for b in &open_blocks {
                            let n = unsafe { probe_front_avx2_shipped(b, &mut sc.front) }.unwrap();
                            ck = ck.wrapping_add(n as u64);
                        }
                    }
                    1 => {
                        for b in &open_blocks {
                            let (end, lits, _max_sym) =
                                unsafe { probe_build_bitmaps_avx2(b, &mut sc.front, usize::MAX) }
                                    .unwrap();
                            let last = end / 64;
                            let mut wi = 0usize;
                            let mut w = sc.front.lit[0];
                            let first = loop {
                                if w != 0 {
                                    let t = w.trailing_zeros() as usize;
                                    w &= w - 1;
                                    break Some(t);
                                }
                                if wi == last {
                                    break None;
                                }
                                wi += 1;
                                w = sc.front.lit[wi];
                            };
                            if let Some(tz) = first {
                                let mut front = [0u8; 32];
                                front.copy_from_slice(&b.seq_to_unseq[..32]);
                                let mut rows = MtfRows([0u8; 224]);
                                rows.0.copy_from_slice(&b.seq_to_unseq[32..]);
                                let mut st = BatchState {
                                    front,
                                    rows,
                                    wi,
                                    w,
                                    tz,
                                    last,
                                    end,
                                    slack: b.max_len / CAP as usize + 1,
                                    big: 0,
                                    extra_recs: 0,
                                    done: false,
                                    huge: None,
                                };
                                let mut batch = [0u32; 256];
                                while !st.done {
                                    let blen = unsafe {
                                        probe_fill_batch_avx2::<256>(
                                            &mut st,
                                            b.syms.as_ptr(),
                                            sc.front.lit.as_ptr(),
                                            sc.front.one.as_ptr(),
                                            b.max_len as u64,
                                            &mut batch,
                                        )
                                    }
                                    .unwrap();
                                    ck = ck.wrapping_add(blen as u64);
                                    if blen != 0 {
                                        ck = ck.wrapping_add(u64::from(batch[blen - 1]));
                                    }
                                }
                            }
                            ck = ck.wrapping_add(lits as u64);
                        }
                    }
                    2 => {
                        for (b, recs) in open_blocks.iter().zip(&pre_runs) {
                            let bound = b.max_len;
                            let strips = bound / (LIMIT as usize + 1) + 2;
                            let (f, _h, ns, _ov) = probe_a6a_from_records::<512, 512>(
                                recs,
                                b.orig_ptr,
                                &mut sc.out[..bound + INLINE],
                                &mut sc.spill512,
                                &mut sc.edges,
                                bound as u32,
                                strips,
                            );
                            ck = ck.wrapping_add(u64::from(f.m)).wrapping_add(ns as u64);
                        }
                    }
                    3 => {
                        for (b, pk) in open_blocks.iter().zip(&pre_packed) {
                            let bound = b.max_len;
                            let strips = bound / (LIMIT as usize + 1) + 2;
                            let mut pass = Pass::<512, 512>::new(
                                &mut sc.out[..bound + INLINE],
                                &mut sc.spill512,
                                &mut sc.edges,
                                bound as u32,
                                strips,
                            );
                            let mut pos = 0u32;
                            for chunk in pk.chunks(256) {
                                probe_flush_batch_direct(chunk, &mut pass, &mut pos, b.orig_ptr);
                            }
                            ck = ck
                                .wrapping_add(u64::from(pass.f.m))
                                .wrapping_add(pass.ns as u64);
                        }
                    }
                    4 => {
                        for (b, pk) in open_blocks.iter().zip(&pre_packed) {
                            let bound = b.max_len;
                            let strips = bound / (LIMIT as usize + 1) + 2;
                            let mut pass = Pass::<496, 512>::new(
                                &mut sc.out[..bound + INLINE],
                                &mut sc.spill512,
                                &mut sc.edges,
                                bound as u32,
                                strips,
                            );
                            let mut pos = 0u32;
                            for chunk in pk.chunks(256) {
                                probe_flush_batch_direct(chunk, &mut pass, &mut pos, b.orig_ptr);
                            }
                            ck = ck
                                .wrapping_add(u64::from(pass.f.m))
                                .wrapping_add(pass.ns as u64);
                        }
                    }
                    _ => {
                        for (b, pk) in open_blocks.iter().zip(&pre_packed) {
                            let bound = b.max_len;
                            let strips = bound / (LIMIT as usize + 1) + 2;
                            let mut pass = Pass::<1008, 1024>::new(
                                &mut sc.out[..bound + INLINE],
                                &mut sc.spill1024,
                                &mut sc.edges,
                                bound as u32,
                                strips,
                            );
                            let mut pos = 0u32;
                            for chunk in pk.chunks(256) {
                                probe_flush_batch_direct(chunk, &mut pass, &mut pos, b.orig_ptr);
                            }
                            ck = ck
                                .wrapping_add(u64::from(pass.f.m))
                                .wrapping_add(pass.ns as u64);
                        }
                    }
                }
                black_box(ck);
                let dt_w = t0.elapsed().as_secs_f64() * 1e3;
                let c1 = clock.now_ns();
                sub_cpu[idx][r] = match (c0, c1) {
                    (Some(a), Some(b)) => (b.saturating_sub(a)) as f64 / 1e6,
                    _ => dt_w,
                };
            }
        }
        for (name, vals) in sub_names.iter().zip(&sub_cpu) {
            let (cm, clo, chi) = stats(vals);
            println!("  {name:<24} cpu={cm:6.2} [{clo:6.2}..{chi:6.2}] ms");
        }
    }
}
