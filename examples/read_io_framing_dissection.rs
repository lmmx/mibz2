//! Dissection of `mibz2` public Rust streaming I/O (`src/read.rs`), whole-buffer decode
//! (`src/lib.rs`), multi-stream `.bz2` framing (`src/stream.rs`), and block header/bit-reader
//! overhead (`src/block.rs`, `src/bits.rs`).
//!
//! Smooth-zone `explore` probe: zero edits to `src/`.

use mibz2::bits::Bits;
use mibz2::block::{self, Item, ScanScratch, BLOCK_MAGIC, END_MAGIC};
use mibz2::bwt::Cells;
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode, decode_block, decode_parallel, BzReader, Kernels, Scratch};
use std::io::{self, BufRead, Cursor, Read};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Default)]
struct Timeval {
    tv_sec: i64,
    tv_usec: i64,
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Default)]
struct Rusage {
    ru_utime: Timeval,
    ru_stime: Timeval,
    ru_maxrss: i64,
    ru_ixrss: i64,
    ru_idrss: i64,
    ru_isrss: i64,
    ru_minflt: i64,
    ru_majflt: i64,
    ru_nswap: i64,
    ru_inblock: i64,
    ru_oublock: i64,
    ru_msgsnd: i64,
    ru_msgrcv: i64,
    ru_nsignals: i64,
    ru_nvcsw: i64,
    ru_nivcsw: i64,
}

#[cfg(target_os = "linux")]
extern "C" {
    fn getrusage(who: i32, usage: *mut Rusage) -> i32;
}

#[derive(Clone, Copy, Default)]
struct Usage {
    cpu_us: u64,
    minflt: u64,
    vmhwm_kib: u64,
}

fn read_usage() -> Usage {
    #[cfg(target_os = "linux")]
    {
        let mut r = Rusage::default();
        // SAFETY: `Rusage` matches Linux's `struct rusage` for `RUSAGE_SELF = 0`.
        unsafe { getrusage(0, &mut r) };
        let cpu_us = (r.ru_utime.tv_sec.max(0) as u64) * 1_000_000
            + (r.ru_utime.tv_usec.max(0) as u64)
            + (r.ru_stime.tv_sec.max(0) as u64) * 1_000_000
            + (r.ru_stime.tv_usec.max(0) as u64);
        let mut vmhwm_kib = r.ru_maxrss.max(0) as u64;
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(rest) = line.strip_prefix("VmHWM:") {
                    if let Some(tok) = rest.split_whitespace().next() {
                        if let Ok(v) = tok.parse::<u64>() {
                            vmhwm_kib = v;
                        }
                    }
                }
            }
        }
        Usage {
            cpu_us,
            minflt: r.ru_minflt.max(0) as u64,
            vmhwm_kib,
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        Usage::default()
    }
}

fn fnv1a_update(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

const FNV_INIT: u64 = 0xcbf2_9ce4_8422_2325;

fn hash_bytes(bytes: &[u8]) -> u64 {
    fnv1a_update(FNV_INIT, bytes)
}

/// Counting wrapper around `Read` to audit exact call counts and buffer sizes requested by stdlib.
struct CountingReader<R> {
    inner: R,
    calls: usize,
    min_req: usize,
    max_req: usize,
}

impl<R> CountingReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            calls: 0,
            min_req: usize::MAX,
            max_req: 0,
        }
    }
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        self.min_req = self.min_req.min(buf.len());
        self.max_req = self.max_req.max(buf.len());
        self.inner.read(buf)
    }
}

/// Plain non-`Vec`, non-`io::Sink` `Write` target (like a `File`, socket, or streaming hasher)
/// so `std::io::copy` exercises its real `8 KiB` default copy loop rather than `Vec` specialization.
struct CountingSink {
    bytes: usize,
    writes: usize,
    acc: u64,
}

