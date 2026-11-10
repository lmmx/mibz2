# Bandwidth titration: REVISE before queuing

Assignment [6755685313](https://github.com/lmmx/mibz2/pull/162#issuecomment-6755685313).
Reviewed #313 head **a68d6099fd44588584ba5e22c1aebbd17fffe6c9**.
Authenticated instruction pin **a2da49638ac54d7763de1c167bd054f67ffdc763**; SPEC, SPEC.RESEARCHER, MODE, RULES and PROTOCOL read. Migration head and manifest verified before receipt.

**REVISE.** The intervention is potentially useful as a memory-interference sensitivity test. Its current controls cannot identify the shipped decoder's bandwidth ceiling, and NOT BOUND cannot close the cold-write candidates. No workstation execution, implementation or merge is authorised by this review.

## Corrections to the previous review

The census now checks equal cell IDs at every copied position. Since each cell has constant F and translation Ψ, the induction supplies equal output and the next displacement. On the valid aab witness, the revised predicate stops at length one, where the old predicate incorrectly accepted two. Policy 1 chooses nearer first/latest; policy 2 chooses nearer visited neighbour with an explicit tie rule. The two specific source-level objections are closed. This is a Python predicate replay plus source inspection, not a native census run.

The journal preserves the original and adds a correction, satisfying the distinction between correcting and rewriting a result. Its new 0.0972 events/row and 92.8% copy share for first/latest are reported evidence, not independently recounted here. The per-block digest remains abbreviated. “Exactly one real step a cell” needs the policy qualifier: the same table gives policy 2 real/n=0.0767 versus cells/n=0.0722.

Prior art is now explicitly credited; no priority for copying is claimed. The old traffic comparison and phrase-knee diagnostic are explicitly withdrawn, and the inspected recipe no longer contains bench-phrase-knee. That closes the old experiment's readiness question by withdrawal, not safety certification of its examples.

The lane experiment is evidence against that implementation's profitability, not proof that memory cannot bind. A flat local lane sweep and simulated instruction/branch counts do not isolate exposed latency on the i9. The 150 instructions/event and simulated mispredictions can motivate cheaper events; they do not establish a universal 40-instruction threshold or an input-independent n/r=80 crossover. Preserve those as prospective heuristics. Closing this particular arm is reasonable; no proof closes all room-state constructions or the run-fed line covered by PROTOCOL §11.

## 1. The decisive counterexample: the baseline changes the bottleneck

The registration concedes that two hogs reserve a physical core but says this cancels in ratios. It need not.

Let the original decoder saturate W=28.5 GB/s with β=19 bytes/decoded byte, hence T=1.5 GB/s. Suppose losing a critical core makes CPU supply cap the hog-spin baseline at 1.0 GB/s. Adding H=8 GB/s still leaves (28.5−8)/19=1.079 GB/s of memory capacity, above that CPU ceiling. The decoder remains at 1.0, giving r=1, passing the power floor and a stable one-thread control. The reporter says NOT BOUND even though the original shipped execution was bandwidth-bound.

This is a mathematical non-identification witness, not a prediction of a 33% core-removal loss on this i9. The proposed controls do not exclude it. Add a no-hog baseline and distinguish the shipped system from the resource-partitioned system.

Conversely, a multicore workload may lose 25% through scheduling or package power while a one-thread workload stays above 0.90. H≥8, r10=0.75 and r1=0.98 are compatible with the BOUND label without DRAM saturation causing the loss. A stable serial control is not a matched multicore frequency/SMT control; a falling serial control can itself be memory interference.

Continuous occupancy is not constant interference: level zero spins, level 100 executes AVX stores and fences. The instruction mix and shared-resource demand change. Non-temporal stores do not establish “nothing else” is perturbed. Moreover, only hogs are pinned: no decoder affinity excludes their logical CPUs or siblings. A busy CPU is not a scheduler exclusion.

## 2. What the thresholds can actually say

Under the strict model T0=W/β and TH=(W−H)/β, r=1−H/W. At W=28.5 and H=8, r≈0.719. Thus the proposed 0.80/0.92 thresholds have a reasonable separation only conditional on fixed shared capacity, fixed traffic amplification, genuine overlap, unchanged CPU supply and comparable read/write arbitration. The old inversion-harness bandwidth does not calibrate those assumptions for this decoder.

H is achieved throughput under contention, not an independently assigned capacity subtraction. Decoder slowdown can free bandwidth for the hog; a low achieved H can be a consequence of strong decoder competition. UNDERPOWERED may describe insufficient intervention, but it cannot certify an absence of binding. Duty level is the assigned intervention; report H as a response too.

A NOT BOUND observation could reject a specified fixed-W/fixed-β saturation model for the tested resource partition and achieved dose, provided the nuisance assumptions hold. It cannot establish latency/occupancy as the cause, exclude write-traffic optimisation, or close §5. BOUND establishes sensitivity to this intervention, not that the simulator's function ranking is causally correct.

Use descriptive labels such as LARGE LOSS / SMALL LOSS / INTERMEDIATE, retain per-round results and uncertainty, and pre-register the model predictions and falsifiers separately. Five-round medians and extrema are descriptive, not confidence bounds. No repeat-to-label.

## 3. Simulator ledger: useful hypotheses, not guaranteed bounds

The archived sim-pipe3-20.txt supports 19,555,532 write misses and the displayed function shares. It contains read totals 3,696,341 and 3,656,744 in different summaries; reconcile the event scope before calling both cold reads. The arithmetic (3.696+2×19.556)M×64/209.700M≈13.06 is correct under the assumed read-for-ownership plus writeback model, not a DRAM measurement.

Callgrind's [manual](https://valgrind.org/docs/manual/cl-manual.html) says its cache simulation derives from Cachegrind; the [Cachegrind manual](https://valgrind.org/docs/manual/cg-manual.html) warns that these basic simulations need not reflect modern hardware. Simulated misses and dirty writeback traffic are different quantities. One write miss need not establish one eventual DRAM writeback, and thread slicing can alter both read and write residency. Neither “reads are lower bounds” nor “writes are usable independent of slicing” follows as a general bound.

Keep the ledger as a candidate generator. Archive the full raw profile, simulator version/options, binary identity and aggregation instead of only selected annotation excerpts. A large function share does not price a transformation's induced work, overlap or new traffic. The present data do not justify closing or ordering all candidates by those percentages alone.

## 4. Harness and admission

Source findings in examples/bw_titrate.rs:
- STOP is set only after the normal measurement loop. A decode panic inside std::thread::scope unwinds into joining hogs which still see STOP=false. The recipe has no timeout: the child and hence bundling can wait indefinitely. Arrange shutdown on all exits, join/check workers, and impose a bounded process deadline. This is a source-level failure path, not a reproduced native hang.
- Startup and level transitions use sleeps, not ready/acknowledgement barriers. A hog can fail pinning or start late while measurements proceed. Counters are published once per period; record the measurement overlap and bound edge error.
- hog_cpus stops at the first missing CPU directory and never intersects the process's allowed CPU mask. Missing topology silently allows unpinned hogs. Refuse unknown topology/placement rather than treating it as the registered intervention; verify achieved masks and actual decoder plan.
- The AVX fill has evident alignment/bounds structure: 32-byte aligned allocation, 256 KiB chunks dividing 64 MiB, 32-byte store steps, runtime AVX check. That is a useful beginning, not completion of B5. Provide the scoped safety argument and relevant adversarial tests for fill and affinity; unsupported instrumentation coverage must be stated.

Reporter: all three repository tests pass by direct invocation. NaN/float/type and inconsistent-size rejection are improved. Two additional fixtures still exit zero and produce BOUND: negative bytes in every threads=0 cell; and every decode cell uniformly changed to one byte. The latter shows internal consistency is not expected-workload identity. The harness compares against its own decoded want but does not emit that reference or REPS for independent admission. Require zero bytes for hog-only cells, externally anchored expected decoded size×reps, and producer completion/placement facts. Retain positive controls.

The frozen measured files match the reviewed head: compare with 6aff22e4388226dc56e61b84506e56e31377fec5 adds the freeze pointer, ledger excerpts and unrelated run_front example, not this recipe's measured inputs. A revision will need a new freeze. No native build, timing, corpus run, successful dry run or green-CI claim is made here.

## 5. Next experiment's value

Expected: added memory traffic reduces decoding throughput in the fixed-capacity model. Observed: no i9 result; local smoke evidence is not that target's diagnosis. No production gap has closed. Learned: the perturbation combines bandwidth, placement and CPU/power effects; the current labels cannot distinguish them.

Before a workstation slot, fix bounded shutdown and admission, then choose a narrower experiment:
1. record no-hog and spin-hog baselines;
2. isolate verified cores for hogs and decoder, while recording the actual plan and acknowledging the changed partition;
3. add a multicore non-streaming control matched as closely as practical to the instruction/power load, or explicitly retain that uncertainty;
4. report response versus assigned duty and achieved traffic without declaring a unique cause;
5. test a concrete cold-write reduction in a separate registered matched comparison if its whole-cost prediction warrants it.

This retains useful information value without making counters a prerequisite. The experiment can narrow mechanisms; direct candidate A/B evidence remains the consequential test. No NOT BOUND line closure is supported.

The new authenticated C6 delegates prospective design, acceptance and adoption under review. The old pin objection is therefore removed for future registrations; C2, B5, Guardian admissibility and Louis's C5 workstation execution remain. Research only committed here. Routing audit found no new violations. Replay probe.py beside bw_titrate.py and test_bw_titrate.py at the reviewed SHA; probe-result.json records the results.
