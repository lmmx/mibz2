# P87's four-core diagnostic: the assembly into the dead `ll` (protocol)

**Status: registered before any timed run.**
- **Evidence cutoff:** 1 November 2026, before any timed P87 experiment. Every figure
  below is from earlier records; nothing here is a P87 result.
- **Freeze:** `2026-11-01-p87-four-core/frozen.json`, the pin `bench/p87-pin`, and
  `2026-11-01-p87-four-core/freeze.md`.
- **The run:** one command, `just bench-p87 LABEL` (`bench/p87_run.sh`). It runs on the
  i9-10900K only under the maintainer's explicit authorisation.

## 0. What this is, and what it is not

**This is a new, separate diagnostic** ("P87, four cores"). It does not evaluate P87's
historical registered criterion, which stays as recorded and is not amended. The entry in
the research portfolio register reads, verbatim (`docs/journal/2026-10-30-research-portfolio-register.md`
on `docs/research-portfolio`, 9723f06, lines 813–818 and 1475):

> **P87 Walk output in L's freed buffer.** Status: **open** (not run; "behind the cooperative work").
> - No run. bz2ngrs's gather (its form of this): 10–15% faster than our assembly on the VM but
>   0.026 ns/B (< 1% of the inverse BWT); 1.03× (1.02–1.04) slower on CI.
> - Corrected mechanism: it saves ~0.9 MB of resident memory a block but "does not shrink the walk's
>   hot working set" (K46).
> - Pass bar: the occupancy curve at 6–10 workers ≥ 1.03 × today's, with DRAM write bytes per
>   output byte.

> | E9: output in L's buffer | P87 | occupancy curve at 6–10 workers ≥ 1.03× today's, with DRAM write bytes per output byte |

That criterion concerns 6–10 workers. This diagnostic runs four, so it can neither meet
nor fail that bar. Its two decisions (§7) are its own, and neither stands in for the other
or for the historical bar.