impl io::Write for CountingSink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        self.bytes += buf.len();
        if let Some(&b) = buf.first() {
            self.acc = self.acc.wrapping_add(b as u64);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Prototype `BzReader::read_to_end` override using `BufRead::fill_buf` + `extend_from_slice`
/// (zero stdlib zero-init, zero `8 KiB` chunk copies, zero `unsafe`).
fn proto_read_to_end_fillbuf<R: BufRead>(
    r: &mut R,
    out: &mut Vec<u8>,
) -> io::Result<(usize, usize, usize)> {
    let start = out.len();
    let mut reallocs = 0usize;
    let mut copied_realloc = 0usize;
    let mut prev_cap = out.capacity();
    let mut prev_ptr = out.as_ptr();
    loop {
        let chunk = r.fill_buf()?;
        if chunk.is_empty() {
            break;
        }
        let n = chunk.len();
        let before_len = out.len();
        out.extend_from_slice(chunk);
        if out.capacity() != prev_cap {
            if prev_cap > 0 && out.as_ptr() != prev_ptr {
                copied_realloc += before_len;
            }
            reallocs += 1;
            prev_cap = out.capacity();
            prev_ptr = out.as_ptr();
        }
        r.consume(n);
    }
    Ok((out.len() - start, reallocs, copied_realloc))
}

/// Prototype sequential `BzReader::read_to_end` decoding directly into caller's `out: &mut Vec<u8>`
/// (eliminates intermediate `Seq.out` copy and stdlib zero-init).
fn proto_seq_direct_read_to_end(
    raw: &[u8],
    out: &mut Vec<u8>,
) -> io::Result<(usize, usize, usize)> {
    let start = out.len();
    let mut sc = Scanner::new(SliceSource(raw));
    let k = Kernels::detect();
    let mut scratch = Scratch::<Cells>::default();
    let mut syms = Vec::new();
    let mut reallocs = 0usize;
    let mut copied_realloc = 0usize;
    let mut prev_cap = out.capacity();
    let mut prev_ptr = out.as_ptr();
    while let Event::Block { symbols, .. } = sc.next(syms).map_err(io::Error::from)? {
        let before_len = out.len();
        decode_block(&symbols, k, &mut scratch, out)
            .map_err(|kind| io::Error::new(io::ErrorKind::InvalidData, format!("{kind}")))?;
        if out.capacity() != prev_cap {
            if prev_cap > 0 && out.as_ptr() != prev_ptr {
                copied_realloc += before_len;
            }
            reallocs += 1;
            prev_cap = out.capacity();
            prev_ptr = out.as_ptr();
        }
        syms = symbols.syms;
    }
    Ok((out.len() - start, reallocs, copied_realloc))
}

/// Instrumented `decode` measuring `realloc` count and bytes copied during `Vec` growth.
fn proto_decode_growth(raw: &[u8], mode: &str) -> (Vec<u8>, usize, usize) {
    let mut sc = Scanner::new(SliceSource(raw));
    let k = Kernels::detect();
    let mut scratch = Scratch::<Cells>::default();
    let mut syms = Vec::new();
    let init_cap = match mode {
        "shipped_0" => 0,
        "init_1x" => raw.len(),
        "init_4x" => raw.len().saturating_mul(4),
        "adaptive" => raw.len().min(4 << 20),
        _ => 0,
    };
    let mut out = Vec::with_capacity(init_cap);
    let mut reallocs = 0usize;
    let mut copied_realloc = 0usize;
    let mut prev_cap = out.capacity();
    let mut prev_ptr = out.as_ptr();
    let mut blocks = 0usize;
    while let Event::Block { end, symbols, .. } = sc.next(syms).expect("decode") {
        let before_len = out.len();
        decode_block(&symbols, k, &mut scratch, &mut out).expect("decode_block");
        blocks += 1;
        if mode == "adaptive" && (blocks == 1 || blocks == 4 || blocks == 16) {
            let in_bytes = ((end / 8) as usize).max(1);
            if in_bytes < raw.len() {
                // Estimate total output from observed expansion ratio + 10% headroom,
                // clamped so adversarial run-only streams never over-reserve absurdly.
                let est = ((out.len() as u128) * (raw.len() as u128) / (in_bytes as u128))
                    .min(512 << 20) as usize;
                let target = est + est / 10;
                if target > out.capacity() {
                    let need = target - out.len();
                    out.reserve_exact(need);
                }
            }
        }
        if out.capacity() != prev_cap {
            if prev_cap > 0 && out.as_ptr() != prev_ptr {
                copied_realloc += before_len;
            }
            reallocs += 1;
            prev_cap = out.capacity();
            prev_ptr = out.as_ptr();
        }
        syms = symbols.syms;
    }
    (out, reallocs, copied_realloc)
}

/// Instrumented `decode_parallel` variants comparing `4x` blind cap vs adaptive cap & chunked-concat.
fn proto_decode_par_growth(raw: &[u8], threads: usize, mode: &str) -> (Vec<u8>, usize, usize) {
    let dec = mibz2::StreamDecoder::new(
        threads,
        mibz2::parallel::SLOTS_PER_THREAD * threads.max(1),
        Kernels::detect(),
    );
    if mode == "chunked_concat" {
        let mut chunks: Vec<Vec<u8>> = Vec::new();
        let mut total = 0usize;
        std::thread::scope(|s| {
            s.spawn(|| {
                for c in raw.chunks(4 << 20) {
                    let mut chunk = dec.spare();
                    chunk.clear();
                    chunk.extend_from_slice(c);
                    if dec.write(chunk).is_err() {
                        return;
                    }
                }
                dec.close();
            });
            loop {
                match dec.read_block(Vec::new()).expect("read_block") {
                    None => break,
                    Some(b) => {
                        total += b.len();
                        chunks.push(b);
                    }
                }
            }
            dec.cancel();
        });
        let mut out = Vec::with_capacity(total);
        for b in chunks {
            out.extend_from_slice(&b);
        }
        return (out, 0, total);
    }

    let init_cap = match mode {
        "shipped_4x" => raw.len().saturating_mul(4),
        "init_1x" => raw.len(),
        "first_512k_1x" => raw.len(),
        _ => raw.len().saturating_mul(4),
    };
    let mut out = Vec::with_capacity(init_cap);
    let mut reallocs = 0usize;
    let mut copied_realloc = 0usize;
    let mut prev_cap = out.capacity();
    let mut prev_ptr = out.as_ptr();
    let first_chunk = if mode == "first_512k_1x" {
        512 << 10
    } else {
        4 << 20
    };
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut off = 0usize;
            let mut first = true;
            while off < raw.len() {
                let sz = if first {
                    first = false;
                    first_chunk
                } else {
                    4 << 20
                };
                let end = (off + sz).min(raw.len());
                let mut chunk = dec.spare();
                chunk.clear();
                chunk.extend_from_slice(&raw[off..end]);
                off = end;
                if dec.write(chunk).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut spent = Vec::new();
        loop {
            match dec.read_block(spent).expect("read_block") {
                None => break,
                Some(b) => {
                    let before_len = out.len();
                    out.extend_from_slice(&b);
                    if out.capacity() != prev_cap {
                        if prev_cap > 0 && out.as_ptr() != prev_ptr {
                            copied_realloc += before_len;
                        }
                        reallocs += 1;
                        prev_cap = out.capacity();
                        prev_ptr = out.as_ptr();
                    }
                    spent = b;
                }
            }
        }
        dec.cancel();
    });
    (out, reallocs, copied_realloc)
}

