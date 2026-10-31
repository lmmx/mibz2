# Proposed implementation prompt: one room-walk transformation, as a research arm

*(Send only after Astra returns a single transformation, not "no". Fill in the `«…»`
fields from Astra's answer verbatim. If Astra says "no", this prompt is not used.)*

---

You are implementing one pre-specified change to lmmx/mibz2's inverse-BWT room walk,
`src/bwt.rs::room_segments::<32, PF_L1>`, as a **research arm**. You are not building a
production change. Use no other optimisation ideas.

## The change (fixed; do not redesign it)

| Field | Value |
|---|---|
| Transformation | «Astra's transformation, source-level» |
| Expected disassembly signature per lane-step | «instructions / macro-ops / loads / stores / k unrolled?» |
| Conservative estimate (7763 basis, CI run 40579732655, room walk 4.49 c/B) | «c/B and assumptions» |
| Falsification criterion (registered now) | «criterion» |

## Rules

**Attribution (non-negotiable):**
- Every commit is authored and committed by `Louis Maddox <louismmx@gmail.com>`. Set
  `git config user.name "Louis Maddox"` and `git config user.email louismmx@gmail.com` in
  your clone before the first commit.
- No AI or tool attribution anywhere: no `Co-authored-by`, model names, tool names,
  session links or generated-by lines in commit messages, PR titles or bodies, comments or
  files.
- Before pushing, check every commit with
  `git log --format='%an <%ae> | %cn <%ce>%n%B' origin/master..HEAD`.

**Scope:**
- Branch `research/room-walk-«slug»` from current master.
- Don't modify PR #117, `research/p74-c-implementation` or `research/p74c-decoder`.
- Don't change production dispatch: the shipped walk stays the default and byte-identical.
- No allocator changes. No T-vector format change. No gathers unless the change above
  says so.
- No Hugging Face Hub access of any kind.
- Don't merge, release or publish anything.

**The arm:**
- Add the new walk as a separate generic or function, for example
  `room_segments_«slug»::<32, PF_L1>`. It is reachable only from the `phases` example and
  the stage bench, through a new arm label. The old walk is untouched.

**Correctness:**
- The arm's output equals the shipped walk's byte for byte on:
  - every block of the bench corpora;
  - the existing walk tests;
  - the room-walk Miri suite (`bwt::tests::miri_*`, run by `.github/workflows/miri.yml`), extended with
    cases that hit the new code's own edges:
    - a segment boundary at every lane position and tile offset;
    - the last segment ending mid-pass;
    - growth (`grow`) mid-pass;
    - ns < L.
- Add a `proptest` or exhaustive small-n differential test against the shipped walk.

**Structural audit, before any timing:**
- Disassemble the arm in the bench binary (`objdump -d --no-show-raw-insn -M intel`).
- Record per lane-step: instructions, loads, stores, prefetches, and whether k is
  unrolled.
- Commit the listing excerpt and the counts. If they don't match the expected signature,
  stop and report. Do not tune toward it.

**Measurement (CI only):**
- Don't benchmark locally beyond a smoke run. No i9 runs.
- Use the existing paired stage bench (the same harness as runs 40579732655 and
  40573190665): shipped vs arm, same binary, 20 paired rounds, rotated order, one copy
  and four copies.
- Also run the lane sweep L ∈ {8, 16, 24, 32, 48} for the arm.
- Report walk c/B, its ratio to the shipped walk with a 95% interval, stage total, and
  CPU model per job.

**The decision is mechanical,** by the criterion above:
- pass: write it up as a candidate for a separate, pre-registered decoder-level trial;
- fail: write it up as negative. No retuning, no second variant.

**Write-up:**
- `docs/journal/2026-11-XX-room-walk-«slug».md`: hypothesis, signature (expected and
  actual), correctness evidence, bench figures with run IDs, verdict.
- Label every figure measured, deduced or model.

**Report back:** branch and commit SHAs, CI run IDs, the audit counts, the bench table, the
verdict and anything that surprised you.