**Not done here:**
- no production default changes;
- no allocator change;
- nothing else is optimised;
- no closed experiment is modified or rerun;
- the neighbour harness (the inverse-BWT note's §5) is not built.

## 1. Why: what the earlier records establish, qualified

From `2026-11-01-bwt-memory-proposal.md` and its corrections after review (reports
125–129, recorded verbatim in their own documents):

- **The inverse BWT is where the four-worker cost grows.** Full-trace phase times on the
  attribution run (i9, CPUs 1–4):
  - per BWT row, on the 197 full 900k-row blocks: 1.234 → 1.806 ns (+0.572);
  - over all 434 blocks, per decoded byte: 1.236 → 1.847 CPU-s/GB (+0.611; the
    researcher's aggregation gives 0.609).
- **Both denominators are kept.** BWT rows and decoded bytes differ by RLE1. The
  decoder-level opportunity accounting uses the whole-corpus figure, ≈ 0.61 CPU-s/GB.
- **That interval is traversal plus assembly.** The archive's `WalkStart` → `InvertEnd`
  bracket includes `SegScratch::assemble`. This diagnostic adds a mark between the two
  (`AssembleStart`, P87's path only; §8).
- **The overlap association is observational.** Concurrent-walk overlap against walk
  cost is subject to:
  - reverse causation;
  - 1,970 walks that are repeated observations of 197 blocks, not independent replicates;
  - sparse extreme bins.

  It motivates looking at memory interference. It does not establish it.
- **"About six in flight at constant MLP" is an inferred model.** It is not a
  measurement. The permissible statement: the increase is *consistent with* slower request
  service at roughly unchanged overlap.
- **The "98% of the walk's traffic is prefetch" figure is withdrawn.** It is unsupported.
- **A neighbour harness that fails calibration means "the harness is not
  representative".** That is not a disproof. That harness is not built here.

**What P87 changes.** Per block, a worker writes MTF's output into `ll`. The build reads
`ll` into the T-vector `tt`, and nothing reads `ll` again. The walk writes the segments.
The assembly joins them into `rle`, and RLE1 and the CRC read `rle`. P87 writes the
assembly into `ll` instead. B therefore touches about one block's size (≈ 0.9 MB) less
distinct memory per worker per block, and B's write target was written by MTF in this
block rather than read by RLE1 in the previous one.

**Neither arm writes fewer bytes.** The assembly writes n bytes in both. What can differ:
- which lines are written;
- their residency and dirtiness when written;
- the read-for-ownership and writeback traffic that follows.

DRAM traffic is therefore measured (S3), not assumed.

## 2. Hypotheses

- **H-mech (cross-worker or persistent memory cost).** With four workers, each worker's
  traversal and assembly cost more partly because the workers' combined footprint and
  writeback traffic compete in the shared L3 and memory controller. B's smaller distinct
  footprint reduces that cost more at four workers than at one.
  - Prediction: D (§7.1) ≥ 0.10 CPU-s/GB.
- **H-local (a single-worker effect).** B's write target is more recently touched, and
  so cheaper to write, in a single worker as much as in four.
  - Prediction: B saves at w = 1 and at w = 4 alike (Δ1 ≈ Δ4), so D ≈ 0.
  - The engineering gate can still pass.
- **H-null.** Neither: D ≈ 0 and no engineering gain.
- **H-harm.** B is slower, for example from aliasing or writeback interference between
  `ll`'s lines and the T-vector's, or a cache-set conflict.
  - Prediction: engineering NEGATIVE; D of either sign.

**Temporal direction.** A block's traversal precedes its own assembly. So a traversal
difference between the arms cannot come from that block's own assembly. It is either:
- **persistent:** an earlier block's assembly left a different cache state, which a
  single worker sees too; or
- **cross-worker:** other workers' assembly traffic, which grows from w = 1 to 4.

S2 reports the two parts apart (§8.1).

## 3. The arms, and what is equalised

Both arms run `mibz2::p87::decode_block::<Segmented>` (`src/p87.rs`, feature `p87`; §5
shows it is one compiled path). It makes `Segmented::invert`'s own calls in its order:
`bwt::build`, then `walk_segments_room_parts`, then `SegScratch::assemble`.

| | A | B | A′ (A2) |
|---|---|---|---|
| the walk's and assembly's destination | `rle` (the shipped buffer) | `ll`, after the build's last read | `rle` |
| RLE1 and CRC read | `rle` | `ll` | `rle` |
| `rle` after the first block | written per block | allocated, resident, never written again | written per block |

**What is the same in every arm:**
- **The code:**
  - the 4-byte T-vector and the build;
  - the 32-lane room walk with its prefetch;
  - the segment buffers and the assembly copy (its byte count is n in both: `out.clear(); out.reserve(n)`, then the joins);
  - RLE1 and the CRC;
  - the decoder's dispatch, admission and ordering;
  - the scanner, feeder and reader;
  - `StreamDecoder::with_inverse::<Segmented>(w, 2w, Kernels::detect())`, as
    `Plan::Workers(w)` builds it;
  - the binary.
- **Borrows.** The build borrows `ll` immutably, and that borrow ends before `ll` is
  borrowed mutably as the destination. The compiler enforces this.
- **Allocation, capacity and initial residency.** On each worker's first block, `ll` and
  `rle` are each:
  - allocated at exactly the level's block capacity (`b.max_len`; 900,000 at level 9);
  - written once with zeros, then emptied, keeping their capacity and resident pages.

  The order of the two allocations is the run's placement (`MIBZ2_P87_PLACEMENT`: 0, `ll`
  first; 1, `rle` first). After that neither buffer is reallocated on this corpus (tested:
  capacities 900,000 at the first block and at the end).
- **B's unused arena.** B keeps `rle` allocated and resident throughout. It is warmed
  once, inside the timed decode, exactly as A's is, and never warmed or dirtied again
  (tested: its length stays 0).
- **Placement balance.** The placement is the round's parity, the same for every arm in a
  round. So each arm meets each placement in exactly 10 of the 20 rounds (checked from the
  rows; an imbalance aborts the analysis).
- **The small-block fallback.** A block shorter than 65,536 rows is walked plainly into
  the destination, in both arms.
- **Errors.** A bad primary index, a CRC mismatch and an RLE1 error end with the same error
  and the same output kept as the shipped path's (tested).
- **A mixed-level stream.** A later stream at a larger level grows a buffer by the
  ordinary `reserve`, in both arms alike. This corpus is level 9 throughout.

**A is not the shipped binary.** The diagnostic's worker body holds:
- both paths (the shipped one, which neither arm takes);
- the arm read (one relaxed atomic load per block);
- one more `Scratch` field.

The trial measures B against A, not B against production. The production default is shown
unchanged separately (E3).

## 4. Corpora

The walk trial's frozen files, byte-identical, as the attribution and dispatch
diagnostics used them (hashes in `frozen.json`):

| Corpus | Role | Compressed | Decoded |
|---|---|---:|---:|
| `wikidata-20261026` | primary | 11,953,225 | 207,000,624 |
| `records` | control | 9,587,840 | 9,600,000 |
| `random` | control | 7,097,921 | 8,000,000 |

## 5. Code equivalence, before any timing (`bench/p87_code_audit.py`)

These checks run on the exact binaries the run times. The run stops on any failure.

- **E1, one compiled path for both arms.**
  - The worker's thread body (`with_inverse<Segmented>`'s closure, `decode_block` inlined
    into it) holds exactly one call to `p87::arm`.
  - It holds exactly two call sites of each kernel: `bwt::build`,
    `room_segments::<32, 1>`, `SegScratch::assemble`, `mtf_avx2` and `unrle_avx2`. One is
    the shipped path's, which neither arm takes. The other is the diagnostic path's, which
    both take.
  - It holds no out-of-line `p87::decode_block` except at most one.
  - A third call site would mean a copy per arm, and fails E1.
  - The destination is selected at run time from one register.
- **E2, the shipped kernels.**
  - **Which functions:** the kernels, their scalar twins, the CRC (`crc32`,
    `clmul_update`) and the Huffman reader (`read_item`, `Huff::new`, `Huff::decode_slow`).
  - **The check:** each is present once in the P87 binary and disassembles the same,
    addresses written out, as in the default-feature build of `examples/attribution_decode`
    at the same commit.
- **E3, the production default unchanged (the attribution diagnostic's C6).** The
  default-feature `examples/attribution_decode` at this commit disassembles the same as at
  the base commit `45b366d`, for every function whose name holds `mibz2`
  (`bench/attribution_same_code.py`).
- **Recorded, not gated.** The same audit on the phase instrument (`examples/occupancy`
  with `trace,p87`).
- **At run time (S3, §8.2).** The arms' retired instructions over whole decodes at w = 1
  must agree within 0.5%. If not, the mechanism decision is withheld (§7.3).

**At preparation** (VM, rustc 1.97.0): E1 held, with one arm call and two call sites of
each kernel. E2 held for all 12 functions. Neither was a timed result.

**Unavoidable confounders, reported.**
- **Not the production binary.** The worker body differs from production's, as above.
  That difference is common to both arms.
- **Code alignment.** It is identical for both arms by E1: one path.
- **Data alignment.** The two buffers' addresses differ by allocation order. The placement
  balance averages that out; S1 reports D by placement descriptively.

## 6. Schedule

**S1, the paired trial (uninstrumented).**
- **Cells and arms:** 3 corpora × w ∈ {1, 4}, so 6 cells; arms A, B and A′.
- **Rounds:** 20 timed rounds and one warm-up round, which is discarded.
- **Order within a cell:** the 6 permutations of (A, B, A′) three times, plus (A, B, A′)
  and (B, A, A′), shuffled per cell (Fisher–Yates).
- **Order across cells:** the cell order is shuffled afresh each round (xorshift seed
  `0x8720261101_0001`).
- **Each measurement is a fresh child process,** its decode timed in-process: wall, and
  the process's user + system CPU from just before the decoder is built to the read that
  returns 0.
- **Checks on each child:** the plan is `Workers(w)`; every thread's affinity lies within
  the allocation; the thread inventory is w workers, the scanner, the feeder and main; the
  output's SHA-256 is the frozen one.
- **The parent's wait4:** it gives max RSS, faults and context switches.

**S2, the phases.**
- **Design:** the primary only; 6 repetitions × w ∈ {1, 4} × arms A and B × the trace
  off (level 0) and full (level 2), from the phase instrument.
- **Order:** per repetition, w = 1 then 4. The arm order is AB on even repetitions and BA
  on odd. The mode order is off–full on even and full–off on odd. The placement is the
  repetition's parity.

**S3, the counters.**
- **Design:** the same design and order as S2, at level 0.
- **The memory controller** (`uncore_imc/data_reads/`, `data_writes/`): machine-wide,
  with an idle reading just before it, as long as the median off-run's wall at that w.
- **The core counters** (`{cycles, ref-cycles, instructions}`): grouped, for the decode's
  process and its threads.

**The allocation:** four physical cores, CPU 0's core last, the lowest CPU of each, their
SMT siblings idle (`bench/attribution.py allocation`). The whole process tree is pinned
there.

## 7. S1's analysis and its two decisions (`bench/p87.py summary`)

C(arm, w) is a decode's CPU-seconds per decoded GB. X is its throughput in MB/s.

**The unit of replication is the round.** Each round holds one observation of every arm in
every cell, each its own process. Intervals are over rounds (independent paired runs).
They are never over blocks.

**The statistics.** For each round's paired quantity, a two-sided 95% t interval. The
lag-1 autocorrelation ρ is taken in round order; for ρ > 0, n_eff = R(1−ρ)/(1+ρ) and
df = max(1, ⌊n_eff⌋ − 1). The quantities:
- **Ratios:** per-round log ratios, exponentiated.
- **D and Δ:** raw per-round differences.

### 7.1 The mechanism decision

Per round, on the primary:

  D = [C(A,4) − C(B,4)] − [C(A,1) − C(B,1)].

| D's 95% interval | Outcome |
|---|---|
| lower ≥ 0.10 CPU-s/GB | **MATERIAL** |
| upper < 0.10 | **NOT MATERIAL** (rejected at that size) |
| otherwise | **INCONCLUSIVE** |

**Reported beside D (descriptive):**
- Δ1 = C(A,1) − C(B,1) and Δ4 = C(A,4) − C(B,4), each with its interval;
- D's mean by placement;
- D_AA, with A′ in place of B.

**Why 0.10 CPU-s/GB.** It is about 16% of the whole-corpus walk-plus-assembly increase
(0.61) and about 2.6% of the four-worker decoder's CPU cost.

**Precision.** The walk trial's i9 rows give the noise for an A/A version of D on this
primary: per-round SD 0.083 CPU-s/GB, a 95% half-width of ±0.039 over 20 rounds. So the
trial is expected to settle MATERIAL or NOT MATERIAL unless the true D lies within about
0.04 of the threshold. This is a model: it assumes P87's noise matches that trial's, whose
design also held w = 2 and 3.

### 7.2 The engineering decision

On the primary at w = 4, with B against A:

| Outcome | Rule |
|---|---|
| **IMPROVES** | all of: CPU upper95 < 1; throughput lower95 > 1; every no-harm row PASS; every RSS row PASS |
| **NEGATIVE** | any of: B demonstrably worse (CPU lower95 > 1, or throughput upper95 < 1); any no-harm row NEGATIVE; any RSS row NEGATIVE |
| **NOT DEMONSTRATED** | otherwise |

The rows these rules refer to:
- **No-harm:** each other cell, including the primary at w = 1, B against A.
  - PASS: CPU upper95 ≤ 1.01 and throughput lower95 ≥ 0.99.
  - NEGATIVE: CPU lower95 > 1.01 or throughput upper95 < 0.99.
  - INCONCLUSIVE otherwise.
- **RSS:** every cell, under the walk trial's max rule.
  - PASS: max(B)/max(A) ≤ 1.01.
  - NEGATIVE: the log-ratio lower95 > 1.01.
- **Practical (descriptive, not a gate):** throughput lower95 ≥ 1.02 and CPU upper95 ≤ 0.98.

**What the engineering gate measures.** B with `rle` still allocated: the speed of reusing
`ll`. A production version would also free `rle`, about 0.9 MB per worker. That is not
measured here, and it would need its own registered trial.

### 7.3 Preconditions and calibration

These apply to both decisions, checked in this order:

1. **Correctness.** Every output's SHA-256 is the frozen one; every plan is `Workers(w)`;
   every thread is within the allocation; the thread inventory is right. Otherwise both
   decisions are NEGATIVE (correctness) or ABORTED.
2. **Placement balanced** (§3). Otherwise ABORTED.
3. **A/A.** Two checks:
   - the walk trial's calibration over the 12 rows (X and CPU, A′/A, every cell): the
     count of intervals excluding 1 must be below the binomial k* (3, at P ≤ 0.05);
   - D_AA's interval must contain 0.

   Otherwise both decisions are INCONCLUSIVE (A/A calibration).
4. **The arms' instructions at w = 1 (S3)** within 0.5%. Otherwise the mechanism decision
   only is INCONCLUSIVE (the work differs). If the counters are unavailable, this is
   recorded as unchecked.

### 7.4 The outcome matrix (each pair as `bench/p87.py` states it)

| Mechanism \ engineering | IMPROVES | NOT DEMONSTRATED | NEGATIVE |
|---|---|---|---|
| **MATERIAL** | The cross-worker term is real and B is faster. A production candidate, which needs its own registered trial. | The term is real; B's net effect isn't shown. No engineering change. | The term is real but outweighed. No engineering change; informative for other designs. |
| **NOT MATERIAL** | B is faster, but locally or persistently. An engineering candidate; the four-core contention hypothesis is not supported. | Closes P87 (four cores) negative. | Closes P87 (four cores) negative. |
| **INCONCLUSIVE** | An engineering candidate; the mechanism stays open. | Nothing changes. | No engineering change. |

## 8. Attribution: S2 and S3 (descriptive; they never change §7's decisions except §7.3.4)

### 8.1 The phases (`bench/p87.py phases`)

**From each worker's clocked marks, per block:**

| Phase | Interval |
|---|---|
| MTF, plus the first block's prefault | `DecodeStart` → `InvertStart` |
| build | `InvertStart` → `WalkStart` |
| traversal | `WalkStart` → `AssembleStart` (segmented blocks) |
| short walk | `WalkStart` → `InvertEnd` (blocks too short to segment) |
| assembly | `AssembleStart` → `InvertEnd` |
| post (RLE1 and CRC) | `InvertEnd` → `DecodeEnd` |

**The figures:**
- **Units:** each is reported as thread CPU-s per decoded GB, and traversal and assembly
  also as CPU-ns per BWT row on the segmented blocks. Rows and decoded bytes are kept
  apart.
- **A − B per phase:** with a t interval over repetitions.
- **The traversal's direction** (§2):
  - its single-worker A − B is the persistent part;
  - its four-worker A − B less the single-worker one is the cross-worker part.

**Perturbation.** It is checked independently, against the same instrument with the trace
off, paired within each repetition. The phase figures are to be read only if:
- every (w, arm)'s median CPU and wall ratio, full/off, is within 3%;
- every run's marks are complete and in order, and sum to the decoded bytes;
- both arms show the same rows and blocks.

**The S1 comparison.** The phases' A − B is a sum of thread-CPU intervals. S1's Δ is
process CPU. They are reported side by side, not reconciled into one figure.

### 8.2 The counters (`bench/p87.py counters`)

- **DRAM:** reads and writes per decoded byte (64-byte lines, idle-subtracted). For each
  of A and B at each w, B − A is given with a t interval over repetitions. The counters
  are system-wide: background traffic is subtracted, not excluded.
- **Instructions:** the median B/A. At w = 1 it is the validity check of §7.3.4. At w = 4
  it is reported against 1%: lock spinning varies with timing.
- **Frequency:** cycles per reference cycle, each arm (descriptive). A difference is an
  effect of the arm, not a confounder, unless the host's state explains it.
- **Availability:** each pass is probed first. What can't be counted whole, without
  multiplexing, is recorded unavailable and never replaced by another event.
- **PEBS:** not included. The decisions don't need it.

### 8.3 Quality controls (recorded with the results)

- the CPU allocation, its governor, frequencies and siblings;
- `/proc/stat` around S1, giving the busy fraction of the idle siblings and of the other
  CPUs;
- `/proc/pressure`;
- the load average;
- a numbers-only process list;
- the thread inventory and affinity of every child.

## 9. Validity concerns, unresolved

1. **Arm A is the diagnostic path.** It is not the shipped binary (§3, §5). A B/A gain
   says what reuse of `ll` buys in this binary. A production change needs its own trial.
2. **B keeps `rle`.** It costs residency, not traffic. Whether freeing it changes anything
   further (page reuse by the allocator, RSS) is outside this diagnostic.
3. **The prefault sits inside the timed decode.** It falls on each worker's first block,
   about 1.8 MB per worker, identical in both arms. It is small against a 207 MB decode.
4. **System-wide IMC counts include the feeder's, scanner's and reader's traffic.** That
   is common to the arms, not attributable to the walk. S3 explains no mechanism by
   itself.
5. **Four cores only.** H-mech at 6–10 workers is the historical bar's question. A
   positive D here motivates, but does not substitute for, that run.
6. **One host.** The decisions are the i9-10900K's.
7. **The phase instrument is not the timed binary.** The phase instrument (`occupancy`
   with marks) and the timed binary (`p87_decoder`) share the diagnostic path, but S2's
   and S3's figures come from the instrument (§8). S2 is gated by its own perturbation
   check.

## 10. Runtime budget (i9, modelled from earlier i9 runs)

| Step | Estimate |
|---|---|
| Builds (three) and the base build for E3 | ≈ 3 min (cold), less when cached |
| Tests, audit, `--check` (3 arms × w ∈ {1, 4} + sequential), self-test | ≈ 1.5 min |
| **S1:** 21 rounds × 3 arms × (primary w1 ≈ 1.0 s + w4 ≈ 0.3 s + each control's two cells ≈ 0.1 s), plus each child's untimed SHA-256 (≈ 0.4 s a primary child) | ≈ 3–4 min |
| **S2:** 48 runs | ≈ 1 min |
| **S3:** 48 runs and 24 idle readings | ≈ 1 min |
| Analysis and archive | < 0.5 min |
| **Total** | **≈ 10–12 minutes**, about 4 of them S1 |
