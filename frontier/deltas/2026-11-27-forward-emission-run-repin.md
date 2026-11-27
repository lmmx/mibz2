claim: forward-emission-run
by: claude-code
outcome: partial
evidence: PR #1168; docs/journal/2026-11-27-forward-emission/registration-disjoint-slabs.md; #1217 (5767825); v0.5.17 (01acd74d)

## What changed and why
Took over #1168's integration (antigravity's lease had lapsed). The recipe was pinned to #1110's
four-lane walk (`7a7a207a`), which #1217 has since replaced in `src/open/lanes.rs`. Running the
old pin would spend an i9 slot deciding code nobody would ship, so the #1110 arm is **superseded,
not run** (a disposition, not a verdict: no dead-end row). Under `registration.md` §4 a new
registration was frozen before any i9 data: both arms at v0.5.17 (`01acd74d`), `base` default
features and `cand` `--features forward-anchor-lanes`. One commit for both isolates the feature.
Every §3 threshold, §2 cell and §5 forced route is unchanged (C2). This is a C1a kill test, not an
adoption registration, so no C6 review is required; independent integration review was done here.

## Review of antigravity's harness (kept)
Exact-commit shallow fetch, isolated target dirs, distinct-binary guard, per-repeat SHA-256 via
`a6b_endpoint_run` (forced lazy route; a second decode is hashed), warm paired-oracle `S ideal` with
the <= 5% scan-residual gate, AB/BA interleave, bootstrap CI of the median paired ratio, KILL
evaluated before PASS, VOID for control/harness failure. Only change: the registrations now copy
from the recipe checkout (the re-pin is not in v0.5.17's tree).

## Verification
- `python3 -W error -m unittest test_forward_lanes` (from `bench/`): 14/14, including a new test
  that both registrations ship and name the pin and the frozen bounds.
- Build of every recipe binary at `01acd74d` (default, `forward-anchor-lanes`, trace +
  `scan-oracle`) with Rust 1.99.1 in a VM, through `build_binaries` and
  `assert_distinct_binaries`: all build, digests distinct. Smoke: both arms' `a6b_endpoint_run`
  at k = 1 on `text` match libbzip2's SHA-256 (`8329e51e…`).

## Next (result to release)
Merge -> `ready/bench-forward-lanes` at the merge commit (run-signal.yml) -> queue entry (queue-only
PR) -> read the bundle against §3. PASS -> a C6 adoption registration for making the walk the
open kernel's default; RESCUE -> vectored phase B (`forward-rle1-crc-vectored-build`); KILL ->
dead-end row for this arm.
