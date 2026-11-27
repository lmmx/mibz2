#![forbid(unsafe_code)]
//! Microsecond lifecycle dissection of `mibz2`'s parallel decoder (`StreamDecoder`,
//! `BzReader::parallel`, and `decode_parallel`):
//!   - `T_spawn`:    `StreamDecoder::new` / `launch` -> worker threads live
//!   - `T_ramp`:     first chunk read/copy -> first block dispatched -> all `k` workers active
//!   - `T_steady`:   steady-state wall vs `sum(worker_busy) / k`, scanner wait, worker idle
//!     gaps, lock/condvar handoff, head-of-line ready stalls, output copy-out
//!   - `T_drain`:    last block dispatched -> last block decoded (straggler) -> EOF read
//!   - `T_teardown`: EOF returned -> `Drop` (`join()` + `Scratch`/`Shared` `munmap`)
//!
//! Compiles under both default features (zero tracing overhead for pure wall/RSS timing and
//! candidate A/B evaluation) and `--features trace` (nanosecond internal timeline via
//! `mibz2::trace::set_level(2)` on unmodified `src/parallel.rs`).

#[path = "../tests/common/mod.rs"]
mod common;

use mibz2::bwt::{Cells, InverseBwt, OpenCells};
use mibz2::parallel::{StreamDecoder, SLOTS_PER_THREAD};
use mibz2::{BzReader, Kernels};
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// 4-lane 64-bit stream hash matching `examples/ab.rs` (>15 GB/s, zero `unsafe`) so the reader
/// sink never artificially bottlenecks the parallel decoder.
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
            let take = (32 - self.nt).min(b.len());
            self.tail[self.nt..self.nt + take].copy_from_slice(&b[..take]);
            self.nt += take;
            b = &b[take..];
            if self.nt == 32 {
                let t = self.tail;
                self.block(&t);
                self.nt = 0;
            }
        }
        let mut chunks = b.chunks_exact(32);
        for c in &mut chunks {
            self.block(c);
        }
        let rem = chunks.remainder();
        if !rem.is_empty() {
            self.tail[..rem.len()].copy_from_slice(rem);
            self.nt = rem.len();
        }
    }

    fn finish(mut self) -> u64 {
        if self.nt > 0 {
            self.tail[self.nt..].fill(0);
            let t = self.tail;
            self.block(&t);
        }
        let mut x = self.n;
        for v in self.h {
            x = (x ^ v).wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(27);
        }
        x
    }
}

fn vmhwm_kb() -> u64 {
    let Ok(s) = std::fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            if let Some(tok) = rest.split_whitespace().next() {
                if let Ok(v) = tok.parse::<u64>() {
                    return v;
                }
            }
        }
    }
    0
}

fn gen_corpora(dir: &Path, scale: usize) {
    std::fs::create_dir_all(dir).expect("create corpora dir");
    for (raw_name, raw_data) in common::corpora(scale) {
        let name = raw_name.replace(' ', "");
        let bz2 = common::compress(&raw_data, 9);
        let p = dir.join(format!("{name}.bz2"));
        std::fs::write(&p, &bz2).expect("write bz2");
        eprintln!(
            "generated {}: {} -> {} bytes",
            p.display(),
            raw_data.len(),
            bz2.len()
        );
    }
    let wiki_src = Path::new("/home/antigravity/wt-a6b-probe/corpora/wikidata.bz2");
    let wiki_dst = dir.join("wikidata.bz2");
    if wiki_src.exists() && !wiki_dst.exists() {
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(wiki_src, &wiki_dst);
    }
}

/// Outer-phase timings (microseconds) of unmodified `mibz2` parallel entry points.
struct OuterSample {
    t_new_us: u64,
    t_first_us: u64,
    t_body_us: u64,
    t_eof_us: u64,
    t_drop_us: u64,
    t_total_us: u64,
    out_bytes: u64,
    hash: u64,
    vmhwm_kb: u64,
}

