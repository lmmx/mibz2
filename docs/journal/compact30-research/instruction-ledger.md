# Canonical cumulative instruction ledger

Baseline B2: frozen v2 `8beffe28bec9b539517b442350d8a3fda62164c8`.
B3: frozen v3 `dbf5139e12ae60b5f5bff40059b88d65ddccea41`.
Unit: one construction of primary `wikidata-20261026.bz2`, 434 blocks,
207,006,515 rows. See [pins and replay](INDEX.md).

ACHIEVED means compiled sites and stated frequencies have been checked **within
the named scope**. It does not mean qualification or a complete whole-constructor
net. CONDITIONAL rows earn no cumulative verified net credit. The checkpoint's
own charter §5 ledger remains authoritative for its gate; any disagreement is a
review finding, never a silent replacement.

## Ranked baseline costs

| Cost center | Instructions / corpus | Scope and evidence |
|---|---:|---|
| Scalar append | 459,538,596 | 12 × 38,294,883; all four B2 loop copies replayed |
| Scanner classification | 221,503,931–222,731,873 | Ultra audit §C, bounded Q/W; per-block bounds independently replayed |
| Fast full finalization | 133,437,375 | 81 × now-exact 1,647,375 eligible finishes; entry calls excluded |
| Nonuniform tile setup/advance | 98,125,833–114,879,024 | 41S+7X; Ultra bounds, no scalar-body double credit |
| Refill wrappers | 69,605,726–111,402,124 | finish body excluded; per-block lower bound independently replayed |
| Fragment filling | 43,436,844–114,277,290 | length-dependent; missing histogram prevents a sharp total |
| Bulk emitter core | 51,273,000–51,883,521 | paired loop/odd tail; run setup excluded |
| RAW reconstruction core | 23,772,928 | 16 × 1,485,808 fields; exceptional detours excluded |

These are disjoint scoped baseline costs, not a closed whole-constructor total.
The audit's revised K is 1,130,193,036–1,261,424,207, plus unresolved E. Its
proven E floor is 25,106,954; it supplies no upper bound. Library allocation,
zeroing/copies, slow paths, canonicalization, padding/PARK and validation cannot
be assigned zero. The large scalar cost is 153,179,532 explicit reads,
76,589,766 writes, including 38,294,883 pointer RMW instructions. RMW is one
instruction, one read and one write, not three instructions.

## Achieved versus conditional savings

