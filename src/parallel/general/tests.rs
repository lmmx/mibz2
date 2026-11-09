use super::*;
use std::path::PathBuf;

const MIB: u64 = 1 << 20;

/// 0.2.6's policy, written out again from the journal's `today` (plan_tables.py), not from
/// `plan_with`: the reference the default is checked against
fn today_026(n: usize, phys: Option<usize>, l3: Option<u64>) -> (Plan, Mode) {
    let Some(phys) = phys else {
        return (Plan::Workers(n), Mode::Plain);
    };
    let c = n.min(phys);
    if c < 6 {
        return (Plan::Workers(n), Mode::Plain);
    }
    if phys == 10 && l3 == Some(20 * MIB) && n >= 10 {
        return (
            Plan::Teams { teams: 3, k: 3 },
            Mode::Pipelined(Levers {
                park: true,
                small: true,
                spin2k: false,
            }),
        );
    }
    let cap = match l3 {
        None => 4,
        Some(b) => ((0.7 * b as f64 / 3.6e6).round() as usize).max(1),
    };
    (
        Plan::Teams {
            teams: (c / 2).min(cap).max(1),
            k: 2,
        },
        Mode::Plain,
    )
}

/// The grid's L3 sizes: unread, 2-128 MiB in 1 MiB steps, and the edges
fn l3_grid() -> Vec<Option<u64>> {
    let mut v = vec![None];
    v.extend((2..=128).map(|m| Some(m * MIB)));
    v.extend([Some(77 * MIB / 4), Some(20 * MIB - 1), Some(20 * MIB + 1)]);
    v
}

/// The threads asked for on `phys` cores: 1 to 2P (to 128 where unread), 128 and the most
fn n_grid(phys: Option<usize>) -> impl Iterator<Item = usize> {
    let top = phys.map_or(128, |p| 2 * p);
    (1..=top).chain([128, usize::MAX])
}

fn used(p: Plan) -> usize {
    p.threads()
}

/// The default (`Plan::for_threads`'s `plan_with`, and `StreamDecoder::defaulted`'s mode)
/// is 0.2.6's logic everywhere on the grid; so is phase 2's step-1 form (`plan_enveloped`),
/// whatever the sharing and core kinds, and whether the affinity's L3 reads as cpu0's or not
#[test]
fn the_default_is_unchanged() {
    let mut points = 0usize;
    let mut plans = BTreeSet::new();
    let phys_grid = std::iter::once(None).chain((1..=64).map(Some));
    for phys in phys_grid {
        for l3 in l3_grid() {
            for n in n_grid(phys) {
                let want = today_026(n, phys, l3);
                assert_eq!(
                    plan_with(n, phys, l3),
                    want.0,
                    "plan_with {phys:?} {l3:?} {n}"
                );
                if let Plan::Teams { .. } = want.0 {
                    plans.insert(format!("{:?}", want));
                }
                for shared in [true, false] {
                    for hybrid in [true, false] {
                        for own_l3 in [l3, None, Some(7 * MIB)] {
                            let mut t = Topology::synthetic(phys, l3, shared, hybrid);
                            t.l3 = own_l3;
                            assert_eq!(plan_today(&t, n), want, "today {t:?} {n}");
                            assert_eq!(plan_enveloped(&t, n), want, "enveloped {t:?} {n}");
                            points += 1;
                        }
                    }
                }
            }
        }
    }
    // (the grid's size, for the record: printed with --nocapture)
    println!(
        "equivalence grid: {points} points, {} distinct team plans",
        plans.len()
    );
    assert!(points > 4_000_000, "{points}");
    // the mode: `defaulted` itself, through the default constructor, on every distinct
    // plan the grid gives (and the others rule A gives), pipelined at 3 × 3 alone
    let mut distinct = BTreeSet::new();
    for phys in (6..=64).map(Some) {
        for l3 in l3_grid() {
            for n in n_grid(phys).filter(|&n| n <= 128) {
                let (p, _) = today_026(n, phys, l3);
                if let Plan::Teams { teams, k } = p {
                    distinct.insert((teams, k));
                }
                if let Plan::Teams { teams, k } =
                    rule_a(&Topology::synthetic(phys, l3, true, false), n).plan
                {
                    distinct.insert((teams, k));
                }
            }
        }
    }
    for (teams, k) in distinct {
        let plan = Plan::Teams { teams, k };
        let dec = super::super::with_plan(plan, || StreamDecoder::new(2, 4, Kernels::detect()));
        assert_eq!(dec.is_pipelined(), plan == PIPELINED_PLAN, "{plan:?}");
        assert_eq!(
            dec.is_pipelined(),
            today_mode(plan) != Mode::Plain,
            "{plan:?}"
        );
        dec.close();
    }
}

