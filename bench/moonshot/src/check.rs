//! The correctness gate: the candidate byte for byte against libbzip2 (tests/common's oracle),
//! with every block's CRC and every stream's combined CRC checked again here, apart from the
//! decoder's own checks, on:
//!
//! - tests/common's samples (text, runs, random, few symbols, periodic, all bytes) at levels
//!   1 and 9, and the crate's own corpora (text, random, one byte, a pattern, runs, records);
//! - crafted blocks no encoder writes (tests/decode.rs' generator): random columns, whose LF
//!   lists are several cycles, and periodic columns (a string repeated: one cycle repeated),
//!   any primary index, short and full blocks, and tests/common/cells.rs' witnesses;
//! - multi-stream and mixed-level inputs (tests/common's pipeline corpus, rebuilt here);
//! - the Wikidata bench corpus (corpora/wikidata.bz2) when it is given;
//! - mutated inputs (the fuzz loop's edits): the candidate's result must be the shipped
//!   kernel's exactly, its bytes or its error, and never a panic.
//!
//! Each valid input is decoded sequentially (`decode_block`, on both kernel sets) and by the
//! shipped workers with the candidate (`parallel_workers_with`) at 1, 2 and every CPU. Any
//! difference fails the gate.
use crate::common::{self, craft, reference, Rng, Spec};
use mibz2::block::Symbols;
use mibz2::bwt::{InverseBwt, LazyCells};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{decode_block, BzReader, Kernels, Scratch};
use std::io::Read;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// The stream end marker's magic (sqrt(pi))
const END_MAGIC: u64 = 0x1772_4538_5090;

/// `n` bits (at most 57) of `data` from bit `at`, most significant first; None past the end
pub fn bits(data: &[u8], at: u64, n: u32) -> Option<u64> {
    let mut v = 0u64;
    for i in 0..n as u64 {
        let b = at + i;
        let byte = *data.get((b / 8) as usize)?;
        v = v << 1 | u64::from(byte >> (7 - b % 8) & 1);
    }
    Some(v)
}

/// The stream end at bit `at` holds the end marker and `combined`
fn check_end(data: &[u8], at: u64, combined: u32) -> Result<(), String> {
    let magic = bits(data, at, 48).ok_or("STREAM END MISSING")?;
    let stored = bits(data, at + 48, 32).ok_or("STREAM CRC MISSING")? as u32;
    if magic != END_MAGIC {
        return Err(format!("STREAM END MAGIC {magic:#x} at bit {at}"));
    }
    if stored != combined {
        return Err(format!(
            "STREAM CRC MISMATCH: stored {stored:#010x}, the output's {combined:#010x}"
        ));
    }
    Ok(())
}

/// The decoder's error as its kind's name (tests/common's `kind_name`)
fn kind(e: &mibz2::ErrorKind) -> String {
    format!("{e:?}")
        .split([' ', '{', '('])
        .next()
        .unwrap_or("")
        .to_string()
}

/// A whole input decoded on this thread by `K`, block by block, each block's CRC recomputed
/// here (the bytewise reference CRC) against the CRC it stores, and each stream's combined
/// CRC against its end marker's. Err: the decoder's error kind, or a CRC mismatch here
/// (prefixed "CRC").
pub fn sequential<K: InverseBwt>(data: &[u8], k: Kernels) -> Result<Vec<u8>, String> {
    let mut sc = Scanner::new(SliceSource(data));
    let mut out = Vec::new();
    let mut s = Scratch::<K>::default();
    let mut syms = Vec::new();
    // the current stream: its index, its last block's end bit, its combined CRC so far
    let mut cur: Option<(u64, u64, u32)> = None;
    loop {
        match sc.next(syms).map_err(|e| kind(&e.kind))? {
            Event::Block {
                index,
                end,
                stream,
                symbols,
                ..
            } => {
                let mut combined = 0;
                if let Some((st, last_end, c)) = cur {
                    if st == stream {
                        combined = c;
                    } else {
                        check_end(data, last_end, c)?;
                    }
                }
                let start = out.len();
                let crc = decode_block(&symbols, k, &mut s, &mut out).map_err(|e| kind(&e))?;
                let mine = common::crc32(&out[start..]);
                if mine != symbols.crc || crc != mine {
                    return Err(format!(
                        "CRC MISMATCH block {index}: stored {:#010x}, the output's {mine:#010x}",
                        symbols.crc
                    ));
                }
                cur = Some((stream, end, combined.rotate_left(1) ^ mine));
                syms = symbols.syms;
            }
            Event::Done => {
                if let Some((_, last_end, c)) = cur {
                    check_end(data, last_end, c)?;
                }
                return Ok(out);
            }
        }
    }
}

