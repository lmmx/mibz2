//! Smooth-zone delivery copy-out characterisation (`delivery-copyout` probe).
//!
//! Measures all four delivery options on real decoded `wikidata.bz2` blocks
//! (209.7 MB, 431 blocks; scaled to `wikidata x4` = 838.8 MB, 1,724 blocks)
//! with exact byte verification (SHA-256, CRC32, and 4-lane streaming hash)
//! and >= 5 interleaved repeats (median and [min..max]):
//!
//! 0. Corpus block-size and RLE1 expansion geometry (empirical limits for
//!    fixed-size caller slices and buffer-pool reallocation).
//! 1. Baseline `io::Read` copy-out across caller buffer sizes (8 KiB, 64 KiB,
//!    256 KiB, 1 MiB, 4 MiB, full-output `Vec`), both warm-L3 and cold-DRAM.
//! 2. Zero-copy block lending (`BzReader` `BufRead::fill_buf` and
//!    `StreamDecoder::read_block`) vs `io::Read` copy-out across five consumer
//!    workloads (discard, 4-lane Hash, newline scan, PCLMULQDQ CRC32, whole-file
//!    materialisation), plus real end-to-end parallel decodes.
//! 3. Caller-provided buffers (`read_block` recycled vs unrecycled) and
//!    deferred RLE1+CRC straight into the user's slice vs worker RLE1 + copy-out.
//! 4. Multi-threaded `memcpy` (1T vs 2T per-block and batched) vs L3-bounded
//!    working-set footprint sweep (0.5 MiB .. 209.7 MiB across the 45 MiB VM L3
//!    cliff) plus end-to-end `StreamDecoder` `slots` sweep.
//!
//! Run:
//!   cargo run --release --example delivery_copyout_census -- [corpora/wikidata.bz2]

use std::hint::black_box;
use std::io::{BufRead, Cursor, Read};
use std::sync::{Arc, Barrier};
use std::time::Instant;

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{bwt, crc, mtf, rle, BzReader, Kernels, StreamDecoder};

/// Target byte volume matching the i9 `wikidata x4` benchmark corpus (4x wikidata.bz2).
const X4_MULTIPLIER: usize = 4;

/// Production `src/parallel.rs:1210` initial pool allocation formula: `n + n / 16 + 64`.
fn prod_pool_cap(n: usize) -> usize {
    n + n / 16 + 64
}

/// 4-lane multiply-rotate streaming hash matching `examples/ab.rs::Hash`.
#[derive(Clone, Debug)]
struct FastHash {
    h: [u64; 4],
    tail: [u8; 32],
    nt: usize,
    n: u64,
}

impl FastHash {
    fn new() -> Self {
        Self {
            h: [1, 2, 3, 4],
            tail: [0; 32],
            nt: 0,
            n: 0,
        }
    }

    #[inline]
    fn block(&mut self, b: &[u8]) {
        for (i, w) in b.chunks_exact(8).enumerate() {
            let w = u64::from_le_bytes(w.try_into().unwrap());
            self.h[i] = (self.h[i] ^ w)
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .rotate_left(31);
        }
    }

    fn update(&mut self, mut b: &[u8]) {
        self.n += b.len() as u64;
        if self.nt > 0 {
            let k = (32 - self.nt).min(b.len());
            self.tail[self.nt..self.nt + k].copy_from_slice(&b[..k]);
            self.nt += k;
            b = &b[k..];
            if self.nt < 32 {
                return;
            }
            let t = self.tail;
            self.block(&t);
            self.nt = 0;
        }
        let mut c = b.chunks_exact(32);
        for x in &mut c {
            self.block(x);
        }
        let rem = c.remainder();
        if !rem.is_empty() {
            self.tail[..rem.len()].copy_from_slice(rem);
            self.nt = rem.len();
        }
    }

    fn finish(mut self) -> (u64, u64) {
        if self.nt > 0 {
            self.tail[self.nt..].fill(0);
            let t = self.tail;
            self.block(&t);
        }
        let mut out = self.n;
        for v in self.h {
            out = (out ^ v).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
        (self.n, out)
    }
}

/// Minimal std-only SHA-256 (FIPS 180-4) for byte-exact verification.
#[derive(Clone)]
struct Sha256 {
    state: [u32; 8],
    buf: [u8; 64],
    buf_len: usize,
    total_bytes: u64,
}

impl Sha256 {
    const H0: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];

    fn new() -> Self {
        Self {
            state: Self::H0,
            buf: [0; 64],
            buf_len: 0,
            total_bytes: 0,
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for (i, chunk) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(chunk.try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for (&rk, &rw) in Self::K.iter().zip(w.iter()) {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(rk)
                .wrapping_add(rw);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        let delta = [a, b, c, d, e, f, g, h];
        for (s, dv) in self.state.iter_mut().zip(delta) {
            *s = s.wrapping_add(dv);
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total_bytes += data.len() as u64;
        if self.buf_len > 0 {
            let take = (64 - self.buf_len).min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];
            if self.buf_len == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buf_len = 0;
            }
        }
        let mut chunks = data.chunks_exact(64);
        for chunk in &mut chunks {
            let block: [u8; 64] = chunk.try_into().unwrap();
            self.compress(&block);
        }
        let rem = chunks.remainder();
        if !rem.is_empty() {
            self.buf[..rem.len()].copy_from_slice(rem);
            self.buf_len = rem.len();
        }
    }

    fn finish(mut self) -> [u8; 32] {
        let bits = self.total_bytes * 8;
        self.buf[self.buf_len] = 0x80;
        self.buf_len += 1;
        if self.buf_len > 56 {
            self.buf[self.buf_len..].fill(0);
            let block = self.buf;
            self.compress(&block);
            self.buf_len = 0;
        }
        self.buf[self.buf_len..56].fill(0);
        self.buf[56..64].copy_from_slice(&bits.to_be_bytes());
        let block = self.buf;
        self.compress(&block);
        let mut out = [0u8; 32];
        for (i, w) in self.state.iter().enumerate() {
            out[4 * i..4 * i + 4].copy_from_slice(&w.to_be_bytes());
        }
        out
    }
}

fn hex_sha256(d: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in d {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Mirror of `src/rle.rs::unrle_in_place` run-growth analysis (read-only, scalar).
/// Returns `(run_count, final_len, max_shift)` where `max_shift` is the in-place
/// upward shift required by `rle::unrle_in_place` (`src/rle.rs:122-127`).
fn analyze_rle1(inverted: &[u8]) -> (usize, usize, usize) {
    let n = inverted.len();
    let mut i = 0usize;
    let mut runs = 0usize;
    let mut grown = 0isize;
    let mut shift = 0isize;
    while i + 3 < n {
        let c = inverted[i];
        if inverted[i + 1] == c && inverted[i + 2] == c && inverted[i + 3] == c {
            let extra = inverted[i + 4] as isize;
            runs += 1;
            grown += extra - 1;
            if grown > shift {
                shift = grown;
            }
            i += 5;
        } else {
            i += 1;
        }
    }
    (runs, (n as isize + grown) as usize, shift as usize)
}

#[derive(Clone, Copy)]
struct Stats {
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

fn summarize(samples_ms: &[f64]) -> Stats {
    let mut v = samples_ms.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let median_ms = if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        0.5 * (v[v.len() / 2 - 1] + v[v.len() / 2])
    };
    Stats {
        median_ms,
        min_ms: v[0],
        max_ms: v[v.len() - 1],
    }
}

fn percentile_usize(sorted: &[usize], num: usize, den: usize) -> usize {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) * num) / den;
    sorted[idx]
}

/// Sequence of block indices whose total bytes equal `target_bytes` (or slightly
/// over, rounded to whole blocks), cycling through the first `prefix_blocks`
/// blocks so the active source footprint stays at `prefix_bytes`.
struct Workload<'a> {
    blocks: &'a [Vec<u8>],
    prefix_blocks: usize,
    prefix_bytes: usize,
    passes: usize,
    total_bytes: usize,
    expected_hash: (u64, u64),
    expected_newlines: u64,
    expected_crc: u32,
}

impl<'a> Workload<'a> {
    fn from_footprint(
        blocks: &'a [Vec<u8>],
        target_footprint: usize,
        target_total: usize,
        k: Kernels,
    ) -> Self {
        let mut prefix_blocks = 0usize;
        let mut prefix_bytes = 0usize;
        for b in blocks {
            prefix_blocks += 1;
            prefix_bytes += b.len();
            if prefix_bytes >= target_footprint {
                break;
            }
        }
        let passes = (target_total / prefix_bytes).max(1);
        let total_bytes = prefix_bytes * passes;

        // Precompute exact reference digests over this exact block sequence.
        let mut fh = FastHash::new();
        let mut nl = 0u64;
        let mut crc_s = !0u32;
        for _ in 0..passes {
            for b in &blocks[..prefix_blocks] {
                fh.update(b);
                nl += b.iter().filter(|&&x| x == b'\n').count() as u64;
                crc_s = crc::update_with(crc_s, b, k);
            }
        }
        Self {
            blocks,
            prefix_blocks,
            prefix_bytes,
            passes,
            total_bytes,
            expected_hash: fh.finish(),
            expected_newlines: nl,
            expected_crc: !crc_s,
        }
    }

