//! The paired workers (construction 2, a design probe) on CI's topology imposed on any
//! machine: every block's bytes as the sequential decoder's, errors in order, cancel and
//! drop with blocks held.
use mibz2::{Kernels, StreamDecoder};
use std::io::Write;

fn z(data: &[u8], level: u32) -> Vec<u8> {
    let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(level));
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

fn text(n: usize, seed: u64) -> Vec<u8> {
    let mut x = seed;
    let words: [&[u8]; 8] = [
        b"claim ",
        b"Q42 ",
        b"{\"id\":",
        b"P31",
        b"\n",
        b"aaaaaaaa",
        b"label ",
        b"en ",
    ];
    let mut v = Vec::with_capacity(n);
    while v.len() < n {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        v.extend_from_slice(words[(x >> 61) as usize]);
        if x >> 58 & 7 == 0 {
            v.push((x >> 40) as u8);
        }
    }
    v.truncate(n);
    v
}

fn run(dec: StreamDecoder, zz: &[u8]) -> Result<Vec<u8>, mibz2::Error> {
    let mut out = Vec::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in zz.chunks(1 << 20) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 1 << 20];
        loop {
            match dec.read(&mut buf) {
                Ok(0) => break Ok(out),
                Ok(n) => out.extend_from_slice(&buf[..n]),
                Err(e) => break Err(e),
            }
        }
    })
}

const CI: &[&[usize]] = &[&[0, 1], &[2, 3]];

fn ci() -> Vec<Vec<usize>> {
    CI.iter().map(|c| c.to_vec()).collect()
}

#[test]
fn paired_decode_equals_sequential() {
    // many full blocks (level 1: 100k a block) and some at level 9, windows tight and wide
    for (len, level, seed) in [
        (3_000_000, 1, 1),
        (2_500_000, 9, 2),
        (777_777, 2, 3),
        (10, 9, 4),
    ] {
        let data = text(len, seed);
        let zz = z(&data, level);
        for (threads, slots) in [(4, 4), (4, 16), (3, 3), (4, 5)] {
            let dec = StreamDecoder::with_pairs_on(threads, slots, Kernels::detect(), &ci());
            assert!(
                run(dec, &zz).unwrap() == data,
                "{len} level {level} t{threads} w{slots}"
            );
        }
    }
}

#[test]
fn paired_errors_in_order() {
    let data = text(1_500_000, 7);
    let mut zz = z(&data, 1);
    // corrupt a byte well inside: some block fails, the bytes before it are exact
    let at = zz.len() / 2;
    zz[at] ^= 0x40;
    let want = mibz2::decode(&zz);
    let dec = StreamDecoder::with_pairs_on(4, 8, Kernels::detect(), &ci());
    let got = run(dec, &zz);
    match (want, got) {
        (Err(a), Err(b)) => assert_eq!((a.kind, a.block), (b.kind, b.block)),
        (a, b) => panic!(
            "sequential {:?} vs paired {:?}",
            a.map(|v| v.len()),
            b.map(|v| v.len())
        ),
    }
}

#[test]
fn paired_cancel_and_drop_with_blocks_held() {
    let data = text(4_000_000, 9);
    let zz = z(&data, 1);
    for _ in 0..20 {
        let dec = StreamDecoder::with_pairs_on(4, 16, Kernels::detect(), &ci());
        std::thread::scope(|s| {
            s.spawn(|| {
                for c in zz.chunks(1 << 18) {
                    if dec.write(c.to_vec()).is_err() {
                        return;
                    }
                }
                dec.close();
            });
            let mut buf = vec![0u8; 1 << 16];
            let _ = dec.read(&mut buf);
            dec.cancel();
        });
        drop(dec);
    }
}
