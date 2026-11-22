use std::fs;
use std::process::Command;
use std::time::Instant;
use mibz2::{Kernels, StreamDecoder};
use mibz2::bwt::OpenCells;

#[repr(C)] #[derive(Default, Clone, Copy)] struct Timeval { tv_sec: i64, tv_usec: i64 }
#[repr(C)] #[derive(Default, Clone, Copy)] struct Rusage {
    ru_utime: Timeval, ru_stime: Timeval, ru_maxrss: i64, ru_ixrss: i64, ru_idrss: i64, ru_isrss: i64,
    ru_minflt: i64, ru_majflt: i64, ru_nswap: i64, ru_inblock: i64, ru_oublock: i64,
    ru_msgsnd: i64, ru_msgrcv: i64, ru_nsignals: i64, ru_nvcsw: i64, ru_nivcsw: i64,
}
extern "C" { fn getrusage(who: i32, usage: *mut Rusage) -> i32; }
fn rusage() -> Rusage { let mut r = Rusage::default(); unsafe { getrusage(0, &mut r); } r }
fn tv_ms(t: Timeval) -> f64 { t.tv_sec as f64 * 1e3 + t.tv_usec as f64 * 1e-3 }

/// Calibrated Comet Lake 10C/20T (20 MiB L3, 42 GB/s DDR4) wall-time predictor
/// Anchored to 4 real i9-10900K measurements on 210 MB Wikidata (431 blks):
///   1) v0.5.12 7W/28 (10T unpinned): 69.5 ms
///   2) v0.5.12 10W/40 untrimmed:     79.6 ms (7W/28 = 0.873x)
///   3) #739 (3296195e3) 7W/28:       66.9 ms (0.963x of 69.5 ms), RSS 0.60-0.68x
///   4) #739 + Mech BlockReader:      8T(8W) = 0.884x (~59.5 ms), 10T(7W) = 0.932x (~62.4 ms)
fn predict_i9_cell(
    w: usize, slots: usize, t_asked: usize, pinned: bool,
    regime: &str, // "739_only" | "739_mech_br" | "739_pr4"
) -> (f64, f64, f64, f64) {
    let cores = if pinned { t_asked.min(10) } else { 10 };
    // Per-worker compute on i9 (OpenCells + runs::front + unrle_in_place + tiled CRC, post-#739)
    // Calibrated: 1T decode_owned total across 431 blks ~ 358 ms on i9 (3.7 GHz turbo ~4.8 GHz)
    let w_total_ms = match regime {
        "739_pr4" => 358.0 + 38.0, // workers also do parallel Huffman/block-read (~38 ms total CPU)
        _ => 358.0,
    };
    let scan_serial_ms = match regime {
        "739_only" => 55.5,
        "739_mech_br" => 43.0, // Mech block reader cuts serial scan ~22.5%
        "739_pr4" => 2.5,      // Bounded-window 4 MiB parallel finder
        _ => 55.5,
    };

    // Physical-core & SMT interference factor
    // Serial scanner needs 1 full core in 739_only / 739_mech_br; reader needs ~0.18 core.
    let active_heavy = w + if regime == "739_pr4" { 0 } else { 1 };
    let smt_eff_workers = if w <= cores {
        let mut eff = w as f64;
        // If serial scanner shares a physical core with a worker (active_heavy > cores), both slow ~18% on that core
        if active_heavy > cores {
            let shared_cores = (active_heavy - cores).min(w);
            eff -= 0.24 * (shared_cores as f64);
        }
        eff
    } else {
        // Oversubscribed SMT workers beyond 10 physical cores: diminishing returns (+12% per SMT sibling) + L2 halving penalty
        let extra = (w - cores) as f64;
        (cores as f64) * (1.0 + 0.08 * (extra / cores as f64)) / (1.0 + 0.06 * (extra / cores as f64))
    };
    let scan_smt_pen = if regime != "739_pr4" && active_heavy > cores { 1.18 } else { 1.0 };

    // Live L3 working set (MiB) on #739 memory profile (0 B ll, 1.06x out, 12 MiB ready cap, immediate syms)
    // OpenCells hot+warm ~ 1.13 MiB/W + runs 0.51 MiB/W = 1.64 MiB/W; ready mean scales slightly with slots up to 12 MiB cap
    let ready_l3 = (2.4 + 0.14 * (slots as f64).min(48.0)).min(9.5);
    let syms_l3 = if regime == "739_pr4" { 0.35 * (w as f64) } else { 0.08 * (w as f64) + 0.6 };
    let in_l3 = if regime == "739_pr4" { 3.0 } else { 3.5 };
    let l3_mb = (w as f64) * 1.64 + ready_l3 + syms_l3 + in_l3;
    let f_miss = (1.0 - 20.0 / l3_mb).max(0.0);

    // Off-chip DRAM traffic (B/B) & DDR4 bandwidth floor (42 GB/s on 210 MB)
    let l2_evict_bb = if regime == "739_pr4" { 3.9 } else { 4.36 };
    let dram_bb = 1.0 + l2_evict_bb * (0.35 + 1.15 * f_miss) + 0.22 * (w as f64 / 10.0) * f_miss;
    let bw_floor_ms = (209.8 * dram_bb) / 42.0;

    // L3-miss latency stall added to worker critical path + cold page-fault / slot HOL penalty
    let l3_stall_ms = 22.0 * f_miss * (1.0 + 0.04 * (w.saturating_sub(8) as f64));
    let pf_ms = 1.1 + 0.19 * (w as f64) + 0.045 * (slots as f64);
    // Small window HOL penalty when slots == 2*W (noticeable on heterogeneous/tiny-mixed streams)
    let hol_ms = if slots <= 2 * w { 1.4 } else if slots <= 3 * w { 0.4 } else { 0.0 };
    let tail_ms = 0.20 * (w as f64);

    let worker_ms = (w_total_ms / smt_eff_workers) + l3_stall_ms + pf_ms + hol_ms + tail_ms;
    let scan_ms = scan_serial_ms * scan_smt_pen + pf_ms * 0.6 + hol_ms * 0.5;
    let wall_ms = worker_ms.max(scan_ms).max(bw_floor_ms + 8.0);

    // Predicted Peak RSS (MiB) on i9 (#739 profile)
    let rss_mb = 18.0 + (w as f64) * 3.1 + (slots.min(48) as f64) * 0.52;
    (wall_ms, l3_mb, dram_bb, rss_mb)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k = Kernels::detect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let data = fs::read(&args[2]).unwrap();
        let w: usize = args[3].parse().unwrap();
        let slots: usize = args[4].parse().unwrap();
        let r0 = rusage(); let t0 = Instant::now();
        let dec = StreamDecoder::with_inverse::<OpenCells>(w, slots, k);
        let mut total = 0usize;
        std::thread::scope(|s| {
            s.spawn(|| {
                for c in data.chunks(4 << 20) {
                    let mut v = dec.spare(); v.clear(); v.extend_from_slice(c);
                    if dec.write(v).is_err() { return; }
                }
                dec.close();
            });
            let mut spent = Vec::new();
            while let Some(b) = dec.read_block(spent).unwrap() { total += b.len(); spent = b; }
        });
        let wall = t0.elapsed().as_secs_f64() * 1e3;
        let r1 = rusage();
        println!("{wall:.2} {:.2} {:.2} {} {} {:.2} {total}",
            tv_ms(r1.ru_utime) - tv_ms(r0.ru_utime), tv_ms(r1.ru_stime) - tv_ms(r0.ru_stime),
            r1.ru_minflt - r0.ru_minflt, r1.ru_nvcsw - r0.ru_nvcsw, r1.ru_maxrss as f64 / 1024.0);
        return;
    }

    // 1. Print calibrated i9-10900K Grid Predictions (10T unpinned, plus 12/16/20T note)
    println!("=== CALIBRATED i9-10900K 10T UNPINNED GRID PREDICTIONS (210 MB WIKIDATA, #739 PROFILE) ===");
    println!("{:<4} {:>5} | {:>12} {:>12} {:>12} | {:>6} {:>7} {:>6}",
        "W", "slots", "3296195e3", "+MechBR", "+PR4_Par", "L3_MB", "DRAM_BB", "RSS_MB");
    for &w in &[6usize, 7, 8, 9, 10, 11, 12, 16] {
        for &mult in &[2usize, 3, 4] {
            let slots = w * mult;
            let (w0, l3, bb, rss) = predict_i9_cell(w, slots, 10, false, "739_only");
            let (w1, _, _, _) = predict_i9_cell(w, slots, 10, false, "739_mech_br");
            let (w2, l3p, bbp, _) = predict_i9_cell(w, slots, 10, false, "739_pr4");
            println!("{:<4} {:>5} | {:>9.1} ms {:>9.1} ms {:>9.1} ms | {:>5.1}M {:>6.2}B {:>5.1}M   (PR4 L3={:.1}M BB={:.2})",
                w, slots, w0, w1, w2, l3, bb, rss, l3p, bbp);
        }
    }

    // 2. Measure actual 3296195e3 OS counters & HOL behavior on Xeon across key grid cells
    let exe = std::env::current_exe().unwrap();
    for &(label, path) in &[
        ("wikidata-l9 (431 blks, 210 MB)", "/tmp/wormhole_r6/wikidata_431b_210m.bz2"),
        ("hetero-l9 (60 blks, 54 MB HOL guard)", "/tmp/wormhole_r6/hetero_60b.bz2"),
    ] {
        println!("\n=== XEON EMPIRICAL GRID CHECK ON 3296195e3: {label} (3-rd med) ===");
        println!("{:<4} {:>5} {:>8} {:>7} {:>7} {:>6} {:>7}", "W", "slots", "wall_ms", "sys_ms", "minflt", "vcsw", "rss_MB");
        for &(w, slots) in &[
            (7,14),(7,21),(7,28),
            (8,16),(8,24),(8,32),
            (9,18),(9,27),(9,36),
            (10,20),(10,30),(10,40),
            (12,24),(12,36),(12,48),
        ] {
            let mut rows = Vec::new();
            for _ in 0..3 {
                let o = Command::new(&exe).args(["--child", path, &w.to_string(), &slots.to_string()]).output().unwrap();
                let s = String::from_utf8(o.stdout).unwrap();
                let p: Vec<&str> = s.split_whitespace().collect();
                rows.push((
                    p[0].parse::<f64>().unwrap(), p[2].parse::<f64>().unwrap(),
                    p[3].parse::<i64>().unwrap(), p[4].parse::<i64>().unwrap(), p[5].parse::<f64>().unwrap(),
                ));
            }
            rows.sort_by(|a,b| a.0.total_cmp(&b.0));
            let m = rows[1];
            println!("{:<4} {:>5} {:>8.2} {:>7.2} {:>7} {:>6} {:>7.2}", w, slots, m.0, m.1, m.2, m.3, m.4);
        }
    }
}
