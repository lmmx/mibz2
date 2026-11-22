//! The plain workers' handoff and output path (the plateau stack): the ready ring, the job
//! queue, the ready blocks' byte cap, each block's own output buffer (inverted into, its
//! RLE1 undone in place), the symbols handed back early, the input chunks recycled, and
//! the whole-block reads (`read_block`, `BzReader`'s `BufRead`).
//!
//! Every decode is checked byte for byte, and every error exactly (its kind, block and
//! bit), against the sequential decoder; every wait is under a watchdog, so a deadlock
//! fails the test instead of hanging it.

use super::*;
use std::io::{BufRead, Read};
use std::time::Duration;

/// How long a decode may take before the test fails
const WATCHDOG: Duration = Duration::from_secs(if cfg!(miri) { 600 } else { 120 });

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn compress(data: &[u8], level: u32) -> Vec<u8> {
    use std::io::Write;
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

/// Text of words (a records'-route block, RLE1 at about its length)
fn words(r: &mut Rng, n: usize) -> Vec<u8> {
    const W: [&str; 12] = [
        "the", "claim", "\"id\":", "Q42", "label", "en", "wiki", "  ", "{", "}", ",", "\n",
    ];
    let mut v = Vec::with_capacity(n + 16);
    while v.len() < n {
        v.extend_from_slice(W[r.below(W.len() as u64) as usize].as_bytes());
        if r.below(3) == 0 {
            v.push(b' ');
        }
    }
    v.truncate(n);
    v
}

/// Whether the tests run at their full size (an optimised build) or a cut-down one (CI's
/// `cargo test`, unoptimised)
const FULL: bool = !cfg!(debug_assertions);

/// Many streams, one after another: tiny ones (a block of a few bytes), and every `every`
/// tiny ones, full blocks of text at levels 1 and 9, a block of one byte (its RLE1 grows it
/// about 50 times: 5 MB out), random bytes (the column's route) and runs; the bytes they
/// decode to
fn mixed(seed: u64, tiny: usize, every: usize) -> (Vec<u8>, Vec<u8>) {
    let mut r = Rng(seed | 1);
    let (mut z, mut plain) = (Vec::new(), Vec::new());
    let mut push = |data: Vec<u8>, level: u32, z: &mut Vec<u8>| {
        z.extend(compress(&data, level));
        plain.extend(data);
    };
    for i in 0..tiny {
        let n = 1 + r.below(40) as usize;
        let data: Vec<u8> = (0..n).map(|j| b'a' + ((i + j) % 26) as u8).collect();
        push(data, 1 + (i % 9) as u32, &mut z);
        let at = i % every;
        match at {
            _ if at == every / 7 => push(words(&mut r, 250_000), 1, &mut z),
            _ if at == every * 2 / 7 => push(words(&mut r, 1_300_000), 9, &mut z),
            _ if at == every * 3 / 7 => push(vec![b'a'; 5_000_000], 9, &mut z),
            _ if at == every * 4 / 7 => {
                push((0..300_000).map(|_| r.next() as u8).collect(), 9, &mut z)
            }
            _ if at == every * 5 / 7 => {
                let mut v = Vec::new();
                while v.len() < 400_000 {
                    let len = 1 + r.below(600) as usize;
                    v.extend(std::iter::repeat(b"xyz"[r.below(3) as usize]).take(len));
                }
                push(v, 9, &mut z)
            }
            _ => {}
        }
    }
    (z, plain)
}

/// `f` on a thread of its own, under the watchdog
fn guarded<T: Send + 'static>(what: &str, f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    let h = std::thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(WATCHDOG) {
        Ok(v) => {
            h.join().unwrap();
            v
        }
        Err(_) => panic!("{what}: watchdog: the decode did not finish"),
    }
}

/// How a test reads the decoder's output
#[derive(Clone, Copy, Debug)]
enum Take {
    /// `read`, into a buffer of this many bytes
    Read(usize),
    /// `read_block`, each block whole
    Blocks,
    /// `read_block`, the reader pausing every few blocks, so that decoded blocks pile up
    /// behind it and the cap binds
    Slow,
    /// `read` of a few bytes, then `read_block`, alternately
    Mixed,
}

