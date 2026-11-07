//! Research, opt-in, not the default: the general plan policy's rule A, the *two-budget
//! rule*, with its evidence envelope, and the topology it reads
//! (docs/journal/2026-11-07-general-plan-policy.md, phase 2's cheaper first step).
//!
//! Nothing here is on the default path: `Plan::for_threads` and `StreamDecoder::new` keep
//! 0.2.6's policy (`plan_with`, pipelined at `PIPELINED_PLAN` alone). This module only
//! computes what rule A would choose, so it can be printed (examples/plan.rs), run by
//! choice (examples/ab.rs's `mibz2-general` arm, `decoder`) and tested.
//!
//! **Rule A,** for N threads asked for, P physical cores in the affinity and an L3 of L
//! bytes (`rule_a`):
//! 1. P unknown, or c = min(N, P) below `COOP_MIN_CORES`: N workers.
//! 2. The old plan: T_b = max(1, min(⌊c / 2⌋, round(0.7 L / F))) teams of 2, plain, with 4
//!    for the second term when L is unread (F, a full table, 3.6 MB).
//! 3. The fast plan, when L is read, that L3 holds the whole affinity, and the cores are of
//!    one kind: with T_cap = ⌊L / 6 MB⌋, T_cap teams of 3, pipelined with park and small,
//!    if T_cap ≥ 2, T_b > T_cap, 3 T_cap ≤ N and 3 T_cap ≤ P − 1.
//! 4. Otherwise the old plan.
//!
//! Pipelined if and only if the width is 3 or more. **The envelope** (`ENVELOPE`) comes
//! first in `plan_general`: the topologies and thread ranges a registered run has measured,
//! read as 0.2.6 reads them (the L3 of cpu0) and before rule A's guards, where 0.2.6's
//! choice is kept.

use super::{
    plan_with, Levers, Plan, COOP_MIN_CORES, FULL_TABLE, PIPELINED_LEVERS, PIPELINED_PLAN,
};
use super::{window, Kernels, StreamDecoder};
use std::collections::BTreeSet;
use std::path::Path;

/// L3 bytes a fast team needs (rule A's one new constant, inside the (5.24, 6.99] MB the
/// i9 allows: the journal's §2, "The constant")
pub const FAST_TEAM_L3: f64 = 6e6;
/// A fast team's width
pub const FAST_WIDTH: usize = 3;
/// The max-frequency spread across the affinity above which its cores count as of more than
/// one kind (wide, since Turbo Boost Max can favour some cores of one kind: the journal's
/// §2, "At the edges")
pub const FREQ_SPREAD: f64 = 0.2;

/// What the policy reads of the machine: the CPUs this process may use, their physical
/// cores, the L3 and whether it holds them all, and whether they are of one kind. A field
/// the machine doesn't say is None.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Topology {
    /// The online CPUs in the affinity
    pub cpus: usize,
    /// Physical cores: the distinct (package, core id) of the affinity's CPUs, as
    /// `physical_cores` counts them
    pub phys: Option<usize>,
    /// The L3's bytes, read from the first online CPU in the affinity
    pub l3: Option<u64>,
    /// The L3's bytes as 0.2.6 reads them (cpu0's `index3`), for the envelope
    pub l3_cpu0: Option<u64>,
    /// Whether every online CPU in the affinity is in that L3's `shared_cpu_list`
    pub l3_shared: Option<bool>,
    /// Intel's hybrid signal: whether the affinity has CPUs of both `cpu_core` and
    /// `cpu_atom` (None where the machine has no such PMUs)
    pub intel_hybrid: Option<bool>,
    /// ARM's: whether `cpu_capacity` differs across the affinity
    pub capacity_differs: Option<bool>,
    /// The spread of `cpufreq/cpuinfo_max_freq` across the affinity, max / min − 1
    pub freq_spread: Option<f64>,
}

impl Topology {
    /// A topology given rather than read (tests, examples/plan.rs): `phys` cores whose
    /// `l3` (read alike from cpu0 and from the affinity) is or isn't shared by all, of one
    /// kind or not
    pub fn synthetic(phys: Option<usize>, l3: Option<u64>, shared: bool, hybrid: bool) -> Self {
        Topology {
            cpus: phys.unwrap_or(0),
            phys,
            l3,
            l3_cpu0: l3,
            l3_shared: Some(shared),
            intel_hybrid: Some(hybrid),
            capacity_differs: None,
            freq_spread: None,
        }
    }

