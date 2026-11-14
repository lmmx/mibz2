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


## Three-team policy design: inbox6693507432

[Policy and prospective evidence plan](records/reviews/inbox-6693507432/report.md), evidence#220@544462881497fc957fe17c35ae7508ca1bac23d5, governing spec@a6a703cefcc40ac4115803f44a8a29ad76a3a7ef.

- **Priority1: narrow3×3 policy candidate.** On10 physical cores /20MiB reportedL3, requests>=10 choose3×3; others retain old policy, including4×2 at8/9. Higher requests/same-topology other CPUs/corpora are explicit extrapolations. No global0.5L3 rule. Separate k4 remains experimental.
- **Evidence:** journal records both frozen primaries qualified twice. Preferred3×3 speed1.076/1.087, CPU0.90/0.89, RSS0.82. Head-to-head k4 superiority not demonstrated; rounded ratios1.013/0.999 for~16% extraCPU. Confirmation raw bundles absent at inspected tree; no independent replay claimed.
- **Admission unresolved:** forced-plan correctness tests cover changed code on small CI; unchanged1/2/nproc timings cannot establish changed-branch benefit. C6/D4 decision needed for representative CI coverage/admission; no relaxation made here. New workstation recipe needs C5 authorisation.
- **C23/C24 lower priority:** confirmed policy economics outrank unpriced buffer changes. C15/C17 prior failures unchanged; no new verified cumulative credit.
- **Correction:**25% footprint equality is sufficient to match aggregate footprints under stated model, not a necessary speed/fit threshold. Researcher6683945162 counterexample and Coordinator6683965024 retraction accepted; no fixed threshold retained.

Only a design was published. The role cannot create the requested production PR under SPEC.OPTIMIZER. Existing evidence PR#220 is not a policy implementation link.


## Pipeline critical-path ranking: inbox 6715370785, resumed by 6729116363

[Source audit, arithmetic and ranked constructions](records/reviews/inbox-6715370785/report.md).
Source/spec **a4f97714cc14366984015869d9d094d89488ac57**; relevant production blobs unchanged from original assignment e01db6a2895711eb13897fdc378816075d927c1e.

| Item | Disposition | Critical-path price / next falsifier |
|---|---|---|
| Existing #258 service-2x3 | First existing priority; frozen comparison untouched | #251's exploratory svc2@10 speed 1.045, CPU reference/arm 1.138; not qualification. One service thread saturated; svc2@12 trade not selected. Complete Researcher review/registered comparison |
| C25 selected-boundary validation | First new bounded diagnostic, zero measured credit | Two helper shares consume one interior checkpoint; pre-dispatch setup validates 55 vectors at n=900000. Distinct selected-boundary interface can remove 53 vectors' validation, retaining disjoint-write/end checks. Count reduction is not an instruction/time verdict |
| C27 chunked RLE1+CRC | Second new construction, conditional | Preserve CLMUL folding and current assembled buffers; whole output reread is logical traffic, not DRAM. Transferred entire-CRC scale 3.45% wall is not removable cost or forecast. Fragmentation may regress |
| C26 bounded leader admission | Census before cap | Default both-arm has 22.4 ms summed join delay, not small-only 26 ms. 198 small blocks carry only 8895 BWT rows by census reconciliation. Equal-rate/balanced toy live-walk opportunity 1.75% before lost priming; not measured bound |
| C25b stride/emission | Follow C25 only if worthwhile | CHECKPOINT also defines MIN_COOP; freeze threshold to avoid policy confound. Actual n unknown until MTF ends; sparse emission cannot assume max_len=n |
| C28 segment CRC/direct fusion | Held for boundary and ownership cost | CRC needs post-RLE bytes in output order. Direct segment fusion needs second scratch or lost overlap; last-walker assembly is not always leader work |
| Run-fed T-vector / Ψ² | #266 verdict retained; separate Coordinator diagnostic acknowledged | No duplicate implementation. Model-only outcomes stay model-only; whole MTF+sort+build and SIMD-comparable emitter required |

Replayed 240 archived release rows: at10 threads 0.2.6/0.2.5 speed1.03730 [0.98401,1.09349], CPU multiplier1.09415. Separate release evidence does not rewrite the levers' registered verdict. Binary service/levers bundles were not replayed; text-reader limitation recorded in report. No new timing, implementation, safety certification, admission or cumulative speed credit. Historical C17 and all previous registered outcomes retained.


## Run-fed critical-path constructions: inbox 6729426658

