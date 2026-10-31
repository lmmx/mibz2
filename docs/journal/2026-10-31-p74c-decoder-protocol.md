# P74-C full-decoder trial: pre-registered protocol

**Status:** pre-registration. Committed before any integration code, before any input of
the different-dump corpus was fetched, and before any timing. Not run. Labels: **[M]**
measured, **[D]** deduced, **[E]** estimated.

**Review:** the reviewer's corrections were folded in before this first commit:
- one primary corpus;
- no multiplicity adjustment on conjunctive gates;
- three outcomes per gate;
- the RSS rule;
- the range-fetch guard.

**What is fixed here:** the hypothesis, the arms, the CPU allocation, the corpora and how
each is sourced, the schedule, the statistics, the acceptance rules and the stop rule.

**What is added at the freeze (§11), and nothing else:** every input's SHA-256 and expected
output SHA-256, the source hashes (`2026-10-31-p74c-decoder/frozen.json`), the different-dump
corpus's window hashes and its selector audit, and the pinned commit (`bench/p74c-decoder-pin`).
No rule below is changed at the freeze.

## 1. Background and hypothesis

The stage trial (CI run 41190706676, three hosts; the i9-10900K) measured the frozen P74-C
policy's MTF stage on Wikidata. As plain T2/T0 time ratios [M]:

| Host | Ratio |
|---|---|
| i9-10900K | 0.819 |
| EPYC 7763 (Zen 3) | 0.719 |
| EPYC 9V74 (Zen 4) | 0.722–0.724 |

Its registered outcome was negative. The failing rows were on small bypassed edge files,
plus one control row on the i9. This trial is the separately authorised next step: the
whole decoder.

- **Primary hypothesis H:** X_P74C(4) ≥ 1.02 · X_shipped(4). X is the decoded output's
  throughput through the production parallel decoder at 4 workers.
  - **Tested on one corpus only:** the out-of-sample different-dump Wikidata set
    `wikidata-20261026`.
  - **Secondary replications:** the fitted and same-dump held-out Wikidata corpora,
    reported at every w and never pooled with the primary.
- **Model prediction, descriptive only [E]:** ΔT/T ≈ s_MTF · (1 − r_plain).
  - s_MTF is the plain MTF stage's share of the decoder's time. An old one-thread
    per-phase profile gives about 0.09 (MTF 2.2 of 24.4 c/B,
    `2026-10-27-mibz2-v2.md`). It has not been re-measured on today's walk, and at 4
    workers it is unknown.
  - With s_MTF = 0.09, the prediction is about −1.6% on the i9 and −2.5% on Zen 3 and
    Zen 4.
  - This is *below* the primary gate's 2% (§8.1) on the i9. The gate was set by the
    reviewer, not from this model. The summary reports each corpus's implied
    s_MTF = (1 − 1/R)/(1 − r_plain), descriptively.

## 2. What is under test

The frozen P74-C kernel from `research/p74-c-implementation` at `8b6caf1` (including
`2e11dc1`, the deferred error tag), unchanged:

- the combined plain and checkpoint kernels (`combined::<0>` and `combined::<16384>`);
- the σ bypass: shipped kernel if S < 16384 or 2S ≥ N_max;
- the inclusive one-way guard: switch iff F + 1 > ⌊S_incl/32⌋ + 123.

No code-generation tuning, no change to thresholds, policy or kernel.

**Integration** (off by default):
- `Kernels` gains an MTF kernel choice, `MtfKernel::{Shipped, P74c}`, default `Shipped`.
  It is read once per block in `decode_block`, outside the token loop.
- The research arms (T1, Free, A/A) and Gate 1's selector are removed from the library.
- `Kernels::detect()` and `Kernels::scalar()` keep `Shipped`, so every default path is
  unchanged.

