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
    writeln!(csv,"stream,rows,first_alphabet,old_first_dictionary_bytes,live_first_dictionary_bytes,cached_words,bytes_spelled_from_trie,final_dictionary_bytes,first_token_references,bytes_from_second_owner,bytes_from_first_owner,bytes_first_trie_full,bytes_first_trie_partial,second_word_full_owners").unwrap();
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
        let owned2 = c.second.owners;
        assert_eq!(owned2[..4].iter().sum::<u64>(), n as u64);
        writeln!(
            csv,
            "{index},{n},{},{all},{},{},{from_trie},{},{},{},{},{},{},{}",
            c.words.len(),
            c.dict.len(),
            owned.len(),
            c.second.events[4],
            c.second.events[1],
            owned2[0],
            owned2[1],
            owned2[2],
            owned2[3],
            owned2[4]
        )
        .unwrap();
        bytes += n;
        accepted += 1;
    }
    println!("{accepted} accepted, {bytes} exact marked bytes");
    assert_eq!((accepted, bytes), (228, 205197794));
}

#[test]
fn ownership_on_arbitrary_marked_cycles() {
    let mut candidate = Candidate::default();
    let mut out = Vec::new();
    let (mut two, mut clipped, mut first_fallback, mut no_section) = (0, 0, 0, 0);
    for (alphabet, max_n) in [(2usize, 10), (3, 7)] {
        for n in 1..=max_n {
            for mut value in 0..alphabet.pow(n as u32) {
                let column: Vec<_> = (0..n)
                    .map(|_| {
                        let byte = QUOTE + (value % alphabet) as u8;
                        value /= alphabet;
                        byte
                    })
                    .collect();
                let caps: Vec<_> = column.iter().map(|&b| u16::from(b) << 8 | 1).collect();
                let mut psi: Vec<_> = (0..n).collect();
                psi.sort_by_key(|&j| column[j]);
                for p in 0..n {
                    let mut pos = p;
                    let want: Vec<_> = (0..n)
                        .map(|_| {
                            pos = psi[pos];
                            column[pos]
                        })
                        .collect();
                    let took = candidate.attempt::<true>(&caps, n, p as u32, &mut out);
                    assert_eq!(took, want.contains(&QUOTE));
                    if !took {
                        no_section += 1;
                        continue;
                    }
                    assert_eq!(want, out);
                    if candidate.second.events[3] == 0 {
                        first_fallback += 1;
                    } else {
                        two += 1;
                        clipped += usize::from(candidate.second.owners[3] > 0);
                        assert_eq!(candidate.second.owners[..4].iter().sum::<u64>(), n as u64);
                    }
                }
            }
        }
    }
    assert!(two > 0 && clipped > 0 && first_fallback > 0 && no_section > 0);
    println!("two-layer marks {two}, clipped uncached first words {clipped}, one-layer fallback {first_fallback}, no section {no_section}");
}