| ID / status | Baseline and scope | Instruction saving | Added/removed memory and dependencies | Gate/overlap |
|---|---|---:|---|---|
| A1 ACHIEVED | B3 fast leaf versus B2 fast body only | 60,952,875 gross | 5 fewer explicit reads and 2 fewer explicit writes/leaf; 6 pushes+6 pops removed/leaf; snapshot chain and frame removed | 44 vs 81 instructions. Gross only; leaf has 12 reads, 6 writes. Compiler STOP stands |
| A3 ACHIEVED | Historical B3 §6 whole-path model | **[60,049,330, 60,345,544] verified** | Corrected explicit reads −6,185,349; writes −3,290,662; push/pop −19,768,500 already in instruction credit. Tag loads, slow guards, setup/epoch stores and residual paths charged | [Independent closeout review](reviews/v3/report.md). Includes A1; never add it again. Formal compiler STOP stands; §7 inclusion remains conditional |
| A2 ACHIEVED, scoped | PR #146 same-binary build own body, `7f74e9f` | 56,195,817 | New spills and loop changes included in own-body Ir. Raw Dr/Dw profiles absent; summary-only memory figures remain conditional | Independent per-instruction replay; excludes callees. Does not clear 60M minimum |
| C2 WITHDRAWN as conservative | PR #146 correction `a1d110c79520aff10e4ce6eb5ad8fb5cc3bbe51b` §8 | No whole-constructor bound | F1 accepted; new raw Dr/Dw archives and header-aware parser address F2. Own reads −3,389,475; writes +1,995,539. Library terms remain unresolved | A2 stands; old prototype remains below 60M and unqualified. Historical run 2 remains unexplained |
| C3 CONDITIONAL | Original scanner handoff versus B2 | 87,622,080–87,632,430 at stated cap | Gross 88,622,080+3Q minus ≤1M **acceptance cap**, not observed overhead | Supersedes, not adds to, scanner checkpoint; safe prototype missed its target |
| C4 SUPERSEDED estimate | Hot descriptor arrays versus B2 | 71,589,766 design target | Actual non-row overhead exceeds the 5M cap | Replaced by A4; do not add |
| C5 CONDITIONAL | SSE2 first-chunk refinement versus B3 | 6,589,500 gross; net unknown | Four fewer instructions/leaf target; setup, mask/broadcast, callers and memory count must be recounted | 4M incremental minimum; overlaps A1's replacement body, not additional B2 leaf credit |
| A4 ACHIEVED, scoped; §5.3b REMEDIED | #151 descriptor arrays `7620c0c667018772a0ba3be28eb62263b55037f8`, journal §10 | Own code 58,209,045; counted interval **[57,830,163, 62,277,795]** | Own writes **+5,004,156**; own reads −1,096,093. Charge 378,882 added library Ir at the lower end; removed 4,447,632-Ir memcpy remains upper-end-only | Gate 3 PASS at this pin: all 34 changed classes counter-supported, normalized uncounted closure unchanged. §6.2 and combined qualification remain open. See [counted replay](reviews/checkpoint-6637121066/report.md) |
| A5 ACHIEVED, scoped; §5.3b REMEDIED | #146 U2 `ba261eff245f45e1c07fdfb05fa2fe6ff1ced835`, journal §11 | Counted interval **[84,060,421, 84,060,421]** | Reads −505,885; writes **+3,391,432**. U1 absent from final source; all 11 library sites match | Replaces A2/C3, never adds. Gate 3 PASS at this pin: all 31 changed classes counter-supported, normalized uncounted closure unchanged. §6.2 and combined qualification remain open. See [counted replay](reviews/checkpoint-6637121066/report.md) |
| O1 LOWER-BOUNDED; combined net CONDITIONAL | A4 ∩ A5 on B2 | Demonstrated **O_lower = 2,393,313**; registered §6.3 floor **F = 60M + 40M − O_lower = 97,606,687** | Exact shared removal: B2 build `+0x12b5`, `mov 0x8(%rsp),%r13`, frequency 2,393,313. Same-frequency 3,575/106/79 classes are not booked as identity | [Canonical class map](reviews/checkpoint-6634996672/report.md). O_upper is not established by component minima and does not set F. Any further demonstrated cross-overlap is subtracted. Fresh combined listing and both §6.3 tests still required |
| A6 ACHIEVED, scoped; qualification HOLD | Assembly 1 `888eccfc86d5d771e881b0f87d46a21e2cf1512a`; signed bridge `95977867fdf318b3f66420c383a73d9c48b36740` versus B2 | Own code **258,873,217**; conservative/inclusive interval **[258,494,335, 262,948,911]** | Own reads −20,479,638; writes −8,815,056. Refill locally regresses: reads +2,562,499, writes +5,197,912; unrolled middle detours spill/reload three registers | [Signed-bridge audit](reviews/checkpoint-6640153383/report.md): 83 changed classes; build nonadditivity −56,391,637 exactly reconciled, refill additive. Registered `O=2,393,313`, `F=147,606,687`; both instruction tests pass. D013 HOLD remains; no economic or timing PASS |
| C6 CONDITIONAL | [Direct interior refill](proposals/refill-boundary/proposal.md) | B2 target **20,315,241 − H**; descriptor-relative target **27,909,632 − H_D**, only at specified compiler caps | ≥1,084,913 eligible transitions, all on large blocks; 26,037,912 transient logical store bytes exposed before shifted work | Outside approved sequence; overlaps descriptor refill/helper work. No verified net credit. Future audit here non-independent |
| C7 CONDITIONAL | Full-state v3 tag, Assembly-1 incremental | **6,293,532 − H** at ≤2 added instructions/open; **8,089,014 − H** at ≤1 | Removes the fast leaf's two fullness-test pointer loads; no logical-byte change. Tag-construction cost and all spills charged | 1,647,416 fast finishes; at most 1,795,482 opens. Incremental to A3/v3 only; no A3 credit repeated |
| C8 CONDITIONAL | Natural residual run-open specialization, Assembly-1 incremental | **8,432,068 − H** at a 16-instruction specialized-open cap | Same logical writes; expected generic bitmap/header read and address chain removal remain unbooked pending replay | 496,004 run opens. Event class is disjoint from scalar 3b refills unless a future formulation generalizes 3b; shared events then count once |
| C9 CONDITIONAL | Four-record natural bulk kernel, Assembly-1 incremental | **10,254,600 − H** at an eight-instruction/group average cap | Same four record stores and bytes/group; loop/address/header recurrence only | 5,127,300 complete groups. Disjoint code site from U2, descriptors, v3 and 3b; no memory saving claimed |
| C10 CONDITIONAL, measurement-gated | U2 terminal-tile equal-prefix extension, descriptor-relative | **10E_f − C_s·610,356 − C_b·G_f − H**; no registered net | Captured rows leave scalar pointer-RMW path; exact memory delta and changed 3b eligibility require replay | `0 ≤ E_f ≤ 9,155,340`; add prefix histogram and `G_f` counters first. U2 uniform-tile credit and A4 credit are not repeated |