/// Multi-stream scanner prototype: preserves `syms` across `Item::End` (instead of `src/stream.rs:161`
/// dropping `1.80 MiB` on every stream end!) and optionally decodes every block with `decode_block`.
fn proto_multistream_keep_syms(raw: &[u8], decode_blocks: bool) -> (usize, u64, usize, usize) {
    let mut bit = 0u64;
    let mut scratch = ScanScratch::default();
    let mut dscratch = Scratch::<Cells>::default();
    let k = Kernels::detect();
    let mut syms: Vec<u16> = Vec::new();
    let mut out: Vec<u8> = Vec::new();
    let mut streams = 0usize;
    let mut blocks = 0usize;
    let mut total_out = 0usize;
    let mut h = FNV_INIT;
    let mut first = true;

    loop {
        let byte_at = (bit / 8) as usize;
        if byte_at >= raw.len() {
            assert!(!first, "empty");
            break;
        }
        let hdr = &raw[byte_at..];
        assert!(hdr.len() >= 4 && &hdr[..3] == b"BZh" && (b'1'..=b'9').contains(&hdr[3]));
        let level = (hdr[3] - b'0') as u32;
        bit += 32;
        first = false;
        let mut combined = 0u32;

        loop {
            let base = (bit / 8) as usize;
            let mut br = Bits::at(&raw[base..], bit % 8);
            // Peek 48-bit magic BEFORE passing `syms` into `read_item_with_scratch` so `END_MAGIC`
            // never drops `syms`!
            let magic = (br.bits(24) as u64) << 24 | br.bits(24) as u64;
            if magic == END_MAGIC {
                let stored = br.bits(32);
                assert_eq!(stored, combined, "stream CRC mismatch");
                let used = br.bit_pos() - (bit % 8);
                bit += used;
                bit = bit.div_ceil(8) * 8;
                streams += 1;
                break;
            }
            assert_eq!(magic, BLOCK_MAGIC, "bad block magic");
            let mut br2 = Bits::at(&raw[base..], bit % 8);
            let item = block::read_item_with_scratch(&mut br2, level, syms, &mut scratch)
                .expect("read_item");
            let used = br2.bit_pos() - (bit % 8);
            bit += used;
            match item {
                Item::Block(b) => {
                    combined = combined.rotate_left(1) ^ b.crc;
                    blocks += 1;
                    if decode_blocks {
                        out.clear();
                        decode_block(&b, k, &mut dscratch, &mut out).expect("decode_block");
                        total_out += out.len();
                        h = fnv1a_update(h, &out);
                    }
                    syms = b.syms;
                }
                Item::End(_) => unreachable!(),
            }
        }
    }
    (total_out, h, streams, blocks)
}

/// Shipped multi-stream scan + optional decode using `mibz2::stream::Scanner` (drops `syms` on every `Item::End`).
fn shipped_multistream_scanner(raw: &[u8], decode_blocks: bool) -> (usize, u64, usize, usize) {
    let mut sc = Scanner::new(SliceSource(raw));
    let mut dscratch = Scratch::<Cells>::default();
    let k = Kernels::detect();
    let mut syms: Vec<u16> = Vec::new();
    let mut out: Vec<u8> = Vec::new();
    let mut max_stream = 0u64;
    let mut blocks = 0usize;
    let mut total_out = 0usize;
    let mut h = FNV_INIT;
    while let Event::Block {
        stream, symbols, ..
    } = sc.next(syms).expect("scan")
    {
        max_stream = max_stream.max(stream + 1);
        blocks += 1;
        if decode_blocks {
            out.clear();
            decode_block(&symbols, k, &mut dscratch, &mut out).expect("decode_block");
            total_out += out.len();
            h = fnv1a_update(h, &out);
        }
        syms = symbols.syms;
    }
    (total_out, h, max_stream as usize, blocks)
}

