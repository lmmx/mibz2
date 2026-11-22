use std::fs;
use mibz2::{Kernels, Scratch};
use mibz2::bwt::OpenCells;
use mibz2::stream::{Event, Scanner, SliceSource};

fn main() {
    let path = "/tmp/wormhole_r5/wikidata_431b_210m.bz2";
    let data = fs::read(path).unwrap();
    let k = Kernels::detect();
    let mut sc = Scanner::new(SliceSource(&data));
    let mut syms = Vec::new();
    let mut scratch = Scratch::<OpenCells>::default();
    let mut out = Vec::new();
    let (mut total_syms, mut total_out, mut full, mut tiny) = (0usize, 0usize, 0usize, 0usize);
    while let Event::Block { symbols, .. } = sc.next(syms).unwrap() {
        total_syms += symbols.syms.len() * 2;
        out.clear();
        mibz2::decode_block(&symbols, k, &mut scratch, &mut out).unwrap();
        total_out += out.len();
        if out.len() > 65_536 { full += 1; } else { tiny += 1; }
        syms = symbols.syms;
    }
    println!("Corpus: {} full + {} tiny = {} blocks | out={:.2} MB | syms={:.2} MB ({:.3} B/B) | bz2={:.2} MiB",
        full, tiny, full + tiny, total_out as f64 / 1e6, total_syms as f64 / 1e6,
        total_syms as f64 / total_out as f64, data.len() as f64 / 1048576.0);

    // Calibrated Comet Lake i9-10900K (10C/20T, 256 KiB L2, 20 MiB L3, 42 GB/s DDR4) model:
    // Anchored to v0.5.12 published i9 data:
    //   - Regime 1 (serial scan = 56.0 ms): 7W/28 = 70.0 ms, untrimmed 8W/10W/12W/16W/20W = 1.06/1.05/1.12/1.14/1.16x
    //   - Off-chip DRAM at 7W ~ 7.0-7.5 B/B (consistent with #559 8.05 B/B pre-diet -> ~7.0 B/B post-#692/#703/#719)
    #[derive(Clone, Copy)]
    struct Arm {
        name: &'static str,
        ws_worker_mib: f64,     // per-worker L3 WS (Open 1.13 + runs 0.47 + ll 0.90/0 + active out 0.90/0.45)
        dirty_ready_mib_w: f64, // dirty ready + dead syms per worker in L3
        l3_traffic_bb: f64,     // total L2-evicting R+W traffic (B/B) before L3 filtering
        compulsory_dram_bb: f64,// streaming/cold writes that always hit DRAM (input, cold alloc, final dirty evictions)
        t_scan_serial_ms: f64,  // 56.0 (Regime 1) or 2.2 (Regime 2: Mechanisms C38b)
        t_worker_1t_ms: f64,    // 363.0 (Regime 1) or 395.0 (Regime 2 w/ ParHuff)
        t_alloc_pf_10w: f64,
        t_handoff_10w: f64,
        t_tail_10w: f64,
        t_reader_ms: f64,
    }

    let arms = [
        Arm {
            name: "R1_v0512 (Shipped, Serial Scan)",
            ws_worker_mib: 3.40, dirty_ready_mib_w: 1.54,
            l3_traffic_bb: 8.86, compulsory_dram_bb: 2.40,
            t_scan_serial_ms: 56.0, t_worker_1t_ms: 363.0,
            t_alloc_pf_10w: 6.2, t_handoff_10w: 2.0, t_tail_10w: 2.4, t_reader_ms: 16.0,
        },
        Arm {
            name: "R2_v0512 + Mech C38b (ParScan only)",
            ws_worker_mib: 3.40, dirty_ready_mib_w: 1.54,
            l3_traffic_bb: 8.86, compulsory_dram_bb: 2.40,
            t_scan_serial_ms: 2.2, t_worker_1t_ms: 395.0,
            t_alloc_pf_10w: 6.2, t_handoff_10w: 2.0, t_tail_10w: 2.4, t_reader_ms: 16.0,
        },
        Arm {
            name: "R2_P1 (C35: C32+H123+C33b + Mech C38b)",
            ws_worker_mib: 3.40, dirty_ready_mib_w: 1.20,
            l3_traffic_bb: 6.73, compulsory_dram_bb: 1.70,
            t_scan_serial_ms: 2.2, t_worker_1t_ms: 395.0,
            t_alloc_pf_10w: 3.4, t_handoff_10w: 0.4, t_tail_10w: 2.4, t_reader_ms: 8.5,
        },
        Arm {
            name: "R2_P2 (+C37 InPlaceRLE/TileCRC +C38 Window)",
            ws_worker_mib: 2.05, dirty_ready_mib_w: 0.55,
            l3_traffic_bb: 4.23, compulsory_dram_bb: 1.15,
            t_scan_serial_ms: 2.2, t_worker_1t_ms: 362.0,
            t_alloc_pf_10w: 1.6, t_handoff_10w: 0.4, t_tail_10w: 2.4, t_reader_ms: 8.5,
        },
        Arm {
            name: "R2_P3 (+C41 Straggler Sched + R4 Open -5%)",
            ws_worker_mib: 1.87, dirty_ready_mib_w: 0.55,
            l3_traffic_bb: 4.23, compulsory_dram_bb: 1.15,
            t_scan_serial_ms: 2.2, t_worker_1t_ms: 344.0,
            t_alloc_pf_10w: 1.6, t_handoff_10w: 0.4, t_tail_10w: 0.8, t_reader_ms: 8.5,
        },
    ];

    let l3 = 20.0_f64;
    let ddr4 = 42.0_f64;
    for a in &arms {
        println!("\n=== {} ===", a.name);
        println!("{:>3} {:>8} {:>7} {:>8} {:>8} {:>8} {:>8} {:>8}",
            "W", "L3_MB", "f_miss", "DRAM_BB", "BW_ms", "Par_ms", "Wall_ms", "GB/s");
        for &w in &[4usize, 7, 8, 10, 12, 16, 20] {
            let wf = w as f64;
            let eff = if w <= 10 { wf } else { 10.0 + (wf - 10.0) * 0.16 };
            let smt_l2 = if w <= 10 { 1.0 } else { 1.0 + 0.022 * (wf - 10.0) };
            let l3_dem = wf * (a.ws_worker_mib + a.dirty_ready_mib_w);
            let f_miss = (1.0 - (l3 / l3_dem).min(1.0)).max(0.0);
            let open_spill = 4.2 * f_miss.powf(1.2);
            let dram_bb = a.compulsory_dram_bb + a.l3_traffic_bb * (0.25 + 0.75 * f_miss) + open_spill;
            let bw_ms = (0.211 * dram_bb / ddr4) * 1e3;
            let lat_pen = 1.0 + 0.34 * f_miss.powf(1.25);
            let t_par = (a.t_worker_1t_ms / eff) * lat_pen * smt_l2
                + a.t_alloc_pf_10w * (wf / 10.0).powf(1.15)
                + a.t_handoff_10w * (wf / 10.0)
                + a.t_tail_10w * (wf / 10.0).min(1.25);
            // Queueing/contention blend between compute+latency and DDR4 bandwidth floor, bounded below by scan & reader
            let par_bw = t_par.max(bw_ms) + 0.12 * t_par.min(bw_ms) * (bw_ms / t_par).min(1.0).powi(2);
            let wall = par_bw.max(a.t_scan_serial_ms + 3.0).max(a.t_reader_ms);
            let gbps = (0.211 * dram_bb) / (wall * 1e-3);
            println!("{:>3} {:>8.1} {:>6.0}% {:>8.2} {:>8.1} {:>8.1} {:>8.1} {:>8.1}",
                w, l3_dem, f_miss * 100.0, dram_bb, bw_ms, t_par, wall, gbps);
        }
    }
}
