# C31's evidence archive (docs/journal/2026-11-09-run-fed-direct.md §10, §11)

**Source:** `37c4ff246e449e392ade2ced4f94488c358de087` on `research/run-fed-direct`
(`src/run_stage.rs` blob `cb09d7da817fa467cbdc4664ddf2289e68157d1f`), rustc 1.97.0
(2d8144b78 2026-07-07), x86_64 with AVX2 detected, the shared 4-core development machine.
Correctness runs only: no figure here is a timing.

## fuzz/

Built with `cargo build --profile fuzz --features run-fed --example fuzz` (the fuzz
profile: release, with debug assertions and overflow checks). Each run:

    target/fuzz/examples/fuzz SECONDS SEED OUT_DIR run-fed-direct

| file | command | what |
|---|---|---|
| `seed1-30s.txt` | `fuzz 30 1 … run-fed-direct` | CI's smoke shape |
| `seed2-240s.txt` | `fuzz 240 2 … run-fed-direct` | |
| `seed3-240s.txt` | `fuzz 240 3 … run-fed-direct` | |
| `head-seed1-30s.txt` | `fuzz 30 1 - run-fed-direct` | at `d8a4c6f0` (master merged; the mode within MSRV), the rest at `37c4ff24` |
| `inject-seed1-5s.txt` | `MIBZ2_FUZZ_INJECT=1 fuzz 5 1 … run-fed-direct` | failures made on purpose, to show saving and minimising |
| `inject-witnesses/` | | the two injected failures' minimised witnesses |

The three real runs found no failure, so they saved no input. `MIBZ2_FUZZ_CASE=N` replays
case N of a seed.

## mutants/

`bash docs/journal/2026-11-09-run-fed-direct/mutants/run.sh` from the repository's root,
on a clean `src/run_stage.rs`: each `M*.patch` applied, the run-fed lib tests (and Miri
with AVX2 for M1; the D fuzz mode for 20 s for M2 and M4), the patch reversed; then the
restored tree's tests. `logs/` holds its output: `source.txt`, one log a mutant, and
`restored.log`. Local paths in the logs are written `<checkout>`.

## census.json

The count-only census of a 2-round local validity run of `examples/run_fed_stage.rs`
(§11): identities, path counts and D's predicted table-store census for the seven
corpora. Every timing field was dropped before the record was read. The verifier's
passing fixture takes its counts from it.