/// FNV-1a, 64 bits, as rule_a_grid.py's
fn fnv1a64(data: &[u8]) -> u64 {
    data.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

fn code(p: Plan, m: Mode) -> String {
    match p {
        Plan::Workers(_) => "w".into(),
        Plan::Teams { teams, k } => {
            format!(
                "{teams}x{k}{}",
                if matches!(m, Mode::Pipelined(_)) {
                    "p"
                } else {
                    ""
                }
            )
        }
    }
}

/// Rule A matches plan_tables.py's `rule_a` on its grid: the whole table's digest and line
/// count (tests/data/rule_a_grid.txt, written by the journal's rule_a_grid.py), and, line by
/// line, every topology where the fast plan fires
#[test]
fn rule_a_matches_plan_tables() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/rule_a_grid.txt");
    let file = std::fs::read_to_string(path).unwrap();
    let field = |key: &str| {
        file.lines()
            .find_map(|l| l.strip_prefix(key))
            .unwrap()
            .trim()
            .to_string()
    };
    let (want_lines, want_digest) = (field("lines "), field("fnv1a64 "));
    let want_fast: Vec<&str> = file
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("lines") && !l.starts_with("fnv"))
        .collect();
    let mut text = String::new();
    let (mut lines, mut fast) = (0, Vec::new());
    for phys in 1..=64usize {
        for l3 in l3_grid() {
            for shared in [true, false] {
                let t = Topology::synthetic(Some(phys), l3, shared, false);
                let mut runs: Vec<(usize, usize, String)> = Vec::new();
                for n in 1..=2 * phys {
                    let d = rule_a(&t, n);
                    let c = code(d.plan, d.mode);
                    // (the branch names the step; pipelined iff the fast plan)
                    assert_eq!(d.branch == Branch::FastPlan, c.ends_with('p'), "{t:?} {n}");
                    match runs.last_mut() {
                        Some(r) if r.2 == c => r.1 = n,
                        _ => runs.push((n, n, c)),
                    }
                }
                let cells: Vec<String> = runs
                    .iter()
                    .map(|(a, b, c)| format!("{a}-{b}:{c}"))
                    .collect();
                let l3s = l3.map_or("-".to_string(), |b| b.to_string());
                text += &format!("{phys} {l3s} {}  {}\n", shared as u8, cells.join(" "));
                lines += 1;
                if let (true, Some((a, _, c))) = (shared, runs.last()) {
                    if c.ends_with('p') {
                        fast.push(format!("{phys} {} {a} {}", l3.unwrap(), &c[..c.len() - 3]));
                    }
                }
            }
        }
    }
    assert_eq!(fast, want_fast, "the fast plan's topologies");
    assert_eq!(lines.to_string(), want_lines);
    assert_eq!(
        format!("{:016x}", fnv1a64(text.as_bytes())),
        want_digest,
        "the whole table"
    );
    // the journal's tables (§3), pinned
    let a = |phys, l3: u64, n| {
        let d = rule_a(
            &Topology::synthetic(Some(phys), Some(l3 * MIB), true, false),
            n,
        );
        code(d.plan, d.mode)
    };
    let i9: Vec<String> = (4..=20).map(|n| a(10, 20, n)).collect();
    assert_eq!(
        i9,
        [
            "w", "w", "3x2", "3x2", "4x2", "3x3p", "3x3p", "3x3p", "3x3p", "3x3p", "3x3p", "3x3p",
            "3x3p", "3x3p", "3x3p", "3x3p", "3x3p"
        ]
    );
    for (cores, l3, want) in [
        (6, 8, "2x2"),
        (8, 16, "2x3p"),
        (8, 20, "4x2"),
        (10, 16, "2x3p"),
        (10, 32, "5x2"),
        (12, 20, "3x3p"),
        (12, 32, "6x2"),
        (16, 20, "3x3p"),
        (16, 32, "5x3p"),
        (16, 64, "8x2"),
        (32, 32, "5x3p"),
        (32, 64, "13x2"),
    ] {
        assert_eq!(a(cores, l3, 2 * cores), want, "{cores} cores, {l3} MiB");
    }
}

