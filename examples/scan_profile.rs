//! (research only, `scan-profile`) The parallel decoder's serial scanner profiled, work against
//! waiting, by component, for the bench-scan-profile diagnostic
//! (docs/journal/2026-11-24-scan-profile-registration.md):
//!
//!     scan_profile FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.jsonl
//!
//! WORKERS is comma-separated. Every decode is production's lazy workers
//! (`BzReader::parallel_workers`, lazy: `OpenCells`, 4 blocks a worker), FILE read into memory
//! once, the output read and discarded (the decoder checks every block's CRC and the stream's).
//! Round r decodes FILE twice at each count, once with the timers on (`scan_profile::set_on`)
//! and once off, the counts in WORKERS' order rotated by `r mod len(WORKERS)`, the timed decode
//! first in even rounds. SETTLE_S seconds of sleep precede each decode.
//!
//! OUT.jsonl, one line a decode: its round, count, whether timed, order, wall and bytes; timed,
//! also every block's scanner record (`scan_profile::BlockRec`, as an array in `BLOCK_FIELDS`'
//! order) and every worker's (`WorkerRec`: jobs, idle, life).
use mibz2::scan_profile as sp;
use std::fmt::Write as _;
use std::io::Read;
use std::time::{Duration, Instant};

/// A block record's fields, in the arrays' order
const BLOCK_FIELDS: [&str; 16] = [
    "index",
    "bits",
    "syms",
    "selectors",
    "switches",
    "tables",
    "slot_wait_ns",
    "read_ns",
    "input_wait_ns",
    "header_ns",
    "bitmap_ns",
    "selectors_ns",
    "tables_ns",
    "prep_ns",
    "decode_ns",
    "send_ns",
];

fn decode(data: &'static [u8], w: usize, timed: bool, buf: &mut [u8]) -> (f64, u64) {
    let _ = sp::take();
    sp::set_on(timed);
    let k = mibz2::Kernels::detect();
    let s = Instant::now();
    let mut r =
        mibz2::BzReader::parallel_workers(data, w, mibz2::parallel::SLOTS_PER_THREAD * w, k, true);
    let mut bytes = 0u64;
    loop {
        match r.read(buf) {
            Ok(0) => break,
            Ok(x) => bytes += x as u64,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => panic!("decode failed: {e}"),
        }
    }
    drop(r);
    let wall = s.elapsed().as_secs_f64();
    sp::set_on(false);
    (wall, bytes)
}

/// Round `round`'s cells (workers, timed), in order
fn cells(round: usize, workers: &[usize]) -> Vec<(usize, bool)> {
    let rot = round % workers.len();
    let first = round % 2 == 0;
    let mut cells = Vec::new();
    for i in 0..workers.len() {
        let w = workers[(i + rot) % workers.len()];
        cells.push((w, first));
        cells.push((w, !first));
    }
    cells
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() != 6 {
        eprintln!("usage: scan_profile FILE.bz2 WORKERS ROUNDS SETTLE_S OUT.jsonl");
        std::process::exit(2);
    }
    let data: &'static [u8] = std::fs::read(&a[1]).expect("read the input").leak();
    let workers: Vec<usize> = a[2]
        .split(',')
        .map(|s| s.parse().expect("WORKERS"))
        .collect();
    let rounds: usize = a[3].parse().expect("ROUNDS");
    let settle = Duration::from_secs_f64(a[4].parse().expect("SETTLE_S"));
    let mut buf = vec![0u8; 4 << 20];
    let mut out = String::new();
    writeln!(
        out,
        "{{\"kind\":\"meta\",\"block_fields\":[{}],\"input_bytes\":{}}}",
        BLOCK_FIELDS.map(|f| format!("\"{f}\"")).join(","),
        data.len()
    )
    .unwrap();
    for round in 0..rounds {
        for (o, &(w, timed)) in cells(round, &workers).iter().enumerate() {
            std::thread::sleep(settle);
            let (wall, bytes) = decode(data, w, timed, &mut buf);
            let (blocks, ws) = sp::take();
            let mut line = format!(
                "{{\"kind\":\"decode\",\"round\":{round},\"workers\":{w},\"timed\":{timed},\"order\":{o},\"wall_s\":{wall:.6},\"bytes\":{bytes}"
            );
            if timed {
                line += ",\"blocks\":[";
                for (i, b) in blocks.iter().enumerate() {
                    let v = [
                        b.index,
                        b.bits,
                        b.syms,
                        b.selectors,
                        b.switches,
                        b.tables,
                        b.slot_wait_ns,
                        b.read_ns,
                        b.input_wait_ns,
                        b.phase_ns[0],
                        b.phase_ns[1],
                        b.phase_ns[2],
                        b.phase_ns[3],
                        b.phase_ns[4],
                        b.phase_ns[5],
                        b.send_ns,
                    ];
                    if i > 0 {
                        line.push(',');
                    }
                    write!(line, "[{}]", v.map(|x| x.to_string()).join(",")).unwrap();
                }
                line += "],\"workers_rec\":[";
                for (i, x) in ws.iter().enumerate() {
                    if i > 0 {
                        line.push(',');
                    }
                    write!(line, "[{},{},{}]", x.jobs, x.idle_ns, x.life_ns).unwrap();
                }
                line.push(']');
            }
            line += "}\n";
            out += &line;
            let busy: u64 = blocks.iter().map(|b| b.read_ns + b.send_ns).sum();
            let slot: u64 = blocks.iter().map(|b| b.slot_wait_ns).sum();
            eprintln!(
                "round {round} workers {w} {}: {wall:.3} s, {bytes} bytes{}",
                if timed { "timed" } else { "untimed" },
                if timed {
                    format!(
                        ", {} blocks, scanner busy {:.3} s, slot wait {:.3} s, worker idle {:.3} s",
                        blocks.len(),
                        busy as f64 / 1e9,
                        slot as f64 / 1e9,
                        ws.iter().map(|x| x.idle_ns).sum::<u64>() as f64 / 1e9
                    )
                } else {
                    String::new()
                }
            );
        }
    }
    std::fs::write(&a[5], out).expect("write OUT.jsonl");
}

#[test]
fn the_order_is_balanced() {
    let w = [1, 6, 10];
    let mut first = std::collections::HashMap::<(usize, bool), usize>::new();
    let mut pos = std::collections::HashMap::<(usize, usize), usize>::new();
    for round in 0..6 {
        let c = cells(round, &w);
        assert_eq!(c.len(), 6);
        for (i, &(count, timed)) in c.iter().enumerate() {
            if i % 2 == 0 {
                *first.entry((count, timed)).or_default() += 1;
                *pos.entry((count, i / 2)).or_default() += 1;
            }
        }
    }
    // each count timed first in half the rounds, and in each count position twice in six
    assert!(first.values().all(|&v| v == 3), "{first:?}");
    assert!(pos.values().all(|&v| v == 2), "{pos:?}");
}