    /// This machine's, for this process (Linux; elsewhere, nothing read)
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            let Some(affinity) = std::fs::read_to_string("/proc/self/status")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
                        .and_then(cpu_list)
                })
            else {
                return Topology::default();
            };
            Self::read(Path::new("/sys"), &affinity)
        }
        #[cfg(not(target_os = "linux"))]
        Topology::default()
    }

    /// The topology under the sysfs root `sys` (`/sys`, or a directory of the same files),
    /// for the CPUs of `affinity`
    pub fn read(sys: &Path, affinity: &[usize]) -> Self {
        let cpu = sys.join("devices/system/cpu");
        let mut t = Topology {
            l3_cpu0: read_size(&cpu.join("cpu0/cache/index3/size")),
            ..Topology::default()
        };
        // the online CPUs (with a core id), and their cores, as `physical_cores` reads them
        let mut online = Vec::new();
        let mut cores = BTreeSet::new();
        let mut phys_ok = true;
        for &c in affinity {
            let topo = cpu.join(format!("cpu{c}/topology"));
            let Some(core) = read_trim(&topo.join("core_id")) else {
                continue;
            };
            online.push(c);
            match read_trim(&topo.join("physical_package_id")) {
                Some(pkg) => {
                    cores.insert((pkg, core));
                }
                None => phys_ok = false,
            }
        }
        t.cpus = online.len();
        t.phys = (phys_ok && !cores.is_empty()).then_some(cores.len());
        let Some(&first) = online.first() else {
            return t;
        };
        // the L3 of a CPU in the affinity, and the CPUs it holds
        if let Some(dir) = l3_dir(&cpu.join(format!("cpu{first}/cache"))) {
            t.l3 = read_size(&dir.join("size"));
            t.l3_shared = read_trim(&dir.join("shared_cpu_list"))
                .and_then(|s| cpu_list(&s))
                .map(|shared| {
                    let shared: BTreeSet<usize> = shared.into_iter().collect();
                    online.iter().all(|c| shared.contains(c))
                });
        }
        // the core-kind signals
        let pmu = |name: &str| {
            read_trim(&sys.join(format!("devices/{name}/cpus"))).and_then(|s| cpu_list(&s))
        };
        if let (Some(big), Some(small)) = (pmu("cpu_core"), pmu("cpu_atom")) {
            let has = |set: &[usize]| online.iter().any(|c| set.contains(c));
            t.intel_hybrid = Some(has(&big) && has(&small));
        }
        let per_cpu = |file: &str| -> Vec<u64> {
            online
                .iter()
                .filter_map(|c| read_trim(&cpu.join(format!("cpu{c}/{file}")))?.parse().ok())
                .collect()
        };
        let caps = per_cpu("cpu_capacity");
        if !caps.is_empty() {
            t.capacity_differs = Some(caps.iter().min() != caps.iter().max());
        }
        let freqs = per_cpu("cpufreq/cpuinfo_max_freq");
        if let (Some(&lo), Some(&hi)) = (freqs.iter().min(), freqs.iter().max()) {
            if lo > 0 {
                t.freq_spread = Some(hi as f64 / lo as f64 - 1.0);
            }
        }
        t
    }

    /// Whether the cores are of more than one kind: Some(true) on any signal, Some(false)
    /// where a signal was read and none says so, None where none could be read (and the
    /// cores then count as of one kind: no positive proof is required)
    pub fn hybrid(&self) -> Option<bool> {
        let signals = [
            self.intel_hybrid,
            self.capacity_differs,
            self.freq_spread.map(|s| s > FREQ_SPREAD),
        ];
        if signals.contains(&Some(true)) {
            Some(true)
        } else if signals.iter().any(Option::is_some) {
            Some(false)
        } else {
            None
        }
    }

    /// Rule A's guards for its fast plan: the L3 read, holding the whole affinity, and the
    /// cores of one kind
    pub fn fast_eligible(&self) -> bool {
        self.l3.is_some() && self.l3_shared == Some(true) && self.hybrid() != Some(true)
    }
}

/// How the decoder runs a plan's teams
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Plain,
    Pipelined(Levers),
}