    #[inline]
    fn scale_to_x4_ms(&self, raw_ms: f64, x4_bytes: usize) -> f64 {
        raw_ms * (x4_bytes as f64 / self.total_bytes as f64)
    }

    fn warm_cache(&self) {
        let mut acc = 0u64;
        for b in &self.blocks[..self.prefix_blocks] {
            for chunk in b.chunks(64) {
                acc = acc.wrapping_add(chunk[0] as u64);
            }
        }
        black_box(acc);
    }
}

/// Option 1 copy-out loop mirroring `StreamDecoder::read` into a caller buffer of size `buf_sz`.
fn copy_out_pure(w: &Workload<'_>, buf_sz: usize) -> usize {
    let mut dst = vec![0u8; buf_sz];
    let mut total = 0usize;
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            let mut at = 0usize;
            while at < b.len() {
                let n = buf_sz.min(b.len() - at);
                dst[..n].copy_from_slice(&b[at..at + n]);
                black_box(&dst[..n]);
                at += n;
                total += n;
            }
        }
    }
    total
}

/// Option 1 copy-out + `FastHash` (mirrors `examples/ab.rs::drain` and `c71c4a56` `copy_out`).
fn copy_out_hash(w: &Workload<'_>, buf_sz: usize) -> (u64, u64) {
    let mut dst = vec![0u8; buf_sz];
    let mut fh = FastHash::new();
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            let mut at = 0usize;
            while at < b.len() {
                let n = buf_sz.min(b.len() - at);
                dst[..n].copy_from_slice(&b[at..at + n]);
                fh.update(&dst[..n]);
                at += n;
            }
        }
    }
    fh.finish()
}

fn copy_out_newlines(w: &Workload<'_>, buf_sz: usize) -> u64 {
    let mut dst = vec![0u8; buf_sz];
    let mut count = 0u64;
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            let mut at = 0usize;
            while at < b.len() {
                let n = buf_sz.min(b.len() - at);
                dst[..n].copy_from_slice(&b[at..at + n]);
                count += dst[..n].iter().filter(|&&x| x == b'\n').count() as u64;
                at += n;
            }
        }
    }
    count
}

fn copy_out_crc(w: &Workload<'_>, buf_sz: usize, k: Kernels) -> u32 {
    let mut dst = vec![0u8; buf_sz];
    let mut s = !0u32;
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            let mut at = 0usize;
            while at < b.len() {
                let n = buf_sz.min(b.len() - at);
                dst[..n].copy_from_slice(&b[at..at + n]);
                s = crc::update_with(s, &dst[..n], k);
                at += n;
            }
        }
    }
    !s
}

/// Option 2 zero-copy lending consumer loops (operating straight on `&[u8]`).
fn lend_discard(w: &Workload<'_>) -> usize {
    let mut total = 0usize;
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            let s = black_box(b.as_slice());
            total += s.len();
        }
    }
    total
}

fn lend_hash(w: &Workload<'_>) -> (u64, u64) {
    let mut fh = FastHash::new();
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            fh.update(b);
        }
    }
    fh.finish()
}

fn lend_newlines(w: &Workload<'_>) -> u64 {
    let mut count = 0u64;
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            count += b.iter().filter(|&&x| x == b'\n').count() as u64;
        }
    }
    count
}

fn lend_crc(w: &Workload<'_>, k: Kernels) -> u32 {
    let mut s = !0u32;
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            s = crc::update_with(s, b, k);
        }
    }
    !s
}

/// Full-output materialisation into a pre-allocated contiguous `Vec<u8>` (like `decode_parallel`).
fn copy_out_contiguous(w: &Workload<'_>, dst: &mut Vec<u8>) -> usize {
    dst.clear();
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            dst.extend_from_slice(b);
        }
    }
    black_box(dst.len())
}

/// Option 4A: 2-thread per-block split copy into a reusable 4 MiB caller buffer.
fn mt_copy_per_block_buf(w: &Workload<'_>, buf_sz: usize) -> usize {
    let mut dst = vec![0u8; buf_sz];
    let b_start = Barrier::new(2);
    let b_end = Barrier::new(2);
    let total = std::thread::scope(|s| {
        let mut total = 0usize;
        // Split destination once into two halves for safe concurrent writes per block.
        let half = buf_sz / 2;
        let (dst0, dst1) = dst.split_at_mut(half);
        let h1 = s.spawn(|| {
            for _ in 0..w.passes {
                for b in &w.blocks[..w.prefix_blocks] {
                    let mut at = 0usize;
                    while at < b.len() {
                        let n = buf_sz.min(b.len() - at);
                        let mid = n / 2;
                        let right_len = n - mid;
                        b_start.wait();
                        dst1[..right_len].copy_from_slice(&b[at + mid..at + n]);
                        b_end.wait();
                        at += n;
                    }
                }
            }
        });
        for _ in 0..w.passes {
            for b in &w.blocks[..w.prefix_blocks] {
                let mut at = 0usize;
                while at < b.len() {
                    let n = buf_sz.min(b.len() - at);
                    let mid = n / 2;
                    b_start.wait();
                    dst0[..mid].copy_from_slice(&b[at..at + mid]);
                    b_end.wait();
                    black_box(&dst0[..mid]);
                    at += n;
                    total += n;
                }
            }
        }
        h1.join().unwrap();
        total
    });
    total
}

