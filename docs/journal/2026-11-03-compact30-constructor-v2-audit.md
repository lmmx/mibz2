# Compact30: constructor v2's checkpoint (report 209) and its independent audit (report 210)

**Constructor v2 passes its correctness checkpoint: PASS.**
- **Report 209:** the guardian reviewed the prototype at `8beffe2` and its journal at
  `cd14fa9` ([the prototype report](2026-11-03-compact30-constructor-v2-prototype.md)),
  and provisionally accepted its compiled structure. The guardian then asked the researcher
  for an independent source audit, and authorised no timing.
- **Report 210:** the researcher audited all 1,767 lines of
  `examples/c30walk/ctor2.rs` at `8beffe2` against the oracle, constructor v1 and the
  generated assembly. No material correctness or memory-safety defect was found for the
  baseline 64-bit x86-64 research build.

**What this does not authorise.** The PASS makes v2 eligible for the guardian to consider a
separately registered construction-cost experiment, and only that. No timing, i9 run,
harness adaptation, protocol amendment or performance claim is authorised. The prototype
and its evidence are kept unchanged at `8beffe2` and `cd14fa9`.

**The audit's evidence** is kept in
[2026-11-03-compact30-constructor-v2-audit/](2026-11-03-compact30-constructor-v2-audit/)
as received:

| file | what it is | sha256 |
|---|---|---|
| `INDEPENDENT_CTOR2_AUDIT.md` | the full audit: six areas, line-specific arguments, test evidence, residual limits, disposition | `6ea3883c…` |

The researcher's summary also lists an independent Python model (`epoch_oracle_checks.py`)
and a complete evidence bundle. Those weren't relayed here, so only the audit report is
archived. Its §3 records the model's coverage:
- 1,110 input configurations and 1,422 snapshots;
- 708,726 ordinary groups, 863 wide groups and 6,483 crossing groups.

These checks are of the mathematics, not executions of the Rust source.

**Scope and limits, as the audit states them:**
- It is a source-level argument for the pinned source on 64-bit x86-64 with the pinned
  rustc 1.97.0 listing. It is not a formal safety proof, and it isn't Miri-complete.
- Miri ended after 473 cases in five classes, stopped externally during the sixth. Within
  that, it reached crossing RAW, an epoch boundary and wide RAW above 61,440 rows. It did
  not reach:
  - the exact 65,533/65,534 span cases;
  - forced RAW;
  - malformed counts;
  - the larger reuse and poisoned cases.

  Release tests are not a memory-safety sanitiser.

**Nonblocking findings** (none changes the prototype):
1. **Low, harness gap:** poisoning covers existing spare capacity only. When a build grows
   `rec`, the new allocation isn't poisoned. Exact oracle comparison remains the evidence
   for those cases.
2. **Low, invariant fragility:** `Desc::rank`'s pointer difference is not defensive against
   corrupted descriptors. Its safety follows from the descriptor-state invariant traced in
   §1, not from a dynamic check in `build::<false>`.
3. **Low, carried from v1:** the unchanged canonicaliser relies on unique provisional RAW
   indices. v2's creation paths give each RAW group a fresh `raw.push` index, and no group
   can be promoted twice. The debug-only duplicate check isn't that proof.
4. **Platform qualification:** 64-bit x86-64 and the pinned toolchain only. Another listing
   must be verified afresh.

**The open question is economic, not about correctness.** Per input row, v2 makes about twice
v1's 16-bit field stores, 0.509 against 0.252. It loads far more fields, 0.274 against
0.002. It runs about 1.72 M ordinary-group normalisations on the primary corpus. Whether
that deferred work costs less than the header, RAW and span chain removed from the scalar
append (49 → 12 static instructions) has not been measured.

## Report 209: the guardian's decision, as relayed

Kept as relayed. The relay lost the formatting, so the headings, paragraph breaks and the
figures table are restored. The words are unchanged.

Guardian decision: constructor v2 passes the preliminary correctness-and-compilation checkpoint. I authorize an independent source audit, but no performance benchmark yet.

I reviewed the implementation at `8beffe2`, the journal at `cd14fa9`, and the generated assembly excerpts. The prototype has achieved what we asked it to demonstrate.

The result is particularly encouraging because the new algorithm removes substantial work from the scalar path rather than merely optimizing the old constructor.

### What v2 has demonstrated

| | | |
|---|--:|---|
| Scalar path | 49 → 12 | Static instructions per iteration, v1 → v2 |
| Bulk coverage | 74.31% | Essentially preserved from v1 |
| Synthetic cases | 53,924 | Reported exact correctness |

The machine-code evidence confirms that the scalar append no longer repeatedly loads record headers, checks RAW status or calculates and checks offsets. Source-coordinate updates are amortized over tiles, and full-group normalization uses bounded SIMD operations confined to the record's 60 field bytes.

The existing bulk emitter is also preserved.

This is a genuine architectural improvement in compiled structure. It does not yet establish faster construction.

### The remaining performance risk

