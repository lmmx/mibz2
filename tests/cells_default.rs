//! The actual public calling-thread defaults, not the experimental Cells reader.
mod common;
use common::{cells, compress, craft, oracle, Spec};
use mibz2::{bwt::Segmented, BzReader, Error, Kernels};
use std::io::Read;

fn check(data: &[u8]) {
    let reference = mibz2::decode_with::<Segmented>(data, Kernels::detect());
    assert_eq!(mibz2::decode(data), reference);
    let (want, err, _) = cells::decode::<Segmented>(data, Kernels::scalar());
    for k in [Kernels::scalar(), Kernels::detect()] {
        for chunk in [1, 17, 4096] {
            let mut reader = BzReader::with_kernels(data, k);
            let mut out = Vec::new();
            let mut buffer = vec![0; chunk];
            let got_err = loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break Ok(()),
                    Ok(n) => out.extend_from_slice(&buffer[..n]),
                    Err(e) => {
                        break Err(e
                            .get_ref()
                            .unwrap()
                            .downcast_ref::<Error>()
                            .unwrap()
                            .clone())
                    }
                }
            };
            assert_eq!((&out, &got_err), (&want, &err));
        }
    }
    if let Ok(want) = reference {
        assert_eq!(want, oracle(data).unwrap());
        let mut got = Vec::new();
        for span in mibz2::scan(data) {
            let span = span.unwrap();
            mibz2::decode_blocks(
                data,
                span.start_bit,
                span.end_bit,
                span.level,
                Kernels::scalar(),
                &mut got,
            )
            .unwrap();
        }
        assert_eq!(got, want);
    }
}

#[test]
fn public_defaults_admission_periods_origins_errors_and_reuse() {
    for z in cells::witnesses() {
        check(&z);
    }
    for ll in [
        vec![0; 25],
        [vec![1; 51], vec![0; 50]].concat(),
        vec![0, 1],
        vec![2, 0, 2, 1, 0, 1],
    ] {
        for p in [0, ll.len() - 1] {
            let mut bad = Spec::of(ll.clone(), p as u32);
            check(&craft(9, std::slice::from_ref(&bad), None));
            bad.crc = Some(123);
            check(&craft(9, &[Spec::of(vec![1; 25], 1), bad], None));
        }
    }
    let mixed = [
        compress(&b"abacaba".repeat(20_000), 9),
        compress(&[0; 20_000], 1),
        compress(&(0..=255).cycle().take(4000).collect::<Vec<u8>>(), 9),
        compress(&b"ab".repeat(10_000), 9),
    ]
    .concat();
    check(&mixed);
    for end in [0, 1, 10, mixed.len() - 1] {
        check(&mixed[..end]);
    }
}

#[test]
#[ignore = "requires seven hash-verified corpus files; untimed pre-benchmark check"]
fn public_defaults_pinned_corpus_parity() {
    let root = std::path::PathBuf::from(std::env::var_os("CELLS_PINNED_INPUTS").unwrap());
    for name in [
        "wikidata", "onebyte", "pattern", "random", "records", "runs", "text",
    ] {
        let z = std::fs::read(root.join(format!("{name}.bz2"))).unwrap();
        let want = oracle(&z).unwrap();
        assert_eq!(
            mibz2::decode_with::<Segmented>(&z, Kernels::detect()).unwrap(),
            want
        );
        assert_eq!(mibz2::decode(&z).unwrap(), want, "decode {name}");
        let mut out = Vec::new();
        BzReader::new(&z[..]).read_to_end(&mut out).unwrap();
        assert_eq!(out, want, "BzReader {name}");
        out.clear();
        for span in mibz2::scan(&z) {
            let span = span.unwrap();
            mibz2::decode_blocks(
                &z,
                span.start_bit,
                span.end_bit,
                span.level,
                Kernels::detect(),
                &mut out,
            )
            .unwrap();
        }
        assert_eq!(out, want, "ranges {name}");
        eprintln!("PUBLIC_DEFAULT_PARITY {name} {} bytes", want.len());
    }
}