A6 is the first achieved combined-scope instruction row. Its checkpoint ledger,
not a sum of the isolated rows, is authoritative: the isolated scanner and
descriptor own deltas, the v3 finish group and the signed compiler interaction
reconcile exactly to the combined own delta. The Ultra 225,253,100 portfolio
remains only a historical conditional cap target. A6 is not a final qualification
or an economic result; Guardian D013's HOLD and the rule that instructions are not
cycles remain in force.

Critical-block relevance: the Ultra audit reports 230 large blocks (n≥899,000)
and 204 tiny blocks. All-full steps contribute approximately 98% of archived
candidate completion; slow workers are enriched in scalar rows and full finishes.
These are descriptive observations from an INVALID capture, not an economic
PASS or a causal cycles claim. They justify prioritizing scalar, tile and refill
work over tiny-block initialization and bulk-only refinements.

## Review state and next trigger

[Scanner review](reviews/scanner-146/report.md): safety PASS, diagnostic partial miss,
qualification REJECT; conservative interval HOLD pending bounded callee accounting
and a replayable raw memory/profile package. [V3 closeout review](reviews/v3/report.md):
archived safety PASS and historical instruction interval verified; three LOW reporting
corrections, compiler STOP unchanged. Neither is the final §6.2 review.

The [structural replay](structural-findings.md) verifies the per-block lower bounds
and distinguishes latest absolute end from longest worker duration. A6 now records a verified combined-scope instruction total, while final
qualification remains on D013 HOLD and economics remain unresolved. WAITING for a new pinned checkpoint/correction/rebuttal,
the completed-sequence evidence index, or a user nudge. No polling or automatic
Goal wake-up is assumed.

## Pinned update: comment 6633116288

[Audit and C6 reassessment](reviews/checkpoint-6633116288/report.md). A4/A5 preserve the checkpoint ledgers' numerical gate-3 PASS classifications, but this review does not certify formal §5.3b compliance or §6.2 qualification. No raw compressed profile was independently replayed in this run. Existing standalone scope credits are retained and new scoped A-rows added; no cumulative sum is claimed. C6 remains a proposal, and a later audit by this thread is non-independent. The event-triggered task supersedes the historical no-wake statement above.


## Pinned update: comment 6634996672

