//! Research only (`open-phases`): fused wiring (F) and queue compaction (Q) against shipped
//! Open, on exact BWT columns, on one thread (docs/journal/2026-11-20-open-fused-wiring.md).
//! Never on a production path.
//!
//! ```text
//! open_fused DIR ROUNDS OUT [BLOCK...]   # DIR holds block-<id>.L and manifest.json
//! ```
//!
//! **Checks first, per block:** each research arm's construction (`edges`, `extra`, `spelling`,
//! `offsets`, `mark`, `bits`) equals shipped Open's (`Kernel::construct::<true, true>`), in both
//! the whole and the phased form. Then every output of every timed round is checked byte for
//! byte against a dense-LF walk, outside the timed span.
//!
//! **Arms:** `shipped` (production's `Kernel::run::<true, true>`), `F`, `Q`, `FQ`, and `Qi`
//! and `FQi` (Q's other form: per-row queues pre-sized and written by index), each timed
//! whole (no stage timers, the spelling interleaved as production does), and each run phased
//! (`shipped` through the phase split's clone, the others through their own `PHASED` form) for
//! the stage columns. Twelve slots a round, the order rotated round by round, two warm-up rounds;
//! each arm keeps its buffers from round to round, as a worker does.
//!
//! Writes `OUT-totals.csv` (each arm's whole time, and its per-round ratio to `shipped`),
//! `OUT-stages.csv` (the phase split's columns per arm) and `OUT-counters.csv` (sizes and live
//! bytes). `FUSED_ONLY=shipped|F|Q|FQ|Qi|FQi` runs that arm whole twice per block (one warm-up),
//! unchecked and untimed, for cachegrind.

use std::time::Instant;

use mibz2::runs::Runs;
use mibz2::{PhasedOpen, Phases, ResearchOpen};

fn primary(manifest: &str, file: &str) -> u32 {
    let at = manifest
        .find(&format!("\"file\": \"{file}\""))
        .expect("block in manifest");
    let rest = &manifest[at..];
    let k = rest.find("\"primary_index\":").unwrap() + "\"primary_index\":".len();
    rest[k..]
        .trim_start()
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap()
        .parse()
        .unwrap()
}

fn quantile(v: &[f64], q: f64) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let i = ((v.len() - 1) as f64 * q).round() as usize;
    v[i]
}

fn dense(ll: &[u8], p: u32) -> Vec<u8> {
    let mut counts = [0u32; 256];
    for &c in ll {
        counts[c as usize] += 1;
    }
    let mut start = [0u32; 256];
    let mut acc = 0;
    for c in 0..256 {
        start[c] = acc;
        acc += counts[c];
    }
    let mut t = vec![0u32; ll.len()];
    for (i, &c) in ll.iter().enumerate() {
        t[start[c as usize] as usize] = i as u32;
        start[c as usize] += 1;
    }
    let mut out = Vec::with_capacity(ll.len());
    let mut j = t[p as usize];
    for _ in 0..ll.len() {
        out.push(ll[j as usize]);
        j = t[j as usize];
    }
    out
}

const STAGES: [&str; 8] = [
    "a_runs", "b_image", "c1_nodes", "c2_elim", "c3_bands", "d_spell", "e_wire", "f_emit",
];
const ARMS: [&str; 6] = ["shipped", "F", "Q", "FQ", "Qi", "FQi"];

fn stage_ns(p: &Phases) -> [u64; 8] {
    [
        p.runs_ns, p.image_ns, p.nodes_ns, p.elim_ns, p.bands_ns, p.spell_ns, p.wire_ns, p.emit_ns,
    ]
}

#[derive(Default)]
struct Arm {
    r: ResearchOpen,
    phased: PhasedOpen,
}