/// Axis D: sub-stage timing of `src/block.rs` framing on any corpus (magic+header+in_use+selectors+codelens vs full `read_item_with_scratch`).
fn axis_d_block_framing(raw: &[u8]) {
    let mut bit = 0u64;
    let mut scratch = ScanScratch::default();
    let mut syms: Vec<u16> = Vec::new();
    let mut blocks = 0usize;
    let mut streams = 0usize;
    let mut total_sel = 0usize;
    let mut total_syms = 0usize;
    let mut total_groups = 0usize;
    let mut header_ns = 0u128;
    let mut full_item_ns = 0u128;
    let mut syms_alloc_ns = 0u128;
    let mut syms_alloc_count = 0usize;

    loop {
        let byte_at = (bit / 8) as usize;
        if byte_at >= raw.len() {
            break;
        }
        let hdr = &raw[byte_at..];
        let level = (hdr[3] - b'0') as u32;
        bit += 32;
        loop {
            let base = (bit / 8) as usize;
            let mut br = Bits::at(&raw[base..], bit % 8);
            let t0 = Instant::now();
            let magic = (br.bits(24) as u64) << 24 | br.bits(24) as u64;
            if magic == END_MAGIC {
                let _ = br.bits(32);
                header_ns += t0.elapsed().as_nanos();
                bit += br.bit_pos() - (bit % 8);
                bit = bit.div_ceil(8) * 8;
                streams += 1;
                // Measure what shipped `Scanner` pays when `syms` is dropped at `Item::End`!
                syms = Vec::new();
                break;
            }
            assert_eq!(magic, BLOCK_MAGIC);
            // Parse just header + in_use + selectors + codelens (to measure framing share vs full read_item)
            let _crc = br.bits(32);
            let _rand = br.bit();
            let _orig = br.bits(24);
            let in_use16 = br.bits(16) as u16;
            let mut n_in_use = 0usize;
            for i in 0..16 {
                if in_use16 & (0x8000 >> i) != 0 {
                    let w = br.bits(16) as u16;
                    n_in_use += w.count_ones() as usize;
                }
            }
            let alpha = n_in_use + 2;
            let n_groups = br.bits(3) as usize;
            let n_sel = br.bits(15) as usize;
            total_groups += n_groups;
            total_sel += n_sel;
            for _ in 0..n_sel {
                while br.bit() {}
            }
            for _ in 0..n_groups {
                let mut curr = br.bits(5) as i32;
                for _ in 0..alpha {
                    while br.bit() {
                        if br.bit() {
                            curr -= 1;
                        } else {
                            curr += 1;
                        }
                    }
                    let _ = curr;
                }
            }
            header_ns += t0.elapsed().as_nanos();

            // Measure pure `syms.reserve(MAX_SYMS)` + fault cost whenever `syms` was dropped
            let ta = Instant::now();
            if syms.capacity() < block::MAX_SYMS {
                syms.reserve(block::MAX_SYMS);
                syms_alloc_count += 1;
            }
            syms_alloc_ns += ta.elapsed().as_nanos();

            // Now measure full shipped `read_item_with_scratch` (includes Huffman table build + decode_runs)
            let mut br2 = Bits::at(&raw[base..], bit % 8);
            let t1 = Instant::now();
            let item = block::read_item_with_scratch(&mut br2, level, syms, &mut scratch)
                .expect("read_item");
            full_item_ns += t1.elapsed().as_nanos();
            bit += br2.bit_pos() - (bit % 8);
            match item {
                Item::Block(b) => {
                    blocks += 1;
                    total_syms += b.syms.len();
                    syms = b.syms;
                }
                Item::End(_) => unreachable!(),
            }
        }
    }
    let full_ms = full_item_ns as f64 / 1e6;
    let hdr_ms = header_ns as f64 / 1e6;
    let alloc_ms = syms_alloc_ns as f64 / 1e6;
    let avg_sel = total_sel as f64 / blocks.max(1) as f64;
    let avg_syms = total_syms as f64 / blocks.max(1) as f64;
    let sel_bound_avg = (avg_sel * 50.0).min(block::MAX_SYMS as f64);
    println!(
        "AXIS_D streams={streams} blocks={blocks} avg_groups={:.2} avg_sel={avg_sel:.1} avg_syms={avg_syms:.0} (sel*50_cap={sel_bound_avg:.0} vs MAX_SYMS={}) | hdr_parse={hdr_ms:.2}ms ({:.1}%) syms_alloc={alloc_ms:.2}ms (n={syms_alloc_count}) full_read_item={full_ms:.2}ms",
        total_groups as f64 / blocks.max(1) as f64,
        block::MAX_SYMS,
        100.0 * hdr_ms / full_ms.max(1e-9),
    );
}

#[derive(Clone, Copy)]
struct SubResult {
    wall_ms: f64,
    cpu_ms: f64,
    vmhwm_kib: u64,
    minflt: u64,
    reallocs: usize,
    copied_bytes: usize,
    out_len: usize,
    cap_final: usize,
    read_calls: usize,
    hash: u64,
}

