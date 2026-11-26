include!("../f2b/checks.rs");

#[test]
#[ignore]
fn native_second_level() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let input = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&input));
    let mut csv =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    writeln!(csv,"stream,rows,took,lookups,copy_spans,final_visits,token_rows,dict_bytes,physical_nodes,merged_pieces").unwrap();
    let (mut c, mut r, mut reference) =
        (Candidate::default(), Runs::default(), Segmented::default());
    let (mut out, mut want) = (Vec::new(), Vec::new());
    let (mut large, mut used, mut checked) = (0, 0, 0);
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        let k = Kernels::detect();
        let n = crate::runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let own = c.attempt::<false>(r.records(), n, symbols.orig_ptr, &mut out);
        if own {
            assert!(reference.invert_runs(&r, n, symbols.orig_ptr, &mut want, k));
            assert_eq!(out, want);
            assert!(c.second.events[3] > 0);
            used += 1;
            checked += n;
        }
        let s = if own { c.second.events } else { [0; 7] };
        writeln!(
            csv,
            "{index},{n},{},{},{},{},{},{},{},{}",
            own as u32, s[0], s[1], s[2], s[3], s[4], s[5], s[6]
        )
        .unwrap();
        large += 1;
    }
    println!("{used}/{large} second-level admissions; {checked} exact raw bytes");
    assert_eq!((large, used), (233, 228));
}
