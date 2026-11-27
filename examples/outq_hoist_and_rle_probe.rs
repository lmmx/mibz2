//! Smooth-zone probe (`explore` / `mtf-fold-l1-batch-redesign` falsification & `OutQ32` census):
//! zero `src/` edits, 100% exactness verified before timing on all 233 Open blocks of
//! `corpora/wikidata.bz2` (+ synthetic stress tests: non-injective `seq_to_unseq`, marked-row
//! inside thin & thick runs, epoch boundaries, dense caps, `w > 16`).
//!
//! Tests:
//! 1. Part 1 (A5 + A6a):
//!    - `shipped_split`: production `runs::front` -> `maximal` -> `Pass::take` + `OutQ16<512,512>`
//!    - `direct_fold_1010`: direct inline `FoldSink` inside `front_avx2` (`#1010` reproduction)
//!    - `batch256_q512`: decoupled `[u32; 256]` L1d batch -> `OutQ16<512,512>` (isolates batching)
//!    - `batch{128,256,512,1024}_q496`: L1d batch + `OutQ16<496,512>` guard zone (`mtf-fold-l1-batch-redesign`)
//!    - `batch256_q1008`: L1d batch + `OutQ16<1008,1024>` guard zone
//!    - `batch256_q496_hoist`: L1d batch + `OutQ16<496,512>` + batch-hoisted `p`/epoch + 1-branch push
//!    - `batch256_q496_avx2`: same with `#[target_feature(enable = "avx2")]` on `flush` (0 `vzeroupper`)
//!    - `batch256_outq32`: L1d batch + run-encoded `OutQ32` (`(m << 4) | (w - 1)`, 8x less write traffic, 0 epochs/cuts)
//!    - `batch256_outq32_hoist`: L1d batch + `OutQ32` + batch-hoisted `p`
//!    - `shipped_outq32`: shipped `runs::front` + `OutQ32` (isolates `OutQ32` without fold)
//! 2. Part 2 (Joint A5 + A6a + A6c on real Wikidata `bands` for BOTH `wire_xor` bidi AND `wire` forward):
//!    - Verifies bit-for-bit identical `edges[0..m]` and `predecessor` across all 233 Open blocks.
//!    - Times end-to-end `A5 + A6a + A6c_bidi` and `A5 + A6a + A6c_fwd` so work moved to A6c is paid in full.
//!
//! Usage:
//!   cargo run --release --example outq_hoist_and_rle_probe -- corpora/wikidata.bz2 [REPEATS]
//!   RUSTFLAGS="-C target-cpu=native" cargo run --release --example outq_hoist_and_rle_probe -- corpora/wikidata.bz2 [REPEATS]

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("outq_hoist_and_rle_probe requires x86_64 with AVX2/BMI1/BMI2/LZCNT/POPCNT");
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use mibz2::block::Symbols;
    use mibz2::runs::{self, Runs};
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::Kernels;
    use std::arch::x86_64::*;
    use std::hint::black_box;
    use std::time::Instant;

    const CAP: u32 = 127;
    const LIMIT: u32 = 16;
    const INLINE: usize = 16;
    const LAZY_DENSITY: (usize, usize) = (3, 20);
    const DENSE: (usize, usize) = (7, 25);
    const DENSE_M: u32 = 256_000;
    const DENSE_STRIPS: usize = 16_384;

    const NONE_CHUNK: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;
    const OVF_BIT: u32 = 1 << 31;

    const PAGE: usize = 64;
    const TERMINAL: u16 = 256;

    // ---- Linux on-CPU thread clock (schedstat) ----

    fn thread_cpu_ns() -> Option<u64> {
        let s = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
        s.split_whitespace().next()?.parse::<u64>().ok()
    }

    // ---- Admission mirror (matching src/open.rs) ----

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

    fn open_cap(records: usize, n: usize, p: u32) -> Option<Cap> {
        if n == 0 || n > 900_000 || p as usize >= n {
            None
        } else if n >= 65_536 && records * 64 >= n && records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0
        {
            Some(Cap::NONE)
        } else if records * 64 >= n && records * DENSE.1 < n * DENSE.0 {
            Some(Cap::dense(n))
        } else {
            None
        }
    }

    fn is_injective_s2u(b: &Symbols) -> bool {
        let mut seen = [0u64; 4];
        let mut count = 0usize;
        for &byte in &b.seq_to_unseq[..256] {
            let w = (byte >> 6) as usize;
            let bit = 1u64 << (byte & 63);
            if seen[w] & bit != 0 {
                return false;
            }
            seen[w] |= bit;
            count += 1;
            if count == 256 {
                break;
            }
        }
        true
    }

    /// Check whether active `seq_to_unseq[0..n_in_use]` has distinct bytes.
    /// In bzip2, `eob = n_in_use + 1`, so active symbols are `0..b.eob - 1` (clamped to 256).
    fn is_active_s2u_injective(b: &Symbols) -> bool {
        let active = (b.eob.saturating_sub(1) as usize).min(256);
        let mut seen = [0u64; 4];
        for &byte in &b.seq_to_unseq[..active] {
            let w = (byte >> 6) as usize;
            let bit = 1u64 << (byte & 63);
            if seen[w] & bit != 0 {
                return false;
            }
            seen[w] |= bit;
        }
        true
    }

    // ---- MTF STEPS table (exact mirror of src/runs.rs) ----

    #[repr(C, align(64))]
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

    // ---- Shared First state & bands (for A6c exact verification & timing) ----

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

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let start = total;
            total += w;
            start
        })
    }

    #[derive(Clone, Copy)]
    struct Tile {
        end: u32,
        delta: u32,
        symbol: u16,
    }

    #[derive(Clone, Copy)]
    struct Band {
        end: u32,
        dst: u32,
        code: u32,
    }

    /// Build exact production Open `(bands, cursors, bits)` from Pass-1 `(f, edges[..3*ns])`
    /// using `src/open.rs::Kernel::trace` (non-induced; identical bands/cursors to `induced`).
    fn build_real_bands(
        f: &First,
        ns: usize,
        edges_strips: &[u32],
        n: u32,
    ) -> (Vec<Band>, [usize; 256], u32) {
        let all = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
        let first = starts(all);
        let mut write = starts(f.thick);
        let mut strips = edges_strips[..3 * ns].to_vec();
        let mut image_order = vec![0u32; ns];
        let mut tiles = Vec::with_capacity(2 * ns + 1);
        let (mut last_x, mut retained_x) = (0u32, 0u32);
        for i in 0..ns {
            let sw = &mut strips[3 * i..3 * i + 3];
            let byte = (sw[2] & 0xFF) as u8;
            let width = sw[2] >> 8;
            let from = sw[0];
            let a = usize::from(byte);
            sw[1] += first[a];
            let to = sw[1];
            image_order[write[a] as usize] = i as u32;
            write[a] += 1;
            if from > last_x {
                tiles.push(Tile {
                    end: from,
                    delta: retained_x.wrapping_sub(last_x),
                    symbol: TERMINAL,
                });
                retained_x += from - last_x;
            }
            last_x = from + width;
            tiles.push(Tile {
                end: last_x,
                delta: to.wrapping_sub(from),
                symbol: u16::from(byte),
            });
        }
        if last_x < n {
            tiles.push(Tile {
                end: n,
                delta: retained_x.wrapping_sub(last_x),
                symbol: TERMINAL,
            });
        }
        let mut directory = Vec::with_capacity(n as usize / PAGE + 1);
        let mut ti = 0usize;
        for x in (0..n).step_by(PAGE) {
            while tiles[ti].end <= x {
                ti += 1;
            }
            directory.push(ti as u32);
        }

        let mut bands = Vec::new();
        let mut cursors = [0usize; 256];
        let mut next_code = 256u32;
        let (mut k, mut si) = (0u32, 0usize);
        let mut trace_range = |mut x0: u32, end: u32, c: u8, k: &mut u32, bands: &mut Vec<Band>| {
            while x0 < end {
                let mut width = end - x0;
                let mut x = x0;
                let mut height = 0usize;
                let exit = loop {
                    let mut q = directory[x as usize / PAGE] as usize;
                    while x >= tiles[q].end {
                        q += 1;
                    }
                    let t = tiles[q];
                    width = width.min(t.end - x);
                    x = x.wrapping_add(t.delta);
                    if t.symbol == TERMINAL {
                        break x;
                    }
                    height += 1;
                };
                let code = if height == 0 {
                    u32::from(c)
                } else {
                    let cd = next_code;
                    next_code += 1;
                    cd
                };
                bands.push(Band {
                    end: *k + width,
                    dst: exit,
                    code,
                });
                *k += width;
                x0 += width;
            }
        };
        for a in 0..256 {
            cursors[a] = bands.len();
            let mut x = first[a];
            while si < write[a] as usize {
                let i = image_order[si] as usize;
                let to = strips[3 * i + 1];
                let width = strips[3 * i + 2] >> 8;
                trace_range(x, to, a as u8, &mut k, &mut bands);
                x = to + width;
                si += 1;
            }
            trace_range(x, first[a] + all[a], a as u8, &mut k, &mut bands);
        }
        assert_eq!(k, f.m);
        let bits = (u32::BITS - (f.m - 1).leading_zeros()).max(1);
        (bands, cursors, bits)
    }

    // ---- OutQ16 with configurable (C = usable ranks/chunk, S = power-of-2 stride ranks/chunk) ----

    #[derive(Default)]
    struct Spill16 {
        next: Vec<u32>,
        ovf: Vec<Vec<u8>>,
        cuts: Vec<[u32; 256]>,
    }

    struct Heads {
        heads: [u32; 256],
        in_out: u32,
    }

    struct Queues16<'a, const C: usize, const S: usize> {
        out: &'a mut [u8],
        spill: &'a mut Spill16,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        tails: [u32; 256],
        fill: [u32; 256],
        limit: u32,
        over: bool,
    }

    impl<'a, const C: usize, const S: usize> Queues16<'a, C, S> {
        const SBYTES: usize = 2 * S;
        const HAS_GUARD: bool = S >= C + 16;

        fn new(out: &'a mut [u8], spill: &'a mut Spill16, limit: u32) -> Self {
            spill.ovf.clear();
            spill.cuts.clear();
            Queues16 {
                in_out: (out.len() / Self::SBYTES) as u32,
                out,
                spill,
                chunks: 0,
                heads: [NONE_CHUNK; 256],
                tails: [NONE_CHUNK; 256],
                fill: [C as u32; 256],
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
                f + w as usize <= C
            } else {
                f + 16 <= C
            };
            if w <= 16 && room_ok && lo + w < EPOCH && tail < self.in_out {
                let at = tail as usize * Self::SBYTES + 2 * f;
                let iota: [u16; 16] = std::array::from_fn(|i| (lo as u16).wrapping_add(i as u16));
                for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                self.fill[a] = (f as u32) + w;
            } else {
                self.push_slow(a, m, w, thin);
            }
        }

        /// Single-branch fast path for hoisted batches (caller guarantees `w <= 16`,
        /// `lo + w < EPOCH`, and `fill[a]` has `OVF_BIT` set if in `ovf` or `C` if full/uninit).
        #[inline(always)]
        fn push_hoisted(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            debug_assert!(
                Self::HAS_GUARD && (1..=16).contains(&w) && (m & (EPOCH - 1)) + w < EPOCH
            );
            let f = self.fill[a];
            let nf = f.wrapping_add(w);
            if nf <= C as u32 {
                let lo = m as u16;
                let tail = self.tails[a] as usize;
                let at = tail * Self::SBYTES + 2 * (f as usize);
                let iota: [u16; 16] = std::array::from_fn(|i| lo.wrapping_add(i as u16));
                for (d, v) in self.out[at..at + 32].chunks_exact_mut(2).zip(iota) {
                    d.copy_from_slice(&v.to_le_bytes());
                }
                self.fill[a] = nf;
            } else {
                self.push_slow_tagged(a, m, w, thin);
            }
        }

        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                if self.fill[a] as usize == C {
                    self.take_chunk(a, false);
                }
                let f = self.fill[a] as usize;
                let lo = m & (EPOCH - 1);
                let take = left.min((C - f) as u32).min(EPOCH - lo);
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

        #[cold]
        #[inline(never)]
        fn push_slow_tagged(&mut self, a: usize, mut m: u32, w: u32, thin: &[u32; 256]) {
            if self.over || m + w > self.limit {
                self.over = true;
                return;
            }
            let mut left = w;
            while left > 0 {
                let raw = self.fill[a];
                let mut f = (raw & !OVF_BIT) as usize;
                let mut is_ovf = raw & OVF_BIT != 0;
                if f == C {
                    is_ovf = self.take_chunk(a, true);
                    f = 0;
                }
                let lo = m & (EPOCH - 1);
                let take = left.min((C - f) as u32).min(EPOCH - lo);
                let chunk = self.chunk_mut(self.tails[a]);
                for (i, d) in chunk[2 * f..2 * (f + take as usize)]
                    .chunks_exact_mut(2)
                    .enumerate()
                {
                    d.copy_from_slice(&((lo + i as u32) as u16).to_le_bytes());
                }
                let nf = f as u32 + take;
                self.fill[a] = if is_ovf { nf | OVF_BIT } else { nf };
                m += take;
                left -= take;
                if m % EPOCH == 0 {
                    let mut cut = *thin;
                    cut[a] += w - left;
                    self.spill.cuts.push(cut);
                }
            }
        }

        fn take_chunk(&mut self, a: usize, tag_ovf: bool) -> bool {
            let id = self.chunks;
            self.chunks += 1;
            let is_ovf = id >= self.in_out;
            if is_ovf && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill.ovf.push(vec![0u8; Self::SBYTES]);
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
            self.fill[a] = if tag_ovf && is_ovf { OVF_BIT } else { 0 };
            is_ovf
        }

        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * Self::SBYTES;
                &mut self.out[at..at + Self::SBYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    struct Reader16<'a, const C: usize, const S: usize> {
        out: &'a [u8],
        spill: &'a Spill16,
        in_out: u32,
        a: usize,
        id: u32,
        at: usize,
        read: u32,
        epoch: u32,
        cut: u32,
    }

    impl<'a, const C: usize, const S: usize> Reader16<'a, C, S> {
        const SBYTES: usize = 2 * S;

        fn new(out: &'a [u8], spill: &'a Spill16, heads: &Heads, a: usize) -> Self {
            Reader16 {
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

        #[inline(always)]
        fn wire_xor(
            &mut self,
            edges: &mut [u32],
            dst_start: usize,
            span: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            if self.at + span <= C && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = self.epoch << 16;
                let at = self.id as usize * Self::SBYTES + 2 * self.at;
                let bytes = &self.out[at..at + 2 * span];
                let mut before = *predecessor;
                for (i, s) in bytes.chunks_exact(2).enumerate() {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        before = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                *predecessor = before;
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_xor_slow(edges, dst_start, span, pack, mark, predecessor);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_xor_slow(
            &mut self,
            edges: &mut [u32],
            mut dst_start: usize,
            mut left: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            while left > 0 {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == C {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = left.min(C - self.at).min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * Self::SBYTES;
                    &self.out[at..at + Self::SBYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = self.epoch << 16;
                for (i, s) in chunk[2 * self.at..2 * (self.at + take)]
                    .chunks_exact(2)
                    .enumerate()
                {
                    let u = dst_start + i;
                    let v = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                    if v == mark {
                        *predecessor = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                }
                self.at += take;
                self.read += take as u32;
                dst_start += take;
                left -= take;
            }
        }

        #[inline(always)]
        fn wire_fwd(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= C && self.read + span as u32 <= self.cut && self.id < self.in_out {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * Self::SBYTES + 2 * self.at;
                for (e, s) in dst
                    .iter_mut()
                    .zip(self.out[at..at + 2 * span].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_fwd_slow(dst, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_fwd_slow(&mut self, mut dst: &mut [u32], pack: u32) {
            while !dst.is_empty() {
                while self.read == self.cut {
                    self.epoch += 1;
                    self.cut = self
                        .spill
                        .cuts
                        .get(self.epoch as usize)
                        .map_or(u32::MAX, |c| c[self.a]);
                }
                if self.at == C {
                    self.id = self.spill.next[self.id as usize];
                    self.at = 0;
                }
                let take = dst
                    .len()
                    .min(C - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * Self::SBYTES;
                    &self.out[at..at + Self::SBYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let hi = pack | (self.epoch << 16);
                let (now, rest) = dst.split_at_mut(take);
                for (e, s) in now
                    .iter_mut()
                    .zip(chunk[2 * self.at..2 * (self.at + take)].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += take;
                self.read += take as u32;
                dst = rest;
            }
        }
    }

    fn drain_q16<const C: usize, const S: usize>(
        out: &[u8],
        spill: &Spill16,
        heads: &Heads,
        thin: &[u32; 256],
    ) -> Vec<u32> {
        let total: usize = thin.iter().map(|&x| x as usize).sum();
        let mut res = Vec::with_capacity(total);
        for (a, &count) in thin.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let mut r = Reader16::<C, S>::new(out, spill, heads, a);
            let mut tmp = vec![0u32; count as usize];
            r.wire_fwd(&mut tmp, 0);
            res.extend_from_slice(&tmp);
        }
        res
    }

    // ---- NEW: Run-encoded OutQ32 (`(m << 4) | (w - 1)`, 4B/run, 0 epochs/cuts, 8x less write traffic) ----

    #[derive(Default)]
    struct Spill32 {
        next: Vec<u32>,
        ovf: Vec<Vec<u8>>,
    }

    struct Queues32<'a, const CH: usize> {
        out: &'a mut [u8],
        spill: &'a mut Spill32,
        in_out: u32,
        chunks: u32,
        heads: [u32; 256],
        tails: [u32; 256],
        /// entries in last chunk; `CH` before first, bit 31 set if tail is in `ovf`
        fill: [u32; 256],
        limit: u32,
        over: bool,
    }

    impl<'a, const CH: usize> Queues32<'a, CH> {
        const BYTES: usize = 4 * CH;

        fn new(out: &'a mut [u8], spill: &'a mut Spill32, limit: u32) -> Self {
            spill.ovf.clear();
            Queues32 {
                in_out: (out.len() / Self::BYTES) as u32,
                out,
                spill,
                chunks: 0,
                heads: [NONE_CHUNK; 256],
                tails: [NONE_CHUNK; 256],
                fill: [CH as u32; 256],
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

        /// One 4-byte store per thin run: ZERO 32B iota, ZERO EPOCH/cuts, ONE branch `f < CH`.
        #[inline(always)]
        fn push(&mut self, a: usize, m: u32, w: u32) {
            debug_assert!((1..=16).contains(&w));
            let f = self.fill[a];
            if f < CH as u32 {
                let tail = self.tails[a] as usize;
                let at = tail * Self::BYTES + (f as usize) * 4;
                let packed = (m << 4) | (w - 1);
                self.out[at..at + 4].copy_from_slice(&packed.to_le_bytes());
                self.fill[a] = f + 1;
            } else {
                self.push_slow(a, m, w);
            }
        }

        #[cold]
        #[inline(never)]
        fn push_slow(&mut self, a: usize, mut m: u32, mut left: u32) {
            if self.over || m + left > self.limit {
                self.over = true;
                return;
            }
            while left > 0 {
                let raw = self.fill[a];
                let mut f = (raw & !OVF_BIT) as usize;
                let mut is_ovf = raw & OVF_BIT != 0;
                if f == CH {
                    is_ovf = self.take_chunk(a);
                    f = 0;
                }
                let take = left.min(16);
                let packed = (m << 4) | (take - 1);
                let chunk = if !is_ovf {
                    let at = self.tails[a] as usize * Self::BYTES;
                    &mut self.out[at..at + Self::BYTES]
                } else {
                    &mut self.spill.ovf[(self.tails[a] - self.in_out) as usize][..]
                };
                chunk[4 * f..4 * f + 4].copy_from_slice(&packed.to_le_bytes());
                let nf = f as u32 + 1;
                self.fill[a] = if is_ovf { nf | OVF_BIT } else { nf };
                m += take;
                left -= take;
            }
        }

        fn take_chunk(&mut self, a: usize) -> bool {
            let id = self.chunks;
            self.chunks += 1;
            let is_ovf = id >= self.in_out;
            if is_ovf && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill.ovf.push(vec![0u8; Self::BYTES]);
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
            self.fill[a] = if is_ovf { OVF_BIT } else { 0 };
            is_ovf
        }
    }

    struct Reader32<'a, const CH: usize> {
        out: &'a [u8],
        spill: &'a Spill32,
        in_out: u32,
        id: u32,
        at: usize,
        cur_m: u32,
        cur_rem: u32,
    }

    impl<'a, const CH: usize> Reader32<'a, CH> {
        const BYTES: usize = 4 * CH;

        fn new(out: &'a [u8], spill: &'a Spill32, heads: &Heads, a: usize) -> Self {
            Reader32 {
                out,
                spill,
                in_out: heads.in_out,
                id: heads.heads[a],
                at: 0,
                cur_m: 0,
                cur_rem: 0,
            }
        }

        #[inline(always)]
        fn next_chunk(&mut self) {
            self.id = self.spill.next[self.id as usize];
            self.at = 0;
        }

        #[inline(always)]
        fn wire_xor(
            &mut self,
            edges: &mut [u32],
            mut u: usize,
            mut left: usize,
            pack: u32,
            mark: u32,
            predecessor: &mut u32,
        ) {
            let mut before = *predecessor;
            if self.cur_rem > 0 {
                let take = left.min(self.cur_rem as usize);
                let mut v = self.cur_m;
                for _ in 0..take {
                    if v == mark {
                        before = u as u32;
                    }
                    edges[u] ^= pack | v;
                    edges[v as usize] ^= u as u32;
                    u += 1;
                    v += 1;
                }
                self.cur_m = v;
                self.cur_rem -= take as u32;
                left -= take;
                if left == 0 {
                    *predecessor = before;
                    return;
                }
            }
            while left > 0 {
                if self.at == CH {
                    self.next_chunk();
                }
                let chunk: &[u8] = if self.id < self.in_out {
                    let base = self.id as usize * Self::BYTES;
                    &self.out[base..base + Self::BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let mut at = self.at;
                while left > 0 && at < CH {
                    let off = at * 4;
                    let rec = u32::from_le_bytes([
                        chunk[off],
                        chunk[off + 1],
                        chunk[off + 2],
                        chunk[off + 3],
                    ]);
                    at += 1;
                    let v0 = rec >> 4;
                    let w = ((rec & 15) + 1) as usize;
                    if w <= left {
                        if w == 1 {
                            if v0 == mark {
                                before = u as u32;
                            }
                            edges[u] ^= pack | v0;
                            edges[v0 as usize] ^= u as u32;
                            u += 1;
                            left -= 1;
                        } else {
                            for v in v0..v0 + w as u32 {
                                if v == mark {
                                    before = u as u32;
                                }
                                edges[u] ^= pack | v;
                                edges[v as usize] ^= u as u32;
                                u += 1;
                            }
                            left -= w;
                        }
                    } else {
                        for v in v0..v0 + left as u32 {
                            if v == mark {
                                before = u as u32;
                            }
                            edges[u] ^= pack | v;
                            edges[v as usize] ^= u as u32;
                            u += 1;
                        }
                        self.cur_m = v0 + left as u32;
                        self.cur_rem = (w - left) as u32;
                        left = 0;
                    }
                }
                self.at = at;
            }
            *predecessor = before;
        }

        #[inline(always)]
        fn wire_fwd(&mut self, mut dst: &mut [u32], pack: u32) {
            if self.cur_rem > 0 {
                let take = dst.len().min(self.cur_rem as usize);
                for (i, e) in dst[..take].iter_mut().enumerate() {
                    *e = pack | (self.cur_m + i as u32);
                }
                self.cur_m += take as u32;
                self.cur_rem -= take as u32;
                dst = &mut dst[take..];
                if dst.is_empty() {
                    return;
                }
            }
            while !dst.is_empty() {
                if self.at == CH {
                    self.next_chunk();
                }
                let chunk: &[u8] = if self.id < self.in_out {
                    let base = self.id as usize * Self::BYTES;
                    &self.out[base..base + Self::BYTES]
                } else {
                    &self.spill.ovf[(self.id - self.in_out) as usize][..]
                };
                let mut at = self.at;
                while !dst.is_empty() && at < CH {
                    let off = at * 4;
                    let rec = u32::from_le_bytes([
                        chunk[off],
                        chunk[off + 1],
                        chunk[off + 2],
                        chunk[off + 3],
                    ]);
                    at += 1;
                    let v0 = rec >> 4;
                    let w = ((rec & 15) + 1) as usize;
                    if w <= dst.len() {
                        if w == 1 {
                            dst[0] = pack | v0;
                            dst = &mut dst[1..];
                        } else {
                            let (now, rest) = dst.split_at_mut(w);
                            for (i, e) in now.iter_mut().enumerate() {
                                *e = pack | (v0 + i as u32);
                            }
                            dst = rest;
                        }
                    } else {
                        let take = dst.len();
                        for (i, e) in dst.iter_mut().enumerate() {
                            *e = pack | (v0 + i as u32);
                        }
                        self.cur_m = v0 + take as u32;
                        self.cur_rem = (w - take) as u32;
                        dst = &mut [];
                    }
                }
                self.at = at;
            }
        }
    }

    fn drain_q32<const CH: usize>(
        out: &[u8],
        spill: &Spill32,
        heads: &Heads,
        thin: &[u32; 256],
    ) -> Vec<u32> {
        let total: usize = thin.iter().map(|&x| x as usize).sum();
        let mut res = Vec::with_capacity(total);
        for (a, &count) in thin.iter().enumerate() {
            if count == 0 {
                continue;
            }
            let mut r = Reader32::<CH>::new(out, spill, heads, a);
            let mut tmp = vec![0u32; count as usize];
            r.wire_fwd(&mut tmp, 0);
            res.extend_from_slice(&tmp);
        }
        res
    }

    // ---- Pass16 (OutQ16) & Pass32 (OutQ32) ----

    struct Pass16<'a, const C: usize, const S: usize> {
        queues: Queues16<'a, C, S>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl<'a, const C: usize, const S: usize> Pass16<'a, C, S> {
        fn new(
            out: &'a mut [u8],
            spill: &'a mut Spill16,
            edges: &'a mut Vec<u32>,
            cap: Cap,
        ) -> Self {
            Pass16 {
                queues: Queues16::new(out, spill, cap.m),
                edges,
                ns: 0,
                cap_strips: cap.strips,
                f: First::default(),
            }
        }

        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            self.f.runs += 1;
            if width <= LIMIT {
                self.piece(c, pos, width, true, p);
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.piece(c, pos, width, false, p);
            } else {
                self.take_marked(c, pos, width, p);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            if pos < p {
                self.piece(c, pos, p - pos, false, p);
            }
            self.piece(c, p, 1, true, p);
            if p + 1 < end {
                self.piece(c, p + 1, end - p - 1, false, p);
            }
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
                self.push_thick(a, c, x, w);
            }
        }

        #[inline(always)]
        fn push_thick(&mut self, a: usize, c: u8, x: u32, w: u32) {
            let f = &mut self.f;
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

        /// Batch flush (standard): `#[inline(never)]` so `front_avx2_batched` keeps 0 spills.
        #[inline(never)]
        fn flush_batch(&mut self, batch: &[u32], pos: &mut u32, p: u32) {
            let mut cur = *pos;
            for &word in batch {
                let c = (word >> 24) as u8;
                let w = word & 0x00FF_FFFF;
                self.take(c, cur, w, p);
                cur += w;
            }
            *pos = cur;
        }

        /// Batch flush compiled with AVX2 (eliminates `vzeroupper` at the batch boundary
        /// AND enables single 32B `vmovdqu` iota store even on baseline x86-64 builds!).
        #[target_feature(enable = "avx2")]
        #[inline(never)]
        unsafe fn flush_batch_avx2(
            &mut self,
            batch: &[u32],
            batch_rows: u32,
            pos: &mut u32,
            p: u32,
        ) {
            self.flush_batch_hoisted_rows_inner(batch, batch_rows, pos, p);
        }

        /// Batch flush with `batch_rows` passed from producer so `cur + batch_rows <= p || cur > p`
        /// hoists `p` on BOTH sides of the marked row (~99.6% of all batches)!
        #[inline(never)]
        fn flush_batch_hoisted_rows(
            &mut self,
            batch: &[u32],
            batch_rows: u32,
            pos: &mut u32,
            p: u32,
        ) {
            self.flush_batch_hoisted_rows_inner(batch, batch_rows, pos, p);
        }

        #[inline(always)]
        fn flush_batch_hoisted_rows_inner(
            &mut self,
            batch: &[u32],
            batch_rows: u32,
            pos: &mut u32,
            p: u32,
        ) {
            let mut cur = *pos;
            let mut m = self.f.m;
            let max_m_add = (batch.len() as u32) * LIMIT;
            let lo = m & (EPOCH - 1);
            let no_mark = cur > p || cur + batch_rows <= p;
            if lo + max_m_add < EPOCH && no_mark {
                self.f.runs += batch.len();
                for &word in batch {
                    let c = (word >> 24) as u8;
                    let w = word & 0x00FF_FFFF;
                    let a = usize::from(c);
                    if w <= LIMIT {
                        self.queues.push_hoisted(a, m, w, &self.f.thin);
                        self.f.thin[a] += w;
                        m += w;
                    } else {
                        self.push_thick(a, c, cur, w);
                    }
                    cur += w;
                }
                self.f.m = m;
                *pos = cur;
                return;
            }
            for &word in batch {
                let c = (word >> 24) as u8;
                let w = word & 0x00FF_FFFF;
                self.take_tagged(c, cur, w, p);
                cur += w;
            }
            *pos = cur;
        }

        /// Fair control for `shipped_q496_hoist`: runs `maximal(records)` with the exact same
        /// `OVF_BIT` single-branch `push_hoisted`, local `m`, and `m + w < epoch_end` check.
        #[inline(never)]
        fn take_maximal_hoisted(&mut self, records: &[u16], p: u32) {
            let mut i = 0usize;
            let mut pos = 0u32;
            let mut m = self.f.m;
            let mut epoch_end = EPOCH;
            while i < records.len() {
                let rec = records[i];
                let c = (rec >> 8) as u8;
                let mut width = u32::from(rec & 255);
                i += 1;
                while i < records.len() && (records[i] >> 8) as u8 == c {
                    width += u32::from(records[i] & 255);
                    i += 1;
                }
                self.f.runs += 1;
                if width <= LIMIT {
                    let a = usize::from(c);
                    if p.wrapping_sub(pos) < width {
                        self.f.mark = m + (p - pos);
                    }
                    if m + width < epoch_end {
                        self.queues.push_hoisted(a, m, width, &self.f.thin);
                    } else {
                        self.queues.push_slow_tagged(a, m, width, &self.f.thin);
                        epoch_end = ((m + width) & !(EPOCH - 1)) + EPOCH;
                    }
                    self.f.thin[a] += width;
                    m += width;
                } else if p.wrapping_sub(pos) >= width {
                    self.push_thick(usize::from(c), c, pos, width);
                } else {
                    self.f.m = m;
                    self.take_marked_tagged(c, pos, width, p);
                    m = self.f.m;
                    epoch_end = (m & !(EPOCH - 1)) + EPOCH;
                }
                pos += width;
            }
            self.f.m = m;
        }

        #[inline(always)]
        fn take_tagged(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            self.f.runs += 1;
            if width <= LIMIT {
                let a = usize::from(c);
                if p.wrapping_sub(pos) < width {
                    self.f.mark = self.f.m + (p - pos);
                }
                let lo = self.f.m & (EPOCH - 1);
                if lo + width < EPOCH {
                    self.queues.push_hoisted(a, self.f.m, width, &self.f.thin);
                } else {
                    self.queues
                        .push_slow_tagged(a, self.f.m, width, &self.f.thin);
                }
                self.f.thin[a] += width;
                self.f.m += width;
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.push_thick(usize::from(c), c, pos, width);
            } else {
                self.take_marked_tagged(c, pos, width, p);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked_tagged(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            let a = usize::from(c);
            if pos < p {
                self.push_thick(a, c, pos, p - pos);
            }
            self.f.mark = self.f.m;
            let lo = self.f.m & (EPOCH - 1);
            if lo + 1 < EPOCH {
                self.queues.push_hoisted(a, self.f.m, 1, &self.f.thin);
            } else {
                self.queues.push_slow_tagged(a, self.f.m, 1, &self.f.thin);
            }
            self.f.thin[a] += 1;
            self.f.m += 1;
            if p + 1 < end {
                self.push_thick(a, c, p + 1, end - p - 1);
            }
        }
    }

    struct Pass32<'a, const CH: usize> {
        queues: Queues32<'a, CH>,
        edges: &'a mut Vec<u32>,
        ns: usize,
        cap_strips: usize,
        f: First,
    }

    impl<'a, const CH: usize> Pass32<'a, CH> {
        fn new(
            out: &'a mut [u8],
            spill: &'a mut Spill32,
            edges: &'a mut Vec<u32>,
            cap: Cap,
        ) -> Self {
            Pass32 {
                queues: Queues32::new(out, spill, cap.m),
                edges,
                ns: 0,
                cap_strips: cap.strips,
                f: First::default(),
            }
        }

        #[inline(always)]
        fn take(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            self.f.runs += 1;
            if width <= LIMIT {
                let a = usize::from(c);
                if p.wrapping_sub(pos) < width {
                    self.f.mark = self.f.m + (p - pos);
                }
                self.queues.push(a, self.f.m, width);
                self.f.thin[a] += width;
                self.f.m += width;
                return;
            }
            if p.wrapping_sub(pos) >= width {
                self.push_thick(usize::from(c), c, pos, width);
            } else {
                self.take_marked(c, pos, width, p);
            }
        }

        #[cold]
        #[inline(never)]
        fn take_marked(&mut self, c: u8, pos: u32, width: u32, p: u32) {
            let end = pos + width;
            let a = usize::from(c);
            if pos < p {
                self.push_thick(a, c, pos, p - pos);
            }
            self.f.mark = self.f.m;
            self.queues.push(a, self.f.m, 1);
            self.f.thin[a] += 1;
            self.f.m += 1;
            if p + 1 < end {
                self.push_thick(a, c, p + 1, end - p - 1);
            }
        }

        #[inline(always)]
        fn push_thick(&mut self, a: usize, c: u8, x: u32, w: u32) {
            let f = &mut self.f;
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

        #[inline(never)]
        fn flush_batch(&mut self, batch: &[u32], pos: &mut u32, p: u32) {
            let mut cur = *pos;
            for &word in batch {
                let c = (word >> 24) as u8;
                let w = word & 0x00FF_FFFF;
                self.take(c, cur, w, p);
                cur += w;
            }
            *pos = cur;
        }

        #[inline(never)]
        fn flush_batch_hoisted(&mut self, batch: &[u32], batch_rows: u32, pos: &mut u32, p: u32) {
            let mut cur = *pos;
            if cur > p || cur + batch_rows <= p {
                self.f.runs += batch.len();
                for &word in batch {
                    let c = (word >> 24) as u8;
                    let w = word & 0x00FF_FFFF;
                    let a = usize::from(c);
                    if w <= LIMIT {
                        self.queues.push(a, self.f.m, w);
                        self.f.thin[a] += w;
                        self.f.m += w;
                    } else {
                        self.push_thick(a, c, cur, w);
                    }
                    cur += w;
                }
                *pos = cur;
                return;
            }
            for &word in batch {
                let c = (word >> 24) as u8;
                let w = word & 0x00FF_FFFF;
                self.take(c, cur, w, p);
                cur += w;
            }
            *pos = cur;
        }
    }

    // ---- Maximal merger for shipped `records: &[u16]` ----

    #[inline(always)]
    fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
        let mut i = 0;
        let mut pos = 0u32;
        while i < records.len() {
            let rec = records[i];
            let c = (rec >> 8) as u8;
            let mut width = u32::from(rec & 255);
            i += 1;
            while i < records.len() && (records[i] >> 8) as u8 == c {
                width += u32::from(records[i] & 255);
                i += 1;
            }
            visit(c, pos, width);
            pos += width;
        }
    }

    // ---- Reusable bitmap scratch (mirrors `Runs.{lit, one}`) ----

    #[derive(Default)]
    struct BitmapScratch {
        lit: Vec<u64>,
        one: Vec<u64>,
    }

    // ---- Direct inline FoldSink (#1010 reproduction) ----

    struct DirectFoldSink<'a, const C: usize, const S: usize> {
        pass: Pass16<'a, C, S>,
        p: u32,
        c: u8,
        pos: u32,
        width: u32,
    }

    impl<'a, const C: usize, const S: usize> DirectFoldSink<'a, C, S> {
        #[inline(always)]
        fn flush(&mut self) {
            if self.width != 0 {
                self.pass.take(self.c, self.pos, self.width, self.p);
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

    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    #[inline(never)]
    unsafe fn front_avx2_direct_1010<const C: usize, const S: usize>(
        b: &Symbols,
        bm: &mut BitmapScratch,
        sink: &mut DirectFoldSink<'_, C, S>,
    ) -> Option<(usize, usize)> {
        let m = b.syms.len();
        let sp = b.syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return None;
        }
        let words = m / 64 + 3;
        bm.lit.clear();
        bm.lit.reserve(words);
        bm.one.clear();
        bm.one.reserve(words);
        let lit32 = bm.lit.as_mut_ptr() as *mut u32;
        let one32 = bm.one.as_mut_ptr() as *mut u32;
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
        bm.lit.set_len(words);
        bm.one.set_len(words);
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
        let (mut n, mut recs, mut big) = (0u64, 0usize, 0u64);
        let litp = bm.lit.as_ptr();
        let onep = bm.one.as_ptr();
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
        sink.flush();
        if n > b.max_len as u64 {
            return None;
        }
        Some((n as usize, recs))
    }

    // ---- Decoupled L1d-batched packed maximal AVX2 front (`mtf-fold-l1-batch-redesign` + enhancements) ----

    /// Zero-spill AVX2 front writing packed maximal `(c << 24) | total` into a stack `[u32; B]`
    /// buffer and calling `flush_fn(&buf[..len], batch_rows)` every `B` runs.
    ///
    /// Requires `is_active_s2u_injective(b)` (checked once per block in O(in_use) before calling;
    /// always true for valid bzip2 streams where `seq_to_unseq` is strictly increasing).
    /// Also counts `lits` in Pass 1 (eliminating both the separate `f.lit` popcount pass AND
    /// per-literal `recs += 1` in Pass 2!).
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    #[inline(never)]
    unsafe fn front_avx2_batched<const B: usize, const TRACK_ROWS: bool>(
        b: &Symbols,
        bm: &mut BitmapScratch,
        lits_refused: usize,
        mut flush_fn: impl FnMut(&[u32], u32),
    ) -> Result<(usize, usize), bool> {
        let m = b.syms.len();
        let sp = b.syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return Err(false); // Bailed
        }

        // ---- Pass 1: bitmaps + fused popcount for `lits` & early `refused` ----
        let words = m / 64 + 3;
        bm.lit.clear();
        bm.lit.reserve(words);
        bm.one.clear();
        bm.one.reserve(words);
        let lit32 = bm.lit.as_mut_ptr() as *mut u32;
        let one32 = bm.one.as_mut_ptr() as *mut u32;
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
        let mut lits = 0usize;
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
        bm.lit.set_len(words);
        bm.one.set_len(words);
        if end == usize::MAX {
            return Err(false); // Bailed
        }
        if lits >= lits_refused {
            return Err(true); // NotOpen
        }

        // ---- Pass 2: register-clean MTF -> L1d `[u32; B]` batch ----
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
        let mut extra_recs = 0usize;
        let mut n = 0u64;
        let mut big = 0u64;
        let litp = bm.lit.as_ptr();
        let onep = bm.one.as_ptr();

        let mut buf = [0u32; B];
        let mut bi = 0usize;
        let mut batch_rows = 0u32;

        macro_rules! push_word {
            ($c:expr, $total:expr) => {{
                let tot = $total as u32;
                *buf.as_mut_ptr().add(bi) = (($c as u32) << 24) | tot;
                bi += 1;
                if TRACK_ROWS {
                    batch_rows += tot;
                }
                if bi == B {
                    flush_fn(&buf[..B], batch_rows);
                    bi = 0;
                    batch_rows = 0;
                }
            }};
        }

        macro_rules! long {
            ($c:expr, $total:expr, $is_first:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                let r = total.div_ceil(CAP as u64);
                let more = r.saturating_sub(1) as usize;
                big += total;
                if more > slack || big > b.max_len as u64 {
                    return Err(false);
                }
                slack -= more;
                extra_recs += if $is_first { r as usize } else { more };
                n += total;
                if total != 0 {
                    // total <= big <= max_len <= 900_000 < 2^24
                    push_word!(c, total);
                }
            }};
        }

        macro_rules! emit {
            ($c:expr, $total:expr, $span:expr) => {{
                let (c, total): (u64, u64) = ($c, $total);
                if total > CAP as u64 {
                    if $span > 22 {
                        return Err(false);
                    }
                    long!(c, total, false);
                } else {
                    n += total;
                    push_word!(c, total);
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
            return Err(false);
        }
        let first_total = _bzhi_u64(*onep, q as u32 + 1) - 1;
        // Note: if `first_total > 0` AND the first literal happens to have the same byte (impossible
        // since MTF index 0 is never a literal, and `seq_to_unseq` is injective!).
        long!(*s2u.as_ptr() as u64, first_total, true);

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
                        return Err(false);
                    }
                    emit!(c, count!(q, q2), q2 - q);
                    match next {
                        Some(t) => (tz, after, spw) = (t, *onep.add(wi) >> 1, sp.add(wi * 64)),
                        None => break,
                    }
                }
            }
        }
        if bi > 0 {
            flush_fn(&buf[..bi], batch_rows);
        }
        if n > b.max_len as u64 {
            return Err(false);
        }
        Ok((n as usize, lits + extra_recs))
    }

    // ---- Exact mirror of shipped `src/runs.rs::front_avx2` (for zero-src-edit paired control) ----

    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    #[inline(never)]
    unsafe fn front_avx2_shipped(
        b: &Symbols,
        bm: &mut BitmapScratch,
        records: &mut Vec<u16>,
        counts: &mut [u32; 256],
        per: &mut [u32; 256],
    ) -> Option<usize> {
        let m = b.syms.len();
        let sp = b.syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return None;
        }
        let words = m / 64 + 3;
        bm.lit.clear();
        bm.lit.reserve(words);
        bm.one.clear();
        bm.one.reserve(words);
        let lit32 = bm.lit.as_mut_ptr() as *mut u32;
        let one32 = bm.one.as_mut_ptr() as *mut u32;
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
        bm.lit.set_len(words);
        bm.one.set_len(words);
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
        records.clear();
        records.reserve(m + slack + 2);
        let out0 = records.as_mut_ptr();
        let mut out = out0;
        let mut cnt = [0u64; 256];
        let mut big = 0u64;
        let litp = bm.lit.as_ptr();
        let onep = bm.one.as_ptr();
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
                cnt[c as usize] += total | recs << 32;
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
                    cnt[c as usize] += total | 1 << 32;
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
        let mut n = 0usize;
        for i in 0..256 {
            let v = cnt[i];
            counts[i] = v as u32;
            per[i] = (v >> 32) as u32;
            n += v as u32 as usize;
        }
        if n > b.max_len {
            return None;
        }
        records.set_len(out.offset_from(out0) as usize);
        Some(n)
    }

    // ---- Concrete arm runners (all #[inline(never)] for clean objdump -d inspection) ----

    struct Scratch {
        runs: Runs,
        records: Vec<u16>,
        counts: [u32; 256],
        per: [u32; 256],
        bm: BitmapScratch,
        out: Vec<u8>,
        edges: Vec<u32>,
        spill16: Spill16,
        spill32: Spill32,
    }

    impl Default for Scratch {
        fn default() -> Self {
            Scratch {
                runs: Runs::default(),
                records: Vec::with_capacity(400_000),
                counts: [0; 256],
                per: [0; 256],
                bm: BitmapScratch::default(),
                out: Vec::with_capacity(900_000 + INLINE),
                edges: Vec::with_capacity(300_000),
                spill16: Spill16::default(),
                spill32: Spill32::default(),
            }
        }
    }

    struct OpenBlock {
        syms: Symbols,
        n: usize,
        recs: usize,
        cap: Cap,
        bands: Vec<Band>,
        cursors: [usize; 256],
        bits: u32,
        ref_first: First,
        ref_ns: usize,
        ref_strips: Vec<u32>,
        ref_q: Vec<u32>,
        ref_edges_bidi: Vec<u32>,
        ref_pred_bidi: u32,
        ref_edges_fwd: Vec<u32>,
    }

    #[inline(never)]
    fn run_shipped_split(blk: &OpenBlock, sc: &mut Scratch) -> (First, usize, Heads) {
        let n = unsafe {
            front_avx2_shipped(
                &blk.syms,
                &mut sc.bm,
                &mut sc.records,
                &mut sc.counts,
                &mut sc.per,
            )
        }
        .unwrap();
        debug_assert_eq!(n, blk.n);
        sc.out.resize(n + INLINE, 0);
        let mut pass =
            Pass16::<512, 512>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, blk.cap);
        maximal(&sc.records, |c, pos, width| {
            pass.take(c, pos, width, blk.syms.orig_ptr);
        });
        (pass.f, pass.ns, pass.queues.heads())
    }

    #[inline(never)]
    fn run_shipped_q496(blk: &OpenBlock, sc: &mut Scratch) -> (First, usize, Heads) {
        let n = unsafe {
            front_avx2_shipped(
                &blk.syms,
                &mut sc.bm,
                &mut sc.records,
                &mut sc.counts,
                &mut sc.per,
            )
        }
        .unwrap();
        sc.out.resize(n + INLINE, 0);
        let mut pass =
            Pass16::<496, 512>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, blk.cap);
        maximal(&sc.records, |c, pos, width| {
            pass.take(c, pos, width, blk.syms.orig_ptr);
        });
        (pass.f, pass.ns, pass.queues.heads())
    }

    #[inline(never)]
    fn run_shipped_q496_hoist(blk: &OpenBlock, sc: &mut Scratch) -> (First, usize, Heads) {
        let n = unsafe {
            front_avx2_shipped(
                &blk.syms,
                &mut sc.bm,
                &mut sc.records,
                &mut sc.counts,
                &mut sc.per,
            )
        }
        .unwrap();
        sc.out.resize(n + INLINE, 0);
        let mut pass =
            Pass16::<496, 512>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, blk.cap);
        pass.take_maximal_hoisted(&sc.records, blk.syms.orig_ptr);
        (pass.f, pass.ns, pass.queues.heads())
    }

    #[inline(never)]
    fn run_shipped_outq32(blk: &OpenBlock, sc: &mut Scratch) -> (First, usize, Heads) {
        let n = unsafe {
            front_avx2_shipped(
                &blk.syms,
                &mut sc.bm,
                &mut sc.records,
                &mut sc.counts,
                &mut sc.per,
            )
        }
        .unwrap();
        sc.out.resize(n + INLINE, 0);
        let mut pass = Pass32::<256>::new(&mut sc.out, &mut sc.spill32, &mut sc.edges, blk.cap);
        maximal(&sc.records, |c, pos, width| {
            pass.take(c, pos, width, blk.syms.orig_ptr);
        });
        (pass.f, pass.ns, pass.queues.heads())
    }

    /// Synthetic edge-case tests: non-injective `seq_to_unseq`, tiny `out` forcing `OVF_BIT`
    /// overflow chunks in `push_hoisted` / `push_slow_tagged`, exact `EPOCH = 65536` crossing
    /// inside a batch, and marked row `p` across thin/thick/batch-boundary positions.
    fn synthetic_stress_tests() {
        // 1. Non-injective `seq_to_unseq` detection
        let mut dup_s2u: [u8; 256] = std::array::from_fn(|i| i as u8);
        dup_s2u[5] = dup_s2u[2];
        let fake_syms = Symbols {
            syms: vec![2, 3, 6],
            seq_to_unseq: dup_s2u,
            eob: 7,
            orig_ptr: 0,
            crc: 0,
            max_len: 100,
        };
        assert!(
            !is_active_s2u_injective(&fake_syms),
            "must detect duplicate s2u"
        );

        // 2. Tiny `out` forcing `OVF_BIT` overflow chunks + `EPOCH` crossing + `p` sweep
        let mut batch = Vec::new();
        let mut total_rows = 0u32;
        for i in 0..70_000u32 {
            let c = (i % 7) as u8;
            let w = if i % 500 == 0 { 20 } else { 1 + (i % 3) };
            batch.push((u32::from(c) << 24) | w);
            total_rows += w;
        }
        for &out_len in &[1024usize, 4096, total_rows as usize + INLINE] {
            for &p in &[0u32, 1, 255, 256, 511, 512, 65_535, 65_536, total_rows - 1] {
                let mut out_ref = vec![0u8; out_len];
                let mut sp_ref = Spill16::default();
                let mut ed_ref = Vec::new();
                let mut pass_ref =
                    Pass16::<512, 512>::new(&mut out_ref, &mut sp_ref, &mut ed_ref, Cap::NONE);
                let mut pos_ref = 0u32;
                pass_ref.flush_batch(&batch, &mut pos_ref, p);
                let h_ref = pass_ref.queues.heads();
                let f_ref = pass_ref.f;
                let ns_ref = pass_ref.ns;
                let q_ref = drain_q16::<512, 512>(&out_ref, &sp_ref, &h_ref, &f_ref.thin);

                let mut out_h = vec![0u8; out_len];
                let mut sp_h = Spill16::default();
                let mut ed_h = Vec::new();
                let mut pass_h =
                    Pass16::<496, 512>::new(&mut out_h, &mut sp_h, &mut ed_h, Cap::NONE);
                let mut pos_h = 0u32;
                for chunk in batch.chunks(256) {
                    let brows: u32 = chunk.iter().map(|&w| w & 0x00FF_FFFF).sum();
                    pass_h.flush_batch_hoisted_rows(chunk, brows, &mut pos_h, p);
                }
                let h_h = pass_h.queues.heads();
                let f_h = pass_h.f;
                let ns_h = pass_h.ns;
                let q_h = drain_q16::<496, 512>(&out_h, &sp_h, &h_h, &f_h.thin);

                assert_eq!(
                    f_h, f_ref,
                    "stress First mismatch (out_len={out_len}, p={p})"
                );
                assert_eq!(ns_h, ns_ref, "stress ns mismatch");
                assert_eq!(
                    ed_h[..3 * ns_h],
                    ed_ref[..3 * ns_ref],
                    "stress strips mismatch"
                );
                assert_eq!(
                    q_h, q_ref,
                    "stress OutQ mismatch (out_len={out_len}, p={p})"
                );
                if out_len <= 4096 {
                    assert!(sp_h.ovf.len() > 100, "expected OVF_BIT chunks on tiny out");
                }
                assert!(
                    !sp_h.cuts.is_empty(),
                    "expected EPOCH cuts on >65536 thin rows"
                );
            }
        }
        println!("  [STRESS OK] non-injective s2u, OVF_BIT overflow (>100 ovf chunks), EPOCH=65536 cuts, p-sweep (27 cases)");
    }

    #[inline(never)]
    fn run_direct_1010(blk: &OpenBlock, sc: &mut Scratch) -> (First, usize, Heads) {
        sc.out.resize(blk.syms.max_len + INLINE, 0);
        let mut sink = DirectFoldSink {
            pass: Pass16::<512, 512>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, blk.cap),
            p: blk.syms.orig_ptr,
            c: 0,
            pos: 0,
            width: 0,
        };
        let (n, recs) =
            unsafe { front_avx2_direct_1010(&blk.syms, &mut sc.bm, &mut sink) }.unwrap();
        debug_assert_eq!((n, recs), (blk.n, blk.recs));
        (sink.pass.f, sink.pass.ns, sink.pass.queues.heads())
    }

    #[inline(never)]
    fn run_batch_q16<const B: usize, const C: usize, const S: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
    ) -> (First, usize, Heads) {
        sc.out.resize(blk.syms.max_len + INLINE, 0);
        let mut pass = Pass16::<C, S>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, blk.cap);
        let mut pos = 0u32;
        let p = blk.syms.orig_ptr;
        let lits_ref = (7 * blk.syms.max_len).div_ceil(25);
        let (n, recs) = unsafe {
            front_avx2_batched::<B, false>(&blk.syms, &mut sc.bm, lits_ref, |batch, _| {
                pass.flush_batch(batch, &mut pos, p);
            })
        }
        .unwrap();
        debug_assert_eq!((n, recs), (blk.n, blk.recs));
        (pass.f, pass.ns, pass.queues.heads())
    }

    #[inline(never)]
    fn run_batch_q16_hoist<const B: usize, const C: usize, const S: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
    ) -> (First, usize, Heads) {
        sc.out.resize(blk.syms.max_len + INLINE, 0);
        let mut pass = Pass16::<C, S>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, blk.cap);
        let mut pos = 0u32;
        let p = blk.syms.orig_ptr;
        let lits_ref = (7 * blk.syms.max_len).div_ceil(25);
        let (n, recs) = unsafe {
            front_avx2_batched::<B, true>(&blk.syms, &mut sc.bm, lits_ref, |batch, brows| {
                pass.flush_batch_hoisted_rows(batch, brows, &mut pos, p);
            })
        }
        .unwrap();
        debug_assert_eq!((n, recs), (blk.n, blk.recs));
        (pass.f, pass.ns, pass.queues.heads())
    }

    #[inline(never)]
    fn run_batch_q16_avx2<const B: usize, const C: usize, const S: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
    ) -> (First, usize, Heads) {
        sc.out.resize(blk.syms.max_len + INLINE, 0);
        let mut pass = Pass16::<C, S>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, blk.cap);
        let mut pos = 0u32;
        let p = blk.syms.orig_ptr;
        let lits_ref = (7 * blk.syms.max_len).div_ceil(25);
        let (n, recs) = unsafe {
            front_avx2_batched::<B, true>(&blk.syms, &mut sc.bm, lits_ref, |batch, brows| {
                pass.flush_batch_avx2(batch, brows, &mut pos, p);
            })
        }
        .unwrap();
        debug_assert_eq!((n, recs), (blk.n, blk.recs));
        (pass.f, pass.ns, pass.queues.heads())
    }

    #[inline(never)]
    fn run_batch_outq32<const B: usize, const CH: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
    ) -> (First, usize, Heads) {
        sc.out.resize(blk.syms.max_len + INLINE, 0);
        let mut pass = Pass32::<CH>::new(&mut sc.out, &mut sc.spill32, &mut sc.edges, blk.cap);
        let mut pos = 0u32;
        let p = blk.syms.orig_ptr;
        let lits_ref = (7 * blk.syms.max_len).div_ceil(25);
        let (n, recs) = unsafe {
            front_avx2_batched::<B, false>(&blk.syms, &mut sc.bm, lits_ref, |batch, _| {
                pass.flush_batch(batch, &mut pos, p);
            })
        }
        .unwrap();
        debug_assert_eq!((n, recs), (blk.n, blk.recs));
        (pass.f, pass.ns, pass.queues.heads())
    }

    #[inline(never)]
    fn run_batch_outq32_hoist<const B: usize, const CH: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
    ) -> (First, usize, Heads) {
        sc.out.resize(blk.syms.max_len + INLINE, 0);
        let mut pass = Pass32::<CH>::new(&mut sc.out, &mut sc.spill32, &mut sc.edges, blk.cap);
        let mut pos = 0u32;
        let p = blk.syms.orig_ptr;
        let lits_ref = (7 * blk.syms.max_len).div_ceil(25);
        let (n, recs) = unsafe {
            front_avx2_batched::<B, true>(&blk.syms, &mut sc.bm, lits_ref, |batch, brows| {
                pass.flush_batch_hoisted(batch, brows, &mut pos, p);
            })
        }
        .unwrap();
        debug_assert_eq!((n, recs), (blk.n, blk.recs));
        (pass.f, pass.ns, pass.queues.heads())
    }

    // ---- A6c wiring (bidi XOR & forward) ----

    #[inline(never)]
    fn wire_bidi_q16<const C: usize, const S: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
        f: &First,
        heads: &Heads,
    ) -> u32 {
        let m = f.m as usize;
        if sc.edges.len() < m {
            sc.edges.resize(m, 0);
        }
        let edges = &mut sc.edges[..m];
        edges.fill(0);
        let mut pred = f.mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = blk.cursors.get(a + 1).copied().unwrap_or(blk.bands.len());
            let mut q = Reader16::<C, S>::new(&sc.out, &sc.spill16, heads, a);
            for &band in &blk.bands[blk.cursors[a]..end] {
                q.wire_xor(
                    edges,
                    band.dst as usize,
                    (band.end - k) as usize,
                    band.code << blk.bits,
                    f.mark,
                    &mut pred,
                );
                k = band.end;
            }
        }
        pred
    }

    #[inline(never)]
    fn wire_fwd_q16<const C: usize, const S: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
        f: &First,
        heads: &Heads,
    ) {
        let m = f.m as usize;
        if sc.edges.len() < m {
            sc.edges.resize(m, 0);
        }
        let edges = &mut sc.edges[..m];
        let mut k = 0u32;
        for a in 0..256 {
            let end = blk.cursors.get(a + 1).copied().unwrap_or(blk.bands.len());
            let mut q = Reader16::<C, S>::new(&sc.out, &sc.spill16, heads, a);
            for &band in &blk.bands[blk.cursors[a]..end] {
                let span = (band.end - k) as usize;
                let dst = band.dst as usize;
                q.wire_fwd(&mut edges[dst..dst + span], band.code << blk.bits);
                k = band.end;
            }
        }
    }

    #[inline(never)]
    fn wire_bidi_q32<const CH: usize>(
        blk: &OpenBlock,
        sc: &mut Scratch,
        f: &First,
        heads: &Heads,
    ) -> u32 {
        let m = f.m as usize;
        if sc.edges.len() < m {
            sc.edges.resize(m, 0);
        }
        let edges = &mut sc.edges[..m];
        edges.fill(0);
        let mut pred = f.mark;
        let mut k = 0u32;
        for a in 0..256 {
            let end = blk.cursors.get(a + 1).copied().unwrap_or(blk.bands.len());
            let mut q = Reader32::<CH>::new(&sc.out, &sc.spill32, heads, a);
            for &band in &blk.bands[blk.cursors[a]..end] {
                q.wire_xor(
                    edges,
                    band.dst as usize,
                    (band.end - k) as usize,
                    band.code << blk.bits,
                    f.mark,
                    &mut pred,
                );
                k = band.end;
            }
        }
        pred
    }

    #[inline(never)]
    fn wire_fwd_q32<const CH: usize>(blk: &OpenBlock, sc: &mut Scratch, f: &First, heads: &Heads) {
        let m = f.m as usize;
        if sc.edges.len() < m {
            sc.edges.resize(m, 0);
        }
        let edges = &mut sc.edges[..m];
        let mut k = 0u32;
        for a in 0..256 {
            let end = blk.cursors.get(a + 1).copied().unwrap_or(blk.bands.len());
            let mut q = Reader32::<CH>::new(&sc.out, &sc.spill32, heads, a);
            for &band in &blk.bands[blk.cursors[a]..end] {
                let span = (band.end - k) as usize;
                let dst = band.dst as usize;
                q.wire_fwd(&mut edges[dst..dst + span], band.code << blk.bits);
                k = band.end;
            }
        }
    }

    // ---- Stats formatting ----

    fn stats(vals: &[f64]) -> (f64, f64, f64) {
        let mut s = vals.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[0], s[s.len() / 2], s[s.len() - 1])
    }

    fn verify_arm_a5a6a<const C: usize, const S: usize>(
        name: &str,
        blocks: &[OpenBlock],
        sc: &mut Scratch,
        mut f_run: impl FnMut(&OpenBlock, &mut Scratch) -> (First, usize, Heads),
    ) {
        let mut max_ovf = 0usize;
        let mut total_ovf = 0usize;
        for (idx, blk) in blocks.iter().enumerate() {
            let (f, ns, heads) = f_run(blk, sc);
            assert_eq!(f, blk.ref_first, "{name} blk {idx}: First mismatch");
            assert_eq!(ns, blk.ref_ns, "{name} blk {idx}: ns mismatch");
            assert_eq!(
                &sc.edges[..3 * ns],
                &blk.ref_strips[..],
                "{name} blk {idx}: strips mismatch"
            );
            let q = drain_q16::<C, S>(&sc.out, &sc.spill16, &heads, &f.thin);
            assert_eq!(q, blk.ref_q, "{name} blk {idx}: OutQ16 drain mismatch");
            let pred = wire_bidi_q16::<C, S>(blk, sc, &f, &heads);
            assert_eq!(pred, blk.ref_pred_bidi, "{name} blk {idx}: pred mismatch");
            assert_eq!(
                &sc.edges[..f.m as usize],
                &blk.ref_edges_bidi[..],
                "{name} blk {idx}: bidi edges mismatch"
            );
            wire_fwd_q16::<C, S>(blk, sc, &f, &heads);
            assert_eq!(
                &sc.edges[..f.m as usize],
                &blk.ref_edges_fwd[..],
                "{name} blk {idx}: fwd edges mismatch"
            );
            total_ovf += sc.spill16.ovf.len();
            max_ovf = max_ovf.max(sc.spill16.ovf.len());
        }
        println!(
            "  [EXACT OK] {name:<24} 233/233 First+strips+OutQ+bidi+fwd | ovf_chunks={total_ovf} (max={max_ovf})"
        );
    }

    fn verify_arm_q32<const CH: usize>(
        name: &str,
        blocks: &[OpenBlock],
        sc: &mut Scratch,
        mut f_run: impl FnMut(&OpenBlock, &mut Scratch) -> (First, usize, Heads),
    ) {
        let mut max_ovf = 0usize;
        let mut total_ovf = 0usize;
        for (idx, blk) in blocks.iter().enumerate() {
            let (f, ns, heads) = f_run(blk, sc);
            assert_eq!(f, blk.ref_first, "{name} blk {idx}: First mismatch");
            assert_eq!(ns, blk.ref_ns, "{name} blk {idx}: ns mismatch");
            assert_eq!(
                &sc.edges[..3 * ns],
                &blk.ref_strips[..],
                "{name} blk {idx}: strips mismatch"
            );
            let q = drain_q32::<CH>(&sc.out, &sc.spill32, &heads, &f.thin);
            assert_eq!(q, blk.ref_q, "{name} blk {idx}: OutQ32 drain mismatch");
            let pred = wire_bidi_q32::<CH>(blk, sc, &f, &heads);
            assert_eq!(pred, blk.ref_pred_bidi, "{name} blk {idx}: pred mismatch");
            assert_eq!(
                &sc.edges[..f.m as usize],
                &blk.ref_edges_bidi[..],
                "{name} blk {idx}: bidi edges mismatch"
            );
            wire_fwd_q32::<CH>(blk, sc, &f, &heads);
            assert_eq!(
                &sc.edges[..f.m as usize],
                &blk.ref_edges_fwd[..],
                "{name} blk {idx}: fwd edges mismatch"
            );
            total_ovf += sc.spill32.ovf.len();
            max_ovf = max_ovf.max(sc.spill32.ovf.len());
        }
        println!(
            "  [EXACT OK] {name:<24} 233/233 First+strips+OutQ+bidi+fwd | ovf_chunks={total_ovf} (max={max_ovf})"
        );
    }

    pub fn run() {
        assert!(
            std::is_x86_feature_detected!("avx2")
                && std::is_x86_feature_detected!("bmi1")
                && std::is_x86_feature_detected!("bmi2")
                && std::is_x86_feature_detected!("lzcnt")
                && std::is_x86_feature_detected!("popcnt"),
            "CPU lacks required AVX2/BMI1/BMI2/LZCNT/POPCNT"
        );
        let args: Vec<String> = std::env::args().collect();
        let path = args
            .get(1)
            .map(String::as_str)
            .unwrap_or("corpora/wikidata.bz2");
        let repeats: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(7);

        let data = std::fs::read(path).expect("failed to read bz2 corpus");
        let k = Kernels::detect();
        println!(
            "Corpus: {path} ({} bytes), repeats={repeats}, kernels={k:?}",
            data.len()
        );

        // Scan all blocks and collect Open-admitted blocks with exact ground truth
        let mut scanner = Scanner::new(SliceSource(&data));
        let mut blocks: Vec<OpenBlock> = Vec::new();
        let mut sc = Scratch::default();
        let mut total_blocks = 0usize;
        let mut non_inj = 0usize;
        let mut total_thin_runs = 0u64;
        let mut total_thin_rows = 0u64;
        let mut max_thin_runs_blk = 0u64;
        let mut sum_in_use = 0usize;

        while let Event::Block { symbols: syms, .. } = scanner.next(Vec::new()).expect("scan error")
        {
            total_blocks += 1;
            if !runs::takes_runs(&syms) {
                continue;
            }
            let n_prod = runs::front(&syms, &mut sc.runs, k).expect("runs::front");
            let n = unsafe {
                front_avx2_shipped(
                    &syms,
                    &mut sc.bm,
                    &mut sc.records,
                    &mut sc.counts,
                    &mut sc.per,
                )
            }
            .expect("front_avx2_shipped");
            assert_eq!(n, n_prod, "mirror front_avx2_shipped length mismatch");
            let recs = sc.records.len();
            let Some(cap) = open_cap(recs, n, syms.orig_ptr) else {
                continue;
            };
            sc.out.resize(n + INLINE, 0);
            let mut pass =
                Pass16::<512, 512>::new(&mut sc.out, &mut sc.spill16, &mut sc.edges, cap);
            maximal(&sc.records, |c, pos, width| {
                pass.take(c, pos, width, syms.orig_ptr);
            });
            let (ref_first, ref_ns, heads, over) =
                (pass.f, pass.ns, pass.queues.heads(), pass.queues.over);
            if over || ref_first.m > cap.m || ref_ns > cap.strips {
                continue;
            }
            if !is_active_s2u_injective(&syms) {
                non_inj += 1;
            }
            let _ = is_injective_s2u(&syms);
            sum_in_use += syms.eob.saturating_sub(1) as usize;
            let blk_thin_runs = (ref_first.runs - ref_ns) as u64;
            total_thin_runs += blk_thin_runs;
            total_thin_rows += ref_first.m as u64;
            max_thin_runs_blk = max_thin_runs_blk.max(blk_thin_runs);

            let ref_strips = sc.edges[..3 * ref_ns].to_vec();
            let ref_q = drain_q16::<512, 512>(&sc.out, &sc.spill16, &heads, &ref_first.thin);
            let (bands, cursors, bits) =
                build_real_bands(&ref_first, ref_ns, &ref_strips, n as u32);
            let mut tmp_blk = OpenBlock {
                syms,
                n,
                recs,
                cap,
                bands,
                cursors,
                bits,
                ref_first,
                ref_ns,
                ref_strips,
                ref_q,
                ref_edges_bidi: Vec::new(),
                ref_pred_bidi: 0,
                ref_edges_fwd: Vec::new(),
            };
            let pred = wire_bidi_q16::<512, 512>(&tmp_blk, &mut sc, &ref_first, &heads);
            tmp_blk.ref_pred_bidi = pred;
            tmp_blk.ref_edges_bidi = sc.edges[..ref_first.m as usize].to_vec();
            wire_fwd_q16::<512, 512>(&tmp_blk, &mut sc, &ref_first, &heads);
            tmp_blk.ref_edges_fwd = sc.edges[..ref_first.m as usize].to_vec();
            blocks.push(tmp_blk);
        }

        println!(
            "Scanned {total_blocks} blocks -> {} Open blocks (non_injective={non_inj}, avg in_use={:.1}, thin_runs={total_thin_runs}, max_thin_runs/blk={max_thin_runs_blk}, thin_rows={total_thin_rows}, avg_w={:.3})",
            blocks.len(),
            sum_in_use as f64 / blocks.len().max(1) as f64,
            total_thin_rows as f64 / total_thin_runs.max(1) as f64,
        );

        // ---- 1. Exactness verification across ALL arms + synthetic stress ----
        println!("\n=== Exactness verification (233/233 Open blocks + synthetic stress) ===");
        synthetic_stress_tests();
        verify_arm_a5a6a::<512, 512>("shipped_split", &blocks, &mut sc, run_shipped_split);
        verify_arm_a5a6a::<496, 512>("shipped_q496", &blocks, &mut sc, run_shipped_q496);
        verify_arm_a5a6a::<496, 512>(
            "shipped_q496_hoist",
            &blocks,
            &mut sc,
            run_shipped_q496_hoist,
        );
        verify_arm_a5a6a::<512, 512>("direct_fold_1010", &blocks, &mut sc, run_direct_1010);
        verify_arm_a5a6a::<512, 512>(
            "batch256_q512",
            &blocks,
            &mut sc,
            run_batch_q16::<256, 512, 512>,
        );
        verify_arm_a5a6a::<496, 512>(
            "batch128_q496",
            &blocks,
            &mut sc,
            run_batch_q16::<128, 496, 512>,
        );
        verify_arm_a5a6a::<496, 512>(
            "batch256_q496",
            &blocks,
            &mut sc,
            run_batch_q16::<256, 496, 512>,
        );
        verify_arm_a5a6a::<496, 512>(
            "batch512_q496",
            &blocks,
            &mut sc,
            run_batch_q16::<512, 496, 512>,
        );
        verify_arm_a5a6a::<496, 512>(
            "batch1024_q496",
            &blocks,
            &mut sc,
            run_batch_q16::<1024, 496, 512>,
        );
        verify_arm_a5a6a::<1008, 1024>(
            "batch256_q1008",
            &blocks,
            &mut sc,
            run_batch_q16::<256, 1008, 1024>,
        );
        verify_arm_a5a6a::<496, 512>(
            "batch256_q496_hoist",
            &blocks,
            &mut sc,
            run_batch_q16_hoist::<256, 496, 512>,
        );
        verify_arm_a5a6a::<1008, 1024>(
            "batch256_q1008_hoist",
            &blocks,
            &mut sc,
            run_batch_q16_hoist::<256, 1008, 1024>,
        );
        verify_arm_a5a6a::<496, 512>(
            "batch256_q496_avx2",
            &blocks,
            &mut sc,
            run_batch_q16_avx2::<256, 496, 512>,
        );
        verify_arm_q32::<256>("shipped_outq32", &blocks, &mut sc, run_shipped_outq32);
        verify_arm_q32::<256>(
            "batch256_outq32",
            &blocks,
            &mut sc,
            run_batch_outq32::<256, 256>,
        );
        verify_arm_q32::<256>(
            "batch256_outq32_hoist",
            &blocks,
            &mut sc,
            run_batch_outq32_hoist::<256, 256>,
        );
        verify_arm_q32::<512>(
            "batch256_outq32_512",
            &blocks,
            &mut sc,
            run_batch_outq32_hoist::<256, 512>,
        );

        // ---- 2. Part 1: Interleaved paired A5 + A6a benchmark ----
        type ArmFn = (&'static str, Box<dyn Fn(&[OpenBlock], &mut Scratch) -> u64>);
        let a5a6a_arms: Vec<ArmFn> = vec![
            (
                "shipped_split",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_shipped_split(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "shipped_q496",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_shipped_q496(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "shipped_q496_hoist",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_shipped_q496_hoist(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "direct_fold_1010",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_direct_1010(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_q512",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16::<256, 512, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch128_q496",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16::<128, 496, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_q496",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16::<256, 496, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch512_q496",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16::<512, 496, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch1024_q496",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16::<1024, 496, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_q1008",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16::<256, 1008, 1024>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_q496_hoist",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16_hoist::<256, 496, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_q1008_hoist",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16_hoist::<256, 1008, 1024>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_q496_avx2",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_q16_avx2::<256, 496, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "shipped_outq32",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_shipped_outq32(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_outq32",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_outq32::<256, 256>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_outq32_hoist",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_outq32_hoist::<256, 256>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
            (
                "batch256_outq32_512",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, ns, _) = run_batch_outq32_hoist::<256, 512>(b, s);
                        acc = acc.wrapping_add(f.m as u64 ^ ns as u64);
                    }
                    acc
                }),
            ),
        ];

        // Warmup
        for (_, f) in &a5a6a_arms {
            black_box(f(&blocks, &mut sc));
        }

        let n_arms = a5a6a_arms.len();
        let mut wall_ms = vec![Vec::with_capacity(repeats); n_arms];
        let mut cpu_ms = vec![Vec::with_capacity(repeats); n_arms];

        for r in 0..repeats {
            let order: Vec<usize> = if r % 2 == 0 {
                (0..n_arms).collect()
            } else {
                (0..n_arms).rev().collect()
            };
            for i in order {
                let c0 = thread_cpu_ns();
                let t0 = Instant::now();
                let acc = (a5a6a_arms[i].1)(&blocks, &mut sc);
                let dt = t0.elapsed().as_secs_f64() * 1e3;
                let dc = match (c0, thread_cpu_ns()) {
                    (Some(a), Some(b)) => (b.saturating_sub(a)) as f64 * 1e-6,
                    _ => dt,
                };
                black_box(acc);
                wall_ms[i].push(dt);
                cpu_ms[i].push(dc);
            }
        }

        println!("\n=== PART 1: A5 + A6a (233 Open blocks, {repeats} interleaved repeats) ===");
        println!(
            "{:<24} {:>10} {:>22} {:>10} {:>10} {:>22}",
            "Arm", "CPU med", "CPU [min..max]", "Ratio", "Delta ms", "Wall med [min..max]"
        );
        let (_, base_med, _) = stats(&cpu_ms[0]);
        for (i, (name, _)) in a5a6a_arms.iter().enumerate() {
            let (cmin, cmed, cmax) = stats(&cpu_ms[i]);
            let (wmin, wmed, wmax) = stats(&wall_ms[i]);
            let ratios: Vec<f64> = (0..repeats).map(|r| cpu_ms[i][r] / cpu_ms[0][r]).collect();
            let (rmin, rmed, rmax) = stats(&ratios);
            println!(
                "{:<24} {:>8.2}ms [{:>6.2}..{:>6.2}] {:>6.4}x ({:>+6.2}%) {:>+7.2}ms {:>6.2} [{:>6.2}..{:>6.2}]  paired=[{:.4}..{:.4}]",
                name,
                cmed,
                cmin,
                cmax,
                rmed,
                (rmed - 1.0) * 100.0,
                cmed - base_med,
                wmed,
                wmin,
                wmax,
                rmin,
                rmax,
            );
        }

        // ---- 3. Part 2: Joint A5 + A6a + A6c (BOTH bidi wire_xor AND forward wire) ----
        let joint_arms: Vec<ArmFn> = vec![
            (
                "bidi_shipped_q512",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_shipped_split(b, s);
                        let pred = wire_bidi_q16::<512, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(pred as u64 ^ s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "bidi_shipped_q496_h",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_shipped_q496_hoist(b, s);
                        let pred = wire_bidi_q16::<496, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(pred as u64 ^ s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "bidi_batch256_q496",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_batch_q16::<256, 496, 512>(b, s);
                        let pred = wire_bidi_q16::<496, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(pred as u64 ^ s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "bidi_batch256_q496_h",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_batch_q16_hoist::<256, 496, 512>(b, s);
                        let pred = wire_bidi_q16::<496, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(pred as u64 ^ s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "bidi_batch256_outq32",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_batch_outq32_hoist::<256, 256>(b, s);
                        let pred = wire_bidi_q32::<256>(b, s, &f, &heads);
                        acc = acc.wrapping_add(pred as u64 ^ s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "fwd_shipped_q512",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_shipped_split(b, s);
                        wire_fwd_q16::<512, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "fwd_shipped_q496_h",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_shipped_q496_hoist(b, s);
                        wire_fwd_q16::<496, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "fwd_batch256_q496",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_batch_q16::<256, 496, 512>(b, s);
                        wire_fwd_q16::<496, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "fwd_batch256_q496_h",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_batch_q16_hoist::<256, 496, 512>(b, s);
                        wire_fwd_q16::<496, 512>(b, s, &f, &heads);
                        acc = acc.wrapping_add(s.edges[0] as u64);
                    }
                    acc
                }),
            ),
            (
                "fwd_batch256_outq32",
                Box::new(|blks, s| {
                    let mut acc = 0u64;
                    for b in blks {
                        let (f, _, heads) = run_batch_outq32_hoist::<256, 256>(b, s);
                        wire_fwd_q32::<256>(b, s, &f, &heads);
                        acc = acc.wrapping_add(s.edges[0] as u64);
                    }
                    acc
                }),
            ),
        ];

        for (_, f) in &joint_arms {
            black_box(f(&blocks, &mut sc));
        }
        let nj = joint_arms.len();
        let mut j_wall = vec![Vec::with_capacity(repeats); nj];
        let mut j_cpu = vec![Vec::with_capacity(repeats); nj];
        for r in 0..repeats {
            let order: Vec<usize> = if r % 2 == 0 {
                (0..nj).collect()
            } else {
                (0..nj).rev().collect()
            };
            for i in order {
                let c0 = thread_cpu_ns();
                let t0 = Instant::now();
                let acc = (joint_arms[i].1)(&blocks, &mut sc);
                let dt = t0.elapsed().as_secs_f64() * 1e3;
                let dc = match (c0, thread_cpu_ns()) {
                    (Some(a), Some(b)) => (b.saturating_sub(a)) as f64 * 1e-6,
                    _ => dt,
                };
                black_box(acc);
                j_wall[i].push(dt);
                j_cpu[i].push(dc);
            }
        }

        println!("\n=== PART 2: Joint A5 + A6a + A6c (real Wikidata bands, {repeats} interleaved repeats) ===");
        println!(
            "{:<24} {:>10} {:>22} {:>10} {:>10} {:>22}",
            "Arm", "CPU med", "CPU [min..max]", "Ratio", "Delta ms", "Paired [min..max]"
        );
        for (i, (name, _)) in joint_arms.iter().enumerate() {
            let base_i = if i < 5 { 0 } else { 5 };
            let (cmin, cmed, cmax) = stats(&j_cpu[i]);
            let (_, bmed, _) = stats(&j_cpu[base_i]);
            let ratios: Vec<f64> = (0..repeats)
                .map(|r| j_cpu[i][r] / j_cpu[base_i][r])
                .collect();
            let (rmin, rmed, rmax) = stats(&ratios);
            println!(
                "{:<24} {:>8.2}ms [{:>6.2}..{:>6.2}] {:>6.4}x ({:>+6.2}%) {:>+7.2}ms [{:.4}..{:.4}]",
                name,
                cmed,
                cmin,
                cmax,
                rmed,
                (rmed - 1.0) * 100.0,
                cmed - bmed,
                rmin,
                rmax,
            );
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn main() {
    x86::run();
}