**Plain MTF only.** The default parallel decoder's plan at w ≤ 4 is `Plan::Workers(w)`
with `bwt::Segmented` (`wants_checkpoints()` false). This trial therefore measures the
**plain** kernel. The checkpoint kernel (Cooperative/teams plans) is covered for
correctness only.

## 3. Arms

One executable, `examples/p74c_decoder.rs`, built once. Every measurement is its own child
process of that executable.

| Code | Arm | Kernels passed to the decoder |
|--:|---|---|
| 0 | **A**, shipped | `Kernels::detect()` |
| 1 | **B**, P74-C | `Kernels::detect().with_mtf(MtfKernel::P74c)` |
| 2 | **A′**, shipped again, a second label (A/A null) | `Kernels::detect()` |

**The decode:**
- **API:** `StreamDecoder::planned(Plan::Workers(w), 2·w, kernels)` for w ∈ {1, 2, 3, 4}.
  This is the type wikidata-pq's `Bz2Decoder` uses. It is fed and read as
  `decode_parallel` does: a feeder thread writes 4 MiB chunks, and the main thread reads
  4 MiB at a time into one output buffer preallocated to the expected decoded size.
- **Why `planned` and not `new`:** to fix the plan.
  - The child asserts that `Plan::for_threads(w)` (the default policy, under the child's
    actual affinity) is also `Workers(w)`, so the measured path is the one `new` and
    `decode_parallel` would take.
  - It also asserts that `threads()` is w.
  - It records the process's threads and asserts there are w + 3: w workers, the scanner,
    main and the feeder (§4a).

**What is measured, per child:**
- the input file read before timing;
- **wall:** `Instant` from just before the decoder is built to the read that returns 0
  (every byte out, every CRC checked);
- **decode CPU:** `getrusage(RUSAGE_SELF)` user + sys, all threads, over the same
  interval;
- then, untimed, the output's SHA-256 checked against the frozen expected hash (any
  mismatch aborts the trial), and the decoder joined;
- **from the parent's `wait4`:** the child's whole-life user + sys and its peak RSS
  (`ru_maxrss`, KiB). The whole-life CPU is recorded but not gated.

**Derived:**
- throughput X = decoded bytes / wall (MB/s, 10⁶);
- CPU-s/GB = decode CPU / decoded bytes × 10⁹;
- peak RSS = `ru_maxrss`.

**Caveat [D]: RSS is diluted.** Peak RSS includes the compressed input and the
preallocated decoded output, both identical across arms. The RSS ratio is therefore a
ratio of totals, diluted towards 1. The decoder's own buffers (≈ 7 MB a worker,
2·w slots) are a minority of it on Wikidata.

### 3a. Arm identity (added before the freeze, at the reviewer's request)

The arms differ only in the MTF choice:
- **One binary.** A, B and A′ are the same executable, `examples/p74c_decoder.rs`.
- **One code path.** In `child`, `decode_once(&z, w, arm_kernels(arm), bytes)` is the
  same call for every arm. `arm_kernels` is the only arm-dependent code:
  - it returns `Kernels::detect()` for A and A′;
  - it returns `Kernels::detect().with_mtf(MtfKernel::P74c)` for B.
- **The same machinery.** `decode_once` builds the same
  `StreamDecoder::planned(Plan::Workers(w), 2·w, k)`, with the same feeder and reader.
  `planned` → `with_inverse::<Segmented>` spawns the same scanner and workers
  (`src/parallel.rs`). `k` reaches `decode_block` unchanged.
