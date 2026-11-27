//! Composed End-to-End Worker Stack v2 Probe (`A5 + A6a + A6b + A6c + A7 + A9`).
//!
//! Smooth-zone diagnostic (`Claim: none`, zero `src/` edits) evaluating the two concrete
//! upgrades identified by `#1059` (`outq_hoist_and_rle_probe.rs`) and `#1062`
//! (`composed_worker_stack_probe.rs`) on top of `#1062`'s `W4_full_fused_vec` across all
//! 233 production-admitted Open blocks of `wikidata.bz2` (warm and L3-cold, on baseline
//! x86-64, `-C target-cpu=native`, and JCC-aligned `-C target-cpu=native`):
//!
//! 1. Upgrade (a) (`#1059` hoisted single-branch `OutQ16` + fused Pass-1 `lits` popcount):
//!    - `OVF_BIT` in `fill[a]` bit 31 collapses `OutQ16::push_hoisted` to a single add + compare
//!      (`f.wrapping_add(w) <= CHUNK as u32`).
//!    - Batch-hoisted `p` check (`cur > p || cur + batch_rows <= p`) and batch-hoisted `EPOCH`
//!      check (`(m & (EPOCH - 1)) + max_m_add < EPOCH`), register-local `m`, AVX2 batch flush
//!      (`#[target_feature(enable = "avx2")] #[inline(never)]`), and fused Pass-1 `lits` popcount.
//!    - Tested at BOTH `496/512` and `1008/1024`.
//! 2. Upgrade (b) (`#1062` cold-outlined 64 KiB slab compaction + CRC32):
//!    - Mark `compact_and_crc_slab_cold` `#[cold] #[inline(never)]` so RLE1+CRC compaction is
//!      kept completely out of `emit_4lane_sampled<true>`'s 4-lane hot loop (and compare 32 KiB
//!      vs 64 KiB slab size).
//!
//! Usage:
//!   composed_worker_v2_probe [FILE.bz2] [--repeats N]

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    eprintln!("composed_worker_v2_probe requires x86_64 (AVX2 MTF front)");
}

