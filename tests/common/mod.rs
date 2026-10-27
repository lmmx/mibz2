//! Shared by the tests and the fuzz loop: libbzip2 as the oracle and the encoder, the
//! decoders under test, a block writer for streams no encoder writes, and test data.
#![allow(dead_code)]

use bunzip::{Error, Kernels};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// libbzip2 1.0.8 (the bzip2 crate's bundled source) on a whole input, its streams one
/// after another: every stream must end, and bytes after a stream must begin another
/// (this decoder's rule; bzip2 and lbzip2 ignore bytes after a stream that are not one)
pub fn oracle(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(data.len() * 4);
    let mut rest = data;
    let mut first = true;
    loop {
        if rest.is_empty() && !first {
            return Ok(out);
        }
        first = false;
        let mut d = bzip2::Decompress::new(false);
        loop {
            if out.capacity() - out.len() < 1 << 16 {
                out.reserve(1 << 20);
            }
            let before = (d.total_in(), d.total_out());
            match d.decompress_vec(&rest[d.total_in() as usize..], &mut out) {
                Ok(bzip2::Status::StreamEnd) => break,
                Ok(_) => {
                    if (d.total_in(), d.total_out()) == before {
                        return Err("truncated".into());
                    }
                }
                Err(e) => return Err(format!("{e}")),
            }
        }
        rest = &rest[d.total_in() as usize..];
    }
}

pub fn compress(data: &[u8], level: u32) -> Vec<u8> {
    use std::io::Write;
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

/// The sequential decoder, a panic caught and reported as one
pub fn ours(data: &[u8], k: Kernels) -> Result<Result<Vec<u8>, Error>, String> {
    catch_unwind(AssertUnwindSafe(|| bunzip::decode_with::<bunzip::bwt::Segmented>(data, k))).map_err(|_| "PANIC".to_string())
}

/// The parallel decoder, fed `chunk` bytes at a time from another thread, read `read`
/// bytes at a time
pub fn parallel(data: &[u8], threads: usize, slots: usize, chunk: usize, read: usize, k: Kernels) -> Result<Vec<u8>, Error> {
    let dec = bunzip::StreamDecoder::new(threads, slots, k);
    let mut out = Vec::new();
    let res = std::thread::scope(|s| {
        s.spawn(|| {
            for c in data.chunks(chunk.max(1)) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; read.max(1)];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break Ok(()),
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => {
                    dec.cancel();
                    break Err(e);
                }
            }
        }
    });
    res.map(|()| out)
}

pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    pub fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.next() as u8).collect()
    }
}

/// Test data of several shapes (text, runs, random, few symbols, repetitions)
pub fn samples(rng: &mut Rng) -> Vec<(String, Vec<u8>)> {
    let words: Vec<&[u8]> = vec![b"{\"id\":\"Q", b"\",\"labels\":{", b"\"en\":", b"\"value\":\"", b"Douglas Adams", b"},", b"\\u00e9", b"P31", b"\"claims\":[", b" ", b"\n"];
    let mut text = Vec::new();
    while text.len() < 2_500_000 {
        text.extend_from_slice(words[rng.below(words.len() as u64) as usize]);
        if rng.below(7) == 0 {
            text.extend_from_slice(rng.below(1_000_000).to_string().as_bytes());
        }
    }
    let mut runs = Vec::new();
    while runs.len() < 1_200_000 {
        let c = rng.next() as u8;
        let n = [1, 2, 3, 4, 5, 255, 259, 260, 1000, 70_000][rng.below(10) as usize];
        runs.extend(std::iter::repeat(c).take(n));
    }
    let few: Vec<u8> = (0..1_000_000).map(|_| b"ab"[rng.below(2) as usize]).collect();
    vec![
        ("empty".into(), Vec::new()),
        ("one".into(), b"x".to_vec()),
        ("short".into(), b"hello, world\n".to_vec()),
        ("text".into(), text),
        ("runs".into(), runs),
        ("random".into(), rng.bytes(1_100_000)),
        ("few".into(), few),
        ("zeros".into(), vec![0u8; 3_000_000]),
        ("abab".into(), b"ab".repeat(600_000)),
        ("period3".into(), b"abc".repeat(400_000)),
        ("all bytes".into(), (0..=255u8).cycle().take(1_000_000).collect()),
    ]
}

/// MSB-first bit writer
#[derive(Default)]
pub struct BitWriter {
    pub out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl BitWriter {
    pub fn put(&mut self, v: u64, bits: u32) {
        for i in (0..bits).rev() {
            self.acc = self.acc << 1 | (v >> i & 1);
            self.n += 1;
            if self.n == 8 {
                self.out.push(self.acc as u8);
                self.acc = 0;
                self.n = 0;
            }
        }
    }
    pub fn bits(&self) -> u64 {
        self.out.len() as u64 * 8 + self.n as u64
    }
    pub fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.put(0, 8 - self.n);
        }
        self.out
    }
}

