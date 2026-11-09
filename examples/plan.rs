//! The plans the decoder's policies choose (docs/journal/2026-11-07-general-plan-policy.md):
//! today's (0.2.6's, the default) and the general policy's rule A (opt-in, with its evidence
//! envelope), at every thread count from 1 to twice the physical cores.
//!
//! ```text
//! plan                                   this machine, as this process sees it
//! plan --cores C --l3 MIB [--hybrid] [--split]
//!                                        a given topology: C physical cores, an L3 of MIB
//!                                        MiB (read alike from cpu0 and the affinity), its
//!                                        cores of more than one kind, or the L3 not holding
//!                                        them all
//! plan --sysfs DIR --cpus LIST           the topology under a sysfs root (`/sys`, or a
//!                                        directory of the same files) for the CPUs of LIST
//! ```
//!
//! Arithmetic only: no decoding. A plan is `N workers`, `T × K` (plain teams of K) or
//! `T × K p` (pipelined, with park and small); a `*` marks where rule A differs from today.

use mibz2::parallel::general::{cpu_list, decide_general, plan_today, Branch, Mode, Topology};
use mibz2::parallel::Plan;

fn show(plan: Plan, mode: Mode) -> String {
    match plan {
        Plan::Workers(1) => "1 worker".into(),
        Plan::Workers(n) => format!("{n} workers"),
        Plan::Teams { teams, k } => format!(
            "{teams} × {k}{}",
            if matches!(mode, Mode::Pipelined(_)) {
                " p"
            } else {
                ""
            }
        ),
    }
}

fn mib(b: Option<u64>) -> String {
    b.map_or("unread".into(), |b| {
        format!("{} MiB", b as f64 / (1 << 20) as f64)
    })
}

fn usage() -> ! {
    eprintln!("usage: plan [--cores C --l3 MIB [--hybrid] [--split]] | [--sysfs DIR --cpus LIST]");
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args.get(i + 1).cloned().unwrap_or_else(|| usage()))
    };
    for a in &args {
        if a.starts_with("--")
            && ![
                "--cores", "--l3", "--hybrid", "--split", "--sysfs", "--cpus",
            ]
            .contains(&a.as_str())
        {
            usage();
        }
    }
    let (topo, source) = match (value("--cores"), value("--l3"), value("--sysfs")) {
        (Some(c), Some(l3), None) => {
            let cores: usize = c.parse().unwrap_or_else(|_| usage());
            let l3: f64 = l3.parse().unwrap_or_else(|_| usage());
            let hybrid = args.iter().any(|a| a == "--hybrid");
            let split = args.iter().any(|a| a == "--split");
            let bytes = (l3 * (1u64 << 20) as f64).round() as u64;
            (
                Topology::synthetic(Some(cores), Some(bytes), !split, hybrid),
                "given",
            )
        }
        (None, None, Some(dir)) => {
            let cpus = value("--cpus")
                .and_then(|s| cpu_list(&s))
                .unwrap_or_else(|| usage());
            (
                Topology::read(std::path::Path::new(&dir), &cpus),
                "read from the sysfs root given",
            )
        }
        (None, None, None) => (Topology::detect(), "this machine, as this process sees it"),
        _ => usage(),
    };
    println!("topology ({source}):");
    println!("  CPUs in the affinity: {}", topo.cpus);
    println!(
        "  physical cores:       {}",
        topo.phys.map_or("unread".into(), |p| p.to_string())
    );
    println!("  L3 (a CPU of the affinity's): {}", mib(topo.l3));
    println!("  L3 (cpu0's, as 0.2.6 reads it): {}", mib(topo.l3_cpu0));
    println!("  L3 holds the whole affinity: {:?}", topo.l3_shared);
    println!(
        "  core kinds: Intel cpu_core/cpu_atom both {:?}, cpu_capacity differs {:?}, max-frequency spread {}",
        topo.intel_hybrid,
        topo.capacity_differs,
        topo.freq_spread
            .map_or("unread".into(), |s| format!("{:.1}%", 100.0 * s))
    );
    println!(
        "  more than one kind: {:?}; rule A's fast plan allowed: {}",
        topo.hybrid(),
        topo.fast_eligible()
    );
    let top = 2 * topo
        .phys
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()));
    println!();
    println!("| threads | today (0.2.6, the default) | general (rule A, opt-in) | its step |");
    println!("|--:|---|---|---|");
    for n in 1..=top {
        let (tp, tm) = plan_today(&topo, n);
        let d = decide_general(&topo, n);
        let mark = if (tp, tm) == (d.plan, d.mode) {
            ""
        } else {
            " *"
        };
        let step = match d.branch {
            Branch::Unread => "topology unread",
            Branch::BelowSix => "below 6 cores",
            Branch::Envelope => "the envelope",
            Branch::OldPlan => "old plan",
            Branch::FastPlan => "fast plan",
        };
        println!(
            "| {n} | {} | {}{mark} | {step} |",
            show(tp, tm),
            show(d.plan, d.mode)
        );
    }
}
