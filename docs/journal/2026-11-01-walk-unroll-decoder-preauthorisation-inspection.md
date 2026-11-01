# The unrolled room walk's i9 trial: inspection of the frozen protocol before authorisation

**What this is:**
- It is a read-only inspection. Nothing is run, rebuilt, retuned or re-registered.
- It adds no criterion and changes no threshold.
- The frozen branch (`research/walk-unroll-decoder`, head `f5c492d`, pin `df90ba8`) is
  untouched.
- **What it reads:**
  - the registered protocol (`2026-10-31-walk-unroll-decoder-protocol.md`);
  - amendments 1 and 2, `freeze.md` and `frozen.json`;
  - the runner, the summariser and the audit;
  - the two exploratory CI runs' records;
  - wikidata-pq master's decoder call.

## 1. The frozen hypothesis

**The hypothesis:** `bwt::SegmentedUnrolled`, the shipped `Segmented` with only its room
walk replaced by the constant-index 32-lane walk, selected through
`StreamDecoder::with_inverse::<SegmentedUnrolled>(w, 2w, Kernels::detect())`, raises
full-decoder throughput.

**The primary gate:**
- **The measure:** B/A on `wikidata-20261026` at w = 4 on the i9-10900K.
- **PASS:** lower 95% bound ≥ 1.02.
- **NEGATIVE:** upper < 1.02.
- **INCONCLUSIVE:** otherwise.

**The stated prediction (model, registered before measurement):**
- +3% to +6%, from the stage saving of 0.43–0.77 c/B measured on EPYC 7763 (one copy);
- +1% to +3% with SMT siblings busy.

## 2. What it targets

| | Frozen scope |
|---|---|
| Decoder path | `Plan::Workers(w)`, the `with_inverse` call `planned` makes; segmented blocks (≥ 65,536 rows); the plain-walk fallback below that is shared |
| Worker counts | w = 1, 2, 3, 4 on four physical cores, siblings idle; primary and CPU gate at w = 4; controls and RSS at every w |
| Deciding host | i9-10900K only |
| Exploratory hosts | CI, w = 1, 2 (done: runs 41235849514 and 41243689201) |
| Not in scope | `Plan::Teams` (`coop::walk_room`); single-threaded `decode` and `BzReader`; P74-C |

## 3. Does the acceptance match what would ship?

**The primary matches wikidata-pq's release configuration:**
- wikidata-pq master decodes with `bz2_threads() = min(physical_cores, 4)`
  (`BZ2_THREADS_CAP = 4`, `src/wikidata/dump.py`).
- `plan_with` takes `cores = min(threads, phys)`, and that is below `COOP_MIN_CORES`
  (6). So the release runs `Plan::Workers(≤ 4)` on every machine, the i9 included.
- The trial's w = 4 primary is that configuration on any machine with four or more cores.

**The controls (every w) and the CPU gate (every corpus)** protect mibz2's general users,
as a published library default should.

**One scope point, read from the existing protocol (not a new criterion):**
- The protocol says a PASS "proposes a default-change PR", without naming its scope.
- **The trial measures only the `Plan::Workers` path.** A PR that changed `Segmented`'s
  own walk would also change three paths the trial doesn't measure:
  - the single-threaded `decode`;
  - `BzReader`;
  - the small-block fallback of team plans.
- **The consistent reading:** a PASS supports a default for the `Workers` path only, the
  kernel `planned` passes to `with_inverse`.
- **When it matters:** this is for the maintainer to confirm before any default PR. It
  doesn't block the trial.

## 4. Existing evidence bearing on the likely outcome (not criteria)

| Exploratory CI host | wikidata-20261026 w1 | `records` w1 | `records` CPU B/A w1 |
|---|--:|--:|--:|
| EPYC 9V45 (Zen 5) | 1.179 [1.151, 1.207] | 1.003 [0.983, 1.023] | 1.000 |
| EPYC 7763 (Zen 3) | 1.081 [1.072, 1.090] | **0.948 [0.942, 0.953]** | 1.041 |
| EPYC 9V74 (Zen 4) | 1.017 [1.001, 1.035] | **0.917 [0.912, 0.923]** | 1.067 |
| Xeon 6975P-C (Intel) | 1.002 [0.999, 1.004] | **0.928 [0.924, 0.933]** | 1.055 |

