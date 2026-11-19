// Compare the actual compiled actions with parsing the full encoded prefix and word
// from scratch. The independent reference consumes each run as a unit.
#[allow(dead_code)]
mod implementation {
    include!("actions.rs");

    fn reference(src: &[u8]) -> (Vec<u8>, State) {
        let mut out = Vec::new();
        let mut i = 0;
        let mut tail = State::default();
        while i < src.len() {
            let b = src[i];
            let mut n = 1;
            while n < 4 && i + n < src.len() && src[i + n] == b {
                n += 1;
            }
            i += n;
            out.extend(std::iter::repeat_n(b, n));
            tail = State {
                last: b,
                count: n as u8,
            };
            if n == 4 && i < src.len() {
                out.extend(std::iter::repeat_n(b, src[i] as usize));
                i += 1;
                tail = State::default();
            }
        }
        (out, tail)
    }

    fn check(word: &[u8], stats: &mut [usize; 4]) {
        let spelling = [b"ab".as_slice(), word].concat();
        let offsets = [0, 2, spelling.len() as u32];
        for budget in [0, word.len() / 2, 32_768] {
            let mut book = Book::default();
            book.compile(&spelling, &offsets, budget);
            assert!(book.expanded.len() <= budget);
            let action = book.actions[1];
            stats[1
                + usize::from(action.source == RAW_ONLY)
                + usize::from(action.source & OWNED != 0 && action.source != RAW_ONLY) * 2] += 1;
            for (last, count) in std::iter::once((0, 0)).chain(
                [0, 1, 255, 43]
                    .into_iter()
                    .flat_map(|last| (1..=4).map(move |k| (last, k))),
            ) {
                let prefix = vec![last; count];
                let (old, entry) = reference(&prefix);
                for take in 0..=word.len() {
                    let mut state = entry;
                    let mut got = old.clone();
                    let actual = book.write(1, &spelling, &offsets, take, &mut state, &mut got);
                    let expected = reference(&[prefix.as_slice(), &word[..take]].concat());
                    assert_eq!(
                        (got, state),
                        expected,
                        "word={word:?}, budget={budget}, prefix={prefix:?}, take={take}"
                    );
                    assert_eq!(actual, take);
                    stats[0] += 1;
                }
            }
        }
    }

    pub(super) fn run() {
        let mut stats = [0; 4];
        for n in 1..=9 {
            for mask in 0..1 << n {
                let w: Vec<_> = (0..n)
                    .map(|pos| if mask & (1 << pos) == 0 { 0 } else { 255 })
                    .collect();
                check(&w, &mut stats);
            }
        }
        for w in [
            vec![0; 67],
            vec![1; 67],
            vec![255; 33],
            [b"aaaabxyz".as_slice(), b"bbbbbcde".as_slice()].concat(),
        ] {
            check(&w, &mut stats);
        }
        assert!(stats.iter().all(|&v| v != 0));
        println!("{} exact entry/word/truncation checks; preparations used {} borrowed, {} raw fallback, {} owned",stats[0], stats[1],stats[2],stats[3]);
    }
}
fn main() {
    implementation::run();
}