fn run_sub(mode: &str, path: &Path, threads: usize) -> SubResult {
    let raw = std::fs::read(path).expect("read corpus");
    let u0 = read_usage();
    let t0 = Instant::now();

    let mut reallocs = 0usize;
    let mut copied_bytes = 0usize;
    let mut cap_final = 0usize;
    let mut read_calls = 0usize;
    let mut out_vec: Option<Vec<u8>> = None;
    let mut direct_len_hash: Option<(usize, u64)> = None;

    match mode {
        // Axis A: BzReader::read_to_end & Read chunking
        "A0_r2e_seq" => {
            let mut r = CountingReader::new(BzReader::new(Cursor::new(raw)));
            let mut out = Vec::new();
            r.read_to_end(&mut out).expect("r2e");
            read_calls = r.calls;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A0_r2e_par" => {
            let mut r = CountingReader::new(BzReader::parallel(Cursor::new(raw), threads));
            let mut out = Vec::new();
            r.read_to_end(&mut out).expect("r2e");
            read_calls = r.calls;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A1_iocopy_seq" => {
            let mut r = CountingReader::new(BzReader::new(Cursor::new(raw)));
            let mut out = Vec::new();
            io::copy(&mut r, &mut out).expect("copy");
            read_calls = r.calls;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A1_iocopy_par" => {
            let mut r = CountingReader::new(BzReader::parallel(Cursor::new(raw), threads));
            let mut out = Vec::new();
            io::copy(&mut r, &mut out).expect("copy");
            read_calls = r.calls;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A1b_iocopy_sink_seq" => {
            let mut r = CountingReader::new(BzReader::new(Cursor::new(raw)));
            let mut sink = CountingSink {
                bytes: 0,
                writes: 0,
                acc: 0,
            };
            io::copy(&mut r, &mut sink).expect("copy");
            std::hint::black_box(sink.acc);
            read_calls = r.calls;
            direct_len_hash = Some((sink.bytes, 0));
        }
        "A1b_iocopy_sink_par" => {
            let mut r = CountingReader::new(BzReader::parallel(Cursor::new(raw), threads));
            let mut sink = CountingSink {
                bytes: 0,
                writes: 0,
                acc: 0,
            };
            io::copy(&mut r, &mut sink).expect("copy");
            std::hint::black_box(sink.acc);
            read_calls = r.calls;
            direct_len_hash = Some((sink.bytes, 0));
        }
        "A2_read64k_seq" => {
            let mut r = BzReader::new(Cursor::new(raw));
            let mut buf = vec![0u8; 64 << 10];
            let mut out = Vec::new();
            loop {
                let n = r.read(&mut buf).expect("read");
                read_calls += 1;
                if n == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..n]);
            }
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A2_read64k_par" => {
            let mut r = BzReader::parallel(Cursor::new(raw), threads);
            let mut buf = vec![0u8; 64 << 10];
            let mut out = Vec::new();
            loop {
                let n = r.read(&mut buf).expect("read");
                read_calls += 1;
                if n == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..n]);
            }
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A3_read1m_seq" => {
            let mut r = BzReader::new(Cursor::new(raw));
            let mut buf = vec![0u8; 1 << 20];
            let mut out = Vec::new();
            loop {
                let n = r.read(&mut buf).expect("read");
                read_calls += 1;
                if n == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..n]);
            }
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A3_read1m_par" => {
            let mut r = BzReader::parallel(Cursor::new(raw), threads);
            let mut buf = vec![0u8; 1 << 20];
            let mut out = Vec::new();
            loop {
                let n = r.read(&mut buf).expect("read");
                read_calls += 1;
                if n == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..n]);
            }
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A4_fillbuf_drain_seq" => {
            let mut r = BzReader::new(Cursor::new(raw));
            let mut total = 0usize;
            let mut acc = 0u64;
            loop {
                let chunk = r.fill_buf().expect("fill");
                read_calls += 1;
                if chunk.is_empty() {
                    break;
                }
                let n = chunk.len();
                acc = acc
                    .wrapping_add(chunk[0] as u64)
                    .wrapping_add(chunk[n - 1] as u64);
                total += n;
                r.consume(n);
            }
            std::hint::black_box(acc);
            direct_len_hash = Some((total, 0));
        }
        "A4_fillbuf_drain_par" => {
            let mut r = BzReader::parallel(Cursor::new(raw), threads);
            let mut total = 0usize;
            let mut acc = 0u64;
            loop {
                let chunk = r.fill_buf().expect("fill");
                read_calls += 1;
                if chunk.is_empty() {
                    break;
                }
                let n = chunk.len();
                acc = acc
                    .wrapping_add(chunk[0] as u64)
                    .wrapping_add(chunk[n - 1] as u64);
                total += n;
                r.consume(n);
            }
            std::hint::black_box(acc);
            direct_len_hash = Some((total, 0));
        }
        "A5_proto_r2e_seq" => {
            let mut r = BzReader::new(Cursor::new(raw));
            let mut out = Vec::new();
            let (_len, rc, cb) = proto_read_to_end_fillbuf(&mut r, &mut out).expect("proto");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A5_proto_r2e_par" => {
            let mut r = BzReader::parallel(Cursor::new(raw), threads);
            let mut out = Vec::new();
            let (_len, rc, cb) = proto_read_to_end_fillbuf(&mut r, &mut out).expect("proto");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "A6_proto_seq_direct" => {
            let mut out = Vec::new();
            let (_len, rc, cb) = proto_seq_direct_read_to_end(&raw, &mut out).expect("proto");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }

        // Axis B: decode() & decode_parallel()
        "B0_decode" => {
            let out = decode(&raw).expect("decode");
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B0_decode_inst" => {
            let (out, rc, cb) = proto_decode_growth(&raw, "shipped_0");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B1_decode_1x" => {
            let (out, rc, cb) = proto_decode_growth(&raw, "init_1x");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B1_decode_adapt" => {
            let (out, rc, cb) = proto_decode_growth(&raw, "adaptive");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B0_decode_par" => {
            let out = decode_parallel(&raw, threads).expect("decode_parallel");
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B0_decode_par_inst" => {
            let (out, rc, cb) = proto_decode_par_growth(&raw, threads, "shipped_4x");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B1_decode_par_1x" => {
            let (out, rc, cb) = proto_decode_par_growth(&raw, threads, "init_1x");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B2_decode_par_concat" => {
            let (out, rc, cb) = proto_decode_par_growth(&raw, threads, "chunked_concat");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        "B3_decode_par_512k_1x" => {
            let (out, rc, cb) = proto_decode_par_growth(&raw, threads, "first_512k_1x");
            reallocs = rc;
            copied_bytes = cb;
            cap_final = out.capacity();
            out_vec = Some(out);
        }

        // Axis C: Multi-stream framing & syms reuse
        "C0_scan_shipped" => {
            let (len, h, _st, blks) = shipped_multistream_scanner(&raw, false);
            read_calls = blks;
            direct_len_hash = Some((len, h));
        }
        "C1_scan_keep_syms" => {
            let (len, h, _st, blks) = proto_multistream_keep_syms(&raw, false);
            read_calls = blks;
            direct_len_hash = Some((len, h));
        }
        "C0_dec_shipped" => {
            let (len, h, _st, blks) = shipped_multistream_scanner(&raw, true);
            read_calls = blks;
            direct_len_hash = Some((len, h));
        }
        "C1_dec_keep_syms" => {
            let (len, h, _st, blks) = proto_multistream_keep_syms(&raw, true);
            read_calls = blks;
            direct_len_hash = Some((len, h));
        }
        "C0_par_shipped" => {
            let out = decode_parallel(&raw, threads).expect("decode_parallel");
            cap_final = out.capacity();
            out_vec = Some(out);
        }
        _ => panic!("unknown mode: {mode}"),
    }

    // Stop timing and capture exact VmHWM/minflt/CPU BEFORE computing post-hoc verification hash!
    let wall_ms = t0.elapsed().as_secs_f64() * 1e3;
    let u1 = read_usage();
    let (out_len, hash) = if let Some(v) = out_vec {
        (v.len(), hash_bytes(&v))
    } else {
        direct_len_hash.unwrap()
    };
    SubResult {
        wall_ms,
        cpu_ms: (u1.cpu_us.saturating_sub(u0.cpu_us)) as f64 / 1e3,
        vmhwm_kib: u1.vmhwm_kib,
        minflt: u1.minflt.saturating_sub(u0.minflt),
        reallocs,
        copied_bytes,
        out_len,
        cap_final,
        read_calls,
        hash,
    }
}