impl Arm {
    fn whole(&mut self, i: usize, recs: &[u16], n: usize, p: u32, out: &mut Vec<u8>) {
        match i {
            0 => self.r.run_shipped(recs, n, p, out),
            1 => {
                self.r.run::<true, false, false, false>(recs, n, p, out);
            }
            2 => {
                self.r.run::<false, true, false, false>(recs, n, p, out);
            }
            3 => {
                self.r.run::<true, true, false, false>(recs, n, p, out);
            }
            4 => {
                self.r.run::<false, false, false, true>(recs, n, p, out);
            }
            _ => {
                self.r.run::<true, false, false, true>(recs, n, p, out);
            }
        }
    }
    fn phased(&mut self, i: usize, recs: &[u16], n: usize, p: u32, out: &mut Vec<u8>) -> Phases {
        match i {
            0 => self.phased.run_phased::<true>(recs, n, p, out),
            1 => self.r.run::<true, false, true, false>(recs, n, p, out),
            2 => self.r.run::<false, true, true, false>(recs, n, p, out),
            3 => self.r.run::<true, true, true, false>(recs, n, p, out),
            4 => self.r.run::<false, false, true, true>(recs, n, p, out),
            _ => self.r.run::<true, false, true, true>(recs, n, p, out),
        }
    }
}

