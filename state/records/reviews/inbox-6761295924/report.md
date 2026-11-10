# Step 3a: counts/STEPS contract economics

Inbox [6761295924](https://github.com/lmmx/mibz2/pull/160#issuecomment-6761295924). Reviewed draft **78c91f7013fa37bf7ab830fb3e89ccb3d8835930**, under SPEC a2da49638ac54d7763de1c167bd054f67ffdc763 and D086. Proposal only; no implementation, compilation, decoder execution or new timing.

**Recommendation: neither contract for production now. Choose A's geometry for a bounded causal diagnostic; defer B.** The trace rate does not rule out a material contribution, but explaining the entire observed harm requires approximately **28–53 nominal cycles per excess candidate**, even before discounting candidates that are not exposed hardware delays. There is no measured penalty or valid hardware upper bound here that establishes either sufficiency or impossibility. Restoring the counter is not evidence of restoring elapsed time.

## Evidence and scope

- [Fallback journal](https://github.com/lmmx/mibz2/blob/78c91f7013fa37bf7ab830fb3e89ccb3d8835930/docs/journal/2026-11-10-run-front-fallback.md), corrected independent-class section, blob f4d219859a8aed5c2f8eac9dbadc9ff8328359b9.
- [MTF source](https://github.com/lmmx/mibz2/blob/78c91f7013fa37bf7ab830fb3e89ccb3d8835930/src/mtf.rs): 64-byte Step rows, three 16-byte loads, contiguous counts reset, literal and run updates. Source supports the proposed residue argument; it is not a final compiled contract.
- [Production registration/results](https://github.com/lmmx/mibz2/blob/78c91f7013fa37bf7ab830fb3e89ccb3d8835930/docs/journal/2026-11-10-run-front-production.md), especially §6.
- Preserved CI [run 2](https://github.com/lmmx/mibz2/pull/324#issuecomment-6758194980), [run 3](https://github.com/lmmx/mibz2/pull/324#issuecomment-6758350884), read directly. Head e88c063155c3e734e748e99ccda025393302720f, base 5e7b99547. EPYC 7763 and 9V45 respectively. These are different hosts, not a controlled architecture comparison.
- Contracts A/B are described in the inbox; the pinned fallback journal does not yet specify them.

The repaired agg3 classification is a sounder statistic than the superseded first-match classes. I checked the reported arithmetic, not the raw trace replay or test execution. Its last-48-observed-stores window is not a measurement of native outstanding stores. Each load can enter several classes. Three STEPS loads can be affected by one unresolved store; their hypothetical costs must not automatically be added as independent serialized penalties.

## 1. Required cost of the named pair

The trace uses a separately compressed first **890,000 output bytes**, one block, of each corpus. CI decodes entire 8,000,000-byte random and 9,600,000-byte records files. Using the first block's rate for the full corpus is an explicit extrapolation, not a measured corpus-wide rate.

| Corpus | Master 4K D←H | Head | Excess | Excess / decoded byte |
|---|---:|---:|---:|---:|
| random | 13,829 | 56,154 | 42,325 | 0.0475562 |
| records | 14,255 | 26,933 | 12,678 | 0.0142449 |

These are **class differences**, used as an attributed-pair proxy. Exact corrected counts for just the named PC pairs, and any cancellation by other pairs, must replace that proxy before claiming pair-specific attribution. Total head class rates are 0.0630944 and 0.0302618/B; charging those totals against incremental harm would also charge master's pre-existing candidates.

Let e be the excess proxy/B; h the paired fractional wall harm; t0 the baseline wall seconds; D output bytes; f the reported GHz. The approximate harm is h*t0*1e9/D ns/B. Nominal cycles/B multiply by f. Required effective nominal cycles per candidate p_req = h*t0*1e9*f/(D*e).

| Corpus/run | Rounded base wall | Paired harm | Approx harm ns/B | Nominal harm c/B | Required nominal c/candidate |
|---|---:|---:|---:|---:|---:|
| random / 2 | 0.132 s | 3.4% | 0.561 | 1.844 | 38.8 |
| random / 3 | 0.077 s | 5.3% | 0.510 | 1.324 | 27.8 |
| records / 2 | 0.146 s | 1.5% | 0.228 | 0.750 | 52.6 |
| records / 3 | 0.082 s | 2.1% | 0.179 | 0.466 | 32.7 |

Reported clocks are 3.2867 and 2.5961 GHz. These are **nominal wall-equivalent cycles**, not PMU core cycles or measured stall latency. Multiplying rounded median baseline by the paired geometric ratio is an approximate scale conversion, not an exact reconstruction or confidence interval. CPU ratios also move in the harmful direction; that does not identify the mechanism.

Sensitivity, assuming the same e transfers to native CI:

| Assumed effective cost / excess candidate | random c/B | records c/B |
|---|---:|---:|
| 5 cycles | 0.238 | 0.071 |
| 10 cycles | 0.476 | 0.142 |
| 20 cycles | 0.951 | 0.285 |
| 50 cycles | 2.378 | 0.712 |

These are assumptions, not processor latency specifications. In particular the earlier roughly-50-cycle fit is not independent evidence for choosing 50. At an effective ceiling of 20 cycles the proxy cannot explain the full point-estimate harm in either run. No such ceiling has been established. If only fraction q of proxy candidates produces an exposed delay, actual delay per exposed candidate must average p_req/q: at q=1/4, approximately 111–211 nominal cycles. Overlap between loads can further reduce exposed cost. Conversely, the artificial window can miss native relationships; it is not an upper bound on all hardware events.

**Verdict:** quantitatively possible only under an unverified effective-cost model; not established as the explanation, and not safely dismissed as too small. The measured quantity needed is the change in native elapsed cost under a controlled change in this pair's overlap, with other interactions checked.

The stack classes are serious competing explanations. For random pad 0, S←H excess is 741,792/890,000 = 0.8335/B and H←S excess 145,359/890,000 = 0.1633/B. Records gives 0.9144 and 0.1638/B. A few effective cycles per candidate there would already match the observed scale, but the classes overlap and cannot be summed into a stall budget. Their native stack placement is unmeasured. Caller spill scheduling, code placement/front-end effects and cache-set conflicts remain alternatives. Identical instruction totals do not exclude them. The out-of-line candidate restores the heap/stack classes together, so it does not isolate STEPS causality. Its read_item 1K S←S rise is a separate unresolved risk.

## 2. Full contract price

### A: contiguous counts, disjoint 4K windows

With STEPS beginning at residue 0, its 2KiB payload occupies [0,2048). Counts beginning at residue 2048 occupies [2048,3072). The addressed byte ranges are disjoint mod4096. Counts spans all mod1024 residues: A cannot provide a 1K guarantee. Its guarantee concerns these two objects only.

- A 4096-byte counts allocation contains 1024 useful bytes plus **3072 padding bytes per owner**, once per retained worker/owner. The payload still touches 16 lines, not 64, if padding is not initialized or accessed.
- A directly over-aligned 2048-byte STEPS wrapper can have 4096-byte type size: **2048 static padding bytes**, plus possible linker placement slack. Actual object/linker layout must be inspected. Over-aligned allocation and allocator overhead add to the accounting; 3KiB is a layout delta, not a universal RSS delta.
- Hoist the counts pointer once. No additional per-literal index arithmetic or compaction is inherently needed. Pointer acquisition, ownership/lifetime, allocation and reset are still charged; retain the allocation rather than allocate per block.
- Moving counts out of Scratch can alter caller layout and counts/stack interactions. The proof does not preserve them, nor code alignment, and does not fix read_item.
- It may improve an overlapping placement but has no pair-related gain to claim on a master execution already free of this pair. It buys a guarantee, not a measured master speedup.
- The final type must establish allocation alignment AND field offset. Merely aligning a box pointer or a containing Vec does not align its pointee at the required offset.

This is the lower-risk economic construction if a 4K pair delay is established. Additional first-touch/cache/TLB effects are host-dependent; no invented cycle charge is assigned.

### B: four counters in each line's final 16 bytes

For slot c, offset = 64*(c>>2)+48+4*(c&3). With both bases aligned to 64, four-byte counter accesses occupy residues48..63; STEPS' three 16-byte loads occupy0..47. Disjointness mod64 implies disjointness mod1024 and4096 for that pair.

- Strided counts occupies **4096 bytes / 64 touched lines**, versus1024 /16. Extra active line footprint is **3072 bytes /48 lines per worker**, not just untouched padding. STEPS is another32 lines, so counts+STEPS rises48→96 lines (3KiB→6KiB), before rows, streams and stack.
- Dense compaction reads256 counters (1024 useful bytes across64 lines) and writes256 contiguous counters (1024 bytes across16 lines) each block. Scalar accounting is256 loads+256 stores plus loop/address work; vectorization changes instruction counts, not useful bytes. Cold line fills/writebacks are not guaranteed: lines may remain resident. No 4KiB DRAM transfer is asserted.
- If the consumer still requires a separate dense1KiB array while strided counts is live, peak storage is **5KiB**, +4KiB against the original array; otherwise prove reuse/ownership. Eight workers would add24KiB active strided footprint alone, or32KiB storage with simultaneous dense copies; not a measured shared-cache or scaling cost.
- Reset must clear active slots; literal increment, run increment, scalar dispatch/twin, checkpoints and downstream consumers require a complete API audit. The existing '*counts=[0;256]' cannot be reused unchanged.
- The brief's ~3 extra ALU operations/literal is a hypothesis for codegen, not3 cycles. With literal density l=L/D and exposed address cost a cycles/literal, charge **a*l + C_compact/D + C_setup/D + C_cache/D**. Run-flush updates add their own address work. Small blocks amplify fixed costs.
- At l≈1, even **one exposed cycle/literal** requires >21.0 random or70.2 records cycles saved per excess proxy candidate to break even, before cache/reset/compaction. At three cycles it requires >63.1/210.6. The observed entire harm budget is only1.324–1.844 random c/B and0.466–0.750 records c/B. Thus one extra exposed cycle/literal already exceeds records' whole observed harm; three exceeds both. Actual overlap may reduce a, but 'off the MTF chain' does not make it free: the address depends on decoded c, and shares registers/front-end/execution capacity.
- As a scale illustration only, 512 scalar compaction memory instructions /890000 =0.000575 instructions/B. At256 output bytes it is2/B, before address/control/reset. This is much smaller than literal work on full high-entropy blocks, but must not be omitted.
- Prove **compiled access widths** in every enabled path: source-level absence of padding reads is insufficient. Wide zero stores to gaps violate a universal 'every counts access' residue claim. Scope the guarantee to MTF row loads versus active counter stores, with initialization and compaction outside that phase and boundary interactions disclosed.

B protects1K as well, but the present4K comparison supplies no evidence that eliminating1K candidates buys time. The corrected1K D←H class is roughly unchanged across builds. B is a changed counter representation, not the requested cheap layout-only diagnostic. Defer it unless a controlled1K/4K result shows enough net benefit.

## 3. Cheapest useful causal falsifier

Propose one **prospectively registered diagnostic**, subject to Guardian admission under D082/D086; not a fourth #324 attempt and not production qualification. No workstation execution or new acceptance rule is authorized here.

Start with master, random, one pinned CI thread, whole decoder, two prescribed geometries. Same executable is preferable: select one of two pre-initialized STEPS locations outside the timed region, with identical hot instructions and counts location. Keep STEPS64-byte alignment and the two locations separated by2048 bytes, preserving mod1024 while changing mod4096 overlap. Counts is fixed in a 4KiB-aligned allocation at a prescribed residue. One STEPS2KiB window overlaps counts; the other is disjoint. Both copies exist and are initialized equally in both arms; only the selected pointer changes. This is a pointer/layout test seam, not a new MTF algorithm. If the seam changes compiled hot work or cannot be hoisted, stop and redesign before timing.

**Why not simply move counts in two separate builds?** That simultaneously changes the much larger counts/stack classes, potentially caller code and cache placement. It tests combined layout sensitivity, not this pair. Even moving STEPS changes its stack and cache-set relationships: do not claim perfect hardware isolation. Record all independent agg3 classes and address ranges, and disclose these remaining confounders.

Untimed admission:
1. Pin code/toolchain/input and both fixed offsets before observing time. Verify same outputs and relevant safety obligations. Verify identical hot listings, instruction work and frame; keep pointer-selection/setup symmetric.
2. Log native selected addresses, stack-relative slots and build identity. Verify targeted pair overlap/disjointness and mod1K/64 preservation; no search over padding or 'good' seeds.
3. Show corrected pair-specific candidate rates change in the intended direction on the exact experimental binary and input, while counts↔stack geometry stays fixed. Valgrind geometry alone is not native verification. Record any D←S change, cache-set change, remaining differences and PMU availability rather than assuming counters exist.
4. Choose rounds, pairing/order, interval method and stopping rule before data. Reuse approved CI pairing machinery; fixed bounded sequence, no retry-to-pass. Researcher specifies prospective precision; historical #324 intervals are not a power calculation for this test.

Outcome rule: measure aliased-minus-disjoint ns/output-byte with an interval. Under the observed experimental candidate-rate contrast, compare its bound to the approximately0.51–0.56ns/B random harm needed above, adjusted if native excess differs. A valid upper bound below that required contribution refutes the hypothesis that this pair alone explains all harm **in those conditions**; a null estimate with a wide interval is inconclusive. Failure to create the intended contrast is an invalid intervention, not a negative causal result. A positive effect establishes sensitivity to the controlled layout intervention, not uniquely false-dependence stalls if cache/stack confounders also moved. A follow-up control or supported hardware evidence must distinguish them before attribution.

Random is the cheapest first screen because its trace excess is3.34× records'. Only if informative, confirm records and then host portability under a separate predeclared extension. Full no-harm/benefit qualification remains necessary for an eventual corrected production candidate. The failed out-of-line screen is unchanged.

## Whole-decoder priority

Retain the run-fed line: its target-data benefit remains substantial despite #324's closed construction. Within it, **A-style causal diagnostic > B representation rewrite**. If the pair cannot carry the required budget, prioritize a controlled counts↔stack/caller-spill experiment, then code/cache placement, rather than polishing a STEPS-only guarantee. No one of those is presently a proven remedy.

Across the team, admitted ports2 then3 remain ahead of an unpriced B rewrite, subject to their current D085/D087/B5 constraints. A bounded diagnostic can justify future effort; it does not displace a better-supported binding-worker reduction or authorize another registration.

## Audit limits

This is a source/arithmetic review and proposal, not independent qualification. No raw agg3 replay, candidate build, safety rerun, hardware event measurement or timing was performed. Nominal cycle conversions and first-block extrapolation are explicitly conditional. See arithmetic.json for inputs and unrounded calculations.
