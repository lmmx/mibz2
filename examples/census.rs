//! (research only, `census`) The block-structure census of the open kernel's blocks
//! (docs/journal/2026-11-24-wikidata-block-census.md; #145 LOG 7126445058):
//!
//!     census FILE.bz2 OUT.tsv
//!
//! Decodes FILE once with production's lazy workers at 1 worker (`BzReader::parallel_workers`,
//! lazy: `OpenCells`, `SLOTS_PER_THREAD` blocks in flight), as bench-emit-cost-sq's 1-worker
//! cells do, so the records come in the order of that run's 1-worker `seq`. One line a block
//! the open kernel took (`src/census.rs`' fields).
use std::io::Read;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 3 {
        eprintln!("usage: census FILE.bz2 OUT.tsv");
        std::process::exit(2);
    }
    let data: &'static [u8] = std::fs::read(&a[1]).expect("read the input").leak();
    let k = mibz2::Kernels::detect();
    let _ = mibz2::census::take();
    let mut r =
        mibz2::BzReader::parallel_workers(data, 1, mibz2::parallel::SLOTS_PER_THREAD, k, true);
    let mut buf = vec![0u8; 4 << 20];
    let mut bytes = 0u64;
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(x) => bytes += x as u64,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("decode failed: {e}"),
        }
    }
    drop(r);
    let recs = mibz2::census::take();
    let mut out = String::from(
        "seq\tid\tn\tp\trecords\tm_kernel\tm\truns\truns_1\truns_2_3\truns_4_16\truns_17_127\truns_128\
         \trows_1\trows_2_3\trows_4_16\trows_17_127\trows_128\tstrips\twords\tbands\tlongest\tcycles\
         \tmarked_cycle\td_lt16\td_lt1024\td_lt8192\td_lt65536\td_log2_sum\td_sum\td_p50\td_p90\n",
    );
    for (i, x) in recs.iter().enumerate() {
        let j = |v: &[u32]| {
            v.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("\t")
        };
        out += &format!(
            "{i}\t{:016x}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{}\t{}\t{}\n",
            x.id,
            x.n,
            x.p,
            x.records,
            x.m_kernel,
            x.m,
            x.runs,
            j(&x.runs_by_len),
            j(&x.rows_by_len),
            x.strips,
            x.words,
            x.bands,
            x.longest,
            x.cycles,
            x.marked_cycle,
            j(&x.d_lt),
            x.d_log2_sum,
            x.d_sum,
            x.d_p50,
            x.d_p90
        );
    }
    std::fs::write(&a[2], out).expect("write OUT.tsv");
    eprintln!("census: {} bytes, {} open blocks", bytes, recs.len());
}
