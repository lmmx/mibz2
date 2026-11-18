//! The lazy kernel's wider admission (`LazyCells`: to r/n 0.15 with a budget of 0.32 n cells,
//! where Cells admits to 0.10 with 0.16 n; `just bench-admit`, #556, #145 LOG 6975512468): the
//! blocks it now admits and Cells refuses decode to the same bytes, through the sequential
//! route and production's workers, on every corpus; and corrupt input gives the same error,
//! with the same block and bit, as the Segmented reference.
mod common;
use common::{block_starts, cells, compress, corpora, drive, plain_parallel};
use mibz2::bwt::{Cells, LazyCells, Segmented};
use mibz2::{Kernels, StreamDecoder};

/// Copies of a 60 kB passage of the crate's text, each byte changed with probability 1 in
/// `rate`: `len` bytes. At rate 75 its blocks lie between Cells' density line and the lazy
/// kernel's; at 120 below both, but over Cells' budget and inside the lazy kernel's
fn copies(len: usize, rate: u64) -> Vec<u8> {
    let corpora = corpora(1);
    let text = &corpora.iter().find(|(name, _)| *name == "text").unwrap().1;
    let mut rng = common::Rng(7);
    let mut d = Vec::new();
    while d.len() < len {
        for &b in &text[..60_000] {
            d.push(if rng.below(rate) == 0 {
                b'a' + rng.below(26) as u8
            } else {
                b
            });
        }
    }
    d
}

/// The blocks the lazy kernel newly admits: between the density lines (Cells refuses them
/// for density) and past Cells' budget (Cells gives up on them inside its induction). The
/// same bytes from the lazy kernel, Cells and Segmented, and through lazy workers
#[test]
fn newly_admitted_blocks_decode_the_same() {
    let k = Kernels::detect();
    for (rate, density) in [(75, true), (120, false)] {
        let data = copies(2_000_000, rate);
        let z = compress(&data, 9);
        let (lazy_out, lazy_res, lazy) = cells::decode::<LazyCells>(&z, k);
        let (cells_out, cells_res, cells) = cells::decode::<Cells>(&z, k);
        let (seg_out, seg_res, _) = cells::decode::<Segmented>(&z, k);
        assert!(
            lazy_res.is_ok() && cells_res.is_ok() && seg_res.is_ok(),
            "rate {rate}"
        );
        assert!(
            lazy_out == data && cells_out == data && seg_out == data,
            "rate {rate}"
        );
        let (l, c) = (lazy.cells_counts(), cells.cells_counts());
        // the stratum: blocks the lazy kernel admits and Cells refuses
        assert!(
            l.admitted >= 2 && c.admitted == 0,
            "rate {rate}: lazy {l:?}, Cells {c:?}"
        );
        if density {
            assert!(
                c.density >= l.density + 2,
                "rate {rate}: lazy {l:?}, Cells {c:?}"
            );
        } else {
            assert!(
                c.budget >= 2 && l.budget == 0,
                "rate {rate}: lazy {l:?}, Cells {c:?}"
            );
        }
        for t in [8, 12] {
            let dec = StreamDecoder::workers(t, 2 * t, k, true);
            assert!(dec.is_lazy());
            let (out, res) = drive(&dec, &z, 1 << 20, 1 << 16);
            assert!(
                res.is_ok() && out == data,
                "rate {rate}, {t} lazy workers: {res:?}"
            );
        }
    }
}

/// Every corpus, at two levels: the lazy kernel's bytes are the data
#[test]
fn every_corpus_the_same_bytes() {
    let k = Kernels::detect();
    for (name, data) in corpora(1) {
        for level in [1, 9] {
            let z = compress(&data, level);
            let (out, res, _) = cells::decode::<LazyCells>(&z, k);
            assert!(res.is_ok() && out == data, "{name} {level}: {res:?}");
        }
    }
}

/// Corrupt input in a stream of newly admitted blocks: each block's stored CRC with a bit
/// flipped, bits flipped in each block's body, and the stream cut short. The lazy kernel's
/// output and error (kind, block and bit) are Segmented's, sequentially; lazy workers' are
/// the plain parallel decoder's
#[test]
fn corrupt_input_the_same_errors() {
    let k = Kernels::detect();
    let z = compress(&copies(2_000_000, 75), 9);
    let starts = block_starts(&z);
    assert!(starts.len() >= 3, "{}", starts.len());
    let flip = |at: u64| {
        let mut bad = z.clone();
        if ((at / 8) as usize) < bad.len() {
            bad[(at / 8) as usize] ^= 0x80 >> (at % 8);
        }
        bad
    };
    let mut cases = Vec::new();
    for &b in &starts {
        // the stored CRC, then the body at a few depths
        for at in [b + 48 + 5, b + 400, b + 20_000, b + 400_000, b + 2_000_000] {
            cases.push((format!("bit {at} (block at {b})"), flip(at)));
        }
    }
    cases.push(("cut short".to_string(), z[..z.len() * 2 / 3].to_vec()));
    let mut errors = 0;
    for (name, bad) in cases {
        let (want_out, want) = {
            let (o, r, _) = cells::decode::<Segmented>(&bad, k);
            (o, r)
        };
        let (got_out, got) = {
            let (o, r, _) = cells::decode::<LazyCells>(&bad, k);
            (o, r)
        };
        errors += usize::from(want.is_err());
        assert_eq!(got, want, "{name}: sequential");
        assert!(got_out == want_out, "{name}: sequential output");
        let (want_out, want) = plain_parallel(&bad);
        let dec = StreamDecoder::workers(8, 16, k, true);
        let (got_out, got) = drive(&dec, &bad, 1 << 20, 1 << 16);
        assert_eq!(got, want, "{name}: lazy workers");
        assert!(got_out == want_out, "{name}: lazy workers' output");
    }
    assert!(
        errors >= starts.len(),
        "{errors} of the corruptions were errors"
    );
}