fn spawn_sub(exe: &Path, mode: &str, corpus: &Path, threads: usize, pin_cpu0: bool) -> SubResult {
    let output = if pin_cpu0 && cfg!(target_os = "linux") {
        Command::new("taskset")
            .args(["-c", "0"])
            .arg(exe)
            .args(["--sub", mode])
            .arg(corpus)
            .arg(threads.to_string())
            .output()
            .expect("spawn taskset")
    } else {
        Command::new(exe)
            .args(["--sub", mode])
            .arg(corpus)
            .arg(threads.to_string())
            .output()
            .expect("spawn sub")
    };
    assert!(
        output.status.success(),
        "sub {mode} {} failed: {}",
        corpus.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let line = String::from_utf8(output.stdout).expect("utf8");
    let parts: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(parts.len(), 10, "bad sub output: {line:?}");
    SubResult {
        wall_ms: parts[0].parse().unwrap(),
        cpu_ms: parts[1].parse().unwrap(),
        vmhwm_kib: parts[2].parse().unwrap(),
        minflt: parts[3].parse().unwrap(),
        reallocs: parts[4].parse().unwrap(),
        copied_bytes: parts[5].parse().unwrap(),
        out_len: parts[6].parse().unwrap(),
        cap_final: parts[7].parse().unwrap(),
        read_calls: parts[8].parse().unwrap(),
        hash: u64::from_str_radix(parts[9], 16).unwrap(),
    }
}

fn median(vals: &[f64]) -> f64 {
    let mut v = vals.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        0.5 * (v[v.len() / 2 - 1] + v[v.len() / 2])
    }
}