/// The decoder's output, written `z` in chunks of `chunk` bytes (from `spare`, as the
/// feeders write), taken as `take` says: the bytes, and the error if any
fn run(dec: StreamDecoder, z: Vec<u8>, chunk: usize, take: Take) -> (Vec<u8>, Option<Error>) {
    let dec = Arc::new(dec);
    let d = dec.clone();
    let feeder = std::thread::spawn(move || {
        for c in z.chunks(chunk) {
            let mut b = d.spare();
            b.clear();
            b.extend_from_slice(c);
            if d.write(b).is_err() {
                return;
            }
        }
        d.close();
    });
    let mut out = Vec::new();
    let mut spent = Vec::new();
    let mut blocks = 0u64;
    let err = loop {
        let step = match take {
            Take::Read(n) => {
                let mut buf = vec![0u8; n];
                dec.read(&mut buf).map(|k| {
                    out.extend_from_slice(&buf[..k]);
                    k > 0
                })
            }
            Take::Mixed if blocks % 2 == 0 => {
                let mut buf = [0u8; 7];
                dec.read(&mut buf).map(|k| {
                    out.extend_from_slice(&buf[..k]);
                    k > 0
                })
            }
            Take::Blocks | Take::Slow | Take::Mixed => {
                if matches!(take, Take::Slow) && blocks % 5 == 4 {
                    std::thread::sleep(Duration::from_millis(3));
                }
                dec.read_block(std::mem::take(&mut spent)).map(|b| match b {
                    Some(b) => {
                        out.extend_from_slice(&b);
                        spent = b;
                        true
                    }
                    None => false,
                })
            }
        };
        blocks += 1;
        match step {
            Ok(true) => {}
            Ok(false) => break None,
            Err(e) => break Some(e),
        }
    };
    dec.cancel();
    feeder.join().unwrap();
    (out, err)
}

fn sequential(z: &[u8]) -> Result<Vec<u8>, Error> {
    crate::decode_with::<crate::bwt::Segmented>(z, Kernels::detect())
}

/// The ready blocks' cap binding, at caps from none (0 bytes: the scanner always keeps to
/// the floor) up to the shipped 12 MiB, at 1 to 4 workers and windows from the workers to 4
/// a worker, on many tiny blocks among full ones and a block that grows to 5 MB: the same
/// bytes as the sequential decoder, no deadlock, and the cap seen to bind
#[test]
fn the_ready_cap_binds_without_deadlock() {
    let (z, plain) = if FULL {
        mixed(0xc0ff_ee11, 160, 50)
    } else {
        mixed(0xc0ff_ee11, 60, 60)
    };
    assert!(sequential(&z).unwrap() == plain);
    let z = Arc::new(z);
    let mut bound = 0u64;
    let mut cell = 0usize;
    for threads in 1..=4usize {
        for slots in [threads, 2 * threads, 4 * threads] {
            for cap in [0usize, 1, 64 << 10, 1 << 20, READY_BYTES] {
                for take in [Take::Slow, Take::Read(1 << 16), Take::Mixed] {
                    // (unoptimised, a ninth of the cells, every cap and reading among them)
                    cell += 1;
                    if !FULL && cell % 9 != 1 && cap != 0 {
                        continue;
                    }
                    let what = format!("{threads} workers, {slots} slots, cap {cap}, {take:?}");
                    let zz = z.as_ref().clone();
                    let (out, err, waits) = guarded(&what, move || {
                        let dec = StreamDecoder::with_inverse_capped::<crate::bwt::OpenCells>(
                            threads,
                            slots,
                            Kernels::detect(),
                            cap,
                        );
                        let shared = dec.shared.clone();
                        let (out, err) = run(dec, zz, 1 << 20, take);
                        let waits = shared.lock().capped_waits;
                        (out, err, waits)
                    });
                    assert!(err.is_none(), "{what}: {err:?}");
                    assert!(out == plain, "{what}: the bytes differ");
                    bound += waits;
                }
            }
        }
    }
    assert!(bound > 0, "the cap never bound");
}

