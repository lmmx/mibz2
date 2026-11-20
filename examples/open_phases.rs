//! Research only (`open-phases`): where the shipped open kernel spends its time inside
//! construction, on exact BWT columns, on one thread
//! (docs/journal/2026-11-20-open-phase-split.md). Never on a production path.
//!
//! ```text
//! open_phases DIR ROUNDS OUT [BLOCK...]   # DIR holds block-<id>.L and manifest.json
//! ```
//!
//! Writes `OUT-stages.csv` (each stage's median and spread per block and arm, and the share of
//! the block's total) and `OUT-counters.csv` (the sizes each stage leaves, and the live bytes).
//!
//! **Arms:** `qi16` (`Kernel::run::<true, true>`, the shipped instance) and `i16`
//! (`<true, false>`, direct sequential wiring), each run two ways: as production runs it,
//! timed whole (`prod`), and as the timed clone (`src/open/endpoint/phases.rs`), stage by stage
//! (`phased`). Every output of every round is checked byte for byte against a dense-LF walk of
//! the column, outside the timed span. Rounds run the four arms once each, the order rotated
//! round by round, after two warm-up rounds; each arm keeps its buffers from round to round, as
//! a worker does from block to block.
//!
//! **Backward Open** (docs/journal/2026-11-20-open-backward-native.md): two more arms, `bwd`
//! (`PhasedOpen::run_backward::<false>`, the worker's output buffer reused) and `bwdz`
//! (`::<true>`, the whole output zero-filled every block), rotated with the four above. The
//! per-round wall and stage times of all six arms go to `OUT-rounds.csv`.
//!
//! `PHASES_ONLY=qi16|i16|bwd|none` runs that arm's clone once per block after one warm-up,
//! unchecked and untimed (for callgrind: its stage functions are `#[inline(never)]`; `none`
//! loads the blocks only, the baseline to subtract).
//!
//! `PHASES_MT=W` (with ROUNDS) instead runs W worker threads, each with its own kernel and its
//! own replica of the block, the arms `qi16` (production), `i16` (production) and `bwd` in
//! turn, rotated round by round, the workers released together by a barrier; each worker times
//! a batch of 20 decodes, and each round's figure (the slowest worker's, per decode) goes to
//! `OUT-mt.csv`.

use std::time::Instant;

use mibz2::runs::Runs;
use mibz2::{PhasedOpen, Phases};
use std::sync::Barrier;

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

/// The reference: a dense LF table walked from the primary row (bzip2's convention)
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

