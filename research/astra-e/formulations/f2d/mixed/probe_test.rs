#[test]
fn hotspot() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let input = std::fs::read(std::env::var("F2_CORPUS").unwrap()).unwrap();
    let (mut scan, mut r, mut c, mut out) = (
        Scanner::new(SliceSource(&input)),
        Runs::default(),
        Candidate::default(),
        Vec::new(),
    );
    let mut w =
        std::io::BufWriter::new(std::fs::File::create(std::env::var("F2_CSV").unwrap()).unwrap());
    writeln!(w,"stream,min_frequency,distinct,hot,original_dictionary,hot_dictionary,decoded,hot_visits,visits,hot_bytes,clipped_visits").unwrap();
    let k = Kernels::detect();
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let n = crate::runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 || !c.attempt::<false>(r.records(), n, symbols.orig_ptr, &mut out) {
            continue;
        }
        let rows = c.second.census(c.second.proj.0, c.second.proj.1, n as u32);
        let sum = |k: usize, selected: u32| -> u64 {
            rows.iter()
                .filter(|x| x.1 >= selected)
                .map(|v| [v.0, v.1, v.2, v.3, v.4][k] as u64)
                .sum()
        };
        assert_eq!(sum(3, 1), n as u64);
        for cutoff in [1, 2, 3, 4, 8, 16, 32, 64] {
            writeln!(
                w,
                "{index},{cutoff},{},{},{},{},{n},{},{},{},{}",
                rows.len(),
                rows.iter().filter(|x| x.1 >= cutoff).count(),
                sum(0, 1),
                sum(0, cutoff),
                sum(2, cutoff),
                sum(2, 1),
                sum(3, cutoff),
                sum(2, 1) - sum(4, 1)
            )
            .unwrap();
        }
    }
}
