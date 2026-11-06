# Compact30 protocol review: synthetic gate checks

Reviewed harness/analysis pin: `b12b49b44e27b96c8e16e0ccab4e348c1ba56bba`.
Reviewed launcher: `.just/bench.just` at `65f7169`.

No decoder was built or executed. No benchmark, PMU measurement, workstation setting change,
or repository modification was performed. These files exercise only the published Python
analysis on invented records.

## Source identity

The two Python source files were retrieved through the GitHub connector. The local copies
match the repository's Git blob SHA-1 values exactly:

- `c30_traversal.py`: `3c560a96f0f6a807f45aa1c55805bb260a85572f`
- `test_c30_traversal.py`: `a0fb65cebc59028073ad05d97670ac11cac7906f`

`review_gate_checks.py` verifies these identities before importing them.

## Reproduce

```sh
python review_gate_checks.py
```

The script runs the original fourteen unit tests, then five adversarial synthetic record
cases. `gate-review-results.json` contains the outputs.

At the reviewed pin, all original tests pass. All five malformed/insufficient synthetic
cases are nevertheless accepted as valid and receive ADVANCE:

1. One measured pair, rather than the registered twenty pairs (zero-width interval).
2. W4 worker identities shifted away from their declared block-list owners, with matching
   CPU identity fields.
3. Candidate W4 row counts doubled without changing block-row metadata: a synthetic
   STOP at cost ratio 0.95 becomes ADVANCE at 0.475.
4. A declared full quartet split into a three-worker step and singleton step; another
   valid full quartet remains, satisfying the current aggregate skew check.
5. Next-step preparation moved into a preceding traversal window; within-step checks pass.

These demonstrate omissions in analysis validation. They do NOT establish that the current
Rust assignment actually produces these malformed records.

## Source-review findings not simulated by this script

1. `Worker::run` records the traversal end, then compares `out` against `want`. The W4
   completion barrier occurs only after this comparison and a record-vector push. An
   early finisher can therefore validate and allocate while a peer still traverses.
2. Traversal windows use `std::time::Instant`. `cpu_now()` calls `sched_getcpu()` and records
   a CPU identifier; it does not measure thread CPU time. The analysis sums elapsed windows.
3. A panic from an output assertion or pinning assertion may leave other threads waiting
   for an unreachable four-party barrier. A coordinated failure path or prompt process
   termination needs an injected-error test, with evidence preserved.

## Recommended scope

Hold the i9 capture. Keep the walker unchanged. Repair completion-phase ordering, make
CPU cost versus elapsed-worker-time explicit, and validate records against the fixed
registered assignment/round count. Then recheck the harness and freeze a new pin.

## Small numerical qualification

The exact unchanged-constructor screen for the stated model is
`1 - 0.25/1.847 = 0.8646453708716838`.
At the rounded policy screen `0.865`, `b_max = 0.9988935810810811`.
Retaining 0.865 as a rounded investment rule is defensible; it does not literally imply
`b_max >= 1` at every accepted upper bound.
