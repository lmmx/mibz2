# 2026-11-10: C29's register-list MTF on the team path (port 3), registered before the kernel

**Status: registered, not built, not counted.** Committed and pushed before any kernel line on
this branch. A count, never a clock. Registered under Guardian D088: a frozen
decoder/harness partition, instruction-identity conditions with a binary and site witness,
the unattributed residual kept visible, resource conditions recorded and not gating.

- **The port:** port 3 of the Pathfinder's residual (#313 §4 and §6).
- **The stack:** port 2 (#337) at `f62d62bfb`, which carries master `8f540ef11`.

## Hypothesis

The team's MTF stage is `mtf_checkpoints_avx2`: 4.715 Ir/D against one worker's `mtf_avx2`
at 4.111 (#330, #313 §4).
- **What it lacks:** #207's register list, #170's 32-byte run stores and #172's byte list.
- **C29 already wrote the kernel:** `mtf_checkpoints_reg_avx2`, on
  `perf/c29-register-checkpoints` at `2153cd40e`, unmerged and opt-in there. It is
  `mtf_avx2`'s loop inside `mtf_checkpoints_avx2`'s outer loop: limit to the nearer of the
  block's end and the next checkpoint, and a run crossing a checkpoint split there.
- **`src/mtf.rs` is byte-identical** between C29's base (`6145b3a98`) and this base, so the
  kernel ports unchanged.

**The change** (not opt-in; this is the port):
- `mtf::mtf_checkpoints`' AVX2 dispatch calls C29's kernel, ported verbatim apart from its
  test counter.
- The old `mtf_checkpoints_avx2` stays, under `#[cfg(test)]`, as the differential's
  reference.
- C29's tests are ported, comparing against the old kernel and the scalar twin.
- No `Levers` change.
- `mtf_avx2` and `mtf::mtf` are untouched.

## Base, control and candidate

- **Control:** `ab` built at `f62d62bfb` (port 2 with master).
- **Candidate:** `ab` built at this branch's kernel commit.
- **Both built by** `cargo build --release --example ab`, rustc 1.99.1. Each SHA-256 is
  recorded.
- **Input:** `wikidata.bz2`, `82f65ace…c2c29915`, all 431 blocks.
- **D** = 209,700,196, each arm's `"out"`.

**Profiler:** `valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes`, one run
per arm:
1. control team: `ab child mibz2-pipe3-both 9`;
2. candidate team: the same;
3. candidate one worker: `ab child mibz2 1`.

**Archiving:** profiles are written to a directory that only ever holds raw files, then
gzipped without `-k` and moved into the archive. Every figure is computed from the archive
through `team_gap_count.load`, which refuses a raw-and-gzip pair or a repeated thread.

**No retries.** Every attempt is kept.

## The frozen partition (D088 §2)

Over all profiles of an arm (each thread once), with process total **P** (the sum of
callgrind's `totals`):

- **Decoder, Dd:** the sum of `bench/team_gap_count.py`'s stages **scanner, MTF, build,
  walk, assembly, RLE1, CRC and coordination**, by its `STAGES` and `LIBRARY` rules as at
  this commit.
  - A non-library function's self Ir goes to the first stage whose regex matches its name.
  - A library routine's inclusive Ir goes, arc by arc, to the stage of its non-library
    caller.
  - Membership is by function, on **any thread**. Decoder work on the harness's threads
    counts as decoder.
- **Harness-only, H:** stage **harness**: functions matching `^ab::|<ab::|…` (`examples/ab.rs`'s
  own code), with library arcs called from them, on any thread.
  - **Verified per arm:** the result lists every function in H, its Ir and its threads.
  - **A mismatch makes the arm INVALID:** a harness-stage function whose name does not
    start with `ab::` or `<ab::`.
- **Unattributed, U = P − Dd − H** (the script's "other"). It stays visible, unclassified
  and never subtracted as harness.
- **Validity:**
  - the ledger `Dd + H + U = P` holds exactly in integers;
  - `U ≥ 0`;
  - each thread appears once;
  - same bytes (209,700,196, hash `a8830caee5c511a6`);
  - objdump exits 0 for each address range read.
- **A whole-decoder statement is bounded, not assumed:** ΔDd, with the conservative range
  [ΔDd + min(0, ΔU), ΔDd + max(0, ΔU)] (as if all of U were decoder work).
- **The whole process P** is reported for every arm. It is not a gate.

## Prediction and falsifier

**Counted** (port 2's archive, `2026-11-10-share-one-pass/cg`; Ir/D):
- the control's MTF stage, `mtf_checkpoints_avx2` with its arcs: **4.7148**;
- one worker's MTF stage, `mtf_avx2` 3.687 + memset 0.424: **4.1112**.

**Model:** C29's kernel is `mtf_avx2` plus about 12,800 checkpoint pushes (a 1 KiB copy each)
and the limit's split of runs.

| figure (Ir/D) | control (port 2, counted) | predicted candidate |
|---|--:|--:|
| **MTF stage** | 4.7148 | **about 4.12** (4.11 to 4.16) |
| ΔDd, candidate − control | | **about −0.59** |

- **Falsifier:** the candidate's MTF stage **not below 4.40** (less than half of the 0.60 gap
  closed).
- **Every other decoder stage** is predicted unchanged. The walk and coordination are
  scheduling-sensitive, and are reported, not predicted.

## Conditions

**Validity** (each must hold, or the count is FAIL-MEASUREMENT):
- **V1:** the partition's validity above, in all three arms.
- **V2:** the control reproduces port 2's deterministic stages exactly: MTF stage
  988,695,280 Ir and build stage 1,168,322,828 Ir (port 2's archived team arm), each within
  0.001 Ir/D. This holds only if master's merge changed neither function; if it fails, it is
  reported as unmet.
- **V3:** the candidate's one-worker MTF stage equals port 2's archived one-worker MTF stage,
  862,123,747 Ir, within 0.001 Ir/D (G3: `mtf_avx2` unchanged).

**Instruction identity.** The witness is each binary's SHA-256, address, objdump text and
execution count. An address objdump cannot decode, or a named instruction not found, is
**unmet**, never zero.

- **I1, function identity:**
  - **Candidate team arm:** `mibz2::mtf::mtf_checkpoints_reg_avx2` is called exactly 431
    times; `mtf_checkpoints_avx2` has 0 Ir (it is not in the binary outside tests);
    `mtf_avx2` has 0 Ir.
  - **Control team arm:** `mtf_checkpoints_avx2` is called 431 times, the reg kernel does
    not exist, and `mtf_avx2` has 0 Ir. Every block's MTF is the checkpoint kernel.
- **I2, the byte steps:** in the candidate's `mtf_checkpoints_reg_avx2`, the executions of
  its one-byte stores to non-stack memory (objdump `mov %Rb,(…)` whose address does not use
  `%rsp`), summed over sites, equal **exactly 15,148,085**.
  - **Where that comes from:** one worker's `mtf_avx2` byte steps in port 2's archived
    one-worker profile: its single such store, `mov %cl,(%r9,%r12,1)` at `0xe70d3`,
    ×15,148,085, binary `a898ef13…` (rebuilt bit-identical for this registration).
  - **It is the count of byte symbols,** independent of checkpoints. The control's
    `mtf_checkpoints_avx2` has 15,135,660 + 12,425 = 15,148,085 at its two non-stack byte
    stores (`0xcea12`, `0xced98`).
- **I3, the 32-byte run stores:** in the candidate's kernel, the non-stack
  `vmovdqu %ymm,(…)` stores come in pairs of equal count. The first store of each pair,
  summed over pairs, executes between **4,027,455 and 4,053,053** times.
  - **The lower bound** is one worker's run-store pairs: `0xe6e4b`/`0xe6e51` ×4,027,455.
  - **Splits only add:** a run split at a checkpoint changes its stores from one vector
    store or one byte fill into at most two vector stores, so each crossing adds at most 2.
  - **The upper bound** adds 2 × 12,799, at most one crossing per checkpoint, ⌊N / 16,384⌋.
  - **The control** has no non-stack `vmovdqu %ymm` store in `mtf_checkpoints_avx2` (its
    runs go through memset).
- **I4, every block:** the 431 calls of I1 are made by the team's front stage (the caller
  named in the call records).

## Resource conditions (recorded, not gating; D088 §3)

Before each arm:
- `pgrep -c rustc`, `pgrep -c valgrind`, the load average, `nproc` and `df -h /`.

After each arm:
- ab's `runq_s` and `sampler_s`.

No threshold applies, and none is set afterwards.

## Dispatch paths (G3)

| path | changes? | why | evidence |
|---|---|---|---|
| pipelined team front (`parallel::front_stage` → `mtf_checkpoints`) | **yes** | the port | this count, the tests |
| ordinary teams (`decode_block` with a checkpoint-wanting inverse → `mtf_checkpoints`) | **yes**, the same function | the same | the tests; not counted |
| plain worker `mtf::mtf` / `mtf_avx2` | **no** | untouched | V3; CI's A/B |
| `mtf_checkpoints_scalar` (no AVX2) | **no** | untouched; C29's kernel falls back to it, as before | the tests |

## Tests before the count

- **C29's tests, ported:**
  - every checkpoint and raw-store class, valid and malformed;
  - the reach of every class;
  - the Miri test of its AVX2 stores.
- **The comparison:** the new `mtf_checkpoints` against the old kernel (kept as the test
  reference) and against the scalar twin.
- **The decode tests:** `tests/pipelined.rs` and `tests/cooperative.rs` through the team
  paths.
- **Mutants:** at least three of the kernel's limit and split logic, each caught.

**Not claimed:** time, adoption, merge. C29's timed R5 was INCONCLUSIVE and closed timing on
its runner. This is a count, and it does not reopen that.

**B5:** C29's earlier B5 evidence is cited, not inherited. An independent review of this
port is a separate agent's.