/// Paired geometric mean ratio (`cand / base`) and 95% t-interval (two-sided, df = n-1).
fn paired_ci(base: &[f64], cand: &[f64]) -> (f64, f64, f64) {
    let n = base.len().min(cand.len());
    let logs: Vec<f64> = (0..n).map(|i| (cand[i] / base[i].max(1e-6)).ln()).collect();
    let mean = logs.iter().sum::<f64>() / n as f64;
    if n < 2 {
        let gm = mean.exp();
        return (gm, gm, gm);
    }
    let var = logs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let se = (var / n as f64).sqrt();
    // Critical t_{0.975, df} for common small df
    let tcrit = match n - 1 {
        1 => 12.706,
        2 => 4.303,
        3 => 3.182,
        4 => 2.776,
        5 => 2.571,
        6 => 2.447,
        _ => 2.306,
    };
    (
        mean.exp(),
        (mean - tcrit * se).exp(),
        (mean + tcrit * se).exp(),
    )
}

#[allow(clippy::too_many_arguments)]
fn run_paired_matrix(
    exe: &Path,
    label: &str,
    corpus: &Path,
    threads: usize,
    pin_cpu0: bool,
    modes: &[&str],
    base_mode: &str,
    repeats: usize,
    check_hash: bool,
) {
    println!(
        "\n=== {label} | corpus={} t={threads} pin={pin_cpu0} n={repeats} ===",
        corpus.file_name().unwrap().to_string_lossy()
    );
    let mut results: Vec<Vec<SubResult>> = vec![Vec::new(); modes.len()];
    for rep in 0..repeats {
        let order: Vec<usize> = if rep % 2 == 0 {
            (0..modes.len()).collect()
        } else {
            (0..modes.len()).rev().collect()
        };
        for idx in order {
            let r = spawn_sub(exe, modes[idx], corpus, threads, pin_cpu0);
            results[idx].push(r);
        }
    }
    let base_idx = modes.iter().position(|&m| m == base_mode).unwrap_or(0);
    let want_len = results[base_idx][0].out_len;
    let want_hash = results[base_idx][0].hash;
    let base_wall: Vec<f64> = results[base_idx].iter().map(|r| r.wall_ms).collect();
    let base_cpu: Vec<f64> = results[base_idx].iter().map(|r| r.cpu_ms).collect();

    println!(
        "{:<24} {:>8} {:>18} {:>8} {:>18} {:>8} {:>8} {:>6} {:>7} {:>7} {:>6}",
        "mode",
        "wall_ms",
        "wall_ratio[95%]",
        "cpu_ms",
        "cpu_ratio[95%]",
        "VmHWM_M",
        "minflt",
        "reall",
        "copy_M",
        "cap_M",
        "calls"
    );
    for (i, &m) in modes.iter().enumerate() {
        for r in &results[i] {
            if check_hash {
                assert_eq!(r.out_len, want_len, "len mismatch on {m}");
                if r.hash != 0 {
                    assert_eq!(r.hash, want_hash, "hash mismatch on {m}");
                }
            }
        }
        let ws: Vec<f64> = results[i].iter().map(|r| r.wall_ms).collect();
        let cs: Vec<f64> = results[i].iter().map(|r| r.cpu_ms).collect();
        let rss: Vec<f64> = results[i]
            .iter()
            .map(|r| r.vmhwm_kib as f64 / 1024.0)
            .collect();
        let flt: Vec<f64> = results[i].iter().map(|r| r.minflt as f64).collect();
        let (wg, wl, wh) = paired_ci(&base_wall, &ws);
        let (cg, cl, ch) = paired_ci(&base_cpu, &cs);
        let sample = results[i][0];
        println!(
            "{:<24} {:>8.1} {:>6.3} [{:.3},{:.3}] {:>8.1} {:>6.3} [{:.3},{:.3}] {:>8.1} {:>8.0} {:>6} {:>7.1} {:>7.1} {:>6}",
            m,
            median(&ws),
            wg,
            wl,
            wh,
            median(&cs),
            cg,
            cl,
            ch,
            median(&rss),
            median(&flt),
            sample.reallocs,
            sample.copied_bytes as f64 / (1024.0 * 1024.0),
            sample.cap_final as f64 / (1024.0 * 1024.0),
            sample.read_calls
        );
    }
    use std::io::Write;
    let _ = io::stdout().flush();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 5 && args[1] == "--sub" {
        let mode = &args[2];
        let path = Path::new(&args[3]);
        let threads: usize = args[4].parse().expect("threads");
        let r = run_sub(mode, path, threads);
        println!(
            "{:.4} {:.4} {} {} {} {} {} {} {} {:016x}",
            r.wall_ms,
            r.cpu_ms,
            r.vmhwm_kib,
            r.minflt,
            r.reallocs,
            r.copied_bytes,
            r.out_len,
            r.cap_final,
            r.read_calls,
            r.hash
        );
        return;
    }

    let dir = args
        .get(1)
        .map(Path::new)
        .unwrap_or_else(|| Path::new("/tmp/read-io-corpora"));
    let repeats: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
    let axis_filter = args.get(3).map(String::as_str).unwrap_or("all");
    let exe = std::env::current_exe().expect("current_exe");

    let wiki = dir.join("wikidata.bz2");
    let text = dir.join("text.bz2");
    let rand = dir.join("random.bz2");
    let zeros = dir.join("zeros_100m.bz2");
    let ms50 = dir.join("multistream_wiki_50.bz2");
    let ss50 = dir.join("singlestream_wiki_50.bz2");
    let ms200 = dir.join("multistream_wiki_200.bz2");
    let ss200 = dir.join("singlestream_wiki_200.bz2");
    let mss200 = dir.join("multistream_small_200.bz2");
    let sss200 = dir.join("singlestream_small_200.bz2");

    if axis_filter == "all" || axis_filter == "A" {
        let seq_modes = [
            "A0_r2e_seq",
            "A1_iocopy_seq",
            "A2_read64k_seq",
            "A3_read1m_seq",
            "A4_fillbuf_drain_seq",
            "A5_proto_r2e_seq",
            "A6_proto_seq_direct",
        ];
        let par_modes = [
            "A0_r2e_par",
            "A1_iocopy_par",
            "A2_read64k_par",
            "A3_read1m_par",
            "A4_fillbuf_drain_par",
            "A5_proto_r2e_par",
        ];
        for c in [&wiki, &text, &rand, &zeros] {
            run_paired_matrix(
                &exe,
                "AXIS A (Seq t=1)",
                c,
                1,
                true,
                &seq_modes,
                "A0_r2e_seq",
                repeats,
                true,
            );
            run_paired_matrix(
                &exe,
                "AXIS A (Par t=4)",
                c,
                4,
                false,
                &par_modes,
                "A0_r2e_par",
                repeats,
                true,
            );
        }
    }

    if axis_filter == "all" || axis_filter == "B" {
        let seq_b = [
            "B0_decode",
            "B0_decode_inst",
            "B1_decode_1x",
            "B1_decode_adapt",
        ];
        let par_b = [
            "B0_decode_par",
            "B0_decode_par_inst",
            "B1_decode_par_1x",
            "B2_decode_par_concat",
            "B3_decode_par_512k_1x",
        ];
        for c in [&wiki, &text, &rand, &zeros] {
            run_paired_matrix(
                &exe,
                "AXIS B (decode t=1)",
                c,
                1,
                true,
                &seq_b,
                "B0_decode",
                repeats,
                true,
            );
            run_paired_matrix(
                &exe,
                "AXIS B (decode_parallel t=4)",
                c,
                4,
                false,
                &par_b,
                "B0_decode_par",
                repeats,
                true,
            );
        }
    }

    if axis_filter == "all" || axis_filter == "C" {
        for (name, ms, ss) in [
            ("wiki_50 (50x512K)", &ms50, &ss50),
            ("wiki_200 (200x256K)", &ms200, &ss200),
            ("small_200 (200x32K)", &mss200, &sss200),
        ] {
            run_paired_matrix(
                &exe,
                &format!("AXIS C scan {name} MULTI"),
                ms,
                1,
                true,
                &["C0_scan_shipped", "C1_scan_keep_syms"],
                "C0_scan_shipped",
                repeats,
                false,
            );
            run_paired_matrix(
                &exe,
                &format!("AXIS C dec t=1 {name} MULTI"),
                ms,
                1,
                true,
                &["C0_dec_shipped", "C1_dec_keep_syms"],
                "C0_dec_shipped",
                repeats,
                true,
            );
            run_paired_matrix(
                &exe,
                &format!("AXIS C dec t=1 {name} SINGLE"),
                ss,
                1,
                true,
                &["C0_dec_shipped", "C1_dec_keep_syms"],
                "C0_dec_shipped",
                repeats,
                true,
            );
            run_paired_matrix(
                &exe,
                &format!("AXIS C par t=4 {name} MULTI"),
                ms,
                4,
                false,
                &["C0_par_shipped"],
                "C0_par_shipped",
                repeats,
                true,
            );
            run_paired_matrix(
                &exe,
                &format!("AXIS C par t=4 {name} SINGLE"),
                ss,
                4,
                false,
                &["C0_par_shipped"],
                "C0_par_shipped",
                repeats,
                true,
            );
        }
    }

    if axis_filter == "sink" {
        run_paired_matrix(
            &exe,
            "AXIS A-sink (Seq t=1)",
            &wiki,
            1,
            true,
            &["A1b_iocopy_sink_seq", "A4_fillbuf_drain_seq"],
            "A1b_iocopy_sink_seq",
            repeats,
            true,
        );
        run_paired_matrix(
            &exe,
            "AXIS A-sink (Par t=4)",
            &wiki,
            4,
            false,
            &["A1b_iocopy_sink_par", "A4_fillbuf_drain_par"],
            "A1b_iocopy_sink_par",
            repeats,
            true,
        );
    }

    if axis_filter == "all" || axis_filter == "D" {
        println!("\n=== AXIS D (src/block.rs framing breakdown, pinned CPU 0) ===");
        for c in [&wiki, &text, &rand, &ms200, &mss200] {
            let raw = std::fs::read(c).expect("read");
            print!("{:<26} ", c.file_name().unwrap().to_string_lossy());
            axis_d_block_framing(&raw);
        }
    }
}
