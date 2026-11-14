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
fn cells_long_writer_decoder_parity() {
    // The F01 long-writer witnesses also pass through front, RLE1 and CRC,
    // with the existing independent libbzip2 and shipped-decoder oracles.
    let mut word = [vec![b'a'; 9], vec![b'b'; 9]].concat().repeat(3);
    word.push(b'c');
    let n = word.len();
    let mut rotations: Vec<_> = (0..n).collect();
    rotations.sort_by_key(|&s| (0..n).map(|i| word[(s + i) % n]).collect::<Vec<_>>());
    let column: Vec<_> = rotations.iter().map(|&s| word[(s + n - 1) % n]).collect();
    for k in [Kernels::scalar(), Kernels::detect()] {
        for p in [0, 3] {
            let z = craft(9, &[Spec::of(column.clone(), p)], None);
            common::oracle(&z).unwrap();
            cells::check(&z, k);
        }
        // eight copies a byte: n = 440 with 8 records, inside the 64r < n gate (ten would be
        // refused as sparse, and only forced Cells would reach the writer)
        let large: Vec<_> = column.iter().flat_map(|&b| [b; 8]).collect();
        let z = craft(9, &[Spec::of(large, 0)], None);
        common::oracle(&z).unwrap();
        let (prod, forced) = cells::check(&z, k);
        assert_eq!((prod.admitted, prod.cells, prod.short_cycles), (1, 57, 1));
        assert_eq!(forced.admitted, 1);
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