/// Each arm's construction against shipped Open's, field by field
fn check_state(recs: &[u16], n: usize, p: u32) {
    let mut shipped = ResearchOpen::default();
    let want = shipped.construct_shipped(recs, n, p);
    let s = shipped.state();
    let mut r = ResearchOpen::default();
    let mut ph = Phases::default();
    let n32 = n as u32;
    let lim = 16;
    let sp = mibz2::Spill::default();
    for (i, label) in [
        "F",
        "Q",
        "FQ",
        "F/phased",
        "Q/phased",
        "FQ/phased",
        "Qi",
        "FQi",
        "Qi/phased",
        "FQi/phased",
    ]
    .iter()
    .enumerate()
    {
        let (_, mark, bits) = match i {
            0 => r.construct::<true, false, false, false>(recs, n32, p, lim, sp, &mut ph),
            1 => r.construct::<false, true, false, false>(recs, n32, p, lim, sp, &mut ph),
            2 => r.construct::<true, true, false, false>(recs, n32, p, lim, sp, &mut ph),
            3 => r.construct::<true, false, true, false>(recs, n32, p, lim, sp, &mut ph),
            4 => r.construct::<false, true, true, false>(recs, n32, p, lim, sp, &mut ph),
            5 => r.construct::<true, true, true, false>(recs, n32, p, lim, sp, &mut ph),
            6 => r.construct::<false, false, false, true>(recs, n32, p, lim, sp, &mut ph),
            7 => r.construct::<true, false, false, true>(recs, n32, p, lim, sp, &mut ph),
            8 => r.construct::<false, false, true, true>(recs, n32, p, lim, sp, &mut ph),
            _ => r.construct::<true, false, true, true>(recs, n32, p, lim, sp, &mut ph),
        };
        let t = r.state();
        assert_eq!((mark, bits), want, "{label}: mark, bits");
        assert!(t.edges == s.edges, "{label}: edges");
        assert!(t.extra == s.extra, "{label}: extra");
        assert!(t.spelling == s.spelling, "{label}: spelling");
        assert!(t.offsets == s.offsets, "{label}: offsets");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(&args[1]);
    let rounds: usize = args.get(2).map_or(41, |s| s.parse().unwrap());
    let out_prefix = args.get(3).cloned().unwrap_or_else(|| "open-fused".into());
    let ids: Vec<String> = if args.len() > 4 {
        args[4..].to_vec()
    } else {
        ["0", "2", "85", "143", "207", "210"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    let manifest = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    let only = std::env::var("FUSED_ONLY").ok();

    let mut totals_csv = String::from(
        "block,arm,median_ms,p10_ms,p90_ms,ratio_median,ratio_p10,ratio_p90,ratio_of_medians\n",
    );
    let mut stages_csv = String::from(
        "block,arm,stage,median_ms,p10_ms,p90_ms,share_median_pct,share_p10_pct,share_p90_pct\n",
    );
    let mut counters_csv = String::from(
        "block,arm,n,retained,runs,strips,queued_u32,bands,words,edges,extra,queue_bytes,live_bytes\n",
    );
    for id in &ids {
        let file = format!("block-{id}.L");
        let ll = std::fs::read(dir.join(&file)).unwrap();
        let runs = Runs::of_column(&ll);
        let recs = runs.records();
        let p = primary(&manifest, &file);
        let n = ll.len();

        if let Some(arm) = &only {
            let i = ARMS.iter().position(|a| a == arm).expect("FUSED_ONLY arm");
            let mut a = Arm::default();
            let mut out = Vec::new();
            for _ in 0..2 {
                a.whole(i, recs, n, p, &mut out);
            }
            eprintln!("block {id}: {arm} run twice (one warm-up)");
            continue;
        }

        check_state(recs, n, p);
        let truth = dense(&ll, p);
        // slots 0..A whole, A..2A phased
        const A: usize = ARMS.len();
        let mut arms: Vec<Arm> = (0..2 * A).map(|_| Arm::default()).collect();
        let mut outs: Vec<Vec<u8>> = vec![Vec::new(); 2 * A];
        let mut whole_ms = vec![Vec::new(); A];
        let mut phased: Vec<Vec<Phases>> = vec![Vec::new(); A];
        for round in 0..rounds + 2 {
            for j in 0..2 * A {
                let slot = (j + round) % (2 * A);
                let arm = slot % A;
                let out = &mut outs[slot];
                let a = &mut arms[slot];
                let t = Instant::now();
                let ph = if slot < A {
                    a.whole(arm, recs, n, p, out);
                    None
                } else {
                    Some(a.phased(arm, recs, n, p, out))
                };
                let dt = t.elapsed().as_secs_f64() * 1e3;
                assert!(
                    *out == truth,
                    "block {id} slot {slot} round {round}: output differs"
                );
                if round >= 2 {
                    match ph {
                        None => whole_ms[arm].push(dt),
                        Some(ph) => phased[arm].push(ph),
                    }
                }
            }
        }
        eprintln!(
            "block {id}: n={n} p={p}: state equal to shipped for F, Q, FQ, Qi, FQi; 12 slots byte-exact every round"
        );
        let base = &whole_ms[0];
        for (i, label) in ARMS.iter().enumerate() {
            let ms = &whole_ms[i];
            let ratio: Vec<f64> = ms.iter().zip(base).map(|(a, b)| a / b).collect();
            totals_csv += &format!(
                "{id},{label},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
                quantile(ms, 0.5),
                quantile(ms, 0.1),
                quantile(ms, 0.9),
                quantile(&ratio, 0.5),
                quantile(&ratio, 0.1),
                quantile(&ratio, 0.9),
                quantile(ms, 0.5) / quantile(base, 0.5)
            );
            let ph = &phased[i];
            let totals: Vec<f64> = ph
                .iter()
                .map(|x| stage_ns(x).iter().sum::<u64>() as f64 / 1e6)
                .collect();
            for (s, name) in STAGES.iter().enumerate() {
                let ms: Vec<f64> = ph.iter().map(|x| stage_ns(x)[s] as f64 / 1e6).collect();
                let share: Vec<f64> = ms.iter().zip(&totals).map(|(a, t)| 100.0 * a / t).collect();
                stages_csv += &format!(
                    "{id},{label},{name},{:.4},{:.4},{:.4},{:.2},{:.2},{:.2}\n",
                    quantile(&ms, 0.5),
                    quantile(&ms, 0.1),
                    quantile(&ms, 0.9),
                    quantile(&share, 0.5),
                    quantile(&share, 0.1),
                    quantile(&share, 0.9)
                );
            }
            stages_csv += &format!(
                "{id},{label},total_phased,{:.4},{:.4},{:.4},100,100,100\n",
                quantile(&totals, 0.5),
                quantile(&totals, 0.1),
                quantile(&totals, 0.9)
            );
            let x = ph.last().unwrap();
            let c = &x.counts;
            counters_csv += &format!(
                "{id},{label},{n},{},{},{},{},{},{},{},{},{},{}\n",
                c.retained,
                c.runs,
                x.strips,
                x.queued,
                x.bands,
                c.words,
                x.edges,
                x.extra,
                x.queue_bytes,
                x.live_bytes
            );
        }
    }
    if only.is_none() {
        std::fs::write(format!("{out_prefix}-totals.csv"), &totals_csv).unwrap();
        std::fs::write(format!("{out_prefix}-stages.csv"), &stages_csv).unwrap();
        std::fs::write(format!("{out_prefix}-counters.csv"), &counters_csv).unwrap();
        print!("{totals_csv}");
        print!("{stages_csv}");
        print!("{counters_csv}");
    }
}