/// A whole input through the shipped workers (`workers` of them, `4 x workers` blocks in
/// flight) inverting with `K`, read in odd-sized pieces
pub fn parallel<K: InverseBwt + 'static>(
    data: &[u8],
    workers: usize,
    k: Kernels,
) -> Result<Vec<u8>, String> {
    let mut r = BzReader::parallel_workers_with::<K>(
        std::io::Cursor::new(data.to_vec()),
        workers,
        4 * workers,
        k,
    );
    let mut out = Vec::new();
    let mut buf = vec![0u8; 1 << 16 | 77];
    loop {
        match r.read(&mut buf) {
            Ok(0) => return Ok(out),
            Ok(n) => out.extend_from_slice(&buf[..n]),
            Err(e) => return Err(format!("{:?}", e.kind())),
        }
    }
}

/// One input's verdict
pub struct Case {
    pub name: String,
    pub bytes: usize,
    pub failure: Option<String>,
}

fn caught<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(f)).map_err(|_| "PANIC".to_string())
}

/// A valid or invalid input: against libbzip2 where it decodes, else against the shipped
/// kernel's error, sequentially on both kernel sets and by the workers at `counts`
fn one<K: InverseBwt + 'static>(name: &str, data: &[u8], counts: &[usize]) -> Case {
    let oracle = common::oracle(data);
    let k = Kernels::detect();
    let mut failure = None;
    let mut fail = |what: String| {
        if failure.is_none() {
            failure = Some(what);
        }
    };
    match &oracle {
        Ok(want) => {
            for (label, kk) in [("detected", k), ("scalar", Kernels::scalar())] {
                match caught(|| sequential::<K>(data, kk)) {
                    Ok(Ok(got)) if got == *want => {}
                    Ok(Ok(got)) => fail(format!(
                        "sequential ({label} kernels): {} bytes differ from libbzip2's {} (first at {})",
                        got.len(),
                        want.len(),
                        first_difference(&got, want)
                    )),
                    Ok(Err(e)) => fail(format!("sequential ({label} kernels): {e} where libbzip2 decodes")),
                    Err(p) => fail(format!("sequential ({label} kernels): {p}")),
                }
            }
            for &w in counts {
                match caught(|| parallel::<K>(data, w, k)) {
                    Ok(Ok(got)) if got == *want => {}
                    Ok(Ok(got)) => fail(format!(
                        "{w} workers: {} bytes differ from libbzip2's {} (first at {})",
                        got.len(),
                        want.len(),
                        first_difference(&got, want)
                    )),
                    Ok(Err(e)) => fail(format!("{w} workers: {e} where libbzip2 decodes")),
                    Err(p) => fail(format!("{w} workers: {p}")),
                }
            }
        }
        Err(_) => {
            // refused by libbzip2: the candidate must refuse it as the shipped kernel does
            let shipped = caught(|| sequential::<LazyCells>(data, k));
            let got = caught(|| sequential::<K>(data, k));
            if got != shipped {
                fail(format!(
                    "an input libbzip2 refuses: {} where 0.5.6's kernel gives {}",
                    summary(&got),
                    summary(&shipped)
                ));
            }
            for &w in counts {
                if let Ok(Ok(_)) | Err(_) = caught(|| parallel::<K>(data, w, k)) {
                    fail(format!(
                        "{w} workers: bytes or a panic on an input libbzip2 refuses"
                    ));
                }
            }
        }
    }
    Case {
        name: name.to_string(),
        bytes: oracle.map(|v| v.len()).unwrap_or(0),
        failure,
    }
}

fn summary(r: &Result<Result<Vec<u8>, String>, String>) -> String {
    match r {
        Ok(Ok(v)) => format!("{} bytes", v.len()),
        Ok(Err(e)) => format!("error {e}"),
        Err(p) => p.clone(),
    }
}

fn first_difference(a: &[u8], b: &[u8]) -> usize {
    a.iter()
        .zip(b)
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()))
}

