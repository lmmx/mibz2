# Run-fed: why B's fallback is slower than A's (#277's all-fallback corpora)

> **Corrected (Optimiser REVIEW
> [6734171410](https://github.com/lmmx/mibz2/issues/145#issuecomment-6734171410)).** This
> entry first concluded that the fallback penalty "is harness layout". The evidence supports
> less: **local same-kernel evidence narrows extra-work explanations; address geometry and
> process placement remain live hypotheses, and 1 KiB dependency evidence requires revising
> the census.** No measured credit and no CPU-causal conclusion follow from it. See
> [§6](#6-correction-what-the-evidence-supports).

9 November 2026. This is a static, untimed investigation for the run-fed line (PROTOCOL §11). It
answers the Researcher's control (6733290162): on #277's three all-fallback corpora, B/A is
1.06–1.37 while D/A is level.
- **No wall-clock figure here is a new measurement.** The only times quoted are #277's
  registered run (sweep run 42567752634, head `aa409f6f`) and #269's (run 42549324025, head
  `fd60fffa`), read back from their artifacts.
- **Everything else is a deterministic count:** disassembly, cachegrind, allocation and fault
  counters, and stack addresses read under gdb.
- **Nothing measured changed.** The probe below is a scratch copy of the harness, not committed
  to `research/run-fed-direct`.

## The puzzle, from the artifact

The pinned copy, `run-fed-1.json`, nominal c/B:

| corpus | A entry | B entry | D entry | A phase fallback | B phase fallback | D phase fallback |
|---|---:|---:|---:|---:|---:|---:|
| text | 10.377 | 11.007 | 10.347 | 10.361 | 10.398 | 10.357 |
| random | 30.278 | 38.968 | 30.426 | 30.265 | 39.002 | 30.290 |
| records | 25.037 | 34.498 | 25.002 | 25.060 | 34.222 | 25.109 |

Three facts in the artifact itself narrow the question before any listing is read:

1. **The phase sweep runs one function for all three arms on a fallback block,** and B is still
   slower there: B/A is 1.2850 on random and 1.3616 on records.
   - That function is `stage`'s `(_, false)` arm: `run_fed_stage::stage::{closure#7}`, at
     0x22e00 in the local build. It calls `mtf_avx2`, then tail-jumps to `bwt::build`.
   - On text, the phase B/A is 1.0033, against 1.0637 at the entry.
2. **The penalty is in every round,** at every rotation position.
   - Entry B/A on random, by round: 1.244 1.297 1.292 1.304 1.276 1.287 1.277 1.286 1.287 1.290
     1.299 1.294 1.270 1.289 1.270.
   - On records it runs from 1.354 to 1.392.
   - So it is a fixed property of B in that process, not cache history, warm-up or arm order.
3. **The four contended copies of the same binary on the same input don't show it.** Their B/A
   at the entry is 0.999–1.001 on random and 0.998–1.007 on records. Their per-round B/A on
   random holds in 0.987–1.009.
   - All three arms are slower there, at about 43.7 c/B on random. That is more than B's 39.0
     pinned.
   - The pinned copy is the only judged one.

#269's registered run, on an Intel 6973P-C, shows its own pinned-copy-only arm effect:
- **Wikidata's fallback blocks:** R_fb B/A 1.1095 pinned, against 0.985–1.006 in its four
  contended copies.
- **Random and records:** B/A is 1.011 and 0.996 pinned.

## 1. What each arm executes on a fallback block

Read from `examples/run_fed_stage.rs` and `src/run_stage.rs` at `aa409f6f`.

| | A (entry sweep) | B (entry sweep) | D (entry sweep) | all arms (phase sweep) |
|---|---|---|---|---|
| wrapper | `entry` arm 0, inlined in `main` | `run_stage::build` | `run_stage::build_direct` | `stage::{closure#7}` |
| selector | none | `takes_runs`: 2 loads, 2 compares | `takes_runs`, the same | (`takes_runs` is called in `stage`, untimed) |
| MTF | `mtf::mtf` (a 3-instruction trampoline), then `mtf_avx2` @0x4b1d0 | `mtf_avx2` @0x4b1d0, called directly | `mtf_avx2` @0x4b1d0 | `mtf_avx2` @0x4b1d0 |
| build | `bwt::build` @0x45d90 | `bwt::build` @0x45d90 | `bwt::build` @0x45d90 | `bwt::build` @0x45d90 (tail jump) |
| walk, RLE1, CRC | `walk_check`, untimed, on its own `Bufs` | the same | the same | the same |

**Buffers:** each arm has its own `Bufs`, the same type, made fresh for each file.
- **What B and D leave untouched on a fallback-only file:** `recs`, and B's `Scratch`. Both stay
  empty `Vec`s, 0 bytes, never touched.
- **On a fallback block of n rows, for every arm** (`max_len` 900,000):
  - **`mtf_avx2`:**
    - `ll.reserve(max_len)`, a no-op once the capacity is there;
    - `counts` zeroed, 1 KiB, by `memset`;
    - one `counts[c]` read-modify-write and one byte stored to `ll` per literal;
    - a 0x1c0-byte frame, 64-aligned, holding the 224-byte second-level list `rows` at
      rsp+0x40, which a literal of index ≥32 rewrites.
  - **`bwt::build` (`build_one_pass`):**
    - a 0x808-byte frame holding `ftab` and `lim`, 2 KiB;
    - `tt.reserve(n)`, a no-op once the capacity is there;
    - 4n bytes scattered into `tt`, and `ll` read 2n times (one sequential stream, one per
      bucket);
    - no zeroing of `ll` or `tt`.
- **The wrapper's own work:** B's fallback path in `run_stage::build` is about 47 instructions
  a block: the prologue, `takes_runs`, moving the arguments, result handling and the epilogue.
  D's in `build_direct` is about the same. A's inline path is shorter by a few dozen. A block's
  stage is about 10⁸ instructions.
- **No per-block work that B does and A doesn't:** no record emission, no scratch, no F array, no
  census, no check and no timer.

## 2. Code generation

Built as the sweep builds it: `cargo +1.99.1 build --release --features run-fed --example
run_fed_stage`, the same rustc (1.99.1, 58cf8f9c3), no RUSTFLAGS, and the release profile
(`lto = "fat"`, `codegen-units = 1`). Disassembled with `objdump -d --no-show-raw-insn -C`.

**Every call into a hot kernel on a fallback block, in the whole binary:**

```text
main (A's entry):            call 4ca60 <mibz2::mtf::mtf>        -> jne 4b1d0 <mtf_avx2>
                             call 45d90 <mibz2::bwt::build>
run_stage::build (B):        call 4b1d0 <mibz2::mtf::mtf_avx2>
                             call 45d90 <mibz2::bwt::build>
run_stage::build_direct (D): call 4b1d0 <mibz2::mtf::mtf_avx2>
                             call 45d90 <mibz2::bwt::build>
stage::{closure#7} (phase):  call 4b1d0 <mibz2::mtf::mtf_avx2>
                             jmp  45d90 <mibz2::bwt::build>
```

- **One instance of each kernel:** `mtf_avx2` is 0x4ba bytes and `bwt::build` 0xe16 bytes, with
  `build_one_pass` inlined. Neither is inlined into any wrapper.
- **No second instantiation:** neither kernel is generic. Their hot loops, register allocation
  and stack spills are the same bytes at the same addresses for A, B and D.
- **Inlining, register pressure, spills in the hot loops, monomorphisation and code layout of
  the kernels can't differ between arms.**
- **The only code that differs** is the wrapper's few dozen instructions a block, outside every
  loop.

**The wrappers' stack frames do differ.** Each puts the kernels' frames at a different depth
below `main`:

| caller of `mtf_avx2` | frame above it | `mtf_avx2`'s `rows` table, from that arm's `counts`, mod 4 KiB |
|---|---|---|
| A, entry: `main` (via the trampoline) | 0 | +0x7b8 |
| B, entry: `run_stage::build` | 8 + 48 + 0x68 = 160 B | **+0x210** |
| D, entry: `run_stage::build_direct` | 8 + 48 + 0x38 = 112 B | +0xd68 |
| all, phase: `main` → `stage` → `{closure#7}` | the same for all arms | A +0x5f8, **B +0x110**, D +0xc28 |

These offsets are read under gdb, not computed. The script is in this entry's directory,
`stackprobe.gdb`.
- **What it does:** it stops in `mtf_avx2` after its prologue and records the `counts` pointer
  (rbx) and the realigned frame base (rsp).
- **The run:** the harness binary itself, on Wikidata plus the synthetics, 2 rounds, with ASLR on.
- **The distinct call sites:**

```text
STACK caller=entry <- main     counts=…cf08 rows%4K=0x6c0  (A, entry)   counts window 0xf08..0x308
STACK caller=mtf <- build      counts=…d3f0 rows%4K=0x600  (B, entry)   counts window 0x3f0..0x7f0
STACK caller=mtf <- build_direct counts=…d8d8 rows%4K=0x640 (D, entry)  counts window 0x8d8..0xcd8
STACK caller=mtf <- {closure#7} counts=…cf08 / …d3f0 / …d8d8  rows%4K=0x500 (phase, all arms)
```

**Only B has `counts` under its own MTF frame, in both sweeps.** `counts` is 1 KiB, in the arm's
`Bufs`, on `main`'s stack, and read-modify-written on every literal.
- **B:** modulo 4 KiB, `counts` covers all of `mtf_avx2`'s frame: `rows` at rsp+0x40..0x120, and
  the spills read every literal, at rsp+0x10 (`max`) and rsp+0x20 (the `counts` pointer). That
  holds at the entry and in the phase sweep.
- **A and D:** their frames sit at least 0x118 bytes (280) clear of their `counts` window in both
  sweeps.
- **The stack's alignment residue doesn't change this.** `mtf_avx2` realigns its frame with
  `and $-64`, which moves `rows` against `counts` by at most 48 bytes. B's frame spans 0xd0–0x290 (phase)
  and 0x1d0–0x390 (entry) of the 0x400 window, at least 112 bytes from either edge, so every
  process has it.

**The same check on #269's binary** (`fd60fffa`, the same toolchain): no arm's `counts` overlaps
its `mtf_avx2` frame in either sweep.
- **The frame bases from `counts`, mod 4 KiB:** A +0xbc0 and +0xa00, B +0x600 and +0x500, D +0xcc0
  and +0xb00, each frame 0x1c0 bytes.
- **What #269 showed:** B/A on random and records was 1.011 and 0.996.

## 3. Dynamic counts, untimed

**The probe.** `fallback_probe.diff` in this entry's directory is the harness unchanged, plus:
- a counting global allocator;
- `/proc/self/stat` minor faults, around each arm's sweep;
- each buffer's address, its mapping, and its transparent-huge-page coverage, from
  `/proc/self/pagemap` and `/proc/kpageflags`;
- `PROBE_ARM`, `PROBE_FILES` and `PROBE_ALIAS_ONLY`, to restrict a run.

It ran on the shared 4-core machine (Intel Xeon @ 2.30 GHz, no SMT, THP `madvise`) at the full
15 rounds.

**Buffers, per arm, steady state:**
- **Fallback corpora:** identical allocation calls and bytes per sweep for all three arms
  (random: 993 calls and 8,574,201 bytes over 15 entry sweeps; records: 1,014 calls and
  8,674,413 bytes).
- **Faults:** 0 minor faults after the first sweep, for every arm on random and records. On text,
  D took 134 minor faults, in its first sweep only.
- **Capacities:** identical: `ll` 900,000, `tt` up to 3.6 MB, `out` about 900 KB.
- **Pages:**
  - **Without the tunable** (`madvise` mode): every buffer of every arm is on 4 KiB pages.
  - **With `GLIBC_TUNABLES=glibc.malloc.hugetlb=1`:** every one is fully THP-backed.
  - So on this machine page backing never separates the arms.

**Cachegrind** (valgrind 3.22): `--cache-sim=yes --I1=32768,8,64 --D1=32768,8,64
--LL=1048576,8,64`, so LL models a Zen 4-sized L2.
- **The method:** each arm alone (`PROBE_ARM`) on one corpus (`PROBE_FILES`), at rounds 3 less
  rounds 2.
- **What the count covers:** exactly one round, one entry sweep and one phase sweep over the
  corpus's blocks, with the untimed walks. Corpus generation and validation cancel.

| corpus | arm | Ir | D1 read misses | D1 write misses | LL read misses | LL write misses |
|---|---|---:|---:|---:|---:|---:|
| random | A | 378,022,341 | 4,714,362 | 788,527 | 3,029,391 | 494,951 |
| random | B | 378,021,953 | 4,714,606 | 788,481 | 3,029,384 | 494,968 |
| random | D | 378,022,592 | 4,714,585 | 788,473 | 3,029,375 | 494,952 |
| records | A | 509,174,450 | 6,226,751 | 1,496,613 | 3,769,051 | 602,442 |
| records | B | 509,174,681 | 6,227,513 | 1,496,476 | 3,769,009 | 602,462 |
| records | D | 509,187,161 | 6,227,532 | 1,496,467 | 3,769,033 | 602,459 |
| text¹ | A | 256,536,944 | 7,070,011 | 950,457 | 4,326,427 | 808,919 |
| text¹ | B | 256,543,772 | 7,069,943 | 950,425 | 4,326,421 | 808,952 |
| text¹ | D | 256,537,344 | 7,070,264 | 950,447 | 4,326,400 | 808,937 |

¹ The text corpus is read from the checkout's sources, and the probe file is one of them, so it
differs slightly from CI's.

**What the table shows:**
- **B and A agree within 0.003% on instructions** on every corpus.
- **Misses agree within 0.02%** on D1 and 0.002% on LL.
- **The leftover differences include the probe's own `/proc` reads,** which vary with where each
  arm's buffers are mapped.
- **So B executes the same work as A.** A 28–37% gap is no instruction, cache-miss or allocation
  count.
- **What cachegrind can't see:** TLB, physical indexing, and false store-to-load dependences.
  Those are what remain.

**Exposure to the overlap, per corpus.** `alias_census` in the probe replays each fallback
block's MTF list. It counts the literals of index ≥32, whose `rows` stores and loads would meet
B's overlapping `counts` slots.
- **B's windows:** at the entry, slots [132, 188); in the phase sweep, [68, 124).
- **"self":** a deep literal whose own byte's slot is in the window.
- **"pair":** a literal in the window followed by a deep literal.

| corpus (fallback blocks) | literals | index ≥32 | entry: self / pair | phase: self / pair | observed B−A, entry / phase (c/B) |
|---|---:|---:|---:|---:|---:|
| Wikidata | 4,759,058 | 1.6% | 0.00% / 0.00% | 1.09% / 1.05% | (fallback blocks only) R_fb 1.021 |
| text¹ | 834,782 | 4.7% | 0.06% / 0.01% | 2.59% / 2.60% | +0.630 / +0.037 |
| random | 1,984,310 | 75.8% | 16.57% / 16.55% | 16.56% / 16.59% | +8.690 / +8.737 |
| records | 2,392,369 | 90.5% | 19.77% / 19.77% | 19.80% / 19.82% | +9.461 / +9.162 |

**Random and records fit one cost per event.** Both corpora are generated from a seeded RNG, so
they are CI's.
- **Per round:** 8.69 c/B × 2.0 MB over 329 k exposed literals is about 53 nominal cycles each,
  and 9.46 c/B × 2.4 MB over 473 k is about 48.
- **The same cost** fits both corpora and both sweeps.

**Text doesn't fit.**
- **The phase sweep** exposes 2.6% of text's literals and costs almost nothing (+0.037 c/B).
- **The entry** exposes 0.06% and costs +0.630 c/B.
- **So text's entry-only 6% is a second effect,** tied to the entry path's frame depth. It is not
  this overlap.

## 4. Ranked hypotheses

1. **A false store-to-load dependence between B's `counts` and its own `mtf_avx2` frame**
   (4 KiB aliasing, or a memory-dependence misprediction keyed on low address bits).
   - **For:**
     - It is the only arm-specific difference found anywhere on the fallback path.
     - It is B's alone, in both sweeps, where A's and D's frames are clear.
     - It survives every alignment residue.
     - It is absent in #269's binary, whose random and records B/A were level.
     - Its exposure is large exactly on the two corpora that pay 28–37% (deep MTF indices,
       uniform bytes), at one consistent cost of about 50 nominal cycles an event.
   - **Against, or unexplained:**
     - Zen 4's exact disambiguation rule is not documented here, and about 50 cycles an event is
       large for plain 4K aliasing.
     - It predicts a penalty in the contended copies too, which they don't show. A possible
       reason is that with SMT siblings sharing a core (unrecorded for the runner), the copy is
       throughput-bound and a latency stall hides. That is unverified.
     - It doesn't explain text.
2. **Per-process memory placement of B's buffer set:** page size or THP coverage, the host's
   backing of those guest frames, or physical cache-set colouring.
   - **For:** it is a per-process lottery, which fits "pinned copy only" and #269's own
     pinned-only Wikidata R_fb 1.11.
   - **Against:**
     - B's buffers were allocated identically, with the same sizes, calls and faults.
     - Nothing on this machine separates their page backing in either THP mode.
     - It can't explain text's entry-only penalty, since both sweeps use the same buffers.
     - The runner's THP mode was not recorded.
3. **Entry-path frame depth for text:** B's wrapper puts the kernels 160 bytes deeper than A's, at
   the entry only.
   - It is the only thing that separates text's entry sweep (1.064) from its phase sweep (1.003).
   - The mechanism inside the kernels is not identified.
4. **Ruled out: code generation of the kernels.** There is one instance of each, shared by every
   arm and by the phase sweep. There are no other instantiations and nothing is inlined.
5. **Ruled out: buffer sizes, allocation, zeroing, or B doing record work on fallback blocks.**
   Instructions, allocations, faults and simulated misses are equal.
6. **Ruled out: cache history and arm order.** The penalty is constant across all 15 rotated
   rounds.

**What this changes** (as corrected in §6): B executes the same kernels, the same instructions
and the same simulated misses as A on a fallback block, so #277's fallback B/A is no measured
cost of B's extra work. It is not shown to be a layout effect either.
- **Charge nothing to the records, the selector or the CPU from it,** and credit B nothing for
  removing it.
- **The same caution applies to #269's pinned-only R_fb 1.11.**

## 5. The proposed control: a shared fallback (superseded by §6.3)

This section's design is kept as first written. §6.3 replaces it.

This is for a later registered diagnostic; it is not implemented on the measured branch. Its
arms share one fallback function, and the layout is crossed with the arm:
- **One fallback function.** `#[inline(never)] fn fallback(b, ll, counts, tt, k)` is the only
  path to `mtf_avx2` and `bwt::build`.
  - A's entry, B's `run_stage::build`, D's `build_direct` and the phase sweep all call it.
  - So the kernels' frames sit at one depth below the wrapper, whatever the arm. The wrappers
    may keep their selectors.
- **`counts` off the stack, at one 4 KiB offset for every arm.** Move `counts` into a
  heap-allocated, 4096-aligned `Bufs` (a `Box`ed `#[repr(C, align(4096))]` struct).
  - That gives every arm's `counts` the same offset modulo 4 KiB.
  - Its relation to the shared frame is then the same in every arm.
- **A layout factor crossed with the arm.** Each round, each arm takes a buffer set by rotation,
  so arm i uses `bufs[(i + r) % 3]`. Times are recorded by set as well as by arm, with one
  deliberately aliasing set.
  - That set's `counts` sits under the shared `mtf_avx2` frame mod 4 KiB, by a padding of the
    measured offset.
  - **The prediction under H1:** the penalty follows the aliasing set, not the arm, and appears
    in A and D when they hold it. B/A and D/A on random and records then fall within ±2% on the
    clean sets.
  - **Under H2:** it follows the process and the set's pages, not the offset.
- **A/A' as a negative control.** A second, independent A arm on its own buffer set; its ratio
  to A must be level.
- **What to record:**
  - each set's `counts` and `ll`/`tt` addresses mod 4 KiB and 2 MiB;
  - `/sys/kernel/mm/transparent_hugepage/enabled`;
  - `AnonHugePages` from `/proc/self/smaps` for each buffer's mapping (no root needed);
  - `lscpu`'s threads per core, so SMT is known for the contended copies.
- **Replication:** at least 3 sequential pinned processes, not one, so a per-process placement
  effect (H2) shows as variance between processes.

The control's cheapest falsification is the aliasing set alone. If B on a clean set is level
with A while A on the aliasing set is 28% slower, H1 is confirmed and the fix is a layout
discipline for every harness. If the penalty stays with B's process and not with the set, H1
falls to H2.

## 6. Correction: what the evidence supports

From the Optimiser's REVIEW 6734171410 (report, ledger and paired arithmetic at
`8a6bf17c33f25bf0c3c358826d7bae0e62b54633`, `state/records/reviews/inbox-6734039567/`).

### 6.1 The discriminator was mod 4 KiB; Zen 4 also shows 1 KiB

Agner Fog's Zen 4 measurements report false store-to-load dependences at **1 KiB spacing**.
§2's control fact, that A's and D's frames are clear of their `counts` window mod 4 KiB, then
separates nothing:
- **`counts` is exactly 1 KiB** (256 × `u32`). Modulo 1 KiB it covers every residue, so every
  byte of every arm's `mtf_avx2` frame meets some `counts` slot mod 1 KiB, in A and D as in B.
- **What differs between arms at 1 KiB** is only *which* slot each frame byte meets, and so
  which literals are exposed. §3's census counted exposure mod 4 KiB only, B's window alone.
- **Open:** the census recounted at both 1 KiB and 4 KiB for all three arms, from the actual
  access addresses. Until then §3's exposure table is no evidence for H1 over H2.

### 6.2 The fitted cost is not a measurement

The ~50 nominal cycles per exposed literal in §3 is a fit of a c/B gap to a count. It is
neither a measured event latency nor evidence that chooses aliasing over placement. §4's H1
and H2 both stay live; H3 (text's entry depth) is unchanged.

### 6.3 The revised control

§5's shared callee still inherits a different caller stack depth per arm. The control, for a
later registered diagnostic:
- **A common-call-site isolation path,** plus an unchanged-entry reference arm, so the layout
  effect and the shipped entry are both read.
- **Crossed factors, varied independently:** backing-set identity, the relative offset of
  `counts` against the kernel frames, the arm, and the order; with **A/A′**.
- **Reciprocal within-set offset interventions:** a penalty that follows a buffer set fits
  physical placement as well as aliasing, so the offset is moved within one set, both ways.
- **Layout as a crossed factor on both hosts** (the EPYC 9V74 and the Intel 6973P-C), not a
  special B-only arm.
- **Logged geometry:** preallocated heap `counts`, every buffer's address mod 1 KiB, 4 KiB and
  2 MiB, THP state and SMT, and matched initialisation.
  - `align(4096)` alone is no cure, and it does not align a `Vec`'s backing allocation.
- **Not acceptance criteria:** §5's "three processes" and "±2%" are dropped. The registration
  sets its criteria.

### 6.4 B's lead, with the fallback excess removed (noncausal)

On #277's archived 15-round replay, B/A is **0.976269848**. Replacing only B's fallback seconds
with the same round's A fallback seconds gives **0.970909695**: the observed fallback excess
costs B about 0.536 percentage points.
- This is a noncausal counterfactual, not a corrected result.
- Layout elsewhere could explain all of B's 2.373% lead. **No layout-free gain is established.**

### 6.5 What stands

- **Ranking:** C29 first; the corrected layout attribution before C43. B stays a diagnostic
  reference. C43's 6.44/run allowance is provisional.
- **Unchanged:** #277's D STOP and every existing registration.

## Commands

```bash
git worktree add --detach wt aa409f6f2ebce9f41dc7902789f0ab8d6958b020
cargo +1.99.1 build --release --features run-fed --example run_fed_stage
objdump -d --no-show-raw-insn -C target/release/examples/run_fed_stage > all.s
# the corpus as CI builds it, from the bench-data branch
git archive origin/bench-data windows/wikidata-20261019 | tar -x
python3 bench/wikidata.py build bench/wikidata.json windows/wikidata-20261019 \
  target/release/examples/windows corpora/wikidata.bz2           # SHA-256 checked
# stack layout per arm (ASLR on)
gdb -q -batch -x stackprobe.gdb --args target/release/examples/run_fed_stage corpora/wikidata.bz2 2 3.6937
# the probe (fallback_probe.diff applied as examples/fallback_probe.rs)
cargo +1.99.1 build --release --features run-fed --example fallback_probe
RUN_FED_HEAD=aa409f6f… target/release/examples/fallback_probe corpora/wikidata.bz2 15 3.6937
GLIBC_TUNABLES=glibc.malloc.hugetlb=1 target/release/examples/fallback_probe corpora/wikidata.bz2 15 3.6937
PROBE_ALIAS_ONLY=1 target/release/examples/fallback_probe corpora/wikidata.bz2 2 3.6937
for c in random records text; do for a in 0 1 2; do for r in 2 3; do
  PROBE_FILES=$c PROBE_ARM=$a valgrind --tool=cachegrind --cache-sim=yes \
    --I1=32768,8,64 --D1=32768,8,64 --LL=1048576,8,64 \
    target/release/examples/fallback_probe corpora/wikidata.bz2 $r 3.6937
done; done; done
```

**Caveats:**
- **The build is local, not CI's binary.** Code addresses may differ from CI's by the embedded
  paths. Frame shapes and stack offsets depend only on the code, which is the same source and
  toolchain.
- **The shared machine is an Intel host without SMT.** Cachegrind's figures are simulated, and
  gdb's offsets are the binary's.
- **Not established:** which hardware rule turns the overlap into about 50 cycles an event on
  the EPYC 9V74.