/// Rule A's invariants on the grid: never more team threads than c, at least one team,
/// pipelined iff the width is 3 or more, and the threads used never fall as N rises in the
/// team region (the workers-to-teams step exempt, as 0.2.6's own); the same of the general
/// policy with its envelope
#[test]
fn rule_a_invariants() {
    let (mut checked, mut exempt) = (0usize, 0usize);
    for phys in std::iter::once(None).chain((1..=64).map(Some)) {
        for l3 in l3_grid() {
            for shared in [true, false] {
                for hybrid in [true, false] {
                    let t = Topology::synthetic(phys, l3, shared, hybrid);
                    for policy in [rule_a, decide_general] {
                        let mut prev: Option<Plan> = None;
                        for n in n_grid(phys) {
                            let d = policy(&t, n);
                            match d.plan {
                                Plan::Teams { teams, k } => {
                                    let c = n.min(phys.unwrap());
                                    assert!(teams >= 1 && teams * k <= c, "{t:?} {n}: {d:?}");
                                    assert_eq!(matches!(d.mode, Mode::Pipelined(_)), k >= 3);
                                    if let Mode::Pipelined(l) = d.mode {
                                        assert_eq!(l, PIPELINED_LEVERS);
                                    }
                                }
                                Plan::Workers(w) => {
                                    assert_eq!((w, d.mode), (n, Mode::Plain));
                                }
                            }
                            if let Some(p) = prev {
                                if used(d.plan) < used(p) {
                                    // only at the step from workers to teams
                                    assert!(
                                        matches!(p, Plan::Workers(_)),
                                        "{t:?} {n}: {p:?} to {d:?}"
                                    );
                                    assert!(matches!(d.plan, Plan::Teams { .. }));
                                    exempt += 1;
                                }
                            }
                            prev = Some(d.plan);
                            checked += 1;
                        }
                    }
                }
            }
        }
    }
    println!("invariants: {checked} points, {exempt} workers-to-teams steps");
    // unread inputs never panic
    for n in [0, 1, 9, usize::MAX] {
        let _ = decide_general(&Topology::default(), n);
        let _ = rule_a(
            &Topology::synthetic(Some(usize::MAX), Some(u64::MAX), true, false),
            n,
        );
    }
}

/// The envelope keeps 0.2.6's choice where its topology matches as 0.2.6 reads it, before
/// the guards; outside it, rule A
#[test]
fn the_envelope() {
    let i9 = Topology::synthetic(Some(10), Some(20 * MIB), true, false);
    for n in 10..=20 {
        let d = decide_general(&i9, n);
        assert_eq!((d.plan, d.branch), (PIPELINED_PLAN, Branch::Envelope));
        assert_eq!(plan_general(&i9, n), plan_today(&i9, n));
    }
    // at 9, rule A's one i9 change from 0.2.6
    assert_eq!(decide_general(&i9, 9).branch, Branch::FastPlan);
    assert_eq!(plan_general(&i9, 9).0, PIPELINED_PLAN);
    assert_eq!(plan_today(&i9, 9).0, Plan::Teams { teams: 4, k: 2 });
    // a hybrid part or a split L3 matching 0.2.6's exception: the envelope keeps 3 × 3
    // where rule A alone would guard to the old plan
    for t in [
        Topology::synthetic(Some(10), Some(20 * MIB), true, true),
        Topology::synthetic(Some(10), Some(20 * MIB), false, false),
    ] {
        assert_eq!(decide_general(&t, 10).branch, Branch::Envelope);
        let a = rule_a(&t, 10);
        assert_eq!(
            (a.plan, a.branch),
            (Plan::Teams { teams: 4, k: 2 }, Branch::OldPlan)
        );
        assert_eq!(decide_general(&t, 9).plan, Plan::Teams { teams: 4, k: 2 });
    }
}

/// A fake sysfs root, written under the temporary directory
struct Sys(PathBuf);