[Canonical B2 overlap map](reviews/checkpoint-6634996672/report.md) and its [exact offset data](reviews/checkpoint-6634996672/overlap-map.json) cover all 63 changed `build::<false>` execution-count classes at the original scanner and descriptor pins. **Correction under charter §6.3 and Guardian D010:** the earlier publication used the wrong end of O and inferred an unsupported 40M upper bound from component minima. The registered floor uses only demonstrated overlap: `F = 60,000,000 + 40,000,000 − 2,393,313 = 97,606,687`. O_upper is not established and plays no role in F; any further demonstrated cross-overlap is subtracted. The exact offset map remains valid. The former profile-only counter gap is superseded by the counted-pin replay below.


## Pinned update: comment 6637121066 (re-delivery of 6635380133 and 6635419092)

[Archive-only counted replay](reviews/checkpoint-6637121066/report.md) exactly matches both committed `ledger.json` files across all fields, not only totals. Scanner `ba261eff245f45e1c07fdfb05fa2fe6ff1ced835`: 31/31 changed classes counter-supported, interval `[84,060,421, 84,060,421]`. Descriptor arrays `7620c0c667018772a0ba3be28eb62263b55037f8`: 34/34 classes counter-supported, interval `[57,830,163, 62,277,795]`; the lower end charges 378,882 added library Ir and the 4,447,632-Ir removed memcpy remains upper-end-only. Both retain increased writes and pass even if the external +868 harness caller delta is charged. Use **normalized uncounted instruction closure unchanged**, not whole-binary byte identity. Frequency-class replay does not prove the absence of equal-frequency zero-net/cancelling sites; the pinned source/listing map remains part of the evidence. Gate 3 PASS for these isolated models does not close §6.2, qualify the combined build, authorize integration, or alter D010's `F = 97,606,687` rule.


## Pinned update: comment 6639496192 (re-delivery of 6637300905)

[Ranked next-cut audit](proposals/ranked-next-cuts-6639496192/report.md) registers four proposal-only CONDITIONAL rows, ordered by credible contribution to the completion gap: C7 full-state v3 tag, C8 natural residual run-open specialization, C9 four-record natural bulk kernel, and C10 a measurement-gated terminal-tile run fringe. All targets are incremental to Assembly 1 and contribute zero verified cumulative credit. U2, descriptor arrays, v3, and registered 3b credit are excluded; any changed 3b event frequency must be charged in a fresh combined replay. The scalar 10-instruction descriptor append is locally bounded as irreducible absent a changed formulation and listing.


## Pinned update: comment 6640153383 (D013 reconciliation)

[Assembly 1 signed-bridge audit](reviews/checkpoint-6640153383/report.md) and [machine-readable arithmetic](reviews/checkpoint-6640153383/bridge-audit.json) reproduce the pinned reconciliation at `95977867fdf318b3f66420c383a73d9c48b36740`. The exact own-code identity is scanner `−84,060,421` + descriptor `−58,209,045` + v3 finish group `−60,212,114` + build nonadditivity `−56,391,637` = `−258,873,217`; refill is exactly additive. The nonadditivity partitions into unroll `−66,676,611`, outer head `−12,015,336`, S-class cost `+19,146,504`, and remaining cost `+3,153,806`. Added library work 378,882 and the upper-only removed memcpy 4,454,576 yield `[258,494,335, 262,948,911]`. This bridge explains existing credit and is never added again. Registered O remains 2,393,313; the further 3,654 is not used. Gate-3 arithmetic passes in the checked scope; Guardian D013's qualification HOLD, the raw-profile completeness limitation, and the absence of any economic PASS remain explicit.


## Historical production ranking: inbox 6645643541 (resumes 6645304470)

[Whole-decoder ranking and corrected parity arithmetic](reviews/inbox-6645643541/report.md), baseline **08e7fc3b46c7a6adb61ce0e70b64e9973b95cbb7**. These rows are a separate production scope, not additions to frozen B2 constructor credits.

