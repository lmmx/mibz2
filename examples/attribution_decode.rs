//! The shipped parallel decoder as one command, for counting and sampling it from outside
//! (docs/journal/2026-11-01-decoder-attribution-protocol.md, M2 and M3): the workload is
//! examples/occupancy.rs's own, `BzReader::parallel_with(File, w, 2w, Kernels::detect())`,
//! its feeder (`mibz2-read`) reading the file, the output drained in 4 MiB reads. Built
//! with default features, so nothing of src/trace.rs is in it.
//!
//! ```text
//! attribution_decode FILE.bz2 W            # drain; print the bytes decoded and the plan
//! attribution_decode FILE.bz2 W --stdout   # the decoded bytes on stdout (the hash check)
//! ```

use std::io::{Read, Write};

use mibz2::parallel::Plan;
use mibz2::{BzReader, Kernels};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (path, w) = match (a.first(), a.get(1).and_then(|s| s.parse::<usize>().ok())) {
        (Some(p), Some(w)) if w > 0 => (p.clone(), w),
        _ => {
            eprintln!("usage: attribution_decode FILE.bz2 W [--stdout]");
            std::process::exit(2)
        }
    };
    let to_stdout = a.get(2).map(String::as_str) == Some("--stdout");
    let plan = Plan::for_threads(w);
    if plan != Plan::Workers(w) {
        eprintln!("attribution_decode: the plan is {plan:?}, not Workers({w})");
        std::process::exit(3)
    }
    let f = std::fs::File::open(&path).expect("open the input");
    let mut r = BzReader::parallel_with(f, w, 2 * w, Kernels::detect());
    let mut buf = vec![0u8; 4 << 20];
    let (mut n, mut fold) = (0u64, 0u64);
    let mut out = std::io::stdout().lock();
    loop {
        let k = r.read(&mut buf).expect("decode");
        if k == 0 {
            break;
        }
        n += k as u64;
        if to_stdout {
            out.write_all(&buf[..k]).expect("write");
        } else {
            // touch what was read, as a consumer would
            fold = fold.wrapping_mul(31).wrapping_add(buf[k - 1] as u64);
        }
    }
    drop(r);
    if !to_stdout {
        println!("bytes={n} plan=workers{w} fold={fold}");
    }
}