/// Tiny-block-heavy input (a thousand streams of a few bytes each, among full blocks), by
/// every reading route of the default decoder at 1 to 4 threads, and the eager and forced
/// open kernels' workers
#[test]
fn tiny_blocks_by_every_route() {
    let (z, plain) = if FULL {
        mixed(0x7157_0001, 1000, 500)
    } else {
        mixed(0x7157_0001, 300, 300)
    };
    assert!(sequential(&z).unwrap() == plain);
    for threads in 1..=4usize {
        let what = format!("decode_parallel at {threads}");
        let zz = z.clone();
        let got = guarded(&what, move || crate::decode_parallel(&zz, threads));
        assert!(got.unwrap() == plain, "{what}");
        for (i, take) in [Take::Read(1), Take::Read(4093), Take::Blocks, Take::Mixed]
            .into_iter()
            .enumerate()
        {
            let what = format!("{threads} threads, {take:?}, kernel {i}");
            let zz = z.clone();
            let (out, err) = guarded(&what, move || {
                let k = Kernels::detect();
                let dec = match i {
                    0 => StreamDecoder::new(threads, 4 * threads, k),
                    1 => StreamDecoder::workers(threads, 2 * threads, k, false),
                    2 => StreamDecoder::workers_with::<crate::bwt::ForcedOpenCells>(
                        threads, threads, k,
                    ),
                    _ => StreamDecoder::workers(threads, 3 * threads, k, true),
                };
                run(dec, zz, 65_537, take)
            });
            assert!(err.is_none(), "{what}: {err:?}");
            assert!(out == plain, "{what}: the bytes differ");
        }
    }
}

/// `BzReader`'s reads across block and chunk boundaries: `read` with buffers of odd sizes,
/// `fill_buf` and `consume` of odd amounts, the two interleaved, on the parallel and the
/// sequential reader
#[test]
fn output_assembly_across_boundaries() {
    let (z, plain) = mixed(0xa55e_4b1e, 120, 120);
    for threads in [1usize, 2, 3, 4] {
        for (pattern, sizes) in [
            ("read", [1usize, 7, 4096, 65_537, 3 << 20]),
            ("bufread", [1, 13, 900_001, 64 << 10, 5]),
            ("both", [3, 1 << 20, 17, 999_999, 2]),
        ] {
            let what = format!("{threads} threads, {pattern}");
            let zz = z.clone();
            let out = guarded(&what, move || {
                let mut r = if threads == 1 {
                    crate::BzReader::new(std::io::Cursor::new(zz))
                } else {
                    crate::BzReader::parallel(std::io::Cursor::new(zz), threads)
                };
                let mut out = Vec::new();
                let mut i = 0usize;
                loop {
                    let n = sizes[i % sizes.len()];
                    let by_buf = match pattern {
                        "read" => false,
                        "bufread" => true,
                        _ => i % 2 == 1,
                    };
                    i += 1;
                    if by_buf {
                        let b = r.fill_buf().unwrap();
                        if b.is_empty() {
                            break;
                        }
                        let k = n.min(b.len());
                        out.extend_from_slice(&b[..k]);
                        r.consume(k);
                    } else {
                        let mut buf = vec![0u8; n];
                        let k = r.read(&mut buf).unwrap();
                        if k == 0 {
                            break;
                        }
                        out.extend_from_slice(&buf[..k]);
                    }
                }
                // (at the end, both say so again)
                assert!(r.fill_buf().unwrap().is_empty());
                assert_eq!(r.read(&mut [0u8; 8]).unwrap(), 0);
                out
            });
            assert!(out == plain, "{what}: the bytes differ");
        }
    }
}

