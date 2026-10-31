//! The whole decoder with each MTF kernel (`MtfKernel::Shipped`, the default, and
//! `MtfKernel::P74c`): the same bytes as libbzip2 (the oracle) and as each other, every
//! block's and stream's CRC checked by the decoder, through every path wikidata-pq or a
//! caller can take: sequential (`decode_with`, the plain MTF; `Cooperative<2>`, the
//! checkpoint MTF) and the parallel decoder at `Plan::Workers(1, 2, 4)` and
//! `Plan::Teams` (checkpoints). Malformed and truncated streams give identical errors
//! (kind, block, bit) and identical bytes before them under both settings, including the
//! MTF stage's own errors in blocks long enough for P74-C's combined kernel. The default
//! kernels select the shipped kernel.
mod common;
use common::*;
use mibz2::bwt::{Cooperative, Segmented};
use mibz2::mtf::CHECKPOINT;
use mibz2::parallel::Plan;
use mibz2::{Error, Kernels, MtfKernel, StreamDecoder};

/// The kernels under test: each MTF choice, detected and scalar (P74-C without AVX2 is
/// the shipped kernel)
fn kernels() -> Vec<Kernels> {
    let mut v = Vec::new();
    for k in [Kernels::detect(), Kernels::scalar()] {
        v.push(k);
        v.push(k.with_mtf(MtfKernel::P74c));
    }
    v
}

/// The parallel decoder as `plan` says, fed `chunk` bytes at a time from another thread:
/// the bytes read before it stopped, and how it stopped
fn planned(z: &[u8], plan: Plan, chunk: usize, k: Kernels) -> (Vec<u8>, Result<(), Error>) {
    let dec = StreamDecoder::planned(plan, 2 * plan.workers(), k);
    let mut out = Vec::new();
    let res = std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(chunk.max(1)) {
                if dec.write(c.to_vec()).is_err() {
                    return;
                }
            }
            dec.close();
        });
        let mut buf = vec![0u8; 1 << 16];
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
    dec.join();
    (out, res)
}

const PLANS: [Plan; 5] = [
    Plan::Workers(1),
    Plan::Workers(2),
    Plan::Workers(4),
    Plan::Teams { teams: 1, k: 2 },
    Plan::Teams { teams: 2, k: 2 },
];

/// Every path with every kernel: (what, bytes before the end, how it ended), the
/// sequential ones' bytes being all or nothing
fn every_way(z: &[u8]) -> Vec<(String, Vec<u8>, Result<(), Error>)> {
    let mut v = Vec::new();
    for k in kernels() {
        let r = mibz2::decode_with::<Segmented>(z, k);
        v.push((
            format!("{k:?} sequential plain"),
            r.clone().unwrap_or_default(),
            r.map(|_| ()),
        ));
        let r = mibz2::decode_with::<Cooperative<2>>(z, k);
        v.push((
            format!("{k:?} sequential checkpoints"),
            r.clone().unwrap_or_default(),
            r.map(|_| ()),
        ));
        for plan in PLANS {
            for chunk in [1 << 16, 4 << 20] {
                let (out, res) = planned(z, plan, chunk, k);
                v.push((format!("{k:?} {plan:?} chunk {chunk}"), out, res));
            }
        }
    }
    v
}

/// A valid stream: every path's bytes the oracle's
fn valid(z: &[u8], what: &str) {
    let want = oracle(z).unwrap();
    for (how, out, res) in every_way(z) {
        assert_eq!(res, Ok(()), "{what}: {how}");
        assert!(
            out == want,
            "{what}: {how}: the bytes differ from libbzip2's"
        );
    }
}

/// A bad stream: each path's error and bytes the same under both MTF kernels
fn same_errors(z: &[u8], what: &str) -> usize {
    let all = every_way(z);
    let (shipped, p74c): (Vec<_>, Vec<_>) = all
        .into_iter()
        .partition(|(how, _, _)| how.contains("Shipped"));
    assert_eq!(shipped.len(), p74c.len());
    let mut errors = 0;
    for ((h0, o0, r0), (h1, o1, r1)) in shipped.iter().zip(&p74c) {
        assert_eq!(r0, r1, "{what}: {h0} against {h1}");
        assert!(
            o0 == o1,
            "{what}: {h0} against {h1}: the bytes before the error"
        );
        errors += r0.is_err() as usize;
    }
    errors
}

#[test]
fn the_default_selects_the_shipped_kernel() {
    assert_eq!(Kernels::detect().mtf, MtfKernel::Shipped);
    assert_eq!(Kernels::scalar().mtf, MtfKernel::Shipped);
    assert_eq!(MtfKernel::default(), MtfKernel::Shipped);
    let k = Kernels::detect();
    assert_eq!(k.with_mtf(MtfKernel::P74c).with_mtf(MtfKernel::Shipped), k);
    assert_eq!(
        (
            k.with_mtf(MtfKernel::P74c).avx2,
            k.with_mtf(MtfKernel::P74c).clmul
        ),
        (k.avx2, k.clmul)
    );
}

