# Researcher review: The knee is gone — round 1

**REVISE the article; retain the release result.** Assignment [6955971254](https://github.com/lmmx/mibz2/pull/162#issuecomment-6955971254), source `ccba7bb644995cf1b958b979cdf1172fb90c486a`, merged PR550. Scope: this article only; companion round4 is not this assignment. No implementation, timing, merge or release performed.

## Independent reconstruction and method

Receipt: 8a6db3c4d740704c6b605c7f0d873e90ef7155bf. The primary-record baseline and raw reconstruction were committed at **55d9c4e26e172b14f95aae961be9b8556819705e before reading the article or PR550's explanation**; see baseline.md and raw-summary.json here. Prior involvement in the research prevents a claim of blind independence. The new arithmetic and counterexamples were derived from primary records, rather than accepting the author's verdict.

Read all 16 available release-scaling runs.jsonl files spanning 0.2.4–0.5.4 at the reviewed source commit, including 0.4.0 as the older arm of 0.5.0's run; the release attribution ledger; LOGs 6946756938, 6952656453, 6948385717 and 6949008682; the original three-team registration results; and the historical IMC bundle. Following the baseline, read article and PR550's review packet. The existing prior research supplies context; it is not a substitute for the calculations.

Recomputed medians, old/current paired geometric-mean wall ratios, and current/count-eight ratios, with log-ratio standard error and t(9)=2.262157. `release-checks.json` and `within-release-checks.json` retain these results. These are descriptive pointwise intervals from ten rounds, not independent replication or simultaneous confidence bounds. The baseline raw summary used geometric means for shape; this final audit uses medians where the article does. Neither should be silently substituted for the other.

Verified archive SHA-256:
- cells-imc/run-20261114T180428Z: **2661310a6622534408bd440717d96f171d1ee25d6850064010fd639c1063f2cb**.
- knee/run-20261117T123939Z: **b145bb1e596b719eae1cdb0a13168dd4b6c8378f42bae42cdf1a323a7792792d**, immutable blob 76fcbe7a637894709e61e895f4477614f48ced46. Read report.txt and raw record schema after extraction. 160 runs, zero failures, 320 timeline records, 60 stream records; IMC unmeasured. The connector could not decode this binary; the public GitHub blob API supplied the archive successfully.

The current unpinned and miss-cost claims were checked against the named LOG/ledger and the available earlier investigation, not freshly recomputed from every raw counter sample this run. This is not an independent reproduction of those experiments or a checksum certification of every archive cited by PR550. No external prior-art search was needed for this release-history editorial audit; no novelty claim is admitted.

## Required corrections

### F1. Affinity is not exclusive thread placement — §3 info box

“At ten threads, CPUs 0–9, so no two threads share a core” is false. The permitted CPU set has one hardware thread from each physical core; it does not bind each software thread to a distinct core. The scanner, main/reader and workers share that set and can be scheduled on the same cores. The harness's taskset mask is exactly why CPU competition remains possible.

Replace with: **“At ten requested threads, the process may use CPUs 0–9: one logical CPU from each physical core. Its workers, scanner and reader share that set.”**

This matters scientifically: the current wording encourages the reader to exclude the CPU competition invoked later in the same section. Do not replace it with a claim that each worker owns a core.

### F2. Figure3 is selected history, not every release — §2 last sentence

“Every release since 0.2.4” is not what is plotted. Figure3 selects eight releases: 0.2.4, 0.3.4, 0.4.0, 0.5.0–0.5.4. It omits 0.2.5–0.3.3.

The omission is consequential to the historical shape: in 0.2.5, ten/eight wall is **0.9184 [0.9098, 0.9271]**, and 12–20/eight is 0.9487–0.9619. A reader should not infer an uninterrupted high-thread regression throughout the intervening history.

Say **“for selected releases since 0.2.4”**; no need to crowd all releases into the chart. Keep the distinction between the old capped default, forced excess-concurrency experiments, and the real 0.5.0 dispatch regression.

### F3. IMC traffic is necessary evidence, not a sufficient causal test — Figure5 caption

“Only a reading of the memory controller's counters at the plateau settles whether memory is full” promises too much. IMC reads+writes establish traffic. To judge saturation they must be compared with a relevant measured sustainable ceiling, on the same machine and with a comparable read/write mix. To identify bandwidth as the binding limit may require an intervention that changes bandwidth while controlling competing explanations.

Replace with: **“Current memory-controller measurements, compared with a measured sustainable ceiling for a similar read/write mix, are needed to test whether bandwidth is limiting the plateau.”**

The 37.6 GB/s line is an L3-miss-read proxy ceiling; the estimated 44–45 GB/s bar is reads plus inferred writes. They are not a like-for-like numerator and ceiling. Their labels are explicit, which is good, but the shared scale encourages that comparison. Mark the 37.6 line as comparable only with the 25.3 proxy-read bar, or separate the proxy and estimated-total comparisons. The 51.2 line is correctly labelled an unverified DDR4-3200 assumption, not a measured sustainable ceiling.

### Small corrections and earlier scope

- Figures1 and2 label 0.2.4 “7 November”; its source run is **6 November**, as Figure1's caption already says. Align them.
- After Figure1, “wherever more workers would make every one of them slower” overstates the planner's reach. It selects measured plans for a recognised machine/placement, not a universal or online slowdown detector. Use “in the measured configurations where extra workers made decoding slower.”
- Bring the concrete **seven workers at requested 12–20**, and **bandwidth not yet established**, into that opening result paragraph. Both are eventually clear, but presently the strongest protection against reading the plateau as twenty-worker scaling arrives near the end of §4.

These are article corrections, not grounds to reverse the qualified release or demand a new benchmark merely to fix prose.

## Claim-by-claim evidence audit

| Claim family | Independent result and boundary |
|---|---|
| Machine and file | i9-10900K topology is ten cores/twenty logical CPUs, shared 20 MiB L3; raw output is 209,700,196 bytes. Article's 210 MB and 209.7 MB denote the same decimal quantity. The 11.9 MB compressed description is consistent with the recorded 11,854,933-byte extract. |
| Original knee and 0.2.5 | 0.2.4's capped four teams of two mostly level off. Its 2–3% drift is not the later deep knee. The original two registered speed ratios 1.089 and 1.088 support “about9%”; the separate release-scaling 1.119 is a different experiment, correctly not substituted. |
| Cells/lazy letters and team dispatch | Ledger and observed thread names support eight one-block workers at eight versus three three-thread teams at ten and above in 0.5.0. Memory work did not benefit that older team path equally. |
| 0.5.0 fall | Recomputed ten/eight 1.1048 [1.0902,1.1197]; twelve/eight 1.1807 [1.1486,1.2138]; sixteen/eight 1.1637; twenty/eight 1.1687. Thus16–18% is supported as paired geometric means; it is not the ratio of displayed medians. |
| Plans through 0.5.4, Figure2 | Distinction between requested threads, teams, and independent workers is sound. 0.5.3 pinned-ten uses ten workers; unpinned recognition selects seven. 0.5.4 trims pinned8 to6 and pinned10 to7; high counts remain7. Keep the split placement counts. |
| 0.5.4 table and output speed | Medians1/2/4/8/10/12/16/20: .38510/.24115/.14365/.10470/.10200/.10580/.10615/.10755 seconds. Eight–twenty throughput1.94979–2.05588 GB/s. Rounded article range1.95–2.06 is correct. |
| Paired release gain | Old/current10 ratio1.122599 [1.094015,1.151930], an elapsed-time reduction10.92%; eight1.045176 [1.037880,1.052524]. At4,1.020422 [.993875,1.047678];1,2,12,16,20 intervals also include1. “No change detected” is appropriate, not proof of equivalence. CPU/RSS directions agree with the release record. |
| Remaining slope | Median20/10=1.05441, approximately5–6%, depending on rounding. Placement changes across that comparison. The article acknowledges the slope and does not claim statistically established superiority of ten over every count. Figure3's10/eight interval excludes1, but does not answer all other pairwise questions. |
| 0.5.3 recognition and later unpinned check | LOG6949008682 supports production7 in6/6 checks, explicit7/10 ratio.8729 [.8656,.8803], and trims8:.9999,6:1.0088,4:1.2143. The article correctly separates later explicit-arm evidence from the shipped release sweep. |
| Narrow policy and hardware discovery | Ledger/release records support separating machine recognition from allowed placement, cached hardware discovery, and pin-only lower-count trims. The previous overclaim that these cannot share a cache mechanism is gone: “implicated” and “not settled” are appropriately limited. |
| Counter study | About-half LLC misses and about25% fewer cycles is a rounded synthesis; source estimates differ by event/run (.51 and .39 miss ratios). Instructions approximately level (.993), frequency approximately level, and .763/.925 busy shares are compatible with the stated rough values. These do not identify DRAM saturation. |
| Current plateau diagnostic | Extracted report supports ×4 unpinned compute/busy.93–.95; idle.010–.015; proxy bytes/output6.89 at4,11.02–11.06 at7,12.65 at8,14.66 at pinned10. Proxy rates25.22–25.33 GB/s at7 and triad ceiling37.592. Recorded hypotheses refute substantial measured starvation, leave fixed costs and DRAM inconclusive. Busy time is not proof of useful instruction retirement without memory stalls. |
| Old IMC series, Figure4 | Historical measured bytes/output at4/6/8/10 are15.5444/20.4229/22.7010/24.1889. Older kernel and different counters are explicitly separated; no line falsely joins it to current proxy traffic. This is an analogy, not current traffic calibration. |
| Figure5 estimate | Old writes/reads .784 and .743 imply25.3×(1.743–1.784)=44.1–45.1 GB/s. Estimate and assumed51.2 peak are properly labelled. Transferring the old ratio is unverified, and comparing that total with37.6 read-only proxy would be invalid. F3 repairs the concluding inference. |
| Prospective mechanism and portability | §5 properly conditions traffic reduction on the bottleneck and rejects universal seven-worker optimality. Larger cache/more channels can change an optimum, but do not guarantee that they will: remeasure worker-count wall and traffic, with placement controlled. |

## Takeaway placement and reader test

| Intended takeaway | First effective location | Assessment |
|---|---|---|
| Shipped curve is flatter near2 GB/s | Dek, opening paragraph2, Figure1 | Immediate and concrete; machine/workload named. |
| Requested count is not worker count | General distinction after Figure1; explicit plans §1/Figure2 | Good distinction, but put the actual seven-worker cap earlier. |
| Old contained knee differs from0.5.0 regression | §1 info box and following paragraph | Strong; avoids falsifying history for a cleaner narrative. |
| Paired release improvement versus within-release shape | §2 table and Figure3 | Strong once selected-history scope is corrected. |
| Placement-dependent gains and unresolved mechanism | §3 | Understandable after F1; current causal scope is sound. |
| Busy workers and rising traffic do not prove bandwidth saturation | §4/Figures4–5, §5 | Evidence well separated; caption F3 undermines the conclusion until corrected. |

**Simulated reader, not a fresh independent person or subagent.** Read the article straight through, then answered its goals against the prior baseline. Summary: the team dispatch path explains0.5.0's high-count drop; one-block workers and capped concurrency remove most of it; high requested counts do not imply more active decoding workers. Motivation: less interference can outweigh lost concurrency. Placement: low-count pinned gains do not transfer as a set to full-affinity use; the seven-versus-ten comparison does. Transfer: a new machine needs measured topology/traffic/worker-count performance, not automatic seven.

**Held-out response, before consulting the PR550 answer key:** halving actual DRAM bytes/output could raise output throughput if sustainable DRAM bandwidth binds, but not necessarily by2× once other costs dominate. It may barely move if the binding path is latency, shared-cache queuing, compute or a serial dependency. Current evidence does not decide which. This agrees with §5. The opening-only reader cannot yet give that answer; a single sentence exposing the unresolved plateau cause would help.

Dense points: §3 moves through recognition, affinity, explicit workers, in-flight blocks and three lower-count controls; the info box helps but currently teaches the wrong placement model. Figure2 earns its complexity by keeping versions and plans together. Figure3's ratio orientation is explained; selected coverage needs naming. Figure4 correctly refuses to join unlike measurements. Figure5 carries the greatest conceptual risk because output bytes, proxy reads, estimated total traffic, a measured proxy ceiling and assumed peak share one scale.

The source's SVG text and captions were inspected, not screenshots in every viewport/theme. Live HTML fetched at17:42:46UTC on2026-11-17, HTTP200, SHA-256 **75a3aa7933701ceedb4d8601937c9f5f130d24c66688dfae94b8dc766b75bb79**; correct article and the affinity sentence present. No independent phone-legibility certification is claimed; PR550 itself reports scaling720-unit figures to about40% on phones. Source/HTML checks are not a rendered visual pass.

## Disposition and next value

Expected: a defensible release result with mechanism uncertainty preserved. Observed: the main measurements reproduce and the prior causal exclusion is repaired; three local statements still distort placement, historical coverage and what IMC can establish. Quantitative gap closed: no unexplained discrepancy in the main release ratios or traffic magnitudes. Mechanistic gap remains intentional: interference is supported, unique bottleneck is not.

Next useful action is an editorial correction of F1–F3 and the minor date/scope points, then a focused source/live check. No new performance run is required for these corrections. A future current-kernel IMC study has value only as a pre-registered bottleneck discriminator with an appropriate sustainable comparison, not as retrospective proof by counter collection alone.

Migration gate passed against old head1283b969e9e295e268257cd1b6be3ad7da29e6dd and manifest digest9b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0. Pinned coord.py audit over new comments since6953475922: no misroutes; REVIEW route is145. Persistent task remains enabled; no automation setting changed.