impl Sys {
    fn new(name: &str) -> Sys {
        let d = std::env::temp_dir().join(format!("mibz2-sysfs-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        Sys(d)
    }
    fn put(&self, rel: &str, s: &str) -> &Self {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, format!("{s}\n")).unwrap();
        self
    }
    /// a CPU: its package and core, its L3 (size and sharers, as index3 with its level),
    /// and optionally its max frequency (kHz) and capacity, `(freq, cap)`
    fn cpu(
        &self,
        c: usize,
        pkg: usize,
        core: usize,
        l3: &str,
        shared: &str,
        (freq, cap): (Option<u64>, Option<u64>),
    ) -> &Self {
        let b = format!("devices/system/cpu/cpu{c}");
        self.put(
            &format!("{b}/topology/physical_package_id"),
            &pkg.to_string(),
        );
        self.put(&format!("{b}/topology/core_id"), &core.to_string());
        for (i, level) in [(0, "1"), (1, "1"), (2, "2")] {
            self.put(&format!("{b}/cache/index{i}/level"), level);
            self.put(&format!("{b}/cache/index{i}/size"), "48K");
        }
        self.put(&format!("{b}/cache/index3/level"), "3");
        self.put(&format!("{b}/cache/index3/size"), l3);
        self.put(&format!("{b}/cache/index3/shared_cpu_list"), shared);
        if let Some(f) = freq {
            self.put(&format!("{b}/cpufreq/cpuinfo_max_freq"), &f.to_string());
        }
        if let Some(c) = cap {
            self.put(&format!("{b}/cpu_capacity"), &c.to_string());
        }
        self
    }
}

impl Drop for Sys {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn all(n: usize) -> Vec<usize> {
    (0..n).collect()
}

/// The i9-10900K: 10 cores (core ids 0-4 and 8-12, as it reports them), 2 CPUs each
/// (siblings c and c + 10), one 20 MiB L3, two favoured cores at 5.3 GHz and the rest 5.1
#[test]
fn fixture_i9() {
    let s = Sys::new("i9");
    let ids = [0, 1, 2, 3, 4, 8, 9, 10, 11, 12];
    for c in 0..20 {
        let f = if ids[c % 10] == 2 || ids[c % 10] == 9 {
            5_300_000
        } else {
            5_100_000
        };
        s.cpu(c, 0, ids[c % 10], "20480K", "0-19", (Some(f), None));
    }
    s.put("devices/cpu/type", "4");
    let t = Topology::read(&s.0, &all(20));
    assert_eq!(
        (t.cpus, t.phys, t.l3, t.l3_cpu0),
        (20, Some(10), Some(20 * MIB), Some(20 * MIB))
    );
    assert_eq!(
        (t.l3_shared, t.intel_hybrid, t.capacity_differs),
        (Some(true), None, None)
    );
    assert!((t.freq_spread.unwrap() - 0.2 / 5.1).abs() < 1e-9);
    assert_eq!(t.hybrid(), Some(false));
    assert!(t.fast_eligible());
    assert_eq!(plan_general(&t, 9).0, PIPELINED_PLAN);
    assert_eq!(plan_general(&t, 20), plan_today(&t, 20));
    // under a mask of one CPU on each of 4 cores: 4 cores, workers
    let m = Topology::read(&s.0, &[0, 1, 2, 3]);
    assert_eq!((m.cpus, m.phys, m.l3_shared), (4, Some(4), Some(true)));
    assert_eq!(plan_general(&m, 4).0, Plan::Workers(4));
    // an offline CPU in the affinity is skipped, as `physical_cores` skips it
    std::fs::remove_file(s.0.join("devices/system/cpu/cpu19/topology/core_id")).unwrap();
    assert_eq!(Topology::read(&s.0, &all(20)).phys, Some(10));
    assert_eq!(Topology::read(&s.0, &all(20)).cpus, 19);
}

/// A 6P + 4E hybrid (an i5-12600K's shape): CPUs 0-11 the performance cores' siblings,
/// 12-15 the efficiency cores, one 20 MiB L3: 0.2.6 matches its exception; rule A's guard
/// does not
#[test]
fn fixture_hybrid_6p_4e() {
    let s = Sys::new("hybrid");
    for c in 0..16 {
        let (core, f) = if c < 12 {
            (4 * (c / 2), 4_900_000)
        } else {
            (32 + c - 12, 3_600_000)
        };
        s.cpu(c, 0, core, "20480K", "0-15", (Some(f), None));
    }
    s.put("devices/cpu_core/cpus", "0-11");
    s.put("devices/cpu_atom/cpus", "12-15");
    let t = Topology::read(&s.0, &all(16));
    assert_eq!(
        (t.cpus, t.phys, t.l3, t.l3_shared),
        (16, Some(10), Some(20 * MIB), Some(true))
    );
    assert_eq!((t.intel_hybrid, t.hybrid()), (Some(true), Some(true)));
    assert!((t.freq_spread.unwrap() - (4.9 / 3.6 - 1.0)).abs() < 1e-9);
    assert!(!t.fast_eligible());
    assert_eq!(plan_today(&t, 16).0, PIPELINED_PLAN);
    let d = decide_general(&t, 16);
    assert_eq!((d.plan, d.branch), (PIPELINED_PLAN, Branch::Envelope));
    assert_eq!(rule_a(&t, 16).plan, Plan::Teams { teams: 4, k: 2 });
    assert_eq!(rule_a(&t, 9).plan, Plan::Teams { teams: 4, k: 2 });
    // its performance cores alone are of one kind
    let p = Topology::read(&s.0, &all(12));
    assert_eq!(
        (p.phys, p.intel_hybrid, p.hybrid()),
        (Some(6), Some(false), Some(false))
    );
    assert!(p.fast_eligible());
}

/// Two sockets of 5 cores (SMT), a 20 MiB L3 each: 0.2.6 counts 10 cores and reads one
/// socket's L3, so matches its exception; the affinity spans two L3s
#[test]
fn fixture_two_sockets() {
    let s = Sys::new("2s");
    for c in 0..20 {
        let pkg = (c % 10) / 5;
        let shared = if pkg == 0 { "0-4,10-14" } else { "5-9,15-19" };
        s.cpu(c, pkg, c % 5, "20480K", shared, (Some(3_000_000), None));
    }
    let t = Topology::read(&s.0, &all(20));
    assert_eq!(
        (t.cpus, t.phys, t.l3, t.l3_cpu0),
        (20, Some(10), Some(20 * MIB), Some(20 * MIB))
    );
    assert_eq!((t.l3_shared, t.hybrid()), (Some(false), Some(false)));
    assert!(!t.fast_eligible());
    assert_eq!(decide_general(&t, 20).branch, Branch::Envelope);
    assert_eq!(rule_a(&t, 20).plan, Plan::Teams { teams: 4, k: 2 });
    // one socket's CPUs: one L3, 5 cores, workers
    let one = Topology::read(&s.0, &[0, 1, 2, 3, 4, 10, 11, 12, 13, 14]);
    assert_eq!((one.phys, one.l3_shared), (Some(5), Some(true)));
    assert_eq!(plan_general(&one, 10).0, Plan::Workers(10));
}

/// CI's runners: 4 CPUs on 2 cores (siblings 0-1 and 2-3), a 32 MiB L3, no cpufreq
#[test]
fn fixture_ci_runner() {
    let s = Sys::new("ci");
    for c in 0..4 {
        s.cpu(c, 0, c / 2, "32768K", "0-3", (None, None));
    }
    let t = Topology::read(&s.0, &all(4));
    assert_eq!(
        (t.cpus, t.phys, t.l3, t.l3_shared),
        (4, Some(2), Some(32 * MIB), Some(true))
    );
    assert_eq!(t.hybrid(), None);
    for n in [1, 2, 4] {
        assert_eq!(plan_general(&t, n), (Plan::Workers(n), Mode::Plain));
        assert_eq!(plan_general(&t, n), plan_today(&t, n));
    }
}

/// Nothing readable (an empty root, or a malformed file): nothing known, workers
#[test]
fn fixture_unknown() {
    let s = Sys::new("none");
    let t = Topology::read(&s.0, &all(8));
    assert_eq!(t, Topology::default());
    assert_eq!(t.hybrid(), None);
    let d = decide_general(&t, 8);
    assert_eq!((d.plan, d.branch), (Plan::Workers(8), Branch::Unread));
    assert_eq!(plan_today(&t, 8).0, Plan::Workers(8));
    // core ids but no package ids: the cores unknown, as `physical_cores` has them
    s.put("devices/system/cpu/cpu0/topology/core_id", "0");
    s.put("devices/system/cpu/cpu0/cache/index3/size", "garbage");
    let t = Topology::read(&s.0, &all(8));
    assert_eq!((t.cpus, t.phys, t.l3, t.l3_cpu0), (1, None, None, None));
    assert_eq!(cpu_list("0-3,x"), None);
    assert_eq!(cpu_list("0-2,8"), Some(vec![0, 1, 2, 8]));
}

/// ARM's signal: capacities that differ
#[test]
fn arm_capacity() {
    let s = Sys::new("arm");
    for c in 0..8 {
        s.cpu(
            c,
            0,
            c,
            "8192K",
            "0-7",
            (None, Some(if c < 4 { 1024 } else { 446 })),
        );
    }
    let t = Topology::read(&s.0, &all(8));
    assert_eq!((t.capacity_differs, t.hybrid()), (Some(true), Some(true)));
    let t = Topology::read(&s.0, &[0, 1, 2, 3]);
    assert_eq!((t.capacity_differs, t.hybrid()), (Some(false), Some(false)));
}

/// On this machine, the reader counts the cores and reads cpu0's L3 as 0.2.6 does
#[test]
fn detect_reads_as_today() {
    let t = Topology::detect();
    assert_eq!(t.phys, super::super::physical_cores());
    assert_eq!(t.l3_cpu0, super::super::l3_bytes());
    for n in [1, 2, 4, 8, 64] {
        assert_eq!(plan_today(&t, n).0, Plan::for_threads(n));
    }
}
