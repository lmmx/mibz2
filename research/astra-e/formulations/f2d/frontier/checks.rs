include!("../../f2c/checks.rs");

#[test]
#[ignore]
fn frontier_lookups() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let bytes = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&bytes));
    let (mut r, mut c, mut ref_, mut want, mut out) = (
        Runs::default(),
        Candidate::default(),
        Segmented::default(),
        Vec::new(),
        Vec::new(),
    );
    let mut csv =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    writeln!(
        csv,
        "stream,rows,lookups,certified_successors,logical_steps"
    )
    .unwrap();
    let k = Kernels::detect();
    let (mut count, mut total) = (0, 0);
    while let Ok(Event::Block { index, symbols, .. }) = scanner.next(Vec::new()) {
        let n = crate::runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 || !c.attempt::<false>(r.records(), n, symbols.orig_ptr, &mut out) {
            continue;
        }
        assert!(ref_.invert_runs(&r, n, symbols.orig_ptr, &mut want, k));
        assert_eq!(out, want);
        writeln!(
            csv,
            "{index},{n},{},{},{}",
            c.stats[0],
            c.stats[5],
            c.stats[0] + c.stats[5]
        )
        .unwrap();
        count += 1;
        total += n;
    }
    assert_eq!((count, total), (228, 205197794));
    println!("{count} admissions, {total} exact marked bytes");
}