/// The step that chose a plan
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    /// the topology unread: workers
    Unread,
    /// fewer than `COOP_MIN_CORES` cores serve the threads: workers
    BelowSix,
    /// inside the evidence envelope: its measured plan (0.2.6's choice)
    Envelope,
    /// rule A's old plan: teams of 2 near 0.7 of the L3
    OldPlan,
    /// rule A's fast plan: T_cap teams of 3, pipelined
    FastPlan,
}

/// A policy's answer: the plan, how its teams run, and the step that chose it
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decision {
    pub plan: Plan,
    pub mode: Mode,
    pub branch: Branch,
}

/// One entry of the evidence envelope: a topology, as 0.2.6 reads it, and the thread range
/// on it whose plan a registered run has measured
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// physical cores in the affinity
    pub phys: usize,
    /// cpu0's L3 bytes
    pub l3_cpu0: u64,
    /// the fewest threads asked for the entry covers (and every number above)
    pub min_threads: usize,
    pub plan: Plan,
    pub mode: Mode,
}

/// The evidence envelope: today, the i9-10900K's 10 cores and 20 MiB at 10 threads or more,
/// three teams of three pipelined with park and small
/// (docs/journal/2026-11-06-three-team-policy.md, 2026-11-07-pipeline-levers-result.md).
/// Data, one entry a registration; it widens only by a registered run.
pub const ENVELOPE: &[Entry] = &[Entry {
    phys: 10,
    l3_cpu0: 20 << 20,
    min_threads: 10,
    plan: PIPELINED_PLAN,
    mode: Mode::Pipelined(PIPELINED_LEVERS),
}];

/// The envelope's entry for `threads` on `topo`, if any (read as 0.2.6 reads the topology:
/// its cores and cpu0's L3, before any guard)
pub fn in_envelope(topo: &Topology, threads: usize) -> Option<&'static Entry> {
    ENVELOPE.iter().find(|e| {
        topo.phys == Some(e.phys) && topo.l3_cpu0 == Some(e.l3_cpu0) && threads >= e.min_threads
    })
}

/// The mode a plan's width gives: pipelined, with park and small, at 3 or more
fn mode_of(plan: Plan) -> Mode {
    match plan {
        Plan::Teams { k, .. } if k >= FAST_WIDTH => Mode::Pipelined(PIPELINED_LEVERS),
        _ => Mode::Plain,
    }
}

/// The old plan's teams: as many teams of 2 as keep the tables near 0.7 of the L3 (4 where
/// it is unread), within ⌊c / 2⌋, at least one
fn old_teams(cores: usize, l3: Option<u64>) -> usize {
    let cap = l3.map_or(4, |b| {
        ((0.7 * b as f64) / FULL_TABLE).round().max(1.0) as usize
    });
    (cores / 2).min(cap).max(1)
}

/// Rule A alone, without the envelope: a pure function of the topology and the threads
/// asked for (at least 1)
pub fn rule_a(topo: &Topology, threads: usize) -> Decision {
    let n = threads.max(1);
    let workers = |branch| Decision {
        plan: Plan::Workers(n),
        mode: Mode::Plain,
        branch,
    };
    let Some(phys) = topo.phys else {
        return workers(Branch::Unread);
    };
    let c = n.min(phys);
    if c < COOP_MIN_CORES {
        return workers(Branch::BelowSix);
    }
    let t_b = old_teams(c, topo.l3);
    if let (Some(l3), true) = (topo.l3, topo.fast_eligible()) {
        let t_cap = (l3 as f64 / FAST_TEAM_L3).floor() as usize;
        let need = FAST_WIDTH.saturating_mul(t_cap);
        if t_cap >= 2 && t_b > t_cap && need <= n && need < phys {
            let plan = Plan::Teams {
                teams: t_cap,
                k: FAST_WIDTH,
            };
            return Decision {
                plan,
                mode: mode_of(plan),
                branch: Branch::FastPlan,
            };
        }
    }
    let plan = Plan::Teams { teams: t_b, k: 2 };
    Decision {
        plan,
        mode: mode_of(plan),
        branch: Branch::OldPlan,
    }
}

/// The general policy, opt-in: the envelope's measured plan inside it, rule A outside
pub fn decide_general(topo: &Topology, threads: usize) -> Decision {
    let n = threads.max(1);
    if let Some(e) = in_envelope(topo, n) {
        return Decision {
            plan: e.plan,
            mode: e.mode,
            branch: Branch::Envelope,
        };
    }
    rule_a(topo, n)
}