fn run_bzread_with<I: InverseBwt + 'static>(
    path: &Path,
    threads: usize,
    use_bufread: bool,
    auto_plan: bool,
) -> OuterSample {
    let f = File::open(path).expect("open bz2");
    let k = Kernels::detect();
    let slots = SLOTS_PER_THREAD * threads.max(1);
    let t0 = Instant::now();
    let mut r = if auto_plan {
        BzReader::parallel(f, threads)
    } else {
        BzReader::parallel_workers_with::<I>(f, threads, slots, k)
    };
    let t_new = t0.elapsed();
    let mut t_first = None;
    let mut out_bytes = 0u64;
    let mut h = FastHash::new();
    if use_bufread {
        loop {
            let buf = r.fill_buf().expect("fill_buf");
            if buf.is_empty() {
                break;
            }
            if t_first.is_none() {
                t_first = Some(t0.elapsed());
            }
            out_bytes += buf.len() as u64;
            h.update(buf);
            let n = buf.len();
            r.consume(n);
        }
    } else {
        let mut buf = vec![0u8; 4 << 20];
        loop {
            let n = r.read(&mut buf).expect("read");
            if n == 0 {
                break;
            }
            if t_first.is_none() {
                t_first = Some(t0.elapsed());
            }
            out_bytes += n as u64;
            h.update(&buf[..n]);
        }
    }
    let t_eof = t0.elapsed();
    drop(r);
    let t_total = t0.elapsed();
    let t_first_us = t_first.unwrap_or(t_eof).as_micros() as u64;
    let t_eof_us = t_eof.as_micros() as u64;
    let t_total_us = t_total.as_micros() as u64;
    OuterSample {
        t_new_us: t_new.as_micros() as u64,
        t_first_us,
        t_body_us: t_eof_us.saturating_sub(t_first_us),
        t_eof_us,
        t_drop_us: t_total_us.saturating_sub(t_eof_us),
        t_total_us,
        out_bytes,
        hash: h.finish(),
        vmhwm_kb: vmhwm_kb(),
    }
}

fn run_stream_mem_with<I: InverseBwt + 'static>(
    data: &[u8],
    threads: usize,
    first_chunk: usize,
    steady_chunk: usize,
    collect_out: bool,
) -> OuterSample {
    let k = Kernels::detect();
    let slots = SLOTS_PER_THREAD * threads.max(1);
    let t0 = Instant::now();
    let dec = StreamDecoder::workers_with::<I>(threads, slots, k);
    let t_new = t0.elapsed();
    let mut out = if collect_out {
        Vec::with_capacity(data.len().saturating_mul(4))
    } else {
        Vec::new()
    };
    let mut t_first = None;
    let mut out_bytes = 0u64;
    let mut h = FastHash::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut pos = 0usize;
            let mut csize = first_chunk;
            while pos < data.len() {
                let end = (pos + csize).min(data.len());
                let mut chunk = dec.spare();
                chunk.clear();
                chunk.extend_from_slice(&data[pos..end]);
                if dec.write(chunk).is_err() {
                    return;
                }
                pos = end;
                csize = (csize * 4).min(steady_chunk);
            }
            dec.close();
        });
        let mut spent = Vec::new();
        while let Some(b) = dec.read_block(spent).expect("read_block") {
            if t_first.is_none() {
                t_first = Some(t0.elapsed());
            }
            out_bytes += b.len() as u64;
            if collect_out {
                out.extend_from_slice(&b);
            } else {
                h.update(&b);
            }
            spent = b;
        }
        dec.cancel();
    });
    let t_eof = t0.elapsed();
    if collect_out {
        h.update(&out);
    }
    let t_before_drop = t0.elapsed();
    drop(dec);
    let t_after_dec_drop = t0.elapsed();
    drop(out);
    let t_total = t0.elapsed();
    let t_first_us = t_first.unwrap_or(t_eof).as_micros() as u64;
    let t_eof_us = t_eof.as_micros() as u64;
    let t_dec_drop_us = t_after_dec_drop.saturating_sub(t_before_drop).as_micros() as u64;
    let t_total_us = t_total.as_micros() as u64;
    OuterSample {
        t_new_us: t_new.as_micros() as u64,
        t_first_us,
        t_body_us: t_eof_us.saturating_sub(t_first_us),
        t_eof_us,
        t_drop_us: t_dec_drop_us,
        t_total_us,
        out_bytes,
        hash: h.finish(),
        vmhwm_kb: vmhwm_kb(),
    }
}