/// Option 4A: 2-thread batched copy of `batch` blocks into a contiguous pre-sized output buffer
/// (simulating copying a ready batch once cumulative offsets are known).
fn mt_copy_batched_contiguous(w: &Workload<'_>, dst: &mut [u8], batch: usize) -> usize {
    let total_blocks = w.passes * w.prefix_blocks;
    // Build flat block table + offsets (outside inner parallel work, as Ready already knows lengths).
    let mut slices: Vec<&[u8]> = Vec::with_capacity(total_blocks);
    for _ in 0..w.passes {
        for b in &w.blocks[..w.prefix_blocks] {
            slices.push(b.as_slice());
        }
    }
    let mut offsets: Vec<usize> = Vec::with_capacity(total_blocks + 1);
    let mut acc = 0usize;
    offsets.push(0);
    for s in &slices {
        acc += s.len();
        offsets.push(acc);
    }
    assert_eq!(acc, w.total_bytes);

    let b_start = Barrier::new(2);
    let b_end = Barrier::new(2);
    let mut rest = &mut dst[..acc];
    let mut batch_slices: Vec<&mut [u8]> = Vec::new();
    let mut i = 0usize;
    while i < total_blocks {
        let end = (i + batch).min(total_blocks);
        let len = offsets[end] - offsets[i];
        let (chunk, tail) = rest.split_at_mut(len);
        batch_slices.push(chunk);
        rest = tail;
        i = end;
    }

    // Divide each batch roughly in half by block count and copy on 2 persistent threads.
    let mut left_jobs: Vec<(&mut [u8], usize, usize)> = Vec::with_capacity(batch_slices.len());
    let mut right_jobs: Vec<(&mut [u8], usize, usize)> = Vec::with_capacity(batch_slices.len());
    for (bi, chunk) in batch_slices.into_iter().enumerate() {
        let start_blk = bi * batch;
        let end_blk = (start_blk + batch).min(total_blocks);
        let mid_blk = start_blk + (end_blk - start_blk) / 2;
        let split_byte = offsets[mid_blk] - offsets[start_blk];
        let (l_dst, r_dst) = chunk.split_at_mut(split_byte);
        left_jobs.push((l_dst, start_blk, mid_blk));
        right_jobs.push((r_dst, mid_blk, end_blk));
    }

    std::thread::scope(|s| {
        let slices_ref = &slices;
        let h1 = s.spawn(|| {
            for (r_dst, lo, hi) in right_jobs {
                b_start.wait();
                let mut pos = 0usize;
                for blk in &slices_ref[lo..hi] {
                    r_dst[pos..pos + blk.len()].copy_from_slice(blk);
                    pos += blk.len();
                }
                b_end.wait();
            }
        });
        for (l_dst, lo, hi) in left_jobs {
            b_start.wait();
            let mut pos = 0usize;
            for blk in &slices_ref[lo..hi] {
                l_dst[pos..pos + blk.len()].copy_from_slice(blk);
                pos += blk.len();
            }
            b_end.wait();
        }
        h1.join().unwrap();
    });
    black_box(dst[0]);
    black_box(dst[acc - 1]);
    acc
}

/// End-to-end delivery modes using the real shipped `BzReader` and `StreamDecoder`.
#[derive(Clone, Copy)]
enum E2eMode {
    BzRead(usize),
    BzBufRead,
    StreamRead(usize),
    StreamReadBlockRecycled,
    StreamReadBlockFresh,
    DecodeParallelVec,
    ReadBlockPreSizedVec(usize),
    ReadBlockChunkedVecs,
}

impl E2eMode {
    fn name(self) -> String {
        match self {
            E2eMode::BzRead(sz) => format!("BzReader::read({}K)", sz / 1024),
            E2eMode::BzBufRead => "BzReader::BufRead(fill_buf)".into(),
            E2eMode::StreamRead(sz) => format!("StreamDecoder::read({}K)", sz / 1024),
            E2eMode::StreamReadBlockRecycled => "StreamDecoder::read_block(recycled)".into(),
            E2eMode::StreamReadBlockFresh => "StreamDecoder::read_block(unrecycled)".into(),
            E2eMode::DecodeParallelVec => "decode_parallel(contiguous Vec)".into(),
            E2eMode::ReadBlockPreSizedVec(_) => "read_block -> pre-sized Vec".into(),
            E2eMode::ReadBlockChunkedVecs => "read_block -> Vec<Vec<u8>> (0-copy)".into(),
        }
    }
}