/// `decide_general`'s plan and mode
pub fn plan_general(topo: &Topology, threads: usize) -> (Plan, Mode) {
    let d = decide_general(topo, threads);
    (d.plan, d.mode)
}

/// The default as phase 2's step 1 would write it (the journal's admission step 1), not
/// used by the default: the envelope inside it, and outside it 0.2.6's old rule (workers
/// below 6 cores or unread, else teams of 2 near 0.7 of cpu0's L3), plain. Equal on the
/// tested grid to `plan_today` (the test `the_default_is_unchanged`).
pub fn plan_enveloped(topo: &Topology, threads: usize) -> (Plan, Mode) {
    let n = threads.max(1);
    if let Some(e) = in_envelope(topo, n) {
        return (e.plan, e.mode);
    }
    match topo.phys {
        Some(phys) if n.min(phys) >= COOP_MIN_CORES => (
            Plan::Teams {
                teams: old_teams(n.min(phys), topo.l3_cpu0),
                k: 2,
            },
            Mode::Plain,
        ),
        _ => (Plan::Workers(n), Mode::Plain),
    }
}

/// 0.2.6's default for `topo`, as `Plan::for_threads` and `StreamDecoder::new` decide it:
/// `plan_with` on the cores and cpu0's L3, pipelined with park and small at
/// `PIPELINED_PLAN` alone (`StreamDecoder::defaulted`'s rule)
pub fn plan_today(topo: &Topology, threads: usize) -> (Plan, Mode) {
    let plan = plan_with(threads.max(1), topo.phys, topo.l3_cpu0);
    (plan, today_mode(plan))
}

/// The mode 0.2.6's default runs `plan` in: pipelined, with park and small, at
/// `PIPELINED_PLAN` alone
pub fn today_mode(plan: Plan) -> Mode {
    if plan == PIPELINED_PLAN {
        Mode::Pipelined(PIPELINED_LEVERS)
    } else {
        Mode::Plain
    }
}

/// A decoder running `plan` as `mode` says, with the window `StreamDecoder::new` gives a
/// plan for `slots` asked on `threads` threads (`with_plan`'s twin for a mode as well)
pub fn decoder(
    plan: Plan,
    mode: Mode,
    threads: usize,
    slots: usize,
    kernels: Kernels,
) -> StreamDecoder {
    let slots = window(plan, threads.max(1), slots);
    match (plan, mode) {
        (Plan::Teams { teams, k }, Mode::Pipelined(levers)) => {
            StreamDecoder::pipelined_levers(teams, k, slots, kernels, levers)
        }
        _ => StreamDecoder::planned(plan, slots, kernels),
    }
}

/// A CPU list (`0-3,8,10-11`), or None if malformed
pub fn cpu_list(s: &str) -> Option<Vec<usize>> {
    let s = s.trim();
    if s.is_empty() {
        return Some(Vec::new());
    }
    let mut v = Vec::new();
    for part in s.split(',') {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        let (a, b): (usize, usize) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
        v.extend(a..=b);
    }
    Some(v)
}

fn read_trim(p: &Path) -> Option<String> {
    std::fs::read_to_string(p)
        .ok()
        .map(|s| s.trim().to_string())
}

/// A sysfs cache size (`20480K`), as `l3_bytes` reads it
fn read_size(p: &Path) -> Option<u64> {
    let s = read_trim(p)?;
    let (num, mult) = match s.as_bytes().last()? {
        b'K' => (&s[..s.len() - 1], 1u64 << 10),
        b'M' => (&s[..s.len() - 1], 1 << 20),
        _ => (&s[..], 1),
    };
    num.parse::<u64>().ok().map(|n| n * mult)
}

/// A CPU's level-3 cache directory: the `indexN` whose `level` is 3, or `index3` where no
/// level is given
fn l3_dir(cache: &Path) -> Option<std::path::PathBuf> {
    let mut any_level = false;
    for i in 0..16 {
        let d = cache.join(format!("index{i}"));
        if let Some(level) = read_trim(&d.join("level")) {
            any_level = true;
            if level == "3" {
                return Some(d);
            }
        }
    }
    let d = cache.join("index3");
    (!any_level && d.join("size").exists()).then_some(d)
}

#[cfg(test)]
mod tests;
