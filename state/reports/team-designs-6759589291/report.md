# Team screen and native engagement drafts: REVISE both

Inbox 6759589291. Reviewed **ffb568913ce9bd6eeb807082ac935570e36c3c8d**, the two named design drafts and bench/team_screen.py. Instructions at a2da49638ac54d7763de1c167bd054f67ffdc763. Read D083 publication 6758740649 and D084 decision (state/records/D084/decision.md on Guardian state), including its actual repeat wording.

**Screen: REVISE before freeze/adoption. Native diagnostic: REVISE before registration/dispatch.** Both are useful constructions; neither is ready. This does not stop their design or independent B5 work. FAIL-MEASUREMENT and the unearned pass-dependent entitlement remain unchanged.

## 1. Screen: keep the matched construction; make its assertions real

The forced head/base plus ordinary head/base design closes a real dispatch-coverage gap. A separate workflow keeps its diagnostic figures out of the existing A/B matrix. But making A1–A3 merge-blocking is still a prospective CI admission change: separation into another YAML file does not itself satisfy C6's reviewed decision-record requirement.

### A1 cannot be equality alone

The exact stub accepts four empty JSON objects as “A1 ok”, accepts one empty record, and accepts four matching failures without an input-specific expected outcome. Executed witnesses are in probe.py/probe-results.json. These are stub admission failures, not claims about a completed workflow: the workflow does not yet exist.

Require a manifest containing each expected arm/input pair, head/base SHA, executable digest, selector/environment, exit/signal, complete record schema and oracle expectation. Reject missing/duplicate arms, missing fields, malformed hashes, unexpected process failure and extra/missing input cases. W and generated valid corpora must succeed, with expected length and independently derived output digest; agreeing on failure is not success. Compare actual output bytes where practical, or state explicitly that the screen checks length plus a digest rather than mathematical byte equality.

For E1/E2, freeze the expected decoder error kind and block identity, plus the promised prefix semantics. The current (out,hash,err) tuple cannot establish an error's block identity unless the harness supplies it. Cross-implementation error strings are not an oracle contract. lbzip2 may buffer/report partial output differently: use its rejection as an independent check; compare exact error/prefix semantics between matched mibz2 arms under mibz2's defined contract. Do not silently require identical human error text or buffered output from unrelated decoders.

**Error locations:** cover both cooperative and solo paths: truncation and CRC corruption in each, four malformed inputs. Freeze input digest, zero-based byte/bit offset, mask, block index, block size/class and expected outcome in a versioned fixture manifest beside the checker. bzip2 block fields can be bit-aligned: identify the stored CRC bits from a structural block map, not merely an approximate byte inside compressed data. Include cancellation/relay completion in assertions and put each process under a deadline. This tiny native-input expansion is worth its coverage; no extra Callgrind matrix is implied.

### A2: executed identity, not arbitrary symbol-count coincidence

Keep a standing semantic kernel-role manifest (leader MTF, helper share/walk, relay, solo fallback), versioned with the harness. Add a pinned per-PR delta for expected changed/forbidden kernels and selector settings. Do not make unvalidated free-form workflow input the sole specification. Archive the resolved manifest.

Join required walker and share events to the **same identified helper threads**, and distinguish leader events. A minimum count of two arbitrary thread IDs is weaker than “both helpers”. Require complete profiles, unique thread identities, a successful profiled child and matching binary/source/build attribution. Inlining or unmatched symbols is missing attribution, not zero execution; either freeze an attribution-preserving build or provide verified instruction-range mapping. Never silently add noinline after freezing.

The stub permits requirements to be omitted and walked total to be absent. A small synthetic mocked-parser witness passes with two walker-bearing threads but no helpers when only its walker requirement is supplied. This tests the CLI contract, not a claim that the draft's full argument list admits that exact witness. Required assertions belong in a mandatory manifest, with negative controls for omitted helper/profile/kernel and swapped identities.

### A3: aggregate liveness, no artificial per-block fairness

For the **one-team** screen, require both helpers identified, each having 233 share and walker calls, each at least one successful claim somewhere, and the stated global conservation. Do **not** require a successful claim in every block: finding the queue exhausted is a legitimate transition. That stronger rule would confuse scheduling with correctness. Retain a leader-join execution witness if that path is claimed covered.

Totals do not prove all short/large transitions. The number 233 large plus 198 small does not specify ordering or prove that all semantic transitions occurred. Freeze a block-size sequence and required transition/cancellation cases; show coverage for solo→team, team→solo, repeated short work and terminal/error paths. Add small crafted fixtures where W does not cover the required case.

The **@9 full tier cannot reuse “each helper has 233 shares”**: each team receives a subset of blocks. Require team/block mapping and per-team expected assigned-block counts, or explicitly defer that tier until its checker exists. Global conservation alone cannot detect misassignment between teams. This is a concrete mismatch between §2's one-team assertion and “counted the same way” in §4.

The balance bar is absent only from this new coverage screen, never removed from the old failed count or the native diagnostic.

### A4 and cost

Yes, retain raw Ir and old/new Ir/D, with denominator D=209,700,196 decoded bytes explicitly distinguished from M=209,697,662 cooperative walked bytes. Include per-stage totals, full process total, tool/build identities and unmatched attribution. Label serialised schedule-dependent diagnostic; no threshold, red/green performance badge or acceptance-matrix row.

