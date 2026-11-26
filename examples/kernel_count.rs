//! bench-kernel-count's probe (docs/journal/2026-11-25-kernel-count-registration.md): what a
//! decoder is built as, without decoding anything. One JSON line on stdout.
//!
//!     kernel_count built T        # the public API, `BzReader::parallel(r, T)` as shipped
//!     kernel_count workers K LAZY # `BzReader::parallel_workers(r, K, 4K, detect, LAZY)`
//!
//! The timed decodes are `ab child` (examples/ab.rs: `mibz2` for the public API,
//! `mibz2-workers-lazy` and `mibz2-workers-eager` for the explicit workers). This probe is run
//! beside them, in a process of its own and in the same placement (unpinned), so that the
//! record says what each asked count built on that machine: its decoding threads, its window
//! (blocks in flight), whether its workers invert with lazy letters, and whether it is the
//! service core or the pipelined teams. The reader is given an empty input and dropped at
//! once; what `StreamDecoder::new` builds does not depend on the input.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let empty = std::io::empty();
    let (r, asked) = match (a.get(1).map(String::as_str), a.len()) {
        (Some("built"), 3) => {
            let t: usize = a[2].parse().expect("T");
            (
                mibz2::BzReader::parallel(empty, t),
                format!("\"asked\":{t}"),
            )
        }
        (Some("workers"), 4) => {
            let k: usize = a[2].parse().expect("K");
            let lazy = match a[3].as_str() {
                "1" | "true" => true,
                "0" | "false" => false,
                x => panic!("LAZY is 1 or 0, not {x:?}"),
            };
            (
                mibz2::BzReader::parallel_workers(
                    empty,
                    k,
                    mibz2::parallel::SLOTS_PER_THREAD * k,
                    mibz2::Kernels::detect(),
                    lazy,
                ),
                format!("\"k\":{k},\"asked_lazy\":{lazy}"),
            )
        }
        _ => {
            eprintln!("usage: kernel_count built T | kernel_count workers K LAZY");
            std::process::exit(2)
        }
    };
    let built = match r.parallel_decoder() {
        Some(d) => format!(
            "\"threads\":{},\"window\":{},\"lazy\":{},\"service\":{},\"pipelined\":{}",
            d.threads(),
            d.window(),
            d.is_lazy(),
            d.is_service(),
            d.is_pipelined()
        ),
        None => {
            "\"threads\":1,\"window\":0,\"lazy\":false,\"service\":false,\"pipelined\":false".into()
        }
    };
    drop(r);
    println!("{{\"kind\":\"built\",{asked},{built}}}");
}
