//! The fused post-walk arm (`bwt::FusedFinish`: RLE1 with the block's CRC folded in as it
//! goes, #145 QUEUED 6946051925): the same bytes and the same errors as its kernel's own
//! finish, on its own and through the parallel decoder.
mod common;
use common::{block_starts, compress, corpora, Rng};
use mibz2::bwt::{FusedFinish, LazyCells};
use mibz2::{decode_with, ErrorKind, Kernels};
use std::io::Read;

fn kernels() -> Vec<Kernels> {
    vec![Kernels::scalar(), Kernels::detect()]
}

/// Every corpus, at two levels: the fused finish gives the lazy kernel's bytes, by every
/// kernel set, alone and through workers
#[test]
fn the_same_bytes() {
    for (name, data) in corpora(1) {
        for level in [1, 9] {
            let z = compress(&data, level);
            for k in kernels() {
                let want = decode_with::<LazyCells>(&z, k).unwrap();
                assert!(want == data, "{name} {level}");
                let got = decode_with::<FusedFinish<LazyCells>>(&z, k).unwrap();
                assert!(got == want, "{name} {level} {k:?}");
            }
            let mut got = Vec::new();
            mibz2::BzReader::parallel_workers_with::<FusedFinish<LazyCells>>(
                std::io::Cursor::new(z.clone()),
                3,
                12,
                Kernels::detect(),
            )
            .read_to_end(&mut got)
            .unwrap();
            assert!(got == data, "{name} {level}: workers");
        }
    }
}

/// A block's stored CRC with a bit flipped: the same error, stored and computed CRCs
/// included, as the kernel's own finish, from every block
#[test]
fn the_same_crc_errors() {
    let mut rng = Rng(0xf05ed);
    let data = rng.bytes(300_000);
    let mut text = corpora(1)
        .into_iter()
        .find(|(n, _)| *n == "text")
        .unwrap()
        .1;
    text.extend_from_slice(&data);
    let z = compress(&text, 1);
    let starts = block_starts(&z);
    assert!(starts.len() >= 2, "{}", starts.len());
    for &b in &starts {
        let mut bad = z.clone();
        let bit = b + 48 + 5;
        bad[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        for k in kernels() {
            let want = decode_with::<LazyCells>(&bad, k).unwrap_err();
            let got = decode_with::<FusedFinish<LazyCells>>(&bad, k).unwrap_err();
            assert!(matches!(want.kind, ErrorKind::BlockCrc { .. }), "{want}");
            assert_eq!(got.kind, want.kind, "block at {b}");
            assert_eq!((got.block, got.bit), (want.block, want.bit), "block at {b}");
        }
    }
}
