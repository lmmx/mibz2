# P74-C confirmatory stage protocol: frozen before timing

**Status:** frozen and committed before any confirmatory timing. Not yet run. The
inputs, harness, schedule, statistics and acceptance below are fixed by `frozen.json`
(every input's SHA-256, every harness source's SHA-256) in
[2026-10-31-p74-c-confirmatory/](2026-10-31-p74-c-confirmatory/) and by the pinned commit
in `bench/p74c-pin`. Labels: **[M]** measured, **[D]** deduced, **[E]** estimated.

## 0. Authorisation

The maintainer's message, as relayed verbatim (ellipses in the relay):

> "I explicitly authorize expediting P74-C performance validation on both GitHub Actions
> CI and my i9-10900K. Finish the essential correctness checks, freeze the protocol and
> inputs, then launch the five-arm CI benchmark. For my i9, push the benchmark code and a
> reproducible one-command runner to GitHub. … Use the same experimental arms and
> acceptance criteria on both machines. Include the fitted and held-out Wikidata corpora,
> random, records and adversarial switching cases. Preserve the 3% negative-control
> regression limit, exact timed-binary audit and paired statistics. Prioritize obtaining
> actual performance measurements over further speculative analysis or assembly
> micro-tuning. … Do not modify production decoding or begin unrelated experiments."

The runner sets `P74_APPROVED_RUNNER` from its host-label argument under this
authorisation. Production decoding and dispatch are unchanged.

## 1. Policy under test (frozen, as implemented)

