# 2026-11-10: The team path's kernels against the one-worker path's, counted

**Status: counted, not timed.** Callgrind instruction counts (Ir) at master `ae44497e9`, on the
whole Wikidata bench corpus. No clock was read and nothing in `src/` changed. Every figure here
is labelled **counted** (callgrind, exact for this input and binary), **read** (from the
source or the workflows) or **model** (arithmetic on counted figures, not observed).

- **The question** (Pathfinder, #145, at `8037cd1ef` on #313): do the cooperative team's
  kernels lack the optimisations the one-worker path received? The four gaps named were MTF
  (#170, #172, #207), the build (#312), the walk (#171's staging) and the output (#164).
  Does CI's A/B ever measure those kernels?
- **The answer:**
  - **Counted:** the team arm (`mibz2-pipe3-both`, 9 threads) executes **37.91** instructions
    a decoded byte against the one-worker path's **25.79**, 12.12 more.
  - **Where the excess goes:** the walk +7.52 (62%), the build +3.94 (33%) and MTF +0.60
    (5%). The output stages differ by +0.01, and coordination and the rest by +0.09.
  - **The four gaps:**
    - **MTF, build and walk are confirmed,** in the source and in the counts.
    - **The output gap is confirmed in the source but costs no instructions.** #164's
      in-place inversion saves a buffer, not work.
  - **Model:** closing each of the three to the one-worker figure leaves the team path at
    25.85 Ir/B.
  - **CI's gate:** the CI bench never runs a team. Teams need 6 *physical* cores, and the
    default 3 × 3 pipelined plan needs 10 physical cores, a 20 MiB L3 and 10 threads or more.
    CI's runners have 4 vCPUs on 2 cores, and CI runs the default arm at 1, 2 and 4 threads.

## 1. Method

**The binary:**
- `examples/ab` at `ae44497e99765f3f1570c749ef5b381d7d8799a6`, built with
  `cargo build --release --example ab` (fat LTO, `debug = "line-tables-only"`) and rustc 1.99.1;
- its SHA-256 is `f4f035a5…5264c6a5`;
- valgrind 3.22.0 ran it on a Xeon @ 2.10 GHz. `Kernels::detect` chose AVX2 under valgrind,
  as the function names show.

**The corpus:** `wikidata.bz2`, SHA-256 `82f65ace…c2c29915` (`bench/wikidata.json`'s pin).
**All 431 blocks**, so N = 431: nothing was cut. The ledger
(`2026-11-10-bandwidth-titration.md`) also ran the whole file.

**The two arms, each a whole decode by the bench harness's own child:**

```
valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes \
    --callgrind-out-file=cg/one.%p  target/release/examples/ab child mibz2 1 wikidata.bz2
valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes \
    --callgrind-out-file=cg/team.%p target/release/examples/ab child mibz2-pipe3-both 9 wikidata.bz2
```

- **`mibz2` at one thread** is `BzReader::new`: the one-worker default (`mtf::mtf` →
  `mtf_avx2`, `bwt::build`, `bwt::room_segments`, in place over `ll`).
- **`mibz2-pipe3-both` at 9** is three pipelined teams of three with park and small, the
  plan and levers the default decoder runs on the ten-core topology (`PIPELINED_PLAN`,
  `PIPELINED_LEVERS`). Its leaders run `mtf_checkpoints_avx2`, RLE1 and the CRC. Its helpers
  run `FusedBuild::share` and `coop::traverse`, and the leader joins the walk.

**Both decodes are correct:** each wrote 209,700,196 bytes with the same hash,
`a8830caee5c511a6` (`one.json`, `team.json`).

**Stages, by function** (`bench/team_gap_count.py`, `STAGES`):
- **Self cost:** each function's own Ir, with the code inlined into it, goes to its stage.
- **Library routines:** memcpy, memmove, memset, the allocator and the sort go to the stage
  of their caller, arc by arc. Each arc's inclusive Ir comes from callgrind's call records.
  So no library cost is pooled into "other", and the memcpy callers are kept apart (§3).
- **The ab harness's own work:** its `drain` copy into the read buffer and its output hash
  are a row of their own, the same in both arms.

**Denominators:**

| | symbol | value | source |
|---|---|--:|---|
| decoded bytes, after RLE1 | D | 209,700,196 | both arms' output |
| table rows (bytes before RLE1) | N | 209,706,557 | `team_rows` (below), the sum of every block's MTF length |
| segment bytes walked, one worker | M₁ | = N | every one of the 431 blocks' permutations is a single cycle (`team_rows` counts its cycles), so the room walk visits each row once |
| segment bytes walked by `coop::traverse` | M_t | 209,697,662 | **counted:** the executions of its byte stores (§4); equal to the rows of the 233 blocks of at least `MIN_COOP` rows, exactly |

- **The rest of N:** the other 8,895 rows are 198 blocks of fewer than 64 Ki rows. The team's
  leader decodes them alone (`solo`), and both arms walk them plainly.
- **N/D = 1.00003.** RLE1 barely changes Wikidata, so every Ir/D below equals Ir a row to the
  third decimal (the report prints both). The walk's figures are also given per walked byte.

`team_rows.rs.txt` is a 40-line example run once and never committed as code. It reads the
blocks with the crate's public API, runs MTF, counts each permutation's cycles, and runs the
build, the walk and RLE1.

## 2. The stage table (counted)

Ir a decoded byte (D), all threads summed. `report.txt` holds the raw Ir.

| stage | one worker | team (9 threads) | team − one |
|---|--:|--:|--:|
| scanner | 3.256 | 3.202 | −0.053 |
| MTF | 4.111 | 4.715 | **+0.604** |
| build | 5.160 | 9.104 | **+3.944** |
| walk | 10.188 | 17.712 | **+7.524** |
| output: assembly | 0.177 | 0.174 | −0.003 |
| output: RLE1 | 0.941 | 0.950 | +0.009 |
| output: CRC | 0.485 | 0.485 | 0.000 |
| coordination (pool, relay, waits) | 0.000 | 0.072 | +0.072 |
| other | 0.003 | 0.020 | +0.016 |
| **decoder** | **24.321** | **36.433** | **+12.112** |
| harness (ab's copy and hash) | 1.470 | 1.477 | +0.007 |
| **whole process** | **25.792** | **37.910** | **+12.119** |

**Each stage's functions** (Ir/D; self cost, then library arcs):

| stage | one worker | team |
|---|---|---|
| scanner | `block::read_item` 2.583, `Huff::new` 0.351, `Huff::decode_slow` 0.139, `decode_entry_slow` 0.041; memset ← `ReadSource::view` 0.060, memcpy ← `read_item` 0.039, memset ← `Huff::new` 0.036 | the same kernels (`read_item` 2.583, …); memcpy ← `ChannelSource::view` 0.007 in place of `ReadSource`'s memset |
| MTF | `mtf_avx2` 3.687; memset ← it 0.424 | `mtf_checkpoints_avx2` 3.983; memset ← it 0.709; memmove (`__memcpy_avx_unaligned_erms`) ← it 0.022 |
| build | `bwt::build` (`build_one_pass` inlined) 5.160 | `FusedBuild::share` 8.937, `FusedBuild::new` 0.162, `bwt::build` 0.002 (the short blocks) |
| walk | `room_segments::<32, 1>` 9.994; memcpy ← it (the tile's flush) 0.123; sort of the starts 0.048; memset 0.014; reserve 0.009 | `coop::traverse::<32>` 17.527; sort of the starts 0.107; `RoomWalk::new` 0.041; reserve ← `traverse` 0.018 and ← `RoomWalk::new` 0.010; memset ← `RoomWalk::new` 0.009 |
| assembly | `SegScratch::assemble` 0.018; memcpy ← it 0.159 | `SegScratch::assemble` 0.035; memcpy ← it 0.126; reserve ← it 0.013 |
| RLE1 | `unrle_avx2` 0.940 | `unrle_avx2` 0.940; reserve ← it 0.009 |
| CRC | `crc::clmul_update` 0.485 | the same |
| coordination | (none) | helper thread loop (`Pool::named`'s closure) 0.029, `Relay::wait_walk_ready_tallied::<true, 4096>` 0.026, `Relay::walk_and_leave` 0.010, `Pool::run` 0.006 |

**What changed since the earlier counts.** These figures are recomputed. Pathfinder's archived
pre-#312 numbers are not reused, and the archived bandwidth files carry no Ir column anyway.

- **The one-worker build** is 5.16 here: #312 took it below the record's 6.2.
- **The team's build** is unchanged by #312, so the build gap is now 3.94.

## 3. memcpy, memmove and memset by caller (counted)

The arcs into libc's routines, inclusive Ir/D. `__memcpy_avx_unaligned_erms` also serves
`memmove` (`std::ptr::copy`).

| routine ← caller | stage | one worker | team |
|---|---|--:|--:|
| memcpy ← `ab::drain` (the harness's read buffer) | harness | 1.000 | 1.000 |
| memcpy ← `SegScratch::assemble` | assembly | 0.159 | 0.126 |
| memcpy ← `room_segments` (the staged tile's flush, #171) | walk | 0.123 | none |
| memcpy ← `block::read_item` | scanner | 0.039 | 0.039 |
| memmove ← `mtf_checkpoints_avx2` (the list's move, index ≥ 32) | MTF | none | 0.022 |
| memcpy ← `ChannelSource::view` | scanner | none | 0.007 |
| memset ← `mtf_avx2` / `mtf_checkpoints_avx2` (runs) | MTF | 0.424 | 0.709 |
| memset ← `ReadSource::view` | scanner | 0.060 | none |
| memset ← `Huff::new` | scanner | 0.036 | 0.036 |
| memset ← `room_segments` / `RoomWalk::new` | walk | 0.014 | 0.009 |

- **memcpy's self cost overall** is 1.332 (one) and 1.232 (team) Ir/D. Of it, the harness is
  1.000 in both. The walk's direct flush arc, 0.123, is the one-worker walk's only memcpy.
- **The team's walk has no flush.** It writes each byte straight to its segment.

## 4. Team engagement (counted)

Callgrind runs one thread at a time. The split below is therefore valgrind's schedule, not
the i9's, but it shows whether the team really shared the work.

**What is counted, per thread:**
- **build shares:** the calls to `FusedBuild::share`;
- **walk calls:** the calls to `coop::traverse`;
- **claims:** the executions of `traverse`'s `lock xadd`, the segment counter's `fetch_add`;
- **walked bytes:** the executions of `traverse`'s byte stores (`objdump -d -l` gives each
  store's line), split by kind:
  - each segment's first byte, written by `take`;
  - the full phase's bytes;
  - the tail's bytes.

| thread | role | total Ir | MTF | build | walk | RLE1 + CRC | coordination | shares | walk calls | claims | take | full phase | tail |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 1 | main (harness) | 308.9 M | | | | | | | | | | | |
| 2 | ab's sampler | 2.6 M | | | | | | | | | | | |
| 6 | scanner | 671.8 M | | | | | | | | | | | |
| 3 | leader | 546.1 M | 353.0 M | 12.3 M | 71.3 M | 107.5 M | 1.3 M | 0 | 42 | 4,668 | 4,626 | 3,888,726 | 168,619 |
| 4 | leader | 530.8 M | 306.1 M | 11.1 M | 118.4 M | 93.3 M | 1.3 M | 0 | 37 | 7,714 | 7,677 | 6,514,029 | 228,161 |
| 5 | leader | 594.8 M | 329.7 M | 11.6 M | 151.2 M | 100.1 M | 1.5 M | 0 | 49 | 9,902 | 9,853 | 8,232,887 | 353,114 |
| 7 | helper, team A | 854.6 M | | 354.8 M | 492.1 M | | 1.9 M | 78 | 78 | 31,567 | 31,489 | 26,785,845 | 931,202 |
| 10 | helper, team A | 878.5 M | | 270.8 M | 600.5 M | | 1.7 M | 78 | 78 | 38,536 | 38,458 | 32,698,869 | 1,117,964 |
| 8 | helper, team B | 859.0 M | | 329.5 M | 522.9 M | | 1.8 M | 72 | 72 | 33,530 | 33,458 | 28,436,324 | 1,003,059 |
| 9 | helper, team B | 766.2 M | | 250.7 M | 506.3 M | | 1.8 M | 72 | 72 | 32,599 | 32,527 | 27,659,163 | 884,600 |
| 11 | helper, team C | 1,026.1 M | | 379.0 M | 639.3 M | | 2.0 M | 83 | 83 | 41,131 | 41,048 | 34,880,302 | 1,186,605 |
| 12 | helper, team C | 910.5 M | | 289.3 M | 612.2 M | | 1.9 M | 83 | 83 | 39,329 | 39,246 | 33,401,916 | 1,087,895 |

**How the rows were assigned:**
- **Helpers to teams:** each team's two helpers run one build share each per block, so equal
  share counts pair them: 78 + 72 + 83 = 233, the cooperative blocks.
- **Leaders:** the leaders run MTF. Their build Ir is `FusedBuild::new`, which runs on the
  leader before the pool does. Which leader is in which team is not recoverable from the
  counts.

**The arithmetic checks:**
- **The walked bytes sum to the cooperative rows:**
  238,382 + 202,498,061 + 6,961,219 = 209,697,662.
- **The claims are the successful takes plus one exhausting claim per call:**
  238,382 + 594 = 238,976, where 594 = 42 + 37 + 49 + 466.
- **The takes match the segments:** 238,382 against the 233 × 1,024 requested; the
  difference, 210, is starts removed as duplicates.

**Engagement:**
- **Every helper walked in every block of its team:** its walk calls equal its build shares.
  On average it took 404 to 495 of a block's 1,024 segments a call.
- **Within each team the two helpers walked about equally:**

  | team | one helper | the other |
  |---|--:|--:|
  | A | 27.7 M | 33.8 M |
  | B | 29.5 M | 28.6 M |
  | C | 36.1 M | 34.5 M |

  (All of each helper's walked bytes: take, full phase and tail.)
- **The leaders joined the walk on 37 to 49 of their blocks.** Each walked 2 to 4% of all
  walked bytes.
- **No single helper dominated.** The team figure is from shared tables, not from one
  thread that walked alone.
- **What the counts cannot show:**
  - the split block by block, since the counts are totals over blocks;
  - the balance on real hardware, where the leader's join time differs.

**The tail:** 3.3% of `traverse`'s walked bytes are in its unbatched tail, and 96.6% are in
its full phase.

**Coordination's 0.07 Ir/B is a floor, not a price.** Under valgrind a waiting thread spins
only until its time slice ends, and with park it parks. On ten cores the spins would count
differently.

## 5. The four gaps (read and counted)

### MTF: confirmed, +0.60 Ir/B

**Read:** `mtf_checkpoints_avx2` (`src/mtf.rs`) keeps none of the three optimisations:

| optimisation | `mtf_avx2` (one worker) | `mtf_checkpoints_avx2` (team) |
|---|---|---|
| #207: the first 32 bytes of the list in two registers, the rest in 7 rows moved in registers | yes | the 256-byte list stays in memory: a shifted reload below index 32, and `std::ptr::copy`, a memmove call, from 32 up |
| #170: a run of at most 64 bytes as two 32-byte stores | yes | `std::ptr::write_bytes`, a memset call, for every run |
| #172: the list holds the bytes, not their indices | yes | `seq_to_unseq` looked up every symbol |

**Counted:**
- **The self cost:** 3.983 against 3.687 Ir/B, +0.296. This is mostly #172 and #207; the
  checkpoints' outer loop is in it too.
- **The memset arc:** 0.709 against 0.424, +0.285, the share #170 removes.
- **The memmove arc:** +0.022.

This port is the one already in flight (C29).

### Build: confirmed, +3.94 Ir/B

**Read:** `FusedBuild::share` (`src/coop.rs`) has two paths, and nothing of #312's:
- **its two paths:** a window of 16 equal bytes, written in one 16-entry loop, and the one-row
  step for every other window;
- **what it lacks:** the windows cut into runs (`run_joins`, `few_runs`, up to `RUNS` = 6) and
  the SSE `run16` write.

So a Wikidata window of two to six runs takes 16 one-row steps in the team and a few run
steps on one worker.

**Counted:** `share` 8.937 and `new` 0.162 Ir/B, against `build` 5.160.

### Walk: confirmed, +7.52 Ir/B

**Read:** `coop::traverse` is `room_segments`'s loop before #171:
- every lane is tested for its START flag at every step;
- every byte is stored through the lane's own base pointer, straight to its segment.

`room_segments` instead stages each offset's bytes in a 32 × 256 tile, tests the flags once an
offset, and flushes the tile with memcpy.

**Counted:** 17.527 Ir a walked byte in `traverse` itself (M_t), against `room_segments`' 9.994
and its 0.123 flush.

**Some of the gap is not staging.** The team asks for 512 × helpers = 1,024 segments a block
against one worker's 512. That doubles:
- the takes (238,382 against about 119,000);
- the starts flagged, sorted (0.107 against 0.048 Ir/B) and searched at each segment's end.

### Output: confirmed in the source, but not an instruction gap

**Read:**
- **No in-place inversion in teams:** `Cooperative` keeps `InverseBwt::invert_in_place`'s
  default, `false`, and `decode_block` doesn't even ask a cooperative kernel.
- **The pipelined team assembles into a buffer of its own:** `Team::bufs`, not `ll`.

So #164 does nothing in teams.

**Counted:** assembly, RLE1 and CRC are 1.609 Ir/B in the team against 1.603 on one worker.

**What #164 is worth:** a buffer, not work. One worker never allocates `rle` (its commit and
`docs/ledger.md`), which is a footprint and cache question that callgrind's Ir cannot price.
Porting #164 to teams would save no instructions.

## 6. What closing each gap would save (model)

**The model's assumption:** each of the team's stages is brought to the one-worker figure per
decoded byte, and nothing else changes. It is arithmetic on §2, not a forecast of time.

| closed | team Ir/B (process) | saved | share of the 12.12 excess |
|---|--:|--:|--:|
| none (counted) | 37.91 | | |
| walk | 30.39 | 7.52 | 62% |
| walk, build | 26.45 | 3.94 | 33% |
| walk, build, MTF | 25.85 | 0.60 | 5% |
| … and output (#164) | 25.85 | 0.00 | 0% |
| one worker (counted) | 25.79 | | |

**What the model leaves out:**
- **The walk's figure is an upper bound on the saving.** A ported `traverse` keeps the team's
  1,024 segments: twice the takes, the sort (+0.06) and the end searches.
- **The ports themselves:** the Optimiser's price for a staged team traverse (inbox
  6758227354) is a hypothesis of 10 to 11 inclusive Ir a row, not a count.
- **The build's shares** keep their per-share setup (`FusedBuild::new`, 0.16).

**Instructions are not time.** Pathfinder's own note stands: at ten threads on the i9 the walk
waits on memory, and what an instruction is worth there is not known.

**#171's evidence is not a team result.** Its four-thread EPYC 9V45 CPU reduction was measured
on plain workers: `Plan::for_threads` gives `Workers` below six physical cores. It shows what
staging does under contention, not what it does in a shared-table team.

## 7. Whether CI measures the team kernels (read)

### `Plan::for_threads` (`src/parallel.rs`, `plan_with`)

| condition | plan |
|---|---|
| the topology unreadable (anything but Linux), or `min(threads, physical cores) < COOP_MIN_CORES` = 6 | `Workers(threads)`: no team |
| 10 physical cores, a 20 MiB L3 and threads ≥ 10 | three teams of three, pipelined with park and small (`PIPELINED_PLAN`, the arm counted here) |
| otherwise | teams of two, `min(cores / 2, round(0.7 × L3 / 3.6 MB))` of them, not pipelined |

**Teams engage at six *physical* cores, not six threads or vCPUs.** The threads must also be
at least six.

### What CI's bench runs

**The decoder A/B (`.github/workflows/bench.yml`):**
- it runs on `ubuntu-latest`, 4 vCPUs on 2 cores, as the workflow and `COOP_MIN_CORES`'s
  comment both say;
- its arms are `mibz2` and `mibz2@base/ab`, at threads `1,2,$(nproc)` = 1, 2 and 4;
- `mibz2` at 1 is `BzReader::new`, and at 2 and 4 it is `BzReader::parallel`, whose plan
  there is `Workers` (min(threads, 2) < 6).

**The stage table (`bench/stages.py` over `examples/phases.rs`)** is one thread:
`mtf::mtf`, `bwt::build`, `walk_segments_room_parts`.

**The other workflows:**
- `target-gated-ni.yml` runs the same arms at 1, 2 and 4;
- `sweep.yml` and `pmu.yml` time no team arm;
- the team and pipelined arms appear only in the workstation scripts
  (`bench/pipeline_levers.py`, `bench/coop_decoder.py`).

### The claim

**Confirmed.** On no PR, at no thread count, does CI's gate execute these:
- `mtf_checkpoints_avx2`;
- `FusedBuild::share`;
- `coop::traverse`;
- the relay.

**A change to any of them reads as "no change" on CI's A/B.** Only correctness covers them
there: the unit tests in `src/coop.rs` and `tests/`, which force plans with `with_plan`.

**The ways to read them:**
- a count like this one;
- a forced team arm on a 4-vCPU runner, as C29's 9-thread sweep was;
- the workstation.

## 8. Reproducing

**The script and the archived counts:** `bench/team_gap_count.py` reads the profiles archived
here (`2026-11-10-team-kernel-gap-count/cg/*.gz`, one file a thread). `report.txt` is its output:

```
python3 bench/team_gap_count.py report \
    docs/journal/2026-11-10-team-kernel-gap-count/cg/one \
    docs/journal/2026-11-10-team-kernel-gap-count/cg/team \
    --binary target/release/examples/ab --rows 209706557 --decoded 209700196
```

**`--binary` must be the profiled build,** at `ae44497e9` with rustc 1.99.1. Its `objdump`
locates `traverse`'s `lock xadd` and byte stores, so it is needed only for §4's claims and
walked bytes. The stage table needs only the profiles.

**The machine time:** both decodes ran under valgrind in about a minute on the shared machine.
That time was not read as a measurement.