fn run_e2e(
    compressed: &Arc<[u8]>,
    threads: usize,
    slots: usize,
    mode: E2eMode,
    k: Kernels,
    check_sha: bool,
) -> ((u64, u64), Option<[u8; 32]>, f64) {
    let t0 = Instant::now();
    let mut fh = FastHash::new();
    let mut sha = check_sha.then(Sha256::new);

    match mode {
        E2eMode::BzRead(buf_sz) => {
            let mut reader =
                BzReader::parallel_with(Cursor::new(Arc::clone(compressed)), threads, slots, k);
            let mut buf = vec![0u8; buf_sz];
            loop {
                let n = reader.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                fh.update(&buf[..n]);
                if let Some(s) = &mut sha {
                    s.update(&buf[..n]);
                }
            }
        }
        E2eMode::BzBufRead => {
            let mut reader =
                BzReader::parallel_with(Cursor::new(Arc::clone(compressed)), threads, slots, k);
            loop {
                let chunk = reader.fill_buf().unwrap();
                let n = chunk.len();
                if n == 0 {
                    break;
                }
                fh.update(chunk);
                if let Some(s) = &mut sha {
                    s.update(chunk);
                }
                reader.consume(n);
            }
        }
        E2eMode::StreamRead(buf_sz) => {
            let dec = StreamDecoder::new(threads, slots, k);
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in compressed.chunks(4 << 20) {
                        let mut chunk = dec.spare();
                        chunk.clear();
                        chunk.extend_from_slice(c);
                        if dec.write(chunk).is_err() {
                            return;
                        }
                    }
                    dec.close();
                });
                let mut buf = vec![0u8; buf_sz];
                loop {
                    let n = dec.read(&mut buf).unwrap();
                    if n == 0 {
                        break;
                    }
                    fh.update(&buf[..n]);
                    if let Some(sh) = &mut sha {
                        sh.update(&buf[..n]);
                    }
                }
            });
        }
        E2eMode::StreamReadBlockRecycled | E2eMode::StreamReadBlockFresh => {
            let recycle = matches!(mode, E2eMode::StreamReadBlockRecycled);
            let dec = StreamDecoder::new(threads, slots, k);
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in compressed.chunks(4 << 20) {
                        let mut chunk = dec.spare();
                        chunk.clear();
                        chunk.extend_from_slice(c);
                        if dec.write(chunk).is_err() {
                            return;
                        }
                    }
                    dec.close();
                });
                let mut spent = Vec::new();
                while let Some(b) = dec.read_block(spent).unwrap() {
                    fh.update(&b);
                    if let Some(sh) = &mut sha {
                        sh.update(&b);
                    }
                    spent = if recycle { b } else { Vec::new() };
                }
            });
        }
        E2eMode::DecodeParallelVec => {
            let out = mibz2::decode_parallel(compressed, threads).unwrap();
            fh.update(&out);
            if let Some(sh) = &mut sha {
                sh.update(&out);
            }
        }
        E2eMode::ReadBlockPreSizedVec(cap) => {
            let dec = StreamDecoder::new(threads, slots, k);
            let mut out = Vec::with_capacity(cap);
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in compressed.chunks(4 << 20) {
                        let mut chunk = dec.spare();
                        chunk.clear();
                        chunk.extend_from_slice(c);
                        if dec.write(chunk).is_err() {
                            return;
                        }
                    }
                    dec.close();
                });
                let mut spent = Vec::new();
                while let Some(b) = dec.read_block(spent).unwrap() {
                    out.extend_from_slice(&b);
                    spent = b;
                }
            });
            fh.update(&out);
            if let Some(sh) = &mut sha {
                sh.update(&out);
            }
        }
        E2eMode::ReadBlockChunkedVecs => {
            let dec = StreamDecoder::new(threads, slots, k);
            let mut chunks: Vec<Vec<u8>> = Vec::new();
            std::thread::scope(|s| {
                s.spawn(|| {
                    for c in compressed.chunks(4 << 20) {
                        let mut chunk = dec.spare();
                        chunk.clear();
                        chunk.extend_from_slice(c);
                        if dec.write(chunk).is_err() {
                            return;
                        }
                    }
                    dec.close();
                });
                while let Some(b) = dec.read_block(Vec::new()).unwrap() {
                    chunks.push(b);
                }
            });
            for b in &chunks {
                fh.update(b);
                if let Some(sh) = &mut sha {
                    sh.update(b);
                }
            }
        }
    }

    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    (fh.finish(), sha.map(Sha256::finish), ms)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let corpus_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("corpora/wikidata.bz2");
    let micro_repeats: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(7);
    let e2e_repeats: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);

    let k = Kernels::detect();
    let compressed =
        std::fs::read(corpus_path).unwrap_or_else(|e| panic!("failed to read {corpus_path}: {e}"));

    println!("=== delivery-copyout smooth-zone census ===");
    println!(
        "corpus: {corpus_path} ({} bytes compressed), kernels: avx2={} clmul={}, micro_repeats={micro_repeats}, e2e_repeats={e2e_repeats}",
        compressed.len(),
        k.avx2(),
        k.clmul()
    );

    // -------------------------------------------------------------------------
    // Section 0: Reference decode + block-size & RLE1 expansion geometry
    // -------------------------------------------------------------------------
    let ref_bytes = mibz2::decode(&compressed).expect("reference mibz2::decode");
    let mut ref_fh = FastHash::new();
    ref_fh.update(&ref_bytes);
    let ref_hash = ref_fh.finish();
    let mut ref_sha = Sha256::new();
    ref_sha.update(&ref_bytes);
    let ref_sha_digest = ref_sha.finish();
    let ref_crc = crc::crc32(&ref_bytes, k);

    let x1_bytes = ref_bytes.len();
    let x4_bytes = x1_bytes * X4_MULTIPLIER;

    // Decode block-by-block capturing both pre-RLE1 `inverted` and final decoded bytes.
    let mut sc = Scanner::new(SliceSource(&compressed));
    let mut syms = Vec::new();
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    let mut tt = bwt::TVector::default();
    let mut inverted_blocks: Vec<Vec<u8>> = Vec::new();
    let mut decoded_blocks: Vec<Vec<u8>> = Vec::new();
    let mut block_crcs: Vec<u32> = Vec::new();

    let mut run_counts = Vec::new();
    let mut shifts = Vec::new();
    let mut overflows_prod_cap = 0usize;
    let mut grows_gt_0 = 0usize;
    let mut shift_gt_0 = 0usize;
    let mut max_ratio = 0.0f64;
    let mut max_shift_ratio = 0.0f64;

    while let Event::Block { symbols, .. } = sc.next(syms).expect("scan block") {
        let n = mtf::mtf(&symbols, &mut ll, &mut counts, k).expect("mtf");
        bwt::build(&ll[..n], &counts, &mut tt);
        let mut inv = Vec::with_capacity(n);
        bwt::walk(&tt, symbols.orig_ptr, &mut inv);

        let (rc, expected_len, shift) = analyze_rle1(&inv);
        let mut dec = Vec::with_capacity(expected_len);
        rle::unrle(&inv, &mut dec, k).expect("unrle");
        assert_eq!(dec.len(), expected_len, "analyze_rle1 length mismatch");
        let c = crc::crc32(&dec, k);
        assert_eq!(c, symbols.crc, "block CRC mismatch");

        run_counts.push(rc);
        shifts.push(shift);
        if dec.len() > n {
            grows_gt_0 += 1;
        }
        if shift > 0 {
            shift_gt_0 += 1;
        }
        if n + shift > prod_pool_cap(n) {
            overflows_prod_cap += 1;
        }
        let ratio = dec.len() as f64 / n as f64;
        if ratio > max_ratio {
            max_ratio = ratio;
        }
        let sratio = (n + shift) as f64 / n as f64;
        if sratio > max_shift_ratio {
            max_shift_ratio = sratio;
        }

        inverted_blocks.push(inv);
        decoded_blocks.push(dec);
        block_crcs.push(symbols.crc);
        syms = symbols.syms;
    }

    // Verify concatenated blocks match `mibz2::decode` byte-for-byte.
    let mut chk_fh = FastHash::new();
    let mut chk_sha = Sha256::new();
    let mut chk_crc_s = !0u32;
    for b in &decoded_blocks {
        chk_fh.update(b);
        chk_sha.update(b);
        chk_crc_s = crc::update_with(chk_crc_s, b, k);
    }
    assert_eq!(chk_fh.finish(), ref_hash, "block hash mismatch");
    assert_eq!(chk_sha.finish(), ref_sha_digest, "block sha256 mismatch");
    assert_eq!(!chk_crc_s, ref_crc, "block crc mismatch");
    drop(ref_bytes);

    let mut dec_lens: Vec<usize> = decoded_blocks.iter().map(Vec::len).collect();
    dec_lens.sort_unstable();
    let n_blocks = decoded_blocks.len();
    let mean_len = x1_bytes as f64 / n_blocks as f64;
    let gt_64k = dec_lens.iter().filter(|&&x| x > 64 << 10).count();
    let gt_256k = dec_lens.iter().filter(|&&x| x > 256 << 10).count();
    let gt_512k = dec_lens.iter().filter(|&&x| x > 512 << 10).count();
    let gt_900k = dec_lens.iter().filter(|&&x| x > 900_000).count();
    let gt_1m = dec_lens.iter().filter(|&&x| x > (1 << 20)).count();
    let mut sorted_shifts = shifts.clone();
    sorted_shifts.sort_unstable();

    println!(
        "\n--- Section 0: Block geometry & RLE1 expansion (verified SHA-256={}) ---",
        hex_sha256(&ref_sha_digest)
    );
    println!(
        "blocks: {n_blocks} (x4 = {}), decoded: {x1_bytes} B ({:.2} MiB; x4 = {x4_bytes} B = {:.2} MB = {:.2} MiB)",
        n_blocks * X4_MULTIPLIER,
        x1_bytes as f64 / (1024.0 * 1024.0),
        x4_bytes as f64 / 1e6,
        x4_bytes as f64 / (1024.0 * 1024.0),
    );
    println!(
        "decoded block size (KiB): min={:.1}, p10={:.1}, median={:.1}, mean={:.1}, p90={:.1}, p99={:.1}, max={:.1}",
        dec_lens[0] as f64 / 1024.0,
        percentile_usize(&dec_lens, 10, 100) as f64 / 1024.0,
        percentile_usize(&dec_lens, 50, 100) as f64 / 1024.0,
        mean_len / 1024.0,
        percentile_usize(&dec_lens, 90, 100) as f64 / 1024.0,
        percentile_usize(&dec_lens, 99, 100) as f64 / 1024.0,
        dec_lens[n_blocks - 1] as f64 / 1024.0,
    );
    println!(
        "blocks exceeding caller buf: >64KiB={gt_64k}/{n_blocks} ({:.1}%), >256KiB={gt_256k}/{n_blocks} ({:.1}%), >512KiB={gt_512k}/{n_blocks} ({:.1}%), >900KB={gt_900k}/{n_blocks} ({:.1}%), >1MiB={gt_1m}/{n_blocks} ({:.1}%)",
        100.0 * gt_64k as f64 / n_blocks as f64,
        100.0 * gt_256k as f64 / n_blocks as f64,
        100.0 * gt_512k as f64 / n_blocks as f64,
        100.0 * gt_900k as f64 / n_blocks as f64,
        100.0 * gt_1m as f64 / n_blocks as f64,
    );
    println!(
        "RLE1 growth: shift>0 on {shift_gt_0}/{n_blocks} ({:.1}%), len>n on {grows_gt_0}/{n_blocks} ({:.1}%), overflow prod_pool_cap(n+n/16+64)={overflows_prod_cap}/{n_blocks} ({:.1}%), max len/n={max_ratio:.3}x, max (n+shift)/n={max_shift_ratio:.3}x, shift p50={} B, p95={} B, max={} B",
        100.0 * shift_gt_0 as f64 / n_blocks as f64,
        100.0 * grows_gt_0 as f64 / n_blocks as f64,
        100.0 * overflows_prod_cap as f64 / n_blocks as f64,
        percentile_usize(&sorted_shifts, 50, 100),
        percentile_usize(&sorted_shifts, 95, 100),
        sorted_shifts[n_blocks - 1],
    );

    // Build Warm-L3 (4 MiB footprint, well inside 45 MiB Xeon L3 and 20 MiB i9 L3)
    // and Cold-DRAM (209.7 MiB = all 431 blocks, 4.7x VM L3 / 10.5x i9 L3) workloads.
    let warm_w = Workload::from_footprint(&decoded_blocks, 4 << 20, x4_bytes, k);
    let cold_w = Workload::from_footprint(&decoded_blocks, x1_bytes, x4_bytes, k);

    // -------------------------------------------------------------------------
    // Section 1: Option 1 — Baseline io::Read copy-out across caller buffer sizes
    // -------------------------------------------------------------------------
    println!(
        "\n--- Section 1: Option 1 — Baseline io::Read copy-out (scaled to {:.2} MB = wikidata x4, {micro_repeats} interleaved repeats) ---",
        x4_bytes as f64 / 1e6
    );
    println!(
        "warm footprint: {:.2} MiB ({} blocks x {} passes); cold footprint: {:.2} MiB ({} blocks x {} passes)",
        warm_w.prefix_bytes as f64 / (1024.0 * 1024.0),
        warm_w.prefix_blocks,
        warm_w.passes,
        cold_w.prefix_bytes as f64 / (1024.0 * 1024.0),
        cold_w.prefix_blocks,
        cold_w.passes,
    );

    let buf_sizes: [usize; 5] = [8 << 10, 64 << 10, 256 << 10, 1 << 20, 4 << 20];
    let mut s1_warm_pure: Vec<Vec<f64>> = vec![Vec::new(); buf_sizes.len() + 1];
    let mut s1_cold_pure: Vec<Vec<f64>> = vec![Vec::new(); buf_sizes.len() + 1];
    let mut s1_warm_hash: Vec<Vec<f64>> = vec![Vec::new(); buf_sizes.len()];
    let mut s1_cold_hash: Vec<Vec<f64>> = vec![Vec::new(); buf_sizes.len()];

    // Pre-fault contiguous buffer once (non-zero fill avoids lazy zero-page CoW).
    let mut contig_buf = vec![1u8; x4_bytes.max(warm_w.total_bytes)];
    contig_buf.clear();

    // 1 warmup + micro_repeats interleaved rounds
    for r in 0..=micro_repeats {
        for (i, &bsz) in buf_sizes.iter().enumerate() {
            warm_w.warm_cache();
            let t0 = Instant::now();
            let n = copy_out_pure(&warm_w, bsz);
            let ms = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
            assert_eq!(n, warm_w.total_bytes);
            if r > 0 {
                s1_warm_pure[i].push(ms);
            }

            warm_w.warm_cache();
            let t0 = Instant::now();
            let h = copy_out_hash(&warm_w, bsz);
            let ms = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
            assert_eq!(h, warm_w.expected_hash);
            if r > 0 {
                s1_warm_hash[i].push(ms);
            }

            let t0 = Instant::now();
            let n = copy_out_pure(&cold_w, bsz);
            let ms = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
            assert_eq!(n, cold_w.total_bytes);
            if r > 0 {
                s1_cold_pure[i].push(ms);
            }

            let t0 = Instant::now();
            let h = copy_out_hash(&cold_w, bsz);
            let ms = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
            assert_eq!(h, cold_w.expected_hash);
            if r > 0 {
                s1_cold_hash[i].push(ms);
            }
        }
        // Contiguous full-output copy (pre-faulted 839 MB Vec)
        warm_w.warm_cache();
        let t0 = Instant::now();
        let n = copy_out_contiguous(&warm_w, &mut contig_buf);
        let ms = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        assert_eq!(n, warm_w.total_bytes);
        if r > 0 {
            s1_warm_pure[buf_sizes.len()].push(ms);
        }

        let t0 = Instant::now();
        let n = copy_out_contiguous(&cold_w, &mut contig_buf);
        let ms = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        assert_eq!(n, cold_w.total_bytes);
        if r > 0 {
            s1_cold_pure[buf_sizes.len()].push(ms);
        }
    }

    println!(
        "{:<16} | {:>22} | {:>7} | {:>22} | {:>7} | {:>22} | {:>22}",
        "caller_buf",
        "warm pure ms [min..max]",
        "GB/s",
        "cold pure ms [min..max]",
        "GB/s",
        "warm +Hash ms",
        "cold +Hash ms"
    );
    for (i, &bsz) in buf_sizes.iter().enumerate() {
        let wp = summarize(&s1_warm_pure[i]);
        let cp = summarize(&s1_cold_pure[i]);
        let wh = summarize(&s1_warm_hash[i]);
        let ch = summarize(&s1_cold_hash[i]);
        println!(
            "{:<16} | {:6.2} [{:5.2}..{:5.2}] | {:7.2} | {:6.2} [{:5.2}..{:5.2}] | {:7.2} | {:6.2} [{:5.2}..{:5.2}] | {:6.2} [{:5.2}..{:5.2}]",
            format!("{} KiB", bsz / 1024),
            wp.median_ms, wp.min_ms, wp.max_ms, (x4_bytes as f64 / 1e6) / wp.median_ms,
            cp.median_ms, cp.min_ms, cp.max_ms, (x4_bytes as f64 / 1e6) / cp.median_ms,
            wh.median_ms, wh.min_ms, wh.max_ms,
            ch.median_ms, ch.min_ms, ch.max_ms,
        );
    }
    {
        let wp = summarize(&s1_warm_pure[buf_sizes.len()]);
        let cp = summarize(&s1_cold_pure[buf_sizes.len()]);
        println!(
            "{:<16} | {:6.2} [{:5.2}..{:5.2}] | {:7.2} | {:6.2} [{:5.2}..{:5.2}] | {:7.2} | {:>22} | {:>22}",
            "full Vec (839MB)",
            wp.median_ms, wp.min_ms, wp.max_ms, (x4_bytes as f64 / 1e6) / wp.median_ms,
            cp.median_ms, cp.min_ms, cp.max_ms, (x4_bytes as f64 / 1e6) / cp.median_ms,
            "-", "-"
        );
    }

    // -------------------------------------------------------------------------
    // Section 2A: Option 2 — Zero-copy block lending vs io::Read across consumers
    // -------------------------------------------------------------------------
    println!("\n--- Section 2A: Option 2 — Zero-copy lending vs io::Read(64K / 4M) across consumers (838.80 MB, {micro_repeats} repeats) ---");
    let consumer_names = [
        "discard/no-op",
        "4-lane Hash (ab.rs)",
        "newline scan (wc -l)",
        "CRC32 (PCLMULQDQ)",
    ];
    let mut s2_warm_lend: Vec<Vec<f64>> = vec![Vec::new(); 4];
    let mut s2_warm_r64: Vec<Vec<f64>> = vec![Vec::new(); 4];
    let mut s2_warm_r4m: Vec<Vec<f64>> = vec![Vec::new(); 4];
    let mut s2_cold_lend: Vec<Vec<f64>> = vec![Vec::new(); 4];
    let mut s2_cold_r64: Vec<Vec<f64>> = vec![Vec::new(); 4];
    let mut s2_cold_r4m: Vec<Vec<f64>> = vec![Vec::new(); 4];

    for r in 0..=micro_repeats {
        // 0: discard
        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(lend_discard(&warm_w), warm_w.total_bytes);
        let wl0 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(lend_discard(&cold_w), cold_w.total_bytes);
        let cl0 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_pure(&warm_w, 64 << 10), warm_w.total_bytes);
        let w64_0 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(copy_out_pure(&cold_w, 64 << 10), cold_w.total_bytes);
        let c64_0 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_pure(&warm_w, 4 << 20), warm_w.total_bytes);
        let w4m_0 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(copy_out_pure(&cold_w, 4 << 20), cold_w.total_bytes);
        let c4m_0 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        // 1: FastHash
        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(lend_hash(&warm_w), warm_w.expected_hash);
        let wl1 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(lend_hash(&cold_w), cold_w.expected_hash);
        let cl1 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_hash(&warm_w, 64 << 10), warm_w.expected_hash);
        let w64_1 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(copy_out_hash(&cold_w, 64 << 10), cold_w.expected_hash);
        let c64_1 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_hash(&warm_w, 4 << 20), warm_w.expected_hash);
        let w4m_1 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(copy_out_hash(&cold_w, 4 << 20), cold_w.expected_hash);
        let c4m_1 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        // 2: newline scan
        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(lend_newlines(&warm_w), warm_w.expected_newlines);
        let wl2 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(lend_newlines(&cold_w), cold_w.expected_newlines);
        let cl2 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(
            copy_out_newlines(&warm_w, 64 << 10),
            warm_w.expected_newlines
        );
        let w64_2 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(
            copy_out_newlines(&cold_w, 64 << 10),
            cold_w.expected_newlines
        );
        let c64_2 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(
            copy_out_newlines(&warm_w, 4 << 20),
            warm_w.expected_newlines
        );
        let w4m_2 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(
            copy_out_newlines(&cold_w, 4 << 20),
            cold_w.expected_newlines
        );
        let c4m_2 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        // 3: CRC32
        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(lend_crc(&warm_w, k), warm_w.expected_crc);
        let wl3 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(lend_crc(&cold_w, k), cold_w.expected_crc);
        let cl3 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_crc(&warm_w, 64 << 10, k), warm_w.expected_crc);
        let w64_3 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(copy_out_crc(&cold_w, 64 << 10, k), cold_w.expected_crc);
        let c64_3 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_crc(&warm_w, 4 << 20, k), warm_w.expected_crc);
        let w4m_3 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        let t0 = Instant::now();
        assert_eq!(copy_out_crc(&cold_w, 4 << 20, k), cold_w.expected_crc);
        let c4m_3 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        if r > 0 {
            for (i, v) in [wl0, wl1, wl2, wl3].into_iter().enumerate() {
                s2_warm_lend[i].push(v);
            }
            for (i, v) in [w64_0, w64_1, w64_2, w64_3].into_iter().enumerate() {
                s2_warm_r64[i].push(v);
            }
            for (i, v) in [w4m_0, w4m_1, w4m_2, w4m_3].into_iter().enumerate() {
                s2_warm_r4m[i].push(v);
            }
            for (i, v) in [cl0, cl1, cl2, cl3].into_iter().enumerate() {
                s2_cold_lend[i].push(v);
            }
            for (i, v) in [c64_0, c64_1, c64_2, c64_3].into_iter().enumerate() {
                s2_cold_r64[i].push(v);
            }
            for (i, v) in [c4m_0, c4m_1, c4m_2, c4m_3].into_iter().enumerate() {
                s2_cold_r4m[i].push(v);
            }
        }
    }

    println!(
        "{:<22} | {:>10} {:>10} {:>10} {:>9} | {:>10} {:>10} {:>10} {:>9}",
        "consumer",
        "warm lend",
        "warm 64K",
        "warm 4M",
        "lend/4M",
        "cold lend",
        "cold 64K",
        "cold 4M",
        "lend/4M"
    );
    for (i, name) in consumer_names.iter().enumerate() {
        let wl = summarize(&s2_warm_lend[i]);
        let w64 = summarize(&s2_warm_r64[i]);
        let w4m = summarize(&s2_warm_r4m[i]);
        let cl = summarize(&s2_cold_lend[i]);
        let c64 = summarize(&s2_cold_r64[i]);
        let c4m = summarize(&s2_cold_r4m[i]);
        println!(
            "{:<22} | {:7.2} ms {:7.2} ms {:7.2} ms {:8.2}x | {:7.2} ms {:7.2} ms {:7.2} ms {:8.2}x",
            name,
            wl.median_ms, w64.median_ms, w4m.median_ms, wl.median_ms / w4m.median_ms,
            cl.median_ms, c64.median_ms, c4m.median_ms, cl.median_ms / c4m.median_ms,
        );
        println!(
            "  ranges [min..max]    | [{:5.2}..{:5.2}] [{:5.2}..{:5.2}] [{:5.2}..{:5.2}]           | [{:5.2}..{:5.2}] [{:5.2}..{:5.2}] [{:5.2}..{:5.2}]",
            wl.min_ms, wl.max_ms, w64.min_ms, w64.max_ms, w4m.min_ms, w4m.max_ms,
            cl.min_ms, cl.max_ms, c64.min_ms, c64.max_ms, c4m.min_ms, c4m.max_ms,
        );
    }

    // -------------------------------------------------------------------------
    // Section 3B: Option 3 — Deferred RLE1+CRC straight into user slice vs copy_out
    // -------------------------------------------------------------------------
    println!("\n--- Section 3B: Option 3 — Deferred serial RLE1+CRC into caller buf vs copy-out (scaled to 838.80 MB, {micro_repeats} repeats) ---");
    let mut s3_unrle_crc_cold: Vec<f64> = Vec::new();
    let mut s3_unrle_crc_warm: Vec<f64> = Vec::new();
    let mut s3_copy_only_warm: Vec<f64> = Vec::new();
    let mut s3_copy_only_cold: Vec<f64> = Vec::new();

    // Warm prefix matching warm_w.prefix_blocks
    let mut rle_dst = Vec::with_capacity(2 << 20);
    for r in 0..=micro_repeats {
        // Warm deferred RLE1+CRC
        let t0 = Instant::now();
        let mut fh = FastHash::new();
        for _ in 0..warm_w.passes {
            for (inv, &stored_crc) in inverted_blocks[..warm_w.prefix_blocks]
                .iter()
                .zip(&block_crcs[..warm_w.prefix_blocks])
            {
                rle_dst.clear();
                rle::unrle(inv, &mut rle_dst, k).unwrap();
                let c = crc::crc32(&rle_dst, k);
                assert_eq!(c, stored_crc);
                fh.update(&rle_dst);
            }
        }
        let ms_w = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        assert_eq!(fh.finish(), warm_w.expected_hash);

        // Cold deferred RLE1+CRC (all 431 blocks x 4)
        let t0 = Instant::now();
        let mut fh = FastHash::new();
        for _ in 0..X4_MULTIPLIER {
            for (inv, &stored_crc) in inverted_blocks.iter().zip(&block_crcs) {
                rle_dst.clear();
                rle::unrle(inv, &mut rle_dst, k).unwrap();
                let c = crc::crc32(&rle_dst, k);
                assert_eq!(c, stored_crc);
                fh.update(&rle_dst);
            }
        }
        let ms_c = t0.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(fh.finish(), cold_w.expected_hash);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_hash(&warm_w, 4 << 20), warm_w.expected_hash);
        let ms_cw = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        let t0 = Instant::now();
        assert_eq!(copy_out_hash(&cold_w, 4 << 20), cold_w.expected_hash);
        let ms_cc = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        if r > 0 {
            s3_unrle_crc_warm.push(ms_w);
            s3_unrle_crc_cold.push(ms_c);
            s3_copy_only_warm.push(ms_cw);
            s3_copy_only_cold.push(ms_cc);
        }
    }
    let u_w = summarize(&s3_unrle_crc_warm);
    let u_c = summarize(&s3_unrle_crc_cold);
    let c_w = summarize(&s3_copy_only_warm);
    let c_c = summarize(&s3_copy_only_cold);
    println!(
        "warm (4 MiB):  deferred serial unrle+CRC+Hash = {:6.2} [{:5.2}..{:5.2}] ms  vs  copy_out(4M)+Hash = {:6.2} [{:5.2}..{:5.2}] ms  (ratio {:.2}x)",
        u_w.median_ms, u_w.min_ms, u_w.max_ms, c_w.median_ms, c_w.min_ms, c_w.max_ms, u_w.median_ms / c_w.median_ms
    );
    println!(
        "cold (210 MiB): deferred serial unrle+CRC+Hash = {:6.2} [{:5.2}..{:5.2}] ms  vs  copy_out(4M)+Hash = {:6.2} [{:5.2}..{:5.2}] ms  (ratio {:.2}x)",
        u_c.median_ms, u_c.min_ms, u_c.max_ms, c_c.median_ms, c_c.min_ms, c_c.max_ms, u_c.median_ms / c_c.median_ms
    );

    // -------------------------------------------------------------------------
    // Section 4A: Option 4 — Multi-threaded memcpy (1T vs 2T per-block & batched)
    // -------------------------------------------------------------------------
    println!("\n--- Section 4A: Option 4 — Multi-threaded memcpy (1T vs 2T on 2-vCPU Xeon, 838.80 MB, {micro_repeats} repeats) ---");
    let mut s4_1t_buf_w = Vec::new();
    let mut s4_2t_buf_w = Vec::new();
    let mut s4_1t_buf_c = Vec::new();
    let mut s4_2t_buf_c = Vec::new();
    let mut s4_1t_ctg_w = Vec::new();
    let mut s4_2t_ctg_w = Vec::new();
    let mut s4_1t_ctg_c = Vec::new();
    let mut s4_2t_ctg_c = Vec::new();

    contig_buf.resize(x4_bytes.max(warm_w.total_bytes), 0);

    for r in 0..=micro_repeats {
        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(copy_out_pure(&warm_w, 4 << 20), warm_w.total_bytes);
        let m1 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        assert_eq!(mt_copy_per_block_buf(&warm_w, 4 << 20), warm_w.total_bytes);
        let m2 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        let t0 = Instant::now();
        assert_eq!(copy_out_pure(&cold_w, 4 << 20), cold_w.total_bytes);
        let m3 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        let t0 = Instant::now();
        assert_eq!(mt_copy_per_block_buf(&cold_w, 4 << 20), cold_w.total_bytes);
        let m4 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

        warm_w.warm_cache();
        let t0 = Instant::now();
        let n = copy_out_contiguous(&warm_w, &mut contig_buf);
        let m5 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        assert_eq!(n, warm_w.total_bytes);

        contig_buf.resize(x4_bytes.max(warm_w.total_bytes), 0);
        warm_w.warm_cache();
        let t0 = Instant::now();
        let n = mt_copy_batched_contiguous(&warm_w, &mut contig_buf, 8);
        let m6 = warm_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        assert_eq!(n, warm_w.total_bytes);

        let t0 = Instant::now();
        let n = copy_out_contiguous(&cold_w, &mut contig_buf);
        let m7 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        assert_eq!(n, cold_w.total_bytes);

        contig_buf.resize(x4_bytes.max(warm_w.total_bytes), 0);
        let t0 = Instant::now();
        let n = mt_copy_batched_contiguous(&cold_w, &mut contig_buf, 8);
        let m8 = cold_w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);
        assert_eq!(n, cold_w.total_bytes);

        if r > 0 {
            s4_1t_buf_w.push(m1);
            s4_2t_buf_w.push(m2);
            s4_1t_buf_c.push(m3);
            s4_2t_buf_c.push(m4);
            s4_1t_ctg_w.push(m5);
            s4_2t_ctg_w.push(m6);
            s4_1t_ctg_c.push(m7);
            s4_2t_ctg_c.push(m8);
        }
    }
    drop(contig_buf);

    for (label, s1, s2) in [
        (
            "4 MiB buf, warm L3 (per-block 2T)",
            &s4_1t_buf_w,
            &s4_2t_buf_w,
        ),
        (
            "4 MiB buf, cold DRAM (per-block 2T)",
            &s4_1t_buf_c,
            &s4_2t_buf_c,
        ),
        (
            "839 MB Vec, warm L3 (batch-8 2T)",
            &s4_1t_ctg_w,
            &s4_2t_ctg_w,
        ),
        (
            "839 MB Vec, cold DRAM (batch-8 2T)",
            &s4_1t_ctg_c,
            &s4_2t_ctg_c,
        ),
    ] {
        let a = summarize(s1);
        let b = summarize(s2);
        println!(
            "{:<36} | 1T={:6.2} [{:5.2}..{:5.2}] ms ({:5.2} GB/s) | 2T={:6.2} [{:5.2}..{:5.2}] ms ({:5.2} GB/s) | speedup={:.2}x",
            label,
            a.median_ms, a.min_ms, a.max_ms, (x4_bytes as f64 / 1e6) / a.median_ms,
            b.median_ms, b.min_ms, b.max_ms, (x4_bytes as f64 / 1e6) / b.median_ms,
            a.median_ms / b.median_ms,
        );
    }

    // -------------------------------------------------------------------------
    // Section 4B: Option 4 — L3 working-set sweep (0.5 MiB .. 209.7 MiB)
    // -------------------------------------------------------------------------
    println!("\n--- Section 4B: Working-set sweep across VM 45 MiB L3 cliff (scaled to 838.80 MB, {micro_repeats} repeats) ---");
    let footprints_mib: [usize; 12] = [1, 2, 4, 8, 12, 16, 24, 32, 48, 64, 128, 210];
    let workloads: Vec<Workload<'_>> = footprints_mib
        .iter()
        .map(|&mib| {
            let target = (mib << 20).min(x1_bytes);
            Workload::from_footprint(&decoded_blocks, target, x4_bytes, k)
        })
        .collect();

    let mut ws_copy_pure: Vec<Vec<f64>> = vec![Vec::new(); workloads.len()];
    let mut ws_copy_hash: Vec<Vec<f64>> = vec![Vec::new(); workloads.len()];
    let mut ws_lend_hash: Vec<Vec<f64>> = vec![Vec::new(); workloads.len()];

    for r in 0..=micro_repeats {
        for (i, w) in workloads.iter().enumerate() {
            w.warm_cache();
            let t0 = Instant::now();
            assert_eq!(copy_out_pure(w, 4 << 20), w.total_bytes);
            let mp = w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

            w.warm_cache();
            let t0 = Instant::now();
            assert_eq!(copy_out_hash(w, 4 << 20), w.expected_hash);
            let mh = w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

            w.warm_cache();
            let t0 = Instant::now();
            assert_eq!(lend_hash(w), w.expected_hash);
            let ml = w.scale_to_x4_ms(t0.elapsed().as_secs_f64() * 1000.0, x4_bytes);

            if r > 0 {
                ws_copy_pure[i].push(mp);
                ws_copy_hash[i].push(mh);
                ws_lend_hash[i].push(ml);
            }
        }
    }

    println!(
        "{:>9} {:>6} {:>7} | {:>22} {:>7} | {:>22} {:>7} | {:>22} {:>7}",
        "W (MiB)",
        "blocks",
        "W/45M",
        "copy_out(4M) ms",
        "GB/s",
        "copy(4M)+Hash ms",
        "GB/s",
        "lend+Hash ms",
        "GB/s"
    );
    for (i, w) in workloads.iter().enumerate() {
        let w_mib = w.prefix_bytes as f64 / (1024.0 * 1024.0);
        let sp = summarize(&ws_copy_pure[i]);
        let sh = summarize(&ws_copy_hash[i]);
        let sl = summarize(&ws_lend_hash[i]);
        println!(
            "{:9.2} {:6} {:6.2}x | {:6.2} [{:5.2}..{:5.2}] {:7.2} | {:6.2} [{:5.2}..{:5.2}] {:7.2} | {:6.2} [{:5.2}..{:5.2}] {:7.2}",
            w_mib,
            w.prefix_blocks,
            w_mib / 45.0,
            sp.median_ms, sp.min_ms, sp.max_ms, (x4_bytes as f64 / 1e6) / sp.median_ms,
            sh.median_ms, sh.min_ms, sh.max_ms, (x4_bytes as f64 / 1e6) / sh.median_ms,
            sl.median_ms, sl.min_ms, sl.max_ms, (x4_bytes as f64 / 1e6) / sl.median_ms,
        );
    }

    // -------------------------------------------------------------------------
    // Section 2B & 3A & 4C: Real end-to-end parallel decode on wikidata.bz2
    // -------------------------------------------------------------------------
    let compressed_arc: Arc<[u8]> = Arc::from(compressed);
    println!(
        "\n--- Section 2B/3A/4C: End-to-end parallel decode on wikidata.bz2 ({:.2} MB, threads=2, {e2e_repeats} interleaved repeats, SHA-256 verified) ---",
        x1_bytes as f64 / 1e6
    );
    let e2e_modes = [
        E2eMode::BzRead(8 << 10),
        E2eMode::BzRead(64 << 10),
        E2eMode::BzRead(256 << 10),
        E2eMode::BzRead(1 << 20),
        E2eMode::BzRead(4 << 20),
        E2eMode::BzBufRead,
        E2eMode::StreamRead(64 << 10),
        E2eMode::StreamRead(4 << 20),
        E2eMode::StreamReadBlockRecycled,
        E2eMode::StreamReadBlockFresh,
        E2eMode::DecodeParallelVec,
        E2eMode::ReadBlockPreSizedVec(x1_bytes),
        E2eMode::ReadBlockChunkedVecs,
    ];
    let mut e2e_times: Vec<Vec<f64>> = vec![Vec::new(); e2e_modes.len()];
    for r in 0..=e2e_repeats {
        for (i, &mode) in e2e_modes.iter().enumerate() {
            let check_sha = r == 0;
            let (h, sha, ms) = run_e2e(&compressed_arc, 2, 8, mode, k, check_sha);
            assert_eq!(h, ref_hash, "e2e hash mismatch for {}", mode.name());
            if let Some(d) = sha {
                assert_eq!(d, ref_sha_digest, "e2e sha256 mismatch for {}", mode.name());
            }
            if r > 0 {
                e2e_times[i].push(ms);
            }
        }
    }
    let baseline_bz4m = summarize(&e2e_times[4]).median_ms;
    println!(
        "{:<38} | {:>22} | {:>10} | {:>9}",
        "end-to-end arm (2T, slots=8)", "wall ms [min..max]", "MB/s", "vs Bz4M"
    );
    for (i, &mode) in e2e_modes.iter().enumerate() {
        let st = summarize(&e2e_times[i]);
        let mbps = (x1_bytes as f64 / 1e6) / (st.median_ms / 1000.0);
        println!(
            "{:<38} | {:6.1} [{:5.1}..{:5.1}] | {:7.1} MB/s | {:+6.2}%",
            mode.name(),
            st.median_ms,
            st.min_ms,
            st.max_ms,
            mbps,
            100.0 * (st.median_ms - baseline_bz4m) / baseline_bz4m,
        );
    }

    // Slots sweep (slots = 2, 4, 8 at threads = 2)
    println!("\n--- Section 4C: StreamDecoder slots sweep (wikidata.bz2, threads=2, {e2e_repeats} interleaved repeats) ---");
    let slot_vals = [2usize, 4, 8];
    let mut sl_read: Vec<Vec<f64>> = vec![Vec::new(); slot_vals.len()];
    let mut sl_block: Vec<Vec<f64>> = vec![Vec::new(); slot_vals.len()];
    for r in 0..=e2e_repeats {
        for (i, &slots) in slot_vals.iter().enumerate() {
            let (h1, _, m1) = run_e2e(
                &compressed_arc,
                2,
                slots,
                E2eMode::StreamRead(4 << 20),
                k,
                false,
            );
            assert_eq!(h1, ref_hash);
            let (h2, _, m2) = run_e2e(
                &compressed_arc,
                2,
                slots,
                E2eMode::StreamReadBlockRecycled,
                k,
                false,
            );
            assert_eq!(h2, ref_hash);
            if r > 0 {
                sl_read[i].push(m1);
                sl_block[i].push(m2);
            }
        }
    }
    for (i, &slots) in slot_vals.iter().enumerate() {
        let sr = summarize(&sl_read[i]);
        let sb = summarize(&sl_block[i]);
        println!(
            "slots={slots:2} ({}xT) | read(4M)={:6.1} [{:5.1}..{:5.1}] ms | read_block(recycled)={:6.1} [{:5.1}..{:5.1}] ms | delta={:+.1} ms",
            slots / 2,
            sr.median_ms, sr.min_ms, sr.max_ms,
            sb.median_ms, sb.min_ms, sb.max_ms,
            sb.median_ms - sr.median_ms,
        );
    }

    // Real wikidata x4 end-to-end check on key delivery arms (5 interleaved repeats)
    let mut compressed_x4 = Vec::with_capacity(compressed_arc.len() * X4_MULTIPLIER);
    for _ in 0..X4_MULTIPLIER {
        compressed_x4.extend_from_slice(&compressed_arc);
    }
    let compressed_x4: Arc<[u8]> = Arc::from(compressed_x4);
    println!(
        "\n--- Section 2B (x4): End-to-end parallel decode on wikidata x4 ({:.2} MB, threads=2, slots=8, {e2e_repeats} interleaved repeats) ---",
        x4_bytes as f64 / 1e6
    );
    let x4_modes = [
        E2eMode::BzRead(64 << 10),
        E2eMode::BzRead(4 << 20),
        E2eMode::BzBufRead,
        E2eMode::StreamRead(64 << 10),
        E2eMode::StreamRead(4 << 20),
        E2eMode::StreamReadBlockRecycled,
        E2eMode::ReadBlockChunkedVecs,
        E2eMode::ReadBlockPreSizedVec(x4_bytes),
        E2eMode::DecodeParallelVec,
    ];
    let mut x4_times: Vec<Vec<f64>> = vec![Vec::new(); x4_modes.len()];
    let mut ref_x4_fh = FastHash::new();
    for _ in 0..X4_MULTIPLIER {
        for b in &decoded_blocks {
            ref_x4_fh.update(b);
        }
    }
    let ref_x4_hash = ref_x4_fh.finish();

    for r in 0..=e2e_repeats {
        for (i, &mode) in x4_modes.iter().enumerate() {
            let (h, _, ms) = run_e2e(&compressed_x4, 2, 8, mode, k, false);
            assert_eq!(h, ref_x4_hash, "x4 e2e hash mismatch for {}", mode.name());
            if r > 0 {
                x4_times[i].push(ms);
            }
        }
    }
    let x4_base = summarize(&x4_times[1]).median_ms;
    for (i, &mode) in x4_modes.iter().enumerate() {
        let st = summarize(&x4_times[i]);
        let mbps = (x4_bytes as f64 / 1e6) / (st.median_ms / 1000.0);
        println!(
            "{:<38} | {:6.1} [{:5.1}..{:5.1}] ms | {:7.1} MB/s | {:+6.2}% ({:+.1} ms)",
            mode.name(),
            st.median_ms,
            st.min_ms,
            st.max_ms,
            mbps,
            100.0 * (st.median_ms - x4_base) / x4_base,
            st.median_ms - x4_base,
        );
    }
}