Shipped kernel if S < 16384 or 2S ≥ N_max. Otherwise P74; at a rank ≥ 32 token, before
its list update, switch one-way iff F + 1 > ⌊S_incl/32⌋ + 123, S_incl counting consumed
symbols *including* the trigger. This is **not** the draft's S_before rule
([correction](2026-10-30-p74-c-implementation.md#correction-2026-10-31-the-inclusive-trigger-is-a-distinct-policy)):
it switches at the same event or later, never earlier, and a stream at density exactly
1/32 at the boundary evades it indefinitely (`hug-eq-p32` below, [M]).

**Caveat [D].** The monitor counts high-rank events F, not tail-chunk work K
(≈ ⌈(q − 31)/32⌉ chunks per event). F ≤ τS + b is a *frequency* invariant. It is a timing
bound only with a conservative per-event cost limit, which nothing here establishes: the
rank-class rows measure the per-event cost by class instead.

## 2. Arms (one binary, `examples/p74_experiment.rs`)

| Index | Arm | Entry |
|--:|---|---|
| 0 | T0 | `p74_c::mtf[_checkpoints](…, Arm::T0)`: the untouched standalone shipped kernel |
| 1 | T1 | combined kernel, shipped mode from token 0, no bypass |
| 2 | T2 | frozen bypass and one-way guard |
| 3 | Free | Gate 1's `p74::mtf[_checkpoints](…, p74::Arm::Free)`: P74, never switches |
| 4 | A/A | T0's call again (same function and arguments), a separate label in the rotation |

Every arm's output length, bytes, 256 counts and checkpoint records are checked against
the shipped kernel's on every block of every pass, untimed (`pass`). The timed interval
is unchanged from the [validation protocol](2026-10-31-p74-c-validation-protocol.md): one
`Instant` pair around each block's call; throughput in MTF L bytes.

## 3. Corpora (order frozen)

| # | Corpus | Role | SHA-256 (compressed) | Reproduced by |
|--:|---|---|---|---|
| 1 | `wikidata-fitted` | primary; **fitted** (b = 123 was fitted on it) | `82f65ace…c29915` | `bench-data` windows, `--windows` |
| 2 | `wikidata-heldout` | **same-dump held-out Wikidata, not independent-source** | `7f5c9a00…128b45` | 12 HTTP-range windows of the dump, `--windows` ([registration](2026-10-31-p74-c-confirmatory/heldout/REGISTRATION.md), [spec](2026-10-31-p74-c-confirmatory/heldout/spec.json)); 473 blocks, 0 handoffs at the frozen policy [M] |
| 3 | `random` | negative control | `10d444e6…dc3f18` | `--prepare` |
| 4 | `records` | negative control | `ff8b5f6e…391f8c` | `--prepare` |
| 5 | `mixed-independent` | negative control (seed `0x74c0202610310001`) | `11639f35…0778db` | `--prepare` |
| 6 | `text` | descriptive | `c06f9591…edf882` | `--prepare` (this crate's sources at the pinned commit) |
| 7–33 | adversarial suite, 27 files | own family | [suite.json](2026-10-31-p74-c-confirmatory/suite.json) | `examples/p74_suite.rs`, seeds recorded |

Full hashes: `frozen.json`. Random, records and mixed reproduce the earlier frozen hashes
exactly [M]; text differs from the earlier manifest because the crate's sources changed,
and is pinned here. The [suite record](2026-10-31-p74-c-confirmatory/suite.md) gives each
file's design, symbols, suffix start, and its audited switch (consumed, output, F):
22 handoffs, 5 non-switching by design, all verified through the production scanner
and the actual kernel's handoff [M].

## 4. Structural assembly audit (replaces the instruction-count gate)

**Record.** The frozen draft's gate (no new fast-path instruction after one fix) failed:
+1 plain / +3, then +1 / +1 checkpoint instructions on the ordinary low-rank path after a
second correction ([assembly correction](2026-10-31-p74-c-assembly-correction.md)). The
validation protocol then accepted that discrepancy without a recorded user authorisation
(found by an independent review, not committed). This
protocol replaces the count with structural gates, under the authorisation in §0 ("exact
timed-binary audit"; "no assembly micro-tuning"). It does not claim the old gate passed.

`bench/p74c_asm_audit.py BINARY` runs on the exact binary that is timed, on the timing
host, before timing; the runner aborts if it fails. It parses `objdump -d`, builds each
kernel's CFG, and takes the ordinary low-rank cycle: the shortest cycle (instructions,
NOPs excluded) through the block holding P74's head update (`vpermq 0x4e`, two `vpshufb`,
`vpblendvb`). It compares T2's P74 loop in `p74_c::combined::<0|16384>` with Free's
`p74::avx2::mtf[_checkpoints]::<true>` in the same binary:

| Gate | Condition on the combined kernel's cycle |
|---|---|
| G1 | conditional branches ≤ Free's |
| G2 | no YMM/XMM load or store to the stack |
| G3 | stack slots both loaded and stored (memory RMW chains) ≤ Free's; head-chain vector ops (vpermq, vpshufb, vpblendvb, mask vmovdqa) the same multiset |
| G4 | no monitor instruction (`add …,0x7b`, `shr …,0x5`) |

Instruction-count differences are reported, not gated. [M] On rustc 1.97.0 (this
preparation host): plain Free 37 / C 36 instructions, 6 / 6 branches, 0 / 0 RMW slots;
checkpoints 36 / 37, 6 / 6, 2 / 1; all gates pass. On rustc 1.90.0 (the pinned
toolchain, this host's dry run of the runner): plain Free 37 / C 38, checkpoints 38 / 39 —
the documented +1 / +1, from the same cycles as the manual listings — with 6 / 6
branches, RMW slots 0 / 0 and 2 / 0, no YMM stack access, the same head chain and no
monitor: all gates pass. The timing host's binary is audited again by the runner.

## 5. Schedule

- Build: rustc **1.90.0** (rustup, minimal profile), `--locked`, release profile (fat LTO,
  codegen-units 1), RUSTFLAGS empty, no `target-cpu=native`.
- Corpus order as §3; variants plain then checkpoints; per variant and corpus, each arm
  warmed twice untimed (order T0, T1, T2, Free, A/A).
- **20 paired rounds:** the Williams design for five arms (10 orders: each arm twice in
  each position, each ordered adjacent pair twice), two copies, each copy's rows shuffled
  by xorshift Fisher–Yates, seed `0x74c0202610310003` (`p74_experiment --schedule` prints
  it). Changed from 12 rounds / 3 arms because five arms need the 10-order design; two
  copies keep each position balanced four times.
- Each observation = the sum of 2 complete corpus passes, block order preserved; per-block
  times are also recorded (`BLOCK` rows).
- One run per host label; no fastest-pass selection, no outlier removal, no adaptive or
  repeated runs.

## 6. Statistics (`bench/p74_experiment.py confirm`)

For each corpus × variant and each comparison X/Y ∈ {T2/T0, T1/T0, T2/T1, Free/T0,
T2/Free, A/A/T0}: x_r = ln(T_X,r / T_Y,r) over the 20 rounds.

- Mean x̄, SD s, lag-1 autocorrelation ρ̂ of x_r in round order.
- **Serial correlation (frozen):** if ρ̂ > 0, n_eff = 20(1 − ρ̂)/(1 + ρ̂), else 20;
  SE = s/√n_eff; df = max(1, ⌊n_eff⌋ − 1); two-sided 95% t interval, reported as ratios
  exp(x̄ ± t·SE).
- **Multiplicity (frozen):** one-sided α = 0.025 (the upper end of a two-sided 95%
  interval), H0: ratio ≥ limit, p from the t distribution above, Holm–Bonferroni within
  each family:
  - family 1: random, records, mixed-independent × {plain, checkpoint} (6 rows), T2/T0,
    limit 1.03;
  - family 2: the 27 adversarial files × 2 variants (54 rows), T2/T0, limit 1.03 for
    blocks that reach the policy, 1.01 for bypassed files.
- **A/A as the empirical null:** each row's A/A/T0 interval is reported. With m rows,
  k* = the smallest k with P(Bin(m, 0.05) ≥ k) ≤ 0.05. If k* or more A/A intervals
  exclude 1, the run is **miscalibrated**: economic criteria are not evaluated
  (inconclusive), correctness still is.
- **Decomposition (adversarial, per block):** paired per-round ns differences with t
  intervals (df 19):

| Difference | Identifies |
|---|---|
| T2 − T1 | P74 prefix exposure + handoff H + monitoring M (both run the same shipped suffix code) |
| T2 − Free, blocks that never switch and aren't bypassed | M + combined-skeleton/layout difference |
| T1 − T0 | refactor and layout tax |
| Free − T0 | the never-switch counterfactual |
| A/A − T0 | per-block noise |

  **Not identified:** H and M separately, the per-event cost, or the prefix exposure apart
  from H. For an unfavourable block ΔT = ΔT_P74prefix + H + M + ΔT_shippedsuffix, only the
  sums in the table are estimable.

## 7. Acceptance (registered limits kept)

1. **Correctness, mandatory:** any output, count, checkpoint or error mismatch on any
   arm, block or pass, any corpus hash mismatch, or a failed suite handoff check fails
   the run; timings are then not interpreted.
2. **Structural audit** (§4) passes on the timed binary, else no timing.
3. **Negative controls:** every family-1 row passes Holm at α = 0.025 against +3%
   (jointly). Failure to establish the bound is failure, not proof of a regression.
4. **Primary, fitted Wikidata checkpoint:** T2/T0 upper 95% ≤ 0.95, and the saving's
   lower bound 1 − upper > 2N, N = the largest |1 − endpoint| of that row's T1/T0 and
   A/A/T0 intervals. Plain reported, not substituted.
5. **Held-out Wikidata checkpoint:** the same criterion, reported separately, never
   pooled with the fitted row.
6. **Adversarial:** correctness mandatory; family 2 passes Holm at α = 0.025 (limits in
   §6); the decomposition reported for every block, pass or fail. Any failing row is a
   **negative** for option C (draft §12 stop rule).
7. T2/T1 (P74's own contribution) and T1/T0 (tax) are reported for every row.

## 8. Host requirements, recorded environment, stop rules

- Linux x86-64 with AVX2; no CPU quota (the binary refuses a capped cgroup).
- The runner picks, from sysfs, an allowed CPU other than CPU 0 and its siblings, with an
  SMT sibling, which stays idle; if no SMT is exposed it takes the first such CPU and
  records `no_smt_exposed`. `P74C_CPU` overrides (recorded).
- Recorded: `lscpu`, topology, SMT control, governor, driver, current/max frequency,
  `intel_pstate/no_turbo` or `cpufreq/boost`, cgroup and `cpu.max`, kernel, rustc,
  HEAD, load average, processes; `/proc/stat`, `/proc/pressure/*` and `cpu.stat` before
  and after; wall time.
- **Stop rules:** no reruns, no retuning (τ, b, S₀, θ, schedule, analysis), no selective
  rows, no edits after the freeze. A crashed run is reported as crashed; the runner
  refuses a second run into the same output directory.

## 9. Execution

The one command (`.just/bench.just` → `bench/p74c_run.sh`):

```sh
git clone https://github.com/lmmx/mibz2 && cd mibz2   # or: git fetch origin
git checkout <pinned commit, bench/p74c-pin's child>
just bench-p74c <host-label>      # e.g. i9-10900K
# result: ~/p74c-results/<host-label>.tar.gz
```

- **The i9-10900K, bare metal:** the confirmatory host (10 cores, SMT; CPU chosen from
  sysfs as above). Close other workloads first; record any that can't be.
- **Zen replication:** the same command on a Zen host, as a separate labelled run.
- **GitHub runners:** `.github/workflows/p74c-validation.yml`, `workflow_dispatch` only,
  three independent `ubuntu-24.04` jobs, artifacts kept 30 days. Exploratory: shared
  virtual hosts, topology read at run time, and their results are reported beside, never
  pooled with, the i9's.

## 10. What a pass qualifies for

Only a separately authorised full-decoder test (1, 2 and 4 workers measured separately;
CPU-s/GB, peak RSS, random and records decoder figures within their limits). No shipping
or production integration follows from this stage experiment alone.

## 11. Preparation evidence [M]

rustc 1.97.0 on an Intel Xeon @ 2.80 GHz (4 CPUs, AVX2), correctness only:

- `cargo test --release --lib inclusive_trigger`: 3 pass (the policy correction's tests).
- Suite: 27 files generated, `bzip2 -t` and `--check` pass, audit 54/54 rows agree with
  the oracle and the designs.
- Structural audit: pass (§4).
- The pinned-toolchain repetition (1.90.0) of build, `--check`, audit and structural audit
  is part of every run, before timing.
