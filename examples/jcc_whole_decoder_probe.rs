#![forbid(unsafe_code)]
//! Whole-decoder & per-station Intel JCC-erratum 32-byte branch-alignment audit probe.
//!
//! Smooth-zone diagnostic (`Claim: none`, zero `src/` edits, `#![forbid(unsafe_code)]`).
//! Exercises the exact shipped `mibz2` library (`BzReader`, `StreamDecoder`, `decode_parallel`,
//! `Scanner`, `runs`, `mtf`, `OpenCells`, `Segmented`, `rle`, `crc`) so separate release builds
//! (baseline x86-64 / `-C target-cpu=native`, with vs without
//! `-C llvm-args=-x86-align-branch-boundary=32 -C llvm-args=-x86-align-branch=fused+jcc+jmp`)
//! can be compared identically under interleaved paired execution and static `objdump` analysis.
//!
//! FROZEN DECISION TEST (recorded before timing):
//!   Recommend i9-10900K paired hardware verification (plus release x86_64 `.cargo`/wheel flag
//!   evaluation & benchmark alignment control) ONLY IF whole-decoder `wikidata` 1T gains >= 3.0%
//!   disjoint (`[min..max]`) on BOTH baseline x86-64 AND `-C target-cpu=native` (steal-free CPU,
//!   7 interleaved repeats), with zero off-Wikidata corpus regression > 1.0% (disjoint) and
//!   `.text` growth <= 5.0%. Otherwise scope as a benchmarking alignment-control tool only.
//!
//! Usage:
//!   jcc_whole_decoder_probe whole    <FILE.bz2> <bzreader|read|stream1|stream2|par1|par2> [REPS]
//!   jcc_whole_decoder_probe stations <FILE.bz2> [REPS]

use mibz2::bwt::{InverseBwt, OpenCells, Segmented};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{crc, decode_block, decode_parallel, mtf, rle, runs, BzReader, Kernels, Scratch};
use std::hint::black_box;
use std::io::{BufRead, Cursor, Read};
use std::time::Instant;

fn thread_cpu_ns() -> u64 {
    if let Ok(s) = std::fs::read_to_string("/proc/thread-self/schedstat") {
        if let Some(first) = s.split_whitespace().next() {
            if let Ok(v) = first.parse::<u64>() {
                return v;
            }
        }
    }
    0
}

/// Process-wide CPU time (utime + stime across all threads, converted to ns assuming 100 Hz).
fn proc_cpu_ns() -> u64 {
    if let Ok(s) = std::fs::read_to_string("/proc/self/stat") {
        if let Some(rparen) = s.rfind(')') {
            let fields: Vec<&str> = s[rparen + 2..].split_whitespace().collect();
            // After comm+state: field index 11 = utime (14th overall), 12 = stime (15th overall)
            if fields.len() > 12 {
                let ut: u64 = fields[11].parse().unwrap_or(0);
                let st: u64 = fields[12].parse().unwrap_or(0);
                return (ut + st) * 10_000_000;
            }
        }
    }
    thread_cpu_ns()
}

fn drain_bufread<R: BufRead>(mut r: R) -> (usize, u32) {
    let mut total = 0usize;
    let mut crc_st = !0u32;
    loop {
        let buf = r.fill_buf().expect("fill_buf");
        if buf.is_empty() {
            break;
        }
        let n = buf.len();
        total += n;
        crc_st = crc::update(crc_st, &buf[..n.min(64)]);
        r.consume(n);
    }
    (total, !crc_st)
}

fn drain_read<R: Read>(mut r: R, buf: &mut [u8]) -> (usize, u32) {
    let mut total = 0usize;
    let mut crc_st = !0u32;
    loop {
        let n = r.read(buf).expect("read");
        if n == 0 {
            break;
        }
        total += n;
        crc_st = crc::update(crc_st, &buf[..n.min(64)]);
    }
    (total, !crc_st)
}

