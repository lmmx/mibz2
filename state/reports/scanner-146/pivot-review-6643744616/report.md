# Binding-cost inference and deferred-handover case 2

Trigger: [6643744616](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643744616). Researcher, 2026-11-04.
Scope: first #157 bench, head 75b13f00, run [41923819666](https://github.com/lmmx/mibz2/actions/runs/41923819666). At retrieval the sticky comment announced a running replicate at 3fc3f8c6 but its result body still belonged to this first run. No replicate result is inferred.

## Verdict

The evidence argues against spending the next effort on more of **this rolled-loop control cost** solely because its instruction count is large. It does not falsify every instruction-limited mechanism at two/four threads, identify memory as the bottleneck, or establish no regression. The binding cost remains unidentified. Investigate the whole decoder, as the revised role directs, without converting an instruction ledger into a timing model.

Case 2 still supplies no semantic counterexample: the correct relation is equality **after commuting the old tail's pending ordinary steps**, not equality at the instant of break. The archived argument's “every other lane's state must equal” wording is too strong and should be replaced with this simulation relation.

## What the archive actually establishes

Independently parsed both raw block-1 profiles at b0f6e89d, separating self costs from call-arc costs:
- self totals 25,751,812 / 20,184,713 match PROGRAM TOTALS;
- decode_block inclusive 25,751,796 / 20,184,697;
- room_segments inclusive 15,426,892 / 9,859,793.

The net cut is 5,567,099; decode ratio **0.783817**, walk ratio about 0.639. Whole-walk charges are included. These are Rust 1.97 counts on the archived block, not retired-instruction measurements of CI's Rust 1.99.1 corpus runs. Raw-profile accounting was recalculated; no target binary was executed.

The first CI report gives:
| Wikidata threads | wall head/base, 95% interval | CPU head/base, 95% interval |
|---:|---:|---:|
| 1 | 0.958 [0.920,0.997] | 0.958 [0.920,0.997] |
| 2 | 1.022 [0.994,1.051] | 1.014 [0.995,1.034] |
| 4 | 1.004 [0.972,1.038] | 1.009 [0.976,1.043] |

“No absolute regression” must mean **no statistically detected regression**, not established noninferiority. These intervals permit modest benefits and material losses. A replicate is useful; noninferiority requires the existing accepted margin and an upper bound meeting it, not simply an interval crossing one.

A proportional whole-decoder instruction/time prediction of ~0.784 is far from these intervals. That simple model is unsupported even at one thread. But its premises—same compiled workload, fixed frequency/IPC, unchanged scheduling and memory behaviour—were not demonstrated. Removing loop-control instructions while leaving the load-dependency chain intact can save retired instructions without shortening that chain. Front-end pressure, prefetch timing, code footprint, memory stalls and scheduling can offset or mask changes. “Walk instructions do not bind” is too broad; “this intervention has not demonstrated a multi-thread throughput gain” is supported.

### Amplification is an identity, not a diagnosis

For paired CPU ratios R_n, amplification head/base = R_n/R_1. Rounded point estimates give:
- 1.014 / 0.958 = **1.05846** at two;
- 1.009 / 0.958 = **1.05324** at four.

These explain the reported +5.9/+5.4% without needing a new contention cause. Yet they do not make the flag irrelevant: the one-thread benefit failed to carry through, which is exactly what the ratio records. Its uncertainty requires the paired observations, not division of independent interval endpoints. I did not replay CI's intervals.

The trigger's scanner 0.11 / worker 1.21 seconds comes from the earlier compact30 report, not this run. This report has scanner **0.07**, summed workers **0.85**, wall **0.305** seconds for head at four. Even a crude worker lower bound is 0.85/4=0.2125, versus scanner CPU 0.07. The report's model places the scanner crossover near twelve workers. This makes scanner *compute alone* a weak first suspect at four; CPU totals do not exclude scanner stalls, queue starvation or scheduling criticality. The approximately 0.0925 wall-minus-worker-average difference is not a measured handoff cost: it mixes imbalance, fill/drain, waiting, unattributed work and overlap.

