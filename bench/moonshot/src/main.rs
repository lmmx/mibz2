//! The moonshot scorer's harness (bench/moonshot/README.md). Usage:
//!
//!     moonshot info
//!     moonshot check [--wikidata FILE] [--quick] [--seed N] [--arm ARM]
//!     moonshot run --arm ARM --workers W --file FILE [--repeat R]
//!     moonshot extract --file FILE --blocks I,J,.. --out SYMBOLS
//!     moonshot proxy --arm ARM --symbols SYMBOLS
//!
//! ARM is one of `lazy` (the baseline: the shipped kernel, OpenCells, 0.5.12), `cells` (eager), `open`, `opend` and
//! `candidate` (MOONSHOT_CANDIDATE at build time). `check` is the correctness gate (exit 1 on
//! any failure). `run` is one timed decode by the shipped workers, W of them and 4W blocks in
//! flight, inverting with ARM: one JSON line with its wall, CPU and peak RSS. `extract` writes
//! the symbols of the blocks named for `proxy`, which decodes them on this thread between two
//! Valgrind client requests that the cache and dependency model (chaingrind/) reads; outside
//! Valgrind those requests do nothing.
#![allow(clippy::needless_range_loop)]

// The candidate's view of mibz2: code written inside the crate (`crate::bwt`, `crate::runs`,
// `crate::Kernels`) compiles here unchanged
pub use mibz2::{bwt, runs, Kernels};

pub mod arms;
mod check;
#[path = "../../../tests/common/mod.rs"]
#[allow(clippy::all, unused)]
mod common;
#[path = "../../../docs/journal/2026-11-18-open-return/open_return.rs"]
#[allow(clippy::all, unused)]
pub mod open_return;

include!(concat!(env!("OUT_DIR"), "/candidate.rs"));

use mibz2::bwt::InverseBwt;
use mibz2::stream::{Event, Scanner, SliceSource};
use std::io::Read;
use std::time::Instant;

fn fail(msg: &str) -> ! {
    eprintln!("moonshot: {msg}");
    std::process::exit(2)
}

/// The value after `--name`, if given
fn opt(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).map(|i| {
        args.get(i + 1)
            .cloned()
            .unwrap_or_else(|| fail(&format!("{name} needs a value")))
    })
}

fn need(args: &[String], name: &str) -> String {
    opt(args, name).unwrap_or_else(|| fail(&format!("{name} is required")))
}

/// A field of /proc/self/status, in kB (VmHWM: the peak RSS)
fn status_kb(field: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with(field))
                .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        })
        .unwrap_or(0)
}

/// This process's user and system CPU seconds so far (/proc/self/stat, at 100 ticks a second)
fn cpu_s() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let rest = s.rsplit_once(')').map_or("", |x| x.1);
    let f: Vec<&str> = rest.split_whitespace().collect();
    let tick = |i: usize| f.get(i).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
    (tick(11) + tick(12)) / 100.0
}

/// One timed decode: the input already in memory, the workers built and the output drained
/// inside the clock, as examples/ab.rs times its in-process arms
fn timed<K: InverseBwt + 'static>(data: Vec<u8>, workers: usize) -> (f64, f64, u64) {
    let c0 = cpu_s();
    let t = Instant::now();
    let mut r = mibz2::BzReader::parallel_workers_with::<K>(
        std::io::Cursor::new(data),
        workers,
        4 * workers,
        Kernels::detect(),
    );
    let mut buf = vec![0u8; 4 << 20];
    let mut n = 0u64;
    loop {
        match r.read(&mut buf) {
            Ok(0) => break,
            Ok(k) => n += k as u64,
            Err(e) => fail(&format!("decode: {e}")),
        }
    }
    drop(r);
    (t.elapsed().as_secs_f64(), cpu_s() - c0, n)
}

/// Valgrind's client request (valgrind.h's VALGRIND_DO_CLIENT_REQUEST_EXPR on amd64): `default`
/// outside Valgrind
#[cfg(target_arch = "x86_64")]
fn client_request(default: u64, args: [u64; 6]) -> u64 {
    let mut r = default;
    // SAFETY: the rotations total 128 bits, so rdi is unchanged, and `xchg rbx, rbx` changes
    // nothing: natively a no-op sequence; Valgrind recognises it and writes rdx
    unsafe {
        std::arch::asm!(
            "rol rdi, 3", "rol rdi, 13", "rol rdi, 61", "rol rdi, 51", "xchg rbx, rbx",
            inout("rdx") r,
            in("rax") args.as_ptr(),
        );
    }
    r
}

#[cfg(not(target_arch = "x86_64"))]
fn client_request(default: u64, _: [u64; 6]) -> u64 {
    default
}

/// chaingrind's requests (VG_USERREQ_TOOL_BASE('M','S')): the measured region's start, and its
/// end with the output bytes it decoded
const REQ_BEGIN: u64 = (b'M' as u64) << 24 | (b'S' as u64) << 16;
const REQ_END: u64 = REQ_BEGIN + 1;
/// (the warm-up's start: until REQ_BEGIN, accesses fill chaingrind's caches and count nothing)
const REQ_WARM: u64 = REQ_BEGIN + 2;

