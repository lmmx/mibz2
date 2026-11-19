include!("../../f2c/checks.rs");

#[test]
#[ignore]
fn spelling_ownership() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let data = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut scan = Scanner::new(SliceSource(&data));
    let mut csv =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    let mut c = Candidate::default();
    let mut r = Runs::default();
    let mut ref_ = Segmented::default();
    let (mut out, mut want) = (Vec::new(), Vec::new());
    let k = Kernels::detect();
    let (mut accepted, mut bytes) = (0, 0);
    writeln!(csv,"stream,rows,first_alphabet,old_first_dictionary_bytes,live_first_dictionary_bytes,cached_words,bytes_spelled_from_trie,final_dictionary_bytes,first_token_references").unwrap();
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = crate::runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 || !c.attempt::<false>(r.records(), n, symbols.orig_ptr, &mut out) {
            continue;
        }
        assert!(ref_.invert_runs(&r, n, symbols.orig_ptr, &mut want, k));
        assert_eq!(out, want);
        assert!(c.second.events[3] > 0);
        let all: u32 = c.words.iter().map(|w| w.width).sum();
        let owned: Vec<_> = c.words.iter().filter(|w| w.id >> 31 != 0).collect();
        let from_trie: u32 = owned.iter().map(|w| w.width).sum();
        writeln!(
            csv,
            "{index},{n},{},{all},{},{},{from_trie},{},{}",
            c.words.len(),
            c.dict.len(),
            owned.len(),
            c.second.events[4],
            c.second.events[1]
        )
        .unwrap();
        bytes += n;
        accepted += 1;
    }
    println!("{accepted} accepted, {bytes} exact marked bytes");
    assert_eq!((accepted, bytes), (228, 205197794));
}