*(Measured.)*

**What each line of evidence implies:**
- **Gate 3, the controls at every w,** is the likely decider.
  - On three of the four hosts, `records` at w = 1 has its upper bound below 0.97, which
    is NEGATIVE under gate 3.
  - Its CPU-s/GB is +4% to +7% there, so gate 4 would also be at risk if this
    reproduced at w = 4.
  - If the i9 reproduces it, the trial is NEGATIVE whatever the primary shows. The status
    note's open question, what causes the `records` slowdown, is uninvestigated.
- **No stage-level measurement exists on Intel client cores.**
  - The stage PASS (run 41222735026) was on EPYC 7763, with a Zen 4 replication.
  - The i9 is a Comet Lake (Skylake-family) core. The only Intel data is the Xeon CI
    host, where Wikidata was neutral.
  - The prediction is carried over from Zen 3.
- **The w = 4 primary cell is noisy on this host.**
  - In the P74-C i9 run, the same cell had A′/A 1.025 [1.005, 1.044], and the fourth
    worker added only 3% throughput over w = 3 (that run's postmortem §3).
  - A lower bound of 1.02 is demanding against a +3% to +6% prediction.
- **The W4 (c) inlining difference** is an accepted, disclosed confounder (amendment 1).
  A result measures the integrated decoder as compiled.
- **RSS:**
  - Unlike P74-C, B doesn't alternate kernels per block. Every segmented block runs the
    unrolled walk, with the same `TVector` and `SegScratch`. So the capacity-contract
    defect class (`docs/p74c-decoder-validation-records`,
    `2026-11-01-buffer-capacity-contract.md`) shouldn't arise.
  - On CI, the RSS gate was INCONCLUSIVE through the max rule only. For example, Xeon
    `records` w2 read 1.068 and `period-3` read 1.037–1.042, while every paired lower
    bound was ≤ 1.00.

## 5. Correctness checks and prerequisites

**Complete, as frozen:**
- **Gate 0:** every child's output SHA-256 against the frozen hash; every plan
  `Workers(w)`; every thread inside the allocation, with amendment 2's three-way
  lifecycle classification.
- **The pre-timing steps, in the runner:**
  - corpus hashes and the frozen sources' SHA-256;
  - `--check` at w = 1–4;
  - `cargo test --lib unrolled` and `--test segmented_unrolled` (rustc 1.97.0);
  - the structural audit W1–W4, with the exact `w4_review` signature required;
  - the harness self-test.
- **The walk's Miri handoff suite** passed 11 of 11 on the stage branch
  (`miri_room_walk_unrolled_handoffs`).
- **The lifecycle fix** was validated by CI run 41243689201: three jobs, all complete.
- **The P74-C i9 trial is done.** The protocol requires it before this one.

**One records erratum (metadata only; it doesn't affect the build):**
- `frozen.json`'s `toolchain` field reads "rustc 1.90.0, --locked, release profile,
  empty RUSTFLAGS".
- It is a string copied from the P74-C harness (`bench/walk_decoder.py:176`).
- **The runner builds with rustc 1.97.0,** as amendment 1 §2 accepts:
  `walk_decoder_run.sh:58–61` sets `RUSTUP_TOOLCHAIN=1.97.0` and `cargo +1.97.0`. It
  records `rustc -vV` in `environment.json`.
- **How to handle it:** correcting it in `frozen.json` would change the frozen sources'
  hashes. So it is recorded here, and the frozen branch is left as it is.

**Missing before a run:**
- **No i9 run card exists for this trial.** P74-C had one, at
  `docs/p74c-decoder-validation-records` `e8155f3`.
- The recipe exists: `just bench-walk-decoder i9-10900K`. The card would hold the
  command, the expected steps and outputs, and the archive to return.
- It is documentation only, and can be written without changing the frozen branch.

**Expected duration (deduced):** shorter than P74-C's 903 s timed trial.
- About 2,800 child processes, against P74-C's 8,568.
- The same three Wikidata corpora dominate the bytes.

## 6. Status

- The protocol is complete, frozen and executable as it stands.
- The i9 trial stays unrun until the maintainer authorises it.
