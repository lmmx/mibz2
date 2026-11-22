pub(super) fn stream<I: InverseBwt + 'static>(data: &'static [u8], workers: usize) -> f64 {
    let clock = std::time::Instant::now();
    let mut reader = crate::BzReader::parallel_workers_with::<I>(data, workers, 2 * workers, Kernels::detect());
    assert_eq!(std::io::copy(&mut reader, &mut std::io::sink()).unwrap(), 209_700_196);
    clock.elapsed().as_secs_f64() * 1e6
}

#[cfg(feature = "block-clocks")]
static BLOCK_NS: [std::sync::atomic::AtomicU64; 431] =
    [const { std::sync::atomic::AtomicU64::new(0) }; 431];

#[cfg(feature = "block-clocks")]
pub(crate) fn record(place: u64, elapsed: std::time::Duration) {
    BLOCK_NS[place as usize].store(elapsed.as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(feature = "block-clocks")]
pub(super) fn read_times() -> [f64; 431] {
    std::array::from_fn(|i| {
        let ns = BLOCK_NS[i].swap(0, std::sync::atomic::Ordering::Relaxed);
        assert!(ns > 0, "missing block {i}");
        ns as f64 / 1000.
    })
}

#[test]
#[ignore]
fn all_blocks() {
    let blocks = load_bz2(&std::env::var("WIKIDATA").unwrap());
    assert_eq!(blocks.len(), 431);
    let mut base = crate::reference::OpenCells::default();
    let mut head = crate::open::OpenCells::default();
    let (mut expected, mut actual) = (Vec::new(), Vec::new());
    for b in &blocks {
        dec(&mut base, b, &mut expected);
        dec(&mut head, b, &mut actual);
        assert!(actual == expected);
        if let Blk::Run { idx, p, runs, .. } = b {
            if [41, 56, 165, 270, 278].contains(idx) {
                let dir = std::env::var("DENSE_DIR").unwrap();
                std::fs::write(format!("{dir}/dense-{idx}.L"), column(runs)).unwrap();
                println!("DENSE {idx} {p}");
            }
        }
    }
}
