mod common;
use common::{cells, compress, craft, Spec};
use mibz2::{bwt::CellsCounts, Kernels};

#[test]
fn cells_real_decoder_admission_reach() {
    for k in [Kernels::scalar(), Kernels::detect()] {
        let (mut prod, mut forced) = (CellsCounts::default(), CellsCounts::default());
        for z in cells::witnesses() {
            let (p, f) = cells::check(&z, k);
            cells::add(&mut prod, p);
            cells::add(&mut forced, f);
        }
        cells::assert_reach(prod, forced);
    }
}

#[test]
fn cells_corpora_and_structured_words() {
    let mut data: Vec<_> = common::corpora(1)
        .into_iter()
        .map(|(_, data)| data)
        .collect();
    let (mut a, mut b) = (b"a".to_vec(), b"ab".to_vec());
    while b.len() < 10_000 {
        let next = [&b[..], &a[..]].concat();
        a = b;
        b = next;
    }
    data.push(b);
    data.push((0u32..8192).map(|i| (i.count_ones() % 2) as u8).collect());
    data.push(b"ab".repeat(4096));
    data.push([b"ab".repeat(4096), vec![b'a']].concat());
    data.push(vec![0; 900_000]);
    data.push((0..8192).map(|i| (i % 256) as u8).collect());
    for bytes in data {
        let z = compress(&bytes, 9);
        cells::check(&z, Kernels::detect());
    }
}

#[test]
fn cells_arbitrary_columns_and_corruption() {
    for (k, l) in [(1, 1), (2, 3), (50, 51), (127, 128), (256, 1023)] {
        let ll = [vec![b'b'; k], vec![b'a'; l]].concat();
        for p in [0, k, k + l - 1] {
            let z = craft(9, &[Spec::of(ll.clone(), p as u32)], None);
            cells::check(&z, Kernels::scalar());
            let mut bad = Spec::of(ll.clone(), p as u32);
            bad.crc = Some(0xdead_beef);
            let z = craft(9, &[bad], None);
            cells::check(&z, Kernels::detect());
            cells::check(&z[..z.len() - 4], Kernels::scalar());
        }
    }
    // A good block before a corrupt one: identical partial output and error identity.
    let mut bad = Spec::of(vec![0; 25], 1);
    bad.crc = Some(123);
    cells::check(
        &craft(9, &[Spec::of(vec![1; 25], 1), bad], None),
        Kernels::detect(),
    );
    let joined = [
        compress(b"hello", 9),
        compress(b"", 1),
        compress(b"world", 1),
    ]
    .concat();
    cells::check(&joined, Kernels::detect());
}

#[test]
fn cells_keeps_text_random_and_records_on_shipped_route() {
    for (name, bytes) in common::corpora(1) {
        if matches!(name, "text" | "random" | "records") {
            let (prod, _) = cells::check(&compress(&bytes, 9), Kernels::detect());
            assert_eq!(prod.admitted, 0, "{name}: {prod:?}");
        }
    }
}
