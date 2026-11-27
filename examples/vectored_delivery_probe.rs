//! Smooth-zone probe (`#![forbid(unsafe_code)]`, zero `src/` edits) pricing the vectored
//! delivery boundary (`Open -> parallel.rs::Ready -> BzReader / Read / Python / decode_parallel`)
//! across 4 delivery layouts:
//!   1. `contiguous_1slice` — shipped single contiguous `out[0..len]`
//!   2. `vectored_4lane`    — 4 disjoint lane slices inside one recycled `out` slab
//!   3. `vectored_16seg`    — 16 sampled-anchor order-statistic slices inside one recycled `out` slab
//!   4. `stitched_4lane`    — 4 lanes compacted via in-place `copy_within` before delivery
//!
//! Also measures exact per-lane (`K=4`) and per-segment (`S=16`) RLE1 expansion ratios and slab
//! overflow rates across all 431 real `wikidata.bz2` blocks plus `text`, `runs`, and `onebyte`,
//! and quantifies `BufRead::fill_buf` short-read counts for Python `#1033` `Reader::read(size)`.
//!
//! Usage:
//!   cargo run --release --example vectored_delivery_probe -- /tmp/wikidata.bz2 [--repeats 7]
#![forbid(unsafe_code)]

use mibz2::{Kernels, StreamDecoder};
use std::fs;
use std::hint::black_box;
use std::time::Instant;

const MAX_SLICES: usize = 16;
const POOL_SLOTS: usize = 16; // SLOTS_PER_THREAD(2) * 8 workers
const FLUSH_BYTES: usize = 64 << 20;

/// Exact safe-Rust prototype of the proposed `parallel.rs` vectored block handle.
/// Zero extra heap allocations: stores up to `MAX_SLICES` `(start, end)` `u32` byte ranges inside
/// a single recycled `Vec<u8>` slab. `total_len` gives exact logical bytes for `Ready::bytes`.
#[derive(Clone)]
struct ReadyBlock {
    buf: Vec<u8>,
    /// `(start, end)` byte offsets inside `buf` in logical output order.
    ranges: [(u32, u32); MAX_SLICES],
    count: u8,
    total_len: u32,
}

impl ReadyBlock {
    fn from_contiguous(buf: Vec<u8>) -> Self {
        let len = buf.len() as u32;
        let mut ranges = [(0u32, 0u32); MAX_SLICES];
        let count = if len > 0 {
            ranges[0] = (0, len);
            1
        } else {
            0
        };
        ReadyBlock {
            buf,
            ranges,
            count,
            total_len: len,
        }
    }

    #[inline(always)]
    fn slice(&self, idx: usize) -> &[u8] {
        let (s, e) = self.ranges[idx];
        &self.buf[s as usize..e as usize]
    }

    #[inline(always)]
    fn slice_count(&self) -> usize {
        self.count as usize
    }

    #[inline(always)]
    fn total_len(&self) -> usize {
        self.total_len as usize
    }
}

/// Linux thread CPU nanoseconds from `/proc/thread-self/schedstat` (field 0), `#![forbid(unsafe_code)]`.
fn thread_cpu_ns() -> u64 {
    if let Ok(s) = fs::read_to_string("/proc/thread-self/schedstat") {
        if let Some(tok) = s.split_whitespace().next() {
            if let Ok(v) = tok.parse::<u64>() {
                return v;
            }
        }
    }
    0
}

/// Exact bzip2 RLE1 encoder: converts decoded output bytes back to the exact pre-RLE1 BWT output
/// stream (runs of 4..=259 identical bytes become `[c, c, c, c, (len - 4) as u8]`).
fn rle1_encode(decoded: &[u8]) -> Vec<u8> {
    let mut pre = Vec::with_capacity(decoded.len());
    let mut i = 0;
    while i < decoded.len() {
        let c = decoded[i];
        let mut run = 1usize;
        while i + run < decoded.len() && decoded[i + run] == c && run < 259 {
            run += 1;
        }
        if run >= 4 {
            pre.extend_from_slice(&[c, c, c, c, (run - 4) as u8]);
        } else {
            for _ in 0..run {
                pre.push(c);
            }
        }
        i += run;
    }
    pre
}