/// Same as `BzReader::on` feeder loop, but with a configurable progressive ramp chunk size
/// (`first_chunk` -> `4x` -> `steady_chunk`) and optional background drop of `StreamDecoder`
/// to test concrete `0`-unsafe candidates.
fn run_bzread_candidate<I: InverseBwt + 'static>(
    path: &Path,
    threads: usize,
    first_chunk: usize,
    steady_chunk: usize,
    use_bufread: bool,
    async_drop: bool,
) -> OuterSample {
    let mut f = File::open(path).expect("open bz2");
    let k = Kernels::detect();
    let slots = SLOTS_PER_THREAD * threads.max(1);
    let t0 = Instant::now();
    let dec = std::sync::Arc::new(StreamDecoder::workers_with::<I>(threads, slots, k));
    let d_feed = dec.clone();
    let feeder = std::thread::Builder::new()
        .name("cand-read".into())
        .spawn(move || {
            let mut target = first_chunk;
            loop {
                let mut buf = d_feed.spare();
                if buf.capacity() < target {
                    buf = vec![0u8; target];
                }
                buf.resize(target, 0);
                let mut n = 0usize;
                while n < buf.len() {
                    match f.read(&mut buf[n..]) {
                        Ok(0) => break,
                        Ok(k) => n += k,
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(_) => {
                            d_feed.cancel();
                            return;
                        }
                    }
                }
                if n == 0 {
                    break;
                }
                buf.truncate(n);
                if d_feed.write(buf).is_err() {
                    return;
                }
                target = (target * 4).min(steady_chunk);
            }
            d_feed.close();
        })
        .expect("spawn feeder");
    let t_new = t0.elapsed();
    let mut t_first = None;
    let mut out_bytes = 0u64;
    let mut h = FastHash::new();
    if use_bufread {
        let mut spent = Vec::new();
        while let Some(b) = dec.read_block(spent).expect("read_block") {
            if t_first.is_none() {
                t_first = Some(t0.elapsed());
            }
            out_bytes += b.len() as u64;
            h.update(&b);
            spent = b;
        }
    } else {
        let mut buf = vec![0u8; 4 << 20];
        loop {
            let n = dec.read(&mut buf).expect("read");
            if n == 0 {
                break;
            }
            if t_first.is_none() {
                t_first = Some(t0.elapsed());
            }
            out_bytes += n as u64;
            h.update(&buf[..n]);
        }
    }
    let t_eof = t0.elapsed();
    if async_drop {
        let _ = std::thread::Builder::new()
            .name("cand-drop".into())
            .spawn(move || {
                dec.cancel();
                let _ = feeder.join();
                drop(dec);
            });
    } else {
        dec.cancel();
        let _ = feeder.join();
        drop(dec);
    }
    let t_total = t0.elapsed();
    let t_first_us = t_first.unwrap_or(t_eof).as_micros() as u64;
    let t_eof_us = t_eof.as_micros() as u64;
    let t_total_us = t_total.as_micros() as u64;
    OuterSample {
        t_new_us: t_new.as_micros() as u64,
        t_first_us,
        t_body_us: t_eof_us.saturating_sub(t_first_us),
        t_eof_us,
        t_drop_us: t_total_us.saturating_sub(t_eof_us),
        t_total_us,
        out_bytes,
        hash: h.finish(),
        vmhwm_kb: vmhwm_kb(),
    }
}

fn run_decode_parallel_direct(data: &[u8], threads: usize) -> OuterSample {
    let t0 = Instant::now();
    let out = mibz2::decode_parallel(data, threads).expect("decode_parallel");
    let t_eof = t0.elapsed();
    let out_bytes = out.len() as u64;
    let mut h = FastHash::new();
    h.update(&out);
    let t_before_drop = t0.elapsed();
    drop(out);
    let t_total = t0.elapsed();
    let t_eof_us = t_eof.as_micros() as u64;
    OuterSample {
        t_new_us: 0,
        t_first_us: 0,
        t_body_us: t_eof_us,
        t_eof_us,
        t_drop_us: t_total.saturating_sub(t_before_drop).as_micros() as u64,
        t_total_us: t_eof_us,
        out_bytes,
        hash: h.finish(),
        vmhwm_kb: vmhwm_kb(),
    }
}

fn print_outer_json(iter: usize, s: &OuterSample) {
    println!(
        "{{\"iter\":{},\"t_new_us\":{},\"t_first_us\":{},\"t_body_us\":{},\"t_eof_us\":{},\"t_drop_us\":{},\"t_total_us\":{},\"out_bytes\":{},\"hash\":\"{:016x}\",\"vmhwm_kb\":{}}}",
        iter,
        s.t_new_us,
        s.t_first_us,
        s.t_body_us,
        s.t_eof_us,
        s.t_drop_us,
        s.t_total_us,
        s.out_bytes,
        s.hash,
        s.vmhwm_kb
    );
}

