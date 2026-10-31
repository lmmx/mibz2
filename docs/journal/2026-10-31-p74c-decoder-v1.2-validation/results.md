# P74-C decoder trial, harness v1.2 (amendment 2): validation (nothing timed)

**The trial at v1.2:** head `33e0618b1e70190f4e8390eb69ebe2778142fc77`, pin `b406a3b`.

**1. Targeted tests** (`cargo +1.90.0 test --release --example p74c_decoder lifecycle`):
the four tests pass.
- A vanished directory is `EXITED`. A directory with no status, or an empty CPU list, is
  `UNRESOLVED`. A readable status gives the live record.
- The check fails on `EXITED` in a live-only snapshot, on `UNRESOLVED`, on an empty
  record and on an out-of-allocation mask, and passes on `EXITED` in the end snapshot.
- **The race reproduced:** 3,000 snapshots taken right after joining three threads gave 50,
  59 and 66 `EXITED` records over three runs. None was `UNRESOLVED` or empty.

**2. CI integrity run 41242541582:** three `ubuntu-24.04` jobs, `P74C_STOP_BEFORE_TIMING=1`,
so not timed.
- All three are AMD EPYC 7763. The binary is `6ee553f0…`, identical in all three.
- **In every job,** these pass:
  - the 25 sources and every corpus hash;
  - `bzip2 -t` and `-dc`;
  - `--check` at w = 1, 2;
  - the selector audit;
  - the P74-C tests;
  - the structural audit.
- The synthetic self-test ran its full trial path: 1,008 children per job, correctness
  and confinement true.
- **Exited records in those 3,024 children: 0.** The race didn't occur there. So this run
  shows the v1.2 runner end to end, and the targeted tests show the race handled.

**3. Local dry run** (4-core preparation VM, rustc 1.90.0, binary `ed11d9bb…`):
- every check up to timing passed;
- `--check` gave w + 3 threads in all 272 decodes (34 corpora × 2 arms × w = 1–4);
- the synthetic self-test ran 2,016 children, correctness and confinement true;
- 0 exited records.

**The i9 run uses v1.2:**

    git fetch origin && git switch research/p74c-decoder && git pull --ff-only
    git rev-parse HEAD   # 33e0618b1e70190f4e8390eb69ebe2778142fc77
    just bench-p74c-decoder i9-10900K