/// The crate's own corpora (tests/common's `corpora(scale)`), read from the repository's root
pub fn corpora(root: &str) -> Vec<(&'static str, Vec<u8>)> {
    let mut text = Vec::new();
    for dir in ["src", "tests", "examples"] {
        let mut files: Vec<_> = std::fs::read_dir(format!("{root}/{dir}"))
            .unwrap()
            .flatten()
            .map(|e| e.path())
            .collect();
        files.sort();
        for f in files.into_iter().filter(|f| f.is_file()) {
            text.extend(std::fs::read(f).unwrap());
        }
    }
    text.extend(std::fs::read(format!("{root}/README.md")).unwrap());
    let one = text.clone();
    let mut i = 0;
    while text.len() < 3_000_000 {
        text.extend(format!("\n-- copy {i} --\n").bytes());
        text.extend(&one);
        i += 1;
    }
    let mut rng = Rng(0xfeed_beef_dead_cafe);
    let records: Vec<u8> = (0..600_000u32)
        .flat_map(|i| (i.wrapping_mul(2_654_435_761) >> 12 | i << 20).to_le_bytes())
        .collect();
    let mut runs = Vec::new();
    while runs.len() < 2_500_000 {
        let n = 1 + rng.below(600) as usize;
        runs.extend(std::iter::repeat(b"xyz"[rng.below(3) as usize]).take(n));
    }
    vec![
        ("text", text),
        ("random", rng.bytes(2_000_000)),
        ("one byte", vec![b'a'; 5_000_000]),
        ("pattern", b"0123456".repeat(700_000)),
        ("runs", runs),
        ("records", records),
    ]
}

/// tests/decode.rs' crafted blocks: `cases` columns, random (several cycles) and periodic
/// (one short cycle repeated), of 1 to 900,000 rows, any primary index
pub fn crafted(cases: usize, seed: u64) -> Vec<(String, Vec<u8>)> {
    let mut rng = Rng(seed);
    let mut v = Vec::new();
    for case in 0..cases {
        let n = match case % 6 {
            0 => 1 + rng.below(10) as usize,
            1 => 1 + rng.below(1000) as usize,
            2 => 65_536 + rng.below(100_000) as usize,
            3 => 899_000 + rng.below(1000) as usize,
            _ => 1 + rng.below(300_000) as usize,
        };
        let alphabet = 1 + rng.below(if case % 2 == 0 { 3 } else { 256 });
        let mut ll: Vec<u8> = (0..n).map(|_| (rng.below(alphabet) * 7) as u8).collect();
        let periodic = case % 5 == 0;
        if periodic {
            let p = 1 + rng.below(5) as usize;
            for i in p..n {
                ll[i] = ll[i % p];
            }
        }
        let orig_ptr = rng.below(n as u64) as u32;
        let z = craft(9, &[Spec::of(ll.clone(), orig_ptr)], None);
        let shape = if periodic {
            "periodic"
        } else {
            "random column"
        };
        let ends = if reference(&ll, orig_ptr).is_some() {
            ""
        } else {
            ", ends in a run"
        };
        v.push((format!("crafted {case}: {shape}, {n} rows{ends}"), z));
    }
    // several blocks, each periodic or several cycles, in one stream
    let mut specs = Vec::new();
    for i in 0..4u64 {
        let mut r = Rng(70 + i);
        let n = 200_000 + 50_000 * i as usize;
        let mut ll: Vec<u8> = (0..n).map(|_| b"abcd"[r.below(4) as usize]).collect();
        if i % 2 == 0 {
            for j in 3..n {
                ll[j] = ll[j % 3];
            }
        }
        if reference(&ll, 17).is_some() {
            specs.push(Spec::of(ll, 17));
        }
    }
    v.push((
        "crafted multi-block: periodic and several-cycle columns".into(),
        craft(9, &specs, None),
    ));
    for (i, w) in common::cells::witnesses().into_iter().enumerate() {
        v.push((format!("Cells witness {i}"), w));
    }
    v
}

