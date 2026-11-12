# One-core inlining on the i9 — bounded diagnostic

Registered 10 November 2026, before this contrast's workstation data. This uses the
standing release-scaling recipe template (RULES C5), not a new statistical method.

## Question and fixed inputs

Does the already-reviewed one-core inlining patch in #361 recover the i9's
one-thread Wikidata cost of release 0.2.9?

- Candidate: `d33787e138426cb441db6975f54b2eccca02e89a` (#361).
- Baseline: `b65b2aa54488f072cf005b8fe62461c32e6ef083` (v0.2.9).
- Driver: unchanged `bench-release-scaling` from master
  `2635f04e9155dd31388b24fd3f7248f87b3ddcda`.
  Its recipe stanza (header and indented body, final blank lines trimmed, one
  final newline) has SHA-256
  `6167171b7889706226340adc0beba89bfbf5e933af944d7f68e49d97bc7bfa07`.
  The untimed check pins this body, so a later template edit cannot silently
  change this prepared run.
- Recipe: `just bench-onecore-inline-i9`: 10 paired rounds, Wikidata, one thread;
  the existing held-spinner behavior, output-hash checks, metadata and bundling.
- No thread-cap, team threshold, default, source layout or benchmark-method changes.
- No repeat of the completed cells stage or team-threshold runs.
- Each attempt is archived with its own timestamp and digest by `just submit`.

## Decision rule and integration consequence

Use the existing summary's paired wall ratio of **baseline / candidate** and its
95% interval. The driver calls `summary --ref mibz2`: `mibz2` is NEW (candidate),
and `mibz2@<OLDAB>` is OLD (baseline). Ratios above 1 therefore favour the
candidate. Do not read this column as candidate / baseline. If converting for
another report, transform `(g, lo, hi)` to `(1/g, 1/hi, 1/lo)`.

1. If its lower endpoint is above 1, the patch recovers measurable i9 one-core
   performance: retain #361 as a live integration candidate and resolve its
   outstanding CI primary-loss evidence before any merge.
2. If the interval includes 1, the targeted recovery is not demonstrated at this
   precision: defer #361 as a performance repair; do not start repeated timing.
3. If its upper endpoint is below 1, reject this patch as an i9 one-core repair.
4. Any byte mismatch, failed decoder, missing paired rows or incomplete bundle
   invalidates the measurement; preserve the failure and diagnose it, never
   call it a performance pass.

This is a diagnostic, not production qualification. It neither rewrites #361's
recorded CI text-2t loss (+5.5%, 95% CI +0.3%, +11.0%) nor replaces its fixed CI gate.
Its authority is the existing handoff and #361 comment 6774987036. No claim that
machine-code identity or a stack-frame change caused the original 0.9% loss.
One run decides this diagnostic. No automatic retry to seek a passing result.

## Preparation and execution gates

The exact #361 source head has six successful Actions workflows and successful
legacy pre-commit. The wrapper's final recipe commit must pass its own required CI
and legacy statuses, and its untimed recipe check before ready-tagging. The exact
recipe commit and validation results are recorded on #145 before execution.

The untimed wrapper check is `ONECORE_REQUIRE_JUST=1 python3
bench/test_onecore_recipe.py`: the real Just parser/dry-run must resolve the frozen
command, shell stand-ins verify exact arguments and failure propagation, and the
driver stanza's digest must match. It does not produce timing evidence or claim a
fresh native parity run; the unchanged decoder/harness retains its separate CI
correctness checks, and the measured run's own hash checks remain mandatory.

Only the Coordinator sets `ready/bench-onecore-inline-i9` through Run signal, at
the final checked 40-character recipe commit. Add the recipe and its branch to
`bench/workstation-queue.txt` only when every preparation gate is met.
Louis starts `just workstation-queue --wait 12`; that is his run authorisation.
The agent does not directly run the i9.

On submission, fetch the pinned evidence archive, verify SHA-256, check identities,
hash parity and all ten paired rounds, then record the diagnostic decision and the
remaining integration gates on #145. Remove the queue entry and clear its tags.