fn run_whole(path: &str, mode: &str, reps: usize) {
    let data: std::sync::Arc<[u8]> = std::fs::read(path)
        .unwrap_or_else(|e| panic!("failed to read {path}: {e}"))
        .into();
    let mut read_buf = vec![0u8; 64 * 1024];

    let mut run_once = || -> (usize, u32) {
        match mode {
            "bzreader" => drain_bufread(BzReader::new(Cursor::new(&*data))),
            "read" => drain_read(BzReader::new(Cursor::new(&*data)), &mut read_buf),
            "stream1" => drain_read(
                BzReader::parallel(Cursor::new(std::sync::Arc::clone(&data)), 1),
                &mut read_buf,
            ),
            "stream2" => drain_read(
                BzReader::parallel(Cursor::new(std::sync::Arc::clone(&data)), 2),
                &mut read_buf,
            ),
            "par1" => {
                let out = decode_parallel(&data, 1).expect("decode_parallel 1");
                let h = crc::update(!0, &out[..out.len().min(64)]);
                let len = out.len();
                black_box(out);
                (len, !h)
            }
            "par2" => {
                let out = decode_parallel(&data, 2).expect("decode_parallel 2");
                let h = crc::update(!0, &out[..out.len().min(64)]);
                let len = out.len();
                black_box(out);
                (len, !h)
            }
            other => panic!("unknown mode {other}"),
        }
    };

    // 1 untimed warmup
    let ref_sig = run_once();

    let use_proc = matches!(mode, "stream1" | "stream2" | "par1" | "par2");
    for rep in 0..reps {
        let c0 = if use_proc {
            proc_cpu_ns()
        } else {
            thread_cpu_ns()
        };
        let w0 = Instant::now();
        let sig = run_once();
        let wall_ns = w0.elapsed().as_nanos() as u64;
        let cpu_ns = if use_proc {
            proc_cpu_ns().saturating_sub(c0)
        } else {
            thread_cpu_ns().saturating_sub(c0)
        };
        assert_eq!(sig, ref_sig, "signature mismatch on rep {rep}");
        println!(
            "WHOLE mode={} file={} rep={} bytes={} sig={:08x} cpu_ms={:.3} wall_ms={:.3}",
            mode,
            path,
            rep,
            sig.0,
            sig.1,
            cpu_ns as f64 / 1e6,
            wall_ns as f64 / 1e6
        );
    }
}