/// tests/common's pipeline corpus, rebuilt from `corpora`: full blocks at levels 1-9, several
/// streams with an empty one among them
fn streams(corp: &[(&str, Vec<u8>)], rng: &mut Rng) -> Vec<(String, Vec<u8>)> {
    let text = &corp[0].1;
    let mut multi = Vec::new();
    for (i, level) in [9, 1, 9, 2, 1, 9].into_iter().enumerate() {
        let part = &text[i * 300_001..i * 300_001 + 250_000 + i * 97_003];
        multi.extend(common::compress(part, level));
    }
    multi.extend(common::compress(b"", 9));
    multi.extend(common::compress(&rng.bytes(400_000), 3));
    vec![
        (
            "text level 5".into(),
            common::compress(&text[..1_600_000], 5),
        ),
        (
            "records level 4".into(),
            common::compress(&corp[5].1[..1_700_000], 4),
        ),
        ("streams of levels 9, 1, 2, empty, 3".into(), multi),
    ]
}

/// The fuzz loop's edits (examples/fuzz.rs, simplified): a bit flipped, a byte set, the
/// input cut, a range deleted
fn mutate(data: &[u8], rng: &mut Rng) -> Vec<u8> {
    let mut d = data.to_vec();
    if d.is_empty() {
        return d;
    }
    match rng.below(4) {
        0 => {
            let at = rng.below(d.len() as u64 * 8);
            d[(at / 8) as usize] ^= 1 << (at % 8);
        }
        1 => {
            let at = rng.below(d.len() as u64) as usize;
            d[at] = rng.next() as u8;
        }
        2 => d.truncate(rng.below(d.len() as u64) as usize),
        _ => {
            let at = rng.below(d.len() as u64) as usize;
            let n = (1 + rng.below(64) as usize).min(d.len() - at);
            d.drain(at..at + n);
        }
    }
    d
}

/// The gate's options
pub struct Opts {
    pub root: String,
    pub wikidata: Option<String>,
    pub quick: bool,
    pub seed: u64,
}

/// Every case, each reported as one JSON line as it is decided; the failures' count returned
pub fn run<K: InverseBwt + 'static>(o: &Opts) -> usize {
    let cpus = std::thread::available_parallelism().map_or(1, |n| n.get());
    let mut counts = vec![1, 2, cpus];
    counts.dedup();
    let mut rng = Rng(o.seed | 1);
    let mut inputs: Vec<(String, Vec<u8>)> = Vec::new();
    for (name, raw) in common::samples(&mut rng) {
        for level in [1, 9] {
            inputs.push((
                format!("sample {name} level {level}"),
                common::compress(&raw, level),
            ));
        }
    }
    let corp = corpora(&o.root);
    for (name, raw) in &corp {
        inputs.push((format!("corpus {name} level 9"), common::compress(raw, 9)));
    }
    inputs.extend(streams(&corp, &mut rng));
    inputs.extend(crafted(if o.quick { 60 } else { 300 }, 99 ^ o.seed));
    let mut failures = 0;
    let mut report = |c: Case| {
        failures += c.failure.is_some() as usize;
        println!(
            "{{\"case\":{:?},\"bytes\":{},\"pass\":{},\"failure\":{}}}",
            c.name,
            c.bytes,
            c.failure.is_none(),
            c.failure
                .as_ref()
                .map_or("null".into(), |f| format!("{f:?}"))
        );
    };
    for (name, data) in &inputs {
        report(one::<K>(name, data, &counts));
    }
    if let Some(path) = &o.wikidata {
        let data = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
        report(one::<K>(&format!("wikidata {path}"), &data, &counts));
    }
    // mutated inputs: the candidate's result is the shipped kernel's, bytes or error
    let bases: Vec<Vec<u8>> = vec![
        common::compress(&corp[0].1[..300_000], 9),
        common::compress(&corp[4].1[..400_000], 1),
        common::compress(&b"abc".repeat(120_000), 6),
        crafted(4, 7).swap_remove(2).1,
    ];
    let edits = if o.quick { 120 } else { 600 };
    let k = Kernels::detect();
    let mut differ = 0;
    let mut first: Option<String> = None;
    for i in 0..edits {
        let d = mutate(&bases[i % bases.len()], &mut rng);
        let shipped = caught(|| sequential::<LazyCells>(&d, k));
        let got = caught(|| sequential::<K>(&d, k));
        if got != shipped {
            differ += 1;
            first.get_or_insert(format!(
                "edit {i}: {} where 0.5.6's kernel gives {}",
                summary(&got),
                summary(&shipped)
            ));
        }
    }
    report(Case {
        name: format!("{edits} mutated inputs against 0.5.6's kernel"),
        bytes: 0,
        failure: first.map(|f| format!("{differ} differ; the first: {f}")),
    });
    failures
}