/// Exact bzip2 RLE1 decoder (matching `src/rle.rs`), returning the expanded segment.
fn rle1_decode_segment(pre: &[u8], out: &mut Vec<u8>) {
    out.clear();
    let mut i = 0;
    let mut run_byte = 0u8;
    let mut run_len = 0u8;
    while i < pre.len() {
        let b = pre[i];
        i += 1;
        if run_len == 4 {
            let extra = b as usize;
            out.resize(out.len() + extra, run_byte);
            run_len = 0;
            continue;
        }
        out.push(b);
        if run_len > 0 && b == run_byte {
            run_len += 1;
        } else {
            run_byte = b;
            run_len = 1;
        }
    }
    assert!(run_len != 4, "truncated RLE1 run in segment");
}

/// Deterministic PRNG for sampled-anchor order statistics (S = 16).
struct XorShift64(u64);
impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// Adjust split points so no 4-byte RLE1 run is cut mid-run (mirrors `#1045`'s boundary handling).
fn safe_rle1_splits(pre: &[u8], raw_cuts: &[usize]) -> Vec<usize> {
    // Identify indices where run_len == 0..3 (safe boundary: not inside a 4-byte count byte).
    let mut is_count_byte = vec![false; pre.len()];
    let mut run_byte = 0u8;
    let mut run_len = 0u8;
    for (idx, &b) in pre.iter().enumerate() {
        if run_len == 4 {
            is_count_byte[idx] = true;
            run_len = 0;
            continue;
        }
        if run_len > 0 && b == run_byte {
            run_len += 1;
        } else {
            run_byte = b;
            run_len = 1;
        }
    }
    let mut cuts = Vec::with_capacity(raw_cuts.len() + 2);
    cuts.push(0);
    for &c in raw_cuts {
        let mut pos = c.min(pre.len());
        // Walk forward past any active run or count byte so each segment decodes independently.
        while pos < pre.len() {
            let prev_same = pos > 0 && pre[pos] == pre[pos - 1];
            if is_count_byte[pos] || prev_same {
                pos += 1;
            } else {
                break;
            }
        }
        if pos > *cuts.last().unwrap() && pos < pre.len() {
            cuts.push(pos);
        }
    }
    cuts.push(pre.len());
    cuts
}

/// Build a `ReadyBlock` placing `K` RLE1-expanded segments into a single slab with per-segment
/// proportional slack `num/den` + `pad` bytes. Returns `(ReadyBlock, overflows, slab_bytes)`.
fn build_vectored_block(
    pre: &[u8],
    cuts: &[usize],
    slack_num: usize,
    slack_den: usize,
    pad: usize,
    expected_decoded: &[u8],
) -> (ReadyBlock, usize, usize) {
    let k = cuts.len() - 1;
    assert!((1..=MAX_SLICES).contains(&k));
    let mut seg_decoded: Vec<Vec<u8>> = Vec::with_capacity(k);
    let mut seg_caps: Vec<usize> = Vec::with_capacity(k);
    let mut overflows = 0usize;
    let mut seg_tmp = Vec::new();
    for i in 0..k {
        let pre_slice = &pre[cuts[i]..cuts[i + 1]];
        rle1_decode_segment(pre_slice, &mut seg_tmp);
        let target_cap = pre_slice.len() + (pre_slice.len() * slack_num) / slack_den + pad;
        if seg_tmp.len() > target_cap {
            overflows += 1;
        }
        seg_caps.push(target_cap.max(seg_tmp.len()));
        seg_decoded.push(seg_tmp.clone());
    }

    let slab_len: usize = seg_caps.iter().sum();
    let mut buf = vec![0u8; slab_len];
    let mut ranges = [(0u32, 0u32); MAX_SLICES];
    let mut off = 0usize;
    let mut total_len = 0usize;
    let mut verify = Vec::with_capacity(expected_decoded.len());
    for i in 0..k {
        let s = &seg_decoded[i];
        buf[off..off + s.len()].copy_from_slice(s);
        ranges[i] = (off as u32, (off + s.len()) as u32);
        verify.extend_from_slice(s);
        total_len += s.len();
        off += seg_caps[i];
    }
    assert_eq!(
        verify, expected_decoded,
        "vectored segment reconstruction mismatch"
    );
    (
        ReadyBlock {
            buf,
            ranges,
            count: k as u8,
            total_len: total_len as u32,
        },
        overflows,
        slab_len,
    )
}

/// Stitch a `ReadyBlock` in-place via `copy_within` (pricing `#1045`'s stitch penalty).
#[inline(never)]
fn stitch_in_place(blk: &mut ReadyBlock) {
    let count = blk.slice_count();
    if count <= 1 {
        return;
    }
    let mut dst = blk.ranges[0].1 as usize;
    if blk.ranges[0].0 != 0 {
        let (s, e) = (blk.ranges[0].0 as usize, blk.ranges[0].1 as usize);
        blk.buf.copy_within(s..e, 0);
        dst = e - s;
    }
    for i in 1..count {
        let (s, e) = (blk.ranges[i].0 as usize, blk.ranges[i].1 as usize);
        let len = e - s;
        if s != dst {
            blk.buf.copy_within(s..e, dst);
        }
        dst += len;
    }
    blk.ranges[0] = (0, dst as u32);
    blk.count = 1;
}

