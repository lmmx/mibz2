# Compact30: the i9 dry run of the construction-cost harness (reports 198–199)

**Authorised (report 198):** exactly one untimed dry run on the i9-10900K, with the pinned
recipe (`.just/bench.just` at `e4ffc8d`, harness pin `14d9ecb`, protocol draft 4):

    just bench-c30-construction target/walk-decoder/work/corpora 20 dry-run

**Not authorised:** the 20-round capture, any retry after a failure, and any change to
thresholds, CPU selection, the harness, the constructor (`e003a32`) or the walker.

The archive's path and sha256 are recorded here when the run returns, for the guardian's
go/no-go decision on the capture. The recorded clock readings are not performance results.

## Report 198: the guardian's decision (as relayed)

Guardian decision: the validation repair is accepted. I authorize one untimed dry run on the i9-10900K, but not yet the 20-round construction-cost capture.

I reviewed the repaired code at `14d9ecb`, protocol draft 4 at `e4ffc8d`, and the archived VM results. The findings that previously blocked progress have been addressed without changing the audited constructor or its walker.

| Checkpoint | Disposition |
|---|---|
| Safe `TVector` accessors | Accepted |
| G2 individual-step synchronization rule | Accepted |
| G9 verified dry-run evidence | Accepted |
| G7 initialization-coverage reporting | Accepted |
| Machine-code identity | Passed in archived VM validation |
| VM dry run and failure injections | Passed in archived records |
| i9 untimed dry run | Authorized |
| i9 20-round capture | Not yet authorized |

The archived VM analysis reports all applicable schedule and non-timing gates passing, with 434/434 blocks covered by the separate poison-validation pass. The evidence tests also rejected all 21 malformed or mismatched cases.

This is enough to proceed without another design or code-review cycle.

### The instruction

Guardian review complete.

I accept the bounded validation repair at `14d9ecb` and construction-cost protocol draft 4 at `e4ffc8d`.

The safe `TVector` accessors, per-step G2 gate, verified G9 evidence, honest G7 poison coverage, memory diagnostics and mixed-step reporting resolve the outstanding harness findings.

The archived VM evidence supports moving to the workstation. The accepted constructor remains frozen at `e003a32`, and both timed builders' normalized machine-code listings remain unchanged.

**Authorization: run exactly the registered untimed i9 dry run.**

Use the pinned recipe:

`just bench-c30-construction target/walk-decoder/work/corpora 20 dry-run`

Before running, confirm the recipe selects four distinct physical cores on the i9-10900K, the correct rustc 1.97.0 toolchain, the frozen source pin and the hash-checked primary corpus. Allow the registered memory preflight and other checks to determine whether the run can proceed.

Archive the complete dry-run evidence, including its JSONL records, analysis, manifest, binary and corpus hashes, CPU placement, memory readings and machine-code listing checks. Report the dry-run archive's full path and SHA-256 so it can serve as verified evidence for a future capture.

The applicable dry-run schedule and correctness gates must pass. G1, G2, G5 and the performance decision are intentionally suppressed in dry-run mode; do not present the recorded clock readings as performance results.

If the dry run fails, archive the failure and report the specific reason. Do not automatically retry, change thresholds, adjust CPU selection, modify the harness or clean unrelated files.

**Do not run the 20-round capture.** Return the complete i9 dry-run result for a final guardian go/no-go decision.

No constructor changes, walker changes, tuning, production integration or new experimental arms are authorized.

If that dry run passes, we'll have the evidence needed to authorize the actual registered measurement. If it fails, we'll know precisely which condition stopped it.

## Report 199: the researcher's note (as relayed)

This is a good checkpoint. The guardian has accepted the repairs addressing the issues I raised, and the experiment can advance without another design or code-review cycle. The important thing is that the distinction between the two approvals remains intact:

- Authorized: one registered, untimed i9-10900K dry run, with complete evidence archived.
- Not authorized: the 20-round construction-cost capture, tuning, retries or changes to the frozen constructor.

The next evidence worth examining is the actual i9 dry-run archive: its schedule, CPU placement, correctness checks, memory preflight, machine-code identity and complete JSONL record. The recorded clock readings must not be interpreted as performance results.
