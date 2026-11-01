# P87's four-core diagnostic: the freeze record

**Evidence cutoff:** 1 November 2026, before any timed P87 experiment. Nothing here is a
performance result.

## The freeze

- **Pin:** `6463e443360c6ad77d866edaadbe9253eb47a847` (`bench/p87-pin`): the arms, the
  harness, the audit, the analysis, the runner and the protocol
  (`../2026-11-01-p87-four-core-protocol.md`).
- **`frozen.json`:**
  - the 25 harness sources' SHA-256;
  - the three corpora, copied from the dispatch diagnostic's frozen record, which are the
    walk trial's files, byte-identical;
  - the schedule;
  - the thresholds;
  - the base commit `45b366d` for E3.
- **Toolchain:** rustc 1.97.0, `--locked`, release profile, empty RUSTFLAGS.
- **The runner refuses to start** if:
  - any source differs from its frozen hash;
  - HEAD differs from the pin outside `bench/p87-pin` and this directory;
  - the tree is dirty;
  - the label's results already exist.

## The i9 command

```
just bench-p87 i9-10900K
```

It asks for sudo once, for the counters. The default corpus directory is the walk trial's
(`target/walk-decoder/work/corpora`); set `P87_CORPORA` to point elsewhere. The modelled
budget is about 10–12 minutes, about 4 of them S1 (protocol §10). The archive is written
to `target/p87-results/i9-10900K.tar.gz`.

## The untimed dry run on the preparation VM (`dry-run-preparation-vm/`)

The VM has 4 CPUs, so the allocation includes CPU 0's core. Its perf counts neither the
memory-controller events nor the core events. The run was
`P87_DRY_RUN=1 PERF=… bash bench/p87_run.sh preparation-vm`, and it exited 0.

**What held:**
- **Sources and pin:** all 25 sources matched; HEAD equalled the pin.
- **Tests:** the arms' (5), the harness's (4), the trace's (2) and the analysis's (13)
  passed.
- **E3:** the default-feature decoder disassembles the same as at `45b366d`.
- **E1:**
  - the timed binary's worker body holds one `p87::arm` call and two call sites of each
    kernel;
  - there is no out-of-line `p87::decode_block`.
- **E2:** all 12 shared functions are identical to the default build's. The phase
  instrument passes E1 and E2 too.
- **`--check`:** all three corpora decode to their frozen SHA-256 in arms A and B and on
  the shipped path, at w = 1 and 4 and sequentially.
- **The harness self-test** (S1's whole path, every figure synthetic) passed. The
  verdict's path was exercised on it (`verdict-synthetic.json`, not a result).
- **S2** at 2 repetitions: 16 runs, every output the instrument's reference.
  - The marks were complete and in order: 434 blocks; 207,006,515 rows; the decoded bytes
    summed to exactly 207,000,624; rows and blocks were the same in both arms.
  - The perturbation check refused interpretation (shared VM, 2 repetitions: medians from
    0.93 to 1.15). That is the intended behaviour.
- **S3:** both passes were probed and recorded unavailable. No pass ran, and none was
  replaced.

**Not done:** S1's timed trial, on any host.

## Validity concerns carried to the review

These are the protocol's §9, plus one from the dry run:

- **Trace perturbation on the i9.** The 3% perturbation bound on S2 is untested on the i9.
  If the level-2 marks cost more there, the phase figures will be reported as not
  interpretable. S1's decisions don't depend on them.
