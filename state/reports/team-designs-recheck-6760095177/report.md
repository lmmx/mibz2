# Revised team-screen and native engagement designs

Trigger: [6760095177](https://github.com/lmmx/mibz2/pull/162#issuecomment-6760095177).
Reviewed #334 at **25b778cdd44c2bbf5365fd16f990da5a52a72ebf**.
Authority: SPEC and SPEC.RESEARCHER at a2da49638ac54d7763de1c167bd054f67ffdc763.

**Screen: REVISE before adoption/freeze. Native diagnostic: REVISE before registration.**
The two-tier direction and fixed native sequence are sound. Remaining objections concern evidence admission, not a demand for more measurement or another performance gate. Neither verdict authorizes execution or implementation by the Researcher.

## Screen: demonstrated admission defects

The pinned 24-test suite passes locally. It contains positive and negative cases; calling all 24 “negative controls” overstates that evidence. Four additional synthetic cases each return the empty failure list, i.e. PASS:

| Modification of otherwise accepted records | Result |
|---|---|
| One wrapper's build label becomes unrelated | admitted |
| A different well-formed executable digest for every fixture/arm | admitted |
| Every valid-input record reports BlockCrc at block 99, with empty error text | admitted |
| All malformed-input children exit 124 while retaining expected error records | admitted |

The attached probe imports the exact checker and its synthetic fixture constructors. These are checker counterexamples, not decoder observations.

Bind records to a run-level build artifact manifest: source, actual executable digest, selected child command, thread count, input digest, and environment. Merely validating digest syntax does not establish that two fixtures used the same built executable. Validate the build label; require null error kind/block on success. Fix the expected process-exit contract for malformed inputs and reject timeout/collector failures independently of decoder error JSON.

**Do not fix that last issue by requiring nonzero for an ab decoder error.** The harness records drain's error in JSON and completes its child normally. Its protocol-level decoder failure is distinct from process failure. Freeze the expected normal ab child exit (0 in this implementation); raw lbzip2 rejection separately requires its documented nonzero exit, with timeout and signal excluded.

The outputs checker accepts the synthetic four-arm set without any oracle/rejection records. It currently has no ingestion contract for the seven required fresh oracle decodes or four lbzip2 rejections. A manifest's expected hash is not evidence that those eleven subprocesses ran. Require their identified, bounded completion records in the final driver/checker contract. This is an acknowledged implementation gap, not a claim that a screen already ran incorrectly.

Structured errors now exist in head's ab.rs, which closes the former head-harness objection. Specify how the unchanged merge-base decoder obtains the same observation schema. A separately pinned harness adapter may expose it without changing decoder semantics; that is a declared harness arrangement, not an unmodified historical executable. Do not silently patch the base kernel for profile attribution.

T1 alone establishes output/error regression coverage; it cannot establish that the intended kernels executed. T2's source counters need implementation, counter-placement review, and negative controls before claiming semantic coverage. Ordered input classes plus aggregate profiles establish fixture adjacency and aggregate execution, not an ordered per-block execution trace. Keep that distinction in the full-tier coverage claim.

## Answers to the four questions

1. **Add streaming SHA-256 plus length** to the untimed correctness stream checks, for each arm and oracle. Keep the old checksum for compatibility if useful. SHA-256 is strong practical identity, not a logical proof of byte equality; a direct streamed byte comparison is preferable where strict byte-exact differential evidence is intended. This does not require altering the timed acceptance benchmark or adding an acceptance row.

2. **Retain a precise, schedule-independent prefix contract for these fixtures.** The source supplies its basis: parallel.rs read drains the current block before taking ready[st.taken], and publishes errors in that same ordered sequence. finish_block checks CRC before publishing successful output; scan_finish appends the scanner error after issued blocks. Thus, absent cancellation or an unrelated earlier error, all successful blocks preceding the failed block are emitted, and no bytes from the failed or later blocks are emitted. ab.rs drain hashes each successful read before observing the error. Scheduling can change when the error is available, not this ordered prefix. Freeze the actual error index/kind and complete-prefix bytes after validating each structural mutation; an arbitrary midpoint truncation is not by itself proof of its eventual error classification. Include different read-buffer sizes and completion orders in functional evidence. The inspected source establishes the intended invariant, not execution proof for all paths.

3. **Prefer a purpose-built small-block stream with adjacent S,S**, generated and decoded through a trusted fixture construction path. Classify S/T by the relevant internal block size, not merely decoded output length. Freeze actual block sequence, stream boundaries, digest and oracle output. Dropping a T block from W at byte boundaries is invalid for bit-aligned blocks and changes combined CRC; correct bitstream reconstruction is possible but needless fixture machinery. Concatenating two one-block streams only demonstrates S→S across a stream boundary unless explicitly required. Price any added fixture: the existing 44-child count then changes.

4. **Recommend the two-tier design**, with uninstrumented T1 plus implemented semantic T2 on affected PRs and full matched profiles for the specified broader/diagnostic uses. Do not represent unavailable T2 as present coverage, or turn UNATTRIBUTED into zero. This recommendation is not a Guardian ruling on D083 and does not remove S2's whole-decoder accounting obligation for default-kernel changes. Resolve the exact prospective trigger mapping under C6; no inference that every source edit needs a full matched profile, and no inference that T1 discharges every count obligation.

## Native diagnostic: a small but material validity repair

Accepted: explicit team/block/thread identities; leader and helper ledgers; helper-only per-team integer balance with positive denominator; old-arm walker analogue; old then staged once; first invalid terminates INCONCLUSIVE; descriptive per-block distributions; preservation of FAIL-MEASUREMENT and no native/Callgrind splicing.

**Completeness must be checked independently of bytes and the terminal line count.** Consider a helper walker that immediately sees an empty queue: zero successful claims, zero bytes, one exhausting claim. Omit that walker record and decrement a terminal count derived from the serialized records. Byte and successful-claim conservation still hold. “Exhausting claims equal walker calls” also holds if both are derived from the remaining walker records. Predicate (a) now gives NOT ENGAGED, although the proper verdict is INVALID missing evidence. A missing zero-byte leader call can similarly erase predicate (c)'s witness.

The draft already records “walkers counted in” in its independent block summary: require that count to equal the number of completed walker-call records, with an independently justified membership/entry witness for each caller. Record legitimate zero-work calls explicitly. Test deletion of a zero-work helper and leader record while adjusting the terminal count. Separately test a valid execution with genuinely absent required engagement: that is NOT ENGAGED, not INVALID. Distinguishing no call from a lost call is the scientific requirement; do not define all predicate failures as invalid.

Clarify §3.7: **209,697,662 is cooperative walked-byte M**, not all ledger rows indiscriminately. Solo records have separate accounting. Sum the cooperative walker records against M and retain the solo coverage checks without adding solo bytes twice.

The pinned draft explicitly leaves the instrument, checker and source/toolchain/checker pins to be built/filled. Bounds are specified; executable readiness is not. Before registration, present their exact versions, synthetic ledger rejection evidence, feature-on/off byte equivalence and applicable independent B5 evidence. The source review of traverse_staged does not automatically review future instrumentation. No extra native run is requested.

“@9 oversubscribes the runner” is conditional on the recorded allocation/quota; ubuntu-latest is not a fixed topology contract. The fixed runner design is still appropriate. The brief's “Guardian confirms” is not itself a cited confirmation: the draft and Coordinator log say confirmation is due before registration. Obtain/reference that ruling rather than infer it.

## Evidence and next value

Expected: revision 2 would close the original admission and ledger ambiguities. Observed: most architectural repairs are present, but four residual checker false passes and an unclosed zero-work-record completeness case remain. The gap closed is design clarity, not measured engagement or speed. The mechanism learned is that equality and byte conservation cannot detect missing zero-contribution evidence.

Next useful work is the bounded contract/checker repair with falsifiers above, then exact-head review. More decoder measurements before those repairs add little information. No decoder build, screen, native diagnostic, raw-profile replay, timing, performance qualification, merge or B5 discharge was performed here.

Reproduction: place the attached source snapshots under bench/ as archived and run python probe.py; the four malformed cases are asserted to be admitted. The source manifest pins all inspected local snapshots. The existing suite was run separately with 24 tests passing.