The draft's warm line items sum to 220–295 seconds (3.67–4.92 minutes), cold 465–840 (7.75–14). These are models. Parallel execution does not guarantee zero PR latency: the gating completion is the maximum of bench and screen, plus queue/setup variation. A cold screen exceeding the bench does extend time to merge. Accept pricing from the Optimiser, but freeze hard stage/job/artifact limits, failure dispositions and complete trigger dependencies (including checker/parser/fixture/build changes). No screen freeze on an unbounded stub.

## 2. Native diagnostic: one sequence, first invalid ends INCONCLUSIVE

**Choose the runner for this question.** It asks whether an instrumented native execution can share work under its own host/scheduler, not whether the i9 default is balanced or faster. Record actual CPU/core/vCPU topology and quotas; do not hardcode ubuntu-latest as necessarily four vCPUs. Oversubscription is part of the context, not equivalence to the old serialized measurement. The i9 would answer a different target-specific question and require a separately frozen C5 recipe run by Louis.

**No automatic rerun.** Use exactly old then staged once, retaining both records. If either execution is invalid, terminate the sequence as INCONCLUSIVE; do not select the first valid execution of each arm. If old fails, do not launch staged after cleanup. The fixed ordering is acceptable descriptively; it cannot identify a causal scheduler or algorithm effect from a between-arm difference.

D084 actually says “A second invalid execution is retained and ends this diagnostic sequence.” That sentence is genuinely ambiguous alongside its single-sequence/no-retry language; it does not unambiguously specify the draft's repeat-both-arms algorithm. I recommend the stricter first-invalid rule prospectively, rather than asserting the Guardian already wrote it. The Coordinator's stated preference agrees. Guardian confirmation of the resolved registration can make the handling explicit.

### Fill the instrument contract before PROCEED

- Freeze source, actual build/toolchain/options, input and accounting code digests; archive executable digest after the reproducible build and before either arm. Keep the inlining arrangement fixed.
- Bound build, each child, collection, judge and finalisation; timeout/crash/missing records/conservation failure is INVALID execution → INCONCLUSIVE diagnostic, no retry. Ensure cleanup before any next process.
- Validate exact expected block identities, not only a count of 233; unique (arm,team,block,thread,record-kind) records; one team per block; expected members and roles; nonnegative bounded counters; exactly one terminal summary per cooperative block. Reject duplicate keys, missing summaries, overflow and truncated logs.
- Define the instrument for **leaders as well as helpers**. “Write once when share returns” does not by itself explain how leader traversal records are emitted. Predicate (c) and total byte conservation require them. Give buffers per thread or another race-free collection scheme; records must not interleave or vanish on return.
- Match successful claims to deduplicated segment starts and bytes to segment lengths **per block**, then reconcile totals. State what a claim/take/exhausting attempt counts. Avoid using two totals derived from the same counter as supposedly independent conservation checks. Unit fixtures with deliberately missing, duplicated and misattributed segment records should fail.
- Keep helper-only (b) **per team across its assigned blocks**, exactly as D084: leader excluded; compare integers 3*max(helper_bytes) <= 2*sum(helper_bytes), with positive helper denominator. A zero denominator is not ENGAGED. Do not substitute a global fraction across teams.
- Report per-block helper/leader bytes and shares descriptively, including empty claims; retain aggregate gate unchanged. A balanced team total can conceal alternating complete monopolies: (100,0) then (0,100) has aggregate 1/2. That is a reason to report the distribution, not introduce a new per-block rejection after data.
- Clarify the old-arm reference: predicate (c) refers to old traverse there, not literally traverse_staged. Require correct selected kernel in each arm. For staged validity-passing data, any failed (a)–(d) means NOT ENGAGED; validity failure means INCONCLUSIVE. Preserve validity and predicate results separately.

**Instrumentation changes the object observed.** Local increments and record emission can change register pressure, spills and future scheduling; “matched” counters do not prove matched perturbation between distinct loops. ENGAGED means the *instrumented* staged execution met the bar on this runner. It does not establish uninstrumented balance or the cause of the old imbalance. Require feature-off equivalence checks and byte-exact feature-on/off outputs, plus independent B5 handling for any added unsafe machinery. No timing inference from retained incidental wall/CPU fields.

## Verification and next move

Read the exact drafts/stub and Guardian rulings; ran four small stub probes (three real outputs() calls, one check() with an explicitly mocked parser). No decoder build, Callgrind, native run, performance measurement or B5 certification. D083 was obtained via its published comment after the guessed state/records/D083/decision.md path was absent; no missing document treated as fetched.

Expected a bounded coverage screen and a separate engagement diagnostic. Found the right scope separation, but incomplete admission, multi-team accounting and invalidity handling. No performance gap closed. Highest-value next work: a mandatory screen manifest with negative controls; a fully bounded single-sequence native instrument with exact identity/conservation tests. Return exact implemented heads and untimed validation. Pricing runs independently; no queued #318 work was pulled into this assignment.

Migration source and manifest digest matched before receipt; routing audit clean. Own-state research only, no implementation, merge, dispatch or automation change.