/// Consumer 1: zero-copy `BufRead::fill_buf` sequential slice drain + slice-boundary-invariant
/// 64-bit word checksum (carries `< 8`-byte remainders across slice edges so all layouts hash
/// identically).
#[inline(never)]
fn consume_fill_buf_hash(blocks: &[ReadyBlock]) -> u64 {
    let mut acc = 0xcbf2_9ce4_8422_2325u64;
    let mut tail = [0u8; 8];
    let mut tlen = 0usize;
    for blk in blocks {
        for si in 0..blk.slice_count() {
            let mut s = blk.slice(si);
            if tlen > 0 {
                let take = (8 - tlen).min(s.len());
                tail[tlen..tlen + take].copy_from_slice(&s[..take]);
                tlen += take;
                s = &s[take..];
                if tlen == 8 {
                    let w = u64::from_le_bytes(tail);
                    acc = acc.wrapping_mul(0x100_0000_01b3) ^ w;
                    tlen = 0;
                }
            }
            let mut chunks = s.chunks_exact(8);
            for c in &mut chunks {
                let w = u64::from_le_bytes(c.try_into().unwrap());
                acc = acc.wrapping_mul(0x100_0000_01b3) ^ w;
            }
            let rem = chunks.remainder();
            if !rem.is_empty() {
                tail[..rem.len()].copy_from_slice(rem);
                tlen = rem.len();
            }
        }
    }
    for &b in &tail[..tlen] {
        acc = acc.wrapping_mul(0x100_0000_01b3) ^ u64::from(b);
    }
    acc
}

/// Consumer 2a: `Read::read(&mut [u8; BUF])` — single-slice-per-call (matches `BzReader::read`
/// draining `Par::cur` / Python `#1033` `Reader::read(size)` via one `fill_buf` per call).
#[inline(never)]
fn consume_read_single_slice<const BUF: usize>(blocks: &[ReadyBlock]) -> (u64, usize) {
    let mut dst = vec![0u8; BUF];
    let mut calls = 0usize;
    let mut acc = 0u64;
    for blk in blocks {
        for si in 0..blk.slice_count() {
            let s = blk.slice(si);
            let mut at = 0usize;
            while at < s.len() {
                let n = BUF.min(s.len() - at);
                dst[..n].copy_from_slice(&s[at..at + n]);
                acc = acc.wrapping_add(u64::from(dst[0]) + u64::from(dst[n - 1]) + n as u64);
                at += n;
                calls += 1;
            }
        }
    }
    (black_box(acc), calls)
}

/// Consumer 2b: `Read::read(&mut [u8; BUF])` — scatter-fill across segments within one block up to
/// `BUF` bytes per call (eliminates short reads for `S = 16` segments!).
#[inline(never)]
fn consume_read_scatter_fill<const BUF: usize>(blocks: &[ReadyBlock]) -> (u64, usize) {
    let mut dst = vec![0u8; BUF];
    let mut calls = 0usize;
    let mut acc = 0u64;
    let mut filled = 0usize;
    for blk in blocks {
        for si in 0..blk.slice_count() {
            let s = blk.slice(si);
            let mut at = 0usize;
            while at < s.len() {
                let n = (BUF - filled).min(s.len() - at);
                dst[filled..filled + n].copy_from_slice(&s[at..at + n]);
                filled += n;
                at += n;
                if filled == BUF {
                    acc = acc
                        .wrapping_add(u64::from(dst[0]) + u64::from(dst[BUF - 1]) + filled as u64);
                    filled = 0;
                    calls += 1;
                }
            }
        }
    }
    if filled > 0 {
        acc = acc.wrapping_add(u64::from(dst[0]) + u64::from(dst[filled - 1]) + filled as u64);
        calls += 1;
    }
    (black_box(acc), calls)
}

/// Consumer 3: `decode_parallel` scatter-copy into a single contiguous caller `Vec<u8>`.
#[inline(never)]
fn consume_decode_parallel(blocks: &[ReadyBlock], out: &mut Vec<u8>) -> usize {
    out.clear();
    for blk in blocks {
        for si in 0..blk.slice_count() {
            out.extend_from_slice(blk.slice(si));
        }
    }
    black_box(out.len())
}

