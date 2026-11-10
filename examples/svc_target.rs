//! The service core's actual-target preflight (docs/journal/2026-11-09-service-guarded-registration.md,
//! DRAFT): untimed and read-only, run by `just bench-service-guarded` before anything is
//! measured, and alone by `just bench-svc-target` (the preflight-only entry,
//! docs/journal/2026-11-09-svc-target-registration.md). It prints one JSON object, schema
//! `mibz2.svc_target/2` (checked by bench/svc_target_check.py):
//! - the machine's facts as `parallel::Machine::here` reads them on this machine, now: the
//!   plan policy's (physical cores, L3) and the predicate's (the topology snapshot with the
//!   processor's name, the cgroup quota), and the kernels `Kernels::detect` finds (schema 2:
//!   the name added for the narrowed predicate,
//!   docs/journal/2026-11-10-service-guarded-target-gated-registration.md);
//! - `service_target` at 9, 10 and 11 requested threads, and `Plan::for_threads` at each;
//! - what `BzReader::parallel(.., t)` builds at each, by its own dispatch
//!   (`parallel::observe_builds`): the service core or not, pipelined or not, its decoding
//!   threads, its window, and, after it has decoded a small input to its end and been joined,
//!   its service counts; the bytes are checked against the input.
//!
//! It exits 0 only if: the predicate holds at 10 and not at 9 or 11; the snapshot is read
//! (not None) and the quota established (not `Unknown`); at 10 the reader built the service
//! core at `SERVICE_PLAN` (8 decoding threads, a window of 8), every hub thread ended and as
//! many blocks finished as prepared (at least one); at 9
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
                        && b.counts.is_some_and(|c| {
                            c.ended && c.finished == c.prepared && c.prepared > 0
                        }) => {}
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
        let plan_json = match plan {
            Plan::Workers(n) => format!("{{\"kind\":\"workers\",\"workers\":{n}}}"),
            Plan::Teams { teams, k } => {
                format!("{{\"kind\":\"teams\",\"teams\":{teams},\"k\":{k}}}")
            }
        };
        let built_json = match b {
            None => "null".to_string(),
            Some(b) => format!(
                "{{\"service\":{},\"pipelined\":{},\"threads\":{},\"window\":{},\"counts\":{}}}",
                b.service,
                b.pipelined,
                b.threads,
                b.window,
                b.counts.map_or("null".to_string(), |c| format!(
                    "{{\"prepared\":{},\"pooled\":{},\"finished\":{},\"ended\":{}}}",
                    c.prepared, c.pooled, c.finished, c.ended
                ))
            ),
        };
        at.push(format!(
            "{{\"threads\":{t},\"predicate\":{pred},\"plan\":{plan_json},\"builds\":{},\"built\":{built_json},\"decoded\":{}}}",
            built.len(),
            bytes == data && err.is_none()
        ));
    }
    if m.target.is_none() {
        why.push("the topology snapshot could not be read (None)".into());
    }
    if m.quota == Quota::Unknown {
        why.push("the CPU quota could not be established (Unknown)".into());
    }
    let ok = why.is_empty();
    let opt = |v: Option<String>| v.unwrap_or_else(|| "null".into());
    let target = opt(m.target.as_ref().map(|t| {
        format!(
            "{{\"phys\":{},\"logical\":{},\"l3\":{},\"cpu\":{{\"vendor\":{},\"family\":{},\"model\":{},\"name\":{}}}}}",
            t.phys,
            t.logical,
            t.l3,
            js(&t.cpu.0),
            t.cpu.1,
            t.cpu.2,
            js(&t.name)
        )
    }));
    let quota = match m.quota {
        Quota::Unlimited => "{\"kind\":\"unlimited\"}".to_string(),
        Quota::Cpus(c) => format!("{{\"kind\":\"cpus\",\"cpus\":{c}}}"),
        Quota::Unknown => "{\"kind\":\"unknown\"}".to_string(),
    };
    println!(
        "{{\"schema\":\"mibz2.svc_target/2\",\"ok\":{ok},\"why\":[{}],\"kernels\":{{\"avx2\":{},\"clmul\":{}}},\"machine\":{{\"phys\":{},\"l3\":{},\"target\":{target},\"quota\":{quota}}},\"service_plan\":{{\"teams\":{},\"k\":{},\"services\":{}}},\"input_bytes\":{},\"at\":[{}]}}",
        why.iter().map(|w| js(w)).collect::<Vec<_>>().join(","),
        k.avx2,
        k.clmul,
        opt(m.phys.map(|v| v.to_string())),
        opt(m.l3.map(|v| v.to_string())),
        SERVICE_PLAN.teams,
        SERVICE_PLAN.k,
        SERVICE_PLAN.services,
        data.len(),
        at.join(",")
    );
    std::process::exit(if ok { 0 } else { 3 });
}