| Priority / ID | Status | Current whole-decoder model | Evidence boundary |
|---|---|---|---|
| 1 / C11 P87 | CONDITIONAL, zero new verified credit | Modest historical total benefit; current instruction and timing delta unknown | Closed positive diagnostic, engineering NOT DEMONSTRATED; production reuse plus allocation removal needs its own validation |
| 2 / C12 true tiled B | CONDITIONAL, zero credit | Eligible rows × incremental work reduction minus replay/tail/setup/rest | Unbuilt; #157 is not B; old 10–11% model cannot transfer |
| 3 / C13 handoff | CONDITIONAL diagnostic, zero credit | Direct decode_block delta zero; whole-process and idle effects unknown | 34.7% of Q2, ~11% worker time; convoy plausible not proven |
| 4 / C14 gather C | CONDITIONAL, zero credit | Incremental to B, with packing, replay and tail charged | Unbuilt; no additive B+C credit |
| 5 / C15 compact parity | CONDITIONAL target, existing port REJECTED | Block-1 walk ≤10,861,938 Ir, ≤12.0692/row if compact build/rest unchanged | Cross-archive rebase, not new compiled measurement; old 18.26 budget obsolete |
| 6 / C16 Precopy | Historical REJECTED; deferred | No positive four-worker forecast | Recorded reopening at 8–10 workstation workers, not CI |

P87 ranks first for a production-shaped comparison, not automatic promotion. No unique four-worker limiting mechanism is established on either topology. Earlier unroll is precedent, not identical; no detected gain is not zero benefit. Historical A1–A6 accounting and their qualifications are preserved; no current production gain is obtained by summing them.


## Historical production ranking: inbox 6646251725, after P87

[Post-P87 review and next-build proposal](reviews/inbox-6646251725/report.md);
[replayed call-arc arithmetic](reviews/inbox-6646251725/arithmetic.json).
Production baseline **e26e3d9cbac6bcd2106b7070c869dd91fcd7217d**.
This section supersedes the production ranking immediately above, not the historical constructor ledger.

| Priority / ID | Status | Whole-decoder accounting at this baseline | Prior verdict and next falsifier |
|---|---|---|---|
| Baseline / A7 (resolves C11) P87 | ACHIEVED, scoped archive instruction reduction; reported production gate passed and merged | Four full-block decode savings vs 08e7fc3b: **400 / 400 / 432 / 542 Ir**; tiny blocks **277 / 270 / 270**. Each published driver total includes 16 Ir. Build and room walk unchanged. Fresh-Scratch scope only, no recurring corpus credit | Validation a995ce87; raw profiles e26e3d9c. Original/amended criteria both supported by reported results; literal amendment-cutoff exception disclosed. No new mechanism or independent safety certification |
| 1 / C12 true tiled B | CONDITIONAL, zero credit; **next build recommended** | E×(9.5−u)−Q×u_spec−H; proposed u≤8.5 is an assembly target, not achieved saving. All-lane eight-round rollback may discard up to 256 steps per failed tile and erase the margin | Unbuilt, distinct from #157. Preserve offset/lane handover semantics or prove changed scheduling; reject formulation on non-positive whole-decoder net or missed prospective minimum |
| 2 / C13 handoff diagnosis | CONDITIONAL diagnostic, zero credit | Direct decode_block saving zero; current whole-process work and wall benefit unknown | Convoy plausible, not demonstrated. Observe queue/owner/scheduling together before any rewrite; historical 11% worker-time share is not a gain forecast |
| 3 / C14 gather C | CONDITIONAL, zero credit | Compare C−B with packing, masks, spills, replay/tail/rest; no additive B+C credit | Unbuilt; old Zen timing thresholds do not transfer to the new Xeon runners. Reject if incremental registered benefit over B fails |
| 4 / C15 compact parity | CONDITIONAL target; existing port REJECTED | Block-1 walk ceiling **10,861,581 Ir**, **12.0688122/row**, holding c493a45d compact build/rest fixed; requires **9,346,119 Ir (46.25%)** off existing compact walk | Cross-archive budget, not a new compact/P87 result. Need new reconstruction path reaching whole-decoder parity; storage size or constructor credit alone insufficient |
| 5 / C16 Precopy | Historical REJECTED; deferred | Negative four-worker prior; no positive forecast or new credit | Recorded reopening at 8–10 workstation workers or materially changed construction; neither supplied here |

