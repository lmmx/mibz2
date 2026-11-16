//! Research only (`induction-trace`): the shipped lazy Cells kernel's induction on each block
//! of a file, recorded (`mibz2::itrace`, format `irec/1`), for MONODROMY's Beyond Rauzy–Veech
//! programme (#145, QUEUED 6915302968). `bench/irec.py` reads the records.
//!
//! ```text
//! induction_trace FILE.bz2 OUTDIR [EVERY] [MIN_ROWS]
//! ```
//!
//! The blocks of at least MIN_ROWS rows (default 65,536: MONODROMY's "233 Wikidata blocks"
//! of corpora/wikidata.bz2) are numbered from 0 among themselves, and every EVERY-th of them
//! (default 1: all) is recorded, as OUTDIR/NNNN.irec, NNNN that number. Each block's record is
//! checked before it is written: the replay's steps against the kernel's log, and its final
//! word against the kernel's output (`itrace::record`). A block that fails stops the run
//! with an error, and nothing more is written. One JSON line a block goes to stdout. With
//! OUTDIR `-`, every block is recorded and checked and nothing is written.

use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::{itrace, Kernels};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: induction_trace FILE.bz2 OUTDIR [EVERY] [MIN_ROWS]");
        std::process::exit(2);
    }
    let path = &args[1];
    let dir = (args[2] != "-").then(|| std::path::Path::new(&args[2]));
    let every: usize = args.get(3).map_or(1, |s| s.parse().expect("EVERY"));
    let min_rows: usize = args.get(4).map_or(65536, |s| s.parse().expect("MIN_ROWS"));
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    if let Some(dir) = dir {
        std::fs::create_dir_all(dir).expect("OUTDIR");
    }
    let corpus = std::path::Path::new(path)
        .file_name()
        .map_or_else(|| path.clone(), |s| s.to_string_lossy().into_owned());
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut seq = 0usize;
    let stdout = std::io::stdout();
    let mut lines = stdout.lock();
    loop {
        let (index, b) = match sc.next(std::mem::take(&mut syms)) {
            Ok(Event::Block { index, symbols, .. }) => (index, symbols),
            Ok(_) => break,
            Err(e) => panic!("{path}: {e:?}"),
        };
        let rows = rows(&b);
        if rows >= min_rows {
            if seq % every == 0 {
                let t = std::time::Instant::now();
                let rec = itrace::record(&b, k)
                    .unwrap_or_else(|e| panic!("{corpus} block {seq} (index {index}): {e}"));
                let bytes = rec.to_bytes(&corpus, seq, index);
                if let Some(dir) = dir {
                    std::fs::write(dir.join(format!("{seq:04}.irec")), &bytes).expect("write");
                }
                writeln!(
                    lines,
                    "{{\"block\":{seq},\"index\":{index},\"n\":{},\"r\":{},\"steps\":{},\"closures\":{},\"period\":{},\"admitted\":{},\"bytes\":{},\"s\":{:.3}}}",
                    rec.n,
                    rec.r,
                    rec.s_a.len(),
                    rec.c_step.len(),
                    rec.period,
                    rec.admitted,
                    bytes.len(),
                    t.elapsed().as_secs_f64()
                )
                .expect("stdout");
            }
            seq += 1;
        }
        syms = b.syms;
    }
}

/// A block's rows, from its symbols (the MTF stage's output length), as MONODROMY's scripts
/// count them
fn rows(b: &mibz2::block::Symbols) -> usize {
    let mut ll = Vec::new();
    let mut counts = [0u32; 256];
    mibz2::mtf::mtf(b, &mut ll, &mut counts, Kernels::detect()).unwrap_or(0)
}
