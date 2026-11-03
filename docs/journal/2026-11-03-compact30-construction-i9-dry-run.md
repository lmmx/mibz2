# Compact30: the i9 dry run of the construction-cost harness (reports 198–199)

**Authorised (report 198):** exactly one untimed dry run on the i9-10900K, with the pinned
recipe (`.just/bench.just` at `e4ffc8d`, harness pin `14d9ecb`, protocol draft 4):

    just bench-c30-construction target/walk-decoder/work/corpora 20 dry-run

**Not authorised:** the 20-round capture, any retry after a failure, and any change to
thresholds, CPU selection, the harness, the constructor (`e003a32`) or the walker.

The recorded clock readings are not performance results.

## The result: the dry run passed

Run on the i9-10900K on 2026-11-03 at 17:21 UTC, in the checkout `~/dev/mibz2` at
`e4ffc8d`, as relayed from its console:

- **The archive**, the evidence a capture on this machine needs:
  `/home/louis/dev/mibz2/target/c30-construction-20261103T172121Z-dry-run.tar.gz`,
  633,111 bytes, sha256
  `f75b00e81b95b14559005cd57b1428a990de00547ddcbed48e4b71f7b240f9f9`, 33 files and
  `MANIFEST.json`, commit `14d9ecb`.
- **The preconditions, checked by the recipe before any build:**
  - the corpus `wikidata-20261026.bz2`, sha256 `cf13f439…16b8fe`: OK;
  - the frozen commit `14d9ecb` in its own worktree;
  - rustc 1.97.0 (`2d8144b78`, 2026-07-07), LLVM 22.1.6, cargo 1.97.0, no rustflags;
  - the analysis tests (94 passed) and the listing tests (6 passed);
  - the timed path's machine code is the reviewed listings (G3: yes).
- **CPU placement:** workers on CPUs 1, 2, 3 and 4, one per physical core (the second to
  fifth of ten), W1 on CPU 1. The recipe's selection matched the read-only preview run
  beforehand. The barrier test passed: 12,000 steps in 3 rounds × 2 arms, 4 workers, uneven
  arrivals and partial final steps, every check passed.
- **The collector** ran rounds −1, 0 and 1 as a dry run and exited 0. The six peak-RSS
  processes ran (three per arm).
- **Schedule:** S0–S10 all true (no failure, no unknown records, meta, rounds, record set,
  placement, windows, W1 sequence, start and completion barriers, cross-step, pass order,
  calibration, preamble).
- **Gates:** G3 identity, G4 allocation, G6 exact output, G7 round −1, G8 one-pass, G9 memory
  and G10 major faults all true. G1, G2, G5 and the decision are suppressed in a dry run, as
  registered.
- **Allocation (W4):** rows 51,301,938 / 51,301,991 / 52,201,043 / 52,201,543; blocks
  109 / 109 / 108 / 108; full blocks 57 / 57 / 58 / 58.
- **Memory:**
  - estimate 1,855,326,234 bytes, required 1,855,326,234, margin 2,147,483,648 (2 GiB);
  - `MemAvailable` at the preflight 121,943,801,856 bytes;
  - peak RSS (VmHWM) 1,769,914,368 bytes, 0.954 of the estimate;
  - preflight and postflight OK, no problems recorded.
  
  `verified_peak` is 0 because a dry run consumes no evidence.
- **Faults:** no major faults, in the measured builds or in round −1. Minor faults, reported
  only, are 0 in warm P and C, 1,658 in cold P and 102 in cold C.

The checkout's untracked files (earlier i9 outputs) don't affect the run: the recipe builds
from its own clean worktree of the pin.

**The archives**, relayed and kept in
[2026-11-03-compact30-construction-i9-dry-run/](2026-11-03-compact30-construction-i9-dry-run/):
- `c30-construction-20261103T172121Z-dry-run.tar.gz`, sha256 `f75b00e8…40f9f9`, matching
  the console. The pinned analyser (`bench/c30_construction.py` at `14d9ecb`), replayed here
  on its own records and RSS files, reproduces its `c30-analysis.json` exactly.
- `c30-construction-20261103T172410Z-verify-evidence.tar.gz`, sha256 `0fded68c…053573`:
  `verify-evidence` run on the i9 against the dry run's archive. The preconditions held as in
  the dry run (corpus, pin, toolchain, 94 + 6 tests, G3 yes, CPUs 1–4, barrier test), and the
  collector verified the evidence and built nothing (exit 0).

## Report 200: the guardian's GO (as relayed)

**Guardian disposition: VALID DRY RUN. One registered 20-round i9 construction-cost capture
is now authorized**, using that exact archive (sha256 `f75b00e8…40f9f9`) as verified
evidence:

    just bench-c30-construction target/walk-decoder/work/corpora 20 capture target/c30-construction-20261103T172121Z-dry-run.tar.gz

One capture only. No changes to the source, walker, constructor, protocol, thresholds,
corpus, CPU selection or allocation regimes. If evidence verification, start
synchronization, clock calibration, stability, correctness or any other registered gate
fails, the failure archive is preserved, with no automatic rerun or threshold adjustment.
The warm four-core decision stays fixed: `b4` upper 95% CI ≤ 1.06714 and `e4` upper 95% CI
≤ 1.02 for PASS. After the capture: independent analysis and archiving of the complete
results (both warm deciding ratios with their intervals, the cold and one-core diagnostics,
the validity gates, memory and fault records, and the implied construction economics),
returned for guardian review. No integration into production on the strength of this
experiment alone.

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