Block 1's normalized production budget is **20,184,340 = 5,429,062 build + 9,859,825 room walk + 4,895,453 rest**.
Rest includes assembly; its 178,330 Ir is not an extra component. Instruction shares are not time shares.
A7 is already in this baseline and never added to C12–C16. Do not multiply fresh worker allocation savings by all corpus blocks.
The smaller-table ceiling changes from 10,861,938 to 10,861,581 only because the normalized production baseline changes; the old compact arm is held fixed, not silently given P87.
The old P87 diagnostic remains closed/INCONCLUSIVE, A1–A6 and D013 remain unchanged, and no unique four-worker limiting mechanism has been established.


## Resource ranking: inbox 6651023083 at c9433318 + assumed #180 window

[Resource review](reviews/inbox-6651023083/report.md) and [static model](reviews/inbox-6651023083/resource-model.json). This supersedes the previous production priority, not historical arithmetic. Instructions are not a wall-time predictor.

| ID | Status and current priority | Whole-decoder accounting / falsifier |
|---|---|---|
| C12/B | Historical proposal superseded by #168/#171 outcomes | Deferred flags alone: no detected benefit; staged combined path merged with 4t gain and unresolved runs 1t control. Do not claim the eight-round B prototype was tested identically |
| C14/C | REJECTED local implementation relative to staged scalar | Lower Ir did not win local wall; not an independent CI qualification |
| C17 fusion | CONDITIONAL, **priority 1** | Ordered assembly→RLE1→CRC: removes 2n+m logical bytes, no random T accesses; actual lower-level traffic saving 0 to a cold-model ceiling. Preserve run/cycle/CRC/error semantics; reject on whole-decoder benefit/control failure |
| C15 compact | CONDITIONAL, priority 2; existing port still REJECTED | Main table compulsory lines 1/16→1/30 per row, NOT steady random misses. Two sibling tables share L2; RAW/build/reconstruction charged. 46.25% old Ir parity remains historical arithmetic, not time parity |
| C18 split condvars | CONDITIONAL, priority 3 | Per-job wake/ownership costs, zero direct random-walk saving; require wrong-predicate wake evidence and no lost-wakeup/cancellation defect |
| C19 byte-budgeted window | CONDITIONAL, priority 4 | Memory/latency objective; speed unknown after assumed window 16. Reserve progress for earliest block; charge capacities and expansion |
| C20 self-scanning | CONDITIONAL, priority 5 | Removes handoff at cost of scanner work on decode cores; old idle shares overlap and do not define the remaining gain |

No new verified cumulative instruction credit is booked. Reported #169/#173 reductions remain economically not shown; merged MTF/staging outcomes are baseline, not additive future credit. #171/#170/#172 unresolved controls remain explicit. From the unpaired 93%-busy observation, 98%/100% busy gives illustrative 5.10%/7% wall reductions only at unchanged useful-work cost; scheduling candidates share this opportunity. Absolute runner latency, actual cache misses and current phase exposure are unmeasured here. Recommended next build is the Coordinator's already-local fused post-walk path, not another Ir-only walk cut.


## Current compact-parity decision: inbox 6653416078

[Post-fusion audit and bounded staged-compact proposal](reviews/inbox-6653416078/report.md), baseline **3c4ed7939d107a7b7b2633ca913f2e640023f15a**. This supersedes the production ranking in inbox 6651023083, not the historical constructor accounting.

