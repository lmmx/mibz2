# D112 — production lazy dispatch, PR452
Incoming 6904418769, PR161. Reviewed head 6d97806ce32fb72db81c11a67d39140ade0caba5.
Operating instructions: docs/team SPEC, SPEC.GUARDIAN, MODE, RULES and PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763.

## Decision
G1 PROCEED for the requested B5 source/reachability review. No new G2 HOLD. D110's completed safety evidence carries for the unchanged lazy implementation; this review supplies the new dispatch analysis. This is not a declaration that the new head's CI is green, a performance verdict, or permission for workstation execution.
D111 remains recorded on PR449: this head does not contain its output-spare-capacity implementation. It does not inherit that HOLD merely by sharing the word lazy.

## Independently reproduced identity
Extracted Rust function text from this head and D110's reviewed 8de6ce01bcbf8aa79fc9d4392c46bda14d77e6a7:
- census, slots_lazy, induce_lazy, first_stage: exact equality.
- induce_variant: exact equality after removing only the new cfg(memory-lever) on the staged match arm.
- TAG, ROW, LEN_BITS, Lazy and Census layouts: exact equality.
- Cells attempt, invert_runs, invert_in_place and write_cells: exact equality.
The full cells.rs diff against D110 consists of exposing variant/lazy state and includes, making first_stage/induce_variant common, retaining the lazy canary check without the feature, and moving the test include. try_invert's size/record/budget validation and error handling remain; only its previously feature-dependent dispatch is made common. No new raw access is introduced by that dispatch.

D110's feature isolation was a reachability observation, not a safety precondition of the lazy algorithm. Its opt-in approval did not authorize a production default; this independent review addresses the changed reachability without treating that old scope as an eternal safety ban.
Read back completed Miri job 132988104549: checkout f244e6aaa3615dc21f024e61d9392b73e8560cc4 (D110 source-equivalent merge), 44 base tests completed, then four feature tests completed including lazy closed-letter recycling and lever boundary/exact-fit reuse, through 2026-11-15T23:15:41Z. D110's immutable report also records native, differential, fuzz and mutation closure. Those results are carried by source identity, not renamed as new-head execution.

## 1. Reachability and failed attempts
LazyCells is CellsVariant<LAZY, false>. Default constructs an ordinary CellsKernel and sets only its per-instance variant. The wrapper forwards the InverseBwt methods. The first stage and induction dispatch agree on LAZY; unknown public V values fall through to ordinary Cells in both, while staged is compiled only with memory-lever.
Each worker creates its own Scratch<I>; lazy buffers, pool and variant are not shared across workers. Increasing worker count changes allocation pressure, not aliasing ownership.
The failed-attempt contract here is the original one: lazy induction may overwrite/resize ll before refusing. invert_runs then invokes the identical Segmented fallback, which owns rewriting the output. It is NOT PR449's stronger untouched-ll contract. Existing differential tests compare partial output and exact errors against plain decoding; the new integration test explicitly includes late budget refusal, corrupted CRC and truncation.

## 2. Preconditions are still checked
Plan selects an algorithm, not admission. Production F=false retains budget floor(4n/25), record-density and sparse gates, checked row sum/tiling, valid primary row, nonzero record lengths <=127, n<2^24 and records<=n.
slots_lazy separately rejects n>=2^22 before encoding its entry length, and rejects allocations/pool offsets reaching bit31. Real bzip2 blocks <=900000 satisfy the length representation; larger direct inputs are rejected rather than trusted because eight workers were chosen.
Every refusal still reaches Segmented. Column/in-place operation forwards to the same fallback; forced fuzz behavior F=true is not used by the workers constructor. The algorithm's pool, tagging and rule bounds therefore have the same entry assumptions D110 reviewed.

## 3. Plan and explicit arms
Plan::lazy is exactly Workers(n) with n>=8. planned uses that result to select LazyCells versus Cells; teams do not pass through it. new still applies service/default/team policy first. Thus eight requested threads is not an unconditional lazy promise: the selected plan must actually be workers. On the named i9, workers at 8/9 are lazy; teams/service at larger budgets remain their separate paths. Unknown/small topologies can select lazy workers at >=8; safety is topology independent, performance is not established for every such machine.
The explicit workers(...,lazy) constructor selects either production-admission kernel, not ForcedLazyCells. parallel_workers and the two exact example arm strings call it explicitly. They mutate no global override; ordinary constructors do not inspect bench arm strings. The existing with_plan override remains thread-local and restored by its guard, unchanged. A caller can deliberately request an arm, but it cannot contaminate another decoder's default.

## New test evidence and limitations
Inspected lazy_letters_from_their_thread_count: exact threshold, both decoder types, forced alternatives, defaults under forced plans, and team exclusion. Inspected tests/lazy_workers.rs: six stream classes, 1/4/8/12 threads, both kernels and planned dispatch, bytes and exact errors compared with Segmented; nontrivial admission and budget-refusal counts required.
Independently parsed the nine mutation replacements and verified each occurs exactly once at this source. The table includes an unmutated passing integration baseline and nine kills, eight with named assertions/canaries and one process death for mismatched induction. These are reported executions, not locally rerun mutants; their scope is dispatch, not exhaustive malformed-input proof. Default fuzz now includes LazyCells/ForcedLazyCells and the three lazy Miri tests are in the default selection.
At the check read, new-head Rust/Miri/fuzz/bench/occupancy jobs were still running. No completed new-head pass is asserted. Unlike D111, the unsafe implementation already has independently verified completed evidence at identical function bodies; a fresh CI job need not invalidate it. The stated green-CI merge condition remains to be met.
No Rust compiler was run locally. Reviewer did not implement the candidate. Independent work: exact text/constant comparison, call-path and ownership review, test/mutation source inspection, old completed Miri receipt readback, and routing audit.

## Adoption and records
LOG 6904285481 records a proposed pre-data adoption-rule amendment attributed to Louis. This review is B5-only: it does not independently authenticate that attribution, amend the rules, approve a new acceptance policy, or certify the constant's performance. C6's decision-record/review requirements and C2's no-after-data-relaxation protection remain. The journal at this head still says the i9 head run decides constants, while the later LOG calls the release run confirmatory; reconcile that documentation when recording adoption, without rewriting old outcomes.
A production exposure can increase resident memory and CPU on other topologies even while preserving safety; this verdict supplies no universal speed claim. No merge, readiness tag, workstation run, other branch or automation setting changed.

Migration verified: previous head f9dab3d2b40739d89d613f86e8c2d67bec3159e4; manifest SHA256 60df88f62692909a122e174463e94c012b28e39ab08c6c8a765d41e762a72488. Inbox had only this unfinished checkpoint. Routing audit since D111 incoming comment: no misroutes. Publication belongs on #145.
