# Compact30 instruction minimisation: a standing research charter (PROPOSED)

**Status: proposed, awaiting one guardian approval.** Until it is approved, nothing here is
authorised. Once approved, it replaces case-by-case authorisation for the work it covers. The
coordinator (Claude) and its subagents then work through the sequence below without asking at
each step, and report back only at the checkpoints listed. It builds on the guardian's
proposal (report 242) and the researcher's gate structure (report 241).

## 1. Objective

Reduce the machine work needed to build the frozen compact30 tables: executed instructions,
dependent operations and memory operations. Measure progress by **net whole-constructor work
removed**, counted from compiled code and observed path frequencies, with every added cost
included. Local instruction targets are diagnostics, not investment gates.

## 2. Fixed for the whole charter

- The frozen representation, the walker, the oracle (`C30::build`), canonical RAW order and
  production's builder are unchanged.
- Frozen v2 (`ctor2.rs` at `8beffe2`) and v3 (`ctor3.rs` at `dbf5139`) stay unmodified. Each new
  prototype is a separate research module.
- No timing, benchmark, perf counter, i9 or workstation run, protocol change, production
  integration, PR into master or release.
- Recorded outcomes are never reinterpreted: v1 STOP, both construction captures INVALID, v3
  STOP under its gate, and the traversal VALID ADVANCE.
- Commits are authored and committed as Louis Maddox, with no AI attribution. No force-pushes.
  One heavy job at a time on the shared machine.

## 3. Delegated without further approval

For each prototype in the sequence, within §2:
- implementing it as a separate module;
- **up to three** Rust-expression attempts to meet its compile target, each one archived;
- running all correctness suites, targeted Miri runs, mutation checks, disassembly and ledger
  accounting;
- repairing defects inside that prototype's scope;
- writing journal entries and evidence;
- moving on to the next prototype when the current one closes, whatever its outcome, unless a
  stop condition in §6 applies.

## 4. Gates, registered before each prototype

1. **Correctness and safety (mandatory).**
   - Byte-exact `rec` and `raw` against the oracle on the corpora, adversarial and full synthetic
     suites.
   - Unchanged walker output and marker restoration.
   - Malformed-count rejection with empty arrays.
   - Reuse across sizes.
   - A written release-mode argument covering initialisation, bounds and pointer validity.
   - Targeted Miri on the prototype's new paths, with exact coverage stated. A timeout is partial
     coverage, not a pass.
   - Where a new invariant is introduced, a mutation check showing the tests catch its violation.

   **Failure means STOP for that prototype.**
2. **Machine code (diagnostic, registered in advance).**
   - The compiled hot-path instruction, load and store counts against the prototype's stated
     target.
   - Per-row loops and call sites unchanged unless the prototype is meant to change them.
   - A miss is recorded as such and the gate is never relaxed. It stops that prototype's
     iterations once its three attempts are used, but does not by itself stop the sequence.
3. **Net work (the structural criterion).**
   - The revised whole-constructor ledger from compiled paths and counters, against the
     prototype's registered minimum net saving.
   - Below the minimum, the change is not carried into the cumulative stack.
4. **Economics (outside this charter).** Only a separately authorised, formally VALID
   measurement decides the registered CPU and completion ceilings.

## 5. The pre-registered sequence

Each prototype is a single change on top of frozen v2, so its saving can be attributed. A
combined build that stacks the changes which passed gates 1–3 is assembled once at the end.

| order | prototype | compile target (diagnostic) | registered minimum net saving |
|--:|---|---|--:|
| 1 | scanner/tile cleanup (report 227 §D.2), after the handoff review | the redundant check sequences named in the handoff are gone; per-row loops otherwise unchanged | 60 M instructions |
| 2 | hot descriptor pointer arrays (report 227 §D.3) | scalar append ≤ 10 instructions; all added non-row work counted | 40 M instructions |
| 3 | SSE2 first-chunk refinement of v3's finaliser | fast-finish body ≤ 40 (≤ 41 with call); ≤ 10 explicit reads | 4 M instructions on top of v3 |
| 4 | the combined build: v3's finaliser plus every prototype that passed gates 1–3 | each component's compiled paths reproduced in the combined listing | sum of components, minus integration effects |

The minimums are set below each prototype's conditional estimate, so that a substantially
smaller real saving still clears them while a negligible one does not. The guardian may change
them on approval.

## 6. Stop conditions for the whole sequence

The sequence stops and reports early if any of these happens:
- a correctness or safety failure that can't be repaired within the prototype's scope;
- a finding that breaks a frozen-representation assumption;
- two consecutive prototypes below their minimum net saving;
- any step that would need something outside §2.

## 7. Reporting

- **Checkpoints:** at the end of each prototype (one journal entry plus a short summary), and at
  the end of the sequence. Nothing in between.
- **Each checkpoint gives:**
  - the pin and the frozen files unchanged;
  - correctness, safety and Miri coverage;
  - the compiled counts against target;
  - the net ledger;
  - each gate's outcome;
  - limitations;
  - what comes next under the charter.

## 8. Independent review

- Adversarial reviews are of completed checkpoints, not intermediate steps.
- Reviewers read the repository directly at a named commit, and may push their reports and
  scripts to a branch of their own.
- The coordinator verifies, archives and replays them, as it did for the evidence of report 235.