#[test]
fn corpora_through_every_path() {
    for (name, data) in corpora(1) {
        valid(&compress(&data, 9), name);
    }
}

#[test]
fn samples_at_levels_1_and_9() {
    let mut rng = Rng(0x74c0_dec0_0000_0001);
    for (name, data) in samples(&mut rng) {
        for level in [1, 9] {
            valid(&compress(&data, level), &format!("{name}, level {level}"));
        }
    }
}

#[test]
fn several_streams_one_after_another() {
    let mut z = Vec::new();
    for (_, data) in corpora(1).into_iter().take(3) {
        z.extend(compress(&data[..data.len().min(1_500_000)], 9));
    }
    valid(&z, "three streams");
}

#[test]
fn truncated_and_corrupted_streams() {
    let text = &corpora(1)[0].1;
    let z = compress(&text[..2_500_000], 9);
    let mut errors = 0;
    // truncations: in the header, in the first block, at the block boundaries, in the end
    let starts = block_starts(&z);
    let mut cuts = vec![
        0,
        1,
        3,
        4,
        10,
        100,
        z.len() / 3,
        z.len() / 2,
        z.len() - 7,
        z.len() - 1,
    ];
    cuts.extend(starts.iter().map(|&b| (b / 8) as usize));
    cuts.extend(starts.iter().map(|&b| (b / 8) as usize + 1_000));
    for cut in cuts {
        let cut = cut.min(z.len());
        errors += same_errors(&z[..cut], &format!("cut at {cut}"));
    }
    // bit flips through the blocks (the Huffman data, a header, a CRC)
    let mut rng = Rng(0x74c0_dec0_0000_0002);
    for i in 0..24 {
        let mut bad = z.clone();
        let at = if i < 4 {
            4 + i * 3
        } else {
            10 + rng.below(z.len() as u64 - 10) as usize
        };
        bad[at] ^= 1 << (rng.below(8) as u8);
        errors += same_errors(&bad, &format!("flip at byte {at}"));
    }
    // bytes after the last stream
    let mut trailing = z.clone();
    trailing.extend(b"not bzip2");
    errors += same_errors(&trailing, "trailing bytes");
    assert!(errors > 30, "{errors} errors");
}

/// A run of `n` (at least 1) as RUNA/RUNB symbols: bijective base 2, least digit first
fn run(mut n: usize, out: &mut Vec<u16>) {
    while n != 0 {
        out.push(((n - 1) % 2) as u16);
        n = (n - 1) / 2;
    }
}

/// A stream of one block of `syms` (in-use: all 256 bytes, the end 257), its CRC 0
fn crafted(syms: Vec<u16>, level: u8) -> Vec<u8> {
    let mut spec = Spec::of(Vec::new(), 0);
    spec.in_use = Some((0..=255).collect());
    spec.syms = Some(syms);
    spec.crc = Some(0);
    craft(level, &[spec], Some(0))
}

#[test]
fn mtf_errors_in_long_blocks() {
    // Blocks past the bypass (S >= 16384, 2S < N_max), so P74-C's combined kernel runs,
    // with the MTF stage's own errors: a run too long (before and after a guard switch),
    // a block too long by a run and by a byte, each at a checkpoint edge too
    let mut errors = 0;
    for prefix in [16_384, 20_000, 3 * CHECKPOINT + 1] {
        for high in [false, true] {
            // `high`: enough rank-32-or-more tokens that the guard switches first
            let tokens: Vec<u16> = (0..prefix)
                .map(|i| {
                    if high && i % 2 == 0 {
                        200
                    } else {
                        2 + (i % 5) as u16
                    }
                })
                .collect();
            let mut runs_too_long = tokens.clone();
            runs_too_long.extend([1u16; 22]);
            runs_too_long.push(257);
            let mut run_too_long = tokens.clone();
            run_too_long.extend([1u16; 20]); // a run of 2^21 - 2 bytes: past 900k
            run_too_long.extend([3, 257]);
            // a run to 6 bytes short of the limit, then 7 explicit bytes
            let mut bytes_too_long = tokens.clone();
            run(900_000 - 6 - prefix, &mut bytes_too_long);
            bytes_too_long.extend([3u16; 7]);
            bytes_too_long.push(257);
            for (what, syms, level) in [
                ("run weight", runs_too_long, 9),
                ("a run past the block", run_too_long, 9),
                ("bytes past the block", bytes_too_long, 9),
            ] {
                let z = crafted(syms, level);
                let e = same_errors(&z, &format!("{what}, prefix {prefix}, high {high}"));
                assert!(e > 0, "{what}: no error");
                errors += e;
            }
        }
    }
    assert!(errors > 0);
}
