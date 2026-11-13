# D104 — bounded i9 machine/activity admissibility

Checkpoint [6867883717](https://github.com/lmmx/mibz2/pull/161#issuecomment-6867883717). Reviewed 2026-11-13.
Authority: docs/team/SPEC.md, SPEC.GUARDIAN.md, MODE.md, RULES.md and PROTOCOL.md at a2da49638ac54d7763de1c167bd054f67ffdc763.
Recipe and PLAN: cf1a36ed429f361839fab6748383187104d8ab27, bench/cells-selective-i9/.
Measured ab source: a8a23c11607864cb431f87fff38169e10c41685f, examples/ab.rs.
Bundle: e04fe229e9d939840d50fea110c7b7c6f74811ef/cells-selective-i9/20261113T233358053141311Z.tar.gz.

**G1: PROCEED on this bounded machine/activity question. The reported 1.65 does not establish a violation of the frozen idle gate or require whole-run invalidation, row exclusion, or a separate per-comparison invalid verdict. Resolve the checkpoint's qualification hold on this question only.** Retain all 252 observations, including the adverse runs observation, with its measurement uncertainty. This is eligibility for the registered diagnostic interpretation, not production qualification, a safety review, or a certification of every bundle predicate. #380's failed default gate / integration DEFER is not lifted. No new G2/G3/G8 constraint or workstation run is authorised.

## Independently checked evidence

Downloaded the original archive without running any archived binary, recipe or measurement. Length 60,935,434 bytes; SHA-256 **330ff61ce5e2659c8f0a86837d26eab274d0eb1fbe4e5563d3f3824f92912fa7**, exactly the submitted identity. Read preflight, static topology/quota/frequency policy, passive facts, all 252 raw rows and measurement command. Replayed the pinned share_nothing.py preflight_problems against archived inputs: no problems. All three collectors exited 0. This replay checks recorded facts, not the current workstation.

Facts establish i9-10900K, 20 allowed/online CPUs, unlimited quota, intel_pstate/powersave/balance_performance/turbo-on. Selected CPU0 has sibling10 on physical core0; parent affinity excludes both, with remaining 18 CPUs. All rows record cpus=0; all 504 pre/post machine states retain powersave/balance_performance/turbo-on. The three preflight samples are sysfs state samples, not per-CPU activity samples: their frequencies must not be mistaken for busy fractions.

Observed raw-row maxima: pre_busy **0.06**, others_busy **1.65**. Recorded steal_pct is 0.0 throughout. Throttled_n and throttled_ms are **null**, not zero; passive facts say throttling=false and establish unlimited quota. Thus there is no reported cgroup throttling, not a demonstrated absence of every form of thermal or frequency interference. Do not convert unavailable counters into PASS measurements.

## Exact gate semantics, including individual CPU

PLAN says external activity is subject to the “standing idle gate (total busy <=1.5 cores, no individual CPU >0.5)” and fixes settling2s and the idle gate **before every child**. It separately requires recording per-run activity and says unexpected machine conditions invalidate qualification.

In the measured source, examples/ab.rs:1470–1498 calculates per-CPU nonnegative busy fractions over settling, then their sum and maximum. At lines2341–2346 it waits while **busy > max_busy OR one >0.5**, remeasuring before spawning. Recipe workstation.py passes --settle2 --max-busy1.5 and disables the held spinner. Nothing in that gate compares post-start others_busy to 1.5.

Ordinary output omits the individual pre-start maximum, but this does not leave the 0.5 requirement unknowable: with no spinner, every per-CPU fraction is nonnegative and max <= sum. The largest archived sum, rounded to two decimals, is0.06 (under0.065 before formatting), already far below0.5. This independently bounds the individual maximum for each observed pre-start window; it is not merely trusting the empty preflight problems list. It does not guarantee exclusivity during decoding or prove uninterrupted idleness between observations.

## What 1.65 measures

Exactly one row exceeds1.5: corpus runs, zero-based round9, original Cells, child wall0.0083s, child CPU0.011s, pre_busy0.01. Its spawn/exit timestamps span **0.011884504s**, not the child decode wall.

At ab.rs:2419–2420:
others_busy = max(aggregate /proc/stat busy-jiffy delta /100 - reported child CPU, 0) / parent-observed process span.
It includes work over spawn/wait/census lifetime, including parent activity, and uses machine-wide jiffy accounting with HZ100 and a separately reported child CPU. It is not a direct measurement of external contention on CPU0 during the0.0083s decode, nor a post-start per-CPU maximum. One10ms aggregate tick over roughly12ms corresponds to about0.84 core in this estimator; boundary/accounting granularity is substantial. The published1.65 therefore cannot establish the duration, location, or impact of an external burst. Equally, these facts do not prove there was no burst or no interference.

The frozen registration explicitly permits external desktop activity subject to a pre-start gate; it does not promise an OS-reserved core or continuous activity ceiling. The broad unexpected-conditions clause still binds for actual host failures/changes, but is not an unstated rule “others_busy >1.5 invalidates.” Adding that numerical post-run rule now would create a different criterion after seeing the outcomes (RULES C2). Nor does a successful script exit alone establish all predicates: the source and archived host facts above are the basis for this bounded finding.

## Consequences and limits

Keep this row in every registered paired estimate; do not trim it, replace it, recompute a favourable subset, or repeat the run. Report its value and coarse temporal/accounting resolution. It is an uncertainty affecting interpretation, especially of this short runs comparison, not an independently established formal invalidation. No effect size attributable to contention is established here.

This review does not rederive the63 intervals, repeat source/binary/input parity checks, or reopen B5. Those remain separately attributed Coordinator checks. The reported Wiki improvements do not erase the reported pattern slowdown, Wiki memory cost, text's unresolved C/B improvement or #380 integration/default failure. No all-machine claim, adoption decision, retuning or retry follows. Routine work is not gated on another Guardian approval.

Migration verified against frozen prior-branch head f9dab3d2b40739d89d613f86e8c2d67bec3159e4 and manifest SHA-25660df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. Inbox has only this unfinished checkpoint. Bounded routing audit since the previous handled event found no new misrouting; route REVIEW ->145. Preserve all earlier decisions.
