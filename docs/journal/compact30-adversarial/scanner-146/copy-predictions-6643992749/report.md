# Per-copy predictions: reconcile runs and model before fitting

Trigger: [6643992749](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643992749), 2026-11-04.
Read the prior register and [B2 at 9723f068](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-b2-gathered-walk.md), the pinned #157 source, and the updated benchmark comment. Research only: no implementation or experiment run.

**The proposed per-copy measurements are useful. The checkpoint currently mixes two hosts/runs, identifies unlike kernels, and treats a non-identifying timing fit as a parameter measurement. Correct those before assigning numerical forecasts.**

## 1. The 0.925 walk ratio is from the 7763 replicate

The mutable [bench comment 6643479692](https://github.com/lmmx/mibz2/pull/157#issuecomment-6643479692) changed at 22:33:47Z. Its completed report is now [run 41925226768](https://github.com/lmmx/mibz2/actions/runs/41925226768), **EPYC 7763**, head 3fc3f8c. The previously read report was [run 41923819666](https://github.com/lmmx/mibz2/actions/runs/41923819666), **EPYC 9V45**, head 75b13f0. Both compare c4b69de; 3fc3f8c is the test-only child.

| Quantity, paired ratio [95% interval] | 9V45 first run | 7763 replicate |
|---|---|---|
| One-thread walk-stage time | .942 [.847,1.049], medians 3.89→3.63 | .925 [.856,.999], medians 4.04→3.78 |
| One-thread stage total | .973 [.927,1.022] | .961 [.927,.995] |
| Whole-decoder wall, 1 thread | .958 [.920,.997] | .976 [.964,.988] |
| Whole-decoder wall, 2 threads | 1.022 [.994,1.051] | .951 [.905,.999] |
| Whole-decoder wall, 4 threads | 1.004 [.972,1.038] | 1.002 [.979,1.027] |

Thus the checkpoint's .925 and −3.9% stage total cannot calibrate the 9V45 prediction. Nor should they be paired with the first run's −4.2%, +2.2%, +0.4% wall figures. The newer run reports CPU-cost improvement, no flagged worsening, and no detected amplification change; this does not certify general scaling.

The .925 estimate is the geometric mean of paired round ratios, not 3.78/4.04=.93564. The difference is expected for these estimators and is not an arithmetic error. Preserve raw pairing.

The [phase harness at 3fc3f8c](https://github.com/lmmx/mibz2/blob/3fc3f8c64fe46eed325208d0d91e6586ebf8d2c1/examples/phases.rs) prints the used walk as **tm.walk + tm.assemble**; assembly is a nested row. B2's boundary excludes assembly. The harness uses Instant elapsed seconds, each stage's fastest pass, then scales by the supplied/reported GHz. Its c/B is not a hardware cycle counter. Pure-traversal paired ratios cannot be recovered by subtracting independently rounded medians.

## 2. #157 is related to B2 B, but not its specified kernel

At [75b13f0 src/bwt.rs](https://github.com/lmmx/mibz2/blob/75b13f009c619300bc914750b1a4a44f365def2e/src/bwt.rs), room_segments resets marked for each offset, tests each q's START bit, immediately writes each unmarked byte to base[k]+off, advances its row, and handles marked lanes after that round. Eight-way unrolling is **eight different lanes**, not eight successive steps buffered per lane.

B2 §§5/8 B instead ORs flags across an eight-step tile, writes into temporary tiles, commits eight-byte outputs, and replays failed lanes from saved starting rows. Its estimated commit, replay, address/port costs and instruction-window behaviour belong to that construction. #157 has the same broader Path-B aim and 32-chain count; similarity does not transfer that ledger.

Moreover, nine **instructions** is not evidence for eight-to-nine **macro-ops**, and a hot-step count is not a whole-walk count. Consequently “the instruction side held” is unestablished at B2's units and boundary. A measured ratio overlapping the old B2 range is consistency under unverified transfer assumptions, not validation of (iv). The very large gain predicted by a naive instruction-proportional model is unsupported; that does not exclude every instruction/port-limited explanation. The miss-cap scenario remains compatible with the timing intervals.

## 3. Predictions that can be registered before measurement

Define for placement p:

    w_X,p = I_X,p + ell_X,p q_X,p + h_X,p
    q_X,p = max(U_X,p/R_p, 1/m_X,p, 1/K_X,p)
    rho_p = w_B,p / w_A,p

I includes the binding issue/port cost; h includes handover, replay and tail at the chosen boundary. This is B2's conditional modelling form, not a new hardware law. Use the actual candidate's costs. Let p be one worker, two workers on separate physical cores, two workers on SMT siblings, and optionally four workers on two cores.

| Candidate explanation | Separate-core prediction | SMT-pair prediction | Discriminator/qualification |
|---|---|---|---|
| Transferable uncapped interval model, equal loaded latency, proportional issue/work terms | rho approximately rho_1 if each core's resources and interference are unchanged | rho approximately rho_1 when both issue and window terms scale equally | Predict ratio-of-ratios near 1, not identical absolute c/B |
| Only common ell rises, both arms window-bound with I proportional to U | rho stays U_B/U_A even as both slow | Same if both remain in that regime | Rising ell alone does **not** explain disappearance of the gain in this regime |
| Common request/chain/traffic penalty dominates both arms | rho moves toward 1 as that common penalty grows | Same trend if SMT aggravates the common limit | Absolute costs should rise; several resources can produce this pattern |
| B increases loaded latency more than A (risk 5) | rho rises only if ell_B q_B grows sufficiently relative to ell_A q_A | May rise further if sharing aggravates it | Neither direction nor magnitude follows from “higher MLP” alone; loaded latency is unmeasured |
| New core-shared issue/port/queue limit (risk 6) | No necessary drift if the resource is private per core | Gain erodes specifically under sibling placement | Loss already on separate cores requires another explanation or additional sharing |
| Shared-cache capacity/request/write interference | Can erode the gain with separate cores | May persist or change with sibling placement | Placement alone cannot distinguish these; reuse/neighbour controls address different sensitivities |
| Scheduling or pipeline limitation | Embedded CPU walk may retain gain while elapsed walk/wall does not | Same possibility, plus core sharing | Off-CPU wait must be separate; elapsed-stage rho→1 does not prove a hardware limit |

A ratio near one is evidence about the tested stage and placement, not unique identification of m, ell, bandwidth or cache. A retained isolated-stage gain with no decoder-wall gain says the isolated improvement failed to transfer; production overlap/feedback can still change the walk itself. Only matching production stage measurements justify locating the residual elsewhere.

Two workers on separate cores and two workers on SMT siblings are distinct cells. Ordinary 1/2/4-thread decoder runs give the former and four workers across two cores, **not an isolated two-worker SMT pair**. Scanner/reader placement and runnable-thread load must be recorded; the measured decoder includes those threads, whereas B2 §8's kernel falsifier does not.

For a formal constant-ratio check, estimate paired log(rho_p)−log(rho_1) with the round structure preserved, and state a practical tolerance. Do not compare a walk prediction directly to a whole-decoder wall interval or infer a confidence level from overlap of separately estimated intervals. Five diagnostic stage rounds also carry less precision than ten decoder rounds.

## 4. ell and m are not pinned

Even with a known I_A and h_A, one timing gives only:

    w_A − I_A − h_A = ell_A * max(U_A/R, 1/m_A, 1/K_A)

If m limits, only ell_A/m_A is identified. If the window or K limits, timings give at most a bound on m. With unknown actual macro-ops, port cost, tail and effective clock, more unknowns remain. Two candidates do not fix this when their latency or limiting regime may differ. Therefore no defensible fitted ell or m for 9V45 follows from the checkpoint.

A numerical sensitivity example makes the ambiguity concrete without pretending to fit hardware. Assume the **7763** rho=.925, hypothetical issue ratio r=9/14.8=.6081, and equal additive miss penalties. Then rho=r f+(1−f) implies issue share f=.1914. Holding A fixed, a B-only miss-penalty multiplier g makes rho=r f+g(1−f); g=1.09275 erases the gain. This is a conditional 9.3% sensitivity, **not a measured latency increase**: r mixes unverified units and different kernels, so it is explicitly a toy assumption.

The corrected model should retain a family of admissible parameters and predict each regime's response, rather than select ell and m to reproduce one number. Existing matched phase instrumentation can test gain retention; actual macro-op/port evidence, loaded-latency evidence or a discriminating concurrency/chain sweep would be needed to resolve the parameters. These are information requirements, not newly imposed gates or authorised experiments.

**Disposition:** accept the retraction of a proven contention cause. Reject the cross-run 9V45 calibration and exact B2-kernel identity. Keep the proposed per-copy question, with correct boundaries, placement and CPU-versus-elapsed interpretation. No implementation, capture, performance PASS or authority change.

[Arithmetic and run ledger](audit.json); [source-comment snapshots](bench-snapshots.json).