/// Cold rotating-pool simulation: cycles `POOL_SLOTS` (16) buffers representing an 8-worker
/// `Ready` queue, flushing 64 MiB before each pass so buffers are L3-cold like real 8W handoff.
fn flush_cache(flusher: &mut [u8]) {
    let mut sum = 0u64;
    for b in flusher.iter_mut().step_by(64) {
        *b = b.wrapping_add(1);
        sum = sum.wrapping_add(u64::from(*b));
    }
    black_box(sum);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    Contiguous1,
    Vectored4,
    Vectored16,
    Stitched4,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Contiguous1 => "contiguous_1slice",
            Arm::Vectored4 => "vectored_4lane",
            Arm::Vectored16 => "vectored_16seg",
            Arm::Stitched4 => "stitched_4lane",
        }
    }
}

fn stats_ms(vals: &[f64]) -> (f64, f64, f64) {
    let mut s = vals.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (s[s.len() / 2], s[0], s[s.len() - 1])
}

fn decode_all_blocks(bz2_bytes: &[u8]) -> Vec<Vec<u8>> {
    let dec = StreamDecoder::new(4, 8, Kernels::detect());
    let mut blocks = Vec::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in bz2_bytes.chunks(4 << 20) {
                let _ = dec.write(c.to_vec());
            }
            dec.close();
        });
        while let Some(b) = dec.read_block(Vec::new()).expect("decode") {
            blocks.push(b);
        }
    });
    blocks
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let wikidata_path = args
        .first()
        .cloned()
        .unwrap_or_else(|| "/tmp/wikidata.bz2".to_string());
    let mut repeats = 7usize;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--repeats" && i + 1 < args.len() {
            repeats = args[i + 1].parse().expect("repeats");
            i += 2;
        } else {
            i += 1;
        }
    }

    let bz2 = fs::read(&wikidata_path).expect("read wikidata.bz2");
    let decoded_blocks = decode_all_blocks(&bz2);
    let total_bytes: usize = decoded_blocks.iter().map(Vec::len).sum();
    println!(
        "Loaded {}: {} blocks, {} decoded bytes ({:.2} MiB)",
        wikidata_path,
        decoded_blocks.len(),
        total_bytes,
        total_bytes as f64 / (1024.0 * 1024.0)
    );

    // Precompute exact pre-RLE1 bytes and 4-lane / 16-segment split points for every block.
    let mut pre_blocks: Vec<Vec<u8>> = Vec::with_capacity(decoded_blocks.len());
    let mut cuts_4: Vec<Vec<usize>> = Vec::with_capacity(decoded_blocks.len());
    let mut cuts_16: Vec<Vec<usize>> = Vec::with_capacity(decoded_blocks.len());
    let mut rng = XorShift64(0x5645_4354_4445_4c56);

    for (bi, dec) in decoded_blocks.iter().enumerate() {
        let pre = rle1_encode(dec);
        // Verify exact RLE1 round-trip on every single block!
        let mut chk = Vec::with_capacity(dec.len());
        rle1_decode_segment(&pre, &mut chk);
        assert_eq!(chk, *dec, "RLE1 round-trip mismatch on block {bi}");

        // 4 equal-stride pre-RLE1 cuts (matching 4-lane forward emission)
        let n = pre.len();
        let raw4 = if n >= 4096 {
            vec![n / 4, n / 2, (3 * n) / 4]
        } else {
            vec![]
        };
        cuts_4.push(safe_rle1_splits(&pre, &raw4));

        // 16 order-statistic sampled-anchor cuts (matching forward-4lane-anchor-probe S=16)
        let raw16 = if n >= 16_384 {
            let mut pts: Vec<usize> = (0..15)
                .map(|_| 1 + (rng.next() as usize % (n - 1)))
                .collect();
            pts.sort_unstable();
            pts
        } else {
            vec![]
        };
        cuts_16.push(safe_rle1_splits(&pre, &raw16));
        pre_blocks.push(pre);
    }

    // -----------------------------------------------------------------------
    // Section 1: Per-lane (K=4) & per-segment (S=16) RLE1 expansion + slab census
    // -----------------------------------------------------------------------
    println!("\n=== 1. Slab Sizing & RLE1 Expansion Census (431 wikidata blocks + synthetic worst-case) ===");
    let shipped_cap_sum: usize = pre_blocks.iter().map(|p| p.len() + p.len() / 16 + 64).sum();
    let shipped_cap_max: usize = pre_blocks
        .iter()
        .map(|p| p.len() + p.len() / 16 + 64)
        .max()
        .unwrap_or(0);
    let pre_sum: usize = pre_blocks.iter().map(Vec::len).sum();
    println!(
        "Wikidata pre-RLE1 sum: {:.2} MiB | decoded sum: {:.2} MiB ({:.4}x) | shipped cap sum (n+n/16+64): {:.2} MiB (max blk cap {:.1} KiB)",
        pre_sum as f64 / 1048576.0,
        total_bytes as f64 / 1048576.0,
        total_bytes as f64 / pre_sum as f64,
        shipped_cap_sum as f64 / 1048576.0,
        shipped_cap_max as f64 / 1024.0
    );

    for (label, cuts_vec) in [("K=4 lanes", &cuts_4), ("S=16 segments", &cuts_16)] {
        let mut ratios: Vec<f64> = Vec::new();
        let mut seg_lens: Vec<usize> = Vec::new();
        let mut seg_tmp = Vec::new();
        for (pre, cuts) in pre_blocks.iter().zip(cuts_vec.iter()) {
            for w in cuts.windows(2) {
                let pre_s = &pre[w[0]..w[1]];
                rle1_decode_segment(pre_s, &mut seg_tmp);
                ratios.push(seg_tmp.len() as f64 / pre_s.len().max(1) as f64);
                seg_lens.push(seg_tmp.len());
            }
        }
        ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
        seg_lens.sort_unstable();
        let qf = |p: f64| ratios[((ratios.len() - 1) as f64 * p).round() as usize];
        let qi = |p: f64| seg_lens[((seg_lens.len() - 1) as f64 * p).round() as usize];
        println!(
            "  {label:14}: n_segs={}, ratio p50={:.4} p95={:.4} p99={:.4} max={:.4} | seg_len p10={:.1}K p50={:.1}K p90={:.1}K",
            ratios.len(),
            qf(0.50),
            qf(0.95),
            qf(0.99),
            *ratios.last().unwrap(),
            qi(0.10) as f64 / 1024.0,
            qi(0.50) as f64 / 1024.0,
            qi(0.90) as f64 / 1024.0
        );
        for (name, num, den, pad) in [
            ("+0% +16B", 0, 1, 16),
            ("+3.125% +64B", 1, 32, 64),
            ("+5% +64B", 1, 20, 64),
            ("+6.25% +64B (shipped)", 1, 16, 64),
            ("+10% +256B", 1, 10, 256),
            ("+15% +256B", 3, 20, 256),
            ("+25% +256B", 1, 4, 256),
        ] {
            let mut ovf = 0usize;
            let mut ovf_blks = 0usize;
            let mut slab_sum = 0usize;
            let mut slab_max = 0usize;
            for ((pre, cuts), dec) in pre_blocks
                .iter()
                .zip(cuts_vec.iter())
                .zip(decoded_blocks.iter())
            {
                let (_, o, sl) = build_vectored_block(pre, cuts, num, den, pad, dec);
                ovf += o;
                if o > 0 {
                    ovf_blks += 1;
                }
                slab_sum += sl;
                slab_max = slab_max.max(sl);
            }
            println!(
                "    slack {:22}: overflows={:4} ({:3}/431 blks) | slab_sum={:.2} MiB ({:.4}x shipped) | max_blk={:.1} KiB ({:.4}x shipped)",
                name,
                ovf,
                ovf_blks,
                slab_sum as f64 / 1048576.0,
                slab_sum as f64 / shipped_cap_sum as f64,
                slab_max as f64 / 1024.0,
                slab_max as f64 / shipped_cap_max as f64
            );
        }
    }

    // Synthetic guard corpora: prose/text, runs, onebyte (900 KB pre-RLE1 worst case!)
    println!(
        "\n  Synthetic RLE1-expansion guards (pre-RLE1 block = 900,000 bytes or real docs text):"
    );
    let mut text_raw = Vec::new();
    if let Ok(entries) = fs::read_dir("docs/journal/2026-11-27-explore") {
        for e in entries.flatten() {
            if let Ok(b) = fs::read(e.path()) {
                text_raw.extend_from_slice(&b);
            }
        }
    }
    while text_raw.len() < 900_000 {
        let cur = text_raw.len();
        text_raw.extend_from_within(..cur.min(900_000 - cur));
    }
    text_raw.truncate(900_000);
    // runs: 50% 4..20-byte runs, 50% literals
    let mut runs_raw = Vec::with_capacity(900_000);
    let mut g2 = XorShift64(0x7275_6e73);
    while runs_raw.len() < 900_000 {
        let c = (g2.next() & 0xFF) as u8;
        let r = if (g2.next() & 3) == 0 {
            4 + (g2.next() as usize % 20)
        } else {
            1
        };
        let take = r.min(900_000 - runs_raw.len());
        runs_raw.resize(runs_raw.len() + take, c);
    }
    // onebyte: 900_000 bytes of 0x41 -> pre-RLE1 is 900_000 / 259 * 5 = 17,375 bytes (51.8x expansion!)
    let onebyte_raw = vec![b'A'; 900_000];
    // worst-case 900KB pre-RLE1 onebyte -> 46.6 MB decoded! (test with 100KB pre-RLE1 = 5.18 MB decoded)
    let onebyte_heavy_raw = vec![b'A'; 5_180_000];
    for (name, dec) in [
        ("text_900k", &text_raw),
        ("runs_900k", &runs_raw),
        ("onebyte_900k_out", &onebyte_raw),
        ("onebyte_5.18M_out", &onebyte_heavy_raw),
    ] {
        let pre = rle1_encode(dec);
        let n = pre.len();
        let c4 = safe_rle1_splits(&pre, &[n / 4, n / 2, (3 * n) / 4]);
        let mut r16 = Vec::new();
        for _ in 0..15 {
            r16.push(1 + (g2.next() as usize % n.max(2) - 1).min(n.saturating_sub(1)));
        }
        r16.sort_unstable();
        let c16 = safe_rle1_splits(&pre, &r16);
        let (_, o4_6, _) = build_vectored_block(&pre, &c4, 1, 16, 64, dec);
        let (_, o4_25, _) = build_vectored_block(&pre, &c4, 1, 4, 256, dec);
        let (_, o16_6, _) = build_vectored_block(&pre, &c16, 1, 16, 64, dec);
        let (_, o16_25, _) = build_vectored_block(&pre, &c16, 1, 4, 256, dec);
        println!(
            "    {:18}: pre={:7} -> dec={:7} ({:5.2}x) | K=4 ovf(+6.25%/+25%)={}/{} | S=16 ovf(+6.25%/+25%)={}/{}",
            name,
            pre.len(),
            dec.len(),
            dec.len() as f64 / pre.len().max(1) as f64,
            o4_6,
            o4_25,
            o16_6,
            o16_25
        );
    }

    // -----------------------------------------------------------------------
    // Build the 4 delivery datasets for all 431 wikidata blocks (at +6.25% + 64B slab)
    // -----------------------------------------------------------------------
    let blocks_1: Vec<ReadyBlock> = decoded_blocks
        .iter()
        .map(|d| ReadyBlock::from_contiguous(d.clone()))
        .collect();
    let blocks_4: Vec<ReadyBlock> = pre_blocks
        .iter()
        .zip(cuts_4.iter())
        .zip(decoded_blocks.iter())
        .map(|((pre, cuts), dec)| build_vectored_block(pre, cuts, 1, 16, 64, dec).0)
        .collect();
    let blocks_16: Vec<ReadyBlock> = pre_blocks
        .iter()
        .zip(cuts_16.iter())
        .zip(decoded_blocks.iter())
        .map(|((pre, cuts), dec)| build_vectored_block(pre, cuts, 1, 16, 64, dec).0)
        .collect();

    // Verify 100% byte-identical hash across all 4 representations!
    let h1 = consume_fill_buf_hash(&blocks_1);
    let h4 = consume_fill_buf_hash(&blocks_4);
    let h16 = consume_fill_buf_hash(&blocks_16);
    let mut stitched_check = blocks_4.clone();
    for b in &mut stitched_check {
        stitch_in_place(b);
    }
    let hs = consume_fill_buf_hash(&stitched_check);
    assert_eq!(h1, h4);
    assert_eq!(h1, h16);
    assert_eq!(h1, hs);
    println!("\nExact hash parity verified across all 4 layouts: 0x{h1:016x}");

    // -----------------------------------------------------------------------
    // Section 2: Python #1033 Reader::read(size) short-read / crossing count census
    // -----------------------------------------------------------------------
    println!(
        "\n=== 2. Read/Python #1033 Crossing Count (431 wikidata blocks, 200.0 MiB decoded) ==="
    );
    for (label, blks) in [
        ("contiguous_1slice", &blocks_1),
        ("vectored_4lane", &blocks_4),
        ("vectored_16seg", &blocks_16),
    ] {
        let (_, c8) = consume_read_single_slice::<8192>(blks);
        let (_, c64) = consume_read_single_slice::<65536>(blks);
        let (_, c256) = consume_read_single_slice::<262144>(blks);
        let (_, c1m) = consume_read_single_slice::<1048576>(blks);
        let (_, sc64) = consume_read_scatter_fill::<65536>(blks);
        let (_, sc256) = consume_read_scatter_fill::<262144>(blks);
        println!(
            "  {:18}: single-slice calls 8K={:5} 64K={:4} 256K={:4} 1M={:4} | scatter-fill 64K={:4} 256K={:4}",
            label, c8, c64, c256, c1m, sc64, sc256
        );
    }

    // -----------------------------------------------------------------------
    // Section 3: Timed delivery benchmark (7 interleaved repeats, warm + cold rotating-pool)
    // -----------------------------------------------------------------------
    let arms = [
        Arm::Contiguous1,
        Arm::Vectored4,
        Arm::Vectored16,
        Arm::Stitched4,
    ];
    let mut flusher = vec![0u8; FLUSH_BYTES];
    let mut dp_out = Vec::with_capacity(total_bytes);

    // Prepare cold rotating-pool slices (431 blocks, but timed with 64 MiB flush before pass;
    // since 431 blocks = 200 MiB > 10x L3, a full pass over all 431 blocks is inherently L3-cold!).
    // For "warm", time a 16-block rotating window (16 full blocks ~= 14 MiB -> warm in L3 / host RAM)
    // AND a single-block-at-a-time warm loop (16x repeats of 16 blocks where each block fits L2/L3).
    let warm_blocks_1: Vec<ReadyBlock> = blocks_1
        .iter()
        .filter(|b| b.total_len() >= 100_000)
        .take(POOL_SLOTS)
        .cloned()
        .collect();
    let warm_blocks_4: Vec<ReadyBlock> = blocks_4
        .iter()
        .filter(|b| b.total_len() >= 100_000)
        .take(POOL_SLOTS)
        .cloned()
        .collect();
    let warm_blocks_16: Vec<ReadyBlock> = blocks_16
        .iter()
        .filter(|b| b.total_len() >= 100_000)
        .take(POOL_SLOTS)
        .cloned()
        .collect();
    let warm_total: usize = warm_blocks_1.iter().map(ReadyBlock::total_len).sum();
    let warm_scale = total_bytes as f64 / warm_total as f64;

    println!(
        "\n=== 3. Delivery Benchmark ({repeats} interleaved repeats; thread-CPU ms scaled to 431-blk 200 MiB wikidata) ==="
    );
    println!("  (Warm: 16 full blocks x 8 passes in L3/L2; Cold: all 431 blocks = 200 MiB after 64 MiB cache flush)");

    for (consumer_name, consumer_id) in [
        ("fill_buf_hash (zero-copy BufRead)", 0u8),
        ("read_64k_single (BzReader/Py #1033)", 1u8),
        ("read_64k_scatter (multi-seg read)", 2u8),
        ("read_256k_single (BzReader/Py #1033)", 3u8),
        ("read_256k_scatter (multi-seg read)", 4u8),
        ("decode_parallel (Vec<u8> extend)", 5u8),
    ] {
        println!("\n--- Consumer: {consumer_name} ---");
        let mut warm_cpu: [Vec<f64>; 4] = Default::default();
        let mut cold_cpu: [Vec<f64>; 4] = Default::default();
        let mut cold_wall: [Vec<f64>; 4] = Default::default();

        // Scratch copy for Stitched4 so we price the actual in-place copy_within every repeat!
        // To avoid timing `.clone()`, we copy the 4-lane slices into a preallocated scratch
        // outside the clock, then time `stitch_in_place + consume` inside the clock.
        let mut warm_stitch_bufs: Vec<ReadyBlock> = warm_blocks_4.clone();
        let mut cold_stitch_bufs: Vec<ReadyBlock> = blocks_4.clone();

        for rep in 0..repeats {
            let mut order: Vec<usize> = (0..4).collect();
            if rep % 2 == 1 {
                order.reverse();
            }
            for ai in order {
                let arm = arms[ai];
                // --- WARM (8 passes over 16 blocks, scaled to full 431 blocks) ---
                let w_blks: &[ReadyBlock] = match arm {
                    Arm::Contiguous1 => &warm_blocks_1,
                    Arm::Vectored4 => &warm_blocks_4,
                    Arm::Vectored16 => &warm_blocks_16,
                    Arm::Stitched4 => &warm_blocks_4,
                };
                if arm == Arm::Stitched4 {
                    // Time 8 iterations: restore outside clock, stitch + consume inside clock.
                    let mut sum_ns = 0u64;
                    for _ in 0..8 {
                        for (dst, src) in warm_stitch_bufs.iter_mut().zip(warm_blocks_4.iter()) {
                            dst.buf.copy_from_slice(&src.buf);
                            dst.ranges = src.ranges;
                            dst.count = src.count;
                        }
                        let c0 = thread_cpu_ns();
                        for b in &mut warm_stitch_bufs {
                            stitch_in_place(b);
                        }
                        run_consumer(consumer_id, &warm_stitch_bufs, &mut dp_out);
                        sum_ns += thread_cpu_ns().saturating_sub(c0);
                    }
                    warm_cpu[ai].push((sum_ns as f64 / 8.0) * warm_scale / 1e6);
                } else {
                    // Warm up once, then time 8 passes.
                    run_consumer(consumer_id, w_blks, &mut dp_out);
                    let c0 = thread_cpu_ns();
                    for _ in 0..8 {
                        run_consumer(consumer_id, w_blks, &mut dp_out);
                    }
                    let ns = thread_cpu_ns().saturating_sub(c0);
                    warm_cpu[ai].push((ns as f64 / 8.0) * warm_scale / 1e6);
                }

                // --- COLD (all 431 blocks = 200 MiB, after 64 MiB cache flush) ---
                if arm == Arm::Stitched4 {
                    for (dst, src) in cold_stitch_bufs.iter_mut().zip(blocks_4.iter()) {
                        dst.buf.copy_from_slice(&src.buf);
                        dst.ranges = src.ranges;
                        dst.count = src.count;
                    }
                    flush_cache(&mut flusher);
                    let w0 = Instant::now();
                    let c0 = thread_cpu_ns();
                    for b in &mut cold_stitch_bufs {
                        stitch_in_place(b);
                    }
                    run_consumer(consumer_id, &cold_stitch_bufs, &mut dp_out);
                    let cns = thread_cpu_ns().saturating_sub(c0);
                    let wns = w0.elapsed().as_nanos() as u64;
                    cold_cpu[ai].push(cns as f64 / 1e6);
                    cold_wall[ai].push(wns as f64 / 1e6);
                } else {
                    let c_blks: &[ReadyBlock] = match arm {
                        Arm::Contiguous1 => &blocks_1,
                        Arm::Vectored4 => &blocks_4,
                        Arm::Vectored16 => &blocks_16,
                        Arm::Stitched4 => unreachable!(),
                    };
                    flush_cache(&mut flusher);
                    let w0 = Instant::now();
                    let c0 = thread_cpu_ns();
                    run_consumer(consumer_id, c_blks, &mut dp_out);
                    let cns = thread_cpu_ns().saturating_sub(c0);
                    let wns = w0.elapsed().as_nanos() as u64;
                    cold_cpu[ai].push(cns as f64 / 1e6);
                    cold_wall[ai].push(wns as f64 / 1e6);
                }
            }
        }

        let (base_w_med, _, _) = stats_ms(&warm_cpu[0]);
        let (base_c_med, _, _) = stats_ms(&cold_cpu[0]);
        for ai in 0..4 {
            let (wm, wlo, whi) = stats_ms(&warm_cpu[ai]);
            let (cm, clo, chi) = stats_ms(&cold_cpu[ai]);
            let (cw, _, _) = stats_ms(&cold_wall[ai]);
            println!(
                "  {:18} | warm_cpu {:6.2} [{:6.2}..{:6.2}] ({:.3}x) | cold_cpu {:6.2} [{:6.2}..{:6.2}] ({:.3}x) | cold_wall {:6.2} ms",
                arms[ai].name(),
                wm,
                wlo,
                whi,
                wm / base_w_med.max(1e-9),
                cm,
                clo,
                chi,
                cm / base_c_med.max(1e-9),
                cw
            );
        }
    }
}

#[inline(never)]
fn run_consumer(id: u8, blks: &[ReadyBlock], dp_out: &mut Vec<u8>) {
    match id {
        0 => {
            black_box(consume_fill_buf_hash(blks));
        }
        1 => {
            black_box(consume_read_single_slice::<65536>(blks));
        }
        2 => {
            black_box(consume_read_scatter_fill::<65536>(blks));
        }
        3 => {
            black_box(consume_read_single_slice::<262144>(blks));
        }
        4 => {
            black_box(consume_read_scatter_fill::<262144>(blks));
        }
        5 => {
            black_box(consume_decode_parallel(blks, dp_out));
        }
        _ => unreachable!(),
    }
}
