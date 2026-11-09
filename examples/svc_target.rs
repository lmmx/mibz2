//! The service core's actual-target preflight (docs/journal/2026-11-09-service-guarded-registration.md,
//! DRAFT): untimed and read-only, run by `just bench-service-guarded` before anything is
//! measured. It prints one JSON object:
//! - the machine's facts as `parallel::Machine::here` reads them on this machine, now: the
//!   plan policy's (physical cores, L3) and the predicate's (the topology snapshot, the cgroup
//!   quota), and the kernels `Kernels::detect` finds;
//! - `service_target` at 9, 10 and 11 requested threads, and `Plan::for_threads` at each;
//! - what `BzReader::parallel(.., t)` builds at each, by its own dispatch
//!   (`parallel::observe_builds`): the service core or not, pipelined or not, its decoding
//!   threads, its window, and, after it has decoded a small input to its end and been joined,
//!   its service counts; the bytes are checked against the input.
//!
//! It exits 0 only if: the predicate holds at 10 and not at 9 or 11; the snapshot is read
//! (not None) and the quota established (not `Unknown`); at 10 the reader built the service
//! core at `SERVICE_PLAN` (8 decoding threads, a window of 8) and every hub thread ended; at 9
//! and 11 it built something else; and every decode gave back the input. Anything else exits
//! 3, with `"ok": false` and the reasons. These are a consistent set of observations at one
//! moment, not an atomic kernel snapshot: affinity, quota and hotplug can change after it.

use mibz2::parallel::{observe_builds, service_target, Machine, Plan, Quota, SERVICE_PLAN};
use mibz2::{BzReader, Kernels};
use std::io::{Read, Write};

/// A JSON string
fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o + "\""
}

fn main() {
    let m = Machine::here();
    let k = Kernels::detect();
    let mut why: Vec<String> = Vec::new();
    // a few blocks of text, so the decode crosses blocks (level 1: 100 kB blocks)
    let data: Vec<u8> = (0..450_000u32)
        .map(|i| b"preflight of the service core "[(i % 30) as usize] ^ ((i / 997) % 2) as u8)
        .collect();
    let z = {
        let mut e = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::new(1));
        e.write_all(&data).unwrap();
        e.finish().unwrap()
    };
    let mut at = Vec::new();
    for t in [9usize, 10, 11] {
        let pred = service_target(t, k, &m);
        let plan = Plan::for_threads(t);
        let ((bytes, err), built) = observe_builds(|| {
            let mut r = BzReader::parallel(std::io::Cursor::new(z.clone()), t);
            let mut out = Vec::new();
            let err = r.read_to_end(&mut out).err().map(|e| e.to_string());
            if let Some(d) = r.parallel_decoder() {
                d.join(); // (the completion fence, before the counts are read)
            }
            (out, err)
        });
        let b = built.first().copied();
        if bytes != data || err.is_some() {
            why.push(format!(
                "{t} threads: the decode did not give back the input ({err:?})"
            ));
        }
        if built.len() != 1 {
            why.push(format!(
                "{t} threads: {} decoders built, not one",
                built.len()
            ));
        }
        let service = b.is_some_and(|b| b.service);
        if t == 10 {
            if !pred {
                why.push("10 threads: the predicate is false".into());
            }
            match b {
                Some(b)
                    if b.service
                        && b.threads == SERVICE_PLAN.threads()
                        && b.window == 8
                        && b.counts.is_some_and(|c| c.ended) => {}
                _ => why.push(format!(
                    "10 threads: not svc2 at 8 threads, window 8: {b:?}"
                )),
            }
        } else {
            if pred {
                why.push(format!("{t} threads: the predicate holds"));
            }
            if service {
                why.push(format!("{t} threads: the service core was built"));
            }
        }
        at.push(format!(
            "{{\"threads\":{t},\"predicate\":{pred},\"plan\":{},\"built\":{}}}",
            js(&format!("{plan:?}")),
            js(&format!("{b:?}"))
        ));
    }
    if m.target.is_none() {
        why.push("the topology snapshot could not be read (None)".into());
    }
    if m.quota == Quota::Unknown {
        why.push("the CPU quota could not be established (Unknown)".into());
    }
    let ok = why.is_empty();
    println!(
        "{{\"ok\":{ok},\"why\":[{}],\"machine\":{{\"phys\":{},\"l3\":{},\"target\":{},\"quota\":{}}},\"kernels\":{},\"at\":[{}]}}",
        why.iter().map(|w| js(w)).collect::<Vec<_>>().join(","),
        js(&format!("{:?}", m.phys)),
        js(&format!("{:?}", m.l3)),
        js(&format!("{:?}", m.target)),
        js(&format!("{:?}", m.quota)),
        js(&format!("{k:?}")),
        at.join(",")
    );
    std::process::exit(if ok { 0 } else { 3 });
}