fn run_stations(path: &str, reps: usize) {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let k = Kernels::detect();

    // Pre-scan blocks once to get owned `Symbols` for isolated station measurement,
    // and count route split (run-route vs column-route).
    let mut blocks = Vec::new();
    let mut run_blocks = 0usize;
    let mut col_blocks = 0usize;
    {
        let mut sc = Scanner::new(SliceSource(&data));
        let mut syms = Vec::new();
        while let Event::Block { symbols: b, .. } = sc
            .next(std::mem::take(&mut syms))
            .unwrap_or_else(|e| panic!("scan: {e:?}"))
        {
            if runs::takes_runs(&b) {
                run_blocks += 1;
            } else {
                col_blocks += 1;
            }
            blocks.push(b);
            syms = Vec::new();
        }
    }

    // Prepare post-MTF inputs once (for isolated C+E timing) and post-BWT RLE1 inputs once (for isolated R timing)
    let mut rle_inputs: Vec<Vec<u8>> = Vec::with_capacity(blocks.len());
    {
        let mut r = runs::Runs::default();
        let mut ll = Vec::new();
        let mut counts = [0u32; 256];
        let mut open = OpenCells::default();
        let mut out = Vec::new();
        for b in &blocks {
            out.clear();
            if runs::takes_runs(b) {
                let n = runs::front(b, &mut r, k).expect("runs::front");
                open.invert_runs(&r, n, b.orig_ptr, &mut out, k);
            } else {
                mtf::mtf(b, &mut ll, &mut counts, k).expect("mtf");
                if !open.invert_in_place(&mut ll, &counts, b.orig_ptr) {
                    open.invert(&ll, &counts, b.orig_ptr, &mut out);
                } else {
                    out.extend_from_slice(&ll);
                }
            }
            rle_inputs.push(out.clone());
        }
    }

    let mut read_buf = vec![0u8; 64 * 1024];

    for rep in 0..=reps {
        // Station S: Scanner::next (A2+A3 bitstream + Huffman)
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut s_syms_total = 0usize;
        {
            let mut sc = Scanner::new(SliceSource(&data));
            let mut syms = Vec::new();
            while let Event::Block { symbols: b, .. } = sc
                .next(std::mem::take(&mut syms))
                .unwrap_or_else(|e| panic!("scan: {e:?}"))
            {
                s_syms_total += b.syms.len();
                syms = b.syms;
            }
        }
        black_box(s_syms_total);
        let s_wall = w0.elapsed().as_nanos() as u64;
        let s_cpu = thread_cpu_ns().saturating_sub(c0);

        // Station M: MTF/RLE2 (A5: runs::front on run-route + mtf::mtf on column-route)
        let mut r = runs::Runs::default();
        let mut ll = Vec::new();
        let mut counts = [0u32; 256];
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut m_rows = 0usize;
        for b in &blocks {
            if runs::takes_runs(b) {
                m_rows += runs::front(b, &mut r, k).expect("runs::front");
            } else {
                mtf::mtf(b, &mut ll, &mut counts, k).expect("mtf");
                m_rows += ll.len();
            }
        }
        black_box(m_rows);
        let m_wall = w0.elapsed().as_nanos() as u64;
        let m_cpu = thread_cpu_ns().saturating_sub(c0);

        // Station M_col_ckpt: mtf_checkpoints on column-route blocks (exercises mtf_checkpoints_avx2!)
        let mut ckpts = Vec::new();
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut mc_rows = 0usize;
        for b in &blocks {
            if !runs::takes_runs(b) {
                mtf::mtf_checkpoints(b, &mut ll, &mut counts, &mut ckpts, k)
                    .expect("mtf_checkpoints");
                mc_rows += ll.len();
            }
        }
        black_box(mc_rows);
        let mc_wall = w0.elapsed().as_nanos() as u64;
        let mc_cpu = thread_cpu_ns().saturating_sub(c0);

        // Station M+CE (OpenCells): MTF + shipped OpenCells inverse BWT
        let mut open = OpenCells::default();
        let mut out = Vec::new();
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut ce_open_bytes = 0usize;
        for b in &blocks {
            out.clear();
            if runs::takes_runs(b) {
                let n = runs::front(b, &mut r, k).expect("runs::front");
                open.invert_runs(&r, n, b.orig_ptr, &mut out, k);
                ce_open_bytes += out.len();
            } else {
                mtf::mtf(b, &mut ll, &mut counts, k).expect("mtf");
                if open.invert_in_place(&mut ll, &counts, b.orig_ptr) {
                    ce_open_bytes += ll.len();
                } else {
                    open.invert(&ll, &counts, b.orig_ptr, &mut out);
                    ce_open_bytes += out.len();
                }
            }
        }
        black_box(ce_open_bytes);
        let mce_open_wall = w0.elapsed().as_nanos() as u64;
        let mce_open_cpu = thread_cpu_ns().saturating_sub(c0);

        // Station M+CE (Segmented): MTF + shipped Segmented inverse BWT
        let mut seg = Segmented::default();
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut ce_seg_bytes = 0usize;
        for b in &blocks {
            out.clear();
            if runs::takes_runs(b) {
                let n = runs::front(b, &mut r, k).expect("runs::front");
                seg.invert_runs(&r, n, b.orig_ptr, &mut out, k);
                ce_seg_bytes += out.len();
            } else {
                mtf::mtf(b, &mut ll, &mut counts, k).expect("mtf");
                if seg.invert_in_place(&mut ll, &counts, b.orig_ptr) {
                    ce_seg_bytes += ll.len();
                } else {
                    seg.invert(&ll, &counts, b.orig_ptr, &mut out);
                    ce_seg_bytes += out.len();
                }
            }
        }
        black_box(ce_seg_bytes);
        let mce_seg_wall = w0.elapsed().as_nanos() as u64;
        let mce_seg_cpu = thread_cpu_ns().saturating_sub(c0);

        // Station R: rle::unrle + crc::update_with (A9)
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut r_crc = 0u32;
        for raw in &rle_inputs {
            out.clear();
            rle::unrle(raw, &mut out, k).expect("unrle");
            r_crc ^= !crc::update_with(!0, &out, k);
        }
        black_box(r_crc);
        let r_wall = w0.elapsed().as_nanos() as u64;
        let r_cpu = thread_cpu_ns().saturating_sub(c0);

        // Station W: shipped decode_block::<OpenCells> (full worker M + C + E + R_inplace)
        let mut scratch: Scratch<OpenCells> = Scratch::default();
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let mut w_crc = 0u32;
        for b in &blocks {
            out.clear();
            w_crc ^= decode_block(b, k, &mut scratch, &mut out).expect("decode_block");
        }
        black_box(w_crc);
        let w_wall = w0.elapsed().as_nanos() as u64;
        let w_cpu = thread_cpu_ns().saturating_sub(c0);

        // Station D_fillbuf vs D_read (whole BzReader 1T)
        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let d_fb = drain_bufread(BzReader::new(Cursor::new(&data)));
        black_box(d_fb);
        let dfb_wall = w0.elapsed().as_nanos() as u64;
        let dfb_cpu = thread_cpu_ns().saturating_sub(c0);

        let c0 = thread_cpu_ns();
        let w0 = Instant::now();
        let d_rd = drain_read(BzReader::new(Cursor::new(&data)), &mut read_buf);
        black_box(d_rd);
        let drd_wall = w0.elapsed().as_nanos() as u64;
        let drd_cpu = thread_cpu_ns().saturating_sub(c0);

        if rep == 0 {
            continue; // skip warmup
        }
        println!(
            "STATIONS file={} rep={} blks={} run={} col={} \
             S_cpu={:.3} S_wall={:.3} \
             M_cpu={:.3} M_wall={:.3} \
             MC_cpu={:.3} MC_wall={:.3} \
             MCE_open_cpu={:.3} MCE_open_wall={:.3} \
             MCE_seg_cpu={:.3} MCE_seg_wall={:.3} \
             R_cpu={:.3} R_wall={:.3} \
             W_cpu={:.3} W_wall={:.3} \
             DFB_cpu={:.3} DFB_wall={:.3} \
             DRD_cpu={:.3} DRD_wall={:.3}",
            path,
            rep - 1,
            blocks.len(),
            run_blocks,
            col_blocks,
            s_cpu as f64 / 1e6,
            s_wall as f64 / 1e6,
            m_cpu as f64 / 1e6,
            m_wall as f64 / 1e6,
            mc_cpu as f64 / 1e6,
            mc_wall as f64 / 1e6,
            mce_open_cpu as f64 / 1e6,
            mce_open_wall as f64 / 1e6,
            mce_seg_cpu as f64 / 1e6,
            mce_seg_wall as f64 / 1e6,
            r_cpu as f64 / 1e6,
            r_wall as f64 / 1e6,
            w_cpu as f64 / 1e6,
            w_wall as f64 / 1e6,
            dfb_cpu as f64 / 1e6,
            dfb_wall as f64 / 1e6,
            drd_cpu as f64 / 1e6,
            drd_wall as f64 / 1e6,
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage:\n  {} whole <FILE.bz2> <bzreader|read|stream1|stream2|par1|par2> [REPS]\n  {} stations <FILE.bz2> [REPS]",
            args[0], args[0]
        );
        std::process::exit(2);
    }
    match args[1].as_str() {
        "whole" => {
            let mode = args.get(3).map(String::as_str).unwrap_or("bzreader");
            let reps = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
            run_whole(&args[2], mode, reps);
        }
        "stations" => {
            let reps = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
            run_stations(&args[2], reps);
        }
        other => panic!("unknown command {other}"),
    }
}