#[cfg(target_arch = "x86_64")]
fn main() {
    x86::run();
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use mibz2::block::Symbols;
    use mibz2::stream::{Event, Scanner, SliceSource};
    use mibz2::{crc, decode_block, rle, ErrorKind, Kernels, Scratch};
    use std::arch::x86_64::*;
    use std::hint::black_box;
    use std::time::Instant;

    // -----------------------------------------------------------------------
    // Shared constants
    // -----------------------------------------------------------------------

    const LIMIT: u32 = 16;
    const CAP: u32 = 127;
    const MAX_PACKED_RUN: u64 = 0x00FF_FFFF;
    const INLINE: usize = 16;
    const LONG: u8 = u8::MAX;
    const RULE: usize = 4;
    const NARROW_NODES: usize = 1 << 16;
    const ID_BITS: u32 = 22;
    const ID_MASK: u32 = (1 << ID_BITS) - 1;
    const SATURATED: u32 = u32::MAX >> ID_BITS;
    const CERTIFIED_WIDTH: u32 = 32_768;
    const SPIRAL_RULES: usize = 2 * 20 + 1;

    const LAZY_DENSITY: (usize, usize) = (3, 20);
    const DENSE: (usize, usize) = (7, 25);
    const DENSE_M: u32 = 256_000;
    const DENSE_STRIPS: usize = 16_384;

    const NONE_CHUNK: u32 = u32::MAX;
    const EPOCH: u32 = 1 << 16;
    /// High bit of `Queues::fill[a]` marks a tail chunk in `spill.ovf` so a single
    /// unsigned comparison `f.wrapping_add(w) <= CHUNK as u32` checks uninitialized,
    /// full/straddling, AND overflow in one branch.
    const OVF_BIT: u32 = 1u32 << 31;

    const U128_LONG_TAG: u128 = 0xFFu128 << 120;
    const U128_PAYLOAD_MASK: u128 = (1u128 << 120) - 1;
    const GEN_PAIR_SLOTS: usize = 1024;
    const GEN_CONTENT_SLOTS: usize = 4096;

    const ANCHOR_TAG: u32 = 1u32 << 31;
    const S_ANCHORS: usize = 16;
    const MAX_ANCHORS: usize = 32;
    const SLAB_32K: usize = 32 * 1024;
    const SLAB_64K: usize = 64 * 1024;
    const RLE_CRC_TILE: usize = 64 << 10;

    // L3-cold rotating worker pool: 16 independent worker states (>50 MiB > 45 MiB L3)
    const COLD_POOL_WORKERS: usize = 16;
    const COLD_FLUSH_BYTES: usize = 64 * 1024 * 1024;

    // i9-10900K reference anchors (wikidata x4; character-stage1 + #1056)
    const I9_WALL_1W_MS: f64 = 1306.0;
    const I9_A5_A6A_MS: f64 = 132.4 + 200.3; // 332.7 ms
    const I9_A6B_MS: f64 = 216.0;
    const I9_A6C_MS: f64 = 161.4;
    const I9_A7_A9_MS: f64 = 490.3 + 70.5; // 560.8 ms
    const I9_WORKER_1W_MS: f64 = I9_A5_A6A_MS + I9_A6B_MS + I9_A6C_MS + I9_A7_A9_MS; // 1,270.9 ms
    const I9_WORKER_8W_MS: f64 = 185.68; // character-stage1 k=8 W/8 (#1056)
    const I9_6GBS_W8_THRESHOLD_MS: f64 = 133.9; // #1056 W'/8 <= 133.9 ms at S'=145 ms
    const I9_6GBS_W8_SMALL_CH0_MS: f64 = 135.4; // #1056 W'/8 <= 135.4 ms with small chunk 0

    // -----------------------------------------------------------------------
    // Pure-safe steal-free Linux CPU timer (`/proc/thread-self/schedstat`)
    // -----------------------------------------------------------------------

    struct CpuClock {
        file: Option<std::fs::File>,
        origin: Instant,
    }

    impl CpuClock {
        fn new() -> Self {
            let file = std::fs::File::open("/proc/thread-self/schedstat")
                .or_else(|_| std::fs::File::open("/proc/self/schedstat"))
                .ok();
            CpuClock {
                file,
                origin: Instant::now(),
            }
        }

        #[inline(always)]
        fn now_ns(&mut self) -> u64 {
            if let Some(f) = self.file.as_mut() {
                use std::io::{Read, Seek, SeekFrom};
                if f.seek(SeekFrom::Start(0)).is_ok() {
                    let mut buf = [0u8; 64];
                    if let Ok(n) = f.read(&mut buf) {
                        let mut val = 0u64;
                        for &b in &buf[..n] {
                            if b.is_ascii_digit() {
                                val = val * 10 + u64::from(b - b'0');
                            } else {
                                break;
                            }
                        }
                        return val;
                    }
                }
            }
            self.origin.elapsed().as_nanos() as u64
        }
    }

    // -----------------------------------------------------------------------
    // Mechanism 1 (#1055): Parameterized OutQ16<CHUNK, STRIDE> + L1-batch MTF fold
    // -----------------------------------------------------------------------

    #[derive(Clone, Copy, Debug, PartialEq)]
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
        records * LAZY_DENSITY.1 < n * LAZY_DENSITY.0 && records * 64 >= n
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

    #[derive(Default)]
    struct Spill<const STRIDE: usize> {
        next: Vec<u32>,
        ovf: Vec<Box<[u8]>>,
        cuts: Vec<[u32; 256]>,
    }

    impl<const STRIDE: usize> Spill<STRIDE> {
        fn scratch_bytes(&self) -> usize {
            self.next.capacity() * 4
                + self.ovf.iter().map(|b| b.len()).sum::<usize>()
                + self.cuts.capacity() * std::mem::size_of::<[u32; 256]>()
        }
    }

    #[derive(Clone, Copy)]
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
                self.push_slow(a, m, w, thin);
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
            if id >= self.in_out && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill
                    .ovf
                    .push(vec![0u8; Self::BYTES].into_boxed_slice());
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

        /// Single-branch fast path for hoisted batches (caller guarantees `w <= 16`,
        /// `lo + w < EPOCH`, and `fill[a]` has `OVF_BIT` set if in `ovf` or `CHUNK` if full/uninit).
        #[inline(always)]
        fn push_hoisted(&mut self, a: usize, m: u32, w: u32, thin: &[u32; 256]) {
            debug_assert!(
                Self::HAS_GUARD && (1..=16).contains(&w) && (m & (EPOCH - 1)) + w < EPOCH
            );
            let f = self.fill[a];
            let nf = f.wrapping_add(w);
            if nf <= CHUNK as u32 {
                let lo = m as u16;
                let tail = self.tails[a] as usize;
                let at = tail * Self::BYTES + 2 * (f as usize);
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
                if f == CHUNK {
                    is_ovf = self.take_chunk_tagged(a);
                    f = 0;
                }
                let lo = m & (EPOCH - 1);
                let take = left.min((CHUNK - f) as u32).min(EPOCH - lo);
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

        fn take_chunk_tagged(&mut self, a: usize) -> bool {
            let id = self.chunks;
            self.chunks += 1;
            let is_ovf = id >= self.in_out;
            if is_ovf && self.spill.ovf.len() <= (id - self.in_out) as usize {
                self.spill
                    .ovf
                    .push(vec![0u8; Self::BYTES].into_boxed_slice());
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

        fn chunk_mut(&mut self, id: u32) -> &mut [u8] {
            if id < self.in_out {
                let at = id as usize * Self::BYTES;
                &mut self.out[at..at + Self::BYTES]
            } else {
                &mut self.spill.ovf[(id - self.in_out) as usize][..]
            }
        }
    }

    struct QueueReader<'a, const CHUNK: usize, const STRIDE: usize> {
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

    impl<'a, const CHUNK: usize, const STRIDE: usize> QueueReader<'a, CHUNK, STRIDE> {
        const BYTES: usize = 2 * STRIDE;

        fn new(out: &'a [u8], spill: &'a Spill<STRIDE>, heads: &Heads, a: usize) -> Self {
            QueueReader {
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
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = self.epoch << 16;
                let at = self.id as usize * Self::BYTES + 2 * self.at;
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
        fn wire(&mut self, dst: &mut [u32], pack: u32) {
            let span = dst.len();
            if self.at + span <= CHUNK
                && self.read + span as u32 <= self.cut
                && self.id < self.in_out
            {
                let hi = pack | (self.epoch << 16);
                let at = self.id as usize * Self::BYTES + 2 * self.at;
                for (e, s) in dst
                    .iter_mut()
                    .zip(self.out[at..at + 2 * span].chunks_exact(2))
                {
                    *e = hi | u32::from(u16::from_le_bytes([s[0], s[1]]));
                }
                self.at += span;
                self.read += span as u32;
            } else {
                self.wire_slow(dst, pack);
            }
        }

        #[cold]
        #[inline(never)]
        fn wire_slow(&mut self, mut dst: &mut [u32], pack: u32) {
            while !dst.is_empty() {
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
                let take = dst
                    .len()
                    .min(CHUNK - self.at)
                    .min((self.cut - self.read) as usize);
                let chunk: &[u8] = if self.id < self.in_out {
                    let at = self.id as usize * Self::BYTES;
                    &self.out[at..at + Self::BYTES]
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

    #[derive(Clone, Copy)]
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
            limit_m: u32,
            cap_strips: usize,
        ) -> Self {
            Pass {
                queues: Queues::new(out, spill, limit_m),
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

        #[cold]
        #[inline(never)]
        fn grow_edges(&mut self, at: usize) {
            let need = (at + 3)
                .max(2 * self.edges.len())
                .max(1024)
                .min(self.cap_strips.saturating_mul(3));
            self.edges.reserve_exact(need - self.edges.len());
            self.edges.resize(need, 0);
        }

        #[inline(always)]
        fn push_thick(&mut self, a: usize, c: u8, x: u32, w: u32) {
            if self.ns < self.cap_strips {
                let at = 3 * self.ns;
                if self.edges.len() < at + 3 {
                    self.grow_edges(at);
                }
                let f = &mut self.f;
                self.edges[at..at + 3].copy_from_slice(&[
                    x,
                    f.thin[a] + f.thick_rows[a],
                    (w << 8) | u32::from(c),
                ]);
            }
            self.ns += 1;
            self.f.thick[a] += 1;
            self.f.thick_rows[a] += w;
        }

        #[cold]
        #[inline(never)]
        fn flush_batch_fallback(&mut self, batch: &[u32], pos: &mut u32, p: u32) {
            let mut cur = *pos;
            for &word in batch {
                let c = (word >> 24) as u8;
                let w = word & 0x00FF_FFFF;
                self.take_tagged(c, cur, w, p);
                cur += w;
            }
            *pos = cur;
        }

        /// Batch flush compiled with AVX2 (`#1059`): eliminates `vzeroupper` at the batch boundary
        /// AND enables single 32B `vmovdqu` iota store on baseline x86-64 builds, while hoisting
        /// `p` (`cur > p || cur + batch_rows <= p`) and `EPOCH` (`lo + max_m_add < EPOCH`) to once
        /// per batch with `OVF_BIT` single-branch `push_hoisted` and register-local `m`.
        #[target_feature(enable = "avx2")]
        #[inline(never)]
        unsafe fn flush_batch_avx2(
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
            if lo + max_m_add >= EPOCH || !no_mark {
                self.flush_batch_fallback(batch, pos, p);
                return;
            }
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

    #[inline(always)]
    fn maximal(records: &[u16], mut visit: impl FnMut(u8, u32, u32)) {
        let mut pos = 0;
        let mut i = 0;
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

    #[derive(Default)]
    struct FrontScratch {
        lit: Vec<u64>,
        one: Vec<u64>,
        runs: Vec<u16>,
    }

    impl FrontScratch {
        fn scratch_bytes(&self) -> usize {
            self.lit.capacity() * 8 + self.one.capacity() * 8 + self.runs.capacity() * 2
        }
    }

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
        Some(n as usize)
    }

    #[repr(align(64))]
    struct MtfRows([u8; 224]);

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
        done: bool,
        huge: Option<(u8, u64)>,
    }

    #[inline(always)]
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

    #[inline(never)]
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    unsafe fn probe_build_bitmaps_avx2(b: &Symbols, f: &mut FrontScratch) -> Option<(usize, u16)> {
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
        let hi = _mm256_extracti128_si256::<1>(vmax);
        let lo = _mm256_castsi256_si128(vmax);
        let m8 = _mm_max_epu16(lo, hi);
        let m4 = _mm_max_epu16(m8, _mm_srli_si128::<8>(m8));
        let m2 = _mm_max_epu16(m4, _mm_srli_si128::<4>(m4));
        let m1 = _mm_max_epu16(m2, _mm_srli_si128::<2>(m2));
        max_sym = max_sym.max(_mm_cvtsi128_si32(m1) as u16);
        Some((end, max_sym))
    }

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
                    pass.queues.push_slow(a, m, width, &pass.f.thin);
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

    fn run_l1_batch<const BATCH: usize, const CHUNK: usize, const STRIDE: usize>(
        b: &Symbols,
        scratch: &mut FrontScratch,
        out: &mut [u8],
        spill: &mut Spill<STRIDE>,
        edges: &mut Vec<u32>,
    ) -> (usize, First, Heads, usize) {
        let (end, max_sym) = unsafe { probe_build_bitmaps_avx2(b, scratch).expect("bitmaps") };
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
        let open_len = unsafe { _bzhi_u64(*onep, q as u32 + 1) } - 1;
        let mut slack = b.max_len / CAP as usize + 1;
        let open_recs = open_len.div_ceil(CAP as u64);
        let open_more = open_recs.saturating_sub(1) as usize;
        slack -= open_more;

        let p = b.orig_ptr;
        let mut pass = Pass::<CHUNK, STRIDE>::new(out, spill, edges, u32::MAX, usize::MAX);
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
                done: false,
                huge: None,
            };
            let mut batch = [0u32; BATCH];
            let sp = b.syms.as_ptr();
            let max_len = b.max_len as u64;
            while !st.done {
                let blen = unsafe {
                    probe_fill_batch_avx2::<BATCH>(&mut st, sp, litp, onep, max_len, &mut batch)
                        .expect("fill_batch")
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
        }

        if !injective && pending_w != 0 {
            pass.take(pending_c, pos, pending_w, p, LIMIT);
            pos += pending_w;
        }
        let n = pos as usize;
        let heads = pass.queues.heads();
        (n, pass.f, heads, pass.ns)
    }

    /// `#1059` zero-spill AVX2 MTF front: fuses Pass-1 `lits` popcount + early `NotOpen` check,
    /// packs maximal `(c << 24) | total` runs into stack `[u32; B]` with exact `batch_rows`,
    /// keeps `second_cold` out-of-line, and flushes every `B` runs via `flush_fn(&buf[..len], batch_rows)`.
    #[target_feature(enable = "avx2,bmi1,bmi2,lzcnt,popcnt")]
    #[inline(never)]
    unsafe fn front_avx2_batched_hoisted<const B: usize>(
        b: &Symbols,
        f: &mut FrontScratch,
        lits_refused: usize,
        mut flush_fn: impl FnMut(&[u32], u32),
    ) -> Result<(usize, usize), bool> {
        let m = b.syms.len();
        let sp = b.syms.as_ptr();
        if b.max_len >= 1 << 31 || m >= 1 << 24 {
            return Err(false);
        }

        // ---- Pass 1: bitmaps + fused popcount for `lits` & early `refused` ----
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
        f.lit.set_len(words);
        f.one.set_len(words);
        if end == usize::MAX {
            return Err(false);
        }
        if lits >= lits_refused {
            return Err(true);
        }

        // ---- Pass 2: register-clean MTF -> L1d `[u32; B]` batch ----
        let s2u = &b.seq_to_unseq;
        let mut f0 = _mm_loadu_si128(s2u.as_ptr() as *const __m128i);
        let mut f1 = _mm_loadu_si128(s2u.as_ptr().add(16) as *const __m128i);
        let mut rows = MtfRows([0u8; 224]);
        rows.0.copy_from_slice(&s2u[32..]);
        let tail = rows.0.as_mut_ptr();
        let steps = STEPS.as_ptr();
        let mut slack = b.max_len / CAP as usize + 1;
        let mut extra_recs = 0usize;
        let mut n = 0u64;
        let mut big = 0u64;
        let litp = f.lit.as_ptr();
        let onep = f.one.as_ptr();

        let mut buf = [0u32; B];
        let mut bi = 0usize;
        let mut batch_rows = 0u32;

        macro_rules! push_word {
            ($c:expr, $total:expr) => {{
                let tot = $total as u32;
                *buf.as_mut_ptr().add(bi) = (($c as u32) << 24) | tot;
                bi += 1;
                batch_rows += tot;
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
                    let v = second_cold(tail, (s - 1).min(255) - 32, f1);
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

    fn run_l1_batch_hoisted<const BATCH: usize, const CHUNK: usize, const STRIDE: usize>(
        b: &Symbols,
        scratch: &mut FrontScratch,
        out: &mut [u8],
        spill: &mut Spill<STRIDE>,
        edges: &mut Vec<u32>,
    ) -> (usize, First, Heads, usize) {
        let used = usize::from(b.eob).saturating_sub(1).clamp(1, 256);
        if !s2u_injective(&b.seq_to_unseq, used) {
            return run_l1_batch::<512, CHUNK, STRIDE>(b, scratch, out, spill, edges);
        }
        let mut pass = Pass::<CHUNK, STRIDE>::new(out, spill, edges, u32::MAX, usize::MAX);
        let mut pos = 0u32;
        let p = b.orig_ptr;
        let lits_ref = (7 * b.max_len).div_ceil(25);
        let (n, _recs) = unsafe {
            front_avx2_batched_hoisted::<BATCH>(b, scratch, lits_ref, |batch, batch_rows| {
                pass.flush_batch_avx2(batch, batch_rows, &mut pos, p);
            })
        }
        .expect("hoisted batched front");
        debug_assert_eq!(n, pos as usize);
        let heads = pass.queues.heads();
        (n, pass.f, heads, pass.ns)
    }

    // -----------------------------------------------------------------------
    // Open A6b: Shipped Elim<Narrow/Wide> AND Mechanism 2 (#1046) Fixed64kSoa + GenCache
    // -----------------------------------------------------------------------

    #[inline(always)]
    fn literal(byte: u8) -> u32 {
        (1 << ID_BITS) | (u32::from(byte) + 1)
    }

    #[inline(always)]
    fn literal_u128(byte: u8) -> u128 {
        (1u128 << 120) | u128::from(byte)
    }

    #[derive(Clone, Copy)]
    struct Strip {
        from: u32,
        to: u32,
        width: u32,
        byte: u8,
    }

    impl Strip {
        #[inline(always)]
        fn unpack(w: &[u32]) -> Self {
            Strip {
                from: w[0],
                to: w[1],
                width: w[2] >> 8,
                byte: (w[2] & 0xFF) as u8,
            }
        }
    }

    #[derive(Clone, Copy)]
    struct Band(u64);

    impl Band {
        #[inline(always)]
        fn new(end: u32, dst: u32, code: u32) -> Self {
            Band(u64::from(end) | (u64::from(dst) << 20) | (u64::from(code) << 40))
        }
        #[inline(always)]
        fn end(self) -> u32 {
            (self.0 & 0xF_FFFF) as u32
        }
        #[inline(always)]
        fn dst(self) -> u32 {
            ((self.0 >> 20) & 0xF_FFFF) as u32
        }
        #[inline(always)]
        fn code(self) -> u32 {
            (self.0 >> 40) as u32
        }
    }

    #[inline(always)]
    fn long_slot(start: u32, end: u32) -> [u8; INLINE] {
        let mut slot = [0; INLINE];
        slot[..4].copy_from_slice(&start.to_le_bytes());
        slot[4..8].copy_from_slice(&end.to_le_bytes());
        slot
    }

    #[inline(always)]
    fn long_word<'a>(slot: &[u8; INLINE], spelling: &'a [u8]) -> &'a [u8] {
        let start = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
        let end = u32::from_le_bytes([slot[4], slot[5], slot[6], slot[7]]) as usize;
        &spelling[start..end]
    }

    fn begin_inline(words: usize, inline16: &mut Vec<[u8; INLINE]>, lens: &mut Vec<u8>) {
        if inline16.len() < 256 {
            inline16.clear();
            inline16.extend((0..=255u8).map(|c| {
                let mut v = [0; INLINE];
                v[0] = c;
                v
            }));
            lens.clear();
            lens.resize(256, 1);
        }
        inline16.truncate(256);
        lens.truncate(256);
        inline16.reserve_exact(words);
        lens.reserve_exact(words);
    }

    fn starts(counts: [u32; 256]) -> [u32; 256] {
        let mut total = 0;
        counts.map(|w| {
            let start = total;
            total += w;
            start
        })
    }

    trait Links {
        const STRIDE: usize;
        const BOTTOM: usize;
        fn prev(a: &[u32], id: u32, side: usize) -> u32;
        fn next(a: &[u32], id: u32, side: usize) -> u32;
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32);
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32);
    }

    struct Narrow;
    struct Wide;

    impl Links for Narrow {
        const STRIDE: usize = 4;
        const BOTTOM: usize = 1;
        #[inline(always)]
        fn prev(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 4 + side] & 0xFFFF
        }
        #[inline(always)]
        fn next(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 4 + side] >> 16
        }
        #[inline(always)]
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
            let w = &mut a[id as usize * 4 + side];
            *w = (*w & 0xFFFF_0000) | to;
        }
        #[inline(always)]
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
            let w = &mut a[id as usize * 4 + side];
            *w = (*w & 0xFFFF) | (to << 16);
        }
    }

    impl Links for Wide {
        const STRIDE: usize = 6;
        const BOTTOM: usize = 2;
        #[inline(always)]
        fn prev(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 6 + 2 * side]
        }
        #[inline(always)]
        fn next(a: &[u32], id: u32, side: usize) -> u32 {
            a[id as usize * 6 + 2 * side + 1]
        }
        #[inline(always)]
        fn set_prev(a: &mut [u32], id: u32, side: usize, to: u32) {
            a[id as usize * 6 + 2 * side] = to;
        }
        #[inline(always)]
        fn set_next(a: &mut [u32], id: u32, side: usize, to: u32) {
            a[id as usize * 6 + 2 * side + 1] = to;
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(a: &mut Vec<u32>, need: usize) {
        a.resize(need.max(2 * a.len()), 0);
    }

    struct Elim<'a, L: Links> {
        a: &'a mut [u32],
        base: usize,
        top: usize,
        nodes: u32,
        _links: std::marker::PhantomData<L>,
    }

    impl<L: Links> Elim<'_, L> {
        #[inline(always)]
        fn width(&self, id: u32) -> u32 {
            self.a[id as usize * L::STRIDE + L::STRIDE - 2]
        }
        #[inline(always)]
        fn set_width(&mut self, id: u32, w: u32) {
            self.a[id as usize * L::STRIDE + L::STRIDE - 2] = w;
        }
        #[inline(always)]
        fn word(&self, id: u32) -> u32 {
            self.a[id as usize * L::STRIDE + L::STRIDE - 1]
        }
        #[inline(always)]
        fn set_word(&mut self, id: u32, w: u32) {
            self.a[id as usize * L::STRIDE + L::STRIDE - 1] = w;
        }
        #[inline(always)]
        fn bottom(&self, id: u32) -> u32 {
            self.a[id as usize * L::STRIDE + L::BOTTOM]
        }
        #[inline(always)]
        fn set_bottom(&mut self, id: u32, x: u32) {
            self.a[id as usize * L::STRIDE + L::BOTTOM] = x;
        }
        fn assign_bottom(&mut self, rows: u32) {
            let mut first = L::next(self.a, 0, 1);
            let mut last = L::prev(self.a, 0, 1);
            let (mut lo, mut hi) = (0, rows);
            while first != last {
                let next = L::next(self.a, first, 1);
                let prev = L::prev(self.a, last, 1);
                let width = self.width(first);
                hi -= self.width(last);
                self.set_bottom(first, lo);
                self.set_bottom(last, hi);
                lo += width;
                if next == last {
                    assert_eq!(lo, hi);
                    return;
                }
                first = next;
                last = prev;
            }
            if first != 0 {
                self.set_bottom(first, lo);
                lo += self.width(first);
            }
            assert_eq!(lo, hi);
        }
        fn node(&mut self, width: u32, word: u32) -> u32 {
            let id = self.nodes;
            self.nodes += 1;
            self.set_width(id, width);
            self.set_word(id, word);
            id
        }
        #[inline(always)]
        fn append_to(&mut self, list: &mut (u32, u32), id: u32, side: usize) {
            let (head, tail) = list;
            if *tail == 0 {
                *head = id;
            } else {
                L::set_next(self.a, *tail, side, id);
            }
            L::set_prev(self.a, id, side, *tail);
            *tail = id;
        }
        fn close(&mut self, side: usize, first: (u32, u32), second: (u32, u32)) {
            let mut prev = 0;
            for (head, tail) in [first, second] {
                if tail == 0 {
                    continue;
                }
                L::set_next(self.a, prev, side, head);
                L::set_prev(self.a, head, side, prev);
                prev = tail;
            }
            L::set_next(self.a, prev, side, 0);
            L::set_prev(self.a, 0, side, prev);
        }
        #[inline(always)]
        fn cut(&mut self, id: u32, side: usize) {
            let (p, n) = (L::prev(self.a, id, side), L::next(self.a, id, side));
            L::set_next(self.a, p, side, n);
            L::set_prev(self.a, n, side, p);
        }
        #[inline(always)]
        fn pop_tail(&mut self, id: u32, side: usize) {
            let prev = L::prev(self.a, id, side);
            L::set_next(self.a, prev, side, 0);
            L::set_prev(self.a, 0, side, prev);
        }
        #[inline(always)]
        fn insert(&mut self, at: u32, id: u32, side: usize) {
            let next = L::next(self.a, at, side);
            L::set_prev(self.a, id, side, at);
            L::set_next(self.a, id, side, next);
            L::set_next(self.a, at, side, id);
            L::set_prev(self.a, next, side, id);
        }
        #[inline(always)]
        fn rule(&self, w: u32) -> usize {
            self.base + (w - 257) as usize * RULE
        }
        #[inline(always)]
        fn len(&self, w: u32) -> u32 {
            if w <= 256 {
                u32::from(w != 0)
            } else {
                let r = self.rule(w);
                let tag = self.a[r + 3];
                if tag >> 28 == 0 {
                    tag >> 24
                } else {
                    self.a[r + 2]
                }
            }
        }
        #[inline(always)]
        fn short_payload(&self, id: u32) -> u128 {
            if id <= 256 {
                return u128::from(id - 1);
            }
            let r = self.rule(id);
            u128::from(self.a[r])
                | (u128::from(self.a[r + 1]) << 32)
                | (u128::from(self.a[r + 2]) << 64)
                | (u128::from(self.a[r + 3] & 0x00ff_ffff) << 96)
        }
        #[inline(always)]
        fn join(&mut self, left: u32, right: u32) -> u32 {
            if left == 0 {
                return right;
            }
            if right == 0 {
                return left;
            }
            let id = ((self.top - self.base) / RULE) as u32 + 257;
            let (left_len, right_len) = (left >> ID_BITS, right >> ID_BITS);
            let (left, right) = (left & ID_MASK, right & ID_MASK);
            let len = if left_len < SATURATED && right_len < SATURATED {
                left_len + right_len
            } else {
                self.len(left) + self.len(right)
            };
            let rule = if len < INLINE as u32 {
                let bytes = self.short_payload(left)
                    | (self.short_payload(right) << (8 * left_len))
                    | (u128::from(len) << 120);
                [
                    bytes as u32,
                    (bytes >> 32) as u32,
                    (bytes >> 64) as u32,
                    (bytes >> 96) as u32,
                ]
            } else {
                [left, right, len, u32::MAX]
            };
            self.a[self.top..self.top + RULE].copy_from_slice(&rule);
            self.top += RULE;
            (len.min(SATURATED) << ID_BITS) | id
        }
        fn power(&mut self, word: u32, q: u32) -> u32 {
            let (mut acc, mut base, mut e) = (0, word, q);
            loop {
                if e & 1 == 1 {
                    acc = self.join(acc, base);
                }
                e >>= 1;
                if e == 0 {
                    break acc;
                }
                base = self.join(base, base);
            }
        }
        fn eliminate<const SPIRALS: bool>(&mut self, remaining: &mut u32) -> bool {
            let mut left = *remaining;
            let room = if SPIRALS { SPIRAL_RULES * RULE } else { RULE };
            let done = loop {
                if left == 0 {
                    break true;
                }
                if self.top + room > self.a.len() {
                    break false;
                }
                let (x, y) = (L::prev(self.a, 0, 0), L::prev(self.a, 0, 1));
                let (aw, bw) = (self.width(x), self.width(y));
                if SPIRALS && x != y {
                    if aw > bw && L::prev(self.a, y, 1) == x && (aw - 1) / bw >= 2 {
                        let q = (aw - 1) / bw;
                        let head = self.power(self.word(x), q);
                        let word = self.join(head, self.word(y));
                        self.set_word(y, word);
                        self.set_width(x, aw - q * bw);
                        left -= q * bw;
                        continue;
                    }
                    if aw < bw && L::prev(self.a, x, 0) == y && (bw - 1) / aw >= 2 {
                        let q = (bw - 1) / aw;
                        let tail = self.power(self.word(y), q);
                        let word = self.join(self.word(x), tail);
                        self.set_word(x, word);
                        self.set_width(y, bw - q * aw);
                        left -= q * aw;
                        continue;
                    }
                }
                let w = aw.min(bw);
                left -= w;
                if x == y {
                    self.pop_tail(x, 0);
                    self.pop_tail(x, 1);
                    continue;
                }
                let word = self.join(self.word(x), self.word(y));
                if aw <= bw {
                    self.pop_tail(x, 0);
                    self.insert(y, x, 0);
                    self.set_word(x, word);
                    if aw == bw {
                        self.cut(y, 0);
                        self.pop_tail(y, 1);
                    } else {
                        self.set_width(y, bw - w);
                    }
                } else {
                    self.pop_tail(y, 1);
                    self.insert(x, y, 1);
                    self.set_word(y, word);
                    self.set_width(x, aw - w);
                }
            };
            *remaining = left;
            done
        }
        fn inline_word(
            &mut self,
            word: u32,
            byte: u8,
            stack: &mut Vec<u32>,
            spelling: &mut Vec<u8>,
        ) -> ([u8; INLINE], u8) {
            let id = word & ID_MASK;
            let height = (word >> ID_BITS) as usize;
            if height < INLINE {
                let mut bytes = self.short_payload(id).to_le_bytes();
                bytes[height] = byte;
                (bytes, (height + 1) as u8)
            } else {
                let start = spelling.len() as u32;
                self.append(id, stack, spelling);
                spelling.push(byte);
                (long_slot(start, spelling.len() as u32), LONG)
            }
        }
        fn append(&mut self, word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
            stack.clear();
            stack.push(word);
            while let Some(id) = stack.pop() {
                if id <= 256 {
                    dest.push((id - 1) as u8);
                } else {
                    let r = self.rule(id);
                    let tag = self.a[r + 3];
                    if tag >> 28 == 0 {
                        let bytes = self.short_payload(id).to_le_bytes();
                        dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                        continue;
                    }
                    let rule = &mut self.a[r..r + RULE];
                    if tag == u32::MAX {
                        rule[3] = 0x8000_0000 | dest.len() as u32;
                        stack.push(rule[1]);
                        stack.push(rule[0]);
                    } else {
                        let start = (tag & 0x7fff_ffff) as usize;
                        dest.extend_from_within(start..start + rule[2] as usize);
                    }
                }
            }
        }
    }

    fn spelling_hash(entry: &[u8; INLINE], len: u8, bytes: &[u8]) -> u64 {
        let (low, high, middle) = if len == LONG {
            let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
            (part(0), part(bytes.len() - 8), part(bytes.len() / 2 - 4))
        } else {
            let value = u128::from_le_bytes(*entry);
            (value as u64, (value >> 64) as u64, 0)
        };
        let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    #[inline(always)]
    fn short_u128_hash(entry_u128: u128, len: u8) -> u64 {
        let low = entry_u128 as u64;
        let high = (entry_u128 >> 64) as u64;
        let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ u64::from(len).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    #[inline(always)]
    fn long_bytes_hash(bytes: &[u8]) -> u64 {
        let part = |i: usize| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
        let low = part(0);
        let high = part(bytes.len() - 8);
        let middle = part(bytes.len() / 2 - 4);
        let h = low.wrapping_mul(0xBF58_476D_1CE4_E5B9)
            ^ high.rotate_left(29).wrapping_mul(0x94D0_49BB_1331_11EB)
            ^ middle.wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ (bytes.len() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h ^ (h >> 30)
    }

    struct Dictionary<'a> {
        pairs: &'a mut [u32],
        contents: &'a mut [u32],
        inline16: &'a mut Vec<[u8; INLINE]>,
        lens: &'a mut Vec<u8>,
        spelling: &'a mut Vec<u8>,
        stack: &'a mut Vec<u32>,
    }

    impl Dictionary<'_> {
        fn code<L: Links>(&mut self, e: &mut Elim<L>, handle: u32, byte: u8) -> u32 {
            if handle == 0 {
                return u32::from(byte);
            }
            let id = handle & ID_MASK;
            let key = (id << 8) | u32::from(byte);
            let slot = 2 * (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
            if !self.pairs.is_empty() && self.pairs[slot] == key {
                return self.pairs[slot + 1];
            }
            let start = self.spelling.len();
            let memoized = handle >> ID_BITS >= INLINE as u32 && e.a[e.rule(id) + 3] != u32::MAX;
            let (entry, len) = e.inline_word(handle, byte, self.stack, self.spelling);
            if self.contents.is_empty() {
                return self.insert(entry, len);
            }
            let bytes = if len == LONG {
                &self.spelling[start..]
            } else {
                &entry[..len as usize]
            };
            let index = spelling_hash(&entry, len, bytes) as usize & (self.contents.len() - 1);
            let old = self.contents[index] as usize;
            let same = old != 0
                && self.lens[old] == len
                && if len == LONG {
                    long_word(&self.inline16[old], self.spelling) == bytes
                } else {
                    self.inline16[old] == entry
                };
            let code = if same {
                if len == LONG {
                    if memoized {
                        self.spelling.truncate(start);
                    } else {
                        self.spelling.pop();
                    }
                }
                old as u32
            } else {
                let code = self.insert(entry, len);
                self.contents[index] = code;
                code
            };
            self.pairs[slot] = key;
            self.pairs[slot + 1] = code;
            code
        }

        fn insert(&mut self, entry: [u8; INLINE], len: u8) -> u32 {
            let code = self.lens.len() as u32;
            self.inline16.push(entry);
            self.lens.push(len);
            code
        }
    }

    // ---- Mechanism 2 (#1046): Safe 64K u16-link inline15 SoA + GenCache ----

    #[derive(Default, Clone, Copy)]
    struct NodeLinks {
        prev0: u16,
        next0: u16,
        prev1: u16,
        next1: u16,
    }

    fn boxed_array_64k<T: Copy>(val: T) -> Box<[T; NARROW_NODES]> {
        vec![val; NARROW_NODES]
            .into_boxed_slice()
            .try_into()
            .ok()
            .unwrap()
    }

    struct GenCache {
        pairs: Vec<u64>,
        contents: Vec<u32>,
        pair_gen: u64,
        content_gen: u32,
    }

    impl Default for GenCache {
        fn default() -> Self {
            GenCache {
                pairs: vec![0; GEN_PAIR_SLOTS],
                contents: vec![0; GEN_CONTENT_SLOTS],
                pair_gen: 0,
                content_gen: 0,
            }
        }
    }

    impl GenCache {
        #[inline(always)]
        fn next_block(&mut self) {
            self.pair_gen += 1;
            if self.pair_gen >= (1u64 << 18) {
                self.pairs.fill(0);
                self.pair_gen = 1;
            }
            self.content_gen += 1;
            if self.content_gen >= (1u32 << 16) {
                self.contents.fill(0);
                self.content_gen = 1;
            }
        }
    }

    struct Fixed64kSoa {
        links: Box<[NodeLinks; NARROW_NODES]>,
        widths: Box<[u32; NARROW_NODES]>,
        words128: Box<[u128; NARROW_NODES]>,
    }

    impl Fixed64kSoa {
        fn new() -> Self {
            Fixed64kSoa {
                links: boxed_array_64k(NodeLinks::default()),
                widths: boxed_array_64k(0),
                words128: boxed_array_64k(0),
            }
        }
    }

    #[derive(Default)]
    struct A6bState {
        image_order64: Vec<u64>,
        fixed_soa: Option<Fixed64kSoa>,
        rules: Vec<[u32; 4]>,
        gen_cache: GenCache,
    }

    impl A6bState {
        fn scratch_bytes_virtual(&self) -> usize {
            let soa = if self.fixed_soa.is_some() {
                NARROW_NODES * (std::mem::size_of::<NodeLinks>() + 4 + 16)
            } else {
                0
            };
            soa + self.image_order64.capacity() * 8
                + self.rules.capacity() * 16
                + self.gen_cache.pairs.capacity() * 8
                + self.gen_cache.contents.capacity() * 4
        }
    }

    #[inline(always)]
    fn join_u128(rules: &mut Vec<[u32; 4]>, left: u128, right: u128) -> u128 {
        if left == 0 {
            return right;
        }
        if right == 0 {
            return left;
        }
        let ll = (left >> 120) as u32;
        let rl = (right >> 120) as u32;
        if ll + rl < 16 {
            return (left + (u128::from(rl) << 120)) | (right << (8 * ll));
        }
        join_u128_long(rules, left, right, ll, rl)
    }

    #[cold]
    #[inline(never)]
    fn join_u128_long(
        rules: &mut Vec<[u32; 4]>,
        left: u128,
        right: u128,
        ll: u32,
        rl: u32,
    ) -> u128 {
        let (lid, l_len) = if ll == 0xFF {
            (left as u32, (left >> 32) as u32)
        } else if ll == 1 {
            ((left as u8) as u32 + 1, 1)
        } else {
            let id = rules.len() as u32 + 257;
            rules.push([
                left as u32,
                (left >> 32) as u32,
                (left >> 64) as u32,
                (left >> 96) as u32,
            ]);
            (id, ll)
        };
        let (rid, r_len) = if rl == 0xFF {
            (right as u32, (right >> 32) as u32)
        } else if rl == 1 {
            ((right as u8) as u32 + 1, 1)
        } else {
            let id = rules.len() as u32 + 257;
            rules.push([
                right as u32,
                (right >> 32) as u32,
                (right >> 64) as u32,
                (right >> 96) as u32,
            ]);
            (id, rl)
        };
        let total_len = l_len + r_len;
        let id = rules.len() as u32 + 257;
        rules.push([lid, rid, total_len, u32::MAX]);
        U128_LONG_TAG | (u128::from(total_len) << 32) | u128::from(id)
    }

    fn power_u128(rules: &mut Vec<[u32; 4]>, word: u128, q: u32) -> u128 {
        let (mut acc, mut base, mut e) = (0u128, word, q);
        loop {
            if e & 1 == 1 {
                acc = join_u128(rules, acc, base);
            }
            e >>= 1;
            if e == 0 {
                break acc;
            }
            base = join_u128(rules, base, base);
        }
    }

    #[inline(always)]
    fn rules_short_payload(rules: &[[u32; 4]], id: u32) -> u128 {
        if id <= 256 {
            return u128::from(id - 1);
        }
        let r = rules[(id - 257) as usize];
        u128::from(r[0])
            | (u128::from(r[1]) << 32)
            | (u128::from(r[2]) << 64)
            | (u128::from(r[3] & 0x00ff_ffff) << 96)
    }

    fn append_rules(rules: &mut [[u32; 4]], word: u32, stack: &mut Vec<u32>, dest: &mut Vec<u8>) {
        stack.clear();
        stack.push(word);
        while let Some(id) = stack.pop() {
            if id <= 256 {
                dest.push((id - 1) as u8);
            } else {
                let idx = (id - 257) as usize;
                let tag = rules[idx][3];
                if tag >> 28 == 0 {
                    let bytes = rules_short_payload(rules, id).to_le_bytes();
                    dest.extend_from_slice(&bytes[..(tag >> 24) as usize]);
                    continue;
                }
                if tag == u32::MAX {
                    rules[idx][3] = 0x8000_0000 | dest.len() as u32;
                    stack.push(rules[idx][1]);
                    stack.push(rules[idx][0]);
                } else {
                    let start = (tag & 0x7fff_ffff) as usize;
                    let len = rules[idx][2] as usize;
                    dest.extend_from_within(start..start + len);
                }
            }
        }
    }

    fn setup_soa_64k(
        strips_raw: &[u32],
        ns: usize,
        n: u32,
        first: &[u32; 256],
        write: &mut [u32; 256],
        img_buf: &mut Vec<u64>,
        soa: &mut Fixed64kSoa,
    ) -> u32 {
        if img_buf.len() < ns {
            img_buf.resize(ns, 0);
        }
        let img = &mut img_buf[..ns];
        let links = &mut *soa.links;
        let widths = &mut *soa.widths;
        let words = &mut *soa.words128;

        links[0] = NodeLinks::default();
        widths[0] = 0;
        words[0] = 0;

        let mut nodes = (1 + ns) as u16;
        let mut widest = 0u32;
        let mut ord0_tail = 0u16;
        let (mut term1_head, mut term1_tail) = (0u16, 0u16);
        let mut x = 0u32;

        for i in 0..ns {
            let base = 3 * i;
            let from = strips_raw[base];
            let to_rel = strips_raw[base + 1];
            let meta = strips_raw[base + 2];
            let byte = (meta & 0xFF) as u8;
            let width = meta >> 8;
            let a = usize::from(byte);
            let to = to_rel + first[a];
            let sid = (i + 1) as u16;

            widths[usize::from(sid)] = width;
            words[usize::from(sid)] = literal_u128(byte);
            widest = widest.max(width);

            let pos = write[a] as usize;
            write[a] = (pos + 1) as u32;
            img[pos] = u64::from(to) | (u64::from(width) << 20) | (u64::from(sid) << 40);

            if x < from {
                let gap = from - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                widths[usize::from(gid)] = gap;
                words[usize::from(gid)] = 0;
                links[usize::from(ord0_tail)].next0 = gid;
                links[usize::from(gid)].prev0 = ord0_tail;
                ord0_tail = gid;
                if term1_tail == 0 {
                    term1_head = gid;
                } else {
                    links[usize::from(term1_tail)].next1 = gid;
                }
                links[usize::from(gid)].prev1 = term1_tail;
                term1_tail = gid;
            }
            links[usize::from(ord0_tail)].next0 = sid;
            links[usize::from(sid)].prev0 = ord0_tail;
            ord0_tail = sid;
            x = from + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            nodes += 1;
            widths[usize::from(gid)] = gap;
            words[usize::from(gid)] = 0;
            links[usize::from(ord0_tail)].next0 = gid;
            links[usize::from(gid)].prev0 = ord0_tail;
            ord0_tail = gid;
            if term1_tail == 0 {
                term1_head = gid;
            } else {
                links[usize::from(term1_tail)].next1 = gid;
            }
            links[usize::from(gid)].prev1 = term1_tail;
            term1_tail = gid;
        }
        let ord0_head = if ord0_tail == 0 { 0 } else { links[0].next0 };

        let mut ord1_tail = 0u16;
        let (mut term0_head, mut term0_tail) = (0u16, 0u16);
        x = 0;
        for &packed in img.iter() {
            let to = (packed & 0xF_FFFF) as u32;
            let width = ((packed >> 20) & 0xF_FFFF) as u32;
            let sid = (packed >> 40) as u16;
            if x < to {
                let gap = to - x;
                widest = widest.max(gap);
                let gid = nodes;
                nodes += 1;
                widths[usize::from(gid)] = gap;
                words[usize::from(gid)] = 0;
                links[usize::from(ord1_tail)].next1 = gid;
                links[usize::from(gid)].prev1 = ord1_tail;
                ord1_tail = gid;
                if term0_tail == 0 {
                    term0_head = gid;
                } else {
                    links[usize::from(term0_tail)].next0 = gid;
                }
                links[usize::from(gid)].prev0 = term0_tail;
                term0_tail = gid;
            }
            links[usize::from(ord1_tail)].next1 = sid;
            links[usize::from(sid)].prev1 = ord1_tail;
            ord1_tail = sid;
            x = to + width;
        }
        if x < n {
            let gap = n - x;
            widest = widest.max(gap);
            let gid = nodes;
            widths[usize::from(gid)] = gap;
            words[usize::from(gid)] = 0;
            links[usize::from(ord1_tail)].next1 = gid;
            links[usize::from(gid)].prev1 = ord1_tail;
            ord1_tail = gid;
            if term0_tail == 0 {
                term0_head = gid;
            } else {
                links[usize::from(term0_tail)].next0 = gid;
            }
            links[usize::from(gid)].prev0 = term0_tail;
            term0_tail = gid;
        }
        let ord1_head = if ord1_tail == 0 { 0 } else { links[0].next1 };

        {
            let mut prev = 0u16;
            for (h, t) in [(term0_head, term0_tail), (ord0_head, ord0_tail)] {
                if t == 0 {
                    continue;
                }
                links[usize::from(prev)].next0 = h;
                links[usize::from(h)].prev0 = prev;
                prev = t;
            }
            links[usize::from(prev)].next0 = 0;
            links[0].prev0 = prev;
        }
        {
            let mut prev = 0u16;
            for (h, t) in [(term1_head, term1_tail), (ord1_head, ord1_tail)] {
                if t == 0 {
                    continue;
                }
                links[usize::from(prev)].next1 = h;
                links[usize::from(h)].prev1 = prev;
                prev = t;
            }
            links[usize::from(prev)].next1 = 0;
            links[0].prev1 = prev;
        }
        widest
    }

    fn eliminate_inline15_soa_64k<const SPIRALS: bool>(
        soa: &mut Fixed64kSoa,
        rules: &mut Vec<[u32; 4]>,
        n: u32,
    ) {
        let links = &mut *soa.links;
        let widths = &mut *soa.widths;
        let words = &mut *soa.words128;
        rules.clear();
        let mut left = n;
        while left != 0 {
            let x = links[0].prev0;
            let y = links[0].prev1;
            let xi = usize::from(x);
            let yi = usize::from(y);
            let aw = widths[xi];
            let bw = widths[yi];
            if SPIRALS && x != y {
                if aw > bw && links[yi].prev1 == x && (aw - 1) / bw >= 2 {
                    let q = (aw - 1) / bw;
                    let head = power_u128(rules, words[xi], q);
                    words[yi] = join_u128(rules, head, words[yi]);
                    widths[xi] = aw - q * bw;
                    left -= q * bw;
                    continue;
                }
                if aw < bw && links[xi].prev0 == y && (bw - 1) / aw >= 2 {
                    let q = (bw - 1) / aw;
                    let tail = power_u128(rules, words[yi], q);
                    words[xi] = join_u128(rules, words[xi], tail);
                    widths[yi] = bw - q * aw;
                    left -= q * aw;
                    continue;
                }
            }
            let w = aw.min(bw);
            left -= w;
            if x == y {
                let xp = links[xi].prev0;
                links[usize::from(xp)].next0 = 0;
                links[0].prev0 = xp;
                let yp = links[yi].prev1;
                links[usize::from(yp)].next1 = 0;
                links[0].prev1 = yp;
                continue;
            }
            let word = join_u128(rules, words[xi], words[yi]);
            if aw > bw {
                let yp = links[yi].prev1;
                links[usize::from(yp)].next1 = 0;
                links[0].prev1 = yp;
                let xn = links[xi].next1;
                links[yi].prev1 = x;
                links[yi].next1 = xn;
                links[xi].next1 = y;
                links[usize::from(xn)].prev1 = y;
                words[yi] = word;
                widths[xi] = aw - bw;
            } else if aw < bw {
                let xp = links[xi].prev0;
                links[usize::from(xp)].next0 = 0;
                links[0].prev0 = xp;
                let yn = links[yi].next0;
                links[xi].prev0 = y;
                links[xi].next0 = yn;
                links[yi].next0 = x;
                links[usize::from(yn)].prev0 = x;
                words[xi] = word;
                widths[yi] = bw - aw;
            } else {
                let xp = links[xi].prev0;
                links[usize::from(xp)].next0 = 0;
                links[0].prev0 = xp;
                let yp0 = links[yi].prev0;
                let yn0 = links[yi].next0;
                links[xi].prev0 = yp0;
                links[xi].next0 = yn0;
                links[usize::from(yp0)].next0 = x;
                links[usize::from(yn0)].prev0 = x;
                let yp1 = links[yi].prev1;
                links[usize::from(yp1)].next1 = 0;
                links[0].prev1 = yp1;
                words[xi] = word;
            }
        }
    }

    fn assign_bottom_soa_64k(
        links: &mut [NodeLinks; NARROW_NODES],
        widths: &[u32; NARROW_NODES],
        rows: u32,
    ) {
        let mut first = links[0].next1;
        let mut last = links[0].prev1;
        let (mut lo, mut hi) = (0u32, rows);
        while first != last {
            let fi = usize::from(first);
            let li = usize::from(last);
            let next = links[fi].next1;
            let prev = links[li].prev1;
            let wf = widths[fi];
            hi -= widths[li];
            links[fi].prev1 = lo as u16;
            links[fi].next1 = (lo >> 16) as u16;
            links[li].prev1 = hi as u16;
            links[li].next1 = (hi >> 16) as u16;
            lo += wf;
            if next == last {
                assert_eq!(lo, hi);
                return;
            }
            first = next;
            last = prev;
        }
        if first != 0 {
            let fi = usize::from(first);
            links[fi].prev1 = lo as u16;
            links[fi].next1 = (lo >> 16) as u16;
            lo += widths[fi];
        }
        assert_eq!(lo, hi);
    }

    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn dict_code_u128_gen(
        word: u128,
        byte: u8,
        rules: &mut [[u32; 4]],
        gen_cache: &mut GenCache,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        spelling: &mut Vec<u8>,
        stack: &mut Vec<u32>,
    ) -> u32 {
        let tag = (word >> 120) as u32;
        if tag < 16 {
            let entry_u128 = (word & U128_PAYLOAD_MASK) | (u128::from(byte) << (8 * tag));
            let len = (tag + 1) as u8;
            let idx = (short_u128_hash(entry_u128, len) as usize) & (GEN_CONTENT_SLOTS - 1);
            let g = gen_cache.content_gen;
            let slot = gen_cache.contents[idx];
            if (slot >> 16) == g {
                let old = (slot & 0xFFFF) as usize;
                if lens[old] == len && u128::from_le_bytes(inline16[old]) == entry_u128 {
                    return old as u32;
                }
            }
            let code = lens.len() as u32;
            inline16.push(entry_u128.to_le_bytes());
            lens.push(len);
            if code < 65_536 {
                gen_cache.contents[idx] = (g << 16) | code;
            }
            code
        } else {
            dict_code_rule_long_gen(
                word as u32,
                byte,
                rules,
                gen_cache,
                inline16,
                lens,
                spelling,
                stack,
            )
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn dict_code_rule_long_gen(
        id: u32,
        byte: u8,
        rules: &mut [[u32; 4]],
        gen_cache: &mut GenCache,
        inline16: &mut Vec<[u8; INLINE]>,
        lens: &mut Vec<u8>,
        spelling: &mut Vec<u8>,
        stack: &mut Vec<u32>,
    ) -> u32 {
        let key = (id << 8) | u32::from(byte);
        let p_idx = (key.wrapping_mul(0x9e37_79b9) >> 22) as usize;
        if id < (1 << 22) {
            let packed = gen_cache.pairs[p_idx];
            if (packed >> 46) == gen_cache.pair_gen && ((packed as u32) & 0x3FFF_FFFF) == key {
                return ((packed >> 30) & 0xFFFF) as u32;
            }
        }
        let start = spelling.len();
        let memoized = rules[(id - 257) as usize][3] != u32::MAX;
        append_rules(rules, id, stack, spelling);
        spelling.push(byte);
        let end = spelling.len();
        let entry = long_slot(start as u32, end as u32);
        let bytes = &spelling[start..end];
        let idx = (long_bytes_hash(bytes) as usize) & (GEN_CONTENT_SLOTS - 1);
        let g = gen_cache.content_gen;
        let slot = gen_cache.contents[idx];
        if (slot >> 16) == g {
            let old = (slot & 0xFFFF) as usize;
            if lens[old] == LONG && long_word(&inline16[old], spelling) == bytes {
                if memoized {
                    spelling.truncate(start);
                } else {
                    spelling.pop();
                }
                let code = old as u32;
                if id < (1 << 22) && code < 65_536 {
                    gen_cache.pairs[p_idx] =
                        (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
                }
                return code;
            }
        }
        let code = lens.len() as u32;
        inline16.push(entry);
        lens.push(LONG);
        if code < 65_536 {
            gen_cache.contents[idx] = (g << 16) | code;
        }
        if id < (1 << 22) && code < 65_536 {
            gen_cache.pairs[p_idx] =
                (gen_cache.pair_gen << 46) | (u64::from(code) << 30) | u64::from(key);
        }
        code
    }

    // -----------------------------------------------------------------------
    // Shipped RLE1+CRC32 & Mechanism 4 (#1045) GF(2) CRC32 combine + AVX2 run scan
    // -----------------------------------------------------------------------

    type Run = (u32, u8, u8);

    #[inline(always)]
    fn find_next_run_scalar(s: &[u8], mut i: usize) -> Option<usize> {
        let n = s.len();
        while i + 3 < n {
            let c = s[i];
            if s[i + 1] == c && s[i + 2] == c && s[i + 3] == c {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn find_next_run_avx2(s: &[u8]) -> Option<usize> {
        let n = s.len();
        let p = s.as_ptr();
        let mut i = 0;
        while i + 35 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
            let eq = _mm256_and_si256(
                _mm256_cmpeq_epi8(v0, v1),
                _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
            );
            let m = _mm256_movemask_epi8(eq) as u32;
            if m != 0 {
                return Some(i + m.trailing_zeros() as usize);
            }
            i += 32;
        }
        find_next_run_scalar(s, i)
    }

    #[inline(always)]
    fn find_next_run(s: &[u8], k: Kernels) -> Option<usize> {
        if k.avx2() {
            unsafe { find_next_run_avx2(s) }
        } else {
            find_next_run_scalar(s, 0)
        }
    }

    #[target_feature(enable = "avx2,bmi1")]
    unsafe fn find_runs_avx2(rle: &[u8], runs: &mut Vec<Run>) -> Result<(), ErrorKind> {
        let n = rle.len();
        let p = rle.as_ptr();
        let mut i = 0;
        while i + 35 <= n {
            let v0 = _mm256_loadu_si256(p.add(i) as *const __m256i);
            let v1 = _mm256_loadu_si256(p.add(i + 1) as *const __m256i);
            let v2 = _mm256_loadu_si256(p.add(i + 2) as *const __m256i);
            let v3 = _mm256_loadu_si256(p.add(i + 3) as *const __m256i);
            let eq = _mm256_and_si256(
                _mm256_cmpeq_epi8(v0, v1),
                _mm256_and_si256(_mm256_cmpeq_epi8(v0, v2), _mm256_cmpeq_epi8(v0, v3)),
            );
            let m = _mm256_movemask_epi8(eq) as u32;
            if m == 0 {
                i += 32;
                continue;
            }
            let j = i + m.trailing_zeros() as usize;
            let Some(&extra) = rle.get(j + 4) else {
                return Err(ErrorKind::RunUnfinished);
            };
            runs.push((j as u32, rle[j], extra));
            i = j + 5;
        }
        while i + 3 < n {
            let c = rle[i];
            if rle[i + 1] == c && rle[i + 2] == c && rle[i + 3] == c {
                let Some(&extra) = rle.get(i + 4) else {
                    return Err(ErrorKind::RunUnfinished);
                };
                runs.push((i as u32, c, extra));
                i += 5;
            } else {
                i += 1;
            }
        }
        Ok(())
    }

    #[inline(never)]
    fn shipped_unrle_in_place(
        buf: &mut Vec<u8>,
        runs: &mut Vec<Run>,
        stage: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        let n = buf.len();
        runs.clear();
        if k.avx2() {
            unsafe { find_runs_avx2(buf, runs)? };
        } else {
            let mut i = 0;
            while i + 3 < n {
                let c = buf[i];
                if buf[i + 1] == c && buf[i + 2] == c && buf[i + 3] == c {
                    let Some(&extra) = buf.get(i + 4) else {
                        return Err(ErrorKind::RunUnfinished);
                    };
                    runs.push((i as u32, c, extra));
                    i += 5;
                } else {
                    i += 1;
                }
            }
        }
        let Some(&(first, _, _)) = runs.first() else {
            return Ok(crc::crc32(buf, k));
        };
        let (mut grown, mut shift) = (0isize, 0isize);
        for &(_, _, extra) in runs.iter() {
            grown += extra as isize - 1;
            shift = shift.max(grown);
        }
        let (shift, len) = (shift as usize, (n as isize + grown) as usize);
        if shift > n {
            std::mem::swap(buf, stage);
            buf.clear();
            rle::unrle(stage, buf, k)?;
            return Ok(crc::crc32(buf, k));
        }
        let first = first as usize;
        if shift > 0 {
            buf.reserve_exact(shift);
            buf.resize(n + shift, 0);
            buf.copy_within(first..n, first + shift);
        }
        let (mut s, mut at) = (!0u32, 0usize);
        let tiles = |s: &mut u32, at: &mut usize, buf: &[u8], end: usize| {
            while end - *at >= RLE_CRC_TILE {
                *s = crc::update_with(*s, &buf[*at..*at + RLE_CRC_TILE], k);
                *at += RLE_CRC_TILE;
            }
        };
        tiles(&mut s, &mut at, buf, first);
        let (mut src, mut dst) = (first + shift, first);
        for &(j, c, extra) in runs.iter() {
            let from = j as usize + shift;
            if from > src {
                if dst != src {
                    buf.copy_within(src..from, dst);
                }
                dst += from - src;
            }
            let run = 4 + extra as usize;
            buf[dst..dst + run].fill(c);
            dst += run;
            src = from + 5;
            tiles(&mut s, &mut at, buf, dst);
        }
        if n + shift > src && dst != src {
            buf.copy_within(src..n + shift, dst);
        }
        buf.truncate(len);
        s = crc::update_with(s, &buf[at..], k);
        Ok(!s)
    }

    struct CrcCombine {
        pow_byte: [[u32; 32]; 32],
    }

    impl CrcCombine {
        fn new() -> Self {
            let mut step1 = [0u32; 32];
            for (j, col) in step1.iter_mut().enumerate() {
                let v = 1u32 << j;
                *col = if v & 0x8000_0000 != 0 {
                    (v << 1) ^ crc::POLY
                } else {
                    v << 1
                };
            }
            let mut byte1 = Self::mat_sq(&step1);
            byte1 = Self::mat_sq(&byte1);
            byte1 = Self::mat_sq(&byte1);
            let mut pow_byte = [[0u32; 32]; 32];
            pow_byte[0] = byte1;
            for k in 1..32 {
                pow_byte[k] = Self::mat_sq(&pow_byte[k - 1]);
            }
            Self { pow_byte }
        }

        #[inline]
        fn mat_vec(mat: &[u32; 32], mut v: u32) -> u32 {
            let mut sum = 0u32;
            let mut i = 0;
            while v != 0 {
                if v & 1 != 0 {
                    sum ^= mat[i];
                }
                v >>= 1;
                i += 1;
            }
            sum
        }

        fn mat_sq(a: &[u32; 32]) -> [u32; 32] {
            let mut out = [0u32; 32];
            for (j, col) in out.iter_mut().enumerate() {
                *col = Self::mat_vec(a, a[j]);
            }
            out
        }

        #[inline]
        fn combine(&self, mut c0: u32, c1: u32, mut len1: usize) -> u32 {
            let mut k = 0;
            while len1 != 0 {
                if len1 & 1 != 0 {
                    c0 = Self::mat_vec(&self.pow_byte[k], c0);
                }
                len1 >>= 1;
                k += 1;
            }
            c0 ^ c1
        }
    }

    #[inline(always)]
    fn leading_eq(s: &[u8], b: u8) -> usize {
        let mut c = 0;
        for &x in s.iter().take(4) {
            if x == b {
                c += 1;
            } else {
                break;
            }
        }
        c
    }

    #[inline(always)]
    fn trailing_run_nonempty(s: &[u8]) -> (u8, u8) {
        let last = s[s.len() - 1];
        let mut cnt = 1u8;
        let mut idx = s.len() - 1;
        while idx > 0 && cnt < 4 && s[idx - 1] == last {
            cnt += 1;
            idx -= 1;
        }
        (last, cnt)
    }

    #[inline(always)]
    fn check_boundary_straddle(
        prev_byte: u8,
        prev_len: u8,
        first4: &[u8; 4],
        first4_len: usize,
    ) -> bool {
        if prev_len == 4 {
            return true;
        }
        if prev_len == 0 || first4_len == 0 {
            return false;
        }
        let lead = leading_eq(&first4[..first4_len], prev_byte);
        usize::from(prev_len) + lead >= 4
    }

    // -----------------------------------------------------------------------
    // Emission kernels: shipped bidi, 1-lane fwd, #1057 4-lane sampled (unfused & #1045 fused)
    // -----------------------------------------------------------------------

    #[inline(always)]
    fn repeat(n: usize, out: &mut Vec<u8>) {
        assert!(!out.is_empty() && out.len() <= n);
        while out.len() < n {
            let take = out.len().min(n - out.len());
            out.extend_from_within(..take);
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_fwd_1lane(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        out.resize(n + INLINE, 0);
        let mut at = 0usize;
        let mask = (1u32 << bits) - 1;
        let mut j = mark;
        for _ in 0..edges.len() {
            let edge = edges[j as usize];
            let code = if extra.is_empty() {
                edge >> bits
            } else {
                extra[j as usize]
            } as usize;
            j = edge & mask;
            let len = lens[code];
            if len != LONG {
                out[at..at + INLINE].copy_from_slice(&inline16[code]);
                at += usize::from(len);
            } else {
                let word = long_word(&inline16[code], spelling);
                let take = word.len().min(n - at);
                out[at..at + take].copy_from_slice(&word[..take]);
                at += take;
            }
            if j == mark || at >= n {
                break;
            }
        }
        out.truncate(at.min(n));
        repeat(n, out);
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_bidi(
        edges: &[u32],
        extra: &[u32],
        spelling: &[u8],
        inline: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        mark: u32,
        predecessor: Option<u32>,
        bits: u32,
        out: &mut Vec<u8>,
    ) {
        let Some(before) = predecessor else {
            emit_fwd_1lane(edges, extra, spelling, inline, lens, n, mark, bits, out);
            return;
        };
        out.resize(n + INLINE, 0);
        let mask = (1u32 << bits) - 1;
        let (mut prev, mut front) = (before, mark);
        let (mut back, mut next) = (before, mark);
        let (mut lo, mut hi) = (0usize, n);
        for pair in 0..=edges.len() / 2 {
            if lo == hi || (pair != 0 && front == next) {
                break;
            }
            let ef = edges[front as usize];
            let cf = (ef >> bits) as usize;
            let single = front == back;
            let lf = lens[cf];
            if lf != LONG && hi - lo >= INLINE {
                out[lo..lo + INLINE].copy_from_slice(&inline[cf]);
                lo += usize::from(lf);
            } else {
                let word = if lf == LONG {
                    long_word(&inline[cf], spelling)
                } else {
                    &inline[cf][..usize::from(lf)]
                };
                let take = word.len().min(hi - lo);
                out[lo..lo + take].copy_from_slice(&word[..take]);
                lo += take;
            }
            if single {
                break;
            }
            let eb = edges[back as usize];
            let cb = (eb >> bits) as usize;
            let lb = lens[cb];
            if lb != LONG && hi - lo >= INLINE {
                let shifted = u128::from_le_bytes(inline[cb]) << (8 * (INLINE - usize::from(lb)));
                out[hi - INLINE..hi].copy_from_slice(&shifted.to_le_bytes());
                hi -= usize::from(lb);
            } else {
                let word = if lb == LONG {
                    long_word(&inline[cb], spelling)
                } else {
                    &inline[cb][..usize::from(lb)]
                };
                let take = word.len().min(hi - lo);
                out[hi - take..hi].copy_from_slice(&word[word.len() - take..]);
                hi -= take;
            }
            (prev, front) = (front, (ef & mask) ^ prev);
            (next, back) = (back, (eb & mask) ^ next);
        }
        if lo < hi {
            out.copy_within(hi..n, lo);
            out.truncate(lo + (n - hi));
            repeat(n, out);
        } else {
            out.truncate(n);
        }
    }

    #[derive(Clone, Copy, Default)]
    struct SliceRef {
        off: u32,
        len: u32,
        next: u16,
    }

    #[derive(Default)]
    struct SampledScratch {
        slices: Vec<SliceRef>,
        ordered: Vec<(usize, usize)>,
        rle_spill: Vec<u8>,
    }

    impl SampledScratch {
        fn scratch_bytes(&self) -> usize {
            self.slices.capacity() * std::mem::size_of::<SliceRef>()
                + self.ordered.capacity() * std::mem::size_of::<(usize, usize)>()
                + self.rle_spill.capacity()
        }
    }

    struct SlabAlloc<'a, const SLAB: usize, const COLD_COMPACT: bool> {
        arena: &'a mut Vec<u8>,
        top: usize,
        slices: &'a mut Vec<SliceRef>,
        seg_head: [u16; MAX_ANCHORS],
        seg_tail: [u16; MAX_ANCHORS],
    }

    impl<'a, const SLAB: usize, const COLD_COMPACT: bool> SlabAlloc<'a, SLAB, COLD_COMPACT> {
        #[inline(always)]
        fn push_slice(&mut self, seg: usize, start: usize, end: usize) {
            if end > start {
                let id = self.slices.len() as u16;
                self.slices.push(SliceRef {
                    off: start as u32,
                    len: (end - start) as u32,
                    next: u16::MAX,
                });
                if self.seg_head[seg] == u16::MAX {
                    self.seg_head[seg] = id;
                } else {
                    self.slices[self.seg_tail[seg] as usize].next = id;
                }
                self.seg_tail[seg] = id;
            }
        }

        #[inline(always)]
        fn take_slab(&mut self, min_bytes: usize) -> (usize, usize) {
            let size = SLAB.max(min_bytes + INLINE);
            let base = self.top;
            self.top = base + size;
            if self.top > self.arena.len() {
                let want = self.top + 4 * SLAB;
                if want > self.arena.capacity() {
                    self.arena.reserve_exact(want - self.arena.len());
                }
                self.arena.resize(want, 0);
            }
            (base, base + size)
        }

        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn compact_and_crc_slab(
            &mut self,
            seg: usize,
            slice_start: &mut usize,
            raw_end: usize,
            slab_end: &mut usize,
            rle_spill: &mut Vec<u8>,
            first4: &mut [u8; 4],
            first4_len: &mut usize,
            run_byte: &mut u8,
            run_len: &mut u8,
            crc_state: &mut u32,
            out_len: &mut usize,
            k: Kernels,
        ) -> usize {
            if COLD_COMPACT {
                self.compact_and_crc_slab_cold(
                    seg,
                    slice_start,
                    raw_end,
                    slab_end,
                    rle_spill,
                    first4,
                    first4_len,
                    run_byte,
                    run_len,
                    crc_state,
                    out_len,
                    k,
                )
            } else {
                self.compact_and_crc_slab_inline(
                    seg,
                    slice_start,
                    raw_end,
                    slab_end,
                    rle_spill,
                    first4,
                    first4_len,
                    run_byte,
                    run_len,
                    crc_state,
                    out_len,
                    k,
                )
            }
        }

        /// Cold-outlined compaction + CRC32 (`#[cold] #[inline(never)]`) so RLE1+CRC codegen
        /// never bloats `emit_4lane_sampled<true>`'s 4-lane hot loop.
        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn compact_and_crc_slab_cold(
            &mut self,
            seg: usize,
            slice_start: &mut usize,
            raw_end: usize,
            slab_end: &mut usize,
            rle_spill: &mut Vec<u8>,
            first4: &mut [u8; 4],
            first4_len: &mut usize,
            run_byte: &mut u8,
            run_len: &mut u8,
            crc_state: &mut u32,
            out_len: &mut usize,
            k: Kernels,
        ) -> usize {
            self.compact_and_crc_slab_body(
                seg,
                slice_start,
                raw_end,
                slab_end,
                rle_spill,
                first4,
                first4_len,
                run_byte,
                run_len,
                crc_state,
                out_len,
                k,
            )
        }

        /// Original `#1062` `#[inline]` compaction variant for exact A/B comparison.
        #[inline]
        #[allow(clippy::too_many_arguments)]
        fn compact_and_crc_slab_inline(
            &mut self,
            seg: usize,
            slice_start: &mut usize,
            raw_end: usize,
            slab_end: &mut usize,
            rle_spill: &mut Vec<u8>,
            first4: &mut [u8; 4],
            first4_len: &mut usize,
            run_byte: &mut u8,
            run_len: &mut u8,
            crc_state: &mut u32,
            out_len: &mut usize,
            k: Kernels,
        ) -> usize {
            self.compact_and_crc_slab_body(
                seg,
                slice_start,
                raw_end,
                slab_end,
                rle_spill,
                first4,
                first4_len,
                run_byte,
                run_len,
                crc_state,
                out_len,
                k,
            )
        }

        /// Zero-scratch single-pass in-place L2-hot RLE1 compaction + PCLMULQDQ CRC32
        /// on `arena[slice_start..raw_end]`. Returns new compacted cursor `dst` inside the
        /// current slab (or a newly allocated slab if expansion spilled past `slab_end`).
        #[inline(always)]
        #[allow(clippy::too_many_arguments)]
        fn compact_and_crc_slab_body(
            &mut self,
            seg: usize,
            slice_start: &mut usize,
            raw_end: usize,
            slab_end: &mut usize,
            rle_spill: &mut Vec<u8>,
            first4: &mut [u8; 4],
            first4_len: &mut usize,
            run_byte: &mut u8,
            run_len: &mut u8,
            crc_state: &mut u32,
            out_len: &mut usize,
            k: Kernels,
        ) -> usize {
            let rs = *slice_start;
            let raw_len = raw_end - rs;
            if raw_len == 0 {
                return rs;
            }
            if *first4_len < 4 {
                let need = (4 - *first4_len).min(raw_len);
                first4[*first4_len..*first4_len + need].copy_from_slice(&self.arena[rs..rs + need]);
                *first4_len += need;
            }

            let raw = &self.arena[rs..raw_end];
            let boundary_ok = *run_len != 4
                && (*run_len == 0
                    || raw[0] != *run_byte
                    || (usize::from(*run_len) + leading_eq(raw, *run_byte)) < 4);

            if boundary_ok {
                match find_next_run(raw, k) {
                    None => {
                        let (last, tr) = trailing_run_nonempty(raw);
                        if usize::from(tr) == raw_len && *run_len > 0 && last == *run_byte {
                            *run_len += tr;
                        } else {
                            *run_byte = last;
                            *run_len = tr;
                        }
                        *crc_state = crc::update_with(*crc_state, raw, k);
                        self.push_slice(seg, rs, raw_end);
                        *out_len += raw_len;
                        return raw_end;
                    }
                    Some(first_rel) => {
                        // Fast zero-scratch in-place compaction while dst <= src
                        let mut src = rs + first_rel;
                        let mut dst = src;
                        let mut spilled = false;
                        loop {
                            let c = self.arena[src];
                            if src + 4 < raw_end {
                                let extra = self.arena[src + 4] as usize;
                                let out_run = 4 + extra;
                                let next_src = src + 5;
                                if dst + out_run <= next_src {
                                    self.arena[dst..dst + out_run].fill(c);
                                    dst += out_run;
                                    src = next_src;
                                } else {
                                    rle_spill.clear();
                                    rle_spill.extend_from_slice(&self.arena[src..raw_end]);
                                    *run_len = 0;
                                    spilled = true;
                                    break;
                                }
                            } else {
                                self.arena[dst..dst + 4].fill(c);
                                dst += 4;
                                *run_byte = c;
                                *run_len = 4;
                                break;
                            }
                            if src >= raw_end {
                                *run_len = 0;
                                break;
                            }
                            match find_next_run(&self.arena[src..raw_end], k) {
                                None => {
                                    let rem = raw_end - src;
                                    let (last, tr) =
                                        trailing_run_nonempty(&self.arena[src..raw_end]);
                                    if dst != src {
                                        self.arena.copy_within(src..raw_end, dst);
                                    }
                                    dst += rem;
                                    *run_byte = last;
                                    *run_len = tr;
                                    break;
                                }
                                Some(r2) => {
                                    if r2 > 0 {
                                        if dst != src {
                                            self.arena.copy_within(src..src + r2, dst);
                                        }
                                        dst += r2;
                                        src += r2;
                                    }
                                }
                            }
                        }
                        if !spilled {
                            *crc_state = crc::update_with(*crc_state, &self.arena[rs..dst], k);
                            self.push_slice(seg, rs, dst);
                            *out_len += dst - rs;
                            return dst;
                        }
                        // Finish spilled suffix into current or overflow slab(s)
                        return self.expand_spill_into_slabs(
                            seg,
                            slice_start,
                            dst,
                            slab_end,
                            rle_spill,
                            run_byte,
                            run_len,
                            crc_state,
                            out_len,
                            k,
                            false,
                        );
                    }
                }
            }

            // Rare boundary run (`!boundary_ok`): spill whole tile and expand cleanly
            rle_spill.clear();
            rle_spill.extend_from_slice(raw);
            self.expand_spill_into_slabs(
                seg,
                slice_start,
                rs,
                slab_end,
                rle_spill,
                run_byte,
                run_len,
                crc_state,
                out_len,
                k,
                true,
            )
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn expand_spill_into_slabs(
            &mut self,
            seg: usize,
            slice_start: &mut usize,
            mut dst: usize,
            slab_end: &mut usize,
            rle_spill: &[u8],
            run_byte: &mut u8,
            run_len: &mut u8,
            crc_state: &mut u32,
            out_len: &mut usize,
            k: Kernels,
            handle_boundary: bool,
        ) -> usize {
            let mut pos = 0usize;
            let slen = rle_spill.len();

            macro_rules! ensure_room {
                ($need:expr) => {{
                    let need = $need;
                    if dst + need + INLINE > *slab_end {
                        if dst > *slice_start {
                            *crc_state =
                                crc::update_with(*crc_state, &self.arena[*slice_start..dst], k);
                            self.push_slice(seg, *slice_start, dst);
                            *out_len += dst - *slice_start;
                        }
                        let (nb, ne) = self.take_slab(need);
                        *slice_start = nb;
                        dst = nb;
                        *slab_end = ne;
                    }
                }};
            }

            if handle_boundary {
                if *run_len == 4 {
                    let extra = rle_spill[0] as usize;
                    if extra > 0 {
                        ensure_room!(extra);
                        self.arena[dst..dst + extra].fill(*run_byte);
                        dst += extra;
                    }
                    *run_len = 0;
                    pos = 1;
                }
                while pos < slen && *run_len > 0 {
                    let b = rle_spill[pos];
                    if b == *run_byte {
                        ensure_room!(1);
                        self.arena[dst] = b;
                        dst += 1;
                        *run_len += 1;
                        pos += 1;
                        if *run_len == 4 {
                            if pos < slen {
                                let extra = rle_spill[pos] as usize;
                                if extra > 0 {
                                    ensure_room!(extra);
                                    self.arena[dst..dst + extra].fill(*run_byte);
                                    dst += extra;
                                }
                                *run_len = 0;
                                pos += 1;
                            }
                            break;
                        }
                    } else {
                        *run_len = 0;
                        break;
                    }
                }
            }

            while pos < slen {
                match find_next_run(&rle_spill[pos..], k) {
                    None => {
                        let rem = slen - pos;
                        ensure_room!(rem);
                        self.arena[dst..dst + rem].copy_from_slice(&rle_spill[pos..]);
                        dst += rem;
                        let (last, tr) = trailing_run_nonempty(&rle_spill[pos..]);
                        *run_byte = last;
                        *run_len = tr;
                        break;
                    }
                    Some(rel) => {
                        if rel > 0 {
                            ensure_room!(rel);
                            self.arena[dst..dst + rel].copy_from_slice(&rle_spill[pos..pos + rel]);
                            dst += rel;
                        }
                        let j = pos + rel;
                        let c = rle_spill[j];
                        if j + 4 < slen {
                            let extra = rle_spill[j + 4] as usize;
                            let total = 4 + extra;
                            ensure_room!(total);
                            self.arena[dst..dst + total].fill(c);
                            dst += total;
                            pos = j + 5;
                        } else {
                            ensure_room!(4);
                            self.arena[dst..dst + 4].fill(c);
                            dst += 4;
                            *run_byte = c;
                            *run_len = 4;
                            break;
                        }
                    }
                }
            }

            if dst > *slice_start {
                *crc_state = crc::update_with(*crc_state, &self.arena[*slice_start..dst], k);
                self.push_slice(seg, *slice_start, dst);
                *out_len += dst - *slice_start;
            }
            dst
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_slow_raw(
            &mut self,
            seg: usize,
            code: usize,
            len: u8,
            inline16: &[[u8; INLINE]],
            spelling: &[u8],
            slice_start: &mut usize,
            at: &mut usize,
            end: &mut usize,
        ) {
            let word = if len == LONG {
                long_word(&inline16[code], spelling)
            } else {
                &inline16[code][..usize::from(len)]
            };
            let wlen = word.len();
            if *at + wlen + INLINE > *end {
                self.push_slice(seg, *slice_start, *at);
                let (new_base, new_end) = self.take_slab(wlen);
                *slice_start = new_base;
                *at = new_base;
                *end = new_end;
            }
            self.arena[*at..*at + wlen].copy_from_slice(word);
            *at += wlen;
        }

        #[cold]
        #[inline(never)]
        #[allow(clippy::too_many_arguments)]
        fn emit_slow_fused(
            &mut self,
            seg: usize,
            code: usize,
            len: u8,
            inline16: &[[u8; INLINE]],
            spelling: &[u8],
            slice_start: &mut usize,
            at: &mut usize,
            end: &mut usize,
            rle_spill: &mut Vec<u8>,
            first4: &mut [u8; 4],
            first4_len: &mut usize,
            run_byte: &mut u8,
            run_len: &mut u8,
            crc_state: &mut u32,
            out_len: &mut usize,
            k: Kernels,
        ) {
            let word = if len == LONG {
                long_word(&inline16[code], spelling)
            } else {
                &inline16[code][..usize::from(len)]
            };
            let wlen = word.len();
            if *at + wlen + INLINE > *end {
                self.compact_and_crc_slab(
                    seg,
                    slice_start,
                    *at,
                    end,
                    rle_spill,
                    first4,
                    first4_len,
                    run_byte,
                    run_len,
                    crc_state,
                    out_len,
                    k,
                );
                let (new_base, new_end) = self.take_slab(wlen);
                *slice_start = new_base;
                *at = new_base;
                *end = new_end;
            }
            self.arena[*at..*at + wlen].copy_from_slice(word);
            *at += wlen;
        }
    }

    #[inline(always)]
    fn tag_uniform_anchors(
        edges: &mut [u32],
        m: usize,
        mark: u32,
        bits: u32,
        wide: bool,
        dict_len: usize,
        anchors: &mut [u32; MAX_ANCHORS],
    ) -> usize {
        let can_tag =
            !wide && m >= S_ANCHORS * 8 && (dict_len - 1) <= ((u32::MAX >> 1) >> bits) as usize;
        if !can_tag {
            return 0;
        }
        let m32 = m as u32;
        anchors[0] = mark;
        for (k, slot) in anchors.iter_mut().enumerate().take(S_ANCHORS).skip(1) {
            let off = ((k as u64 * m as u64) / S_ANCHORS as u64) as u32;
            let mut a = mark + off;
            if a >= m32 {
                a -= m32;
            }
            *slot = a;
        }
        for &a in &anchors[..S_ANCHORS] {
            edges[a as usize] |= ANCHOR_TAG;
        }
        S_ANCHORS
    }

    /// Re-walk a single segment `seg` sequentially with initial RLE1 state `(init_rb, init_rl)`
    /// on the rare occurrence of a cross-segment RLE1 boundary straddle.
    #[cold]
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn repair_segment_fused<const SLAB: usize, const COLD_COMPACT: bool>(
        seg: usize,
        start_row: u32,
        end_row: u32,
        edges: &[u32],
        bits: u32,
        mask: u32,
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        spelling: &[u8],
        init_rb: u8,
        init_rl: u8,
        alloc: &mut SlabAlloc<'_, SLAB, COLD_COMPACT>,
        rle_spill: &mut Vec<u8>,
        k: Kernels,
    ) -> (u32, usize, u8, u8) {
        alloc.seg_head[seg] = u16::MAX;
        alloc.seg_tail[seg] = u16::MAX;
        let (mut slice_start, mut end) = alloc.take_slab(0);
        let mut at = slice_start;
        let mut rb = init_rb;
        let mut rl = init_rl;
        let mut crc_st = 0u32;
        let mut out_len = 0usize;
        let mut dummy_f4 = [0u8; 4];
        let mut dummy_f4l = 4usize;

        let mut j = start_row;
        loop {
            let ed = edges[j as usize];
            j = ed & mask;
            let c = (ed >> bits) as usize;
            let l = lens[c];
            if l != LONG && at + INLINE <= end {
                alloc.arena[at..at + INLINE].copy_from_slice(&inline16[c]);
                at += usize::from(l);
            } else {
                alloc.emit_slow_fused(
                    seg,
                    c,
                    l,
                    inline16,
                    spelling,
                    &mut slice_start,
                    &mut at,
                    &mut end,
                    rle_spill,
                    &mut dummy_f4,
                    &mut dummy_f4l,
                    &mut rb,
                    &mut rl,
                    &mut crc_st,
                    &mut out_len,
                    k,
                );
            }
            if j == end_row {
                break;
            }
        }
        alloc.compact_and_crc_slab(
            seg,
            &mut slice_start,
            at,
            &mut end,
            rle_spill,
            &mut dummy_f4,
            &mut dummy_f4l,
            &mut rb,
            &mut rl,
            &mut crc_st,
            &mut out_len,
            k,
        );
        (crc_st, out_len, rb, rl)
    }

    /// Dynamic 4-lane sampled-anchor emitter, const-generic over `FUSED_RLE`, `SLAB`, and `COLD_COMPACT`:
    /// - `FUSED_RLE = false`: raw slab emit (`#1057`), returning `(0, ordered_raw_slices)`.
    /// - `FUSED_RLE = true`: `#1057 + #1045` — in-flight `SLAB`-byte L2-hot zero-scratch RLE1
    ///   compaction + PCLMULQDQ CRC32 + exact straddle repair + GF(2) `crc32_combine`,
    ///   returning `(final_crc, ordered_decoded_slices)` with ZERO stitch copy!
    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn emit_4lane_sampled<const FUSED_RLE: bool, const SLAB: usize, const COLD_COMPACT: bool>(
        edges: &mut [u32],
        extra: &[u32],
        spelling: &[u8],
        inline16: &[[u8; INLINE]],
        lens: &[u8],
        n: usize,
        m: usize,
        mark: u32,
        bits: u32,
        anchors: &[u32],
        arena: &mut Vec<u8>,
        scratch: &mut SampledScratch,
        crc_combine: &CrcCombine,
        k: Kernels,
        rle_runs: &mut Vec<Run>,
        rle_stage: &mut Vec<u8>,
    ) -> Result<u32, ErrorKind> {
        let s = anchors.len();
        if s < 4 {
            for &a in anchors {
                edges[a as usize] &= !ANCHOR_TAG;
            }
            emit_fwd_1lane(edges, extra, spelling, inline16, lens, n, mark, bits, arena);
            let c = if FUSED_RLE {
                shipped_unrle_in_place(arena, rle_runs, rle_stage, k)?
            } else {
                0
            };
            scratch.ordered.clear();
            scratch.ordered.push((0, arena.len()));
            return Ok(c);
        }

        let mask = (1u32 << bits) - 1;
        let want_arena = n + 8 * SLAB + 4 * INLINE;
        if arena.len() < want_arena {
            if want_arena > arena.capacity() {
                arena.reserve_exact(want_arena - arena.len());
            }
            arena.resize(want_arena, 0);
        }
        scratch.slices.clear();
        scratch.ordered.clear();

        let mut alloc = SlabAlloc::<SLAB, COLD_COMPACT> {
            arena,
            top: 0,
            slices: &mut scratch.slices,
            seg_head: [u16::MAX; MAX_ANCHORS],
            seg_tail: [u16::MAX; MAX_ANCHORS],
        };

        let mut seg_end_row = [u32::MAX; MAX_ANCHORS];
        let mut seg_first4 = [[0u8; 4]; MAX_ANCHORS];
        let mut seg_first4_len = [0usize; MAX_ANCHORS];
        let mut seg_rb = [0u8; MAX_ANCHORS];
        let mut seg_rl = [0u8; MAX_ANCHORS];
        let mut seg_crc = [0u32; MAX_ANCHORS];
        seg_crc[0] = !0u32; // seg 0 is always mark (chronologically first!)
        let mut seg_out_len = [0usize; MAX_ANCHORS];

        let mut lj = [0u32; 4];
        let mut lseg = [0usize; 4];
        let mut lstart = [0usize; 4];
        let mut lat = [0usize; 4];
        let mut lend = [0usize; 4];
        let mut lactive = [false; 4];

        let mut next_seg = 0usize;
        let mut steps_done = 0usize;
        let mut budget_exceeded = false;

        macro_rules! emit_slow_lane {
            ($lane:expr, $seg:expr, $c:expr, $l:expr, $a:expr, $e:expr) => {{
                if FUSED_RLE {
                    alloc.emit_slow_fused(
                        $seg,
                        $c,
                        $l,
                        inline16,
                        spelling,
                        &mut lstart[$lane],
                        &mut $a,
                        &mut $e,
                        &mut scratch.rle_spill,
                        &mut seg_first4[$seg],
                        &mut seg_first4_len[$seg],
                        &mut seg_rb[$seg],
                        &mut seg_rl[$seg],
                        &mut seg_crc[$seg],
                        &mut seg_out_len[$seg],
                        k,
                    );
                } else {
                    alloc.emit_slow_raw(
                        $seg,
                        $c,
                        $l,
                        inline16,
                        spelling,
                        &mut lstart[$lane],
                        &mut $a,
                        &mut $e,
                    );
                }
            }};
        }

        macro_rules! start_seg {
            ($lane:expr, $seg:expr) => {{
                lseg[$lane] = $seg;
                lstart[$lane] = lat[$lane];
                let edge = edges[anchors[$seg] as usize] & !ANCHOR_TAG;
                lj[$lane] = edge & mask;
                let code = (edge >> bits) as usize;
                let len = lens[code];
                if len != LONG && lat[$lane] + INLINE <= lend[$lane] {
                    let at = lat[$lane];
                    alloc.arena[at..at + INLINE].copy_from_slice(&inline16[code]);
                    lat[$lane] = at + usize::from(len);
                } else {
                    emit_slow_lane!($lane, $seg, code, len, lat[$lane], lend[$lane]);
                }
                steps_done += 1;
            }};
        }

        macro_rules! handle_hit {
            ($lane:expr) => {{
                let mut keep = true;
                while (edges[lj[$lane] as usize] as i32) < 0 {
                    let finished = lseg[$lane];
                    seg_end_row[finished] = lj[$lane];
                    if FUSED_RLE {
                        let dst = alloc.compact_and_crc_slab(
                            finished,
                            &mut lstart[$lane],
                            lat[$lane],
                            &mut lend[$lane],
                            &mut scratch.rle_spill,
                            &mut seg_first4[finished],
                            &mut seg_first4_len[finished],
                            &mut seg_rb[finished],
                            &mut seg_rl[finished],
                            &mut seg_crc[finished],
                            &mut seg_out_len[finished],
                            k,
                        );
                        lat[$lane] = dst;
                    } else {
                        alloc.push_slice(finished, lstart[$lane], lat[$lane]);
                    }
                    if next_seg < s {
                        if steps_done >= m {
                            lactive[$lane] = false;
                            keep = false;
                            break;
                        }
                        let seg = next_seg;
                        next_seg += 1;
                        start_seg!($lane, seg);
                    } else {
                        lactive[$lane] = false;
                        keep = false;
                        break;
                    }
                }
                keep
            }};
        }

        for lane in 0..4 {
            let (base, end) = alloc.take_slab(0);
            lstart[lane] = base;
            lat[lane] = base;
            lend[lane] = end;
            lactive[lane] = true;
            let seg = next_seg;
            next_seg += 1;
            start_seg!(lane, seg);
        }
        for lane in 0..4 {
            if (edges[lj[lane] as usize] as i32) < 0 {
                let _ = handle_hit!(lane);
            }
        }

        // Phase 1: 4-lane counted hot loop
        if lactive[0] && lactive[1] && lactive[2] && lactive[3] {
            let (mut j0, mut j1, mut j2, mut j3) = (lj[0], lj[1], lj[2], lj[3]);
            let (mut a0, mut a1, mut a2, mut a3) = (lat[0], lat[1], lat[2], lat[3]);
            let (mut e0, mut e1, mut e2, mut e3) = (lend[0], lend[1], lend[2], lend[3]);

            loop {
                let max_iters = (m - steps_done) / 4;
                let mut done = 0usize;
                let mut hit = false;
                for _ in 0..max_iters {
                    let ed0 = edges[j0 as usize];
                    let ed1 = edges[j1 as usize];
                    let ed2 = edges[j2 as usize];
                    let ed3 = edges[j3 as usize];

                    if ((ed0 | ed1 | ed2 | ed3) as i32) < 0 {
                        hit = true;
                        break;
                    }

                    j0 = ed0 & mask;
                    j1 = ed1 & mask;
                    j2 = ed2 & mask;
                    j3 = ed3 & mask;

                    let c0 = (ed0 >> bits) as usize;
                    let l0 = lens[c0];
                    if l0 != LONG && a0 + INLINE <= e0 {
                        alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                        a0 += usize::from(l0);
                    } else {
                        emit_slow_lane!(0, lseg[0], c0, l0, a0, e0);
                    }

                    let c1 = (ed1 >> bits) as usize;
                    let l1 = lens[c1];
                    if l1 != LONG && a1 + INLINE <= e1 {
                        alloc.arena[a1..a1 + INLINE].copy_from_slice(&inline16[c1]);
                        a1 += usize::from(l1);
                    } else {
                        emit_slow_lane!(1, lseg[1], c1, l1, a1, e1);
                    }

                    let c2 = (ed2 >> bits) as usize;
                    let l2 = lens[c2];
                    if l2 != LONG && a2 + INLINE <= e2 {
                        alloc.arena[a2..a2 + INLINE].copy_from_slice(&inline16[c2]);
                        a2 += usize::from(l2);
                    } else {
                        emit_slow_lane!(2, lseg[2], c2, l2, a2, e2);
                    }

                    let c3 = (ed3 >> bits) as usize;
                    let l3 = lens[c3];
                    if l3 != LONG && a3 + INLINE <= e3 {
                        alloc.arena[a3..a3 + INLINE].copy_from_slice(&inline16[c3]);
                        a3 += usize::from(l3);
                    } else {
                        emit_slow_lane!(3, lseg[3], c3, l3, a3, e3);
                    }

                    done += 1;
                }
                steps_done += done * 4;
                lj = [j0, j1, j2, j3];
                lat = [a0, a1, a2, a3];
                lend = [e0, e1, e2, e3];
                if !hit
                    && ((edges[j0 as usize]
                        | edges[j1 as usize]
                        | edges[j2 as usize]
                        | edges[j3 as usize]) as i32)
                        >= 0
                {
                    budget_exceeded = true;
                    break;
                }
                let mut all_ok = true;
                for lane in 0..4 {
                    if (edges[lj[lane] as usize] as i32) < 0 && !handle_hit!(lane) {
                        all_ok = false;
                    }
                }
                if !all_ok {
                    break;
                }
                (j0, j1, j2, j3) = (lj[0], lj[1], lj[2], lj[3]);
                (a0, a1, a2, a3) = (lat[0], lat[1], lat[2], lat[3]);
                (e0, e1, e2, e3) = (lend[0], lend[1], lend[2], lend[3]);
            }
        }

        // Phase 2: 3 -> 2 -> 1 lane tail drain
        let mut act = [0usize; 4];
        let mut n_act = 0usize;
        for (lane, &active) in lactive.iter().enumerate() {
            if active {
                act[n_act] = lane;
                n_act += 1;
            }
        }

        while !budget_exceeded && n_act == 3 {
            let (p0, p1, p2) = (act[0], act[1], act[2]);
            let (mut j0, mut j1, mut j2) = (lj[p0], lj[p1], lj[p2]);
            let (mut a0, mut a1, mut a2) = (lat[p0], lat[p1], lat[p2]);
            let (mut e0, mut e1, mut e2) = (lend[p0], lend[p1], lend[p2]);
            loop {
                let ed0 = edges[j0 as usize];
                let ed1 = edges[j1 as usize];
                let ed2 = edges[j2 as usize];
                if ((ed0 | ed1 | ed2) as i32) < 0 {
                    (lj[p0], lj[p1], lj[p2]) = (j0, j1, j2);
                    (lat[p0], lat[p1], lat[p2]) = (a0, a1, a2);
                    (lend[p0], lend[p1], lend[p2]) = (e0, e1, e2);
                    break;
                }
                if steps_done + 3 > m {
                    budget_exceeded = true;
                    break;
                }
                j0 = ed0 & mask;
                j1 = ed1 & mask;
                j2 = ed2 & mask;
                let c0 = (ed0 >> bits) as usize;
                let l0 = lens[c0];
                if l0 != LONG && a0 + INLINE <= e0 {
                    alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                    a0 += usize::from(l0);
                } else {
                    emit_slow_lane!(p0, lseg[p0], c0, l0, a0, e0);
                }
                let c1 = (ed1 >> bits) as usize;
                let l1 = lens[c1];
                if l1 != LONG && a1 + INLINE <= e1 {
                    alloc.arena[a1..a1 + INLINE].copy_from_slice(&inline16[c1]);
                    a1 += usize::from(l1);
                } else {
                    emit_slow_lane!(p1, lseg[p1], c1, l1, a1, e1);
                }
                let c2 = (ed2 >> bits) as usize;
                let l2 = lens[c2];
                if l2 != LONG && a2 + INLINE <= e2 {
                    alloc.arena[a2..a2 + INLINE].copy_from_slice(&inline16[c2]);
                    a2 += usize::from(l2);
                } else {
                    emit_slow_lane!(p2, lseg[p2], c2, l2, a2, e2);
                }
                steps_done += 3;
            }
            if budget_exceeded {
                break;
            }
            let mut next_act = [0usize; 4];
            let mut new_n = 0;
            for &lane in &act[..n_act] {
                if (edges[lj[lane] as usize] as i32) < 0 {
                    if handle_hit!(lane) {
                        next_act[new_n] = lane;
                        new_n += 1;
                    }
                } else {
                    next_act[new_n] = lane;
                    new_n += 1;
                }
            }
            act = next_act;
            n_act = new_n;
        }

        while !budget_exceeded && n_act == 2 {
            let (p0, p1) = (act[0], act[1]);
            let (mut j0, mut j1) = (lj[p0], lj[p1]);
            let (mut a0, mut a1) = (lat[p0], lat[p1]);
            let (mut e0, mut e1) = (lend[p0], lend[p1]);
            loop {
                let ed0 = edges[j0 as usize];
                let ed1 = edges[j1 as usize];
                if ((ed0 | ed1) as i32) < 0 {
                    (lj[p0], lj[p1]) = (j0, j1);
                    (lat[p0], lat[p1]) = (a0, a1);
                    (lend[p0], lend[p1]) = (e0, e1);
                    break;
                }
                if steps_done + 2 > m {
                    budget_exceeded = true;
                    break;
                }
                j0 = ed0 & mask;
                j1 = ed1 & mask;
                let c0 = (ed0 >> bits) as usize;
                let l0 = lens[c0];
                if l0 != LONG && a0 + INLINE <= e0 {
                    alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                    a0 += usize::from(l0);
                } else {
                    emit_slow_lane!(p0, lseg[p0], c0, l0, a0, e0);
                }
                let c1 = (ed1 >> bits) as usize;
                let l1 = lens[c1];
                if l1 != LONG && a1 + INLINE <= e1 {
                    alloc.arena[a1..a1 + INLINE].copy_from_slice(&inline16[c1]);
                    a1 += usize::from(l1);
                } else {
                    emit_slow_lane!(p1, lseg[p1], c1, l1, a1, e1);
                }
                steps_done += 2;
            }
            if budget_exceeded {
                break;
            }
            let mut next_act = [0usize; 4];
            let mut new_n = 0;
            for &lane in &act[..n_act] {
                if (edges[lj[lane] as usize] as i32) < 0 {
                    if handle_hit!(lane) {
                        next_act[new_n] = lane;
                        new_n += 1;
                    }
                } else {
                    next_act[new_n] = lane;
                    new_n += 1;
                }
            }
            act = next_act;
            n_act = new_n;
        }

        while !budget_exceeded && n_act == 1 {
            let p0 = act[0];
            let mut j0 = lj[p0];
            let mut a0 = lat[p0];
            let mut e0 = lend[p0];
            loop {
                let ed0 = edges[j0 as usize];
                if (ed0 as i32) < 0 {
                    lj[p0] = j0;
                    lat[p0] = a0;
                    lend[p0] = e0;
                    break;
                }
                if steps_done + 1 > m {
                    budget_exceeded = true;
                    break;
                }
                j0 = ed0 & mask;
                let c0 = (ed0 >> bits) as usize;
                let l0 = lens[c0];
                if l0 != LONG && a0 + INLINE <= e0 {
                    alloc.arena[a0..a0 + INLINE].copy_from_slice(&inline16[c0]);
                    a0 += usize::from(l0);
                } else {
                    emit_slow_lane!(p0, lseg[p0], c0, l0, a0, e0);
                }
                steps_done += 1;
            }
            if budget_exceeded {
                break;
            }
            if !handle_hit!(p0) {
                n_act = 0;
            }
        }

        // Untag anchor bits in O(S)
        for &a in anchors {
            edges[a as usize] &= !ANCHOR_TAG;
        }

        // Chain segments chronologically from anchor 0 (= mark) in O(S)
        let mut order = [0usize; MAX_ANCHORS];
        let mut chain_len = 0usize;
        let mut cycle_closed = false;
        if !budget_exceeded {
            let mut cur = 0usize;
            for _ in 0..s {
                order[chain_len] = cur;
                chain_len += 1;
                let end_row = seg_end_row[cur];
                let Some(nxt) = anchors.iter().position(|&a| a == end_row) else {
                    break;
                };
                cur = nxt;
                if cur == 0 {
                    cycle_closed = true;
                    break;
                }
            }
        }

        if !cycle_closed || chain_len == 0 {
            emit_fwd_1lane(edges, extra, spelling, inline16, lens, n, mark, bits, arena);
            let c = if FUSED_RLE {
                shipped_unrle_in_place(arena, rle_runs, rle_stage, k)?
            } else {
                0
            };
            scratch.ordered.clear();
            scratch.ordered.push((0, arena.len()));
            return Ok(c);
        }

        if FUSED_RLE {
            // Check cross-segment RLE1 straddles (0..2 across 16*233 boundaries) and repair if any
            for idx in 1..chain_len {
                let prev = order[idx - 1];
                let cur = order[idx];
                if check_boundary_straddle(
                    seg_rb[prev],
                    seg_rl[prev],
                    &seg_first4[cur],
                    seg_first4_len[cur],
                ) {
                    let (new_crc, new_len, new_rb, new_rl) = repair_segment_fused(
                        cur,
                        anchors[cur],
                        seg_end_row[cur],
                        edges,
                        bits,
                        mask,
                        inline16,
                        lens,
                        spelling,
                        seg_rb[prev],
                        seg_rl[prev],
                        &mut alloc,
                        &mut scratch.rle_spill,
                        k,
                    );
                    seg_crc[cur] = new_crc;
                    seg_out_len[cur] = new_len;
                    seg_rb[cur] = new_rb;
                    seg_rl[cur] = new_rl;
                }
            }
            if seg_rl[order[chain_len - 1]] == 4 {
                return Err(ErrorKind::RunUnfinished);
            }
            let mut c = seg_crc[order[0]];
            for &seg in &order[1..chain_len] {
                c = crc_combine.combine(c, seg_crc[seg], seg_out_len[seg]);
            }
            for &seg in &order[..chain_len] {
                let mut sid = alloc.seg_head[seg];
                while sid != u16::MAX {
                    let sl = alloc.slices[sid as usize];
                    scratch.ordered.push((sl.off as usize, sl.len as usize));
                    sid = sl.next;
                }
            }
            return Ok(!c);
        }

        // Unfused vectored output: collect ordered raw slices
        let mut raw_bytes = 0usize;
        for &seg in &order[..chain_len] {
            let mut sid = alloc.seg_head[seg];
            while sid != u16::MAX {
                let sl = alloc.slices[sid as usize];
                scratch.ordered.push((sl.off as usize, sl.len as usize));
                raw_bytes += sl.len as usize;
                sid = sl.next;
            }
        }
        if raw_bytes != n {
            emit_fwd_1lane(edges, extra, spelling, inline16, lens, n, mark, bits, arena);
            scratch.ordered.clear();
            scratch.ordered.push((0, n));
        }
        Ok(0)
    }

    /// Unfused post-emission vectored RLE1+CRC32 pass over ordered raw slices in `buf`
    /// (for `W3b_plus_fwd4_vec_unfused` and `I3_only_fwd4_vec`): reads slices from L3
    /// after A7 finishes, compacting each slice in-place and combining CRCs.
    #[inline(never)]
    fn unfused_vectored_rle1_crc(
        buf: &mut Vec<u8>,
        ordered: &mut [(usize, usize)],
        rle_spill: &mut Vec<u8>,
        k: Kernels,
    ) -> Result<u32, ErrorKind> {
        let mut rb = 0u8;
        let mut rl = 0u8;
        let mut crc_st = !0u32;
        let mut write_top = buf.len();

        for slot in ordered.iter_mut() {
            let (off, len) = *slot;
            if len == 0 {
                continue;
            }
            let raw_end = off + len;
            let raw = &buf[off..raw_end];
            let boundary_ok =
                rl != 4 && (rl == 0 || raw[0] != rb || (usize::from(rl) + leading_eq(raw, rb)) < 4);
            if boundary_ok {
                match find_next_run(raw, k) {
                    None => {
                        let (last, tr) = trailing_run_nonempty(raw);
                        if usize::from(tr) == len && rl > 0 && last == rb {
                            rl += tr;
                        } else {
                            rb = last;
                            rl = tr;
                        }
                        crc_st = crc::update_with(crc_st, raw, k);
                        continue;
                    }
                    Some(first_rel) => {
                        let mut src = off + first_rel;
                        let mut dst = src;
                        let mut spilled = false;
                        loop {
                            let c = buf[src];
                            if src + 4 < raw_end {
                                let extra = buf[src + 4] as usize;
                                let out_run = 4 + extra;
                                let next_src = src + 5;
                                if dst + out_run <= next_src {
                                    buf[dst..dst + out_run].fill(c);
                                    dst += out_run;
                                    src = next_src;
                                } else {
                                    rle_spill.clear();
                                    rle_spill.extend_from_slice(&buf[src..raw_end]);
                                    rl = 0;
                                    spilled = true;
                                    break;
                                }
                            } else {
                                buf[dst..dst + 4].fill(c);
                                dst += 4;
                                rb = c;
                                rl = 4;
                                break;
                            }
                            if src >= raw_end {
                                rl = 0;
                                break;
                            }
                            match find_next_run(&buf[src..raw_end], k) {
                                None => {
                                    let rem = raw_end - src;
                                    let (last, tr) = trailing_run_nonempty(&buf[src..raw_end]);
                                    if dst != src {
                                        buf.copy_within(src..raw_end, dst);
                                    }
                                    dst += rem;
                                    rb = last;
                                    rl = tr;
                                    break;
                                }
                                Some(r2) => {
                                    if r2 > 0 {
                                        if dst != src {
                                            buf.copy_within(src..src + r2, dst);
                                        }
                                        dst += r2;
                                        src += r2;
                                    }
                                }
                            }
                        }
                        if !spilled {
                            let new_len = dst - off;
                            *slot = (off, new_len);
                            crc_st = crc::update_with(crc_st, &buf[off..dst], k);
                            continue;
                        }
                        // Spilled suffix: expand out-of-place at `write_top`
                        let prefix_len = dst - off;
                        let new_off = write_top;
                        let max_grow = prefix_len + rle_spill.len() * 64 + 256;
                        if buf.len() < new_off + max_grow {
                            buf.resize(new_off + max_grow, 0);
                        }
                        buf.copy_within(off..dst, new_off);
                        let mut out_at = new_off + prefix_len;
                        let mut pos = 0usize;
                        let slen = rle_spill.len();
                        while pos < slen {
                            match find_next_run(&rle_spill[pos..], k) {
                                None => {
                                    let rem = slen - pos;
                                    if out_at + rem > buf.len() {
                                        buf.resize(out_at + rem + 4096, 0);
                                    }
                                    buf[out_at..out_at + rem].copy_from_slice(&rle_spill[pos..]);
                                    out_at += rem;
                                    let (last, tr) = trailing_run_nonempty(&rle_spill[pos..]);
                                    rb = last;
                                    rl = tr;
                                    break;
                                }
                                Some(rel) => {
                                    if rel > 0 {
                                        if out_at + rel > buf.len() {
                                            buf.resize(out_at + rel + 4096, 0);
                                        }
                                        buf[out_at..out_at + rel]
                                            .copy_from_slice(&rle_spill[pos..pos + rel]);
                                        out_at += rel;
                                    }
                                    let j = pos + rel;
                                    let c = rle_spill[j];
                                    if j + 4 < slen {
                                        let total = 4 + rle_spill[j + 4] as usize;
                                        if out_at + total > buf.len() {
                                            buf.resize(out_at + total + 4096, 0);
                                        }
                                        buf[out_at..out_at + total].fill(c);
                                        out_at += total;
                                        pos = j + 5;
                                    } else {
                                        if out_at + 4 > buf.len() {
                                            buf.resize(out_at + 4096, 0);
                                        }
                                        buf[out_at..out_at + 4].fill(c);
                                        out_at += 4;
                                        rb = c;
                                        rl = 4;
                                        break;
                                    }
                                }
                            }
                        }
                        *slot = (new_off, out_at - new_off);
                        crc_st = crc::update_with(crc_st, &buf[new_off..out_at], k);
                        write_top = out_at;
                        continue;
                    }
                }
            }

            // Boundary straddle between slices: expand whole slice cleanly at `write_top`
            rle_spill.clear();
            rle_spill.extend_from_slice(&buf[off..raw_end]);
            let new_off = write_top;
            let mut out_at = new_off;
            let mut pos = 0usize;
            let slen = rle_spill.len();
            macro_rules! grow_out {
                ($need:expr) => {
                    if out_at + $need > buf.len() {
                        let want = out_at + $need + 16384;
                        if want > buf.capacity() {
                            buf.reserve_exact(want - buf.len());
                        }
                        buf.resize(want, 0);
                    }
                };
            }
            if rl == 4 {
                let extra = rle_spill[0] as usize;
                if extra > 0 {
                    grow_out!(extra);
                    buf[out_at..out_at + extra].fill(rb);
                    out_at += extra;
                }
                rl = 0;
                pos = 1;
            }
            while pos < slen && rl > 0 {
                let b = rle_spill[pos];
                if b == rb {
                    grow_out!(1);
                    buf[out_at] = b;
                    out_at += 1;
                    rl += 1;
                    pos += 1;
                    if rl == 4 {
                        if pos < slen {
                            let extra = rle_spill[pos] as usize;
                            if extra > 0 {
                                grow_out!(extra);
                                buf[out_at..out_at + extra].fill(rb);
                                out_at += extra;
                            }
                            rl = 0;
                            pos += 1;
                        }
                        break;
                    }
                } else {
                    rl = 0;
                    break;
                }
            }
            while pos < slen {
                match find_next_run(&rle_spill[pos..], k) {
                    None => {
                        let rem = slen - pos;
                        grow_out!(rem);
                        buf[out_at..out_at + rem].copy_from_slice(&rle_spill[pos..]);
                        out_at += rem;
                        let (last, tr) = trailing_run_nonempty(&rle_spill[pos..]);
                        rb = last;
                        rl = tr;
                        break;
                    }
                    Some(rel) => {
                        if rel > 0 {
                            grow_out!(rel);
                            buf[out_at..out_at + rel].copy_from_slice(&rle_spill[pos..pos + rel]);
                            out_at += rel;
                        }
                        let j = pos + rel;
                        let c = rle_spill[j];
                        if j + 4 < slen {
                            let total = 4 + rle_spill[j + 4] as usize;
                            grow_out!(total);
                            buf[out_at..out_at + total].fill(c);
                            out_at += total;
                            pos = j + 5;
                        } else {
                            grow_out!(4);
                            buf[out_at..out_at + 4].fill(c);
                            out_at += 4;
                            rb = c;
                            rl = 4;
                            break;
                        }
                    }
                }
            }
            *slot = (new_off, out_at - new_off);
            crc_st = crc::update_with(crc_st, &buf[new_off..out_at], k);
            write_top = out_at;
        }

        if rl == 4 {
            return Err(ErrorKind::RunUnfinished);
        }
        Ok(!crc_st)
    }

    // -----------------------------------------------------------------------
    // Full WorkerState & end-to-end decoder executing any composed arm
    // -----------------------------------------------------------------------

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Arm {
        W0Shipped,
        W3bV1Unfused,
        W4V1_32kInline,
        W4AHoist496,
        W4B32kCold,
        W4B64kCold,
        W4V2Q496_32k,
        W4V2Q1008_32k,
        W4V2Q1008_64k,
        W3bV2Q1008_32kUnf,
    }

    impl Arm {
        fn name(self) -> &'static str {
            match self {
                Arm::W0Shipped => "W0_shipped",
                Arm::W3bV1Unfused => "W3b_v1_unfused",
                Arm::W4V1_32kInline => "W4_v1_32k_inline",
                Arm::W4AHoist496 => "W4_a_hoist496",
                Arm::W4B32kCold => "W4_b_32k_cold",
                Arm::W4B64kCold => "W4_b_64k_cold",
                Arm::W4V2Q496_32k => "W4_v2_q496_32k",
                Arm::W4V2Q1008_32k => "W4_v2_q1008_32k",
                Arm::W4V2Q1008_64k => "W4_v2_q1008_64k",
                Arm::W3bV2Q1008_32kUnf => "W3b_v2_q1008_32k_unf",
            }
        }
        fn use_hoisted_mtf(self) -> bool {
            matches!(
                self,
                Arm::W4AHoist496
                    | Arm::W4V2Q496_32k
                    | Arm::W4V2Q1008_32k
                    | Arm::W4V2Q1008_64k
                    | Arm::W3bV2Q1008_32kUnf
            )
        }
        fn use_l1_mtf(self) -> bool {
            !matches!(self, Arm::W0Shipped)
        }
        fn use_q1008(self) -> bool {
            matches!(
                self,
                Arm::W4V2Q1008_32k | Arm::W4V2Q1008_64k | Arm::W3bV2Q1008_32kUnf
            )
        }
        fn use_a6b_redesign(self) -> bool {
            !matches!(self, Arm::W0Shipped)
        }
        fn use_fwd4(self) -> bool {
            !matches!(self, Arm::W0Shipped)
        }
        fn use_fused_rle(self) -> bool {
            matches!(
                self,
                Arm::W4V1_32kInline
                    | Arm::W4AHoist496
                    | Arm::W4B32kCold
                    | Arm::W4B64kCold
                    | Arm::W4V2Q496_32k
                    | Arm::W4V2Q1008_32k
                    | Arm::W4V2Q1008_64k
            )
        }
        fn use_slab_64k(self) -> bool {
            matches!(self, Arm::W4B64kCold | Arm::W4V2Q1008_64k)
        }
        fn use_cold_compact(self) -> bool {
            matches!(
                self,
                Arm::W4B32kCold
                    | Arm::W4B64kCold
                    | Arm::W4V2Q496_32k
                    | Arm::W4V2Q1008_32k
                    | Arm::W4V2Q1008_64k
            )
        }
    }

    const ARMS: &[Arm] = &[
        Arm::W0Shipped,
        Arm::W3bV1Unfused,
        Arm::W4V1_32kInline,
        Arm::W4AHoist496,
        Arm::W4B32kCold,
        Arm::W4B64kCold,
        Arm::W4V2Q496_32k,
        Arm::W4V2Q1008_32k,
        Arm::W4V2Q1008_64k,
        Arm::W3bV2Q1008_32kUnf,
    ];

    #[derive(Default)]
    struct WorkerState {
        front: FrontScratch,
        spill512: Spill<512>,
        spill1024: Spill<1024>,
        stack: Vec<u32>,
        bands: Vec<Band>,
        spelling: Vec<u8>,
        edges: Vec<u32>,
        extra: Vec<u32>,
        inline16: Vec<[u8; INLINE]>,
        lens: Vec<u8>,
        a6b: A6bState,
        sampled: SampledScratch,
        rle_runs: Vec<Run>,
        rle_stage: Vec<u8>,
        out: Vec<u8>,
    }

    impl WorkerState {
        fn working_set_bytes(&self, arm: Arm) -> (usize, usize) {
            let spill_bytes = if arm.use_q1008() {
                self.spill1024.scratch_bytes()
            } else {
                self.spill512.scratch_bytes()
            };
            let shared = self.front.scratch_bytes()
                + spill_bytes
                + self.stack.capacity() * 4
                + self.bands.capacity() * 8
                + self.spelling.capacity()
                + self.edges.capacity() * 4
                + self.extra.capacity() * 4
                + self.inline16.capacity() * INLINE
                + self.lens.capacity()
                + self.out.capacity();
            let rle_extra = if arm.use_fwd4() {
                self.sampled.scratch_bytes()
            } else {
                self.rle_runs.capacity() * std::mem::size_of::<Run>() + self.rle_stage.capacity()
            };
            let a6b_virt = if arm.use_a6b_redesign() {
                self.a6b.scratch_bytes_virtual()
            } else {
                0
            };
            // On wikidata Open blocks, Fixed64kSoa touches avg ~9,720 / max ~15,000 nodes (~420 KiB resident)
            let a6b_res = if arm.use_a6b_redesign() {
                420 * 1024
                    + self.a6b.image_order64.capacity() * 8
                    + self.a6b.rules.capacity() * 16
                    + 24 * 1024
            } else {
                0
            };
            (shared + rle_extra + a6b_res, shared + rle_extra + a6b_virt)
        }

        fn induced_shipped<L: Links>(
            &mut self,
            ns: usize,
            n: u32,
            thin: &[u32; 256],
            m: u32,
            most: usize,
        ) -> [usize; 256] {
            let (hdr, arena) = self.edges.split_at_mut(4 * ns);
            let (strips_raw, image_order) = hdr.split_at_mut(3 * ns);
            let mut e = Elim::<L> {
                a: arena,
                base: 0,
                top: 0,
                nodes: 0,
                _links: std::marker::PhantomData,
            };
            e.node(0, 0);
            let mut widest = 0;
            for i in 0..ns {
                let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
                e.node(s.width, literal(s.byte));
                widest = widest.max(s.width);
            }
            let (mut terminals, mut order) = ([(0, 0); 2], [(0, 0); 2]);
            let mut x = 0;
            for i in 0..ns {
                let s = Strip::unpack(&strips_raw[3 * i..3 * i + 3]);
                if x < s.from {
                    widest = widest.max(s.from - x);
                    let id = e.node(s.from - x, 0);
                    e.append_to(&mut order[0], id, 0);
                    e.append_to(&mut terminals[1], id, 1);
                }
                e.append_to(&mut order[0], i as u32 + 1, 0);
                x = s.from + s.width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[0], id, 0);
                e.append_to(&mut terminals[1], id, 1);
            }
            x = 0;
            for &i in image_order.iter() {
                let idx = i as usize;
                let s = Strip::unpack(&strips_raw[3 * idx..3 * idx + 3]);
                if x < s.to {
                    widest = widest.max(s.to - x);
                    let id = e.node(s.to - x, 0);
                    e.append_to(&mut order[1], id, 1);
                    e.append_to(&mut terminals[0], id, 0);
                }
                e.append_to(&mut order[1], i + 1, 1);
                x = s.to + s.width;
            }
            if x < n {
                widest = widest.max(n - x);
                let id = e.node(n - x, 0);
                e.append_to(&mut order[1], id, 1);
                e.append_to(&mut terminals[0], id, 0);
            }
            for side in 0..2 {
                e.close(side, terminals[side], order[side]);
            }
            e.base = L::STRIDE * e.nodes as usize;
            e.top = e.base;
            let mut remaining = n;
            let spirals = widest > CERTIFIED_WIDTH;
            while !if spirals {
                e.eliminate::<true>(&mut remaining)
            } else {
                e.eliminate::<false>(&mut remaining)
            } {
                let (base, top, nodes) = (e.base, e.top, e.nodes);
                grow(&mut self.edges, 4 * ns + top + RULE);
                e = Elim {
                    a: &mut self.edges[4 * ns..],
                    base,
                    top,
                    nodes,
                    _links: std::marker::PhantomData,
                };
            }
            assert!(e.nodes as usize <= most);
            e.assign_bottom(m);
            const PAIR_WORDS: usize = 2 * 1024;
            const CACHE_WORDS: usize = PAIR_WORDS + 4096;
            let (base, top, nodes) = (e.base, e.top, e.nodes);
            let (header, arena) = self.edges.split_at_mut(4 * ns);
            let (live, tail) = arena.split_at_mut(top);
            let cache = if header.len() >= CACHE_WORDS {
                header
            } else {
                tail
            };
            let (pairs, contents) = if cache.len() >= CACHE_WORDS {
                cache[..CACHE_WORDS].fill(0);
                cache[..CACHE_WORDS].split_at_mut(PAIR_WORDS)
            } else {
                (&mut [][..], &mut [][..])
            };
            let mut e = Elim::<L> {
                a: live,
                base,
                top,
                nodes,
                _links: std::marker::PhantomData,
            };
            begin_inline(
                if n >= 65_536 { 1536 } else { 0 },
                &mut self.inline16,
                &mut self.lens,
            );
            let mut dictionary = Dictionary {
                pairs,
                contents,
                inline16: &mut self.inline16,
                lens: &mut self.lens,
                spelling: &mut self.spelling,
                stack: &mut self.stack,
            };
            let mut cursors = [0; 256];
            let mut id = L::next(e.a, 0, 0);
            let mut in_node = 0;
            let mut k = 0;
            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let node_width = e.width(id);
                    let node_word = e.word(id);
                    let width = (end - k).min(node_width - in_node);
                    let dst = e.bottom(id) + in_node;
                    let code = dictionary.code(&mut e, node_word, a as u8);
                    self.bands.push(Band::new(k + width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = L::next(e.a, id, 0);
                        in_node = 0;
                    }
                }
            }
            assert_eq!(id, 0);
            cursors
        }

        fn induced_64k_soa(
            &mut self,
            ns: usize,
            n: u32,
            thin: &[u32; 256],
            m: u32,
            first: &[u32; 256],
            write: &mut [u32; 256],
        ) -> [usize; 256] {
            let strips_raw = &self.edges[..3 * ns];
            let soa = self.a6b.fixed_soa.get_or_insert_with(Fixed64kSoa::new);
            let widest = setup_soa_64k(
                strips_raw,
                ns,
                n,
                first,
                write,
                &mut self.a6b.image_order64,
                soa,
            );
            if widest > CERTIFIED_WIDTH {
                eliminate_inline15_soa_64k::<true>(soa, &mut self.a6b.rules, n);
            } else {
                eliminate_inline15_soa_64k::<false>(soa, &mut self.a6b.rules, n);
            }
            assign_bottom_soa_64k(&mut soa.links, &soa.widths, m);
            self.a6b.gen_cache.next_block();
            begin_inline(
                if n >= 65_536 { 1536 } else { 0 },
                &mut self.inline16,
                &mut self.lens,
            );
            let mut cursors = [0usize; 256];
            let links = &*soa.links;
            let widths = &*soa.widths;
            let words = &*soa.words128;
            let rules = &mut self.a6b.rules[..];

            let mut id = links[0].next0;
            let mut node_width = widths[usize::from(id)];
            let mut node_word = words[usize::from(id)];
            let l0 = links[usize::from(id)];
            let mut node_bottom = u32::from(l0.prev1) | (u32::from(l0.next1) << 16);
            let mut node_next = l0.next0;
            let mut in_node = 0u32;
            let mut k = 0u32;

            for a in 0..256 {
                cursors[a] = self.bands.len();
                let end = k + thin[a];
                while k < end {
                    let width = (end - k).min(node_width - in_node);
                    let dst = node_bottom + in_node;
                    let code = if node_word == 0 {
                        a as u32
                    } else {
                        dict_code_u128_gen(
                            node_word,
                            a as u8,
                            rules,
                            &mut self.a6b.gen_cache,
                            &mut self.inline16,
                            &mut self.lens,
                            &mut self.spelling,
                            &mut self.stack,
                        )
                    };
                    self.bands.push(Band::new(k + width, dst, code));
                    in_node += width;
                    k += width;
                    if in_node == node_width {
                        id = node_next;
                        in_node = 0;
                        if id != 0 {
                            let l = links[usize::from(id)];
                            node_width = widths[usize::from(id)];
                            node_word = words[usize::from(id)];
                            node_bottom = u32::from(l.prev1) | (u32::from(l.next1) << 16);
                            node_next = l.next0;
                        }
                    }
                }
            }
            assert_eq!(id, 0);
            cursors
        }

        #[inline(never)]
        fn decode_block_arm(
            &mut self,
            b: &Symbols,
            arm: Arm,
            crc_combine: &CrcCombine,
            k: Kernels,
            ph: &mut PhaseWallNs,
        ) -> u32 {
            let t0 = Instant::now();
            // --- A5 + A6a ---
            let bound = b.max_len;
            let room = bound + bound / 16 + 64;
            if self.out.capacity() < room {
                self.out.reserve_exact(room - self.out.len());
            }
            let (n, f, heads, ns) = if arm.use_hoisted_mtf() {
                self.out.resize(bound + INLINE, 0);
                if arm.use_q1008() {
                    run_l1_batch_hoisted::<256, 1008, 1024>(
                        b,
                        &mut self.front,
                        &mut self.out[..],
                        &mut self.spill1024,
                        &mut self.edges,
                    )
                } else {
                    run_l1_batch_hoisted::<256, 496, 512>(
                        b,
                        &mut self.front,
                        &mut self.out[..],
                        &mut self.spill512,
                        &mut self.edges,
                    )
                }
            } else if arm.use_l1_mtf() {
                self.out.resize(bound + INLINE, 0);
                run_l1_batch::<512, 496, 512>(
                    b,
                    &mut self.front,
                    &mut self.out[..],
                    &mut self.spill512,
                    &mut self.edges,
                )
            } else {
                let n =
                    unsafe { probe_front_avx2_shipped(b, &mut self.front).expect("shipped front") };
                self.out.resize(n + INLINE, 0);
                let mut pass = Pass::<512, 512>::new(
                    &mut self.out[..],
                    &mut self.spill512,
                    &mut self.edges,
                    u32::MAX,
                    usize::MAX,
                );
                let p = b.orig_ptr;
                maximal(&self.front.runs, |c, pos, width| {
                    pass.take(c, pos, width, p, LIMIT);
                });
                let heads = pass.queues.heads();
                (n, pass.f, heads, pass.ns)
            };
            let t1 = Instant::now();

            // --- A6b ---
            let all: [u32; 256] = std::array::from_fn(|a| f.thin[a] + f.thick_rows[a]);
            let first = starts(all);
            let mut write = starts(f.thick);
            let most = 3 * ns + 3;
            let wide_links = most > NARROW_NODES;
            self.bands.clear();
            self.spelling.clear();
            if n >= 65_536 {
                self.spelling.reserve_exact(48 * 1024);
            }
            let cursors = if arm.use_a6b_redesign() && !wide_links {
                self.induced_64k_soa(ns, n as u32, &f.thin, f.m, &first, &mut write)
            } else {
                let stride = if wide_links {
                    Wide::STRIDE
                } else {
                    Narrow::STRIDE
                };
                let need = (4 * ns + (stride + RULE) * most).max(f.m as usize);
                if self.edges.len() < need {
                    let add = need - self.edges.len();
                    self.edges.reserve_exact(add);
                    self.edges.resize(need, 0);
                }
                {
                    let (strips_raw, rest_edges) = self.edges.split_at_mut(3 * ns);
                    let image_order = &mut rest_edges[..ns];
                    for i in 0..ns {
                        let sw = &mut strips_raw[3 * i..3 * i + 3];
                        let byte = (sw[2] & 0xFF) as u8;
                        let a = usize::from(byte);
                        sw[1] += first[a];
                        image_order[write[a] as usize] = i as u32;
                        write[a] += 1;
                    }
                }
                if wide_links {
                    self.induced_shipped::<Wide>(ns, n as u32, &f.thin, f.m, most)
                } else {
                    self.induced_shipped::<Narrow>(ns, n as u32, &f.thin, f.m, most)
                }
            };
            let t2 = Instant::now();

            // --- A6c ---
            let m = f.m as usize;
            let bits = (u32::BITS - (m.max(1) as u32 - 1).leading_zeros()).max(1);
            let wide_edges = self.lens.len() - 1 > (u32::MAX >> bits) as usize;
            if self.edges.len() < m {
                self.edges.resize(m, 0);
            }
            self.extra.clear();
            if wide_edges {
                self.extra.resize(m, 0);
            }
            let mut predecessor = None;
            let mut anchors = [0u32; MAX_ANCHORS];
            let mut anchor_cnt = 0usize;

            if arm.use_fwd4() {
                let mut kk = 0u32;
                if arm.use_q1008() {
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut q =
                            QueueReader::<1008, 1024>::new(&self.out, &self.spill1024, &heads, a);
                        for &band in &self.bands[cursors[a]..end] {
                            let bend = band.end();
                            let span = (bend - kk) as usize;
                            let dst = band.dst() as usize;
                            let bcode = band.code();
                            let pack = if wide_edges { 0 } else { bcode << bits };
                            if wide_edges {
                                self.extra[dst..dst + span].fill(bcode);
                            }
                            q.wire(&mut self.edges[dst..dst + span], pack);
                            kk = bend;
                        }
                    }
                } else {
                    for a in 0..256 {
                        let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                        let mut q =
                            QueueReader::<496, 512>::new(&self.out, &self.spill512, &heads, a);
                        for &band in &self.bands[cursors[a]..end] {
                            let bend = band.end();
                            let span = (bend - kk) as usize;
                            let dst = band.dst() as usize;
                            let bcode = band.code();
                            let pack = if wide_edges { 0 } else { bcode << bits };
                            if wide_edges {
                                self.extra[dst..dst + span].fill(bcode);
                            }
                            q.wire(&mut self.edges[dst..dst + span], pack);
                            kk = bend;
                        }
                    }
                }
                anchor_cnt = tag_uniform_anchors(
                    &mut self.edges[..m],
                    m,
                    f.mark,
                    bits,
                    wide_edges,
                    self.lens.len(),
                    &mut anchors,
                );
            } else if !wide_edges && m >= 65_536 {
                let edges = &mut self.edges[..m];
                edges.fill(0);
                let mut pred = f.mark;
                let mut kk = 0u32;
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut q = QueueReader::<512, 512>::new(&self.out, &self.spill512, &heads, a);
                    for &band in &self.bands[cursors[a]..end] {
                        q.wire_xor(
                            edges,
                            band.dst() as usize,
                            (band.end() - kk) as usize,
                            band.code() << bits,
                            f.mark,
                            &mut pred,
                        );
                        kk = band.end();
                    }
                }
                predecessor = Some(pred);
            } else {
                let mut kk = 0u32;
                for a in 0..256 {
                    let end = cursors.get(a + 1).copied().unwrap_or(self.bands.len());
                    let mut q = QueueReader::<512, 512>::new(&self.out, &self.spill512, &heads, a);
                    for &band in &self.bands[cursors[a]..end] {
                        let bend = band.end();
                        let span = (bend - kk) as usize;
                        let dst = band.dst() as usize;
                        let bcode = band.code();
                        let pack = if wide_edges { 0 } else { bcode << bits };
                        if wide_edges {
                            self.extra[dst..dst + span].fill(bcode);
                        }
                        q.wire(&mut self.edges[dst..dst + span], pack);
                        kk = bend;
                    }
                }
            }
            let t3 = Instant::now();

            // --- A7 + A9 ---
            let crc_out = if arm.use_fused_rle() {
                match (arm.use_slab_64k(), arm.use_cold_compact()) {
                    (false, false) => emit_4lane_sampled::<true, SLAB_32K, false>(
                        &mut self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        bits,
                        &anchors[..anchor_cnt],
                        &mut self.out,
                        &mut self.sampled,
                        crc_combine,
                        k,
                        &mut self.rle_runs,
                        &mut self.rle_stage,
                    )
                    .expect("fused 4lane 32k inline"),
                    (false, true) => emit_4lane_sampled::<true, SLAB_32K, true>(
                        &mut self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        bits,
                        &anchors[..anchor_cnt],
                        &mut self.out,
                        &mut self.sampled,
                        crc_combine,
                        k,
                        &mut self.rle_runs,
                        &mut self.rle_stage,
                    )
                    .expect("fused 4lane 32k cold"),
                    (true, true) => emit_4lane_sampled::<true, SLAB_64K, true>(
                        &mut self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        bits,
                        &anchors[..anchor_cnt],
                        &mut self.out,
                        &mut self.sampled,
                        crc_combine,
                        k,
                        &mut self.rle_runs,
                        &mut self.rle_stage,
                    )
                    .expect("fused 4lane 64k cold"),
                    (true, false) => emit_4lane_sampled::<true, SLAB_64K, false>(
                        &mut self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        bits,
                        &anchors[..anchor_cnt],
                        &mut self.out,
                        &mut self.sampled,
                        crc_combine,
                        k,
                        &mut self.rle_runs,
                        &mut self.rle_stage,
                    )
                    .expect("fused 4lane 64k inline"),
                }
            } else if arm.use_fwd4() {
                if arm.use_slab_64k() {
                    emit_4lane_sampled::<false, SLAB_64K, false>(
                        &mut self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        bits,
                        &anchors[..anchor_cnt],
                        &mut self.out,
                        &mut self.sampled,
                        crc_combine,
                        k,
                        &mut self.rle_runs,
                        &mut self.rle_stage,
                    )
                    .expect("raw 4lane 64k");
                } else {
                    emit_4lane_sampled::<false, SLAB_32K, false>(
                        &mut self.edges[..m],
                        &self.extra,
                        &self.spelling,
                        &self.inline16,
                        &self.lens,
                        n,
                        m,
                        f.mark,
                        bits,
                        &anchors[..anchor_cnt],
                        &mut self.out,
                        &mut self.sampled,
                        crc_combine,
                        k,
                        &mut self.rle_runs,
                        &mut self.rle_stage,
                    )
                    .expect("raw 4lane 32k");
                }
                unfused_vectored_rle1_crc(
                    &mut self.out,
                    &mut self.sampled.ordered,
                    &mut self.sampled.rle_spill,
                    k,
                )
                .expect("unfused vec rle+crc")
            } else {
                emit_bidi(
                    &self.edges[..m],
                    &self.extra,
                    &self.spelling,
                    &self.inline16,
                    &self.lens,
                    n,
                    f.mark,
                    predecessor,
                    bits,
                    &mut self.out,
                );
                shipped_unrle_in_place(&mut self.out, &mut self.rle_runs, &mut self.rle_stage, k)
                    .expect("unrle_in_place")
            };
            let t4 = Instant::now();

            ph.a5_a6a += t1.duration_since(t0).as_nanos() as u64;
            ph.a6b += t2.duration_since(t1).as_nanos() as u64;
            ph.a6c += t3.duration_since(t2).as_nanos() as u64;
            ph.a7_a9 += t4.duration_since(t3).as_nanos() as u64;
            crc_out
        }

        fn extract_output(&self, arm: Arm, dst: &mut Vec<u8>) {
            dst.clear();
            if arm.use_fwd4() {
                for &(off, len) in &self.sampled.ordered {
                    dst.extend_from_slice(&self.out[off..off + len]);
                }
            } else {
                dst.extend_from_slice(&self.out);
            }
        }
    }

    #[derive(Default, Clone, Copy)]
    struct PhaseWallNs {
        a5_a6a: u64,
        a6b: u64,
        a6c: u64,
        a7_a9: u64,
    }

    #[derive(Default, Clone, Copy)]
    struct PassResult {
        cpu_ns: u64,
        wall_ns: u64,
        ph: PhaseWallNs,
    }

    // -----------------------------------------------------------------------
    // Corpus loading & exact verification against `mibz2::decode_block`
    // -----------------------------------------------------------------------

    struct AdmittedBlock {
        syms: Symbols,
        expected_bytes: Vec<u8>,
        expected_crc: u32,
    }

    fn load_admitted_blocks(path: &str, k: Kernels) -> Vec<AdmittedBlock> {
        let data = std::fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
        let mut sc = Scanner::new(SliceSource(&data));
        let mut syms = Vec::new();
        let mut scratch: Scratch = Scratch::default();
        let mut front_tmp = FrontScratch::default();
        let mut out_tmp = Vec::new();
        let mut admitted = Vec::new();
        let mut total = 0usize;

        while let Event::Block { symbols: b, .. } = sc
            .next(std::mem::take(&mut syms))
            .unwrap_or_else(|e| panic!("scan error: {e:?}"))
        {
            total += 1;
            let n = unsafe { probe_front_avx2_shipped(&b, &mut front_tmp).expect("front") };
            let is_admitted = match open_cap(front_tmp.runs.len(), n, b.orig_ptr) {
                Some(Cap::NONE) => true,
                Some(cap) => within_cap(&front_tmp.runs, b.orig_ptr, LIMIT, cap),
                None => false,
            };
            if is_admitted {
                out_tmp.clear();
                let crc_want = decode_block(&b, k, &mut scratch, &mut out_tmp)
                    .unwrap_or_else(|e| panic!("decode_block failed: {e:?}"));
                admitted.push(AdmittedBlock {
                    syms: b,
                    expected_bytes: out_tmp.clone(),
                    expected_crc: crc_want,
                });
            } else {
                syms = b.syms;
            }
        }
        println!(
            "Loaded {total} total blocks from {path}; {} production-admitted Open blocks ({} decoded bytes).",
            admitted.len(),
            admitted.iter().map(|b| b.expected_bytes.len()).sum::<usize>()
        );
        admitted
    }

    fn flush_l3(buf: &mut [u8]) {
        let mut acc = 0u8;
        for b in buf.iter_mut().step_by(64) {
            *b = b.wrapping_add(1);
            acc ^= *b;
        }
        black_box(acc);
    }

    fn median_min_max(vals: &[f64]) -> (f64, f64, f64) {
        let mut s = vals.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        (s[s.len() / 2], s[0], s[s.len() - 1])
    }

    fn print_results(label: &str, all_res: &[Vec<PassResult>], use_cpu: bool) {
        println!("\n=== {label} ===");
        println!(
            "{:<26} {:>7} {:>7} {:>7} {:>7} {:>24} {:>8} {:>9} {:>9}",
            "arm",
            "A5+6a",
            "A6b",
            "A6c",
            "A7+A9",
            "total med [min..max]",
            "vs_W0",
            "disj_W0",
            "i9_W'/8"
        );

        let totals: Vec<Vec<f64>> = all_res
            .iter()
            .map(|v| {
                v.iter()
                    .map(|r| (if use_cpu { r.cpu_ns } else { r.wall_ns }) as f64 / 1e6)
                    .collect()
            })
            .collect();
        let (w0_med, w0_min, w0_max) = median_min_max(&totals[0]);

        for (idx, &arm) in ARMS.iter().enumerate() {
            let (med, mn, mx) = median_min_max(&totals[idx]);
            // Scale wall sub-phases proportionally to `med` so phase columns sum cleanly to `med`
            let ph_sums: Vec<f64> = all_res[idx]
                .iter()
                .map(|r| (r.ph.a5_a6a + r.ph.a6b + r.ph.a6c + r.ph.a7_a9) as f64)
                .collect();
            let p_a56a = median_min_max(
                &all_res[idx]
                    .iter()
                    .zip(&ph_sums)
                    .map(|(r, &s)| med * (r.ph.a5_a6a as f64 / s))
                    .collect::<Vec<_>>(),
            )
            .0;
            let p_a6b = median_min_max(
                &all_res[idx]
                    .iter()
                    .zip(&ph_sums)
                    .map(|(r, &s)| med * (r.ph.a6b as f64 / s))
                    .collect::<Vec<_>>(),
            )
            .0;
            let p_a6c = median_min_max(
                &all_res[idx]
                    .iter()
                    .zip(&ph_sums)
                    .map(|(r, &s)| med * (r.ph.a6c as f64 / s))
                    .collect::<Vec<_>>(),
            )
            .0;
            let p_a79 = median_min_max(
                &all_res[idx]
                    .iter()
                    .zip(&ph_sums)
                    .map(|(r, &s)| med * (r.ph.a7_a9 as f64 / s))
                    .collect::<Vec<_>>(),
            )
            .0;

            let ratio = med / w0_med;
            let delta_pct = (ratio - 1.0) * 100.0;
            let disj = if idx == 0 {
                "base"
            } else if mx < w0_min {
                "FASTER"
            } else if mn > w0_max {
                "SLOWER"
            } else {
                "overlap"
            };
            let w8_proj = I9_WORKER_8W_MS * ratio;
            println!(
                "{:<26} {:>7.2} {:>7.2} {:>7.2} {:>7.2} {:>7.2} [{:>6.2}..{:>6.2}] {:>+7.2}% {:>9} {:>7.2}ms",
                arm.name(),
                p_a56a,
                p_a6b,
                p_a6c,
                p_a79,
                med,
                mn,
                mx,
                delta_pct,
                disj,
                w8_proj
            );
        }

        // v2 upgrade decomposition & 6 GB/s contour check
        let med_of = |i: usize| median_min_max(&totals[i]).0;
        let m_w0 = med_of(0); // W0_shipped
        let m_w3b1 = med_of(1); // W3b_v1_unfused
        let m_w4v1 = med_of(2); // W4_v1_32k_inline
        let m_w4a = med_of(3); // W4_a_hoist496
        let m_w4b32 = med_of(4); // W4_b_32k_cold
        let m_w4b64 = med_of(5); // W4_b_64k_cold
        let m_w4v2_496_32 = med_of(6); // W4_v2_q496_32k
        let m_w4v2_1008_32 = med_of(7); // W4_v2_q1008_32k
        let m_w4v2_1008_64 = med_of(8); // W4_v2_q1008_64k
        let m_w3b2_1008_32 = med_of(9); // W3b_v2_q1008_32k_unf

        let gain_a_hoist496 = m_w4v1 - m_w4a;
        let gain_b_32k_cold = m_w4v1 - m_w4b32;
        let gain_b_64k_cold = m_w4v1 - m_w4b64;
        let gain_v2_496_32 = m_w4v1 - m_w4v2_496_32;
        let gain_v2_1008_32 = m_w4v1 - m_w4v2_1008_32;
        let gain_64k_vs_32k = m_w4v2_1008_32 - m_w4v2_1008_64;
        let fusion_v1 = m_w3b1 - m_w4v1;
        let fusion_v2_32k = m_w3b2_1008_32 - m_w4v2_1008_32;

        println!("\n--- v2 Upgrade Decomposition ({label}) ---");
        println!(
            "  vs W4_v1 ({:.2}ms):  +Upgr(a) hoist496={:>+6.2}ms  +32K_cold={:>+6.2}ms  +64K_cold={:>+6.2}ms",
            m_w4v1, gain_a_hoist496, gain_b_32k_cold, gain_b_64k_cold
        );
        println!(
            "  Combined vs W4_v1:   W4_v2_q496_32k={:>+6.2}ms ({:>+5.2}% W0) | W4_v2_q1008_32k={:>+6.2}ms ({:>+5.2}% W0) | 64K vs 32K (q1008)={:>+6.2}ms",
            gain_v2_496_32,
            100.0 * gain_v2_496_32 / m_w0,
            gain_v2_1008_32,
            100.0 * gain_v2_1008_32 / m_w0,
            gain_64k_vs_32k
        );
        println!(
            "  RLE+CRC fusion edge: v1 (W3b_v1 - W4_v1)={:>+6.2}ms  |  v2 @32K (W3b_v2 - W4_v2_q1008_32k)={:>+6.2}ms",
            fusion_v1, fusion_v2_32k
        );
        for &(name, m_arm) in &[
            ("W4_v1_32k_inline", m_w4v1),
            ("W4_b_32k_cold   ", m_w4b32),
            ("W4_v2_q496_32k  ", m_w4v2_496_32),
            ("W4_v2_q1008_32k ", m_w4v2_1008_32),
            ("W4_v2_q1008_64k ", m_w4v2_1008_64),
            ("W3b_v2_q1008_32k", m_w3b2_1008_32),
        ] {
            let ratio = m_arm / m_w0;
            let i9_1w_save = I9_WORKER_1W_MS * (1.0 - ratio);
            let i9_w8 = I9_WORKER_8W_MS * ratio;
            let margin_std = I9_6GBS_W8_THRESHOLD_MS - i9_w8;
            let margin_ch0 = I9_6GBS_W8_SMALL_CH0_MS - i9_w8;
            println!(
                "  i9 model {name}: {:>+6.2}% vs W0 | 1W save={:>+6.1}ms ({:>+5.2}% wall) | W'/8={:.2}ms (vs 133.9: {:>+5.2}ms, vs 135.4: {:>+5.2}ms)",
                (ratio - 1.0) * 100.0,
                i9_1w_save,
                100.0 * i9_1w_save / I9_WALL_1W_MS,
                i9_w8,
                margin_std,
                margin_ch0
            );
        }
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        let path = args
            .get(1)
            .filter(|s| !s.starts_with("--"))
            .map(|s| s.as_str())
            .unwrap_or("/tmp/wikidata.bz2");
        let mut repeats = 7usize;
        for i in 1..args.len() {
            if args[i] == "--repeats" {
                if let Some(v) = args.get(i + 1) {
                    repeats = v.parse().expect("invalid --repeats");
                }
            }
        }

        let k = Kernels::detect();
        let target_avx2 = cfg!(target_feature = "avx2");
        println!(
            "composed_worker_v2_probe corpus={path} repeats={repeats} target_avx2={target_avx2} runtime_avx2={} SLAB=32KiB/64KiB",
            k.avx2(),
        );

        let blocks = load_admitted_blocks(path, k);
        let crc_combine = CrcCombine::new();

        // 1. Exact byte- and CRC32-verification across ALL 233 admitted blocks & ALL 9 arms
        println!("\n=== EXACTNESS & PER-WORKER WORKING-SET AUDIT (233 admitted Open blocks) ===");
        let mut check_buf = Vec::new();
        for &arm in ARMS {
            let mut w = WorkerState::default();
            let mut ph = PhaseWallNs::default();
            let mut total_ovf = 0usize;
            for (bi, blk) in blocks.iter().enumerate() {
                let got_crc = w.decode_block_arm(&blk.syms, arm, &crc_combine, k, &mut ph);
                total_ovf += if arm.use_q1008() {
                    w.spill1024.ovf.len()
                } else {
                    w.spill512.ovf.len()
                };
                assert_eq!(
                    got_crc,
                    blk.expected_crc,
                    "CRC32 MISMATCH on {} blk {}: got {:08x} want {:08x}",
                    arm.name(),
                    bi,
                    got_crc,
                    blk.expected_crc
                );
                w.extract_output(arm, &mut check_buf);
                assert_eq!(
                    check_buf,
                    blk.expected_bytes,
                    "BYTE MISMATCH on {} blk {} (len {} vs {})",
                    arm.name(),
                    bi,
                    check_buf.len(),
                    blk.expected_bytes.len()
                );
            }
            let (res_b, virt_b) = w.working_set_bytes(arm);
            println!(
                "  EXACT {:<26} 233/233 byte+CRC32 exact | ovf_chunks={total_ovf} | worker scratch: {:>6.1} KiB resident ({:>6.1} KiB virtual)",
                arm.name(),
                res_b as f64 / 1024.0,
                virt_b as f64 / 1024.0
            );
        }

        // 2. WARM benchmark (reused single WorkerState, 7 interleaved forward/reverse repeats)
        let mut clock = CpuClock::new();
        let mut warm_workers: Vec<WorkerState> =
            (0..ARMS.len()).map(|_| WorkerState::default()).collect();
        // Warm-up pass
        for (idx, &arm) in ARMS.iter().enumerate() {
            let mut ph = PhaseWallNs::default();
            for blk in &blocks {
                black_box(warm_workers[idx].decode_block_arm(
                    &blk.syms,
                    arm,
                    &crc_combine,
                    k,
                    &mut ph,
                ));
            }
        }

        let mut warm_res: Vec<Vec<PassResult>> = vec![Vec::with_capacity(repeats); ARMS.len()];
        for rep in 0..repeats {
            let rev = rep % 2 == 1;
            for step in 0..ARMS.len() {
                let idx = if rev { ARMS.len() - 1 - step } else { step };
                let arm = ARMS[idx];
                let w = &mut warm_workers[idx];
                let mut ph = PhaseWallNs::default();
                let mut xor_crc = 0u32;
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                for blk in &blocks {
                    xor_crc ^= w.decode_block_arm(&blk.syms, arm, &crc_combine, k, &mut ph);
                }
                let wall_ns = t0.elapsed().as_nanos() as u64;
                let cpu_ns = clock.now_ns().saturating_sub(c0);
                black_box(xor_crc);
                warm_res[idx].push(PassResult {
                    cpu_ns,
                    wall_ns,
                    ph,
                });
            }
        }
        drop(warm_workers);

        print_results(
            "WARM CPU (steal-free schedstat, reused worker, 7 interleaved)",
            &warm_res,
            true,
        );
        print_results(
            "WARM WALL (Instant, reused worker, 7 interleaved)",
            &warm_res,
            false,
        );

        // 3. L3-COLD benchmark (rotating 16-worker pool > 50 MiB > 45 MiB L3 + 64 MiB flush)
        let mut pool: Vec<WorkerState> = (0..COLD_POOL_WORKERS)
            .map(|_| WorkerState::default())
            .collect();
        let mut l3_flush_buf = vec![0u8; COLD_FLUSH_BYTES];
        // Pre-warm allocations in all 16 pool workers so no arm pays first-touch mmap page faults
        for &arm in &[Arm::W0Shipped, Arm::W4V2Q1008_64k] {
            let mut ph = PhaseWallNs::default();
            for (bi, blk) in blocks.iter().take(COLD_POOL_WORKERS).enumerate() {
                black_box(pool[bi].decode_block_arm(&blk.syms, arm, &crc_combine, k, &mut ph));
            }
        }

        let mut cold_res: Vec<Vec<PassResult>> = vec![Vec::with_capacity(repeats); ARMS.len()];
        for rep in 0..repeats {
            let rev = rep % 2 == 1;
            for step in 0..ARMS.len() {
                let idx = if rev { ARMS.len() - 1 - step } else { step };
                let arm = ARMS[idx];
                flush_l3(&mut l3_flush_buf);
                let mut ph = PhaseWallNs::default();
                let mut xor_crc = 0u32;
                let c0 = clock.now_ns();
                let t0 = Instant::now();
                for (bi, blk) in blocks.iter().enumerate() {
                    let w = &mut pool[bi % COLD_POOL_WORKERS];
                    xor_crc ^= w.decode_block_arm(&blk.syms, arm, &crc_combine, k, &mut ph);
                }
                let wall_ns = t0.elapsed().as_nanos() as u64;
                let cpu_ns = clock.now_ns().saturating_sub(c0);
                black_box(xor_crc);
                cold_res[idx].push(PassResult {
                    cpu_ns,
                    wall_ns,
                    ph,
                });
            }
        }

        print_results(
            "L3-COLD CPU (steal-free schedstat, 16-worker rotating pool >50MiB + 64MiB flush)",
            &cold_res,
            true,
        );
        print_results(
            "L3-COLD WALL (Instant, 16-worker rotating pool >50MiB + 64MiB flush)",
            &cold_res,
            false,
        );
    }
}