/// A block as a test writes it: its last column (the BWT's) and primary index, written
/// with fixed-length codes; `syms` replaces the symbols the column gives (to write runs
/// no encoder writes), `crc` the CRC (None: the right one, by `reference`)
pub struct Spec {
    pub ll: Vec<u8>,
    pub orig_ptr: u32,
    pub crc: Option<u32>,
    pub randomised: bool,
    pub syms: Option<Vec<u16>>,
    /// the bytes in use (None: ll's)
    pub in_use: Option<Vec<u8>>,
}

impl Spec {
    pub fn of(ll: Vec<u8>, orig_ptr: u32) -> Spec {
        Spec { ll, orig_ptr, crc: None, randomised: false, syms: None, in_use: None }
    }
}

/// The MTF/RLE2 symbols of a last column (bzip2's generateMTFValues), the bytes in use
pub fn mtf_symbols(ll: &[u8], used: &[u8]) -> Vec<u16> {
    let mut list: Vec<u8> = used.to_vec();
    let mut syms = Vec::new();
    let mut run = 0u32;
    let flush = |run: &mut u32, syms: &mut Vec<u16>| {
        if *run > 0 {
            let mut z = *run - 1;
            loop {
                syms.push((z & 1) as u16);
                if z < 2 {
                    break;
                }
                z = (z - 2) / 2;
            }
            *run = 0;
        }
    };
    for &c in ll {
        let j = list.iter().position(|&x| x == c).unwrap();
        if j == 0 {
            run += 1;
            continue;
        }
        flush(&mut run, &mut syms);
        list.remove(j);
        list.insert(0, c);
        syms.push(j as u16 + 1);
    }
    flush(&mut run, &mut syms);
    syms.push(used.len() as u16 + 1);
    syms
}

