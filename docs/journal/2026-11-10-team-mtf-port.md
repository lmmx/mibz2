# 2026-11-10: C29's register-list MTF on the team path (port 3), counted

**Status: counted, not timed. Two validity conditions are UNMET as registered (V1 and V2), so
the count is FAIL-MEASUREMENT. The Guardian disposes.**
- **Registered** at `964fbfacb` (`2026-11-10-team-mtf-port-registration.md`), before the
  kernel, under D088's rules.
- **Kernel and tests:** `7323196d5`.
- **Stacked on port 2** (#337) at `f62d62bfb`, which includes master `8f540ef11`.
- **Every figure** comes from the archive (`cg/*.gz`) by `count.py.txt`. Its full output, with
  each profile's SHA-256, is `count.txt`.

**What was counted, as diagnostic observations:**
- **The MTF stage:** 4.7148 → **4.1322** Ir/D. That is inside the predicted band (4.11 to
  4.16), and the falsifier (not below 4.40) is not triggered.
- **The decoder Dd:** 32.8635 → 32.2824 (**−0.5811**). The bounded whole-decoder change is
  [−0.5815, −0.5811].
- **The instruction-identity conditions I1 to I4 are all met.**
- **V1 failed** on a registered name rule. **V2 failed** on an allocator arc that turned out
  to depend on scheduling.

## 1. The change

- **The dispatch:** `mtf::mtf_checkpoints`' AVX2 path calls `mtf_checkpoints_reg_avx2`, C29's
  kernel from `perf/c29-register-checkpoints` at `2153cd40e`. It is ported verbatim apart
  from its test counter and its doc's opening. `src/mtf.rs` was byte-identical to C29's base.
- **The old kernel:** `mtf_checkpoints_avx2` is now `#[cfg(test)]`, and `mtf_checkpoints_mem`
  is its dispatch, as the tests' reference.
- **Tests:** C29's `src/mtf/tests.rs` is ported to compare the new kernel against the old one
  and against the scalar twin. I added the scalar comparison.
- **Miri workflow:** `.github/workflows/miri.yml` gains `src/mtf*` paths, the list check and
  C29's AVX2 step (`mtf::tests::miri_`, built with `+avx2,+bmi1`).
- **Untouched:** `Levers`, `mtf::mtf` and `mtf_avx2`.

**Safety:** C29's own argument stands, in its doc and comments.
- The 64-byte run stores write only below `max`, and so inside `ll`'s capacity.
- Every byte past `n` is rewritten before `n` passes it, or lies past the length set at the
  end.

C29's B5 evidence is cited, not inherited. **B5 for this port is not discharged by the
implementer.**

## 2. Tests (before the count)

- **The ported C29 tests:**
  - `register_checkpoints_are_checkpoints`;
  - `register_checkpoints_reach_every_class` (every class reached);
  - `miri_register_checkpoints`.
- **Results:**
  - `cargo test --release --lib mtf::` passes 3 of 3;
  - all of `cargo test --release` is green, `tests/pipelined.rs` and `tests/cooperative.rs`
    included;
  - fmt and clippy (`-D warnings`) are clean.
- **Miri,** with AVX2 (`RUSTFLAGS=-C target-feature=+avx2,+bmi1`, `MIBZ2_EXPECT_AVX2=1`):
  `miri_register_checkpoints` passes.

**Mutants** of the kernel's limit, split and store logic:

| mutant | result |
|---|---|
| M1 no checkpoint limit (`limit = max`) | caught (3 tests) |
| M2 split loop `>=` made `>` | **survived C29's suite.** I added three columns with a run that crosses one checkpoint and ends exactly on the next. With them it is caught |
| M3 vector store for runs of 65 | caught |
| M4 no checkpoint at a block ending on one | caught |
| M5 the split loop's next checkpoint one short | caught |
| M6 vector bound laxer (`n + 63 <= max`) | caught natively: the test process aborts (`malloc(): invalid next size`, SIGABRT) |

## 3. The count

**Binaries** (both built by `cargo build --release --example ab`, rustc 1.99.1):
- control `ab` at `f62d62bfb`: `18c03d15…`;
- candidate `ab` at `7323196d5`: `45b6fa1a…`.

**Resources** (`resources.txt`, recorded, not gating): before each arm, rustc 0, valgrind 0,
load 1.38/1.23/1.13, 4 CPUs, 5.0 G free. ab's `runq_s` was 0.209, 0.118 and 0.002.

**Inventory:** `control-team.25682-01..12.gz`, `port3-team.25720-01..12.gz` and
`port3-one.25754-01.gz`.
- Threads 1 to 12 appear once each in each team arm.
- The loader refuses doubled input.
- All three arms write 209,700,196 bytes, hash `a8830caee5c511a6`.

**The partition** (Ir/D; the ledger Dd + H + U = P holds exactly in every arm):

| | control (port 2) | candidate (port 3) | candidate − control |
|---|--:|--:|--:|
| **MTF** | 4.7148 | **4.1322** | **−0.5826** |
| scanner | 3.2024 | 3.2026 | +0.0002 |
| build | 5.5695 | 5.5695 | 0.0000 |
| walk | 17.7072 | 17.7054 | −0.0018 |
| assembly | 0.1697 | 0.1740 | +0.0043 |
| RLE1 + CRC | 1.4305 | 1.4305 | 0 |
| coordination | 0.0695 | 0.0683 | −0.0012 |
| **decoder Dd** | **32.8635** | **32.2824** | **−0.5811** |
| harness H | 1.4757 | 1.4751 | −0.0006 |
| unattributed U | 0.0187 | 0.0183 | −0.0004 |
| process P (reported, not a gate) | 34.3579 | 33.7758 | −0.5821 |
| one worker (candidate binary) | | 25.7918 (MTF 4.1112) | |

- **The bounded whole-decoder change** is [−0.5815, −0.5811], if all of U were decoder work.
- **The MTF kernel itself:**
  - `mtf_checkpoints_reg_avx2` self is 779,679,144 Ir (3.718 Ir/D), against
    `mtf_checkpoints_avx2`'s 835,326,277 (3.983) and one worker's `mtf_avx2`, 773,179,796
    (3.687);
  - the rest of the drop is the memset arc (§6).

## 4. Conditions

| # | condition | result |
|---|---|---|
| V1 | the ledger exact, U ≥ 0, threads once, same bytes, objdump exit 0; **every harness-stage function's name starts with `ab::` or `<ab::`** | **UNMET.** The ledger, U, threads, bytes and objdump are all met. But in both team arms, four harness-stage functions are std or core instantiations over `ab`'s types: `core::ptr::drop_glue::<ab::Threads>`, `std::sys::backtrace::__rust_begin_short_backtrace::<<ab::Threads>::watch::…>`, `<std::thread::lifecycle::spawn_unchecked<<ab::Threads>::watch…>…>::call_once` and `core::ptr::drop_glue::<<ab::Threads>::watch::…>`. `STAGES`' unanchored `<ab::` matched them. Their names fail the registered rule. They total 1,383 Ir (control) and 1,319 (candidate), under 0.00001 Ir/D. The one-worker arm meets V1 |
| V2 | the control's MTF and build stages equal port 2's archived ones within 0.001 Ir/D | **UNMET.** MTF: 988,696,041 against 988,695,280, met. Build: 1,167,925,659 against 1,168,322,828, a difference of 0.0019 Ir/D. All of it, −396,662 Ir, is the allocator arc `reserve ← FusedBuild::new` (794,775 → 398,113). That is the table's growth, which depends on which team got which block. `share` and `new`'s self costs are equal to 13 Ir. So the build stage was not deterministic, as the registration assumed |
| V3 | the candidate's one-worker MTF stage equals 862,123,747 | **met**, exactly |
| I1 | candidate: `mtf_checkpoints_reg_avx2` called 431 times, `mtf_checkpoints_avx2` and `mtf_avx2` 0 Ir; control: the reverse | **met:** candidate 431 calls, the old kernel and `mtf_avx2` absent; control 431 calls of `mtf_checkpoints_avx2`, no reg kernel, no `mtf_avx2` |
| I2 | the candidate kernel's non-stack byte stores sum to exactly 15,148,085 | **met:** `0xd21a2` ×15,135,660 + `0xd2775` ×12,425, both `mov %al,(%r8,%rcx,1)` (binary `45b6fa1a…`). The control's `0xd1f32` + `0xd22b8` and one worker's `0xeaa93` give the same total |
| I3 | the non-stack `vmovdqu %ymm` stores come in equal pairs; first stores sum to between 4,027,455 and 4,053,053 | **met:** pairs `0xd1da5`/`0xd1dab` ×4,025,878, `0xd23d3`/`0xd23d7` ×2,617, `0xd2495`/`0xd249b` ×2,550; the sum is 4,031,045. The control has none |
| I4 | the 431 calls come from the team's front stage | **met:** the caller is `mibz2::parallel::stage` |

- **Prediction:** the MTF stage about 4.12 (4.11 to 4.16), ΔDd about −0.59. **Counted:**
  4.1322 and −0.5811.
- **Falsifier** (MTF stage not below 4.40): not triggered.
- **Classification:** none here. V1 and V2 are unmet, and each validity condition was
  required. The Guardian disposes.

## 5. Deviations and what the failures teach

1. **V1's name rule was stricter than `STAGES`' own regex.** The regex attributes
   monomorphisations of std over `ab`'s types to the harness. They are the sampler thread's
   own machinery, but the rule registered names, not their meaning. The next registration
   should state the rule as `STAGES`' match plus a list of the allowed std instantiation
   shapes, or charge them to U.
2. **V2 assumed the build stage deterministic.** It is not: the allocator's `reserve` arc from
   `FusedBuild::new` depends on the scheduling of blocks to teams. A deterministic control
   needs the functions' self costs, or the stage less its allocator arcs.
3. **M2 exposed a gap in C29's suite,** closed here by three columns. The kernel did not
   change.
4. **The control binary was built** by checking out `f62d62bfb`'s `src` and `.github` in this
   worktree. The untracked `src/mtf/tests.rs` stayed on disk, unused: `mtf.rs` at
   `f62d62bfb` declares no `tests` module.

## 6. Dispatch paths (G3)

| path | changes? | evidence |
|---|---|---|
| pipelined team front (`parallel::stage` → `mtf_checkpoints`) | **yes** | I1, I4, this count |
| ordinary teams (`decode_block` → `mtf_checkpoints` when the inverse wants checkpoints) | **yes**, the same function | the tests; not counted |
| plain worker `mtf::mtf` / `mtf_avx2` | **no** | V3 |
| `mtf_checkpoints_scalar` | **no** | the tests |

**The memset arc:** in the candidate, MTF's memset arc falls with the 32-byte run stores, as
#170 did on one worker. Read off the stage: 4.1322 − 3.718 = 0.414, against one worker's
0.424.

**Not claimed:** time, adoption, merge. C29's timed R5 (INCONCLUSIVE) is not reopened.

## Reproduce

```text
python3 docs/journal/2026-11-10-team-mtf-port/count.py.txt . docs/journal/2026-11-10-team-mtf-port AB_CONTROL AB_PORT3
```