V2 performs approximately twice as many explicit 16-bit field stores as v1 and substantially more field loads. It also executes about 1.72 million ordinary-group normalizations on the primary corpus.

Those operations could consume much of the instruction savings, particularly if records leave L1 cache before finalization.

The previous invalid capture still gives us a demanding directional target: about 19% less construction CPU time and 25% less completion time than v1. No v2 timing evidence exists yet.

### Why independent review is the right next step

I would not require Claude to repeat the entire 1-hour-35-minute Miri attempt. The partial run covered important epoch and wide-group paths, and the release suites exercised the remaining cases.

But v2 introduces new unsafe-pointer and initialization invariants. Before we consider measuring it, an independent reviewer should verify that those invariants really follow from the source.

The most important questions are whether each pending portion is finalized exactly once, whether `Desc::rank()` and the raw field pointers remain valid, and whether crossing RAW portions can overwrite one another.

### Send this to the researcher

Please perform an independent source-level correctness and memory-safety audit of compact30 constructor v2.

Source to audit:
https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs

Prototype report and evidence:
https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype.md

Repository: `lmmx/mibz2`

Implementation commit: `8beffe2`

Research report commit: `cd14fa9`

Guardian status: The correctness-and-compilation prototype has provisionally passed its architectural checkpoint. No timing is authorized. I want an independent audit before deciding whether the implementation is safe and correct enough for a separate construction-cost comparison.

The prototype reports byte-identical `rec` and `raw` arrays against the frozen oracle on 11 corpora, adversarial inputs and 53,924 synthetic cases. Its compiled scalar append is reduced from the previous constructor's 49-instruction path to 12 instructions, while retaining approximately 74.31% bulk-path coverage.

Please verify the claims against the actual Rust source rather than relying on the journal.

Priority areas:

1. Unsafe initialization and pointers. Trace `Desc`, its raw write pointers, `rank()`, `open()`, `close()`, `refill()`, and the record arena through `set_len`. Establish whether every read and write is valid on accepted, rejected and reused builds.
2. Deferred finalization. Verify that every partially populated portion is finalized exactly once, including portions pending until end of input. Confirm that finished bulk records are never normalized again.
3. Epoch reconstruction. Independently verify snapshots, duplicate snapshot values, absent characters, source labels crossing 61,440-row boundaries, and reconstruction of full successor values.
4. RAW correctness. Verify crossing-group ownership, wide-group classification using full labels, forced RAW and canonical ordering. Include the exact 65,533/65,534 span boundary and much wider spans whose low 16 bits appear small.
5. SIMD and memory safety. Check that full-group normalization reads and writes only bytes 0–59, never touches the potentially uninitialized header at byte 60, and never subtracts an overlapping field twice. Inspect both source and assembly.
6. Harness blind spots. Evaluate whether the tests could overlook uninitialized fields, malformed-count failures, allocation reuse, incorrect epoch assignments or shared-oracle mistakes.

The existing Miri run completed 473 cases without reporting undefined behavior, including epoch and wide-RAW paths, but was interrupted before finishing. Treat its coverage accurately. Do not require another lengthy Miri run merely for completeness; use targeted independent checks where they materially improve confidence.

Please also inspect the generated scalar-append and finalization paths for consistency with the claimed structure. Do not infer runtime speed from instruction counts.

Return a clear PASS, CONDITIONAL PASS or HOLD, with source locations, triggering conditions and severity for any findings. Distinguish demonstrated defects, incomplete proof and missing test coverage.

No repository modifications, performance measurements, i9 work, constructor redesign or walker changes are authorized by this review.

That's the only new assignment I recommend. Claude should preserve the prototype and evidence without making further changes, and Astra has completed its current design work.

Once the independent audit returns, we can decide whether v2 merits adapting the construction-cost harness. If it does, we'll measure v2 against production—not assume it wins because the scalar loop is shorter.

Current position: v1 remains STOP; v2 is promising and provisionally accepted, pending independent code audit.

## Report 210: the researcher's audit summary, as relayed

Kept as relayed. Four formatting repairs, with the words unchanged:
- the audit-results table, which the relay had broken, is rebuilt;
- the three independent-check figures are put in a table;
- link fragments split across one URL are joined;
- tracking parameters are removed from the links.

The "Files" list names the researcher's attachments. Only `INDEPENDENT_CTOR2_AUDIT.md` was
received.

PASS — independent source audit

I completed the review of all 1,767 substantive lines of [`ctor2.rs` at `8beffe2`](https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs), the prototype report at `cd14fa9`, the frozen oracle and constructor v1, the correctness harness, and the generated assembly.

No material correctness or memory-safety defect was found.

My recommendation is that constructor v2 is sufficiently supported by source-level reasoning and correctness evidence to proceed to the design of a separately registered construction-cost experiment.

That is not authorization to benchmark. Nor is it a claim that v2 will be economically successful.

### Audit results