/// The inverse BWT and RLE1 as libbzip2 defines them, written plainly: the bytes a
/// block decodes to (None: a block ending inside a run, which libbzip2 refuses)
pub fn reference(ll: &[u8], orig_ptr: u32) -> Option<Vec<u8>> {
    let n = ll.len();
    if orig_ptr as usize >= n {
        return None;
    }
    let mut counts = [0usize; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    let mut cum = [0usize; 256];
    let mut s = 0;
    for i in 0..256 {
        cum[i] = s;
        s += counts[i];
    }
    let mut tt = vec![0u32; n];
    for (i, &c) in ll.iter().enumerate() {
        tt[cum[c as usize]] |= (i as u32) << 8;
        cum[c as usize] += 1;
    }
    for (k, x) in tt.iter_mut().enumerate() {
        *x |= ll[k] as u32;
    }
    let mut p = tt[orig_ptr as usize] >> 8;
    let mut walked = Vec::with_capacity(n);
    for _ in 0..n {
        let e = tt[p as usize];
        walked.push(e as u8);
        p = e >> 8;
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let c = walked[i];
        if i + 3 < n && walked[i + 1] == c && walked[i + 2] == c && walked[i + 3] == c {
            let extra = *walked.get(i + 4)? as usize;
            out.extend(std::iter::repeat(c).take(4 + extra));
            i += 5;
        } else {
            out.push(c);
            i += 1;
        }
    }
    Some(out)
}

pub fn crc32(data: &[u8]) -> u32 {
    !bunzip::crc::update_bytewise(!0, data)
}

/// A stream of `level` holding these blocks; its combined CRC their CRCs' (or `stream_crc`)
pub fn craft(level: u8, blocks: &[Spec], stream_crc: Option<u32>) -> Vec<u8> {
    let mut w = BitWriter::default();
    for &b in b"BZh" {
        w.put(b as u64, 8);
    }
    w.put((b'0' + level) as u64, 8);
    let mut combined = 0u32;
    for b in blocks {
        let crc = b.crc.unwrap_or_else(|| crc32(&reference(&b.ll, b.orig_ptr).unwrap_or_default()));
        combined = combined.rotate_left(1) ^ crc;
        write_block(&mut w, b, crc);
    }
    w.put(0x1772_4538_5090, 48);
    w.put(stream_crc.unwrap_or(combined) as u64, 32);
    w.finish()
}

fn write_block(w: &mut BitWriter, b: &Spec, crc: u32) {
    let mut used: Vec<u8> = match &b.in_use {
        Some(u) => u.clone(),
        None => {
            let mut seen = [false; 256];
            for &c in &b.ll {
                seen[c as usize] = true;
            }
            (0..=255u8).filter(|&c| seen[c as usize]).collect()
        }
    };
    used.sort_unstable();
    used.dedup();
    let syms = b.syms.clone().unwrap_or_else(|| mtf_symbols(&b.ll, &used));
    let alpha = used.len() + 2;
    let len = (usize::BITS - (alpha - 1).leading_zeros()).max(1);
    w.put(0x3141_5926_5359, 48);
    w.put(crc as u64, 32);
    w.put(b.randomised as u64, 1);
    w.put(b.orig_ptr as u64, 24);
    let mut in16 = 0u64;
    for &c in &used {
        in16 |= 0x8000 >> (c / 16);
    }
    w.put(in16, 16);
    for i in 0..16 {
        if in16 & (0x8000 >> i) != 0 {
            let mut word = 0u64;
            for &c in &used {
                if (c / 16) as u64 == i {
                    word |= 0x8000 >> (c % 16);
                }
            }
            w.put(word, 16);
        }
    }
    let n_sel = syms.len().div_ceil(50).max(1);
    w.put(2, 3);
    w.put(n_sel as u64, 15);
    for _ in 0..n_sel {
        w.put(0, 1);
    }
    for _ in 0..2 {
        w.put(len as u64, 5);
        for _ in 0..alpha {
            w.put(0, 1);
        }
    }
    for &s in &syms {
        w.put(s as u64, len);
    }
}

/// The bit at which each block of a .bz2 starts, and each stream's level, by the
/// decoder's own scanner (with the decoded length of each block, from `decode_blocks`)
pub fn block_starts(data: &[u8]) -> Vec<u64> {
    let mut sc = bunzip::stream::Scanner::new(bunzip::stream::SliceSource(data));
    let mut v = Vec::new();
    let mut syms = Vec::new();
    while let Ok(bunzip::stream::Event::Block { bit, symbols, .. }) = sc.next(syms) {
        v.push(bit);
        syms = symbols.syms;
    }
    v
}

/// Corpora with nothing of Wikidata, each a few blocks long (`scale` times): a text
/// corpus (this crate's own sources and README), random bytes, one byte repeated, a
/// short pattern repeated, runs of random lengths, and binary records
pub fn corpora(scale: usize) -> Vec<(&'static str, Vec<u8>)> {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut text = Vec::new();
    for dir in ["src", "tests", "examples"] {
        let mut files: Vec<_> = std::fs::read_dir(format!("{root}/{dir}")).unwrap().flatten().map(|e| e.path()).collect();
        files.sort();
        for f in files.into_iter().filter(|f| f.is_file()) {
            text.extend(std::fs::read(f).unwrap());
        }
    }
    text.extend(std::fs::read(format!("{root}/README.md")).unwrap());
    let one = text.clone();
    let mut i = 0;
    while text.len() < 3_000_000 * scale {
        text.extend(format!("\n-- copy {i} --\n").bytes());
        text.extend(&one);
        i += 1;
    }
    let mut rng = Rng(0xfeed_beef_dead_cafe);
    let records: Vec<u8> = (0..600_000 * scale as u32).flat_map(|i| (i.wrapping_mul(2_654_435_761) >> 12 | i << 20).to_le_bytes()).collect();
    let mut runs = Vec::new();
    while runs.len() < 2_500_000 * scale {
        let n = 1 + rng.below(600) as usize;
        runs.extend(std::iter::repeat(b"xyz"[rng.below(3) as usize]).take(n));
    }
    vec![
        ("text", text),
        ("random", rng.bytes(2_000_000 * scale)),
        ("one byte", vec![b'a'; 5_000_000 * scale]),
        ("pattern", b"0123456".repeat(700_000 * scale)),
        ("runs", runs),
        ("records", records),
    ]
}


/// A bz2 decoder under test, behind one small interface, so that the harness around it
/// (libbzip2 as the oracle, the fuzz loop, the check of real dump windows) runs on any
/// in-process decoder: this crate's, or another put beside it for comparison
pub trait Candidate {
    fn name(&self) -> String;
    /// A whole .bz2 decoded, or the decoder's error as text
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>, String>;
}

/// This crate's decoder: sequential (`threads` 0) or parallel, on the kernels given
pub struct Bunzip {
    pub kernels: Kernels,
    pub threads: usize,
}

impl Candidate for Bunzip {
    fn name(&self) -> String {
        let k = if self.kernels == Kernels::scalar() { "scalar" } else { "simd" };
        if self.threads == 0 { format!("bunzip {k}") } else { format!("bunzip {k} {}t", self.threads) }
    }
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let r = if self.threads == 0 {
            bunzip::decode_with::<bunzip::bwt::Segmented>(data, self.kernels)
        } else {
            parallel(data, self.threads, 2 * self.threads, 1 << 20, 1 << 20, self.kernels)
        };
        r.map_err(|e| format!("{:?}", e.kind).split([' ', '{', '(']).next().unwrap_or("").to_string())
    }
}

/// The decoders the harness checks: this crate's, sequential on both kernel sets and
/// parallel (another decoder joins here)
pub fn candidates() -> Vec<Box<dyn Candidate>> {
    vec![
        Box::new(Bunzip { kernels: Kernels::scalar(), threads: 0 }),
        Box::new(Bunzip { kernels: Kernels::detect(), threads: 0 }),
        Box::new(Bunzip { kernels: Kernels::detect(), threads: 3 }),
    ]
}

/// A candidate's decode, a panic caught and reported as one
pub fn try_decode(c: &dyn Candidate, data: &[u8]) -> Result<Result<Vec<u8>, String>, String> {
    catch_unwind(AssertUnwindSafe(|| c.decode(data))).map_err(|_| "PANIC".to_string())
}
