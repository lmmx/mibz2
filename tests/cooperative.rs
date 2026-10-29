//! The decoder with the cooperative inverse BWT (`bwt::Cooperative<K>`) against libbzip2
//! and against the default kernel: valid streams of every level and shape byte for byte,
//! sequential and parallel, at K = 2, 3 and 4; corrupted and truncated streams failing as
//! the default kernel fails, at the same block.

mod common;
use common::*;
use mibz2::bwt::{Cooperative, InverseBwt, Segmented};
use mibz2::{Error, ErrorKind, Kernels, StreamDecoder};

fn kernels() -> Vec<Kernels> {
    let mut v = vec![Kernels::scalar()];
    if Kernels::detect() != Kernels::scalar() {
        v.push(Kernels::detect());
    }
    v
}

fn seq<I: InverseBwt>(z: &[u8], k: Kernels) -> Result<Vec<u8>, Error> {
    mibz2::decode_with::<I>(z, k)
}

/// The parallel decoder on kernel `I`, fed 100,003 bytes at a time from another thread
fn par<I: InverseBwt + 'static>(z: &[u8], workers: usize, slots: usize) -> Result<Vec<u8>, Error> {
    let dec = StreamDecoder::with_inverse::<I>(workers, slots, Kernels::detect());
    let mut out = Vec::new();
    std::thread::scope(|s| {
        s.spawn(|| {
            for c in z.chunks(100_003) {
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
    })
    .map(|()| out)
}

#[test]
fn every_level_and_shape() {
    let mut rng = Rng(0x1234_5678_9abc_def1);
    for (name, data) in samples(&mut rng) {
        for level in [1, 5, 9] {
            let z = compress(&data, level);
            for k in kernels() {
                for (kk, got) in [
                    (2, seq::<Cooperative<2>>(&z, k)),
                    (3, seq::<Cooperative<3>>(&z, k)),
                    (4, seq::<Cooperative<4>>(&z, k)),
                ] {
                    let got =
                        got.unwrap_or_else(|e| panic!("{name} level {level} K {kk} {k:?}: {e}"));
                    assert!(got == data, "{name} level {level} K {kk} {k:?}: differs");
                }
            }
            for (w, got) in [
                (1, par::<Cooperative<2>>(&z, 1, 2)),
                (2, par::<Cooperative<2>>(&z, 2, 4)),
                (3, par::<Cooperative<3>>(&z, 3, 3)),
            ] {
                let got = got.unwrap_or_else(|e| panic!("{name} level {level} {w} teams: {e}"));
                assert!(got == data, "{name} level {level}, {w} teams: differs");
            }
        }
    }
}

#[test]
fn multi_stream() {
    let mut rng = Rng(3);
    let s = samples(&mut rng);
    let mut z = compress(&s[3].1, 9);
    z.extend(compress(&s[5].1, 6));
    z.extend(compress(&s[4].1, 9));
    let mut want = s[3].1.clone();
    want.extend(&s[5].1);
    want.extend(&s[4].1);
    assert!(seq::<Cooperative<2>>(&z, Kernels::detect()).unwrap() == want);
    assert!(par::<Cooperative<2>>(&z, 2, 4).unwrap() == want);
}

/// A corrupted stream fails as the default kernel fails: the same kind, at the same bit
#[test]
fn errors_as_the_default_kernel() {
    let mut rng = Rng(5);
    let data = samples(&mut rng).swap_remove(3).1;
    let z = compress(&data, 9);
    let starts = block_starts(&z);
    assert!(starts.len() >= 3);
    let mut bad: Vec<Vec<u8>> = Vec::new();
    for &b in &starts {
        // the block's stored CRC, a bit flipped
        let mut t = z.clone();
        let bit = b + 48 + 5;
        t[(bit / 8) as usize] ^= 0x80 >> (bit % 8);
        bad.push(t);
        // a byte inside the block's data
        let mut t = z.clone();
        t[(b / 8) as usize + 4000] ^= 0x5a;
        bad.push(t);
    }
    bad.push(z[..z.len() / 2].to_vec());
    bad.push(z[..z.len() - 3].to_vec());
    for (i, t) in bad.iter().enumerate() {
        let want = seq::<Segmented>(t, Kernels::detect());
        let got = seq::<Cooperative<3>>(t, Kernels::detect());
        match (&want, &got) {
            (Err(a), Err(b)) => {
                assert_eq!(a.kind, b.kind, "case {i}: the error");
                assert_eq!(a.bit, b.bit, "case {i}: the error's bit");
            }
            (Ok(a), Ok(b)) => assert!(a == b, "case {i}: the output"),
            _ => panic!(
                "case {i}: {:?} against {:?}",
                want.as_ref().err(),
                got.as_ref().err()
            ),
        }
        let pw = parallel(t, 2, 4, 1 << 16, 1 << 16, Kernels::detect());
        let pg = par::<Cooperative<2>>(t, 2, 4);
        match (&pw, &pg) {
            (Err(a), Err(b)) => assert_eq!(a.kind, b.kind, "case {i}: the parallel error"),
            (Ok(a), Ok(b)) => assert!(a == b, "case {i}: the parallel output"),
            _ => panic!(
                "case {i}, parallel: {:?} against {:?}",
                pw.as_ref().err(),
                pg.as_ref().err()
            ),
        }
        if let Err(e) = &got {
            assert!(
                !matches!(e.kind, ErrorKind::Internal(_)),
                "case {i}: an internal error"
            );
        }
    }
}