/// The proxy's driver: the blocks decoded on this thread by `K`, the first once beforehand
/// outside the region (its buffers sized, as a worker's are after its first block)
/// Decodes `blocks` on one thread between the two client requests that bound chaingrind's region.
/// Before the region it decodes the first block once, to size the buffers. With `warm`, that
/// first block is a warm-up only and is left out of the region: a worker's state after some other
/// block, not after the measured block itself, and chaingrind's caches are warmed by it. The
/// measured blocks are decoded once before that, so that no buffer grows inside the region.
fn proxy<K: InverseBwt>(blocks: &[mibz2::block::Symbols], warm: bool) -> (u64, u64) {
    let k = Kernels::detect();
    let mut s = mibz2::Scratch::<K>::default();
    let mut out = Vec::new();
    if warm {
        // the measured blocks once first, outside the region and unseen by chaingrind's
        // caches: the kernel's buffers grow only when a block needs more than they hold (a
        // zero-filled reallocation, which Valgrind runs a byte a store), and a real worker's
        // have grown to their size within its first few blocks, never during a later one
        for b in blocks.iter().skip(1) {
            out.clear();
            mibz2::decode_block(b, k, &mut s, &mut out).unwrap_or_else(|e| fail(&format!("{e:?}")));
        }
        client_request(0, [REQ_WARM, 0, 0, 0, 0, 0]);
    }
    if let Some(b) = blocks.first() {
        out.clear();
        mibz2::decode_block(b, k, &mut s, &mut out).unwrap_or_else(|e| fail(&format!("{e:?}")));
    }
    let mut total = 0u64;
    let under = client_request(0, [REQ_BEGIN, 0, 0, 0, 0, 0]);
    for b in &blocks[(warm as usize).min(blocks.len())..] {
        out.clear();
        mibz2::decode_block(b, k, &mut s, &mut out).unwrap_or_else(|e| fail(&format!("{e:?}")));
        total += out.len() as u64;
    }
    client_request(0, [REQ_END, total, 0, 0, 0, 0]);
    (total, under)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let arm = opt(&args, "--arm").unwrap_or_else(|| "candidate".into());
    let unknown = || -> ! { fail(&format!("unknown arm {arm} (one of {:?})", arms::ARMS)) };
    match cmd {
        "info" => println!(
            "{{\"candidate\":{:?},\"candidate_type\":{:?},\"arms\":{:?},\"cpus\":{}}}",
            env!("MOONSHOT_CANDIDATE_PATH"),
            std::any::type_name::<candidate::Candidate>(),
            arms::ARMS,
            std::thread::available_parallelism().map_or(1, |n| n.get())
        ),
        "check" => {
            let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..").to_string();
            let o = check::Opts {
                root,
                wikidata: opt(&args, "--wikidata"),
                quick: args.iter().any(|a| a == "--quick"),
                seed: opt(&args, "--seed").map_or(1, |s| s.parse().unwrap()),
            };
            let failures = by_arm!(arm.as_str(), run_check(&o), unknown());
            println!(
                "{{\"arm\":{arm:?},\"verdict\":{:?},\"failures\":{failures}}}",
                if failures == 0 { "PASS" } else { "FAIL" }
            );
            std::process::exit((failures > 0) as i32);
        }
        "run" => {
            let workers: usize = need(&args, "--workers").parse().unwrap();
            let file = need(&args, "--file");
            let repeat: usize = opt(&args, "--repeat").map_or(1, |r| r.parse().unwrap());
            let one = std::fs::read(&file).unwrap_or_else(|e| fail(&format!("{file}: {e}")));
            let data = one.repeat(repeat.max(1));
            let (wall, cpu, bytes) = by_arm!(arm.as_str(), timed(data, workers), unknown());
            println!(
                "{{\"arm\":{arm:?},\"workers\":{workers},\"wall_s\":{wall:.6},\"cpu_s\":{cpu:.2},\"out_bytes\":{bytes},\"peak_rss_kb\":{}}}",
                status_kb("VmHWM:")
            );
        }
        "extract" => {
            let file = need(&args, "--file");
            let want: Vec<u64> = need(&args, "--blocks")
                .split(',')
                .map(|x| x.parse().unwrap())
                .collect();
            let data = std::fs::read(&file).unwrap_or_else(|e| fail(&format!("{file}: {e}")));
            let mut sc = Scanner::new(SliceSource(&data));
            let mut buf = Vec::new();
            let mut found = 0;
            while let Ok(Event::Block { index, symbols, .. }) = sc.next(Vec::new()) {
                if want.contains(&index) {
                    check::write_symbols(&mut buf, &symbols);
                    found += 1;
                }
            }
            std::fs::write(need(&args, "--out"), buf).unwrap();
            println!("{{\"blocks\":{found}}}");
        }
        "proxy" => {
            let path = need(&args, "--symbols");
            let blocks = check::read_symbols(&std::fs::read(&path).unwrap());
            let warm = args.iter().any(|a| a == "--warm");
            let (bytes, under) = by_arm!(arm.as_str(), proxy(&blocks, warm), unknown());
            println!(
                "{{\"arm\":{arm:?},\"blocks\":{},\"out_bytes\":{bytes},\"under_chaingrind\":{}}}",
                blocks.len(),
                under != 0
            );
        }
        _ => fail("usage: moonshot info | check | run | extract | proxy (see src/main.rs)"),
    }
}

fn run_check<K: InverseBwt + 'static>(o: &check::Opts) -> usize {
    check::run::<K>(o)
}
