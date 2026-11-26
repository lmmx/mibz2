include!("../output-owned/checks.rs");

#[test]
#[ignore]
fn checked_thresholds() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let data = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&data));
    let (mut rare, mut common, mut reference) = (
        Mixed::<2>::default(),
        Mixed::<64>::default(),
        Segmented::default(),
    );
    let (mut r, mut want, mut out) = (Runs::default(), Vec::new(), Vec::new());
    let mut csv =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    writeln!(csv,"stream,rows,threshold2_dictionary,threshold64_dictionary,threshold2_visits,threshold64_visits,threshold2_read_bytes,threshold64_read_bytes,threshold2_first_read,threshold64_first_read").unwrap();
    let k = Kernels::detect();
    let (mut count, mut bytes) = (0, 0);
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        let n = crate::runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let p = symbols.orig_ptr;
        let a = rare.attempt::<false>(r.records(), n, p, &mut out);
        assert!(reference.invert_runs(&r, n, p, &mut want, k));
        if a {
            assert_eq!(out, want);
        }
        assert_eq!(common.attempt::<false>(r.records(), n, p, &mut out), a);
        if !a {
            continue;
        }
        assert_eq!(out, want);
        assert!(rare.second.events[3] > 0 && common.second.events[3] > 0);
        writeln!(
            csv,
            "{index},{n},{},{},{},{},{},{},{},{}",
            rare.second.events[4],
            common.second.events[4],
            rare.second.events[2],
            common.second.events[2],
            rare.second.hot_reads,
            common.second.hot_reads,
            rare.second.first_hot_reads,
            common.second.first_hot_reads
        )
        .unwrap();
        count += 1;
        bytes += n;
    }
    assert_eq!((count, bytes), (228, 205197794));
    println!("both native threshold arms checked {count} admissions and {bytes} inverse bytes");
}
