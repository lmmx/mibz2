#### FRONT nogo
MACHINE: Every CI runner is 4 vCPU = 2 physical cores x 2 SMT siblings (0,1 | 2,3), per env.json of 41980640481. CPU model per run:
- master bench 41980640481 (d1fd4405): Xeon Platinum 8573C
- occupancy 41976148952 (c90b3625): EPYC 7763
- walk sweeps 41989592674 / 41989596336 (0769b52f): EPYC 9V45 / EPYC 9V74
- phases runs 42008210159 (b7734313) and 42011789918 (4bfaf7cb): Xeon 8370C and EPYC 7763. Another front dispatched these on master. `git diff d1fd4405 4bfaf7cb -- src examples Cargo.*` is empty, so they measure production code.
- #155: 41908271917, 7763
- #157: 41925226768, 7763
- #183: PR runs 41986885355 (7763, then 9V45); fusion-alone 41986908473 (8370C); refactor-alone 41986912287 (7763)
- #181 merged-head run: 41979947814, 9V45
- P74 Gate 1: 41077291982 (9V74) and 41081427181 (9V45)
- P74-C stage trial 41190706676: 7763, 9V74, 9V74
- P74-C decoder trial 41198956443: 7763, 8573C, 9V74. Its w=1 and w=2 jobs ran one CPU per core; no w=4 was run on CI.

L1D is 32 KiB on the 7763 and 9V74, 48 KiB on the 9V45 and 8573C (tile-size.md:150-157). No CI runner exposes a PMU: the sweep files say 'counters: unavailable here'.

The i9-10900K is the secondary case. Teams engage there only at 6 or more physical cores, so every team or level idea applies to the i9 only. Its P74-C and walk-unroll data are from 31 Oct to 1 Nov, before #170, #171, #172, #180 and #181. Nothing here infers CI behaviour from the i9 or the reverse.

Local machine: no builds, no profiles and no timings were run. Th

- [OBSERVED] 4t wall -> total CPU on 4 SMT vCPUs -> per-block CPU inflated 1.61x by co-run
  CLAIM: At 4 threads on CI the decode is CPU-saturated. The wall is bound by total CPU-seconds on SMT-shared cores, not by idle time, so any idle or scheduling idea has at most about 4% to recover.
  EVID: 41980640481 (Xeon 8573C, d1fd4405), runs.jsonl, wikidata, mibz2 arm, 4t, 10 rounds:
- median wall 0.343 s, CPU 1.314 s, CPU/(4*wall) 0.961; at 2t, 0.980
- per-thread split: workers 1.06 s, scanner 0.13-0.14 s, reader 0.03 s, unattributed 0.08 s
- amplification 1.62x (1.59-1.65)
Occupancy 41976148952 (EPYC 7763, c90b3625) at 4t, window 16:
- workers decode 93.0% of their time, 3.72 of 4 on average, idle 5%
- CPU per block 2.26 ms against 1.40 ms at 1t, and 1.55 ms at 2t spread
- run-queue wait 0.346 ms a block
  SHARE: busy 96.1%; idle 3.9%
  RECOV: Idle and scheduling ideas: at most 3.9% (about 13 ms of 343 ms) all together. CPU-cutting ideas: about 1:1, wall saving ≈ 4t CPU saving / 0.961.
  FALSIFIER: A 4t run with CPU/(4*wall) below 0.90, or a pure scheduling change gaining more than 4% at 4t on wikidata.

- [OBSERVED] stage c/B alone -> x(SMT inflation) -> share of 4t CPU
  CLAIM: SMT inflation per stage, from four pinned copies against one alone. The four-copy shares are the ceilings for any stage idea at 4t, and the ranking depends on the CPU model.
  EVID: sweep.yml tool=phases, 5 rounds, src = d1fd4405. Each stage: c/B alone -> mean of the four copies (inflation), share of four-copy time.

42008210159, Xeon 8370C (total 13.45 -> 21.82, x1.62):
- walk 7.27 -> 10.64 (x1.46), 48.8%
- build 2.01 -> 4.52 (x2.25), 20.7%
- MTF+RLE2 2.22 -> 3.42 (x1.54), 15.7%
- scanner x1.56, 10.7%
- RLE1 x2.08, 3.0%; assembly x1.77, 3.2%; CRC x1.59, 1.2%

42011789918, EPYC 7763 (total 10.24 -> 17.46, x1.71):
- walk 4.33 -> 9.20 (x2.12), 52.7%
- build x1.43, 17.7%
- MTF+RLE2 x1.33, 15.0%
- scanner x1.43, 10.1%
- RLE1 x1.53, 2.4%; assembly x2.97, 2.7%; CRC x1.41, 2.1%

  SHARE: as listed (four-copy proxy)
  RECOV: Ceilings at 4t: walk ~49-53% (incl. assembly), build 18-21%, MTF+RLE2 15-16%, scanner 10-11%, post-walk ~7%.
  FALSIFIER: Four threads in one process (occupancy with per-stage marks) giving shares more than 5 points away, or phases runs on 9V45/9V74/8573C reversing the build/walk ordering.

- [INFERRED] window 4t -> saturation -> CPU = wall
  CLAIM: Since #180, a 4t CPU cut reaches the 4t wall. Before #180 it did not, so CPU-only gains rejected before #180 were judged under a premise that no longer holds.
  EVID: After #180:
- #181 run 3 (41979947814, EPYC 9V45): 4t CPU -3.5%, wall -3.2%
- #183 fusion-alone (41986908473, Xeon 8370C): CPU -1.9% (-2.8,-1.0), wall -1.5% (-2.4,-0.6)
Before #180:
- #157 (41925226768, EPYC 7763): 4t CPU -1.6% (-3.0,-0.2) with wall +0.2%
- #114 (one-pass-build.md:98-101): 4t CPU +1.1% with wall +0.8% (ns)
Derivation: the 4t decode is saturated (0.961) only after #180 (window journal: workers 75% busy at window 8 before #171, 93% after #180).
  SHARE: n/a
  RECOV: Turns 1t/CPU-only wins (P74-C, #183, #169) into 4t wall at roughly their 4t-CPU share.
  FALSIFIER: A post-#180 PR whose 4t CPU falls beyond noise while its 4t wall does not, in two runs.

- [OBSERVED] MTF serial chain -> store-forwarding stall per symbol -> 15% of 4t CPU
  CLAIM: The register-held MTF head (P74-C) is the largest unshipped decoder-level gain on record. Its no-go was the registration gates, never the CI PR bench. The store-forwarding pattern it removes is still in production.
  EVID: Production: d1fd4405:src/mtf.rs:146-155. The list stays on the stack: each symbol does a 32-byte load at base and a loadu at base-1 over the previous symbol's 32-byte store and 1-byte store. #170 changed only the run stores (:125-134) and #172 only the list contents (:91-96).
P74-C stage trial 41190706676 (7763, 9V74, 9V74):
- wikidata checkpoints T2/T0 0.654-0.747, held-out 0.666-0.760
- controls 0.997-1.007
- failed only on bypassed adversarial edge files at about +1% (about 1 µs per block)
P74-C decoder trial 41198956443 (7763, 8573C, 9V74):
- +4.7 to +5.7% at w1, +2.3 to +4.0% at w2
- CPU-
  SHARE: MTF+RLE2 15.0-15.7% of four-copy CPU
  RECOV: -0.5 to -3.9% (mid -2%, about 7 ms) at 4t on CI
  FALSIFIER: A rebased P74-C PR bench showing 1t wikidata wall within ±0.5% of master.

- [INFERRED] walk -> per-core misses in flight shared between siblings
  CLAIM: The no-gos of the pair walk, Precopy and the move structure rest on a premise ('the walk is bound by instructions per step') that the walk round reversed. The reversal does not rescue them on CI: their losses come from added build work and per-step work that the SMT sibling shares.
  EVID: The premise:
- pair-walk.md:61
- move-structure-walk.md:122, :143
- mlp-scheduling-and-run-structures.md:40-41
The reversal: walk-round.md:67 ('bound by the number of misses a core can keep in flight'). Supporting it: #181's -4% walk instructions cost time; #171 staging gained at 4t with -2.2% Ir.
Why the reversal doesn't rescue them:
- pair-walk build 7.1-7.6 against 2.5 c/B (pair-walk.md:53-58); CI 4t +24.3%
- move step 41 against 16 instructions; i9 E1 pairwalk/v0.1.4 0.816 at 4 workers (expert-review.md:262)
No counters exist: no PMU on CI.
  SHARE: walk 49-53% of four-copy CPU
  RECOV: about 0
  FALSIFIER: A PMU host showing the walk's fill buffers idle under SMT co-run, which would make instruction count the binding term again.

- [OBSERVED] policy -> teams halve the blocks in flight on 2 cores
  CLAIM: Cooperative teams lose on CI-sized machines, now measured: forced teams decode at 0.71x the workers at 4 threads.
  EVID: 41976148952 (EPYC 7763, c90b3625), unpaired, both off-mode medians:
- occupancy-teams.txt: 491.2 MB/s
- occupancy.txt (workers): 694.9 MB/s
- under teams, 1.95 of 4 workers decode and the window is used to 8 of 16
Corroborating the mechanism: 2t-spread amplification 1.04x against 1.62x at 4t (41980640481), and packed against spread at 2t +40% (2026-10-27-four-thread-ceiling.md §4). So the 4t inflation is mostly within-core SMT sharing, not L3 capacity.
  SHARE: n/a
  RECOV: 0; teams would cost about 29%
  FALSIFIER: A paired teams-vs-workers A/B at 4t on a 2-core SMT runner within 10%.

- [OBSERVED] methodology
  CLAIM: Two earlier no-gos have already flipped after a premise change. Closures on i9 gates or on pre-#171/#180 baselines are not final.
  EVID: Window:
- demoted at #53 to a 7-8% bound at window 2t (2026-10-27-four-thread-ceiling.md:18)
- merged as #180 at -8.1, -10.3 and -8.9% 4t wall (41976148934, Xeon 8370C, EPYC 7763, Xeon 8370C) once #171 had sped the workers
Walk-unroll:
- registered i9 NEGATIVE (origin/docs/walk-unroll-decoder-records 270333c2)
- merged as #157 on the CI bench: 1t -2.4% (-3.6,-1.2), 4t CPU -1.6%
Caution: the bench arm-order bias before 3.0.0 favoured head by about 2%, so earlier rejections stand a fortiori (early-warning.md:118).
  SHARE: n/a
  RECOV: calibration only
  FALSIFIER: n/a

- NOGO P74 / P74-C: the MTF list head held in a ymm register, with a σ bypass. Lineage: #25 -> #118 -> research/p74-c-implementation -> research/p74c-decoder (archived).: of=The registrations and an integration defect, not the kernel and not the idea.
- #118 Gate 1: wikidata MTF stage -33% (9V74) and -19% (9V45). It failed the ≤+3% random/records rule: the high-rank fallback was 9-19% slower, and the selector's SSE2 count added +6-7%.
- P74-C stage trial (41190706676): passed primary, held-out and controls. It failed on about +1% over the limit on bypassed adversarial edge files.
- P74-C decoder trial: CI exploratory +4.7 to +5.7% at w1 and +2.3 to +4.0% at w2 (41198956443). The registered i9 run was NEGATIVE on: (a) the paired-RSS rule, tripped by ll reserved at max rather than max+32; (b) controls inconclusive from w4 noise; (c) the 1.02 bar at w4.
- It was never run on the CI PR bench.
  changed: - Baseline: #170 shipped P74's short-run stores and #172 the byte-valued list, which takes part of the margin. #157, #171 and #181 shrank the walk, which grows MTF+RLE2 from 17.1% of a 1t decode (P74 gate-1 result) to 19.4-19.8% (41980640481 8573C; 42011789918 7763).
- The store-forwarding head is still in production (d1fd4405:src/mtf.rs:146-155).
- #180 saturation means CPU-s/GB savings now reach the 4t wall.
- Methodology: the CI bench decides, and the #157 walk-unroll precedent shows i9-gate 
  now: Faster: 1t wall -2 to -4%; 4t wall -1 to -3% (lo -0.5%, hi -3.9%, about 2-13 ms of 0.343 s on 8573C).
Mechanism: shorter per-symbol serial dependency.
Derivation (inferred): P74-C alone ≈ 0.76-0.84x of current MTF+RLE2, from T2/T0 0.72-0.76 divided by #170/#172's 0.90-0.96. MTF+RLE2 is 15.0-15.7% of four-copy CPU and inflates only x1.33-1.54 under SMT; assume 50-100% of the alone gain survives co-run.
  test: Rebase the P74-C kernel onto master: keep #170's run stores, make the ymm list byte-valued per #172, have both kernels reserve max+32, and decide on the guard in the registration. Open it as a PR so the PR-tier bench runs 2-3 times.
Register on wikidata 1t wall (resolves ±0.5-1%) and 4t CPU, with 4t wall reported.
This is the strongest existing construction for coordinator queue item #184 (MTF ser
  score: ≈0.50 x 2.0% = 1.0% of 4t wall (≈3.4 ms). Rank 1.

- NOGO Fused post-walk: RLE1 over the walk's segments with the CRC folded in (#183, research/fused-postwalk beef4e8f). Includes #48's slice-chain RLE1.: of=The decision rule and its power.
- The registration required 4t wall beyond noise. PR runs: 7763 -4.2% (-10.0,+1.8); 9V45 +1.5% (-3.9,+7.2) (41986885355).
- The fusion-alone attribution run (41986908473, Xeon 8370C) gave 4t wall -1.5% (-2.4,-0.6) and CPU -1.9% (-2.8,-1.0). A text 2t +3.4% (+0.0,+6.8) flag failed it, and the clause gave no rerun to settle it (fused-postwalk.md:151-153).
- 1t was faster beyond noise in both PR runs (-1.1%, -2.5%).
  changed: No code premise changed: it was measured on today's src. The interpretive premise, 'post-walk traffic doesn't bind 4t wall' (fused-postwalk.md:155-158), is contradicted by:
- 4t saturation (0.961, 41980640481), with CPU cuts reaching the wall since #180
- post-walk stages inflating under SMT: assembly x1.77 (8370C) and x2.97 (7763), RLE1 x2.08 and x1.53
- post-walk at 7.2-7.4% of four-copy CPU (42008210159, 42011789918)
  now: Faster: 4t wall -1 to -2% (lo 0, hi -3%; 0-10 ms); 1t -1 to -2.5%.
Ceiling: post-walk's ~7.3% of 4t CPU, less RLE1's irreducible read of n and write of m.
Risks: `runs` 4t (+1.3%, flagged once) and `text` 2t.
  test: No code change. Register a pooled criterion first: at least 3 fresh PR runs of beef4e8f rebased onto master, with the pooled 4t wall and 4t CPU intervals, and a flagged row settled by one rerun.
  score: ≈0.45 x 1.5% = 0.68% (≈2.3 ms). Rank 2.

- NOGO First-column packing in the build (the write-only scatter): bz2ngrs.md §4 hypothesis 1. Parked, never built.: of=A premise measured on another code base: bz2ngrs's build was 1.4-1.9x slower than ours even net of its count, because it lacks run stores (bz2ngrs.md:100, :146-164). It was parked behind a workstation run that never happened.
  changed: - #114 removed the init pass. What remains between ours and a write-only scatter is the read of ll at each destination slot: 256 extra read streams beside the 256 write streams (d1fd4405:src/bwt.rs:310, :325-327).
- #114 on CI: build -6.0% at 1t, but 4t CPU +1.1% (+0.2,+2.1) (one-pass-build.md:98-101). That is the sign expected if these reads cost under SMT.
- Under four copies the build inflates x2.25 on Xeon 8370C, the largest of any stage (42008210159), against x1.43 on EPYC 7763 (42011789918
  now: Small and CPU-model dependent: build at 4t -5 to -15% on Intel, about 0 on Zen 3; 4t wall 0 to -2.5% (mid -1.2%, about 4 ms).
Cost: the walk emits F[j], so every segment's first and last byte and the start handling move by one. That touches the room walk, staging, the segment oracle and Miri.
  test: First time the build alone in the bench/alternatives `kernels` table, a write-only build against today's. Only if the build is faster, open a full PR, judged on 4t CPU and wall and the 1t build stage.
  score: ≈0.20 x 1.2% = 0.24% (≈0.8 ms). Rank 3.

- NOGO Huge pages (THP) for the T-vector and segment buffers: #40 closed; parked at walk-round.md:100.: of=Machine and authority:
- the i9 measured page walks at 1-2% of cycles at 2-10 workers, with no SMT co-run (workstation-scaling.md:176)
- 'no change' on the shared VM at 1 thread (compute-frontier.md:470)
- now 'allocator-adjacent, wait for Louis'
  changed: CI's 4t is SMT co-run at window 16. Two T-vectors plus segment buffers per core (worker buffers 12.4-15.9 MB, 41976148952) roughly double the per-core page footprint that the i9 and VM figures covered. Runners are VMs with nested paging (inferred: costlier walks). No counters on CI.
  now: 0 to -3% at 4t (mid -1%); the sign cannot be shown without counters.
  test: Needs the maintainer's authority first. Then madvise(MADV_HUGEPAGE) on tt and seg.bufs in a PR, read on 4t CPU. A subagent may not build it under the current rules.
  score: ≈0.2 x 1% = 0.2%. Authority-gated.

- NOGO compact30 records as the default inverse BWT (#155). Its staged form is coordinator task #185, in progress.: of=Implementation against an old baseline. Record walk +31% Ir (+5.3 Ir a row) against the constructor's -19.5%. CI 41908271917 (EPYC 7763): wikidata wall +18.7% at 1t, +11.0% at 2t, +7.3% at 4t; CPU +8.2% at 4t; RSS -10%; amplification -8.8%. Base c4b69de predates the #157 unroll, #171 staging and #180's window.
  changed: The T-vector walk has since got #157 (-22% Ir) and #171 staging (4t -5 to -14%; amplification 2.23 -> 1.84x on 9V45). compact30's own walk is still unstaged. Its 4t edge was lower amplification on a decoder that was not then saturated; now saturated, it is 4t CPU that counts.
  now: Still slower: +5 to +15% at 1t and +2 to +7% at 4t. A flip needs the record walk at or below about 18.3 Ir a row (#155's closing comment).
  test: As #185 registers it: staged arms against P on the PR bench, after a Callgrind check that walk Ir a row is at or below master's.
  score: ≈0.10 x 2% = 0.2%

- NOGO Build: a run's 16 entries as four SSE2 stores (#169): of=Power: build stage -3.1% (-3.5,-2.7) on 8573C, with no decoder figure beyond noise, before #180.
  changed: Since #180 the 4t decode is saturated, and the build is 18-21% of four-copy CPU.
  now: -0.3 to -0.8% at 4t, under the 4t rows' ±2-5%.
  test: Only as a rider on a build PR such as first-column packing; alone it cannot resolve.
  score: ≈0.15 x 0.5% = 0.08%

- NOGO The room walk's tail in passes (#173): of=Formulation plus power. The tail used the deferred-flag replay (a second row slot and an out-of-line spec_offset), which #181 later showed costs time. -2% Ir: no wikidata gain on 7763 or 9V45; pattern -4.0% and -7.9%.
  changed: #181 removed the deferred flags. A tail in passes using #181's per-lane start test has never been built.
  now: At most -0.5% on wikidata at 4t. Segment counts 256 and 1024 are within noise of 512 in today's sweep (41989592674, 9V45), so the tail's share is small.
  test: Callgrind the tail Ir; bench it only when bundled with another walk change.
  score: ≈0.15 x 0.4% = 0.06%

- NOGO Pair walk ψ², LF² and libsais's 2-byte walk (#29, #34; E1, E6): of=Implementation plus machine:
- the bigram build costs 3x ours, 7.1-7.6 against 2.5 c/B (pair-walk.md:53-58)
- CI wikidata +40.6% at 1t and +24.3% at 4t against 0.1.4
- on the i9 it wins only past the knee: 1.107-1.117 at 8-10 workers, 0.816 at 4 (expert-review.md:262-291)
It was closed under the instruction-bound premise (pair-walk.md:61).
  changed: The instruction-bound premise was reversed (walk-round.md:67). The bench's arm order was fixed in 3.0.0; the earlier bias favoured head, so the rejection stands (early-warning.md:118).
  now: Still at least 10% slower at 4t on CI. Half the dependent loads would help a miss-bound walk, but the build adds about 5 c/B and the build inflates x1.4-2.25 under SMT.
  test: Nothing worth running on CI. It is an i9 question at 8 or more workers, a regime the team policy already avoids.
  score: ≈0.02 x 3% = 0.06%

- NOGO Lane counts 16-64 and prefetch hints L2, NTA, off and late (walk round; the sweeps): of=The idea, on today's kernel: sweeps 41989592674 (9V45) and 41989596336 (9V74) at 0769b52f found no setting faster beyond noise, alone or with four copies. Locally L1 was fastest; NTA was +33% at 4t.
  changed: None: src is unchanged since.
  now: Neutral. 48 lanes was -3.3% alone on 9V45, not beyond noise.
  test: Closed.
  score: ≈0.05 x 1% = 0.05%

- NOGO Staging tile at 512 and 1024 offsets (#177, #178): of=The mechanism on 32 KiB L1D: 1024 was +23.5% at 1t and +8.2% at 4t on 9V74; 512 was confirmed harmful on pattern. Local VM gains did not transfer. Baseline 17daef2.
  changed: Only #181, which drops the 128 B replay slot: negligible.
  now: Neutral on 48 KiB L1D (8573C: 512 at -0.8%, not beyond noise); harmful on 7763 and 9V74. A per-CPU tile is worth at most 1%.
  test: Closed unless a dispatch on L1D size is wanted.
  score: ≈0.05 x 0.8% = 0.04%

- NOGO Precopy: backward neighbour search (#30), forward chains (#36): of=The mechanism: the copied steps were the cache-resident ones (precopy-forward-chains.md:47-52). +118% and +6% at 4t.
  changed: The instruction-bound premise was reversed, but copies still replace steps that don't miss.
  now: Neutral to slower.
  test: Closed.
  score: ≈0.03 x 1% = 0.03%

- NOGO Compact ψ tables: 20/24-bit (#91, #112 u), FOR-14 records (#130), move structure (#42, E8): of=Per-step work against an L3-capacity mechanism measured on the i9: u's walk +16-28%; the move step 41 against 16 instructions, +90-108% under the room walk.
  changed: On CI the 4t inflation is mostly within-core SMT sharing (2t spread 1.04x against 4t 1.62x; packed against spread +40%), so footprint is the minor term there.
  now: Slower on CI.
  test: Closed for CI; an i9 question only.
  score: ≈0.02 x 1% = 0.02%

- NOGO Split condition variables; byte-budgeted admission; self-scanning workers; a window wider than 4t: of=The idea, counted: about one wasted wake a block, 0.2-0.4% of CPU (split-condvars.md:49-56; local VM without SMT).
  changed: Since #180 and saturation the bound is smaller still: 4t CPU/(4*wall) 0.961, and worker idle 5% (41976148952).
  now: At most 0.5% at 4t on CI.
  test: Closed. Reopen only with many more threads than blocks in flight.
  score: ≈0.05 x 0.5% = 0.03%

- NOGO Shape dispatch for co-resident walks (16 lanes with prefetcht1): of=The idea on the staged kernel: +62.6% and +28.0% alone, +11 to +54% with four copies (coresident-walk.md:79-83).
  changed: None.
  now: Slower.
  test: Closed.
  score: ≈0

- NOGO AMAC and dynamic lane scheduling (E10); the vpgatherdd walk: of=The idea: every walk step has the same shape, and coordination is about 4% (expert-review.md §3). The gathers cost more than the scalar loads they replace and leave no room for the per-lane prefetch (staged-walk.md:54-62).
  changed: None relevant.
  now: No gain.
  test: Closed.
  score: ≈0

- NOGO Cooperative teams below 6 physical cores; level-aware plan (#115): of=Machine and policy: the i9's 4-core pass read 0.997; #115 failed its mixed-concatenation gates (i9 only).
  changed: Window is now 4t. Forced teams on CI ran at 491 MB/s against the workers' 695 MB/s (0.71x, unpaired; occupancy-teams in 41976148952, EPYC 7763).
  now: Teams would be 25-30% slower on CI. The level-aware plan does not apply there: teams never engage.
  test: Closed for CI.
  score: 0

- NOGO Deferred flags in the busy phase (#168, tiled-B); #23 4-lane build; #27 per-block walk choice: of=#168: formulation and codegen, the replay slot and the out-of-line spec_offset; #181 won by removing them. #23 was superseded by #32's run stores. #27: noise on wikidata.
  changed: Settled.
  now: No gain.
  test: Closed.
  score: 0

- NOGO Parallel scanner / speculative Huffman in the workers (#10): of=Machine: the scanner binds only from about 8 workers (the whole-dump model in 41980640481).
  changed: At 4t on CI the scanner is 0.13-0.14 s of CPU (about 10%) on a saturated machine. Moving its work to the workers doesn't cut CPU.
  now: 0 at 4t on CI; possibly positive on the i9 at 8 or more threads.
  test: Closed for CI.
  score: 0 on CI

DISPATCHED: None. The decisive missing measurement was the SMT inflation of each stage on production code: sweep.yml tool=phases, alone against four copies. Another front had already dispatched it on master, as runs 42008210159 (Xeon 8370C, b7734313) and 42011789918 (EPYC 7763, 4bfaf7cb), with src identical to d1fd4405. I used those runs rather than spend my one dispatch.
UNKNOWNS: - **Four copies are not four threads.** The phases shares come from four processes, each with its own scanner and no window. Using them as the 4-thread shares is inferred; the support is that the four-copy total inflation (x1.62) matches the decoder's 4t amplification on 8573C (1.62x).
- **The ranking depends on the CPU model.** The walk/build inflation reverses between Xeon 8370C (walk x1.46, build x2.25) and EPYC 7763 (walk x2.12, build x1.43). Phases on 9V45, 9V74 and 8573C have not been measured, so build ideas such as first-column packing are worth more on some runners than others.
- **P74-C's margin after #170/#172 is inferred, not measured.** It is also unknown whether the ymm head can be byte-valued (#172) at no cost; the P74-C head holds indices and maps through seq_to_unseq, per its asm-audit.
- **No PMU on any runner.** There are no store-forwarding, fill-buffer or dTLB counts, so the MLP and THP premises are inferences. The runners' THP setting is unknown.
- **The 4t rows' ±2-5% half-widths are as large as the predicted 1-3% effects.** The live candidates need 1t or CPU criteria, or pooling across runs.
- **The i9 has not been measured since #171, #180 and #181.**
- **Fewer, longer passes** (walk-round.md:89-91: half of each block's 142 chunks are a pass's short last chunk) is untried, not a no-go, and is unpriced.
NEXT: P74-C rebased onto master (keeping #170's run stores, with a byte-valued ymm list per #172, both kernels reserving max+32), opened as a PR so bench.yml's PR tier measures it in 2-3 runs. Register it beforehand on wikidata 1t wall (±0.5-1% resolves it) and 4t CPU, with 4t wall reported.

Predicted: 1t -2 to -4%, 4t -1 to -3%.

What it decides: whether rank 1 flips. A 1t gain at or below 0.5% falsifies the claim that the register-held head still has margin after #170/#172.

In parallel and independent of it: re-register #183 (beef4e8f) on a pooled 4t criterion over at least 3 fresh runs, with no code change. Both are the coordinator's call, and P74-C overlaps queue item #184.

#### FALSIFIER

- FALLS: Binding 1: at 4t on CI the decode is CPU-saturated (CPU/(4*wall) 0.961), so every idle or scheduling idea has at most about 4% to recover.
  WHY: The saturation half survives, recomputed from 41980640481 runs.jsonl (Xeon 8573C), wikidata, mibz2 arm, 4t, 10 rounds: wall 0.3426 s, CPU 1.3139 s, median ratio 0.961 (range 0.855-0.970). The occupancy run agrees: decoding 93.0%, 3.72 of 4 (41976148952, EPYC 7763).

The 'scheduling' half does not survive. The 3.9% bounds idle time only. Scheduling also moves the larger term, CPU per block under SMT co-run:
- #180, a pure scheduling change, raised 4t CPU by 4.3-6.1% (window.md:102-104, 114-117);
- packed against spread at 2t was +40% (four-thread-ceiling.md:51-53);
- forced teams ran 0.71x.
The co-run term is about 1 - 1/1.62 = 38% of 4t CPU (OBSERVED 1.634x amplification from the same runs.jsonl). No measurement bounds what co-run-aware placement could recover from it.
  CORR: Idle-only ideas: at most 3.9% (about 13 ms of 343 ms), which holds. Co-run-aware scheduling, meaning what runs beside what on a sibling: unbounded by this data and never examined.

- FALLS: Binding 2: per-stage SMT inflation from four pinned copies. The four-copy shares are the ceilings for stage ideas at 4t, and the total x1.62 matches the decoder's 4t amplification.
  WHY: The numbers reproduce exactly from the artifacts:
- Xeon 8370C (42008210159): walk x1.46 / 48.8%, build x2.25 / 20.7%, MTF x1.54 / 15.7%;
- EPYC 7763 (42011789918): walk x2.12 / 52.7%, build x1.43 / 17.7%, MTF x1.33 / 15.0%, total x1.71.

The interpretation does not survive.
- The 'match' compares two CPU models: four-copy 8370C x1.62 against the decoder on 8573C at 1.62x.
- On the same CPU (EPYC 7763) the four-copy total is x1.71, against the decoder's 2.26/1.40 = x1.61 ms a block. That decoder figure comes from occupancy at c90b3625, which predates #181.
- Each stage is the fastest of 5 passes, and co-runners are not controlled (coresident-walk.md notes this).
- A stage change also moves its siblings' inflation. #171's walk change cut amplification from 2.23x to 1.84x (coresident-walk.md:103-105).
So the shares are proxies, not ceilings.
  CORR: Use them as rough shares with an error of a few points. They are not upper bounds.

- FALLS: Binding 3: since #180 a 4t CPU cut reaches the 4t wall; before #180 it did not, so CPU-only gains rejected before #180 were judged under a premise that no longer holds.
  WHY: There are pre-#180 counterexamples:
- #170 run 2 (EPYC 7763): 4t wall −1.9% (−3.6, −0.2) with CPU −2.0% (−3.3, −0.7) (mtf-short-runs.md:60);
- #172 run 1: CPU −2.2%, wall −2.2%;
- #171 itself gained 4t wall.

The rejections the front would flip did not fail on CPU-to-wall. They failed because the 1t gain never became a 4t CPU gain:
- #25: 1t CPU −1.7%, 4t CPU −0.6% (ns), on bench 1.0.0, whose arm order favoured head by about 2% (early-warning.md:104-121);
- #114: 4t CPU +1.1%.

#180 does not touch that link. After #180, #183 on EPYC 9V45 had 1t CPU −2.5% (−4.1, −0.8) and 4t CPU +1.0% (−3.8, +6.1) (artifact 13396557790).

CPU ≈ wall at saturation is supported (INFERRED from 0.961). The history claim and its corollary are not.
  CORR: Only an observed 4t CPU cut maps about 1:1 to 4t wall. A 1t gain becomes a 4t CPU gain at an unpinned ratio: 0 to over 1 across the post-#180 runs, depending on CPU model.

- FALLS: P74-C (an MTF list head held in a ymm register) is the largest unshipped decoder gain. Its no-go was registration gates and an integration defect, never the CI PR bench, and it would recover −0.5 to −3.9% of 4t wall (rank 1, score 1.0%).
  WHY: (1) The idea was benched on CI. PR #25, 'the list's first 32 entries kept in two registers' (run 40526993242, Xeon 8573C), was closed on the bench:
- wikidata 1t −1.6%, 4t wall +1.0%, 4t CPU −0.6% (ns);
- random +11.1% / +8.2%, records 4t +9.7%, text +2-3%.
The bench's arm order favoured head by about 2% then, so the wikidata figures overstate it. P74-C's bypass fixes the control regressions, but #25 is the only SMT 4t datum for a register head, and it is null.

(2) The i9 no-go was about effect size, not only gates. Its w4 regime was already saturated (postmortem §3). The primary was B/A 1.036, with its own A′/A null at 1.025 [1.005, 1.044]; without round 1, 1.028 against 1.019. That is about +1% net. CPU-s/GB was 0.977-0.984.

(3) The upper bound is miscomputed. It applies the alone fractional gain (16-24%) to the inflated four-copy share. Applying the alone absolute saving instead giv
  CORR: 1t wall: −2 to −4% (likely). 4t wall on CI: 0 to −2.7%, mid about −1% (about 3 ms of 343 ms on 8573C). Score about 0.5 × 1.0 = 0.5%, within uncertainty of #183's, and with no 4t CI measurement behind it.

- SURVIVES: The no-gos of the pair walk, Precopy and the move structure rested on an instruction-bound premise that the walk round reversed. The reversal does not rescue them; recoverable about 0.
  WHY: The premise lines check (pair-walk.md:61; move-structure-walk.md:122, :143; mlp-scheduling:40-41). The losses were dominated by build cost (7.1-7.6 against 2.5 c/B) and per-step work.

The 'reversal' (walk-round.md:67) is itself a 'reading that fits', INFERRED with no counters. Its weight here is moot, because the pair walk's halved steps did not halve its walk time (≈10 against 10.4-10.8 c/B, pair-walk.md:53).
  CORR: 

- SURVIVES: Cooperative teams lose on CI-sized machines: forced teams decode at 0.71x the workers at 4t.
  WHY: OBSERVED in 41976148952 (EPYC 7763): forced-team rounds 478.5 and 503.9 MB/s against worker rounds 631.7-702.7 MB/s (median 694.9). The comparison is unpaired, with 2 rounds against 6.

It is a confirmation, not a new no-go: Plan::for_threads never engages teams below 6 physical cores. Recoverable 0 either way.

One supporting figure is wrong. 2t-spread amplification is 1.11x on 7763 (1.55/1.40 ms a block), not the 1.04x taken from 8573C. So the cross-core share of the 4t inflation is about 20% on a log scale there.
  CORR: 

- FALLS: Two earlier no-gos have already flipped after a premise change (the window; the walk unroll), so closures on i9 gates or pre-#171/#180 baselines are not final.
  WHY: Window: #53 demoted it with a 7-8% upper bound (four-thread-ceiling.md:16-21), and the window registration put the bound at 10% (window.md:33-34). #180 then realised −8.1 to −10.3%, about the bound. The bound held; only the ranking changed.

Walk unroll: the i9 NEGATIVE was B/A 1.000 at w4 (status.md in 270333c2). On CI, #157 measured 4t wall +0.2% (−2.1, +2.7) (41925226768), so the 4-worker null reproduced. #157 merged on its 1t (−2.4%) and 4t CPU criterion. It was also a different construction (perf/walk-two-phase, not research/walk-unroll-decoder).

Neither is a mechanism flip. If anything, #157 shows that 4-worker nulls carry over from the i9 to CI.
  CORR: 

- SURVIVES: Fused post-walk (#183) failed on its decision rule and power, not its mechanism. Its interpretive premise is contradicted by saturation, so a pooled re-run should give −1 to −2% at 4t (score 0.68%).
  WHY: Its closing note (fused-postwalk.md:159-162) says the 1t and fusion-alone gains 'need a registration of their own, judged only on fresh runs', so the pooled fresh-run test is legitimate.

The three 4t wall rows (7763 −4.2%, 9V45 +1.5%, 8370C −1.5%) average about −1.4%, inside the prediction.

The 'premise changed' argument does not hold. All four runs were at #180's saturation already, and nothing has changed since.

The 7.3% ceiling is too high. The fusion keeps RLE1's work and the CRC computation; it removes the assembly copy (2.7-3.2% of four-copy CPU) and the CRC's re-read. Harm risks remain: `runs` at 4t +1.3%, `text` at 2t +3.4%, and 9V45's 4t CPU +1.0%.
  CORR: 4t wall 0 to −2% (pooled point about −1.4%). Ceiling about 3-4% of 4t CPU, not 7.3%. A power case, not a premise change. It has better 4t evidence than P74-C.

- SURVIVES: First-column (write-only) build scatter: its premise changed when #114 shipped. Rank 3, about −1.2% at 4t, CPU-model dependent.
  WHY: The code claim checks: build_one_pass reads ll[s] at each destination slot (d1fd4405:src/bwt.rs, the one-step closure and the 16-row path). The low rank is honest.

The supporting sign is ambiguous. #114's CI 4t CPU was +1.1% (+0.2, +2.1), but the i9 4t CPU was 0.971 (one-pass-build.md:98-112). The front's reading of the CI sign as the cost of those reads under SMT is INFERRED.
  CORR: 0 to −1.5%; keep it as a build-kernels-table screen first.

- SURVIVES: Huge pages for the T-vector and segment buffers: authority-gated, 0 to −3% at 4t, sign unknowable without counters.
  WHY: Pending the maintainer (walk-round.md:100). There is no core PMU on any runner (PMU probe 40857395037: core_pmu false, every hardware event 'not supported'). Rightly unpriced.
  CORR: 

- SURVIVES: compact30 (#155) stays slower; #169, #173, lane counts, prefetch hints, tiles, Precopy, the compact ψ tables, split condvars, the co-resident walk, AMAC and gathers, deferred flags and the parallel scanner stay closed or riders.
  WHY: #155's figures match its bench comment (41908271917: +18.7%, +11.0%, +7.3%; CPU +8.2%), and so does its 18.26 Ir-a-row revival condition. The co-resident result matches coresident-walk.md:79-83, and no setting beat the default beyond noise.

One caveat on the compact-ψ closure. 'Footprint is the minor term' rests on 8573C's 1.04x 2t-spread figure; 7763 shows 1.11x. The closure stands anyway, on its per-step costs.
  CORR: 

MISSED: 1. PR #25 is a CI PR-bench no-go for the register-held MTF head (run 40526993242, Xeon 8573C, bench 1.0.0).
   - The front lists #25 in P74-C's lineage but says the idea never reached the CI bench.
   - #25's 4t result was null: wall +1.0%, CPU −0.6% (ns). Its 1t −1.6% is within the head-favouring order bias of about 2% (early-warning.md:104-121).

2. The P74-C i9 postmortem already measured a saturated 4-worker regime (§3: 'w = 4 is a saturated regime'), so #180's saturation is not a new premise for it.
   - Its primary cell's own A′/A null was +2.5% [0.5, 4.4], leaving the net w4 gain at about +1%.
   - CPU-s/GB at w4 was 0.977-0.984.
   - The front quoted B/A 1.013-1.036 without the null.

3. The P74-C upper bound has an arithmetic error: it scales the alone fractional gain by the SMT-inflated share.
   - With the alone absolute saving: 1.98 or 2.22 c/B × 0.16-0.24 ÷ 17.46 or 21.82 c/B ≈ 1.6-2.7%, not 3.9%.
   - The quoted T2/T0 0.654-0.747 is the teams-only checkpoints path; the workers' path is 0.719-0.735.

4. #180 itself raised 4t CPU by 4.3-6.1% while cutting wall by 8-10% (window.md:102-117).
   - Scheduling therefore moves the co-run CPU term, about 38% of 4t CPU (1 - 1/1.634, from runs.jsonl).
   - The 4% 'scheduling cap' covers idle only. Co-run-aware placement (which stage runs beside which on a sibling) was never examined, and no journal entry tests it.

5. Transfer from 1t to 4t CPU is the unpinned link behind every rank-1 and rank-2 prediction, and it varies with the CPU model:
   - #183 on EPYC 9V45: 1t CPU −2.5% beyond noise, 4t CPU +1.0%;
   - #183 fusion alone on Xeon 8370C: 1t −2.3%, 4t CPU −1.9%;
   - #181 run 3 on EPYC 9V45: 1t −0.9% (ns), 4t −3.2%.
   No candidate's 4t figure should be derived from 1t or stage gains without a 4t CI run.

6. Like-for-like amplification checks the front skipped:
   - EPYC 7763: four-copy x1.71 against the decoder's x1.61 (occupancy, pre-#181), not the 8370C/8573C 1.62 = 1.62 pairing;
   - 7763's 2t-spread inflation is 1.11x, not 1.04x.

7. Ranking consequence. P74-C (mid about −1% at 4t, no CI 4t data) and #183 (pooled about −1.4% from three CI 4t rows) are within each other's uncertainty. #183 has the better-pinned 4t evidence. P74-C's distinctive strength is 1t, which the decisive test should be registered on, with 4t CPU as the second criterion.

8. The THP item rests on the runners' THP setting, which nobody has recorded. It can be read without a PMU, from /sys/kernel/mm/transparent_hugepage/enabled in any CI job, before anyone asks for authority.
