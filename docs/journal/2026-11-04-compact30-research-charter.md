# Compact30 instruction minimisation: a standing research charter (ACTIVE)

**Status: revision 4 (with the narrowing clarifications of reports 247–248, the user's
standing autonomy policy of report 265, §6.4, and the production accounting of report 272,
§5.6, and the whole-decoder delegation of report 273), ACTIVE (stage 1, §13). The guardian confirmed activation in report 248.** The guardian's
review of revision 2 (report 246, relayed by the user in the coordinator's session) approved it
subject to two clarifications, and authorised activation once they were made, if no other
material change was introduced. Both are in revision 4:
- §6.3's gate 3 now uses a single B2 baseline, translated minimums and overlap.
- §12 says that repository comments carry no authority.

Revision 3 also added §12 and §13 after the reviewed text. They state the guardian's own
clarification 2 and its two-stage rollout (reports 245–246), and only narrow authority. Before
activation, nothing here was authorised. It replaces case-by-case authorisation for
the work it covers. The coordinator (Claude) and its subagents then work through the sequence
below without asking at each step. They report at the checkpoints in §9, and escalate only under
§8. §13 says when activation happens.

It builds on:
- the guardian's proposal (report 242) and the researcher's gate structure (report 241);
- the guardian's conditional approval of revision 1 (b1598a1), with four amendments (report 243);
- the researcher's conditional approval of revision 1, with four further amendments (report 244);
- the guardian's approval of the model in principle (report 245), which adds a seventh
  requirement: explicit provenance and authorisation for each decision. That requirement is
  §12, and §13 sets out the two-stage rollout.

§11 maps every amendment to the text. One point where the reviews differ, whether v3's finaliser
goes into the combined build, is reconciled in §7, and that reconciliation is for the guardian to
confirm.

## 1. Objective

**Make production's inverse BWT faster (report 273).** Reduce the machine work of
`decode_block`: construction, the walk and the rest of the decode. That means executed
instructions, dependent operations, memory operations, and wherever the evidence supports it,
cache and latency effects. The whole inverse-BWT implementation is in scope: the constructor,
the walker and the representation between them.
- **Constructor prototypes** qualify on net whole-constructor work removed (the §5 ledger).
- **Every prototype**, whatever it changes, is judged for production on the whole decoder
  (§5.6).
- Local instruction targets are diagnostics.

**Permission to continue research isn't permission to claim success.** Every miss is reported
as a miss, whatever happens to the variant afterwards.

**The constructor is not the decoder (report 272).** The §5 ledger measures construction only,
and it stays the gate for constructor work. But the campaign's purpose is a faster production
decoder, and construction is only about a fifth of a block's decode. Every checkpoint therefore
also reports the whole-decoder figures of §5.6. No construction result is described as a decoder
result, and no candidate goes towards production on construction figures alone.

## 2. Fixed for the whole charter

- **Scope: the whole inverse BWT (report 273).** The constructor, the walker, the
  representation and the rest of the decode path may all be changed, through new modules and
  through PRs into master under the gate below.
- **Frozen as evidence.** These are never modified, and every new variant is a separate module,
  developed and validated independently:
  - v2 (`ctor2.rs` at `8beffe2`) and v3 (`ctor3.rs` at `dbf5139`);
  - Assembly 1 at its checkpoint commit;
  - the walker and the oracle (`C30::build`) those results were measured with.
- **Invariants.**
  - Output is byte-exact in every case.
  - A variant that keeps the compact30 representation keeps its invariants: canonical RAW
    order, and `rec` and `raw` identical to the oracle's.
  - A variant that changes the representation states its invariants and proves them under
    gate 1 before any ledger of it counts.
- **Diagnostic instrumentation is delegated.** That covers per-stage timing, counters, and what
  `examples/phases.rs` times. The bench's workload, its statistical method and its acceptance
  criteria stay unchanged.
- **None of these is authorised:**
  - timing on the i9 or a workstation, or local timing used as evidence;
  - changes to the bench's workload, statistical method or acceptance criteria, or to the
    channel protocol;
  - relaxing a gate or a minimum.
- **Delegated to the Coordinator (report 266):** production integration, PRs into master, merges and
  releases. The route is the same one the project already uses for the decoder:
  - a PR into master runs CI's paired A/B bench, which is the authority on performance;
  - the Coordinator merges when parity passes, CI is green, and an end-to-end figure improves
    beyond noise (its 95% interval excludes zero), with no unacceptable regression;
  - releases go through the repository's Deploy Release workflow and trusted publishing.
  - Downstream pin bumps, such as wikidata-pq, follow the same bench gate.
- **Beyond this campaign (report 267):** the user made the Coordinator mibz2's standing production and
  release steward, with this campaign under that role: master:docs/STEWARDSHIP.md.
- Recorded outcomes are never reinterpreted:
  - v1: STOP;
  - both construction captures: INVALID;
  - v3: STOP under its compiler gate;
  - the traversal: VALID ADVANCE.

  Carrying a variant forward (§6) never changes its recorded gate outcomes.
- Commits are authored and committed as Louis Maddox, with no AI attribution.
- No force-pushes.
- One heavy job at a time on the shared machine.

## 3. Delegated without further approval

For each prototype in the sequence, within §2:
- implementing it as a separate module;
- **up to three** Rust-expression attempts at its compile target, chosen by the coordinator. Each
  attempt is archived with its listing. An attempt may change only how the registered design is
  expressed. One that adds a safety obligation isn't an attempt: it follows §6.4 before it continues.
- running every correctness suite, targeted Miri runs, mutation checks, disassembly at the pinned
  toolchain, and the ledger accounting (§5);
- repairing routine defects inside that prototype's scope (§8.1);
- writing journal entries and evidence;
- moving on to the next prototype when the current one closes, whatever its outcome, unless §8
  applies;
- running the pre-integration review (§6.2) and assembling the combined build (§6.3);
- introducing or extending `unsafe` code, and registering new bounded prototypes, under the
  standing policy of §6.4. Neither needs the user.

## 4. Gates, registered before each prototype

1. **Correctness and safety (mandatory, and a qualifier).**
   - Byte-exact `rec` and `raw` against the oracle on these suites:
     - the corpora;
     - the adversarial suite;
     - the full synthetic suite.
   - Unchanged walker output and marker restoration.
   - Malformed counts rejected, with both arrays empty.
   - Reuse across sizes.
   - A written release-mode argument covering initialisation, bounds and pointer validity.
   - Targeted Miri on the prototype's new paths, with the classes it must reach registered before
     it runs. Completing every registered class is required (§8.3).
   - Where a new invariant is introduced, a mutation check showing the tests catch a violation of
     it.
2. **Machine code (diagnostic, not a qualifier).**
   - The compiled hot-path counts against the prototype's stated target:
     - instructions;
     - loads;
     - stores.
   - Per-row loops and call sites unchanged, unless the prototype is meant to change them.
   - A miss is recorded formally as a miss, and is never retrospectively called a pass. It ends
     that prototype's attempts once all three are used. **It doesn't by itself exclude the
     variant** from qualification or from the combined build.
3. **Net work (the qualifier).**
   - The prototype's ledger against its own baseline:
     - the §5 ledger for a constructor prototype;
     - the §5.6 production ledger (whole `decode_block`) for a walker, representation or other
       decode-path prototype. Its §5.3 conventions apply: whole functions recounted, bounded
       frequencies at their least favourable end, unknown costs never zero.
   - The lower end of its conservative bound must meet the minimum registered before it ran
     (§6.4).
4. **Economics (outside this charter).** Only a separately authorised, formally VALID measurement
   decides the registered CPU and completion ceilings.

**A prototype qualifies** for the combined build when:
- gate 1 passes completely;
- gate 3 meets its minimum;
- the pre-integration review (§6.2) leaves no unresolved material finding on it.

Gate 2 is reported, never a qualifier.

## 5. The net-saving ledger (the gate)

### 5.1 Unit and method

The unit is the instructions executed by the uncounted instantiation `build::<false>`, including
everything it calls, in one pass over the primary corpus: 434 blocks, 207,006,515 rows. It is
computed as:

  Σ over compiled paths (instructions on the path) × (the path's observed execution count).

- Path lengths come from the disassembly of the uncounted build. It is compiled with rustc 1.97.0
  (`2d8144b78`, LLVM 22.1.6), `--locked --release`, with the same flags as v2.
- Counts come from the counted instantiation's per-block counters over the same corpus.
- Nothing is timed, and perf counters aren't used.
- Loads, stores and stack operations are counted the same way and reported alongside.

### 5.2 Baselines (fixed)

- **B2, frozen v2 at `8beffe2`:**
  - listings in `2026-11-03-compact30-constructor-v2-prototype/` (cd14fa9);
  - frequencies from v2's capture counters, as recomputed in the scanner handoff (25d7eb59).

  B2 is the baseline for prototypes 1 and 2, and for the combined build.
- **B3, frozen v3 at `dbf5139`:**
  - listings and ledger in `2026-11-04-compact30-epoch-tagged-finaliser/` and its journal's §6.

  B3 is the baseline for prototype 3 only.

### 5.3 Accounting conventions

a. **Whole functions are recounted.** Every function whose listing changes is recounted in full,
   not only the targeted sequence. That includes:
   - added instructions anywhere in it;
   - new spills and reloads;
   - changed callers and callees;
   - per-block, per-epoch, per-group and per-run setup;
   - initialisation;
   - fallback and exceptional paths.
b. **Every changed path has a count.** Every path with changed code needs an observed execution
   count. Where no existing counter covers it, a counter is added to the counted instantiation
   only, and the uncounted listing is checked unchanged.
c. **Bounded frequencies take their least favourable end.** Where a frequency is known only
   within bounds, the end least favourable to the saving is used. For example, the scanner's
   Q ∈ [0, 3,450].
d. **Moved work is counted where it lands.** An example is a check hoisted to block setup.
e. **Unknown costs are never zero.** A cost that can't be counted or bounded is listed as
   unresolved. While any relevant added work remains unbounded, no verified net saving is
   claimed, and the prototype doesn't qualify.
f. **The figure is a conservative interval.** Saving = baseline ledger − prototype ledger, given
   as an interval from (c) and (e). Gate 3 tests its lower end.
g. **Memory operations are reported, not gated.** A net increase in loads or stores on a per-row
   path is reported as a finding for the review.

### 5.4 The minimums and how they were set

| order | prototype | baseline | conditional estimate (source) | registered minimum | minimum / estimate |
|--:|---|---|---|--:|--:|
| 1 | scanner and tile-bounds cleanup | B2 | gross 88,622,080 + 3Q, with Q ∈ [0, 3,450] (handoff 25d7eb59); 87.62 M in report 227 §D.2 | 60 M | ≈ 68% |
| 2 | hot descriptor pointer arrays | B2 | 71.59 M (report 227 §D.3) | 40 M | ≈ 56% |
| 3 | SSE2 first-chunk refinement of v3's finaliser | B3 | 4 instructions × 1,647,375 leaf finishes = 6,589,500 | 4 M on top of v3 | ≈ 61% |
| 3b | refill-boundary successor shortcut (C6) | prototype 2 (`7620c0c6`), incremental | (69 − C) × 1,084,913 − g × 710,569 − H_D: 27,909,632 − H_D at C = 40, g = 5 (Optimiser @ `ea3ace12`) | 10 M on top of prototype 2 | ≈ 36% of the pre-H_D figure |
| 4 | full-state v3 tag (C7, report 271) | Assembly 1 at its checkpoint commit, incremental | 6 × 1,647,375 − k × 1,795,482 − H7: 6,293,286 − H7 at k = 2; 8,088,768 − H7 at k = 1 (Optimiser @ `82bc69b5`, corrected by the Researcher @ `a237a758`) | 4 M on top of Assembly 1 | ≈ 64% of the k = 2 pre-H7 figure |

- The minimums are screens, not predictions. They sit below each estimate, so that a
  substantially smaller real saving still clears them while a negligible one does not.
- The estimates are conditional: on the compiler removing what the design removes, and, for
  prototype 2, on its integration costs.

### 5.5 Overlap

- Prototypes 1 and 2 are each measured against B2 alone. An instruction that both remove is
  therefore credited to both.
- Each checkpoint reports its overlap: the intersection of its listing diff with the earlier
  prototypes' diffs.
- The sum of isolated savings is a prediction only. Only the combined ledger (§6.3) decides the
  combined build.

### 5.6 Production accounting (report 272)

**Why.** PR #155 put Assembly 1 into production. Construction fell by 1,059,858 instructions
per Wikidata block (−19.5%). The record walk rose by 4,780,808 (+31.0%). The whole block decode
rose by 3,778,665 (+14.7%), and CI's paired bench measured the decoder 7–19% slower. The §5
ledger could not see this, because the walk and the rest of the decode are outside it. On
production master a block's `decode_block` is about 21% build, 60% walk and 19% the rest (MTF,
CRC, assembly): 5,429,062 + 15,426,892 + 4,895,840 instructions on block 1.

**The rules.**

a. **A production ledger at every checkpoint.** Every checkpoint and every combined build
   reports, next to its §5 ledger, the instructions executed in `decode_block`:
   - for production master, and for the candidate wired into production, or into a harness
     that calls the same build, walk and remaining code;
   - split into build, walk and the rest;
   - on matched toolchain, flags and input;
   - with the raw profiles, the driver and the command archived for replay
     (`compact30-channel/evidence/port-b4786f66/` is the template).
b. **Sample.** A checkpoint uses at least the archived Wikidata blocks (1 and 41). An
   integration decision uses a sample covering the primary corpus's block classes, and the
   whole corpus when that is practical. One block is a screen, not a result.
c. **Ranking.** Proposals are ranked by whole-decoder instructions removed per block, after every
   cost they induce in other phases. A change to the representation is priced for every consumer
   of the representation, the walk included, not only for the code that produces it.
d. **PRs into master for a candidate kernel** (revised by report 273).
   - **An integration PR, meant to merge,** needs the production ledger at or below master's
     across the sample. Otherwise it is a measurement PR.
   - **A measurement PR** is an experiment, and CI is where the hypothesis is tested. It is
     opened when a hypothesis is registered first, in the channel and the PR body, stating:
     - the mechanism;
     - the predicted whole-decoder effect at each of CI's thread counts (1, 2 and 4), with its
       sign and size;
     - the instruction excess the mechanism must overcome;
     - the result that falsifies it.
   - **What doesn't count:**
     - timing on the shared development machine is never a hypothesis's evidence;
     - a projection that predicts a loss can't be set aside unless the hypothesis says why the
       projection is wrong.
   - **Closing out.** A falsified measurement PR is closed and written up. Merges stay under
     §2's gate.
e. **What parity is.** Instruction parity is a screen, neither necessary nor sufficient (D016).
   CI's end-to-end paired bench decides every merge, under §2.
f. **Scope.** This ledger is a diagnostic and a precondition. It does not replace gate 3, and it
   does not change any recorded result (D015).
g. **Optimise the binding cost.** The walk is in scope (§2, report 273). Optimise whichever cost
   the production ledger shows binding.
h. **The phase table.** Until `examples/phases.rs` times the kernel under evaluation, its rows
   say nothing about a candidate's phases. Correcting it is delegated diagnostic
   instrumentation (§2).

## 6. The sequence

### 6.1 Isolated prototypes

| order | prototype | starts from | compile target (diagnostic) |
|--:|---|---|---|
| 1 | scanner and tile-bounds cleanup (report 227 §D.2), as in the handoff 25d7eb59 | frozen v2 | the handoff's per-path targets: fresh pair 12 → 4, lookahead 16 → 11, nonuniform-tile overhead 41 + 7X → 34 + 7X; per-row loops otherwise unchanged |
| 2 | hot descriptor pointer arrays (report 227 §D.3) | frozen v2 | scalar append ≤ 10 instructions, with all added non-row work counted |
| 3 | SSE2 first-chunk refinement of v3's finaliser | frozen v3 (`dbf5139`) | fast-finish body ≤ 40 (≤ 41 with the call); ≤ 10 explicit reads |
| 3b | refill-boundary successor shortcut (C6, proposal `0ac2e67`), refill transition only (report 264) | prototype 2 @ `7620c0c6` | eligible refill wrapper C ≤ 40 (diagnostic), failed-guard cost g ≤ 5; scalar row path unchanged |
| 4 | full-state v3 tag (C7), fast-finish leaf and tag construction only (report 271) | Assembly 1 at its checkpoint commit | fast leaf: one FULL-tag compare replaces the active-tag check plus the `write − w0` fullness test; ≤ 2 added instructions per open; no new spill on the scalar append or refill paths |

- Each prototype is a single change on its own baseline, so its saving can be attributed.
- Prototype 3 is optional. It is run only if v3 is eligible under §7(a).
- **Unsafe code follows the standing policy in §6.4** (report 265). It supersedes the
  safe-Rust-first and owner-approval clauses of reports 247–248 and 259, which were written by the
  agents, not requested by the user.
- **U2 in the combined build (report 264).** The U2 unchecked access is carried unchanged into the
  combined build: the same full-tile bounds and initialisation invariants, and its Miri classes and
  mutation checks run again on the combined code. Any further `unsafe` follows §6.4.
- **Prototype 3b (report 264).** It is registered as one bounded experiment on prototype 2's
  pinned descriptors. It changes only the refill transition and the mode plumbing that strictly
  needs.
  - It has the ordinary three attempts and the full gate 1 obligations, plus successor-boundary
    cases: an empty descriptor, exactly 29 and exactly 30 successor ranks, a forced RAW successor,
    an exhausted or malformed bucket, and exactly one finish on every non-empty path.
  - An increase in the scalar common-row path, or an unbounded residual, is a STOP or HOLD.
  - It needs a counted, mutually exclusive partition of refill classes under §5.3(b).
  - It earns no credit before its own gate 3, and doesn't delay the combined build of the
    components that already qualified.
- **Prototype 4, C7 (report 271).** Registered by the Coordinator under §6.4, on the Researcher's
  assessment @ `a237a758`. It has no `unsafe`.
  - **Eligible leaves:** 1,647,375. The 41 same-epoch partials are excluded. The count is redone
    on Assembly 1's pinned combined baseline.
  - **Invariants:**
    - FULL is set if and only if the portion holds 30 fields. FULL at open proves capacity, not
      initialisation, so the finish caller still proves `write == end`.
    - The FULL bit is stripped from the output header and from epoch decoding. Keeping
      `tag & !ORD` would corrupt every FULL header.
    - The bit cannot collide with RAW indices.
    - The `w0` load stays; only the `write` load may go.
    - The slow-tag diagnostics are audited.
  - **Gate 1 cases:**
    - every finish class: fast, same-epoch partial, old-epoch, crossing, RAW and wide;
    - header bytes identical to the oracle;
    - epoch wrap;
    - a mutation that sets FULL on a partial portion must fail the suite.
  - **Rules:** the ordinary three attempts. Every open, epoch, slow-path, caller and spill cost is
    charged. It earns no credit before its own gate 3.

### 6.2 Pre-integration review (before any combined build)

1. When the isolated prototypes have closed, the coordinator pins one commit holding:
   - every isolated result;
   - an evidence index of the listings, consoles, ledgers and Miri coverage.
2. The Researcher, the standing independent scientific reviewer, reviews adversarially from that
   pinned evidence: the source, the listings, the counters and the tests (report 270).
   - Independence means independence of judgement and implementation, not ignorance of earlier
     discussion. The Researcher verifies consequential claims itself and actively looks for
     counterexamples.
   - It discloses any part it played in a design under review, reviews that implementation
     adversarially, and never calls such a review blind.
   - No fresh reviewer, new chat or context-isolated agent is spawned for a review. Any
     additional agent needs Louis's explicit authorisation.
   - It may push its report and scripts to a branch of its own.
3. The coordinator verifies, archives and replays the review, as for report 235's evidence.
4. **A material finding** is one that, if true, would change any of:
   - a gate outcome;
   - a safety argument;
   - a ledger interval across its minimum.
5. The coordinator resolves each material finding before integration. It either:
   - fixes the problem and re-establishes the evidence; or
   - writes a rebuttal that the reviewer accepts.

   A finding still unresolved after one round of rebuttal goes to the guardian.

### 6.3 The combined build

**Components:**
- every prototype that qualified (§4);
- v3's finaliser, only if it meets §7 (and prototype 3, if it qualified on top of v3).

**Gates, of its own, never assumed from the components:**
1. **Gate 1, in full, on the combined module:**
   - every suite;
   - Miri over the union of the components' registered classes;
   - each component's mutation check, run again against the combined code.
2. **Gate 2, diagnostic:**
   - the whole `build::<false>` listing and its callees, diffed against B2 and against each
     component's isolated listing;
   - each component's compiled path either reproduced, or its deviation listed and recounted;
   - new spills or reloads, register-allocation changes and new memory dependences on per-row
     paths reported as findings.
3. **Gate 3, the combined whole-constructor ledger, on one baseline (B2):**
   - **What is measured.** The combined build's actual compiled work, counted once and from
     scratch against B2 under §5. This charges every integration effect, and counts each
     instruction once however many components touch it. The measured figure is never a sum of
     isolated savings.
   - **The floor,** F, is built from each included component's minimum, translated to B2:

     | included component | its baseline | its minimum, translated to B2 |
     |---|---|--:|
     | scanner (prototype 1) | B2 | 60 M |
     | descriptor arrays (prototype 2) | B2 | 40 M |
     | v3's finaliser | B2, by §7(b) | 50 M |
     | SSE2 refinement (prototype 3), only with v3 | B3, incremental | +4 M on top of v3's 50 M |
| refill shortcut (prototype 3b), only with prototype 2 | prototype 2, incremental | +10 M on top of prototype 2's 40 M |

     Then F = Σ (the translated minimums of the included components) − O.
   - **O, the overlap,** is the B2 work that more than one included component's isolated ledger
     credited as removed.
     - It is computed from the intersection of their listing diffs × observed frequencies, as
       reported under §5.5.
     - Each shared instruction is counted once.
     - The lower bound of O is used, so only demonstrated overlap lowers F.
     - v3 and the SSE2 refinement are a single chain on B3, so they have no overlap with each
       other. Overlap between v3's finish and the scanner or descriptor diffs is counted like any
       other.
   - **Pass conditions,** both required:
     - (i) the combined ledger's lower end ≥ F;
     - (ii) the combined ledger's lower end > the best single qualified component's own isolated
       lower bound, on B2. For v3 with the SSE2 refinement, that bound is v3's isolated lower
       bound plus the refinement's incremental lower bound.

     If (ii) fails, the stack isn't kept, and the best single component is the outcome.
   - **v3's own condition.** If v3 is included, its §7(b) contribution, re-derived from the
     combined listing, must also hold on its own.

**Attempts:**
- Up to three assemblies, each archived.
- An assembly may drop a component. The reason is recorded, and F and O are recomputed for the
  components that remain.

### 6.4 Standing policy: `unsafe` code and new prototypes (report 265)

This is the user's standing policy, given in the coordinator's session (report 265). It makes
autonomous, evidence-based decisions the default. It does not weaken any correctness or safety
standard in this charter.

**`unsafe` Rust is permitted** when it offers a worthwhile, measurable benefit and its safety
invariants can be clearly justified and rigorously tested.
- **The justification is written down before the code is relied on.** It is a release-mode safety
  argument naming every unsafe operation and its location, the dominating bounds invariant, the
  integer-overflow proof, and the pointer-validity and initialisation argument, plus the expected
  benefit.
- **It is tested with every appropriate method:**
  - Miri over registered classes that reach each operation, with no timeout counted as a PASS
    (§8.3);
  - differential testing, byte-exact against the oracle and the reference builder;
  - fuzzing of malformed and adversarial inputs;
  - mutation checks, each of which must fail its intended test.

  Passing tests alone is not proof of soundness. The argument and the tests are both required.
- **Another agent reviews it.** An agent other than the implementer reviews the safety argument
  adversarially: the Researcher (report 270). The work proceeds when that
  review and the verification support it. The Guardian may HOLD it if the evidence falls short
  (narrowing), but its approval is not a precondition (report 266). No user
  approval is needed for introducing or extending `unsafe` code.
- **If verification fails,** the implementation is fixed, or the work falls back to a safe
  alternative. Undefined behaviour stops that variant at once (§8.2) until it is fixed or
  abandoned, and is recorded; it is not escalated to the user.

**New prototypes may be registered by the agents** without the user, when the registration gives:
- a technical justification;
- a pinned baseline;
- a pre-registered incremental minimum, set below the estimate as in §5.4;
- defined safety conditions under this section;
- an evaluation plan with its gates (§4), within §2.

The Optimiser or the Coordinator proposes it, and the Researcher assesses it independently. The
Coordinator registers it in this charter, citing that assessment. The Guardian may HOLD a
registration (narrowing, §12.2), but no Guardian approval is needed (report 266). Registration is permission to experiment, not a
commitment to adopt. If the experiment fails its checks or doesn't deliver its minimum, the agents
reject it themselves and record it.

**Still not delegated** (§12.3): relaxing a gate or a minimum, i9 or workstation timing, and new
infrastructure, credentials or spending. Production integration, merges and releases are delegated
to the Coordinator under §2 (report 266).

## 7. V3's finaliser in the combined build

- V3's formal STOP stands, and its ≈ 60.2 M saving is preserved as a result.
- V3 is a candidate component. **It isn't included automatically.** It enters only if all three
  of these hold:
  - **(a) Safety evidence, complete and reviewed.** The closeout at 46c3222 is complete: 134
    targeted Miri cases, exact, with no undefined behaviour; the mutation check; the release-mode
    argument; and the identical rebuild. The §6.2 review must accept it with no unresolved
    material finding.
  - **(b) A structural rule, registered here, before any combined listing exists.** The
    finaliser's contribution is re-derived from the combined listing. That means B2's fast full
    finish against the combined leaf, × the observed count, plus every added cost under §5.3. It
    must be ≥ **50 M instructions** at its lower end. 50 M is ≈ 76% of report 227's 66.04 M
    budget, and ≈ 83% of v3's isolated 60.197 M.
  - **(c) No more instructions than at `dbf5139`.** Its compiled leaf in the combined build is no
    longer than at `dbf5139`: 44 instructions including `ret`. If prototype 3 is included, the
    limit is prototype 3's own compiled count.
- **This rule isn't prospective about v3 in isolation**, whose figure is already known. What it
  fixes in advance is the bar for the re-derived figure in the combined build, which doesn't exist
  yet.
- If v3 is included, the record says: "included under §7; compiler-gate STOP unchanged".
- **This reconciles the two reviews.** The guardian requires completed safety evidence and an
  explicit structural rule; the researcher holds that v3 belongs in the combined build. §7 adopts
  the researcher's position conditionally, on the guardian's terms.

## 8. Repairs, escalation and stops

### 8.1 Routine repairs

These are handled within a prototype's three attempts, without escalation:
- a test mismatch or panic whose cause lies in the prototype's own new code, and which is fixed
  without adding unsafe code, a new invariant or a change to the safety argument;
- compile-target misses.

### 8.2 Immediate stop and escalation

The affected work stops at once, with no further attempts, and the guardian is told at once,
when any of these happens:
- **Undefined behaviour:** Miri, or any other tool, reports undefined behaviour that is
  unresolved.
- **An unsafe access can't be justified:** the release-mode argument for it can't be written, or
  is found wrong.
- **A frozen invariant is violated:**
  - a finding contradicts the representation, the walker, the oracle, canonical RAW order or
    production's builder; or
  - a frozen file differs from its pin.
- **An attempt adds a safety obligation outside the registered design** before §6.4's argument
  and review exist for it. Examples:
  - a new unsafe block or unchecked access;
  - new pointer arithmetic;
  - a read of possibly uninitialised memory;
  - an invariant that the safety argument would newly rely on.

  That is a design change, not tuning.
- **A mutation check fails** to catch the violation it targets.

After an escalation:
- work continues only on prototypes that share no code with the affected one;
- no combined build is assembled until the guardian has decided;
- the affected work resumes once the cause is fixed, or a safe alternative is in place, and the
  guardian records it. That needs no user.

### 8.3 Miri coverage

- A timeout or partial Miri run is partial coverage, never a PASS.
- The classes it didn't reach are listed.
- The coordinator may split the cases into smaller runs to complete coverage.
- Until every registered class is complete, the variant's qualification is paused. It doesn't
  enter the combined build while paused.

### 8.4 Continuing or stopping a line of work (reports 267–269)

A prototype that falls below its net minimum is recorded as a miss, never as a pass. A failed gate
is a verdict on the tested candidate, and not automatically on its line of work. Reports 217–219,
224–227 and 238–241 show the cost of letting one gate's failure stand in for a verdict on the
whole programme. A line can still legitimately end. Cumulative evidence may establish that its
payoff is insufficient: for example, a proof that an essential invariant makes the
representation impossible to implement correctly, or a convincing lower bound showing that the
remaining economic gap cannot be closed.

What decides a line is its **trajectory toward production viability**, meaning the economic
limits in the campaign's entry document
(research/compact30-channel:docs/journal/compact30-channel/CAMPAIGN.md).

**Viability is judged on the whole decoder (report 272).** "The remaining gap" in this section
means the gap in the §5.6 production ledger and in CI's end-to-end bench, not the constructor's
gap alone. A line whose constructor gains are outweighed by costs elsewhere in the decoder is
not progressing towards viability, however much its constructor ledger improves.
- **Keep climbing** while either of these holds:
  - the accumulated evidence shows meaningful progress toward viability;
  - a bounded next experiment offers a credible chance to overcome an identified remaining
    obstacle.

  Not every experiment must beat the last one's numbers. A bounded exploratory step whose
  immediate value is information is allowed, provided it has a specific hypothesis and a
  credible link to the objective.
- **Progress comes in two kinds, which are not interchangeable:**
  - *Performance progress:* valid end-to-end measurements, or compiled and correctly
    accounted reductions in instructions, memory operations and dependencies that strengthen
    the performance case.
  - *Research progress:* new evidence that materially increases confidence in a specific route
    to improvement. An example is a failed compiler experiment that pins down which part of an
    anticipated reduction is missing. Proposals with no compiler-achievable savings are not
    progress, however promising they look.
- **The model is the builder's v1 → v2 step.** The common scalar append path fell from about 49
  instructions to 12. Four-worker build CPU fell from 1.316× production to 1.086×, and
  completion from 1.364× to 1.114×, against limits of 1.067× and 1.02×. These ratios are
  descriptive figures from captures that failed their validity gates, not validated speedups.
- **Stop or redirect** a line when cumulative evidence establishes any of these:
  - insufficient payoff;
  - persistent stagnation;
  - no credible route forward.

  The signs are sunk cost, a deteriorating outlook, or a plateau with no path in sight.
- **Every significant experiment's report answers four questions:**
  1. What improvement was expected, and what actually happened?
  2. How much of the remaining economic gap has credibly been closed?
  3. What did the experiment establish about the limiting mechanism?
  4. What is the highest-value next experiment, and is its expected payoff worth the compute?

  The report always considers STOP and REDIRECT explicitly, alongside "next experiment". The
  Optimiser and the Researcher are responsible for that judgement, not for a stream of new
  ideas.
- **Distinguish research and structural progress from validated production performance.**
  Never confuse progress in understanding with qualification for production, and never weaken
  a qualification gate.
- **Before a further prototype in a line that has missed,** record its trajectory:
  - the gap after each attempt;
  - the mechanism the next attempt tests that earlier ones did not;
  - its pre-registered minimum.

  The Optimiser identifies the opportunity, and the Researcher assesses it independently. The
  Coordinator records the payoff case and registers the prototype under §6.4. The Guardian
  may HOLD.
- **Gates and minimums are never relaxed** to justify earlier effort.

This doesn't prevent:
- the proper closeout of work already done;
- the §6.2 review;
- combined builds of the variants that already qualified.

The sequence also stops early, and reports, in two other cases:
- a step would need something outside §2;
- an escalation under §8.2 arises in the combined build itself.

## 9. Reporting

- **Per-prototype checkpoints:** one journal entry plus a short summary, giving:
  - the pin, and the frozen files unchanged;
  - correctness, safety and Miri coverage;
  - the compiled counts against target;
  - the §5 ledger interval and its overlap;
  - each gate's outcome;
  - limitations;
  - what comes next.

  These need no guardian action.
- **The pre-integration review (§6.2),** and its resolution.
- **The end-of-sequence report:** the combined build's gates, the cumulative net-work ledger,
  pinned commits, reproducible evidence and an explicit research disposition.
- **The guardian is needed only for:**
  - escalations under §8.2;
  - review findings still unresolved after rebuttal;
  - proposed scope changes;
  - any future measurement decision.

## 10. Independent review

- Adversarial reviews are of completed checkpoints, not intermediate steps.
- Reviewers read the repository directly at a named commit, and may push their reports and
  scripts to a branch of their own.
- The Researcher is the standing independent reviewer. It works from pinned evidence, verifies
  claims itself and discloses its own involvement. It is not kept ignorant of the discussion,
  and no fresh or context-isolated reviewer is spawned (report 270). Additional agents need
  Louis's explicit authorisation.
- The coordinator verifies, archives and replays every review.

## 11. Amendments incorporated

| source | amendment | where |
|---|---|---|
| guardian 1 | v3 not included automatically; safety evidence and a prospective structural rule | §7 |
| guardian 2 | exact baselines and accounting; the combined build has its own ledger, not a sum | §5, §6.3 gate 3 |
| guardian 3 | routine repairs separated from safety failures; immediate escalation; a Miri timeout never PASS | §8.1–§8.3 |
| guardian 4 | independent adversarial review before combining; material findings resolved first | §6.2 |
| researcher 1 | local machine-code targets are diagnostic, not qualifiers; misses recorded, never called passes | §4, §1 |
| researcher 2 | scanner and descriptor from v2; SSE2 from v3; each against its actual baseline | §5.2, §6.1 |
| researcher 3 | an early stop halts new prototypes but not closeout or one combined build | §8.4 |
| researcher 4 | all changed paths in the ledger; conservative bounds; unknown costs never zero | §5.3 |
| researcher (additional) | complete correctness evidence and Miri coverage before entering the combined build | §4, §8.3 |
| guardian 7 (report 245) | explicit provenance and authorisation for each decision; a self-declared header isn't authentication; routine charter work needs no fresh PROCEED | §12 |
| guardian (report 245) | two-stage rollout; the channel is tested end to end before unattended use | §13 |
| guardian 1 (report 246) | the combined-build floor on one B2 baseline, with translated minimums, no double counting and all integration effects | §6.3 gate 3 |
| researcher (report 247) | the scanner handoff's unsafe reads are not delegated: safe Rust first, and the unsafe form escalated with its exact operations and proofs | §6.1 |
| guardian (report 248) | the unsafe fallback's approval package (operations, locations, invariants, overflow, validity and initialisation, expected saving); autonomous once approved | §6.1 |
| guardian 2 (report 246) | the standing charter authorises work within its scope; repository comments are communication, never a source of authority | §12 |
| user (report 273) | the objective is the whole production decoder; the whole inverse BWT (constructor, walker, representation) is in scope, with earlier modules frozen as evidence; per-stage diagnostic instrumentation delegated; measurement PRs allowed on a registered quantitative hypothesis; technical direction delegated, escalating only reserved policy, resources or measurement methodology | §1, §2, §4, §5.6, §12.3 |
| user (report 272) | production accounting: every checkpoint reports whole-decoder instructions (build, walk, rest) against production master; ranking by whole-decoder saving; no PR into master for a candidate kernel without whole-decoder parity or an assessed quantitative case; viability judged on the whole decoder; local timing never a reason to integrate | §1, §5.6, §8.4 |
| user (report 270) | the Researcher is the standing independent reviewer; no fresh, new-chat or context-isolated reviewers; additional agents need Louis's explicit authorisation | §6.2, §6.4, §10 |
| user (report 269) | a failed gate is a verdict on the candidate, not automatically its line; a line may end on cumulative evidence (impossibility, lower bound); every significant experiment reports expected vs actual, gap closed, mechanism learned, and the next step including STOP/REDIRECT | §8.4 |
| user (report 268) | §8.4 progress is of two kinds, performance and research, never interchangeable; bounded exploratory steps allowed; every STOP scoped to what failed; understanding is not qualification | §8.4 |
| user (report 267) | §8.4's two-misses stop replaced by a trajectory test: keep climbing while there is real forward motion toward viability; stop on sunk cost, deterioration, or a plateau with no path | §8.4 |
| user (report 266) | release control already delegated: integration, PRs into master, merges and releases go through the CI bench gate, run by the Coordinator; registration and `unsafe` sign-off need no Guardian approval (the Guardian may HOLD) | §2, §6.4, §12.3 |
| user (report 265) | standing policy: `unsafe` is permitted on a justified argument, rigorous verification and independent review; agents register bounded prototypes; neither needs the user | §6.4, §3, §8.2, §12.3 |
| user (report 264) | U2 carried unchanged into the combined build; prototype 3b (C6) registered as a bounded refill-only experiment, with a 10 M incremental minimum | §6.1, §5.4, §6.3 |

## 12. Provenance and authorisation

### 12.1 The problem

- Every party's GitHub actions appear as `lmmx`: the user, the coordinator, Astra and any Codex
  thread.
- A `FROM:` header on a comment names a role, but proves nothing.
- The only authenticated channel to the coordinator is the user's own messages in the
  coordinator's session.

### 12.2 The rule: the repository can narrow authority, never widen it

| source | may trigger |
|---|---|
| researcher or other reviewer (repository) | findings, objections, recommended changes |
| guardian (repository) | HOLD, STOP, or a REVISE that tightens a gate or narrows scope |
| the coordinator | only work already authorised by this charter |
| the user, in the coordinator's session | anything that widens authority (§12.3) |

- **Narrowing messages are honoured whoever wrote them.** A HOLD, a STOP, a finding or a
  tightening REVISE is acted on even if its author can't be confirmed. Honouring a false one costs
  only time.
- **Routine work under this charter needs no PROCEED.** The coordinator proceeds through §6 by
  itself. A PROCEED in the repository is acknowledged and recorded, but it authorises nothing
  beyond the charter.
- **Comments from any account other than `lmmx`** are recorded as data and never acted on as
  instructions.
- **Conflicting messages:** narrowing wins over continuing. Any ambiguity is treated as a HOLD.

### 12.3 What needs the user in the coordinator's session

The following count only when the user says so in the coordinator's own session, naming the
decision and, where there is one, the repository message it refers to:
- approving or amending this charter;
- relaxing a gate or a minimum;
- widening scope;
- i9 or workstation timing, or a capture;
- changes to the bench's workload, statistical method or acceptance criteria (per-stage
  diagnostic instrumentation is delegated, §2, report 273);
