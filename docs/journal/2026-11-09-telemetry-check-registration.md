# The machine-state recorder's own check: registration

9 November 2026. Registered before any use, for Guardian D065 (REVISE on #298), and revised
before any use for D066 (the status of failed reads and controls, below). This is a
bounded diagnostic under RULES C1: an information objective, no performance minimum. It can
qualify nothing. It is independent of the service core's revised registration (#299).

**How it is frozen.** `bench/telemetry-check.frozen` names the commit this registration, the
harness (`examples/ab.rs`'s `machine::check`) and the recipe (`just bench-telemetry-check`) are
frozen at. The recipe refuses to run if a measured input differs from it, or if the checkout has
uncommitted changes. The measured inputs are `src`, the Cargo files and toolchain, `examples`,
`tests/common`, `bench/bundle.sh`, `.just/bench.just` and this file.

## The objective

Whether the opt-in machine-state recorder (`ab run --machine-state`, bench 3.4.0) records
faithfully on the machine it runs on. Each case asks one question:

| case | question | PASS when |
|---|---|---|
| `control-failure` | is a failed enable and disable recorded, and only that counter invalidated? | the counter is null with "enable failed (errno …)" and "disable failed (errno …)", both are in `perf_ctl_errors`, and the other counters are valid |
| `missing-counter` | does an event that can't be opened leave every field null with its reason? | every field is null with its `_why`, never zero |
| `multiplex-predicate` | is `eff_ghz` null for multiplexed cycles or task clock, and in user-only scope? | as stated, on fixed synthetic readings |
| `multiplex-live` | with 24 extra cycle counters (attempted, opened and enabled counted separately), were the record's counters multiplexed, and does the record say so? | no opened counter's control or read failed (else INCOMPLETE, judged first), a valid counter was multiplexed, `perf_multiplexed` lists it, and `eff_ghz` follows its rule. A counter that couldn't be opened is left out. If no valid counter was multiplexed: OBSERVED, "multiplexing not exercised", never PASS |
| `user-only-fallback` | forced into user mode, does the record keep its scope and claim no ratio? | scope `user`, `eff_ghz` null with its reason |
| `task-clock-modes` | does a user-only task clock exclude kernel time here? | never PASS: OBSERVED, a ratio from two valid, whole and nonzero readings (over 50 ms of the thread's CPU, a zero either way is no reading of the flag, so INCOMPLETE), with the thread's own system-time share. Below 20% system time, it says nothing of the flag |
| `region-task-clock` | does the (software) task clock count the region's threads and nothing outside it? | 0.090–0.130 s for 100 ms of CPU in the region, 120 ms outside it, and a thread started before the counters open. **The task clock only:** the hardware counters' region coverage is not validated here |
| `inheritance-{software,hardware}-{counter}` | is each counter, separately, inherited by a thread started in the region? | both regions' readings are inspected first: a control or read failure in either is INCOMPLETE, whatever the other shows; else a counter that couldn't be opened in either is UNAVAILABLE. Then both are valid and whole, and the count with the work (80 ms of CPU) in a thread started inside the region is at least 10 times the count with the same work in a thread started before the counters open (the opener idle in both). Removing that counter's inheritance makes the first the opener's idle count, which fails |

## Statuses, and how each is read

- **PASS:** the property holds on this machine, for this case only.
- **FAIL:** it doesn't. The recorder is wrong here, and its output can't be relied on for that
  property.
- **OBSERVED:** a behaviour shown with nothing it is checked against. It establishes nothing
  beyond itself.
- **UNAVAILABLE:** the events can't be opened here, with the reason. It establishes no
  capability: an all-UNAVAILABLE hardware run shows only that there is no PMU access. It leads
  to no credential, permission or host-setting change.
- **INCOMPLETE:** the case started but couldn't finish validly, with the reason:
  - a failed control operation or read;
  - multiplexing in a case that needs whole counts;
  - failed CPU accounting;
  - a deadline.

  Not a pass.

**Where a reading failed decides its status** (D066): a counter that couldn't be opened is
UNAVAILABLE; one that was opened and then failed its enable, disable or read is INCOMPLETE.
Each failed reading carries its stage (`_stage`: Open, Control or Read) beside its reason.

**Missing values** are null with their reason, never zero. **The check's exit:**
- 1 if any case FAILs;
- else 3 if any is INCOMPLETE;
- else 0.

The recipe keeps that through the bundle and `just submit`: the bundle is always made and
submitted, and the command still exits nonzero for FAILED (1), REFUSED (1) or INCOMPLETE (3).

## Bounds

- Each spin has a wall-time deadline of 10 s, whatever its CPU budget. Past it, its case is
  INCOMPLETE.
- The whole check runs under `timeout 300`. Past it, the attempt is INCOMPLETE.
- The three `ab sample` readings take 2 s.

## Execution identity

- The recipe runs the executable Cargo reports building
  (`--message-format=json-render-diagnostics`), never a guessed path.
- `build.txt` records its path and `rustc -vV` and `cargo -V`. `inputs.txt` records its
  SHA-256, checked again after the check ran.

## Run once

One attempt, through `just submit bench-telemetry-check` (RULES C5's established route). There
is no retry and no retry-to-pass. A FAILED, REFUSED or INCOMPLETE attempt is kept and read,
and what follows from it goes to review, not to an automatic new attempt. A new commit is no
justification for a rerun. Its result bears only on the recorder, never on a decoder's
performance.

## What it does not establish

- The counters' perturbation of a timed decode, and their overhead.
- Hybrid-core coverage.
- Complete thread attribution.
- The hardware counters' region boundaries (only the task clock's are checked).
- Any frequency or performance verdict.