#[cfg(feature = "trace")]
fn run_child_trace(kernel: &str, threads: usize, path: &Path, api: &str) {
    use mibz2::trace::{self, Kind};
    use std::collections::BTreeMap;

    trace::set_level(2);
    let t0 = Instant::now();
    let sample = match (api, kernel) {
        ("bzread", "cells") => run_bzread_with::<Cells>(path, threads, false, false),
        ("bzread", "open") => run_bzread_with::<OpenCells>(path, threads, false, false),
        ("bzbuf", "cells") => run_bzread_with::<Cells>(path, threads, true, false),
        ("bzbuf", "open") => run_bzread_with::<OpenCells>(path, threads, true, false),
        ("stream_mem", "cells") => {
            let data = std::fs::read(path).expect("read bz2");
            run_stream_mem_with::<Cells>(&data, threads, 4 << 20, 4 << 20, false)
        }
        ("stream_mem", "open") => {
            let data = std::fs::read(path).expect("read bz2");
            run_stream_mem_with::<OpenCells>(&data, threads, 4 << 20, 4 << 20, false)
        }
        _ => panic!("unknown trace api={api} kernel={kernel}"),
    };
    let _ = t0;
    let traces = trace::take();
    trace::set_level(0);

    let mut spawn_live_ns = 0u64;
    let mut first_chunk_ns = 0u64;
    let mut first_block_ns = 0u64;
    let mut last_issued_ns = 0u64;
    let mut scan_busy_ns = 0u64;
    let mut scan_input_wait_ns = 0u64;
    let mut scan_slot_wait_ns = 0u64;
    let mut blocks = 0u64;

    let mut worker_first_start_ns: Vec<u64> = Vec::new();
    let mut worker_busy_sum_ns = 0u64;
    let mut worker_cpu_sum_ns = 0u64;
    let mut worker_runq_sum_ns = 0u64;
    let mut worker_lock_handoff_ns = 0u64;
    let mut worker_idle_steady_ns = 0u64;
    let mut worker_first_blk_ns: Vec<u64> = Vec::new();
    let mut worker_steady_blk_ns: Vec<u64> = Vec::new();
    let mut max_decode_end_ns = 0u64;
    let mut ready_at: BTreeMap<u64, u64> = BTreeMap::new();
    let mut taken_at: BTreeMap<u64, u64> = BTreeMap::new();
    let mut reader_wait_ns = 0u64;
    let mut first_taken_ns = 0u64;
    let mut last_taken_ns = 0u64;

    for tr in &traces {
        if tr.name == "mibz2-scan" {
            let mut input_wait_start = None;
            let mut slot_wait_start = None;
            let mut slot_got_start = None;
            for m in &tr.marks {
                match m.kind {
                    Kind::InputWait => input_wait_start = Some(m.ns),
                    Kind::InputGot => {
                        if first_chunk_ns == 0 {
                            first_chunk_ns = m.ns;
                        }
                        if let Some(s) = input_wait_start.take() {
                            scan_input_wait_ns += m.ns.saturating_sub(s);
                        }
                    }
                    Kind::InputEnd => {
                        if let Some(s) = input_wait_start.take() {
                            scan_input_wait_ns += m.ns.saturating_sub(s);
                        }
                    }
                    Kind::SlotWait => slot_wait_start = Some(m.ns),
                    Kind::SlotGot => {
                        if let Some(s) = slot_wait_start.take() {
                            scan_slot_wait_ns += m.ns.saturating_sub(s);
                        }
                        slot_got_start = Some(m.ns);
                    }
                    Kind::Issued => {
                        if first_block_ns == 0 {
                            first_block_ns = m.ns;
                        }
                        last_issued_ns = m.ns;
                        blocks += 1;
                        if let Some(s) = slot_got_start.take() {
                            scan_busy_ns += m.ns.saturating_sub(s);
                        }
                    }
                    _ => {}
                }
            }
        } else if tr.name.starts_with("mibz2-") {
            if let Some(first) = tr.marks.first() {
                spawn_live_ns = spawn_live_ns.max(first.ns);
            }
            let mut got_ns = None;
            let mut dec_start: Option<(u64, u64, u64)> = None;
            let mut dec_end_ns = None;
            let mut prev_ready_ns = None;
            let mut blk_idx = 0usize;
            for m in &tr.marks {
                match m.kind {
                    Kind::JobGot => {
                        got_ns = Some(m.ns);
                        if let Some(pr) = prev_ready_ns.take() {
                            worker_idle_steady_ns += m.ns.saturating_sub(pr);
                        }
                    }
                    Kind::DecodeStart => {
                        if blk_idx == 0 {
                            worker_first_start_ns.push(m.ns);
                        }
                        if let Some(g) = got_ns.take() {
                            worker_lock_handoff_ns += m.ns.saturating_sub(g);
                        }
                        dec_start = Some((m.ns, m.cpu_ns, m.wait_ns));
                    }
                    Kind::DecodeEnd => {
                        max_decode_end_ns = max_decode_end_ns.max(m.ns);
                        dec_end_ns = Some(m.ns);
                        if let Some((s_ns, s_cpu, s_wait)) = dec_start.take() {
                            let dt = m.ns.saturating_sub(s_ns);
                            worker_busy_sum_ns += dt;
                            if blk_idx == 0 {
                                worker_first_blk_ns.push(dt);
                            } else {
                                worker_steady_blk_ns.push(dt);
                            }
                            blk_idx += 1;
                            if s_cpu != trace::UNREAD && m.cpu_ns != trace::UNREAD {
                                worker_cpu_sum_ns += m.cpu_ns.saturating_sub(s_cpu);
                            }
                            if s_wait != trace::UNREAD && m.wait_ns != trace::UNREAD {
                                worker_runq_sum_ns += m.wait_ns.saturating_sub(s_wait);
                            }
                        }
                    }
                    Kind::Ready => {
                        if let Some(de) = dec_end_ns.take() {
                            worker_lock_handoff_ns += m.ns.saturating_sub(de);
                        }
                        ready_at.insert(m.place, m.ns);
                        prev_ready_ns = Some(m.ns);
                    }
                    _ => {}
                }
            }
        } else {
            // Main reader thread
            let mut rw_start = None;
            for m in &tr.marks {
                match m.kind {
                    Kind::ReadWait => rw_start = Some(m.ns),
                    Kind::Taken => {
                        if first_taken_ns == 0 {
                            first_taken_ns = m.ns;
                        }
                        last_taken_ns = m.ns;
                        taken_at.insert(m.place, m.ns);
                        if let Some(s) = rw_start.take() {
                            reader_wait_ns += m.ns.saturating_sub(s);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    worker_first_start_ns.sort_unstable();
    let all_active_ns = worker_first_start_ns.last().copied().unwrap_or(0);
    let mut max_hol_ns = 0u64;
    for (place, r_ns) in &ready_at {
        if let Some(&t_ns) = taken_at.get(place) {
            max_hol_ns = max_hol_ns.max(t_ns.saturating_sub(*r_ns));
        }
    }
    let avg_first_blk_us = if worker_first_blk_ns.is_empty() {
        0
    } else {
        (worker_first_blk_ns.iter().sum::<u64>() / worker_first_blk_ns.len() as u64) / 1000
    };
    let avg_steady_blk_us = if worker_steady_blk_ns.is_empty() {
        0
    } else {
        (worker_steady_blk_ns.iter().sum::<u64>() / worker_steady_blk_ns.len() as u64) / 1000
    };
    let drain_straggler_us = max_decode_end_ns.saturating_sub(last_issued_ns) / 1000;
    let drain_eof_us = sample.t_eof_us.saturating_sub(max_decode_end_ns / 1000);
    let steady_wall_us = last_issued_ns.saturating_sub(all_active_ns) / 1000;
    let reader_copy_us = sample.t_eof_us.saturating_sub(reader_wait_ns / 1000);
    let _ = last_taken_ns;

    println!(
        "{{\"api\":\"{}\",\"kernel\":\"{}\",\"t\":{},\"blocks\":{},\"t_new_us\":{},\"t_spawn_live_us\":{},\"t_first_chunk_us\":{},\"t_first_block_us\":{},\"t_all_active_us\":{},\"t_first_taken_us\":{},\"t_steady_wall_us\":{},\"t_drain_straggler_us\":{},\"t_drain_eof_us\":{},\"t_eof_us\":{},\"t_drop_us\":{},\"t_total_us\":{},\"scan_busy_us\":{},\"scan_input_wait_us\":{},\"scan_slot_wait_us\":{},\"w_busy_sum_us\":{},\"w_cpu_sum_us\":{},\"w_runq_sum_us\":{},\"w_handoff_us\":{},\"w_idle_steady_us\":{},\"w_first_blk_us\":{},\"w_steady_blk_us\":{},\"reader_wait_us\":{},\"reader_copy_us\":{},\"max_hol_us\":{},\"vmhwm_kb\":{},\"out_bytes\":{},\"hash\":\"{:016x}\"}}",
        api,
        kernel,
        threads,
        blocks,
        sample.t_new_us,
        spawn_live_ns / 1000,
        first_chunk_ns / 1000,
        first_block_ns / 1000,
        all_active_ns / 1000,
        first_taken_ns / 1000,
        steady_wall_us,
        drain_straggler_us,
        drain_eof_us,
        sample.t_eof_us,
        sample.t_drop_us,
        sample.t_total_us,
        scan_busy_ns / 1000,
        scan_input_wait_ns / 1000,
        scan_slot_wait_ns / 1000,
        worker_busy_sum_ns / 1000,
        worker_cpu_sum_ns / 1000,
        worker_runq_sum_ns / 1000,
        worker_lock_handoff_ns / 1000,
        worker_idle_steady_ns / 1000,
        avg_first_blk_us,
        avg_steady_blk_us,
        reader_wait_ns / 1000,
        reader_copy_us,
        max_hol_ns / 1000,
        sample.vmhwm_kb,
        sample.out_bytes,
        sample.hash
    );
}

#[cfg(not(feature = "trace"))]
fn run_child_trace(_kernel: &str, _threads: usize, _path: &Path, _api: &str) {
    panic!("--child-trace requires --features trace");
}

fn run_new_only(threads: usize, iters: usize) {
    let k = Kernels::detect();
    let slots = SLOTS_PER_THREAD * threads.max(1);
    for iter in 0..iters {
        let t0 = Instant::now();
        let d1 = StreamDecoder::new(threads, slots, k);
        let t_new = t0.elapsed().as_micros() as u64;
        let t1 = Instant::now();
        drop(d1);
        let t_drop1 = t1.elapsed().as_micros() as u64;

        let t2 = Instant::now();
        let d2 = StreamDecoder::workers_with::<OpenCells>(threads, slots, k);
        let t_workers = t2.elapsed().as_micros() as u64;
        let t3 = Instant::now();
        drop(d2);
        let t_drop2 = t3.elapsed().as_micros() as u64;

        println!(
            "{{\"iter\":{},\"t\":{},\"new_us\":{},\"drop_new_us\":{},\"workers_us\":{},\"drop_workers_us\":{}}}",
            iter, threads, t_new, t_drop1, t_workers, t_drop2
        );
    }
}

fn run_child_untraced(args: &[String]) {
    // --child-untraced API KERNEL THREADS PATH ITERS [FIRST_KB STEADY_KB]
    let api = &args[0];
    let kernel = &args[1];
    let threads: usize = args[2].parse().expect("threads");
    let path = PathBuf::from(&args[3]);
    let iters: usize = args[4].parse().expect("iters");
    let first_kb: usize = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(4096);
    let steady_kb: usize = args.get(6).and_then(|s| s.parse().ok()).unwrap_or(4096);

    if api == "new_only" {
        run_new_only(threads, iters);
        return;
    }

    let cached_data = if matches!(
        api.as_str(),
        "decode_parallel" | "stream_mem" | "stream_mem_collect"
    ) {
        Some(std::fs::read(&path).expect("read file"))
    } else {
        None
    };

    for iter in 0..iters {
        let s = match (api.as_str(), kernel.as_str()) {
            ("bzread", "auto") => run_bzread_with::<OpenCells>(&path, threads, false, true),
            ("bzread", "cells") => run_bzread_with::<Cells>(&path, threads, false, false),
            ("bzread", "open") => run_bzread_with::<OpenCells>(&path, threads, false, false),
            ("bzbuf", "auto") => run_bzread_with::<OpenCells>(&path, threads, true, true),
            ("bzbuf", "cells") => run_bzread_with::<Cells>(&path, threads, true, false),
            ("bzbuf", "open") => run_bzread_with::<OpenCells>(&path, threads, true, false),
            ("cand_read", "open") => run_bzread_candidate::<OpenCells>(
                &path,
                threads,
                first_kb << 10,
                steady_kb << 10,
                false,
                false,
            ),
            ("cand_read", "cells") => run_bzread_candidate::<Cells>(
                &path,
                threads,
                first_kb << 10,
                steady_kb << 10,
                false,
                false,
            ),
            ("cand_buf", "open") => run_bzread_candidate::<OpenCells>(
                &path,
                threads,
                first_kb << 10,
                steady_kb << 10,
                true,
                false,
            ),
            ("cand_buf", "cells") => run_bzread_candidate::<Cells>(
                &path,
                threads,
                first_kb << 10,
                steady_kb << 10,
                true,
                false,
            ),
            ("cand_asyncdrop_read", "open") => run_bzread_candidate::<OpenCells>(
                &path,
                threads,
                first_kb << 10,
                steady_kb << 10,
                false,
                true,
            ),
            ("cand_asyncdrop_buf", "open") => run_bzread_candidate::<OpenCells>(
                &path,
                threads,
                first_kb << 10,
                steady_kb << 10,
                true,
                true,
            ),
            ("decode_parallel", _) => {
                run_decode_parallel_direct(cached_data.as_ref().unwrap(), threads)
            }
            ("stream_mem", "open") => run_stream_mem_with::<OpenCells>(
                cached_data.as_ref().unwrap(),
                threads,
                first_kb << 10,
                steady_kb << 10,
                false,
            ),
            ("stream_mem", "cells") => run_stream_mem_with::<Cells>(
                cached_data.as_ref().unwrap(),
                threads,
                first_kb << 10,
                steady_kb << 10,
                false,
            ),
            ("stream_mem_collect", "open") => run_stream_mem_with::<OpenCells>(
                cached_data.as_ref().unwrap(),
                threads,
                first_kb << 10,
                steady_kb << 10,
                true,
            ),
            _ => panic!("unknown api={api} kernel={kernel}"),
        };
        print_outer_json(iter, &s);
    }
}

type PoolJob = (u64, mibz2::block::Symbols);
type PoolJobs = std::sync::Arc<(
    std::sync::Mutex<(std::collections::VecDeque<PoolJob>, bool)>,
    std::sync::Condvar,
)>;

/// 100% safe persistent worker pool reusing worker threads, `Scratch<OpenCells>`, symbol buffers,
/// and decoded output buffers across calls (zero per-call thread spawn, zero `Scratch` `mmap`/
/// page-faults, zero `munmap`/join teardown between files).
struct WarmPool {
    threads: usize,
    slots: usize,
    jobs: PoolJobs,
    shared: std::sync::Arc<(
        std::sync::Mutex<PoolShared>,
        std::sync::Condvar,
        std::sync::Condvar,
    )>,
    handles: Vec<std::thread::JoinHandle<()>>,
}

struct PoolShared {
    ready: std::collections::BTreeMap<u64, Vec<u8>>,
    free_out: Vec<Vec<u8>>,
    free_syms: Vec<Vec<u16>>,
    issued: u64,
    taken: u64,
    end: Option<u64>,
}

impl WarmPool {
    fn new(threads: usize) -> Self {
        let threads = threads.max(1);
        let slots = SLOTS_PER_THREAD * threads;
        let jobs = std::sync::Arc::new((
            std::sync::Mutex::new((std::collections::VecDeque::new(), false)),
            std::sync::Condvar::new(),
        ));
        let shared = std::sync::Arc::new((
            std::sync::Mutex::new(PoolShared {
                ready: std::collections::BTreeMap::new(),
                free_out: Vec::new(),
                free_syms: Vec::new(),
                issued: 0,
                taken: 0,
                end: None,
            }),
            std::sync::Condvar::new(), // reader cv
            std::sync::Condvar::new(), // scanner cv
        ));
        let k = Kernels::detect();
        let mut handles = Vec::with_capacity(threads);
        for i in 0..threads {
            let jobs = jobs.clone();
            let shared = shared.clone();
            handles.push(
                std::thread::Builder::new()
                    .name(format!("warmpool-{i}"))
                    .spawn(move || {
                        let mut scratch = mibz2::Scratch::<OpenCells>::default();
                        loop {
                            let job = {
                                let (m, cv) = &*jobs;
                                let mut g = m.lock().unwrap();
                                loop {
                                    if let Some(j) = g.0.pop_front() {
                                        break Some(j);
                                    }
                                    if g.1 {
                                        break None;
                                    }
                                    g = cv.wait(g).unwrap();
                                }
                            };
                            let Some((place, syms)) = job else {
                                return;
                            };
                            let mut out = {
                                let mut st = shared.0.lock().unwrap();
                                st.free_out.pop().unwrap_or_default()
                            };
                            out.clear();
                            mibz2::decode_block(&syms, k, &mut scratch, &mut out)
                                .expect("decode_block");
                            let mut st = shared.0.lock().unwrap();
                            if syms.syms.capacity() > 0 && st.free_syms.len() < slots {
                                st.free_syms.push(syms.syms);
                            }
                            st.ready.insert(place, out);
                            let wake = place == st.taken;
                            drop(st);
                            if wake {
                                shared.1.notify_one();
                            }
                        }
                    })
                    .expect("spawn warmpool worker"),
            );
        }
        Self {
            threads,
            slots,
            jobs,
            shared,
            handles,
        }
    }

    fn decode_slice(&self, data: &[u8]) -> (u64, u64, u64) {
        let t0 = Instant::now();
        {
            let mut st = self.shared.0.lock().unwrap();
            st.ready.clear();
            st.issued = 0;
            st.taken = 0;
            st.end = None;
        }
        let slots = self.slots as u64;
        let jobs = self.jobs.clone();
        let shared = self.shared.clone();
        let mut out_bytes = 0u64;
        let mut h = FastHash::new();
        std::thread::scope(|s| {
            s.spawn(move || {
                let mut sc = mibz2::stream::Scanner::new(mibz2::stream::SliceSource(data));
                loop {
                    let syms = {
                        let mut st = shared.0.lock().unwrap();
                        while st.issued - st.taken >= slots {
                            st = shared.2.wait(st).unwrap();
                        }
                        st.free_syms.pop().unwrap_or_default()
                    };
                    match sc.next(syms).expect("scan") {
                        mibz2::stream::Event::Block { symbols, .. } => {
                            let place = {
                                let mut st = shared.0.lock().unwrap();
                                let p = st.issued;
                                st.issued += 1;
                                p
                            };
                            let (m, cv) = &*jobs;
                            m.lock().unwrap().0.push_back((place, symbols));
                            cv.notify_one();
                        }
                        mibz2::stream::Event::Done => {
                            let mut st = shared.0.lock().unwrap();
                            st.end = Some(st.issued);
                            drop(st);
                            shared.1.notify_one();
                            return;
                        }
                    }
                }
            });
            loop {
                let blk = {
                    let mut st = self.shared.0.lock().unwrap();
                    loop {
                        let taken = st.taken;
                        if let Some(v) = st.ready.remove(&taken) {
                            st.taken += 1;
                            drop(st);
                            self.shared.2.notify_one();
                            break Some(v);
                        }
                        if st.end == Some(taken) {
                            break None;
                        }
                        st = self.shared.1.wait(st).unwrap();
                    }
                };
                let Some(v) = blk else {
                    break;
                };
                out_bytes += v.len() as u64;
                h.update(&v);
                let mut st = self.shared.0.lock().unwrap();
                if st.free_out.len() < self.threads + 4 {
                    st.free_out.push(v);
                }
            }
        });
        (t0.elapsed().as_micros() as u64, out_bytes, h.finish())
    }
}

impl Drop for WarmPool {
    fn drop(&mut self) {
        {
            let (m, cv) = &*self.jobs;
            m.lock().unwrap().1 = true;
            cv.notify_all();
        }
        for h in self.handles.drain(..) {
            let _ = h.join();
        }
    }
}

fn run_pool_ab(mode: &str, threads: usize, path: &Path, iters: usize) {
    let data = std::fs::read(path).expect("read bz2");
    match mode {
        "warmpool" => {
            let pool = WarmPool::new(threads);
            for iter in 0..iters {
                let (us, out_bytes, hash) = pool.decode_slice(&data);
                println!(
                    "{{\"mode\":\"warmpool\",\"iter\":{},\"t_total_us\":{},\"out_bytes\":{},\"hash\":\"{:016x}\",\"vmhwm_kb\":{}}}",
                    iter, us, out_bytes, hash, vmhwm_kb()
                );
            }
        }
        "percall" => {
            for iter in 0..iters {
                let t0 = Instant::now();
                let pool = WarmPool::new(threads);
                let (_, out_bytes, hash) = pool.decode_slice(&data);
                drop(pool);
                let us = t0.elapsed().as_micros() as u64;
                println!(
                    "{{\"mode\":\"percall\",\"iter\":{},\"t_total_us\":{},\"out_bytes\":{},\"hash\":\"{:016x}\",\"vmhwm_kb\":{}}}",
                    iter, us, out_bytes, hash, vmhwm_kb()
                );
            }
        }
        "stream" => {
            for iter in 0..iters {
                let s = run_stream_mem_with::<OpenCells>(&data, threads, 4 << 20, 4 << 20, false);
                println!(
                    "{{\"mode\":\"stream\",\"iter\":{},\"t_total_us\":{},\"out_bytes\":{},\"hash\":\"{:016x}\",\"vmhwm_kb\":{}}}",
                    iter, s.t_total_us, s.out_bytes, s.hash, s.vmhwm_kb
                );
            }
        }
        _ => panic!("unknown pool-ab mode: {mode}"),
    }
}

fn spawn_child_lines(args: &[&str]) -> Vec<String> {
    let exe = std::env::current_exe().expect("current_exe");
    let out = Command::new(exe).args(args).output().expect("spawn child");
    assert!(
        out.status.success(),
        "child failed {:?}: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "usage: parallel_lifecycle_probe --gen DIR [SCALE] | --child-untraced API KERNEL T PATH ITERS [FIRST_KB STEADY_KB] | --child-trace KERNEL T PATH API | --pool-ab MODE T PATH ITERS | --spawn-cold ARGS..."
        );
        std::process::exit(2);
    }
    match args[0].as_str() {
        "--gen" => {
            let dir = Path::new(&args[1]);
            let scale: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
            gen_corpora(dir, scale);
        }
        "--child-untraced" => run_child_untraced(&args[1..]),
        "--child-trace" => {
            run_child_trace(
                &args[1],
                args[2].parse().unwrap(),
                Path::new(&args[3]),
                &args[4],
            );
        }
        "--pool-ab" => {
            run_pool_ab(
                &args[1],
                args[2].parse().unwrap(),
                Path::new(&args[3]),
                args[4].parse().unwrap(),
            );
        }
        "--spawn-cold" => {
            let refs: Vec<&str> = args[1..].iter().map(String::as_str).collect();
            for line in spawn_child_lines(&refs) {
                println!("{line}");
            }
        }
        other => panic!("unknown flag: {other}"),
    }
}