| Area | Source-level finding | Disposition |
|---|---|---|
| Unsafe initialization | Every destination slot on an accepted build is written before publication. Ordinary headers are installed during finalization; RAW records are fully initialized. `set_len` occurs only after finalization and padding. | PASS |
| Descriptor pointers | `open()` bounds each descriptor to one character-owned portion of one record. `one()` and `fill()` stay within that range. No pointer into the reallocatable RAW arena is retained. | PASS |
| Deferred finalization | Each pending portion is finalized exactly once. Bulk-emitted records bypass normalization. End-of-input finalization covers portions awaiting another occurrence. | PASS |
| Epoch reconstruction | Snapshots include pending occurrences; the greatest-qualifying-epoch rule handles repeated snapshots and absent characters. Full successors are recovered correctly across boundaries. | PASS |
| RAW correctness | Crossing portions occupy disjoint field ranges. Wide classification uses full labels. Provisional RAW indices are unique by construction, satisfying the unchanged canonicalizer's precondition. | PASS |
| SIMD normalization | The compiled full-group path accesses exactly the 60 field bytes, without reading or overwriting the header. Partial groups use bounded accesses. | PASS |

#### The most important source-level proof

The central obligation is not simply that the destination pointers remain inside their allocations. It is that every field containing a temporary successor label is finalized before the `Rec` vector becomes visible.

The relevant paths are [`close`, `refill` and `finish`, lines 369–495](https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs#L369-L495).

`close()` can finalize an open portion only when that portion is full. I traced all three circumstances that lead there.

In the scalar path, `refill()` is entered when the current write pointer reaches its end. In the run path, a portion is closed only if the run still has rows after consuming all the available space. Finally, at end of input, the constructor first verifies that every character's destination rank has reached its bucket limit. Since a portion ends at either its group boundary or its bucket limit, a remaining open portion must then be complete.

That establishes the condition which is only a `debug_assert!` in the uncounted release build.

Once a portion is finished, `close()` discards its pointers and advances its stored rank. The source offers no path to finalize it again.

The same argument covers malformed counts: an overflowing bucket is rejected before its next write, and a short bucket causes rejection before finalization and `set_len`. The partially written record allocation retains length zero.

### Independent checks

I also wrote a separate Python model of the epoch reconstruction and record arithmetic, using stable counting scatter as its oracle.

| | |
|---|--:|
| Independent inputs | 1,110 |
| Wide groups checked | 863 |
| Crossing groups checked | 6,483 |

The model covered the maximum block size of 983,010 rows, repeated epoch snapshots, characters absent across as many as thirteen epochs, the exact 65,533/65,534 distinction, and much wider spans whose low 16 bits misleadingly suggest a small offset.

Every reconstructed successor and classified group matched the independent oracle.

These are mathematical state-model checks, not Rust execution or memory-safety verification.

#### Assembly verification

I checked the archived assembly directly, including the full-group normalization at `61f74–61fb3`.

The five non-overlapping normalization regions

The first three operations cover 16 bytes each, followed by an eight-byte operation and a four-byte operation. Nothing in the vector normalizer reaches byte 60, where the header begins.

The scalar append's quoted 12-instruction steady-state path is also present and lacks v1's repeated header load, RAW test and span calculation.

This confirms the architectural improvement in the generated code. It says nothing about whether the additional group finalization pays off at runtime.

### Evidence limits and minor observations

I found no issue requiring repair before the next research gate, but three limitations should accompany the PASS.

Miri was interrupted. The archived log shows 473 completed cases before an external stop, not a successful 524-case Miri run. It exercised crossing RAW, epoch boundaries and some wide-group reconstruction beyond 61,440 rows. It did not finish the exact span cases, forced RAW, malformed counts or all reuse cases.

Poison coverage is incomplete on allocation growth. The harness alternates two poison patterns, which improves its ability to detect omitted writes. But when an allocation must grow, the newly allocated records are not necessarily poisoned. The byte-exact oracle checks and source proof remain the stronger evidence in those cases.

The unchanged canonicalizer still relies on index uniqueness. That invariant is justified by v2's RAW-creation paths: each new RAW group receives a fresh index, and a group cannot be promoted twice. Its debug-only checks are not, by themselves, proof of that property.

None of these represents a demonstrated source-level defect.

### Files

Full independent source audit

Line-specific proofs, test evidence, unresolved assumptions and disposition

Independent epoch and record checks

Standalone non-timing Python model

Complete audit evidence bundle

Report, model and recorded check results

### Final disposition

PASS — constructor v2 at `8beffe2` has no material source-level issue identified, and is suitable for the Guardian to consider the next experimental gate.

The architecture is meaningfully different from v1, and the correctness evidence supports that difference. It avoids repeated classification along the scalar append path while preserving the exact compact30 representation and walker.

The unresolved question is economic, not a correctness defect: whether deferred normalization and its associated memory traffic cost less than the work removed from the append path.

I recommend preserving this source pin and proceeding only to a separately approved construction-cost protocol. The previous v1 STOP and the validated compact30 traversal ADVANCE remain unchanged.

No repository changes, Rust performance runs or i9 work were performed.
