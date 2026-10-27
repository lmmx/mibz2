//! bakeoff DECODER THREADS FILE: decode FILE (a .bz2) to stdout.
//!
//! DECODER: bz2k | bunzip | pbz2 | fbz | lbzip2rs | bzip2rs
//! THREADS: 1 uses each crate's single-thread API where it has one; N > 1 its parallel API
//! with N workers (lbzip2rs has no thread count: it takes the CPUs the process may run on,
//! so run.py pins it with taskset).
//! A decode error is printed to stderr and the exit code is 2; a panic exits 101.
use std::fs::File;
use std::io::{self, Read, Write};

const BUF: usize = 4 << 20;

fn copy(mut r: impl Read, w: &mut impl Write) -> io::Result<u64> {
    let mut buf = vec![0u8; BUF];
    let mut n = 0u64;
    loop {
        match r.read(&mut buf) {
            Ok(0) => return Ok(n),
            Ok(k) => {
                w.write_all(&buf[..k])?;
                n += k as u64;
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

/// bz2k (the prototype): its `cat` path (phased, AVX2 MTF/RLE1, PCLMULQDQ CRC, the walk in
/// 12 joined chains), sequential; or the same per-block decode on N scoped threads over the
/// blocks found by a bit scan for the block magic, written in order. bz2k panics on bad input.
mod ours {
    use bz2k::*;
    use std::io::Write;

    fn block(data: &[u8], bit: u64, out: &mut Vec<u8>) -> u32 {
        let mut br = BitReader::new(&data[(bit / 8) as usize..]);
        br.bits((bit % 8) as u32);
        let b = match read_block(&mut br, Vec::new()) {
            Next::Block(b) => b,
            Next::End(_) => unreachable!(),
        };
        let (mut ll, mut tt, mut rle) = (Vec::new(), Vec::new(), Vec::new());
        let mut counts = [0u32; 256];
        simd::mtf(&b, &mut ll, &mut counts);
        build(&ll, &counts, &mut tt);
        do_walk(&tt, b.orig_ptr, &mut rle);
        out.clear();
        let crc = unrle_crc(&rle, out, crc::Kind::Simd);
        assert_eq!(crc, b.crc, "block CRC");
        crc
    }

    /// Every block's start bit and every stream's (first block, end, stored CRC), by a bit
    /// scan for the block and end-of-stream magics (as bz2k's `probe` finds blocks; a false
    /// match, 2^-48 a bit, would fail its block's CRC or panic, never pass silently).
    fn starts(data: &[u8]) -> (Vec<u64>, Vec<(usize, usize, u32)>) {
        let mut v = Vec::new();
        let mut streams = Vec::new();
        let mut first = 0;
        for i in 0..data.len().saturating_sub(10) {
            let w = u64::from_be_bytes(data[i..i + 8].try_into().unwrap());
            for s in 0..8 {
                let m = (w >> (16 - s)) & 0xffff_ffff_ffff;
                if m == 0x3141_5926_5359 {
                    v.push(i as u64 * 8 + s as u64);
                } else if m == 0x1772_4538_5090 {
                    let mut br = BitReader::new(&data[i..]);
                    br.bits(s as u32 + 48);
                    streams.push((first, v.len(), br.bits(32)));
                    first = v.len();
                }
            }
        }
        (v, streams)
    }

    pub fn run(data: &[u8], threads: usize, w: &mut impl Write) {
        if threads == 1 {
            // bz2k's own cat_phased, inlined (Kind::Simd: AVX2 MTF and RLE1, PCLMULQDQ CRC)
            let mut br = BitReader::new(data);
            let (mut syms, mut ll, mut tt, mut rle, mut out) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let mut counts = [0u32; 256];
            while stream_header(&mut br) {
                let mut combined = 0u32;
                loop {
                    match read_block(&mut br, syms) {
                        Next::End(crc) => {
                            assert_eq!(crc, combined, "stream CRC");
                            syms = Vec::new();
                            break;
                        }
                        Next::Block(b) => {
                            simd::mtf(&b, &mut ll, &mut counts);
                            build(&ll, &counts, &mut tt);
                            do_walk(&tt, b.orig_ptr, &mut rle);
                            out.clear();
                            let crc = unrle_crc(&rle, &mut out, crc::Kind::Simd);
                            assert_eq!(crc, b.crc, "block CRC");
                            combined = combined.rotate_left(1) ^ crc;
                            w.write_all(&out).unwrap();
                            syms = b.syms;
                        }
                    }
                }
                br.align_byte();
            }
            return;
        }
        // N threads: the block starts first (a bit scan, ~0.03 s for 64 MB), then batches
        // of 4N blocks decoded by N threads and written in order (no overlap of the write
        // with the next batch: a simple driver, so this is a floor for bz2k's own walk on
        // N threads, not a pipeline).
        let (bits, streams) = starts(data);
        let mut crcs = vec![0u32; bits.len()];
        let batch = 4 * threads;
        let mut outs: Vec<Vec<u8>> = vec![Vec::new(); batch];
        for (g, chunk) in bits.chunks(batch).enumerate() {
            let next = std::sync::atomic::AtomicUsize::new(0);
            let slots: Vec<std::sync::Mutex<&mut Vec<u8>>> = outs.iter_mut().map(std::sync::Mutex::new).collect();
            let got: Vec<std::sync::Mutex<u32>> = (0..chunk.len()).map(|_| std::sync::Mutex::new(0)).collect();
            std::thread::scope(|s| {
                for _ in 0..threads {
                    s.spawn(|| loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= chunk.len() {
                            break;
                        }
                        let mut o = slots[i].lock().unwrap();
                        *got[i].lock().unwrap() = block(data, chunk[i], &mut o);
                    });
                }
            });
            for (i, c) in got.iter().enumerate() {
                crcs[g * batch + i] = *c.lock().unwrap();
            }
            drop(slots);
            for o in &outs[..chunk.len()] {
                w.write_all(o).unwrap();
            }
        }
        for (a, b, stored) in streams {
            let combined = crcs[a..b].iter().fold(0u32, |c, &x| c.rotate_left(1) ^ x);
            assert_eq!(combined, stored, "stream CRC");
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (dec, threads, path) = (args[1].as_str(), args[2].parse::<usize>().unwrap(), &args[3]);
    let stdout = io::stdout();
    let mut w = io::BufWriter::with_capacity(BUF, stdout.lock());
    let r: io::Result<u64> = match dec {
        "bz2k" => {
            std::env::set_var("BZ2K_WALK", "seg");
            let data = std::fs::read(path).unwrap();
            ours::run(&data, threads, &mut w);
            Ok(0)
        }
        "bunzip" => {
            let f = File::open(path).unwrap();
            if threads == 1 { copy(bunzip::BzReader::new(f), &mut w) } else { copy(bunzip::BzReader::parallel(f, threads), &mut w) }
        }
        "pbz2" => {
            let f = File::open(path).unwrap();
            if threads == 1 { copy(pbz2::DecoderReader::new(f), &mut w) } else { copy(pbz2::ParallelDecoder::with_threads(f, threads), &mut w) }
        }
        "fbz" => {
            let opts = fbz::DecodeOptions { format: fbz::DecodeFormat::Bzip2, threads, ..Default::default() };
            match fbz::Reader::open(path, opts) {
                Ok(r) => copy(r, &mut w),
                Err(e) => Err(io::Error::other(e.to_string())),
            }
        }
        "lbzip2rs" => {
            let f = File::open(path).unwrap();
            match lbzip2::reader::StreamingBz2Read::new(f) {
                Ok(r) => copy(r, &mut w),
                Err(e) => Err(io::Error::other(format!("{e:?}"))),
            }
        }
        "bzip2rs" => {
            assert_eq!(threads, 1);
            copy(bzip2::read::MultiBzDecoder::new(File::open(path).unwrap()), &mut w)
        }
        _ => panic!("unknown decoder {dec}"),
    };
    let r = r.and_then(|n| w.flush().map(|_| n));
    if let Err(e) = r {
        eprintln!("decode error: {e}");
        std::process::exit(2);
    }
}