- **The only branch on the choice** is `decode_block`'s per-block `match (cooperative,
  k.mtf)` (`src/lib.rs`). `Shipped` calls `mtf::mtf` or `mtf::mtf_checkpoints`, as before
  the integration. `P74c` calls `mtf::p74_c::mtf` or `mtf_checkpoints`.
  - The P74-C entries take the bypass to the same shipped functions, or enter the frozen
    `combined`.
  - The rest of `decode_block` (inverse BWT, RLE1, CRC) does not read `k.mtf`.
- **Which kernel ran, recorded per block.** `mtf::selections` counts each block's path:
  one relaxed atomic increment a block, outside the token loop, in both arms.
  - Every child records its counts, and asserts: for A and A′, every block
    `shipped_plain` and nothing else; for B, every block `p74c_plain`, split into
    bypass and combined, and nothing else.
  - `tests/p74c_selection.rs` checks the same for the default constructors and every
    setting: `decode`, `decode_parallel`, `StreamDecoder::new` and `Shipped` reach exactly
    `mtf::mtf` (plain) and `mtf::mtf_checkpoints` (cooperative).
- **The counter's cost [D]:** the same in all three arms. It is a change to the default
  path's code, counting only; behaviour is unchanged.

## 4. CPU allocation

The **whole process tree** (parent, children, and every child's scanner, feeder, workers
and reader) is confined by `taskset -c ALLOC` on the parent; children inherit it.

**How ALLOC is built, from sysfs (`topology/thread_siblings_list`):**
1. Group the CPUs the runner may use into physical cores.
2. Take up to 4 cores, first those other than CPU 0's core, in CPU order. Take CPU 0's
   core only if fewer than 4 others exist.
3. Use the lowest-numbered CPU of each core. Its SMT siblings stay idle.

On a host with P < 4 physical cores, ALLOC is all P cores and w ranges over 1..P.

**Expected allocations:**

| Host | ALLOC | Idle siblings | w |
|---|---|---|---|
| i9-10900K (10 cores, SMT) | CPUs 1–4 | 11–14 | 1–4 |
| GitHub `ubuntu-24.04` (2 cores × 2 threads) | CPUs 0, 2 | 1, 3 | 1, 2 |
| This preparation VM (4 cores, no SMT) | CPUs 0–3 | none | 1–4 |

**Recorded:**
- ALLOC and its siblings;
- `lscpu`, `lscpu -e`, SMT control;
- the CPU model, governor, driver and frequencies, `intel_pstate/no_turbo` or
  `cpufreq/boost`;
- the cgroup and `cpu.max`, the kernel, rustc, HEAD, load average and processes;
- `/proc/stat` before and after the timed run, from which the summary extracts the
  allocated CPUs' and their siblings' busy fractions, steal and irq;
- `/proc/pressure/*`;
- each child's `Cpus_allowed_list`, its physical-core count, and the default plan it saw.

A child whose default plan would not be `Workers(w)` aborts the trial.

### 4a. Affinity proof, per measurement (added before the freeze)

**At spawn.** Each child reads `/proc/self/task/*/status` and `stat` right after the
decoder is built and before any input is written. Every decoder thread then exists and is
blocked, so none can have ended: w workers, the scanner and main.
- **Recorded per thread:** its name, `Cpus_allowed_list`, last CPU (`psr`) and context
  switches.
- **The feeder** records its own entry as it starts (`/proc/thread-self`).
- **The cost:** this is inside the timed interval, the same for every arm, about 0.1 ms
  [E].
- **Names can lag:** a thread that has not started yet can still carry the parent's name.
  Its allowed set is the inherited one, which is what is checked.

**At the end, untimed.** The surviving threads are read again, after the read that
returns 0 and before exit.

**Checks:**
- the child asserts w + 3 threads at spawn;
- the parent asserts that every recorded thread's allowed set is non-empty and ⊆ ALLOC;
- the parent itself asserts its own affinity equals ALLOC.

**Recorded:** the raw rows keep every snapshot. The summary reports confinement, the
thread names seen, and the `psr` histogram per thread kind. Migrations are not counted
(`/proc/*/sched` needs `CONFIG_SCHED_DEBUG`).

## 5. Corpora (order frozen)

| # | Name | Role | Sourcing rule |
|--:|---|---|---|
| 1 | `wikidata-fitted` | primary (b = 123 was fitted on it) | `bench-data` branch, `windows/wikidata-20261019`, each window hash-checked against `bench/wikidata.json`, then `--windows`. Compressed SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915` (the stage trial's) |
| 2 | `wikidata-heldout` | primary, **same-dump** held-out | HTTP range fetch per `research/p74-c-implementation:docs/journal/2026-10-31-p74-c-confirmatory/heldout/spec.json` (copied unchanged), then `--windows`. `7f5c9a0044e5a30589bf343b9b31e698849b2acc69461a55c97b550193128b45` |
| 3 | `wikidata-20261026` | primary, **different-dump** held-out (out of sample) | 12 × 1 MiB windows of `https://dumps.wikimedia.org/wikidatawiki/entities/20261026/wikidata-20261026-all.json.bz2` (103,565,615,653 B) at ⌊(2i+1)/24 · size⌋, i = 0..11, by HTTP range, then `--windows`. Positions in `2026-10-31-p74c-decoder/different-dump/spec.json`, committed with this protocol before fetching |
| 4 | `random` | control | `--prepare` (the stage trial's generator: `tests/common` corpora at scale 4, level 9) |
| 5 | `records` | control | `--prepare` |
| 6 | `mixed-independent` | control (seed `0x74c0202610310001`) | `--prepare` |
| 7 | `text` | control | `--prepare`: this crate's own `src`, `tests` and `examples` at the pinned commit. Its hash therefore differs from the stage trial's, because the integration adds sources |
| 8–34 | adversarial suite, 27 files | descriptive (timed); correctness mandatory | `examples/p74_suite.rs` (copied unchanged), base seed `0x74c0202610310100`. Each file's hash must equal the stage trial's `suite.json` |

**Different-dump corpus:**
- **Fetch:** the 12 windows are fetched only after this protocol and the spec are
  committed. Each window's hash is recorded at the freeze.
- **Audit:** the corpus is built, and its selector audit (blocks bypassed, P74 to the
  block's end, handoffs) is run and committed at the freeze, untimed.
- **Out of sample:** nothing is tuned on it. If a window cannot be fetched, the freeze
  records that and the corpus is built from the windows fetched, no substitute positions.
- **Caveat [D]:** Wikidata dumps are ordered by entity, and this dump is a week later
  than the fitted one. The same relative positions therefore cover largely the same
  entities, re-serialised and re-compressed. It is a different dump and a different
  compression, not independent content.

**Correctness, for every corpus (mandatory):**
- the compressed SHA-256 equals the frozen one;
- `bzip2 -t` passes;
- `bzip2 -dc`'s SHA-256 equals the frozen expected output hash;
- before timing, `p74c_decoder --check` decodes the corpus with A and B at every w through
  the same `StreamDecoder` path, and also sequentially (`decode_with`). Every output's
  SHA-256 must equal the expected hash, and A's and B's must be byte-identical.

**Wikidata roles:**
- **Corpus 3 is the only primary.**
- **Corpora 1 and 2 are secondary replications.** They come from the same dump
  (`20261019`), so they are not independent evidence of each other. Corpus 1 is in-sample
  for the guard's constant b.
- None is pooled with another.

**Range-fetch guard (corpora 2 and 3).** Each window is one HTTP request with
`Range: bytes=a-b`. The fetch is accepted only if:
- the status is exactly 206;
- `Content-Range` is exactly `bytes a-b/DUMP_BYTES`, with the spec's dump size;
- the body is exactly b − a + 1 bytes.

A 200, or any full-object or otherwise mismatched response, aborts at once, with no
retry. Only transport errors (connection reset, timeout) are retried, at most 3 times.
Then the window is hashed: against the spec (corpus 2), or registered at the freeze
(corpus 3).

## 6. Schedule

- **Build:** rustc **1.90.0** (rustup, minimal profile), `--locked`, release profile (fat
  LTO, codegen-units 1), RUSTFLAGS empty, no `target-cpu=native`.
- **Cells:** corpus × w, w ∈ {1..min(4, P)}.
- **Warm-up:** every cell runs each arm once, untimed (its row is flagged and discarded),
  before round 0.
- **20 paired rounds:**
  - Each round visits every cell once. The cell order within a round is shuffled by
    xorshift Fisher–Yates from seed `0x74c0202610310201`, a fresh shuffle each round.
  - Within each cell, the round's arm order comes from that cell's sequence of 20
    orders:
    - the 6 permutations of (A, B, A′) three times (18 rounds), then (A, B, A′) and
      (B, A, A′);
    - shuffled per cell by Fisher–Yates from the same RNG stream.
  - So A and B have identical position counts (7, 7, 6) in every cell, and A′ has
    (6, 6, 8). `p74c_decoder --schedule` prints the schedule. No timing.
- **Raw rows** are written and flushed as each child completes.
- **One timed run per host label:**
  - no rerun, no fastest-run selection, no outlier removal, no adaptive repetition;
  - a crashed run is reported as crashed;
  - the runner refuses a second run into the same label.

**Duration [E]:** about 8,200 children at w ≤ 4. Roughly 30–60 minutes on the i9,
dominated by the three Wikidata corpora at w = 1. Not measured.

## 7. Statistics (`bench/p74c_decoder.py summary`)

**Per round.** For each corpus × w and each comparison B/A and A′/A, take per-round log
ratios x_r over the 20 rounds of:
- throughput (X_B/X_A);
- CPU-s/GB;
- peak RSS.

**The ρ₁ correction, as in the stage protocol:**
- mean x̄, SD s, and lag-1 autocorrelation ρ̂ in round order;
- if ρ̂ > 0, n_eff = 20(1 − ρ̂)/(1 + ρ̂), else 20;
- SE = s/√n_eff, df = max(1, ⌊n_eff⌋ − 1);
- a two-sided 95% t interval, reported as ratios exp(x̄ ± t·SE).

Each end of that interval is a one-sided α = 0.025 bound. "Lower" and "upper" below
always mean these bounds.

**No multiplicity adjustment.** Every gate below is conjunctive: "every listed row must
pass". That is an intersection–union test.
- **Why:** the overall claim is rejected unless each row's H0 is rejected separately. Its
  size is at most the largest single row's α, so testing each row at one-sided α = 0.025
  with no adjustment controls the error rate of the joint claim. Holm or Bonferroni would
  only make each row stricter and add nothing to that control.
- **Where Holm would apply:** only to "at least one row" claims. This protocol makes none,
  so Holm is not used.

**Peak RSS:**
- **Fresh measurements:** every arm in every round is a fresh child process, so each
  `ru_maxrss` is its own observation.
- **The max rule (an explicit engineering choice, not a test):** M_B = max over the 20
  rounds of B's `ru_maxrss`, and M_A likewise.
- **Descriptive, beside it:** the paired per-round B/A log ratios (interval as above,
  median, min, max), and A′ against A: M_A′/M_A and the A′/A interval, which give the RSS
  noise.

**A/A calibration:**
- the rows are the A′/A intervals for throughput and CPU-s/GB on corpora 1–7 at every w
  (m = 56 at P ≥ 4);
- k* is the smallest k with P(Bin(m, 0.05) ≥ k) ≤ 0.05;
- if k* or more A′/A intervals exclude 1, the run is **miscalibrated**.

**Descriptive only:**
- CPU-s/GB at every w other than 4;
- the adversarial suite's rows;
- the whole-life CPU from `wait4`;
- the thread counts;
- the primary's lower bound against 1.00;
- a CPU-s/GB sensitivity table: the w = 4 rows against upper bounds of 1.00 and 1.02;
- the model's implied s_MTF.

## 8. Acceptance (registered: three outcomes per gate and overall)

Every gate has exactly one outcome: **PASS**, **NEGATIVE** or **INCONCLUSIVE**. Where a
row meets both a PASS and a NEGATIVE condition, NEGATIVE takes precedence.

0. **Correctness, a precondition.**
   - **Checked:** every output hash of every timed and untimed decode matches, A's and B's
     outputs are byte-identical, every corpus hash matches, the structural audit passes
     (§9), and every child's plan is `Workers(w)`.
   - **An input or host failure** (a corpus hash, the audit or the plan) aborts before or
     during timing. Then there is no economic outcome: the run is reported as aborted.
   - **A B output that differs** from A's or from the expected hash is **NEGATIVE**
     overall: P74-C decoded wrongly.

1. **Primary throughput:** B/A at w = 4 on `wikidata-20261026` only.

   | Outcome | Condition |
   |---|---|
   | PASS | lower ≥ 1.02 |
   | NEGATIVE | upper < 1.02: the gain is shown to be below the economic threshold |
   | INCONCLUSIVE | otherwise |

   - **Demonstrated harm** (upper < 1.00) is reported separately.
   - **Descriptive secondary:** whether lower > 1.00.

2. **Secondary Wikidata replications** (`wikidata-fitted`, `wikidata-heldout`): B/A
   throughput reported at every w. Not a gate, but a pre-defined **material
   deterioration flag** is raised, and must be reported in the verdict, if at w = 4
   either:
   - the point ratio < 1.00; or
   - lower < 0.97.

   A raised flag does not change the overall outcome. It can't be dismissed, and it is
   carried into any default-enable proposal.

3. **Controls:** throughput B/A on `random`, `records`, `mixed-independent` and `text`,
   at **every** w. Each row:

   | Outcome | Condition |
   |---|---|
   | PASS | lower ≥ 0.97 |
   | NEGATIVE | upper < 0.97: a regression of more than 3% demonstrated |
   | INCONCLUSIVE | otherwise |

   The gate passes iff every row passes, and is NEGATIVE iff any row is.

4. **CPU-s/GB:** B/A at w = 4 on every corpus 1–7. Other w are reported. Each row:

   | Outcome | Condition |
   |---|---|
   | PASS | point ≤ 1.00 and upper ≤ 1.01 |
   | NEGATIVE | lower > 1.01 |
   | INCONCLUSIVE | otherwise |

   The gate passes iff every row passes, and is NEGATIVE iff any row is.

5. **Peak RSS:** every corpus 1–7 at every w. Each row:

   | Outcome | Condition |
   |---|---|
   | PASS | M_B ≤ 1.01 · M_A (the max rule) |
   | NEGATIVE | lower of the paired log-ratio mean > log 1.01 |
   | INCONCLUSIVE | the max rule fails, but NEGATIVE isn't met |

   The gate passes iff every row passes, and is NEGATIVE iff any row is.

6. **A/A calibration:** PASS if not miscalibrated (§7), else INCONCLUSIVE. It can't be
   NEGATIVE.

**Overall:**

| Outcome | Condition |
|---|---|
| PASS | every gate (1, 3, 4, 5, 6) passes and correctness holds |
| NEGATIVE | any gate is NEGATIVE, or B decoded wrongly |
| INCONCLUSIVE | otherwise |

**Stop rule.** Every outcome stops this experiment: no rerun, no tuning, no selector
revision and no post-hoc change of threshold.
- **PASS:** propose a default-enable change for the maintainer's approval. Nothing is
  enabled by this trial.
- **NEGATIVE or INCONCLUSIVE:** P74-C is archived at decoder level.

**Which run decides:** the registered outcome is the i9-10900K's run (bare metal, ≥ 4
physical cores, so w = 4 exists).

## 9. Structural audit (on the exact timed binary, before timing)

`bench/p74c_decoder_asm_audit.py` is `bench/p74c_asm_audit.py` adapted to the integrated
binary. Gate 1's Free kernel is no longer linked, so the combined kernel's ordinary
low-rank cycle is compared with **frozen reference counts**. These are the 8b6caf1
audits, identical on all three CI hosts and the i9, rustc 1.90.0 [M]:

| Variant | Free (reference) | Combined at the pin |
|---|---|---|
| plain | 37 instructions, 6 jcc, 0 RMW slots | 38, 6, 0 |
| checkpoints | 38, 6, 2 | 39, 6, 0 |

Head chain in all four: {vpermq 1, vpshufb 2, vmovdqa 1, vpblendvb 1}.

**Gates on the integrated binary's `p74_c::combined::<0>` and `::<16384>`:**

| Gate | Condition |
|---|---|
| G1 | conditional branches on the cycle ≤ Free's reference (6), so no extra low-rank conditional branch |
| G2 | no YMM/XMM stack access on the cycle |
| G3 | stack RMW slots ≤ Free's reference (0 plain, 2 checkpoints), and the head-chain multiset equal to the reference, so no new loop-carried dependency |
| G4 | no monitor instruction (`add …,0x7b`, `shr …,0x5`) on the cycle |

- **Instructions:** reported against the pin's combined counts. The +1/+1 over Free was
  already waived by the stage protocol's §4, so it is not a gate.
- **Run:** the runner aborts before timing if any gate fails.
- **The shipped side:** the audit also confirms that the shipped kernels' symbols are
  present.

## 10. What each host can and cannot show

**The i9-10900K (bare metal, 10 cores):**
- **Can show:** the registered outcome at w = 1–4 on an isolated 4-core allocation.
- **Cannot show:** a Zen result, or a laptop's thermal envelope. The governor is
  recorded but not controlled.

**GitHub `ubuntu-24.04` (exploratory, 3 independent jobs, w ∈ {1, 2}):**
- **Can show:** replication on shared virtual Zen 3 and Zen 4 hosts at 1–2 workers, with
  steal recorded.
- **Cannot show:** w = 3 or 4, so not the primary gate or the w = 4 CPU gate. Never pooled
  with the i9.
- **Reading it:** each gate's rows at w ≤ 2 (controls, RSS, CPU-s/GB descriptively) are
  reported with their outcomes, and no overall outcome or stop rule is applied.

**This preparation VM:** correctness, audits and dry runs only, never timed.

**No host shows:**
- the guard's economics on Wikidata (0 handoffs in the stage trial's corpora);
- the checkpoint kernel in the decoder (no teams at w ≤ 4);
- wikidata-pq's end-to-end split time. That would be a separate measurement through its
  own CI bench.

## 11. Freeze and execution

- **The freeze commit** adds `2026-10-31-p74c-decoder/frozen.json`:
  - every corpus's compressed and expected output SHA-256;
  - the suite's 27 files;
  - the source hashes of the harness;
  - the different-dump windows' hashes and its selector audit.
- **`bench/p74c-decoder-pin`** names the freeze commit. The runner checks that HEAD
  equals it except for the pin file, and that the tree is clean.
- **The one command:** `just bench-p74c-decoder LABEL` (→ `bench/p74c_decoder_run.sh`).
  It writes to the git-ignored `bench/results/p74c-decoder/LABEL/` and `LABEL.tar.gz`
  (`P74C_RESULTS_DIR` to override), with the input cache under `target/p74c-decoder/work`.
- **The harness self-test, before the timed run:** the whole trial path (children,
  `wait4`, affinity checks, summary) runs on small files placed under the gated corpora's
  names. Every time and CPU figure in it is a fixed synthetic value
  (`P74C_SYNTHETIC_TIMES=1`). It proves the plumbing on the host and measures nothing.
- **`P74C_STOP_BEFORE_TIMING=1`** runs everything up to the timed run and stops. It
  writes to `LABEL.dry-run`, so the label stays free for the one timed run.
- **The freeze record:** `2026-10-31-p74c-decoder/freeze.md` holds the hashes, the audits
  and the dry run's evidence.
