//! Force the hierarchy at small block sizes and compare with independent stable sorting.
pub use mibz2::{bwt, runs, Kernels};
mod candidate;

fn check(engine: &mut candidate::Solver, column: &[u8], p: u32, stop: usize) -> usize {
    let mut psi: Vec<usize> = (0..column.len()).collect();
    psi.sort_by_key(|&r| column[r]);
    let mut row = p as usize;
    let reference: Vec<_> = (0..column.len())
        .map(|_| {
            row = psi[row];
            column[row]
        })
        .collect();
    let mut runs = Vec::<u16>::new();
    for &b in column {
        if let Some(last) = runs.last_mut() {
            if *last >> 8 == b as u16 && *last & 255 < 127 {
                *last += 1;
                continue;
            }
        }
        runs.push((b as u16) << 8 | 1);
    }
    let mut out = Vec::new();
    engine.decode(&runs, column.len(), p, &mut out, stop);
    assert!(
        out == reference,
        "column length {} primary {} stop {}",
        column.len(),
        p,
        stop
    );
    out.len()
}

fn main() {
    let mut engine = candidate::Solver::default();
    let mut checks = 0;
    for (alphabet, limit) in [(2u32, 7), (3, 6), (4, 4)] {
        for n in 1..=limit {
            for mut id in 0..alphabet.pow(n) {
                let column: Vec<u8> = (0..n)
                    .map(|_| {
                        let byte = (id % alphabet) as u8;
                        id /= alphabet;
                        byte
                    })
                    .collect();
                for p in 0..n {
                    check(&mut engine, &column, p, 2);
                    checks += 1;
                }
            }
        }
    }
    println!("forced exhaustive marks {checks}");
    for arg in std::env::args().skip(1) {
        let (pk, file) = arg.split_once(':').unwrap();
        let p = pk.parse().unwrap();
        let data = std::fs::read(file).unwrap();
        let stop = if data.len() < 65536 { 2 } else { 32768 };
        check(&mut engine, &data, p, stop);
        println!(
            "passed rows={} primary={p} stop={stop} file={file}",
            data.len()
        );
    }
}