- accepting a rebuttal that the review didn't accept (§6.2);
- new infrastructure, credentials or spending.

**Not reserved (§6.4, reports 265–266):** introducing or extending `unsafe` code, registering
bounded prototypes, resuming after a §8.2 stop, and production integration, PRs into master,
merges and releases under §2's CI-bench gate. These are routine decisions for the agents and the
Coordinator.

**Technical direction is delegated (report 273).** The Coordinator and the three agents decide
for themselves whether to optimise, redesign, measure, redirect or stop. They decide on
credible whole-decoder payoff per unit of compute, within the gates, the invariants and the
reservations above. The user is not asked to choose experiments or to approve routine
implementation choices. Escalation is only for a genuine change to reserved policy, resources
or measurement methodology.

The coordinator records the user's words verbatim in the journal as that decision's provenance.
The user may relay the guardian's text. What authenticates it is the user's act of relaying it,
not the header.

### 12.4 The record

Every decision the coordinator acts on is logged with:
- its source (repository comment id and time, or the user's message in the session);
- the charter section it relies on;
- the action taken.

Repository messages are transcribed verbatim into the channel log in the coordinator's next
commit.

## 13. Activation: two stages

1. **Stage 1, the charter.** The sequence in §6 starts when the user relays the guardian's final
   approval of this revision in the coordinator's session (§12.3). From then on, checkpoints are
   written to the repository and summarised to the user. No routine relay is needed.
2. **Stage 2, the channel.** Using the repository channel (`docs/journal/compact30-channel/`) for
   unattended reviews and decisions starts only after its end-to-end test has passed. The test is
   in the protocol, §7, and its results are recorded. Until then, the user relays any repository
   decision that must reach the coordinator.