The one-thread phase diagnostic now follows the changed T-vector room path, unlike #155. But walk −5.8% [−15.3,+4.9] is uncertain and one-thread only; it cannot locate a two/four-thread bottleneck. The “19% rest” is an instruction share, not an elapsed-time share.

## Case 2: inspect the actual intermediate observers

Inspected src/bwt.rs at 3fc3f8c6, including the full-loop exit, live-lane construction, tail entry and tail. Let j>k be unmarked when k exhausts the queue.

Old state: row=r, write cursor p. New state: byte(T[r]) written at p, row=successor(r), cursor=p+1. That ordinary step writes no next[], queued or other lane state.

Between break and tail:
1. lanes[..active] is copied; this exposes no bytes externally.
2. next_unset reads **seg and next[seg]**, not row/p. The old and new predicates agree, since j's ordinary step cannot change either.
3. Tail-entry capacity checks **do read p/end**. “Nothing reads lane state” is therefore literally false. If j became full, grow sets length to the already-written prefix, reallocates only j's buffer and preserves it. In the old execution the corresponding tail step writes that last byte and then invokes grow. Allocation addresses and event timing may differ; logical byte sequence and cursor index agree.
4. Tail ordinary/finish actions on different lanes commute under distinct-buffer ownership and immutable T. swap_remove changes processing order, not segment IDs or the edges written by finish.

If j's step lands on START, the new tail immediately finishes at that successor. The old tail emits the same byte, reaches the same successor, then finishes; exactly one byte and the same next[j] result. If the emitted byte filled the allocation, both routes grow before the next table read. Pending marked lanes remain at off. Previously handed-over lanes already point to the new segment's byte-one cursor. No intermediate consumer assembles or exports partially written segment buffers.

Thus the intermediate mismatch is required by the transformation, not a correctness failure. This argument assumes existing valid table/row, unique segment-buffer ownership and successful allocation invariants. It is not a claim of identical trace counters, allocator chronology or performance.

### Test discrimination

The new room_walk_deferred_handovers test compares complete segments to the oracle at 32 and 5 lanes and asserts three transition classes. It is good evidence, but neither named exhaustion counter explicitly asserts **unmarked j>k already stepped**, particularly a step landing on START or filling its buffer. The two reported mutants (“all waiting at off+1”, “waiting=false”) attack the same pending-marked error direction. A complementary mutant leaving a later *unmarked* lane at off with its row already advanced would test this obligation directly. A targeted witness should assert the case occurred and check both segment bytes and next[], including exact-full and successor-START cases. This is a review recommendation, not a patch or new execution.

My prior 40,923-case abstract check exercised the ordinary-lane commutation at exhaustion, but model checking and source reasoning do not substitute for native memory-safety evidence. No new Rust/Miri run is claimed.

## Measurement that would discriminate the causes

First use the already-running paired replicate to check whether the small multi-thread effect reproduces; it settles neither resource attribution nor universal absence of gain. Do not pool different runner CPU models as interchangeable replicates.

The discriminating diagnostic is matched **actual CI/release binaries and corpus**, at pinned one, two physical-core and four SMT placements, with paired:
- retired instructions and cycles per decoded byte, together with frequency and on-CPU/off-CPU accounting;
- per-stage/worker attribution of cycles and relevant load-stall/cache-miss or memory-traffic evidence;
- scanner/worker queue readiness and block start/end timings sufficient to identify worker imbalance and the critical path.

If retired instructions fall but cycles do not, CPI rose: instruction count alone is refuted for that intervention; it does not yet identify the cause. Lower instructions and worker cycles with unchanged wall implicate overlap/critical-path limits. A memory-latency claim needs long dependent-load stalls, while bandwidth saturation needs traffic approaching a measured relevant limit; cache-miss counts alone establish neither. SMT-specific worsening must be separated from two distinct cores.

This proposes the minimum useful causal distinctions, not permission to alter reserved workload/statistics/acceptance criteria, add infrastructure, or run new timing here. If the existing runner cannot expose counters, the current report can decide practical candidate value but cannot settle microarchitectural causality.

Prior involvement: earlier deferred-handover assessment and abstract checker. No implementation, merge, new timing or authority expansion. Evidence is pinned archive/source plus the first published CI report; no fresh independent CI interval certification.