[Full source audit and ranked proposals](records/reviews/inbox-6729426658/report.md).
Source **187c0ba517c3324aa28c5e7ccd2667f6c0572145** (#269); specs **a4f97714cc14366984015869d9d094d89488ac57**.

| Rank | Proposal | Net-work account and falsifier |
|---|---|---|
| 1 / C29 | Matched modern-list checkpointed MTF control | Emitter/checkpoint comparison also changes list kernel. Isolate before crediting checkpoint removal; source/listing counts then registered matched phase comparison |
| 2 / C30 | Helper parallel sort and exact bounded row fill | Leader release M-E-A; charge two histograms (record count AND row length), prefix, scatter, barriers, fill and changed walk joining. Row-balanced fill; no cross-share overstores |
| 3 / C31 | Direct F-interval placement, no sorted records | Length prefixes alone give each run's a; intersect its F interval with original L runs for the low byte. Removes 8(r+1)-byte F array; adds r predecessor searches, bucket streams and ownership costs. Count these before implementation |
| 4 / C32 | Bounded tiled/multi-cell fill | Current wide stores write W=sum 8ceil(ell/8), n<=W<=n+7m. Logical 4n is not actual store count. Exact W census before mask/tile work |
| 5 / C33 | Streaming full-line stores / field tiling | Charge immediate walk cache misses, publication fence and tails. Global two-pass table costs about12n logical read/write bytes, not4n. ArmD is a probe, not universal lower bound |
| 6 / C34 | Lazy/hybrid interiors | Need touched-line census and dependent lookup/marker budget; smaller footprint cannot inherit a speed verdict from failed move walks |

Model T=U+max(L,P(L))+D: helper completion depends on leader arrival. Local E=.75M gives only a hypothetical .25M gross leader release; no25% wall claim. C31 replaces F sorting with stable per-helper byte-length prefixes; it preserves T[q]=L[q]|Psi(q)<<8 and the existing walk. Derivation only, no new constructor.

**Scope correction:** #269 Q1 tests one worker; Q2 includes leader-side sort. Their negative results cannot close new helper-side formulations. Preserve all actual registered outcomes. Current floor needs compiled-store/observability inspection; nominal-GHz elapsed cycles are not PMU cycles.

**Whole-decoder priority:** #258 is now explicitly HELD for admission (Researcher6729358962, Coordinator6729367020); resolve that evidence issue before a run. C25 remains a cheaper narrow alternative. D042's separate B5 HOLD is untouched. Zero new timing, implementation, safety certification, speed credit or automation changes.


## Helper-side run-fed ranking: inbox 6729932062

[Source review and prices](records/reviews/inbox-6729932062/report.md), #270 at **f94551ab8c359ec28dca02956ddca150595cc7cb**.

- **C31 first helper candidate:** per-byte L cursors reduce predecessor searches to at most256/helpers' share, not r; previous r-search price superseded for this implementation. Eliminated scatters are F-order metadata versus hs. FusedBuild already uses bucket streams and16-row identical-byte runs. No45k production table stores eliminated.
- **C35 row-aware partitioning next:** current equal-record split can imbalance row/cell work. Price max helper span, not summed timers; census r/n/m/W per helper before changing the partition.
- **C32 exact tails:** W/n1.59 implies37.1% fewer written words if exact, not37.1% fewer store instructions. Masked tails generally retain vector-store count. Compare instructions, words and readiness.
- **C36 optional leader build assistance:** ideal divisible equal-rate saving (B−L)/3 for two helpers finishing at B and leader arriving at L<B. Do not assign prefix or an unbounded byte class exclusively to the leader.
- **Early walk deferred:** arbitrary Psi targets, table initialization, markers and overstores require new readiness/ownership machinery. Keep current global barrier.
- **Counter correction:** build_ns starts before setup/dispatch; lead_ns starts inside pool entry. leader_late compares unlike origins and can undercount actual late completion. join_wait includes scheduling; CI wall/CPU are not pure work or i9 critical-path evidence.
- **C29 independent narrow lead:** register-list checkpoint kernel can remove front work without new representation; plain workers already have register-list MTF. Selected-block smoke numbers are local indications only.
- **Current economics:** service-2x3 now has submitted run6729877078 and reported result6729896575; earlier admission HOLD description is historical. Reported CPU ref/arm1.146 implies12.7% reduction, not14.6%. No bundle replay here. D044's B5 timing HOLD remains; no unsafe diagnostic authorization.

No implementation, timing, tests, dispatch or measured speed credit. Prior registered outcomes unchanged.


## Pair consumer constructions: inbox 6730271993

[Source audit, cost model and falsifiers](records/reviews/inbox-6730271993/report.md), #272 at **280643cef83b09ac89707b747670e8198b21b68e**.

| Rank | Proposal / disposition | Price and falsifier |
|---|---|---|
| 1 / C37 | Shared-offset staged pair rooms, conditional | Already32 lanes/flags/prefetch. Remove per-lane pointer/capacity work; 256-byte tile is128 pair rounds. Charge flushes, odd LF handoffs and tails; native common-path/counted-work falsifier first |
| 2 / C38 | Flag-compatible symbol tags, conditional | Bit30 is PAIR_LF: only2 immediately spare bits. Two-bit tag +3,516B directory; at most63 exceptional256-row blocks. Three bits need marker-kind redesign. Count actual segmented exceptions |
| 3 | Pair latency/MLP hypothesis | Same32 lanes do not give64 independent misses. Half the navigation cost only when latency/occupancy remain comparable and that term binds; issue cost may erase it |
| 4 / C39 | Wide full-pair word, lower priority | Eliminates symbol lookup, but38 bits needed;8-byte words double resident table. Logical4B/output does not establish physical traffic |
| 5 / C40 | Pair/one-hop trajectory hybrid, census first | Long cells are not consecutive trajectories. Psi²-only word cannot supply one-hop fallback; charge mixed semantics/metadata/switching |

**Corrections at this pin:** first_byte loads first[] then dependent f[c+1] even with zero corrections; .002 loop iterations do not bound the ordinary-path prize. Extra6.2Ir/row cannot by itself reject the archive's plus1c/B condition or imply IPC>6. Counts establish neither MET nor NOT MET. Q1/Q2 are expectations in the draft, not received results.

Whole-entry requires walk saving to exceed extra build/setup/assembly; whole-decoder further charges MTF, admission, synchronization and critical-worker effects. No new measured c/B or cumulative credit. C29/C31 remain preferable overall bets; D045 now holds service timing/admission pending targetedB5, separate from D042/D044. Prior registered results unchanged.

**Publication update:** Guardian D046 / REVIEW6730359769 stops #272's exposed pair path and its proposed CI timing at this exact pin. Safe-entry/fallback/prepared-state repairs and targetedB5 evidence are required. C37–C40 remain research proposals; timing suggestions are conditional on closure. No independent reproduction of its witnesses in this report.


## Measured run-fed losses: inbox6730769410

[Source audit and ranked constructions](records/reviews/inbox-6730769410/report.md). Stage fd60fffada25d475bbcc2b54165b96cf5fd7a92f; result66edecbe2b1924594e485e1f956dbee8ed9beda2; helperff6c2f74677af80ea9813bd081d34a68d07e6f1b.

- Registered Q1 STOP retained: journal R_stage1.1592 [1.1387,1.1801]; no raw artifact replay here.
- C31 first construction, not a promised win: ideal sort deletion leaves+.218 nominal phase-sum c/B; also deleting fallback gap leaves+.118. Sorted fill1.332 is not direct-fill timing. Separate medians/denominators prevent paired counterfactual claims.
- Corpus C35 lookup census163/block over186 team blocks, not one per record; actual one-worker384-block cost unmeasured. Charge resumed cursor work and256 write streams.
- Fallback correction: selector lies outside the phase bracket; A/B fallback bodies match. No wasted emitter on ordinary rejected paths. Buffer history/native layout need isolation before assigning+.100 to selector/dispatch.
- Emitter:4r≈.248n stored bytes on accepted rows, not4n. Keep cheap u32 indexable records pending instruction/spill evidence; literal-stream/extension hybrid must pay reconstruction. Three bytes cannot hold20-bit start+8-bit byte.
- C41 admission: actual n/literals/run extensions/length bins, acquired in existing symbol production if economical. Current syms/max_len predicate is notr/n. Freeze model prospectively; no threshold fitting passed off as qualification.
- #270 helper critical max differs from serial stage sum. D049 timing HOLD remains. R_floor/R_warm do not uniquely identify the binding resource. No implementation, timing, safety certification or measured speed credit.


## Pair measured STOP sharpening: inbox6731656628

[Raw replay and ranked constructions](records/reviews/inbox-6731656628/report.md), run42556998342 at1c4d0f0714183f83798a0b746463ae34535fbc50. Artifact digest verified; all five CSVs replayed.

- Q1.686538 [.684484,.688598] INCONCLUSIVE; Q2 1.894994 [1.885046,1.904994] NEGATIVE STOP retained. M1 extra3.457402 [3.451597,3.463207] nominal c/paired row.
- Paired cross-sweep residuals: R-run6.07213 [6.02397,6.12029], P-one3.98378 [3.96314,4.00443]; gap2.08834 [2.03103,2.14566] nominal c/output byte. NOT consumer-only measurements: differing admission/block sets/setup/cache state. Pre-built walk interval absent.
- C37 then C38: proxy34.4% residual reduction needed to match room; even that does not repay current build premium. C38 retains high-symbol directory lookup. No compiled-loop budget verified.
- C31 one-hop direct-F remains first: remove pair composition/directory/fi/js and L scan only with emitted records; keep checked counts/prefix/cursor/cell/store costs. Compare #269 budget separately.
- Admission correction: one byte run/ONE2.27402, pattern1.00913; .343/.239 were versus ROW pair builder. No aggregate corpus demonstrates run-pair build beating production ONE. Per-block gain cannot be inferred from corpus totals; no threshold or Wikidata recovery claimed.
- No implementation, timing, safety certification or new speed credit. D052 registration closure recorded; new consumer requires own evidence. #270 restrictions not lifted.


## Run-fed leader attribution: inbox6732659575

[Raw replay, source audit and ranked registrations](records/reviews/inbox-6732659575/report.md). Measured03e52206ac248737bc856f064928adf238265917; production/resultd1260c59047846401ed0e717ba6c66d33b699f70.

- Artifact13650328398 digest verified; pinned reporter reproduces VALID attributed LEAD: dir/default33 .966859 [.959421,.974355]. Existing verdict unchanged.
- C29 first: reg/ctl .973935 at9 and .958780 at3. This replaces list kernel AND short-run stores; not list-only attribution. Record arms still use old checkpoints on rejected blocks. Pure records/leader-release causal shares unidentified.
- LS primary next only in a NEW matched-C29 registration. ls/reg .974647 [.969726,.979593] at9, but1.015168 [1.001754,1.028762] at3. At3 its run-span saving10.874ms is outweighed by18.239ms extra drained-front work; whole+6.980ms. Aggregate helper clocks cannot price target span.
- DIR helper_sort .754ms is summed count/prefix/gate waiting, not F sorting. It skips scatter.163 lookups/block do not price that entire term.
- Standard bench default1/2/4 never enters a pipelined-only C29 replacement. Expected direct gain0; no canonical grid change. Targeted diagnostic and target-machine evidence required; neutral unexposed CI is not changed-path qualification.
- C42 proposed: choose leader sort for in-run successor preparation, direct fill for drained fronts, charging transition costs and carrying checked path identity. Not implemented; not added to existing frozen transfer.
- Existing workstation transfer keeps DIR primary, LS secondary. No percentage extrapolation9-on-4 to i9; freeze10-thread admission and retain C5 authorization. C29 B5 covers checkpoint boundaries, raw short-run stores and shipped worker integration. D055 still holds#277; D056/D057 scoped dispositions recorded.

No implementation, compilation, timing, merge, safety certification or automation change. Proposal thresholds are prospective diagnostics, not new production gates.


## Packet census and pair-step budgets: inbox6733004424

[Report and break-even model](records/reviews/inbox-6733004424/report.md), census **292d65b9128b8f8d3d941b755885ab2349ebf8a3**.

- **Priority:** C29 production evidence first, existing matched LS/direct-F work next; C37 then conditional C38 are secondary prebuilt-consumer diagnostics.
- Each stand-alone consumer must remove **4.17669 nominal residual-equivalent cycles/ordinary pair step** (2.08834/output byte;34.3923%) to match room. This is a cross-sweep planning proxy, not a consumer interval. Combined savings must be measured together; construction premium still unpaid.
- C37 charges flush/128 only on full256-byte lane tiles, plus odd ends, occupancy, compaction and spills. C38 retains high-directory load and exact fallback; actual exceptional visits, not63 static blocks, price its benefit.
- Certified k≤8 packets save314 loads/209697662 rows under the stated model; close that bounded Wikidata packet lead without asserting a universal consumer-cycle impossibility.
- **Tandem correction:**6.1%/10.01% count union coverage including seed bytes, before RLE1. They do not bound general nonlocal Precopy; historical#36 copied31.5% and still lost18%/6% wall at1/4threads.
- Ideal uniform-cost, zero-copy-cost detector allowances per all rows: room.2430/.3988; pair.3704/.6078 nominal c/row for periods≤8/≤64. Real cost must include source availability, resume certification, seeds, copies and metadata. Tandem-only copying cannot close34.4% under this model.
- Precopy outside implementation queue. Reopen only with a materially cheaper detector/certifier and priced ready-source mechanism; no repeated all-step neighbour experiment.
- D058 conditional#277 disposition supersedes historical D055 subject to frozen registration, greenCI and completed targeted scalar/AVX2Miri. No execution clearance added here.

Source/count review and arithmetic only; census raw gzip and historical Precopy runs not independently replayed. No implementation, compilation, timing, safety certification, acceptance change or new measured credit.

Publication-time channel update: Coordinator LOG6733033907 reports that D058's conditions are met and the unchanged #277 registered sweep42567752634 has been dispatched at aa409f6f2ebce9f41dc7902789f0ab8d6958b020. This is a reported dispatch, not a result or an independent CI/Miri verification by this review. The frozen comparison remains in flight; no duplicate run is proposed.


## Direct-placement STOP and runner contrast: inbox6733116723

[Report and bounded diagnostic](records/reviews/inbox-6733116723/report.md). Measured aa409f6f2ebce9f41dc7902789f0ab8d6958b020; result c41ef38d7df2f596193c01d9adbb2f4893f1d217.

- Artifact13651923399 and old13645275344 SHA256 verified; raw pinned-copy arithmetic replayed. D/A1.052451, D/B1.078033; registered STOP and screens retained. Full corpus admission not independently rerun.
- **Ranking:** C29 first; crossed common-binary host/build diagnostic next; preserve existing frozen transfer; C43 compact run-index order conditional. No B promotion to production lead.
- Measured old/new heads have identical whole mtf.rs and bwt.rs blobs and B's runtime prefix. Both declared master baselines also have identical mtf/bwt sources. Compiler version matches. Host, harness/layout/history remain confounded; no demonstrated CPU-family cause.
- B/A.976270 on Wikidata hides random1.28423, records1.37493, runs1.38307 and Wikidata fallback1.02107. No all-corpus qualification.
- D's compound placement exceeds sort+fill by.270698 nominal c/output B:5.46398/run or4.47799/cell. Seeds alone would need2927.86/seed; equal slots do not identify locality vs dependencies/control. D has38456 more source-level table-store operations.
- **C43 proposal:** stable u32 indices per byte, two grouping passes then global sequential fill;4r rather than8r ordering scratch. Charge histogram/validation/scatter and dependent recs[j],recs[j+1] gathers. G+incremental_fill+other<.318992/output B (6.43878/run) to beat B's1.328894 sort+fill. Not implemented.
- Diagnostic A_old/A_new/B at exact pins on both named families, same corpus and frozen binaries. A kernel aliases are a negative control; full-revision comparison requires same B control in both builds. ubuntu-latest does not guarantee CPU model. Missing host stratum remains missing; no unbounded recruitment, infrastructure or C6 change.
- Observed front and fill/build ratios both improve across archival runs; sort/build does not. Report seconds and within-host ratios; nominal GHz is not a comparable PMU-cycle measurement.

No new measured credit, implementation, compilation, timing, safety certification or acceptance change.


## Fallback layout control: inbox6734039567

[Report](records/reviews/inbox-6734039567/report.md), source pin adb16fb75cb7e25eb734aeebf4cc9e7bf28ff8c5.

- Layout is a live confound, not established 4KiB causality. Local shared kernels/equal simulated counts do not establish target-machine geometry or hardware behaviour.
- Primary Zen4 measurements report false dependencies at1KiB spacing; A/D being clear modulo4KiB does not make them clean. Recompute actual access exposure modulo1KiB/4KiB. Fitted50 nominal c/literal is not measured event latency.
- Shared callee does not equalise caller stack depth. Require common call-site diagnostic, unchanged-entry reference, reciprocal offset/backing-set controls, independent arm/set/order schedule and A/A′. A penalty following a set also fits physical placement.
- Archive replay: B/A.976269848; equal-fallback counterfactual.970909695, difference.005360153 of A total. This is not causal correction. All2.373% lead remains potentially layout-confounded; observed fallback excess itself hurts B.
- C29 first; corrected layout/host crossing before C43. C43 .318992c/outputB or6.43878/run allowance remains conditional on matched layout. D STOP unchanged.
- Heap counts/preallocation/logged geometry recommended as proposals; align4096 is no cure and does not align Vec backing storage. Standalone counts padding about3KiB/set; extra call costs scale with fallback blocks, diagnostic replication with arm/offset/process factors. C5/C6 authority unchanged.

No implementation, compilation, timing, safety certification, admission change or new measured credit.


## C29 transfer and default-on evidence: inbox6735168430

[Report](records/reviews/inbox-6735168430/report.md); reviewed1197bbcced86eae74c6d99eb8096b5c816a834d7.

- C29 remains first for investigation; run42582708226 remains terminal VALID/UNJUDGED. Reported .9700 not replayed here and never rejudged.
- Explicit nine-thread forced3x3 is not public default reach: natural special case needs10 physical cores,20MiB L3,requestedthreads>=10. Prove all three constructors and actual kernel selection untimed, then target default performance.
- D060: switched held-prior/current/next and partial-front unwind/reuse, exact constructor reach, current-source independent B5; unchanged evidence reusable only with scope reconciliation.
- Beyond judged LEAD: minimum two performance packages, target C5 actual-default comparison plus normal CI release/off-path bench; layout diagnostic integrated or additional if material. From today add one separately frozen repaired CI attempt (80 decodes), never retry-to-pass.
- Cross semantics×build/layout, explicit/default entry, target environment; record data geometry and unchanged-path controls. Stable src and mixed flags do not prove harmless layout. No best-padding selection or changed estimand/grid.
- Critical-path model f(1-r)=.03; aggregate front CPU is not exposed fraction. Hypothetical3% exposed gain vs1.1% off-path loss needs26.83% baseline-time exposed share merely to break even; not a gate or observed deployment estimate.
- C43 below C29 transfer; existing B layout inquiry stays diagnostic. No new measured credit or default-on clearance.

## Service core adoption versus J4: inbox6739270804

[Report and run budget](records/reviews/inbox-6739270804/report.md). Brief baseline a773853f05fa9e12bb65fb93bf5c07e1bc7fea11; r2 separately reviewed 5738616e58542cc7af2d6e5eeb906a4070abcb32.

- #271 remains VALID/INCONCLUSIVE. J4 is window-only at threshold4; joint J vs a2w8 is1.027 [.993,1.063], reported only. .979×1.049≈1.027; no interval multiplication or qualification.
- Trace-label correction removes the claimed demonstrated prep→finish bottleneck transition. #293 remains the mechanism dependency; backlog and thread-life-minus-idle are not CPU/critical-path proof.
- Prefer guarded current-default service qualification to #299's third old-runtime replication. #299 remains draft,120 decodes plus6 trace rounds; counters change measurement conditions. D064 full-data/no clock-conditioned verdict preserved.
- Narrow proposed exposure: exact budget10 and explicitly bounded target predicate, L2/window8; no inherited≥10 or all-machine service claim. Master has no Service dispatch. Prove public constructors, window scaling, guard/M12 and whole-thread/resource accounting.
- D057 resolved D045/D053 within its stated old-runtime scope. Reconcile/reuse evidence on exact candidate; no old performance transfer or fresh safety certification here.
- Minimum adoption cost: ≥1 target C5 campaign plus≥1 ordinary CI package, after untimed safety/reach/memory checks. No guarantee of passing unchanged CI acceptance. Extra exposed budgets/hosts need their own approved coverage.
- J tuning optional: bare40 decodes; proposed A/A′+joint60, withJ4 80, fullfactorial100, plusdefault120 at20rounds. Cell/family/repeat/workload changes remain C6 proposals.
- Ranking: service first for adoption closure; C29 first bounded kernel investigation; J4 after diagnostic; C43 behind layout/host. C29 UNJUDGED and D060 unchanged; gains not additive.
- No raw-bundle replay, timing, implementation, admission change or measured credit.

## Two-mode engineering: inbox6756316971

[Report](records/reviews/inbox-6756316971/report.md). Specs a2da49638ac54d7763de1c167bd054f67ffdc763; baseline3e1a7ec361ee6c8ea6b540264aa69189d4823d9e; #313 ce6954e65f285953b0cb8dd18f2390b4dc2cc91b; #316 b63f6bee2e3ceb821198129d649bed2bd2fb349e.

- Current C6 delegates prospective design/acceptance/adoption with review; prior reserved-C6 statements are historical. C5 and frozen-outcome/no-retry protections remain.
- H1 favours amortised setup/work-sensitive activation; H2 placement if actual packing established; H3 useful-work reduction without promised duty repair; H4 bounded reuse/footprint; H5 work/traffic reduction with concurrency tradeoff. No measured numeric gain assigned.
- Endpoints do not exclude in-run events; every arm having both modes does not exclude arm×state effects. No universal SMT2× bound. Spin probe is an intervention/progress meter; CPU seconds are not energy or guaranteed proportional throughput.
- Buildable proposal A: pool waits2000/0/64, walk policy fixed; coop.rs/parallel.rs/wait_audit/ab, lost-wakeup and whole-decoder falsifiers. Not another bimodality spin explanation. B: readiness-bounded activation, deferred behind A; no compressed-size proxy or duplicate credit for existing lazy pools.
- Service first for adoption: #302 historical6.37% wall/14.75% CPU central reductions remain scoped; #316 fresh route still needs diagnostic, freeze and D077 protocol. R600 shardedCI is a design candidate, not qualification.
- Dir attributedCI LEAD preserved; both failed-fidelity target attempts and closed sequence preserved. Off immediate adoption queue; any future registration needs new diagnostic-supported information, explicit reconciliation and route coverage. No gate-renamed third attempt or additive service/dir gain.
- Rounds require paired-log variance and dependence model; illustrative19 versus460 independent pairs is sensitivity, not a registration.
- Raw archive not replayed; no implementation, timing, compilation, safety certification or measured credit.


## Pool waits: inbox6757179363

[Report](records/reviews/inbox-6757179363/report.md), [archive replay](records/reviews/inbox-6757179363/replay.json).
#321 pin02cc5f89988abb9ab436eae1af4b89576c19d590; artifact13728908238 SHA-256 verified.

- Independently replayed1512 timed rows and18 count rows; complete grid, consistent output/hash. Both registered screen passes retained; no target qualification.
- Four logical CPUs are TWO physical cores; forced1×3 at requested1/2/4. “Uncrowded4t” and ordinary-default transfer not established. Counts omit timed affinity and use instrumentation.
- pool0 preferred candidate, not default. 64's counted spins all precede parking and do not materially reduce1×3 caller parks; paired0/64 intervals do not prove general dominance.
- 1×3 pool spin scale .02437s versus observed mean CPU saving .02814s is consistency, not causal allocation. Idle wall calibration, unpark/park call semantics and changed walk events prevent exact pricing.
- New proposal: helper0/join2000 against both2000/both0, retaining roughly72% of baseline pool spin removal while testing exposed join latency. Charge helper wake latency too. No adaptive budget search.
- i9 diagnostic after two-modes review: forced1×3, forced3×3, actual public10, Wikidata/text;14R decodes including old/master2000 versus candidate2000 code/layout control before removing proven redundant contexts. Qualification separately fresh: targetpublic10 and fallback9/11 on both corpora; N2/K4/M4,12R. Prospective Researcher/Guardian review and C5 apply; no timing authorised here.
- Service first adoption; emitter higher useful-work opportunity; pool split bounded next wait experiment; activation only after ready-work/lifetime census. No bundled gains or unchanged old service evidence transfer.
- No implementation, compilation, new timing, safety certification, acceptance change or measured target credit.

D080 arrived before publication: HOLD measurement-only merge classification; production budget field/loads change code/layout even at2000. Coordinator6757250382 accepted. Research continues, no default-neutral merge exception. Added old/master2000 control to public10 diagnostic; candidate0 must beat actual master directly.


## Small tables after E: inbox6757808815
[Report](records/reviews/inbox-6757808815/report.md), [replay](records/reviews/inbox-6757808815/replay.json).
Source84bf177bd0af9fa416964eca2c0f56ad278b119e. Artifact13731585881 digest verified; five head stage rows independently replayed. E FALSIFIED3.665[3.591,3.740] preserved.
- Step22–26Ir plus≈22 bookkeeping; deleting bookkeeping alone still misses15Ir. Build time/Ir≈2.21× scatter; serial distinct-symbol chain plausible, not measured attribution.
- E2.039B/row=1.835MB/full block before bitmap/scratch: misses1.2MB.
- Fresh Pathfinder6757876056 withdraws C; Coordinator6757895217 permits considering a counting probe only. Replaces earlier unpublished D2-first ranking.
- Rank D4-delta first FOR COUNTING: encode (base−first) mod2^20 with four output bytes and stop length in one u64. At r4/n=.1088, explicit20B/64-row directory gives1.06461MB/900k; conservative3072 singleton records add24,576B. Capacities, buffers and doubling peak remain charged. New proof obligations stated; no implementation.
- Paper walk12 plus doubling5.91≈17.91Ir/row versus15.3 production, before omitted build terms. Count inclusive walk≤15 and construction≤7.35 prospectively; singleton, tail, validation and directory work included. No speed qualification from these filters.
- D2 conditional backup only if actual shared skeleton counts can meet15; estimated23 fails. C withdrawn; D8 footprint too large. No parallel builds.
-1.44 historical CPU growth is not a derived allowable kernel slowdown. Share-nothing survives as a hypothesis.
- par-scan6757860786 read as reported, raw bundle NOT replayed: level1 finder2563MB/s at10 with near-flat CPU strengthens probe value; level9 still collapses. Block-level/work/mode confounding prevents causal cache-only or production claims.
- Service first adoption, emitter useful-work priority; bounded D4-delta count before target expenditure. No implementation, compilation, new timing, safety certification or gate relaxation.

## Rescue pass: inbox6757979390

[Report](records/reviews/inbox-6757979390/report.md). PR313 source0a649c696db745ce83a1629381590dc9ded061f0; pinned specs a2da4963. Proposals only.
- Rank new bounded opportunities: owned output lending; dense pair split4+1/wide fill; chunk-parallel MTF only after post-emitter critical-path census. Service/emitter and accepted D4 counting priorities unchanged.
- BufRead source copy is real, but the numerical match to17.7% comes from decoded bytes divided by Dw accesses and does not establish a byte share;52% reader-only cold-read attribution not reproduced from aggregate memcpy snippets. Lending preserves consumer reads of worker lines; no eliminated-coherence-transfer claim.
- Private owned output Vec/cursor needed for parallel BzReader; cannot return a slice after unlocking StreamDecoder's Mutex. Preserve CRC/error/cancel/recycling/partial-read contracts. Read_line still copies into String; Python PyBytes still owns/copies output, though one intermediate Rust copy can disappear.
- Dense pair source zeros5n then stores8n at stride5. Split4+1 fully initialising vector spans target5n store bytes, final5n layout; not DRAM-byte equivalence. Charge second walk load, tails/short pieces and marker fix. Full build+walk break-even and equal ordinary-fill control required.
- MTF relabel at run consumption changes O(n) output pass into O(r) lookup plus O(256K) prefix; all moves remain. No same-core lane rescue; K4 conditional on exposed post-emitter MTF and idle helper capacity.
- Scanner interleaving duplicates state and has no new eliminated work beyond existing paired LUT/finder. Claimed<5% is not a universal bound.
- One-third contraction is not maximum independent-set size; obtaining a half-cycle phase and expansion are not free. Restrict useful composition to existing D4 probe.
- Phrase current kernel unrescued:4-byte state per copied byte remains. First-only increases events8%; no demonstrated40Ir/event construction. Withdrawn recipe stays withdrawn; hardware claims remain scoped.
- Bandwidth question survives; direct copy/column intervention preferred. Hog revision estimates isolated nine-core interference, not full-machine causal bandwidth; no new slot.
- dir sequence stays closed; no new dir-specific mechanism. No PGO/global-v3 CI slot now; local null not universal, #201 frozen closure preserved, future source changes need a new identified opportunity.
- No new timing, implementation, compilation, safety certification or measured credit.

## Complete-algorithm pricing: inbox 6758176258

[Record](records/reviews/inbox-6758176258/report.md). Acknowledged complete construction/traversal/metadata pricing before recommending a Pathfinder build, including loop skeleton, lanes, stops/tails, branches, traffic and parallel live state. Return the dominant-cost removal and cheapest falsifier; a failed construction is not an absent advantage. This is within the existing pinned pricing mandate, not a re-pin or added execution authority. Lending copy/allocation counts, dense-pair count against an equally improved ordinary fill, conditional MTF and unchanged D4 filters carried forward. Next named share-nothing design awaits inbox dispatch; no substitute candidate priced or approved.

## Staged cooperative traverse: inbox 6758227354

[Report](records/reviews/inbox-6758227354/report.md). Source 2c70fd13e, coop/bwt matching master ae44497e9.
- Recommend a bounded private-tile counting prototype, before timing: no shared tile, new per-byte atomic or chunk barrier. Flush old segment before finish/take; take writes first byte directly; new st=j+1. On exhaustion flush all unmarked lanes through j+1, even those after the exhausted lane; pending marked lanes through j.
- Flush and finish all owned segments before Relay walkers decrement. Existing ownership and AcqRel last-walker publication suffice; no partial-segment transfer between members.
- Tile 8 KiB per walker, up to24 KiB/team and72 KiB/three teams; rows128B/st256B per walker, replacing oldbase256B. Table, segment buffers and final writes remain; +2B logical tile traffic for B staged bytes.
- Pipelined K3 requests1024 starts, plain K3 requests1536; tails/late leader matter. Claims S+W, no new per-byte atomic. Full-loop price needs B/T, chunks, fragments, copies, grows, marker searches and all callees.
- #171 four-thread evidence is independent workers, not teams. Raw historical block1 replay reproduces inclusive −1,132,562 Ir; direct flush memcpy125,321, aggregate memcpy348,489.
- Published bandwidth snippets have no Ir; 17.6/10.1 and30.5 remain reported hypothesis pending raw same-denominator inclusive attribution. Current share-nothing summary says17.8. 10–11/team is plausible, not established; current staged source still has per-lane START tests.
- Priority: staged-team kernel count before a new dense-pair implementation; lending attribution count independent. Service/emitter priorities and registered D4 filters unchanged. No measured credit or new timing.


## #324 rescue and tunnel lane: inbox 6758452562

[Report](records/reviews/inbox-6758452562/report.md). Part A checkpoint: #324 remains FAIL; high-priority untimed rescue under D082. Four raw historical profiles reproduce +61/block, decomposed +6 caller, +7 memset, +48 newly outlined finish; MTF/room counts unchanged. Scratch construction and Huffman outside this profile. Rank actual data/caller geometry, then changed call/code placement; mod4KiB discriminator withdrawn by #277 §6. One boxed diagnostic conditional on actual production evidence; lazy per-worker sidecar offers stronger fallback allocation isolation, no geometry guarantee. No timing/implementation. Part B: certificate REVISE. Pinned census uses1+minK; valid aab BWT shows safe height=minK. Fixed-k quantum copy retains room, but .0736 lookup groups/row is conditional on unverified coverage. State/entry costs can consume the gain; .2B/row words alone exceed D4's ~.123B/row residual under1.2MB. First validate words/exits and record actual segmented copy quanta/entries/capacities. No hybrid build recommendation; D4 filters unchanged. CI and local-profile baselines explicitly separated.


## Compact30 plain-worker price: inbox 6758723399

[Report](records/reviews/inbox-6758723399/report.md), with archived-grid replay. Register3a63542b2; candidate ba94ff5c; #326 draft482df588. Proposals only.
- 420 raw rows reproduce2350.48MB/s at7 only under g1.18,a1,s1.95MB. u24 ratio range .872–1.117; one-worker point calibrated. Neither error range nor separate team2430–2584 reference establishes a tie.
- g1.18 is inferred from EPYC full-decoder losses assuming70% inversion, not measured i9 build+walk. Ctor2c includes canonicalisation; oldinvalidv2 and walk-only evidence cannot fill that cost.
- X includes construction contention: production build inflation5.19 at8,7.54 at10. Walk-load fractions cannot discount that term without testing. Whole worker capacities and other decoder stages remain charged.
- Separate diagnostic preferred. #326 adding five level9 cells costs305 slots(+41.7%), both levels610(+83.3%); still lacks1/7 anchors. Five-cell P1/C1/P7/C7/team9 pilot proposed, not frozen; full p-grid transfer test11 cells. Common harness adapter required, no new kernel by this role.
- D4 walk15Ir/outputbyte, build7.35Ir/row,1.2MB filters unchanged. #195FAIL and production reopening restriction retained. Regime/default-team census and service/emitter/geometry work retain priority; compact30 small diagnostic is information, not adoption.
- No implementation, compilation, new timing or safety certification. Zero new measured performance credit.

Publication-time context: QUEUED6758831274 reports D4F2FAIL14.45>7.35; bundle not replayed here, rescue awaits inbox dispatch. No relaxing/reopening. Tunnel families remain open pending actual coverage per6758774853; outside this brief.


## D4 construction rescue: inbox6759188202

[Report](records/reviews/inbox-6759188202/report.md), source561665b61ea61ba8027e4e15068322d938ea0077. F2FAIL retained. Phase14.21 differs from registered14.45; scan-only removal leaves11.49 attribution/11.73 illustrative registered subtraction.30Ir/piece plus fixed terms still10.22.
- Rank run-fed acquisition plus two-byte-word cursor composition: LF² images monotone per fixed pair. Cursor setup/storage and stop lookup replacement charged; optimistic combined budget6.29 leaves~.82–1.06 for replacements, no demonstrated pass. Direct LF4 refinement second, cross-block reuse unsupported.
- Correct prior30Ir default: observed41.8/65.9 per piece, fixed work separate. Corpus densities and split counts matter; no global2.2 multiplier.
- Table-only law gives2628–3578MB/s at7–10 under unmeasured equal-CPI proxy. Not forecast. Scratch up to4.40MB/worker, oversized blocks and late fallback omitted by simple model. Complete worker/decoder supply and output bound required.
- Reported tests/counts source-audited, raw profiles not replayed. No implementation, timing, gate relaxation or production qualification. Team/service/emitter and geometry retain priority.


## Three-table architecture price: inbox6759345168

[Report](records/reviews/inbox-6759345168/report.md). Pin7bf5769be; correction96b7937 separately read. No12-cell timing recommendation now: chunked full fill/segmentation absent; D4F2 no-timing disposition stands; compact30 adapter deferred.
- Chunked correction allocates16B directory entries but bytes() charges12. Extra~56.25KB/full block; proposed start bitmap~112.5KB. Corrected layout+bitmap≈1.31MB Wikidata,1.44MB text before slack, contradicting1.3MB universal admission. Oracle start traversal/conversion excluded from local timings.
- g≤1.8 possible but unmeasured; fill+2Ir unsupported. Whole fill includes bucket-ordered record collection/packing, flags, rank, validation, allocation and assembly. Selector and late fallback charged.
-12 cells×21 rounds252 vs105; with2sentinels294; two corpora504/588. No claimed power/target wall-time. Whole-decoder adaptive selector needs separate mixed-workload pair.
- Tunnel1470–2200 reproduces optimaW5/W4, not10; X42MB unobserved. c=.611 k8 stored-exit differs from priced k1 learned-exit rider. Defer named build, not close family.
- Team/service/emitter/geometry remain ahead. No new timing, implementation or raw local-profile replay.


## D083 screen economics: inbox6759587407

[Report](records/reviews/inbox-6759587407/report.md), draft ffb568913ce9bd6eeb807082ac935570e36c3c8d. Bounded affected-PR coverage justified; full-Callgrind mandatory tier not ready.
- Warm stated rows220–300s, cold465–845s before omitted oracle/cache/Valgrind setup costs. Seven oracle runs omitted from36 native decodes. @9 count+parse increment80–160s, not60–120. Values are models; historical warm head/base27.08/25.14s verified from job129639441404.
- Parallel latency=max(0,queue+screen−existing required completion); no unconditional zero. Cold and queue tails matter, runner-minutes always added.
- Reproduced four empty JSON records passing A1. Error block identity not emitted; per-arm source/asm and validated helper identities required. Merge-base standalone walker symbol not guaranteed under LTO; don't mutate base to manufacture attribution.
- Rank native semantic execution/conservation probes with exact-result checks before full profiles per PR. Symbol presence/sampling cannot replace exact execution/absence assertions. Instrumented binary/probe builds charged; full matched counts remain separate diagnostic with Guardian's prospective disposition.
- D084 FAIL-MEASUREMENT unchanged. No new decoder execution, compilation, timing or qualification; parser-only counterexample. Team/service/emitter remains ahead of speculative table work.


## Share-nothing repricing: inbox6759983287

[Report](records/reviews/inbox-6759983287/report.md). Verified submitted bundle1e347594…36ad;732 matched slots/runs, round0 excluded via slots. Reproduced CPU G2=1.183382,G9=3.261240,R=.362863.
- Old production table-only X predictsG9=6.46154; observed/predicted .50472. Small-load flat extension givesG2=1.06809 (strict domain lacks baseline3.2MB). No causal table-byte percentage identified; recompression changes1831/421 blocks, +33.59% compressed input, Huffman/MTF, buffers and scanner occupancy.
- Whole-decoder-2 anchors1862/1819 at8/10 are scenarios, not candidate bounds. With unmeasuredf=.7,h=1,g estimates, C30≈1653/1616,D4≈1422/1390,chunked≈1193/1166; not forecasts. Real full footprint, scratch, late fallback and supply charged; F2 unchanged.
- Team ports2 then3 remain ahead of three-table builds, subject to D085/B5. Insert cheap fixed-2 expanded-address-layout contrast4/16B atW4/10 to test shared premise.126 illustrative slots, no build or run authorization. Allocated unused padding is not a footprint experiment.

## Phase census and width intervention: inbox6760167271

[Report](records/reviews/inbox-6760167271/report.md), [arithmetic](records/reviews/inbox-6760167271/arithmetic.json). Source/register4ce34b6093a0f7a10885004dc20c8138602b1ae2. Pricing only.
- Merge descriptive phase census with addressed4/16B -2 width intervention, after untimed boundary/code/data review. Matched on/off×width×W4/10 =8 cells; +2sentinels×21 rounds=210 illustrative slots vs126. Optional -9W4/10 on-only references give252; no automatic full610-slot census, no power claim or execution authority.
- Four reads/block:1684 at-9 but7324 at-2; assumed25ns gives42.1/183.1microseconds summedCPU/decode. Logging, allocation/capacity, dump, clock implementation, code/data placement and critical-worker exposure additional; nanoseconds are not instruction counts or wall-overhead bounds.
- Scratch persists across jobs. Four edges measure build/walk envelopes, not held-table lifetime, cache residency or all running workers. Source walk wrapper includes setup/assembly; RLE/CRC follow. Keep workers arm's2Wslotwindow fixed.
- Within-block regression remains endogenous: target[0,T], independentneighbor[1,3], T2/3 gives overlap.5/.667 absent any neighbor effect. Shared shocks, collinearity and weak support prevent causal three-model selection;60rounds do not promise power. Cluster decodes, report residual support/rank/uncertainty; retain overlap histogram because mean X(n) differs from X(mean n).
- Correct compulsory-scan arithmetic: f=.78,1862 anchor,g1.18 =>1632.76>1499; proxybreak-even g1.310463. Anchor notbound andg/f unvalidated. Old decoder law prices remain withdrawn.
- Joint scanner/table2x2 economics justified, mandatory paired implementation notproved. Charge fullfeed,cores,window,constructor,scratch,metadata,fallback,output/tail; no throughput-gain addition.
- Teamports2then3 remain ahead of speculative three-table engineering under D085/B5. No implementation, compilation, new timing, safety discharge or qualification.

## Addressed-width draft: inbox6760363087

[Report](records/reviews/inbox-6760363087/report.md), [arithmetic](records/reviews/inbox-6760363087/arithmetic.json), [historical variance replay](records/reviews/inbox-6760363087/variance-replay.json). Draft f918f783aa6b9f683d867c057a4c4937c99392ca: REVISE before freeze.
- Scaled physical u32 index k*r permits identical hot load/shift/prefetch, not demonstrated whole-walk equality. Charge orig_ptr, starts, flag cleanup, tails, logical-n/bounds and constructor addressing. Constant shifts8→10 can have equal Ir; destination scale/reconstruction may add work. Actual delta requires matched inclusive counts and listings; no invented numeric charge.
- One-row A differs from production; archived f/occupancy not transferable. Same-source const generics do not guarantee same code. First untimed semantic/B5/count admission; no implementation or counts run here.
- Every row visited once is not a universal walker invariant: periodic/multi-cycle tables are explicitly supported. Build touches all logical records; walk coverage must be measured separately.
- 200k-row page counts196/782 when page-aligned, up to197/783 with64B alignment. Stores4N both arms; unique dirty-line spans notDRAMtraffic. 1.5/5.9GB uses1831fullblocks; RLE1 boundN<=1.25D gives unique-line budget<=1.049/4.194GB for this decode, actualrowsneeded.
- B4/A4 does not isolate side effects: table-only12.8MB fitting nominalLLC is notwholeactivefootprint. Traffic/TLB/faults/privatecache/geometry can interact withW. Additive or worker-only multiplicative penalties do not generically cancel on whole-processlogCPU.
-210timedslots unchanged; optional-9references+42, optionalcontemporaneousproduction-2anchors+42 separately. Fiveclocks/block now9155reads/-2decode,228.875microseconds nominal25ns/read; setup/count/dump/placementruntimeadditional.
- exp(I)<1.15 rules out only a specified15% interaction undervalidintervals, notallcompacttables. B4=2A4,B10=2A10 givesexpI1 withlargewidthcost. >3.5doesnotrestorelaw. Fixmissing2.8–3.5categoryandE4log-versus-factorunits.
- RecheckedhistoricalarchiveSHA1e347594…36ad. RetainedloggrowthSD.105273(-2),.057699(-9),paireddifference.111984. ±5%remainsroughiidscenario,notnewwidthcovarianceor20roundmoving-blockcoverage. Researcherfixesprospectiveuncertainty.
- Teamports2then3ahead asD085/D087/B5allow; retainboundedlayoutdiagnosticafteradmission. No decoder execution, compilation, newtiming, safety discharge or qualification.

## Counts/STEPS contract price: inbox 6761295924

[Report](records/reviews/inbox-6761295924/report.md), [arithmetic](records/reviews/inbox-6761295924/arithmetic.json). Draft 78c91f7013fa37bf7ab830fb3e89ccb3d8835930; D086 screen remains FAILED. Proposal only.
- Corrected4K D←H excess42,325/890,000=0.0475562/B random;12,678/890,000=0.0142449/B records. Class differences are a pair proxy, not hardware stalls.
- Archived #324 runs2/3 imply approx random0.561/0.510ns/B harm, records0.228/0.179ns/B. At reported clocks:1.844/1.324 and0.750/0.466 nominalc/B; full attribution requires38.8/27.8 and52.6/32.7 nominalc/excesscandidate. First-block rate extrapolation and rounded-median/geometric-ratio conversion are conditional. No hardware penalty measured; neither sufficiency nor impossibility established.
- A guarantees only this pair's4K disjointness: +3KiB allocation padding/owner,16 activecounts lines unchanged, possible2KiB static wrapperpadding plusalignment/allocator overhead. No inherent perliteral arithmetic; ownership/stack/code/firsttouch changes remain.
- B4KiB activecounts/64lines vs1KiB/16; counts+STEPS48→96lines. Dense consumer temporary can make5KiB peak. Charge~3ALU/literal as instructions notcycles, run updates,reset,256counterload/storecompaction and consumers. At1exposedcycle/literal withL/D≈1, records overhead already exceeds observed wholeharm; break-even>21.0/70.2cycles/excesscandidate beforefixed/cachecosts. No1K latency evidence.
- Choose neither forproduction; A-style causal diagnostic beforeB. Two fixed STEPS offsets2048apart in onebinary, fixedcounts/frame andmatchinghotwork, preserves1K/64 whilechanges4K. Nativeaddresses/allclasses/remainingcacheandstackconfounds required. Random1t first, fixedbounded pairedCI onlyafteradmission; interval mustexclude requiredcontribution torefutefullharm. Notfourth#324run orno-harmqualification.
- Stack classes offer larger candidate budgets but areplacement-dependent andoverlapping; no summedstallclaim. Ifpairinsufficient, controlledcounts/stackandcallerspillcausesnext. Teamports2then3remainahead ofBrewriteasD085/D087/B5allow. Noimplementation,compilation,decoderexecution,newtiming orsafetydischarge.

## Held-spinner implementation handoff: inbox6765024673

[Report](records/reviews/inbox-6765024673/report.md). Build not performed: active pin a2da4963 forbids implementation/merge; newer O1 at3664ada282fb6e3cedf4c6ad91053725d9ced18f is not the user-selected pin. Newer Optimiser spec itself still forbids merge. Response complete, feature outstanding; route to authorized implementer or re-pin before redispatch.
- Source ab.rs inspected: supervisor-owned spinner, one instance, readiness before settle, explicit cleanup/reap across error/interruption, unchanged old ARM@EXE interface.
- Account spinner separately from decoder CPU and max-busy; highest logical CPU may share a physical core at10threads. Validate actual allowed topology rather than assume spare core.
- W1 B30/30 and0.141s versusG0.293s are reported, not independently replayed. Benchmark conditioning is not library speedup; preserve cold-start/old result meanings. Auxiliary CPU cost includes settling/gaps.
- No code, PR, timing, merge, or prompt change. Implementation request remains outstanding; scoped response only.

## Rauzy kernel source cost: inbox6767027944

[Report](records/reviews/inbox-6767027944/report.md), [arithmetic](records/reviews/inbox-6767027944/arithmetic.json). PR351 pinned149307d6c20868a18edf9d835c30ca213a54ec6e. Static partial result; dynamic profiles outstanding.
- No Rust/Valgrind tools; pinned example embeds timers and no untimed corpus mode, so no fresh Ir/miss/misprediction counts claimed.
- Kernel ordinary source path:9u32 stores+32arena bytes+16Rule bytes=84addressedstorebytes/step (~10.992/row), notmachineinstructions/DRAM. Both word representations written evenwhenone discarded; dummy writes andprefetch cursor recurrences counttargets.
- 93rounds issegmentDAGdepth, notindependentrounds. Current sharedtop/nr/sentinel/neighborwrites preclude assumingcurrentkernelparallel-safe fromdisjointconsumedlettersalone.
- Node link narrowing alone leaves20usefulbytes,align32still32, not16. SoA alone preservesdependentindices.
- Fixed order+write.944ns/row; parity induction<=1.228 (1.487×); halfproduction<=.142 (12.859×), beforecoordination. No93×ceiling. Fullr scans93times≈6.715n visits;35Ir/cell would≈235Ir/row, notobservedcost. Needactivefrontiercensus.
- Deliveron145forCoordinatorforwarding;346outsidepinnedwriteauthority. Staticresponsecomplete; countrequestexplicitlyoutstanding. Noimplementation/timing.


## Cells stage-to-decoder gap: inbox6782112235

[Report](records/reviews/inbox-6782112235/report.md), [archive arithmetic](records/reviews/inbox-6782112235/arithmetic.json). Frozen #365 57f51bbc0ca62ac3ee1cc148a1010842eea6222a; #362 design/handoff7611f52b. Recommend one cells-live-path-gap diagnostic for Coordinator preparation; no implementation or launch.
- Downloaded and SHA256-verified i9 bundle552e26aa…57baed705c and final CI artifactc36061ac…cb575141. Reproduced all7 raw paired wall ratios; Wikidata1.000093, pattern2.231663, onebyte1.109001. Original intervals/outcomes retained; hashes match.
- Archived census predicts196 admissions/35density/0floor/2budget refusals,84.1206% row coverage. All233 takes=true is only records-path eligibility. Ideal selected inversion0.773–0.782 beforeattemptcosts; coverage alone cannot explain loss. Linear budget-prefix scenario0.780–0.789 is not measurement/bound.
- Same compressed corpus manifest/hash in both builds; i9 warm minimum-of5 unbounded example versus EPYC fresh-process bounded decoder. Ordering/writer already in c_whole. Frozen stages.jsonl is old column/forward path, not live run-front/backward path; no valid live f from it.
- Cells scratch48r+84B+4528 plus baseline/output/front buffers. Census size replay predicts10 buf growths,148399232 cumulative zero-initialized requestedbytes; not physical traffic/time. Full validation/budgetchecks,writerpasses/copies,latefallback must be charged. No CI route census or causal allocation attribution claimed.
- Conditional Amdahl originalstagegain17.3–18.5% timeslivef minusoverhead; f=.25 yields4.33–4.63% beforecosts. No i9→EPYC causal subtraction or W4/W10 forecast.
- Rank production-transport/firsttouch, fallbackmix, hardware/commonwork. Proposed fixed3corpora×4off/on-baseline/cells×10rounds=120 diagnosticdecodes plus9untimedchecks; ~50s historicalarithmetic withsettle beforebuild/overhead, not runtimepromise/powerclaim. Separate registration, no365edits, no caps/defaultchanges, no B5claim.

## Cells live-path writer sharpening: inbox6850318070

[Report](records/reviews/inbox-6850318070/report.md), [saved-data replay](records/reviews/inbox-6850318070/replay.json). Recipe cb73bf909397e2fff8c90f0a00437061ad06478e; corrected source d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2; submitted bundle01f5bc23ea2242969b084ebff7bf3b178814f3a2.
- Bundle SHA25623c8712e1d1f2a64bbbb2f10232db7f21a8a8499826ac8f9ac42a2bd27374414 verified. All120 measured records/resources +9 savedchecks reconciled; four binaries, three inputs and inspected source/hook hashes match. Pinned analysis byte-identical. Three Wikidata byte files absent; no decoder/oracle execution or missing-byte reconstruction.
- Frozen off/on ratios: Wikidata0.881643/1.338008, onebyte0.535508/0.994670: quantitative phase attribution STOP, costtables null. Pattern1.530338/2.913851: instrumented accounting only. Preserve every sample; no inferred conditioning cause or warm20+20.
- Pattern22/22 admitted, nofallback;18,899,586small bytes+22marked=18,899,608period;700,392repeat bytes completes19.6Mrows. Zero long-copy bytes DOES NOT establish zero long seeds/reachable expansion. Actual eligibility unknown.
- writer177.428ms includes per-copy TLS/RefCell/twoBTreeMap bookkeeping. Unisolated hook overhead cannot be subtracted. Conditional zero-writer total41.681ms; replacement allowance33.378ms for whole instrumented parity,26.417ms within inversion. Neither is an off-arm estimate.
- Rank1 structural all-small-spine fast path: ifeligible, remove4R NILfill+2R position scans+thirdspinewalk; retaincopy/repeat. Pattern upper10,800,036logicalinitializationbytes +5,400,018scanvisits; eligibilityzero meanssavingzero pluspredicatecost. Cheapest falsifier untimedseeds/reach/depth; independentB5 required.
- Rank2 lazyposition/reachable-ruleprocessing deferred: charge4R epochcapacity+4R_live touchedindices,tagchecks,ordering/sort andwrapreset. Dense reach defeatsits premise.
- Exactlyone recommendation: boundedpreparation ofrank1 conditionedonuntimedeligibility. Laterdiagnosticproposal60hook-freeprocesses+9checks, fixed10pairs/corpus; patternnew/old<=.95 and95%upper<1, othersupper<=1.05, noextensions. Noadmissionhere; no productionparityclaim.
- #366onecore remainsdeferredwithoutnewworkstationevidence; broaderproductiontarget(~0.9%releaseharm) butno numericrankagainstpattern. Reachpreparationfirstoninformationcost; generalwriterbelowpreparedonecore untilsparsityshown.365DEFER/B5/MONODROMY/Pathfinder preserved.

- Prepublication reconciliation: Researcher6850440971/report9c920c64 and Coordinator6850491677/6850515770 read. Output-order emitter priced asrank2; lazysparse rank3. Pattern observedlong-copyremovalceiling0; logicaloutputalreadyN. Coarse two-tree traversalceiling75,598,388nodes +2,700,009initialdescentsteps, notprediction. ExistingMONODROMYqueue owns proof; fold narrowguardquestions there, no parallelproject/fullcorpuspreparationdecode. Actualeligibilityonlyinlateradmittedninechecks. No additionalrecommendation.


## Seven-family and early-dispatch delta: inbox6850660172

[Report](records/reviews/inbox-6850660172/report.md), [coverage](records/reviews/inbox-6850660172/coverage.json). Parent6850318070/REVIEW6850572409 retained; queue6850583026 handled as a bounded delta.
- Seven exact compressed identities inventoried from historical CI57f51bbc artifact13802703606. text pinned to that source tree and compressedSHA256f0031809…4710d75; scale4 explicit. RNG runs-before-random and text immediate-file ordering preserved.
- Actual old matched seven-family Cells slice exists; not Alternatives. Historical wall/CPU/RSS included, but four current live-reader families remain NOT MEASURED with no route or decodedSHA256 evidence. Three current saved correctness hashes retained, absent Wikidata byte files not reconstructed.
- Saved64r<n screen bypasses22pattern/1onebyte/0admittedWikidata, but admission is not a per-block speed label. No four-family validation or robust cheap speed discriminator; no dispatcher selected.
- One next preparation: CI-first seven-input identity/byte-correctness/routes plus conditional all-small-spine writer eligibility. MONODROMY6850491677 retains proof, Pathfinder architecture; no parallel implementation.
- Prospective two-arm Segmented/currentCells programme:140 measured+21untimed+2memorycontrols=163 executions, ten balanced pairs per family, fixed settle/schedule/deadlines, simultaneous14wall/CPU intervals, <=1.02 per-corpus upper bounds. RSS nativeVmHWM/exactPIDwait4 controls; uniform41168KiB not parity. All proposed/unadmitted, exact future recipe commit still needed.
- Supersedes unadmitted60+9 as next coverage preparation; unchanged120+9, Wikidata0.881643 gain, quantitativeWikidata/onebyteSTOP,365DEFER, warm20+20unadmitted, independentB5. No compiler/decoder/generator/timing/census/workstation/queue/default/gate execution or change.


## Guarded writer next decision: inbox6852341904

[Report](records/reviews/inbox-6852341904/report.md). Audited835984e source/PLAN and first experimental job131868260723 log; seven families, ten pairs, same-bytes yes. Wikidata wall1.099[1.041,1.160], CPU1.098; pattern1.782[1.599,1.986], CPU1.721; logged RSS26/34MB and16/18MB. Artifact digest remains producer-verified, not independently downloaded in this delta.
- Select exactly one disposition: park Cells integration/new dispatcher, preserve exact Segmented everywhere, await existing6851728088 final receipt and specific reopening evidence. Not a closure of the Cells line.
- Point parity requires9.01%/43.88% reduction of candidate wall on Wikidata/pattern, before added dispatch costs. No matched old-Cells arm: no guard-effect claim or cross-host subtraction. Originali9 Wikidata0.881643 remains intact for originalbuild only.
-64r<n spares pattern but retains all admittedWikidata; cannot cure AMD loss by corpus separation alone. No vendor/model whitelist inferred from confounded hosts/builds. Charge descriptor/branch/fallback/misclassification/capacity; no proven qualified target.
- Screen378 fills seven route/correctness identities; candidate AMD covers seven performance rows, not changedwriteri9. Adjacent emitter overlaps alreadyremovedpatternwork; representationselective32U+16S stores remainsunpriced. ReuseMONODROMY6850491677/Pathfinder, no new implementation/reviewer/recipe.
- Reopen on exact-final-build matched target netwin with sevenfamily whole-reader wall/CPU/RSS, validroute/correctness, independentB5. Fixture/cache repairs and skippedsteps preserve firstnegative; no rerun-to-pass. No execution/admission/default/merge/release changes.


## Default integration attribution after matched i9: inbox6867986711

[Report](records/reviews/inbox-6867986711/report.md). Matched i9 now demonstrates selective/original Wikidata0.9256 [0.8823,0.9711], selective/Segmented0.8630 [0.8319,0.8952]; D104 closes only pre-start machine/activity hold. All252 rows retained; pattern1.4352/RSS1.3107 costs remain. PR380 default gate failure and consumed rerun history unchanged.
- Independent source-diff review agrees with completed3c61b203 attribution: no newly executed default operation or justified patch. Default4t random/text/Wikidata first losses2.79/3.87/2.30% remain.
- New archive gap: exact workflow uploads default .text hashes, not default binaries/raw sections; selective-codegen captures experimental induction only. No saved default hot-code witness inferred from904→827bytes.
- Rank1: one bounded untimed default-executable correspondence preparation, if Coordinator commissions it; inventory first, at most3clean builds/60min job plus90min inspection,0decodes/0samples. Three-source timing can localise whole-source effects, cannot alone establish layout/codegen causality. Corrects earlier “decisive” wording; no new admission.
- Fixed stop: absent correspondence/named relevant difference/isolating intervention => no sound causal discriminator, no more samples. Equal accounted default code/data/placement rejects that machine-change explanation only in compared context. Differences are leads, not causal proof. Historical digest mismatch means reconstruction only.
- Conditional parity recovery budgets2.71/3.73/2.25% current candidate wall; not hard bounds or production benefit. Preserve Cells line, completed B5, MONODROMY/Pathfinder, all default/team/cap policy and integration DEFER. No implementation, build, timing, registration, rerun or merge executed.
