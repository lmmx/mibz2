//! bench-kernel-choice's probe (docs/journal/2026-11-25-kernel-choice-registration.md): what the
//! public API builds at T threads asked, without decoding anything. One JSON line on stdout.
//!
//!     kernel_choice_probe BUILD T   # `BzReader::parallel(r, T)` as the build ships it
//!
//! Not an example of the crate: `just bench-kernel-choice` copies it into each build's checkout
//! as `examples/kernel_choice_probe.rs` (after checking the checkout is clean) and builds it
//! beside `ab`, so each build reports its own plan. It is bench-kernel-count's probe
//! (examples/kernel_count.rs on #849), its `built` mode alone, with the build's label. Run in the
//! timed runs' placement (unpinned), it records the decoding threads, the window (blocks in
//! flight), whether the workers invert with lazy letters, and whether it is the service core or
//! the pipelined teams. The reader is given an empty input and dropped at once: what
//! `StreamDecoder::new` builds does not depend on the input.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 3 {
        eprintln!("usage: kernel_choice_probe BUILD T");
        std::process::exit(2)
    }
    let build = &a[1];
    let t: usize = a[2].parse().expect("T");
    let r = mibz2::BzReader::parallel(std::io::empty(), t);
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
    println!("{{\"kind\":\"built\",\"build\":\"{build}\",\"asked\":{t},{built}}}");
}