fn stage_ns(p: &Phases) -> [u64; 8] {
    [
        p.runs_ns, p.image_ns, p.nodes_ns, p.elim_ns, p.bands_ns, p.spell_ns, p.wire_ns, p.emit_ns,
    ]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = std::path::PathBuf::from(&args[1]);
    let rounds: usize = args.get(2).map_or(41, |s| s.parse().unwrap());
    let out_prefix = args.get(3).cloned().unwrap_or_else(|| "open-phases".into());
    let ids: Vec<String> = if args.len() > 4 {
        args[4..].to_vec()
    } else {
        ["0", "2", "85", "143", "207", "210"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    let manifest = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    let only = std::env::var("PHASES_ONLY").ok();
    if let Ok(w) = std::env::var("PHASES_MT") {
        multi(
            &dir,
            &manifest,
            &ids,
            rounds,
            w.parse().unwrap(),
            &out_prefix,
        );
        return;
    }
    let mut rounds_csv = String::from(
        "block,round,arm,wall_ms,a_runs_ms,b_image_ms,c1_nodes_ms,c2_elim_ms,c3_bands_ms,d_spell_ms,e_wire_ms,f_emit_ms\n",
    );

    let mut stages_csv = String::from(
        "block,arm,stage,median_ms,p10_ms,p90_ms,share_median_pct,share_p10_pct,share_p90_pct\n",
    );
    let mut counters_csv = String::from(
        "block,arm,n,retained,runs,strips,queued_u32,open_nodes,steps,rules,bands,words,spelling_bytes,offsets,edges,extra,table_bytes,live_bytes,\
a_bytes_written,b_bytes_written,c1_bytes_written,c2_bytes_written,c3_bytes_written,d_bytes_written,e_bytes_written,f_bytes_written\n",
    );
    for id in &ids {
        let file = format!("block-{id}.L");
        let ll = std::fs::read(dir.join(&file)).unwrap();
        let runs = Runs::of_column(&ll);
        let recs = runs.records();
        let p = primary(&manifest, &file);
        let n = ll.len();

        if let Some(arm) = &only {
            let mut k = PhasedOpen::default();
            let mut out = Vec::new();
            for _ in 0..2 {
                match arm.as_str() {
                    "qi16" => k.run_phased::<true>(recs, n, p, &mut out),
                    "i16" => k.run_phased::<false>(recs, n, p, &mut out),
                    "bwd" => k.run_backward::<false>(recs, n, p, &mut out),
                    "none" => Phases::default(),
                    _ => panic!("PHASES_ONLY is qi16, i16, bwd or none"),
                };
            }
            eprintln!("block {id}: {arm} clone run twice (one warm-up)");
            continue;
        }

        let truth = dense(&ll, p);
        // arms: 0 qi16 prod, 1 qi16 phased, 2 i16 prod, 3 i16 phased, 4 bwd, 5 bwdz
        const ARMS: usize = 6;
        const NAMES: [&str; ARMS] = [
            "qi16_prod",
            "qi16_phased",
            "i16_prod",
            "i16_phased",
            "bwd",
            "bwdz",
        ];
        let mut ks: Vec<PhasedOpen> = (0..ARMS).map(|_| PhasedOpen::default()).collect();
        let mut outs: Vec<Vec<u8>> = vec![Vec::new(); ARMS];
        let mut prod_ms = vec![Vec::new(); ARMS];
        let mut phased: Vec<Vec<Phases>> = vec![Vec::new(); ARMS];
        for round in 0..rounds + 2 {
            for j in 0..ARMS {
                let i = (j + round) % ARMS;
                let out = &mut outs[i];
                let k = &mut ks[i];
                let t = Instant::now();
                let ph = match i {
                    0 => {
                        k.run_production::<true>(recs, n, p, out);
                        None
                    }
                    1 => Some(k.run_phased::<true>(recs, n, p, out)),
                    2 => {
                        k.run_production::<false>(recs, n, p, out);
                        None
                    }
                    3 => Some(k.run_phased::<false>(recs, n, p, out)),
                    4 => Some(k.run_backward::<false>(recs, n, p, out)),
                    _ => Some(k.run_backward::<true>(recs, n, p, out)),
                };
                let dt = t.elapsed().as_secs_f64() * 1e3;
                // the check, outside the timed span, every round
                assert!(
                    *out == truth,
                    "block {id} arm {i} round {round}: output differs"
                );
                if round >= 2 {
                    let st = ph.map(|x| stage_ns(&x)).unwrap_or_default();
                    rounds_csv += &format!("{id},{},{},{dt:.5}", round - 2, NAMES[i]);
                    for v in st {
                        rounds_csv += &format!(",{:.5}", v as f64 / 1e6);
                    }
                    rounds_csv += "\n";
                    prod_ms[i].push(dt);
                    if let Some(ph) = ph {
                        phased[i].push(ph);
                    }
                }
            }
        }
        eprintln!(
            "block {id}: n={n} p={p} records={}: 6 arms byte-exact on every round",
            recs.len()
        );
        for (i, label) in [(1usize, "qi16"), (3, "i16"), (4, "bwd"), (5, "bwdz")] {
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
            // the backward arms have no production twin: their wall is the clone's own
            let prod = &prod_ms[if i <= 3 { i - 1 } else { i }];
            let wrapped = &prod_ms[i];
            let ratio: Vec<f64> = totals
                .iter()
                .zip(prod)
                .map(|(a, b)| 100.0 * a / b)
                .collect();
            stages_csv += &format!(
                "{id},{label},total_production,{:.4},{:.4},{:.4},{:.2},{:.2},{:.2}\n",
                quantile(prod, 0.5),
                quantile(prod, 0.1),
                quantile(prod, 0.9),
                quantile(&ratio, 0.5),
                quantile(&ratio, 0.1),
                quantile(&ratio, 0.9)
            );
            stages_csv += &format!(
                "{id},{label},total_phased_wall,{:.4},{:.4},{:.4},,,\n",
                quantile(wrapped, 0.5),
                quantile(wrapped, 0.1),
                quantile(wrapped, 0.9)
            );
            let x = ph.last().unwrap();
            let c = &x.counts;
            // bytes each stage writes (elements times their size): a the strips (16 B) and the
            // queues (4 B); b image_order (4 B); c1 the nodes (24 B) and the four order lists
            // (4 B); c2 the rules (16 B); c3 the bottoms (4 B) and the bands (12 B); d the
            // spelling (1 B) and offsets (4 B); e edges and extra (4 B); f the output (1 B)
            let order_lists = 2 * (x.nodes - 1);
            let wb = [
                16 * x.strips + 4 * x.queued,
                4 * x.strips,
                24 * x.nodes + 4 * order_lists,
                16 * x.rules,
                4 * x.nodes + 12 * x.bands,
                x.spelling + 4 * x.offsets,
                4 * (x.edges + x.extra),
                n,
            ];
            counters_csv += &format!(
                "{id},{label},{n},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                c.retained,
                c.runs,
                x.strips,
                x.queued,
                c.open_nodes,
                c.steps,
                x.rules,
                x.bands,
                c.words,
                x.spelling,
                x.offsets,
                x.edges,
                x.extra,
                c.table_bytes,
                x.live_bytes,
                wb[0],
                wb[1],
                wb[2],
                wb[3],
                wb[4],
                wb[5],
                wb[6],
                wb[7]
            );
        }
    }
    if only.is_none() {
        std::fs::write(format!("{out_prefix}-stages.csv"), &stages_csv).unwrap();
        std::fs::write(format!("{out_prefix}-counters.csv"), &counters_csv).unwrap();
        std::fs::write(format!("{out_prefix}-rounds.csv"), &rounds_csv).unwrap();
        print!("{stages_csv}");
        print!("{counters_csv}");
    }
}

/// `PHASES_MT`: W workers, each its own kernel and block replica, the three arms in turn; in each
/// round each worker times its own batch of `BATCH` decodes of an arm, after a barrier, and the
/// round's figure is the slowest worker's batch (per decode)
fn multi(
    dir: &std::path::Path,
    manifest: &str,
    ids: &[String],
    rounds: usize,
    w: usize,
    out: &str,
) {
    const BATCH: usize = 20;
    let mut csv = String::from("block,round,arm,workers,wall_ms_per_decode\n");
    for id in ids {
        let file = format!("block-{id}.L");
        let ll = std::fs::read(dir.join(&file)).unwrap();
        let p = primary(manifest, &file);
        let n = ll.len();
        let truth = dense(&ll, p);
        let replicas: Vec<Vec<u16>> = (0..w)
            .map(|_| Runs::of_column(&ll).records().to_vec())
            .collect();
        let barrier = Barrier::new(w);
        let per_worker: Vec<Vec<(usize, usize, f64)>> = std::thread::scope(|s| {
            let hs: Vec<_> = replicas
                .iter()
                .map(|recs| {
                    let (barrier, truth) = (&barrier, &truth);
                    s.spawn(move || {
                        let mut ks: Vec<PhasedOpen> =
                            (0..3).map(|_| PhasedOpen::default()).collect();
                        let mut outs = vec![Vec::new(); 3];
                        let mut times = Vec::new();
                        for round in 0..rounds + 1 {
                            for j in 0..3 {
                                let i = (j + round) % 3;
                                barrier.wait();
                                let t = Instant::now();
                                for _ in 0..BATCH {
                                    match i {
                                        0 => ks[0].run_production::<true>(recs, n, p, &mut outs[0]),
                                        1 => {
                                            ks[1].run_production::<false>(recs, n, p, &mut outs[1])
                                        }
                                        _ => {
                                            ks[2].run_backward::<false>(recs, n, p, &mut outs[2]);
                                        }
                                    }
                                }
                                let ms = t.elapsed().as_secs_f64() * 1e3 / BATCH as f64;
                                assert!(outs[i] == *truth, "block {id} mt arm {i}: output differs");
                                if round >= 1 {
                                    times.push((round - 1, i, ms));
                                }
                            }
                        }
                        times
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        for (k, &(round, i, _)) in per_worker[0].iter().enumerate() {
            let ms = per_worker.iter().map(|t| t[k].2).fold(0.0, f64::max);
            let arm = ["qi16_prod", "i16_prod", "bwd"][i];
            csv += &format!("{id},{round},{arm},{w},{ms:.5}\n");
        }
        eprintln!("block {id}: {w} workers, 3 arms byte-exact on every round");
    }
    std::fs::write(format!("{out}-mt.csv"), &csv).unwrap();
}