/// Corrupt streams still fail exactly as before: a bit flipped at spots across the input,
/// and the input cut short, each read by the plain workers (every route of reading) and by
/// `decode_parallel`: the error is the sequential decoder's (kind, block and bit), and the
/// bytes before it are exactly the stream's up to the bad block
#[test]
fn corrupt_streams_fail_as_before() {
    let (good, plain) = mixed(0xbad0_c0de, 40, 40);
    let mut r = Rng(0x5eed);
    let mut cases: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..if FULL { 48 } else { 16 } {
        let mut z = good.clone();
        let at = if i < 8 {
            r.below(64) as usize // (the first stream's header and its first block's)
        } else {
            r.below(z.len() as u64) as usize
        };
        z[at] ^= 1 << r.below(8);
        cases.push((format!("bit flip at byte {at}"), z));
    }
    for cut in [
        10usize,
        1000,
        good.len() / 3,
        good.len() / 2,
        good.len() - 3,
    ] {
        cases.push((format!("cut at {cut}"), good[..cut].to_vec()));
    }
    let mut failed = 0;
    for (what, z) in cases {
        let want = sequential(&z);
        let Err(want) = want else {
            // (a flip the format cannot see: a stream's padding, or an unused bit)
            continue;
        };
        failed += 1;
        // the stream's bytes before the bad block: the blocks the sequential reader read
        let prefix = {
            let mut r = crate::BzReader::new(std::io::Cursor::new(z.clone()));
            let mut out = Vec::new();
            let mut buf = vec![0u8; 1 << 16];
            loop {
                match r.read(&mut buf) {
                    Ok(0) => panic!("{what}: the sequential reader didn't fail"),
                    Ok(n) => out.extend_from_slice(&buf[..n]),
                    Err(_) => break out,
                }
            }
        };
        assert!(plain.starts_with(&prefix), "{what}");
        for threads in [1usize, 2, 4] {
            for take in [Take::Read(1 << 16), Take::Blocks, Take::Slow] {
                let what = format!("{what}, {threads} workers, {take:?}");
                let zz = z.clone();
                let (out, err) = guarded(&what, move || {
                    let dec = StreamDecoder::workers(threads, 4 * threads, Kernels::detect(), true);
                    run(dec, zz, 1 << 20, take)
                });
                assert_eq!(err.as_ref(), Some(&want), "{what}");
                assert!(out == prefix, "{what}: the bytes before the error differ");
            }
            let zz = z.clone();
            let got = guarded(&what, move || crate::decode_parallel(&zz, threads));
            assert_eq!(got.err().as_ref(), Some(&want), "{what}: decode_parallel");
        }
    }
    assert!(
        failed > if FULL { 30 } else { 10 },
        "only {failed} cases failed to decode"
    );
}

/// The ring of ready blocks: places in and out of order, a slot held by another place (its
/// map), replacements, and the bytes counted throughout, against a map
#[test]
fn miri_the_ready_ring() {
    let mut r = Ready::new(3); // (4 slots)
    let mut m: BTreeMap<u64, Res> = BTreeMap::new();
    let mut x = Rng(0x41);
    let check = |r: &Ready, m: &BTreeMap<u64, Res>| {
        assert_eq!(r.len, m.len());
        assert_eq!(r.bytes, m.values().map(Ready::size).sum::<usize>());
        let got: Vec<_> = r.iter().map(|(p, v)| (*p, v.clone())).collect();
        let want: Vec<_> = m.iter().map(|(p, v)| (*p, v.clone())).collect();
        assert_eq!(got, want);
    };
    for step in 0..if cfg!(miri) { 200 } else { 5000 } {
        let place = x.below(12);
        match x.below(3) {
            0 | 1 => {
                let v: Res = if x.below(5) == 0 {
                    Err(Error::new(ErrorKind::BadOrigPtr))
                } else {
                    Ok(vec![step as u8; x.below(9) as usize])
                };
                r.insert(place, v.clone());
                m.insert(place, v);
            }
            _ => assert_eq!(r.remove(&place), m.remove(&place)),
        }
        assert_eq!(r.get(&place), m.get(&place));
        check(&r, &m);
        if step % 97 == 0 {
            r.clear();
            m.clear();
            check(&r, &m);
        }
    }
}

/// The job queue: every job taken once, in order, by workers waiting and not, and each
/// worker ends once the queue is closed and empty
#[test]
fn miri_the_job_queue() {
    let jobs = Arc::new(Jobs::default());
    let n = if cfg!(miri) { 40 } else { 4000 };
    let workers: Vec<_> = (0..3)
        .map(|_| {
            let j = jobs.clone();
            std::thread::spawn(move || {
                let mut got = Vec::new();
                while let Some((place, _, _)) = j.recv() {
                    got.push(place);
                }
                got
            })
        })
        .collect();
    {
        let tx = JobsTx(jobs.clone());
        for place in 0..n {
            let symbols = crate::block::Symbols {
                crc: 0,
                orig_ptr: 0,
                seq_to_unseq: [0; 256],
                syms: vec![],
                eob: 1,
                max_len: 100_000,
            };
            assert!(tx.send_job((place, 0, symbols)));
        }
        // (dropped: the queue closes)
    }
    let mut all: Vec<u64> = Vec::new();
    for w in workers {
        let got = w.join().unwrap();
        assert!(
            got.windows(2).all(|p| p[0] < p[1]),
            "a worker's jobs in order"
        );
        all.extend(got);
    }
    all.sort();
    assert_eq!(all, (0..n).collect::<Vec<_>>());
    assert!(jobs.try_recv().is_none());
}