/// A block's symbols, written for the proxy's driver (`proxy`): the fields of `Symbols`
pub fn write_symbols(out: &mut Vec<u8>, b: &Symbols) {
    out.extend(b.crc.to_le_bytes());
    out.extend(b.orig_ptr.to_le_bytes());
    out.extend(b.seq_to_unseq);
    out.extend(b.eob.to_le_bytes());
    out.extend((b.max_len as u64).to_le_bytes());
    out.extend((b.syms.len() as u64).to_le_bytes());
    for s in &b.syms {
        out.extend(s.to_le_bytes());
    }
}

/// `write_symbols`' blocks read back
pub fn read_symbols(mut d: &[u8]) -> Vec<Symbols> {
    fn take<'a>(d: &mut &'a [u8], n: usize) -> &'a [u8] {
        let (a, b) = d.split_at(n);
        *d = b;
        a
    }
    let mut v = Vec::new();
    while !d.is_empty() {
        let crc = u32::from_le_bytes(take(&mut d, 4).try_into().unwrap());
        let orig_ptr = u32::from_le_bytes(take(&mut d, 4).try_into().unwrap());
        let seq_to_unseq: [u8; 256] = take(&mut d, 256).try_into().unwrap();
        let eob = u16::from_le_bytes(take(&mut d, 2).try_into().unwrap());
        let max_len = u64::from_le_bytes(take(&mut d, 8).try_into().unwrap()) as usize;
        let n = u64::from_le_bytes(take(&mut d, 8).try_into().unwrap()) as usize;
        let syms = take(&mut d, 2 * n)
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        v.push(Symbols {
            crc,
            orig_ptr,
            seq_to_unseq,
            syms,
            eob,
            max_len,
        });
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_reads_msb_first() {
        assert_eq!(bits(&[0b1010_0000, 0xff], 0, 3), Some(0b101));
        assert_eq!(bits(&[0b1010_0000, 0xff], 6, 4), Some(0b0011));
        assert_eq!(bits(&[0xff], 4, 8), None);
    }

    #[test]
    fn the_gate_passes_the_shipped_kernel_and_its_checks_catch_a_wrong_one() {
        let z = common::compress(&b"hello moonshot ".repeat(50_000), 1);
        let want = common::oracle(&z).unwrap();
        assert!(sequential::<LazyCells>(&z, Kernels::detect()).unwrap() == want);
        assert!(parallel::<LazyCells>(&z, 2, Kernels::detect()).unwrap() == want);
        // a stream CRC broken in the input: the end check here refuses it
        let mut specs = vec![Spec::of(b"banana".to_vec(), 3)];
        specs.push(Spec::of(b"abracadabra".to_vec(), 2));
        let bad = craft(9, &specs, Some(0x1234_5678));
        assert!(sequential::<LazyCells>(&bad, Kernels::detect()).is_err());
        // a kernel that gets a byte wrong fails the gate on every valid input
        let c = one::<Wrong>("wrong", &z, &[1, 2]);
        assert!(c.failure.is_some());
        let c = one::<LazyCells>("right", &z, &[1, 2]);
        assert!(c.failure.is_none(), "{:?}", c.failure);
    }

    #[test]
    fn symbols_round_trip() {
        let z = common::compress(&b"round trip ".repeat(30_000), 1);
        let mut sc = Scanner::new(SliceSource(&z));
        let mut buf = Vec::new();
        let mut n = 0;
        while let Ok(Event::Block { symbols, .. }) = sc.next(Vec::new()) {
            write_symbols(&mut buf, &symbols);
            n += 1;
        }
        let back = read_symbols(&buf);
        assert_eq!(back.len(), n);
        let mut s = Scratch::<LazyCells>::default();
        let mut out = Vec::new();
        for b in &back {
            decode_block(b, Kernels::detect(), &mut s, &mut out).unwrap();
        }
        assert!(out == common::oracle(&z).unwrap());
    }

    /// Segmented with its first output byte changed: a wrong kernel
    #[derive(Default)]
    struct Wrong(mibz2::bwt::Segmented);
    impl InverseBwt for Wrong {
        fn invert(&mut self, ll: &[u8], c: &[u32; 256], p: u32, out: &mut Vec<u8>) {
            self.0.invert(ll, c, p, out);
            if let Some(b) = out.first_mut() {
                *b ^= 1;
            }
        }
    }
}
