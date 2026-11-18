# D113 — PR581 one-scan writer: source review supported; B5 completion HOLD

18 November 2026. Incoming [6988393925](https://github.com/lmmx/mibz2/pull/161#issuecomment-6988393925).
Reviewed head **cd7e5de2b7c61442f7a391436eee910a5ebb126a**, base 50612a8f7b141e4999645669cd67dce95c2dc0ad.
Authority: SPEC/SPEC.GUARDIAN, MODE, RULES and PROTOCOL at a2da49638ac54d7763de1c167bd054f67ffdc763.
Independent reviewer: Guardian; no involvement in implementing this candidate.

## Disposition

**G1:** the full-period ownership schedule and its translation into this source are supported. No byte-error or unsafe-access counterexample found in the unmutated writer.
**G2 HOLD:** integration of this new unsafe writer until its B5 evidence is complete: a completed Miri receipt covering the new path, and identifiable runtime failures for the eight claimed mutation kills. This is an evidence HOLD on #581, not a line STOP. Existing writer work and correctness testing continue. No performance or adoption ruling; paired bench is separate.

The mutation summary alone does not establish eight runtime kills. The submitted script treats *any* nonzero cargo exit as a kill, suppresses both captured logs, and does not first establish a passing baseline. I reproduced it printing **8 of 8 mutations caught**, exit 0, with all sixteen subprocess calls returning a simulated dependency/build error and **no tests executed**. That demonstrates an unsound scorer, not that the implementer's actual runs failed to compile. Preserve the actual successful builds/test selection and each mutant's runtime failure (or rerun with those checks); a code rewrite is not required if adequate original receipts exist.

Miri run 44374581630/job 135050178989 was in progress at the inspected snapshot. Its default Miri step was running and the feature step pending; job-log retrieval returned BlobNotFound at 18:58:37 UTC. The PR's local-Miri assertion is not an inspected completed receipt. Partial coverage is not PASS (B7). No polling or rerun was requested.

## Adversarial source findings

1. **Every scan store stays behind consumption.** The loop reads both original descriptors into a,b before storing any command. Before iteration j, tail >= j+1: at most one command per previously visited parent. A permitted store decrements tail only when tail>j, so its 16 bytes belong to record j or a higher record. The next iteration reads j-1, untouched. The code sets na,nb from the saved a,b; it never reloads overwritten record j+1. Position-table writes are in dead top slots or a separate Vec, outside rule and literal storage. Literal output writes are in the separate output allocation. The initial marked walk finishes before any command store. These are all scan-time store classes.

2. **Owner certificate has the necessary identity premise.** A surviving operand stays on the same front; its next cut necessarily stores a rule if the operand itself is a stored rule. Descriptor equality on that same side therefore means survival. An exhausted identity cannot reappear. Distinct stored identities, including short marked rules, are preserved by this change. At full period all mass reaches the marked root, so its owner edges cover every stored rule. These statements are not true for arbitrary DAGs or interned equal spellings.

3. **Deferred sources are ready in increasing parent order.** Any hole in a selected child's occurrence belongs to that child or a descendant, whose birth index is earlier than the current parent. Commands compact from the high end while scanning descending parents; reading tail..nr restores increasing-parent order. Short writes and deferred destinations partition the period. Occurrences of the same positive-length rule are disjoint: overlapping occurrences would be nested and imply a strict self-descendant, impossible under creation order. Later-address sources are allowed. copy_long uses ptr::copy, which also permits overlap; completeness, not memcpy direction, is the substantive obligation. My reversed-command mutant reaches an unfinished source on a source-text BWT.

4. **Guard and wrap are exact.** try_invert establishes MARK, 0<period<=n and n<2^24. write_owned additionally requires period=n, nonempty rule array and the root at its last record. All rejected cases enter unchanged write_cells before destructive writes. Period-one and one-byte roots therefore retain their old path. Children have positive lengths shorter than their parent, hence q+left_length<2p; one subtraction is sufficient without u32 overflow. Only the marked spine wraps. SHIFT=1 puts the marked singleton at p-1 and leaves unmarked factors contiguous. The 35-row later-source witness is admitted; the 36-row, period-18 witness is rejected. Both primary positions and all rows were exercised in the model. The existing exhaustive tests explicitly count full-period admission on eager and lazy kernels.

5. **Unsafe bounds and aliasing.** The mark walk and pos indices inherit the existing induction's valid earlier-rule descriptor contract. Literal descriptors inherit length<=16 and readable arena storage; their destinations below p plus the n+32 output allocation bound copy_small. The copies' subtraction-form guards avoid overflow and reject NIL/marked lengths before pointer formation. The command's descriptor is an original valid rule reference, so its pos index is valid. Commands use unaligned 64-bit reads/writes; positions_table establishes four-byte alignment and capacity. Reallocation of out occurs after induction, when its bottom slots are dead. There are no concurrent users of these per-block buffers. This is a conditional proof over valid induction records, not protection against arbitrary corrupt descriptors: the prose saying every added access is locally checked should make that inherited premise explicit. CRC is not the proof of byte-exactness or memory safety.

6. **Research-path instrumentation note, not a default-path defect.** write_owned replaces write_cells for memory-lever variants as well. The test-only PENDING-before-writer assertion remains only in write_cells; the new path lacks that exact observation point. The unchanged staged Fence still drops before return, and the dedicated exit test checks cleared state and fence counts, so this review does not infer a missing production fence. Preserve the reader-entry assertion in the shared entry or new writer when maintaining that research model.

## Reproduction and coverage

The adjacent [model](d113-model.py) is a separately implemented explicit two-order scalar induction with a byte-addressed destructive writer; dense LF is the output oracle. It asserts true last-parent ownership, unread-rule preservation, one command per parent, assigned positions, destination partition, source completeness and source/destination disjointness. It is not execution of the Rust code and does not independently implement both eager and lazy storage kernels.

[Results](d113-model.json):
- 41,331 input/primary cases classified: binary columns through 10, ternary through 7, all rows of both 35/36-row witnesses, 1,500 deterministic random columns, and two 12,000-byte source-derived BWTs.
- **5,249 owned executions** matched LF; **36,082 fallback selections** were checked only for the guard. The model does not execute write_cells on those fallbacks.
- 8,440 wraps, 41 deferred copies, including 19 later-address copies; one command store reached the next-record slot.
- Reversing command order fails source completeness on the 12,000-byte cells_owned.rs prefix. A mutant re-reading both later descriptors at the start of the iteration was not killed by this bounded suite; this is not claimed as a test of every unsafe re-read ordering. The actual code carries descriptors and its store-order proof above is what is accepted.

To reproduce, place the reviewed head's src/cells.rs and src/cells_owned.rs beside the model as cells.rs and cells_owned.rs, then run python3 d113-model.py. For the scorer issue, run [d113-mutation-audit.py](d113-mutation-audit.py) beside the same cells_owned.rs and the verbatim submitted [d113-mutator.py](d113-mutator.py). The subprocess responses in that scorer test are deliberately simulated; no Rust tests execute there.

No cargo/rustc/rustup is available in this review environment. Native tests/Miri were not independently run here.

**CI inspected:** exact-head Test run 44374581551: native Tests step succeeded (18:53:51–18:57:34 UTC); remaining job steps were still running. Completed fuzz-smoke job 135050179518 log: 23,964 Cells cases, seed 42, production admitted 611 and forced admitted 12,611; short-cycle counts 610 and 8,874. Feature fuzz also completed (18,522 cases). This is genuine completed native fuzz evidence, not a green overall CI claim. The log lacks a dedicated owned_taken count; new-path reach is supported by source/guard and targeted tests rather than asserted from the fuzz label alone.

Tests at the pinned head cover exhaustive binary/ternary columns, periodic witnesses, long/reused allocations, complemented output and nested source-text copies. The new Miri test reaches eager/lazy owned calls and a smaller reused allocation; the old long-writer tests explicitly retain fallback-writer coverage. The eight mutation targets are relevant, especially nested copy ordering after the original survivor. Their receipt classification remains the gap above.

Evidence read: actual changed source/tests, journal and mutation script at reviewed head; study 5's proof and model; Researcher REVIEW 6986491194 and MONODROMY REVIEW 6986559098, used as cross-checks, not substitutes for this review. Timing claims were not validated.

## Lift condition and publication

Provide completed Miri evidence for this exact source/test scope (or establish source identity to a completed run), plus successful unmutated test selection and concrete runtime diagnostic/test failures for each claimed mutation. Then lift this scoped HOLD; a new algorithm or broader review is not required unless evidence changes the source or reveals a defect. Preserve the earlier missed nested-copy mutation in history.

Migration old-branch head and manifest SHA-256 were independently checked before receipt. Inbox reconciliation found only this unfinished assignment. Routing audit from this event is empty; route(coordinator,REVIEW) returns #145. One REVIEW will cite this incoming ID and immutable decision. Automation remains unchanged.