| ID | Updated disposition | Resource finding and next falsifier |
|---|---|---|
| C17 fused post-walk | REJECTED for registered four-thread claim (#183); fresh single-thread question separate | Two PR runs fail the four-thread benefit rule; runs 4t harm unresolved; attribution fails on text 2t. Simulated line reduction is not a hardware resource measurement or proof of zero benefit. No cumulative credit |
| C18 split condvars | CLOSED before implementation of rewrite (#185) | Local counts argue against a material wake-driven opportunity at window 16. Wasted-wake CPU estimate is not an SMT wall bound |
| C19 byte budget / C20 self-scan | DEPRIORITIZED as throughput builds | Backpressure weakens scanner-starvation premise; memory/tail objectives remain distinct. No shared slack ceilings added |
| Co-resident 16/L2 | NOT REPRODUCED on current kernel | Seven of eight four-copy CPU cells slower beyond noise. No demonstrated better grid setting; not proof of a universal optimum |
| C15 staged compact parity | CONDITIONAL, bounded next comparison recommended; old #155 port remains REJECTED | Compact changes footprint, not the existence of random references. Ordinary field/header share one line but add load/reconstruction work; RAW adds dependent service. Compare current staged compact direct-build decoder with production and an equal-byte split-header layout control. Include build/walk/rest, four workers on two SMT cores, window 16. Close if whole-decoder benefit rule fails even when misses/bytes fall |

No numerical four-thread gain is forecast and no verified instruction credit is added. Historical 46.25% walk-Ir parity is not a time gate. The report distinguishes net random line counts from latency-weighted service demand, charges RAW and construction, and specifies the equal-size control's attribution limits. The proposal is a changed-walker retest, not a new constructor or authorisation. A1–A6 and D013 remain unchanged.


## Current binding-census decision: inbox 6657883434

[Model-specific ranking and whole-block phase-pairing proposal](reviews/inbox-6657883434/report.md);
[static arithmetic](reviews/inbox-6657883434/model.json).
Evidence **decc71b866a1d55ae86251e6e227b9c11987f357**, reported production source d1fd4405.
This supersedes production priorities above, not historical constructor accounting.

| Item | Disposition | Conditional whole-decoder price / next falsifier |
|---|---|---|
| AMD walk interference | Largest identified resource opportunity, no demonstrated lever | 7763 ~16–18% own-stage pool at unchanged rest/utilisation; 9V74 ~20–22% scenario with explicitly transferred worker share. Neither is an achievable forecast or hard bound. 9V45 unpriced without matched decomposition |
| 8370C / 9V74 build interference | Raised above generic global rank | 8370C ~5–6% own-stage pool; 9V74 ~4% scenario; 7763 ~0–1%. 8573C unpriced. Store-stream versus issue explanation unresolved |
| Huffman / MTF / handoff | Existing isolated workflows; no duplicate build | Huffman 0–4.3% conditional direct pool, MTF 0–3% working envelope with larger unverified ideal scenarios, handoff 0–2% working envelope. No positive lower bound. Scanner-starvation and handoff overlap supply |
| C21 whole-block phase pairing | CONDITIONAL design-only, zero credit | Four end-to-end workers, private scratch, bounded ready-job selection. Pair fixed-work makespans and job diversity must support benefit after coordination and ordered-output delay. No token, stage handoff or software-interleaved walker |
| C15 staged compact | Existing regime discriminator; no duplicate port | Pinned models mostly predict harm; no new numerical gain or qualification. Build/rest and RAW costs remain charged |
| C17 fusion / C18 condvars | CLOSED under existing decisions | Heterogeneous small effects do not reopen the registered fusion gate; receiver mutex is distinct from condvars |
| C19 byte budget / C20 self-scan | Throughput-deprioritized | Shared measured steady supply scenario ~1.2–1.8%; fixed bench startup/tail not recurring release gains |

The four-copy factors are uncontrolled-phase fastest-pass proxies, not exact parallel-decoder shares or W/W pairs. Inferred MLP and a fitted lane-independent term do not settle issue versus miss slots. Placement changes both siblings and is not additional credit above the same walk/build pool. A1–A6, A7 and D013 remain unchanged; no new verified instruction or wall-time credit.


## Tail-front update: inbox 6659992544

[Report and scope correction](reviews/inbox-6659992544/report.md), [model](reviews/inbox-6659992544/model.json).
Evidence **1132857e18819ce689424dd2df7751630b29522a**.

| Item | Disposition | Accounting / falsifier |
|---|---|---|
| C15 staged C-resolve | REJECTED, #195 rule failed | Archived 4t wall +4.8% and +7.7%; RSS does not carry harm. Amplification-product identity is not independent cache causality evidence |
| C21 block phase pairing | HELD for pair matrix M/P | No new credit; C outcome closes partner-selection mechanism |
| Tail concurrency | CONDITIONAL resource pool | Independent exponential-residual model: ~0.25–0.27 c/B alone, ~0.21–0.25 at four copies on fitted 9V45/9V74. Live-lane histogram and CI tail clock required; not transferable measured bounds |
| C22 length-aware segment launch with reusable bounded scouting | CONDITIONAL, zero credit, low priority | Same starts/count/table; predictable remaining length required. Memoryless residuals falsify cheap scouting's benefit. Charge resume metadata/cache/copy effects; no free oracle |
| Cross-chain lockstep | CLOSED | Lean path misses its 10-cycle gate; inferred cycle decomposition not independently validated by fitting CI net cost |

Tail gains are not confined to 1t/2t. Current placement/window-light variants leave those rows unchanged by policy. Teams require a slowest-thread/barrier model, not summed per-thread tail credit. No implementation, new timing, new safety certification or historical-credit change.


## Live-byte/team-count decision: inbox 6683037641

[Access audit and ranked proposals](reviews/inbox-6683037641/report.md), [archived arithmetic](reviews/inbox-6683037641/arithmetic.json), [sensitivity model](reviews/inbox-6683037641/model.json).
Source **f4f99ab82652411feb4a5a6e2ccb96d04902fb58**; results **cbc6eb14b065fdb7092911147ce3374f63541467**; three-table registration **656d2794cf0146e65af9223b8a10c31297518f35**.

| Item | Disposition | Whole-decoder accounting / falsifier |
|---|---|---|
| #219 four-table primaries | FAIL registered benefit | Replayed 320 rows: 4×3 1.016955; 4×4 1.003463 and CPU 1.52452. No counter evidence |
| #220 three-table policy | CONDITIONAL, existing first priority | Screen 3×4 1.096728 speed ratio (8.82% wall reduction), exploratory only. Same-cell fresh confirmation required; k/window/placement confounds retained |
| C23 cooperative ll reuse | CONDITIONAL, preferred new bounded proposal | Remove separate rle allocation (~1.35 MB/team endpoint), not T loads or assembly/RLE payload work. Extend in-place lifetime safely after build; fixed-plan net benefit required for speed claim |
| C24 reusable segment page arena | HELD for counted growth/page census | Reserve at n900000,k2 is 1,864,704 B before growth; observed6.1 MB is historical capacity, not touched-prefix demand. Charge page chains, handoffs and copy costs; no lower traffic/pages means no throughput premise |
| Placement | Existing lead, no duplicate experiment | Mask intervention not thread assignment; unpinned release gain unpriced |
| C15/C17/C18 | Prior rejection/closure unchanged | New cache hypothesis does not reopen failed registered claims |

T footprint≤4n, ll dead after build, rle untouched during cooperative traversal. L3 is20MiB; C/W using13.9MB allocation per team is invalid. At fixed shared load, four teams need25% effective footprint reduction to match three, not25% less allocated capacity. No calibrated speedup per MB; all new proposal credit zero. A1–A7, D013 and prior tail/phase-pairing qualifications unchanged.
