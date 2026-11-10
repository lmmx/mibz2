# MONODROMY orientation

Snapshot: `lmmx/mibz2` master `0422f6b7c` (10 Nov 2026). Source line numbers were read at `39ed2ca27` and `0422f6b7c`. The commits between them touch team and routing files only, so the `src/` references are believed unchanged. That belief was not re-verified line by line. "PF:" marks a path on branch `origin/claude/agent-swarm-fable-breakthrough-0u7j4d`: the Pathfinder's #313 work, which is not on master. `branch:path` marks other branches.

Evidence labels used throughout:
- **M**: measured (machine and source given)
- **C**: counted (callgrind Ir or a census, exact for its input)
- **P**: proved, or argued in code and checked by tests or Miri
- **Mod**: model
- **Conj**: conjecture or assertion

## 1. Purpose of this file

This file gives MONODROMY a map of where the decoder and the team's research stand. OWNERSHIP.md §4a describes MONODROMY as the exploratory lane for new mathematics of inverse BWT. That includes asking whether LF, a materialised permutation, the current traversal order or the serial dependencies are needed at all. The map covers:
- the problem as the decoder actually meets it;
- the constructions in production and their source locations;
- what has been measured, counted, modelled or proved;
- what has been tried and how each attempt ended;
- the bounds the team has stated, with how strongly each was established;
- the machinery for testing, measuring and collaborating.

Nothing here is a conclusion MONODROMY must accept. Closures, "refuted" labels, bounds and open questions are the team's records, each at its stated strength. Several were written for one implementation, one machine or one premise. Some have since been withdrawn. The team's own policy says a failed gate is a verdict on the candidate, not on the line (`docs/journal/2026-11-04-compact30-research-charter.md:532`; `docs/journal/decisions.md`, 10 Nov: "disprove the underlying advantage, not merely the first implementation"). Where this file is uncertain or the readers disagreed, it says so.

## 2. The problem as the decoder faces it

**Block facts** (`src/block.rs`):
- A block begins with magic `0x314159265359`, and a stream ends with `0x177245385090` plus a combined CRC (block.rs:10-11).
- The header holds the block CRC (32 bits), a randomised bit and `orig_ptr` (24 bits).
- `max_len = 100_000 × level`, level 1-9, so n ≤ 900,000 rows. The header check is `orig_ptr ≤ 10+max_len` (block.rs:234-238). `decode_block` then requires `orig_ptr < n` (lib.rs:337).
- Randomised blocks are refused (`ErrorKind::Randomised`, block.rs:229-233), so the decoder never inverts a randomised permutation.

**Before inversion:**
- Huffman decoding uses ≤6 tables, an alphabet of ≤258 symbols, ≤18,002 selectors, codes of ≤20 bits and groups of 50.
- `MAX_SYMS` = 900,100 u16 symbols (1.8 MB).
- RLE2 (RUNA/RUNB) runs into MTF, with run weight below 2^21 (`MAX_RUN_WEIGHT`, mtf.rs:30).
- MTF produces the last column L (`ll`, 1 B per row) and `counts[256]`.

**Inversion inputs and outputs:**
- Input: L, its counts and `orig_ptr`.
- Output: the n bytes of the cycle of the standard permutation through `orig_ptr`, in text order. The output is still RLE1-coded.

**After inversion:**
- RLE1 decoding (4 equal bytes, then a count of 0-255; a missing count byte is `RunUnfinished`, rle.rs:1).
- CRC-32 with polynomial 0x04C11DB7, MSB-first (crc.rs:7), checked per block and combined per stream (lib.rs:511).
- A block whose compressed form exceeds 64 MiB is an error (lib.rs:77).

**The permutation:**
- Ψ maps an F-position (slot s) to the row i whose byte lands at s; it is the inverse of LF. The plain inversion is n dependent loads.
- The permutation may have several cycles, for example when the text is a repeated string. Only r0's cycle is emitted.
- The correctness bar is byte-identical output on every bench corpus and the same error on corrupt input as before the change (PERFORMANCE-POLICY.md §1). The walks emit n bytes from r0, going round r0's cycle again if it is shorter than n (bwt.rs:625-635). A wrong cycle structure is not detected by the inversion itself; it surfaces through the block CRC.

**Sizes:**
- The production table is 4 B per row: 3.6 MB at n = 900k (`FULL_TABLE`, parallel.rs:1173).
- 198 of the Wikidata sample's 431 blocks are below 262,144 rows (doc comment on the pipelined team).

## 3. Current architecture

### 3.1 Pipeline

1. **Scanner and Huffman decode (serial).**
   - `stream::Scanner` (stream.rs:55) has a 2 MiB lookahead and `MAX_ITEM` = 64 MiB. It calls `block::read_item` (block.rs:211).
   - `Huff` (block.rs:50) uses a 10-bit LUT (`LUT_BITS`) of u32 entries holding up to 2 symbols each. Longer codes fall back to limit/base/perm. There are 4 lookups per 56-bit refill (`STEPS`), and `bits::Reg` stays in registers (bits.rs:17).
   - Output: `Symbols {crc, orig_ptr, seq_to_unseq, syms, eob, max_len}` (block.rs:23).
   - The code comments describe this stage as serial: in the current design, a block's end is known only once it has been decoded (lib.rs:41-42, parallel.rs:5-7). Whether that dependency is necessary has not been examined in the record.
2. **RLE2 and MTF.**
   - Entry point: `mtf::mtf` (mtf.rs:12). The scalar version is `mtf_scalar` (mtf.rs:32).
   - `mtf_avx2` uses libbzip2's two-level list. The first 32 bytes live in two XMM registers and move with the `Step` shuffle table (32×64 B, mtf.rs:87-132). Bytes 32-255 sit in 7 aligned 32-byte rows. The list holds bytes, so no `seq_to_unseq` lookup is needed.
   - `mtf_checkpoints` (mtf.rs:298) records the prefix counts of `ll[..i·CHECKPOINT]`, with `CHECKPOINT` = 16,384 rows (mtf.rs:292). Each entry is a [u32;256], about 56 per full block. Only the cooperative kernels request them.
3. **Inverse BWT** (§3.2).
4. **RLE1 and CRC.**
   - `rle::unrle` (rle.rs:9): AVX2 tests 32 positions for runs of 4.
   - `crc::crc32` (crc.rs:42): PCLMULQDQ folding for inputs of at least 128 B, slice-by-8 below that.
   - `Kernels::detect` (lib.rs:134) selects kernels once at run time.
5. **Parallel layer.**
   - `StreamDecoder` (parallel.rs:101): one scanner thread sends `Job=(place, bit, Symbols)` to workers running `decode_block` (lib.rs:326). Results are reordered through a BTreeMap, errors are delivered in order, and panics become `Internal`.
   - `INPUT_QUEUE`=2 and `SLOTS_PER_THREAD`=4. Each worker keeps about 6-7 MB of buffers (parallel.rs:16 says ~7 MB, the `Scratch` doc at lib.rs:276 says about 6 MB).
   - `decode_parallel` (lib.rs:384) feeds 4 MiB chunks. `BzReader` (read.rs:33) is `Seq` or `Par`.
   - The parallelism is across blocks.
6. **Pipelined teams.**
   - Code: `pipelined_worker` (parallel.rs:963) and `coop::invert_pipelined_with` (coop.rs:1517).
   - While helpers 1..k build and walk block i, the leader does RLE1 and CRC of block i−1 and MTF of block i+1 into a second `Front`, then joins the walk.
   - A team holds 2 fronts, 2 assembled buffers, 1 table and a `Solo` kernel for blocks below `MIN_COOP`.
   - `Levers`: `park`, `small`, `spin2k`.

**Plan selection** (parallel.rs:1162-1287). `plan_with` reads the physical cores in the process's affinity and the L3 size from Linux sysfs.

| Condition | Plan and kernel |
|---|---|
| `decode`, `BzReader::new`, `decode_with` | 1 thread, `Segmented` |
| Non-Linux, or min(threads, cores) < `COOP_MIN_CORES`=6 | `Workers(threads)` × `Segmented`. This includes every CI runner. |
| Exactly 10 physical cores, L3 = 20 MiB, threads ≥ 10 | `PIPELINED_PLAN = Teams{3,3}` (parallel.rs:1186), pipelined with `PIPELINED_LEVERS` (park+small) |
| Otherwise, ≥6 cores | `Teams{max(1, min(cores/2, cap)), 2}` with `Cooperative<2>`, where cap = max(1, round(0.7·L3/3.6e6)) if the L3 size is known and 4 if it is not (parallel.rs:1278-1284) |

Other plan details:
- `planned` maps k ∈ {2,3,4} to `Cooperative<k>`, and any other k to that many `Segmented` workers (parallel.rs:156-164).
- The window is scaled per team: `slots·teams/threads` (parallel.rs:1258).
- The "service core" (two service threads doing MTF and the finish while two teams of three build and walk) exists only on `research/service-core` and `research/service-lookahead`. `grep -i service src/` finds nothing on master.

### 3.2 Inversion constructions on master

- **Trait.** `trait InverseBwt` (bwt.rs:23).
  - Implementations: `Plain` (bwt.rs:78), `Segmented` (bwt.rs:97, the default), `Cooperative<K>` (bwt.rs:153).
  - `invert_in_place` (bwt.rs:70, :114) writes the output over `ll`. It saves a buffer, not work (`docs/journal/2026-11-10-team-kernel-gap-count.md`).
- **Table layout.** `TVector(Vec<u32>)` (bwt.rs:257), with `T[s] = L[s] | Ψ(s)<<8` (code: `ll[s] | i<<8`, bwt.rs:288-300).
  - Bits 0-7 hold the byte and bits 8-31 the link, so `n < 2^24` is required.
  - Starts are flagged in bit 31 (`START`/`LINK`, bwt.rs:710), which is link bit 23. The room walk therefore asserts `n ≤ 2^23`.
- **Build.** `build` (bwt.rs:276) tries `build_one_pass` (bwt.rs:288) and falls back to `build_two_pass` (bwt.rs:497) when the counts are not `ll`'s own.
  - One-pass is a counting-sort scatter with cursors `ftab[c]` and limits `lim[c]`, over 16-row windows.
  - `run_joins` (bwt.rs:436) compares each byte with its predecessor using SSE2, giving a 15-bit mask.
  - An `ALL_JOINED` window takes one `run16` (four SSE2 stores of `src[j] | base + j<<8`).
  - A window with ≤`RUNS`=6 runs (`few_runs`, counted by the POP8 table, bwt.rs:417) takes one `run16` per run. Its overwritten tail slots are rewritten later.
  - Other windows take one-row steps. `row_byte` (bwt.rs:402) is a volatile read that prevents a stack spill.
  - Soundness: every write is checked against its bucket end, the final check is `ftab == lim`, and `set_len` runs only after that check (argument at bwt.rs:381-386).
- **Walks.** Each one is exact to `walk` on any table, cycles included (proof sketch at bwt.rs:620-635).
  - `walk` (bwt.rs:549): `p = T[p>>8]; out.push(p as u8)` from `T[orig_ptr]`. This is a chain of n dependent loads (bwt.rs:1-6).
  - `walk_segments` (bwt.rs:636): 12 lanes and 256 segments, stopped by a bitmap.
  - `walk_segments_flagged` (bwt.rs:729, :752): starts are flagged in their own entry.
  - **Production:** `walk_segments_room` (bwt.rs:962) → `room_segments<L,PF>` (bwt.rs:1026).
    - `FLAGGED_LANES`=32, `FLAGGED_SEGMENTS`=512, `PREFETCH`=`PF_L1` (bwt.rs:569-573).
    - Starts are 511 xorshift-seeded rows (`random_rows`, bwt.rs:1420) plus r0, sorted and deduplicated.
    - Passes run `room` rounds, where `room` is the fewest spare bytes over all lanes, so there is no per-step end test (after libsais's `decode_chains`).
    - The busy phase is staged in TILE=256-offset chunks (bwt.rs:1341; 8 KiB at 32 lanes).
    - Segment buffers have capacity `2n/ns+64`. `next[seg]` is found by binary search over the starts.
    - `SegScratch::assemble` (bwt.rs:600) joins the segments from r0's.
    - Contract: Rows, Ownership, Spare, Handoffs, Initialised, Lifetimes (bwt.rs:991-1005), checked against `tests::segment_oracle`.
    - Blocks below `MIN_SEGMENTED` = 65,536 rows use `walk`.
- **Cooperative** (`src/coop.rs`).
  - `Pool` (coop.rs:265-276): K−1 persistent helpers. They spin `SPINS`=2000 then park, publish epochs with Release/Acquire, and catch and resume panics (doc at coop.rs:1-28).
  - `build_fused` (coop.rs:488; `FusedBuild` around :540-735):
    - Rows are cut at `edge(n,k,stride,j)` (coop.rs:454), at checkpoint multiples.
    - Share j's window in bucket b is `C(b)+checkpoints[start/stride][b]`, so the scatter is disjoint, with no count, copy or barrier before it.
    - `check_inputs`: the checkpoints rise, stay within the counts and sum to their offsets.
    - The end check proves the table equals `build`'s word for word (coop.rs:470-487).
    - Shares are claimed by `claimed`/`passed` AtomicU64 masks (`MAX_SHARES` 64).
  - `walk_room` (coop.rs:887; `RoomWalk` around :900-1000):
    - K threads × 32 lanes take segments from one `fetch_add` counter over `512·K` segments.
    - The table is read-only during the walk, and every buffer and successor slot has one writer.
    - The inner loop is `traverse` (coop.rs:1009). It is unstaged (no TILE) on master; a staged version is on `research/staged-coop-traverse`, and its D083 screen is pending.
    - Contract at coop.rs:874-886.
  - `Cooperative<K>::invert_checkpointed` (bwt.rs:193) runs `build_fused`, then `walk_room::<32>`, then `assemble`.
  - Fallbacks: blocks below `MIN_COOP` = 262,144 rows (bwt.rs:163), a pool that fails to spawn, and a refused fused build (falls back to `build`).
  - Pipelined: `Relay`/`Stage{Build,Walk,Over}` (coop.rs:1204-1245). The last builder runs `finish` and `RoomWalk::new`, then sets `walk_ready`. The last walker out assembles.
- **Research only, not called by the decoder:** `src/run_fed.rs` (header at :1-13).
  - Line A fills Ψ from run records (c,p,len,k) in one monotone sweep over ≤2r−1 cells.
  - Line B is a census of run-compressed Ψ².

## 4. What binds today

**The compute-frontier journal's reading: latency rather than bandwidth** (`docs/journal/2026-10-27-compute-frontier.md` §2; the measurements below are M, and their interpretation is partly modelled).
- Plain walk 31.4 c/B against 14.1 with 12 chains.
- The `bz2k walkm` sweep on one block: 28.5 / 17.6 / 10.9 / 6.8 / 4.4 c/B at K = 1 / 2 / 4 / 8 / 16 chains.
- Interleaving across blocks gave 1.03-1.27×.
- The journal's reading is a 3.6 MB table at about 50 cycles a load. The table-size and per-load figures are partly modelled.
- The bz2 decode is 53.5% of wikidata-pq's split CPU on CI and 51% on the measuring box's real split (compute-frontier.md:176). lbzip2 scales 3.82× on 4 threads.

**Measured on the i9-10900K** (`docs/journal/2026-10-28-workstation-scaling.md`):
- From 4 to 10 workers, the demand L3 hit share falls 0.76→0.38 (§8).
- Occupancy per L3-missing read rises 396→858 cycles (81→175 ns), uncore queueing included (§8).
- Memory-controller bytes written per output byte rise from 0.3 at 1 worker to 11.5-12.8 at 6-10 workers (§7, workstation-scaling.md:148-157). The journal places this under "A mechanism it exposes (hypothesis, testable)".
- An effective L3 of about 6-7 MB for the walk is a fitted model (Mod; §7, workstation-scaling.md:148-157).

**Measured on CI:**
- Pointer-chase effective cache on the 9V74 is 12.8-16.8 MB (`2026-10-28-early-warning.md:223`).

**Simulated and derived (not hardware counters):**
- Analytic L2 miss rate ≈ 1 − C/T = 0.854 against 0.849-0.862 from a cache simulator on production-identical traces (Wikidata). The simulator agrees with callgrind D1/LL within 0.5%. These are simulated, not hardware-measured (`2026-11-05-partitioned-walk/results.json`, sanity_check).
- Walk lines in flight per thread (Little's law) fall from 9.3-9.8 alone to about 5.5 at four copies (`2026-11-05-binding-census.md:71-72`; derived from measured latency and occupancy).

**Literature baseline:** 83-98 ns per LF query as one dependent chain (`2026-10-27-mlp-scheduling-and-run-structures.md:189`).

**Binding census** (`docs/journal/2026-11-05-binding-census.md`):
- M: at 4 threads the decoder is CPU-capacity-bound, at 0.91-0.97 utilisation.
- Mod: wall ≈ Σ over stages of (alone cost × co-run inflation).
- M: the walk inflates ×2.12-2.22 on AMD (7763, 9V74), against ×1.2-1.4 for the compute passes (binding-census.md:50, :62).
- Rank 1, the walk's co-run excess, is worth up to about 16-18% of 4-thread wall. Its mechanism is **contested**: core-side versus lines in flight.
- Recorded by the binding census as closed at four copies on CI (as implemented there): lanes 16-64, prefetch hints, tile 512/1024, the co-resident 16-lane shape, AMAC. Recorded as closed for build cost or for being slower on CI: compact ψ tables, Precopy, pair walk. Also recorded as closed: teams below 6 cores (0.71× the workers at 4 threads) (binding-census.md:103-107).

**Counted** (callgrind):
- Production walk 10.4-10.5 Ir per row (callgrind, `2026-11-05-partitioned-walk/results.json`). Build about 5.1-5.2 Ir/row (one-worker 5.160 Ir/B in `2026-11-10-team-kernel-gap-count.md`; 5.06 in PF chunked-link).
- Segment cost model (callgrind block 3): walk = 10.9 Ir/B + 108 Ir per segment (walk 37.3, join 70.6). Master at 512 segments: 10.74 Ir/B.
- Team gap (`2026-11-10-team-kernel-gap-count.md`, at `ae44497e9`): team pipe3-both 37.91 Ir/B against one-worker 25.79. Walk +7.52, build +3.94, MTF +0.60.
  - Closing all three gaps would give 25.85 Ir/B. That figure is Mod.

**Team scaling** (`2026-11-07-knee-mechanism.md`):
- M: throughput peaks at 3 teams, 1,382-1,399 MB/s.
- M: 28.8-32.6% of each block's time is outside the inversion call. At 4×2: 1.394 ms per block, 0.993 ms of it inside.
- M: RSS rises about 15 MB per team, and packed placement is 23% slower than spread at k=2.
- The journal labels its "why" **inferred, not established**.

**Cooperative and pipelined results** (all M, i9):
- `2026-10-29-cooperative-decoder.md`: policy vs 4 workers 1.257 (95% lower bound 1.197), CPU 1.214×, RSS 1.015×. At equal threads: 1.80× at 8 and 2.62× at 20. On 4 cores: 0.997.
- Levers (`2026-11-07-pipeline-levers-result.md`): both at 3×3@10 give 1.038 [1.021, 1.055]. `small` alone 1.043 [1.020, 1.066]. `park` alone is null on speed and saves CPU (1.055).
- Service core (result journals on master; code on `research/service-core*`):
  - `2026-11-09-service-2x3-result.md`: speed 1.045 [1.017, 1.073], CPU saving 1.146 [1.123, 1.170] (default CPU / svc2 CPU, so 12.7% less CPU).
  - Replication `2026-11-09-service-lookahead-result.md`: 1.073 [1.041, 1.106].
  - The causes of "prep starvation" are open hypotheses.

**Two machines decide differently** (PF:`2026-11-10-hypothesis-register.md` §2):
- CI's 4-vCPU runners price instructions.
- On the i9, one-thread wall did not move with instruction gains across three releases (0.991, 0.985, 1.011). The register attributes the binding there to live bytes and dependent loads.
- **CI never runs a team path.**

**Asserted, uncited:** the comment that the segmented walk takes "about half" the plain walk's time (bwt.rs:6).

## 5. The research record

### 5.1 Constructions tried

| Construction | Idea | Status as recorded (verdict on the implementation and gate named, not on the idea) | Key number | Record |
|---|---|---|---|---|
| Compact T-vector / 20-bit ψ | Link-only ψ, byte from F through an n/32 window table. Arms `c` and `b` | Refuted as built | M (VM): c4/t4 build 0.15×, walk 0.74×. Knee untested | `research/compact-tvector:docs/journal/2026-10-28-compact-tvector.md` |
| 24-bit link `u` (#112) | 2.7 MB table plus F window | Refuted for the decoder; the journal says "the science holds" | M (i9): knee 4→6 blocks; u1/o1 0.681 against gate 0.85; F recovery 16-28% of the walk | `docs/journal/2026-10-30-compact-psi-feasibility.md` §1-3; `research/compact-psi:…/2026-10-29-compact-psi.md` |
| Hybrid ψ records RUN/FOR/EF; FOR-14 (#130 `h`, `e`) | 64-B line per 32 rows, about 16 bits per row | Stopped by rule 1, which was conditional on the one-block premise | C: groups RUN 77.4%, FOR 20.7%, RAW 1.91%; 1.868 MB; step 38 instructions vs 16. M (i9): h1/o1 0.475; removes 69% of the 4-block penalty | `research/for14-records:…/2026-11-01-for14-stage1.md` |
| compact30 | 30 entries per 64-B record plus a RAW area, about 2.13 B/row | Rule failed (#195). Never run with 6-10 plain workers on the i9 | M (i9): CPU per row 0.840× at 4 workers, +14.85% on 1 core. M (CI): +11-14% wall at 1t, +5-8% at 4t | `docs/journal/2026-11-05-compact30-staged.md:106`; blog `2026-11-02-…`, `2026-11-04-compact30-constructor.md` |
| ψ² pair walk with run lanes (#6, #34) | libsais 2-byte walk | This implementation was slower on CI at 1t and 4t; the record calls it "an i9 question at 8 or more workers" | M (CI): +40.6% at 1t, +24.3% at 4t. Build 7.1-7.6 vs 2.5 c/B. M (i9): 1.107-1.117 at 8-10 workers, 0.816 at 4 (nogo.md:193, citing expert-review.md:262-291) | `docs/journal/2026-10-27-pair-walk.md`; `2026-11-05-binding-census/nogo.md:190-198` |
| Run-fed T-vector (A) and run-compressed Ψ² (B) | Fill from run records; two-hop records | Both stopped before timing | C: r₂/r = 1.22. Mod: 0.96 B/row; walk about 3× its gate | `docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md`; `src/run_fed.rs` |
| Tunnels (Baier) | Whole-tunnel table; tall tunnels copied | Construction closed; advantage open | C: 94.5% of rows in tunnels; table 0.97-1.25 MB at about 19-20 Ir/row; tall cells store 1.24-1.73 words per word delivered; pool 0.083-0.088 B/row | PF:`docs/journal/2026-11-10-tunnels.md` §0, §0a |
| Chunked link | Low byte per row, upper bits once per stretch | Held; kernel incomplete | C: 1.31 MB (Wikidata), 1.44 MB (text); 22.2 vs 8.2 Ir/row. M (local, indicative): 1.30× (Wikidata) to 1.50× (text) the T-vector's walk time at 32 lanes (slower); 0.98-1.02 at one lane. Fill 1.45-1.93× build | PF:`…/2026-11-10-chunked-link.md` §0 |
| D4-delta (k-step) | Walk on Ψ^k cells | Fails the count on construction (F2) | C: build 14.45 Ir/row vs filter 7.35; walk 7.73 Ir/B; table ≤857 KB; inversion 1.44× production (Wikidata), 3.66× (text) | `research/d4-delta-count:…/2026-11-10-d4-delta-count-result.md` |
| D4 pair-cursor | Rescue of D4 attribution | Pending an untimed census | Priced in Optimiser review 6759188202 | `team-state/optimizer:state/records/reviews/inbox-6759188202/report.md` |
| Tiled-B / deferred flags (#168) | Defer busy-phase flags | No-go; #181 won by removing them | No gain | `nogo.md:248`; `research/tiled-b` |
| Design E | 64-B line per 32 rows, backward table | Falsified as built | M (CI, EPYC 9V74): 3.67× [3.59, 3.74]. Step 22-26 instructions, about 48 per row with bookkeeping | `research/design-e-backward-table:…/2026-11-10-design-e-result.md` |
| Backward table / run-record front (#324) | Walk LF; fill `P[i]=L[i]\|LF(i)<<8` from run records in arrival order, since LF(p+j)=C(c)+k+j within a run, so no sort is needed | Closed under its frozen rule (#328; no-harm failed). Advantage not disproved | M (CI, 3 runs × 10 rounds, run 42747668401): at 1t Wikidata −12.8/−14.5/−15.8%, text −23.3/−24.5/−26.2%; random +3.4/+5.3%, records +1.5/+2.1% (confirmed harms); RSS +4-10% on Wikidata and text at 2-4 threads (about 3 MB) (run-front-production.md:504). M (i9, `2cc082961`): 8-10% faster at 1-4t, none on teams | `docs/journal/2026-11-10-run-front-production.md`; code at `perf/run-front-records` `e88c0631`, `research/run-front-fallback` `e8a417c69`; designs A-D in PF:`…/2026-11-10-backward-table.md` |
| STEPS diagnostic | Does the 4 KiB placement of `mtf::STEPS` against `counts` cost time? | Draft rev 4, not run | C: 4 KiB D←H candidates 13,829→56,154 per block. The price needs 11-67× a frame pair's bound. Frame class is a per-launch lottery (20/60 vs 25/60) | `research/run-front-fallback:…/2026-11-10-steps-placement-diagnostic-DRAFT.md`; PF:`…/2026-11-10-gate-support.md` §2 |
| Cooperative inversion | k cores share one block's table | Shipped (teams 0.2.0; 3×3 since 0.2.5) | M (VM pilot): kernel costs 10-14% on 1 core. M (i9): 4×2 1.230×, 3×3 1.323× p4, 20/20 rounds; inversion-only pinned 4×2 1.439× (commit `f6ab1f2ef`) | `research/coop-fused:…/2026-10-28-cooperative-inversion.md` §4, §9, §17.4 (cited in code, absent from master); `docs/journal/2026-10-29-cooperative-decoder.md` |
| Law of the knee | t = g + a·X(L), L = live table bytes | Holds in the inversion harness; fails on the whole decoder | M: predicts u within 0.87-1.12. Whole decoder measured/predicted 0.58 at 10. −2 grows ×1.18 vs −9 ×3.26 | PF:`…/2026-11-10-hypothesis-register.md` §1, §1a; `2026-11-07-knee-mechanism.md`; `2026-11-10-share-nothing-pricing.md:431` |
| Share-nothing | Shrink each worker's live set instead of cooperating | Recorded as a team objective (`docs/journal/decisions.md:305-307`; Louis's words quoted at :286-292); premise test pending | Mod: break-even g = 1.31 in a scanner-fed proxy | PF:`…/2026-11-10-share-nothing.md` |
| Team ports #335/#337/#339 | Port one-worker kernels to the team path | Dropped 10 Nov (LOG 6764355047) | C: 37.91 vs 25.79 Ir/B. M (i9): no cell beyond noise | `2026-11-10-team-kernel-gap-count.md`; `research/team-ports-combined:…/2026-11-10-team-ports-combined-registration.md` |
| Idle gap / two modes (#318) | i9 slow mode at 10 threads | Measured | M: slow after a 0.25 s gap 26/30, with no gap 0/30 (0.303 vs 0.136 s). Cold start: only a held spinner is fast (30/30) | PF:`…/2026-11-10-two-modes.md` ("not the gap" withdrawn); `docs/journal/2026-11-10-cold-start.md` |
| Interval exchange | LF as an exchange of r intervals; first-return induction towers; decode as copying | Open; counted, not timed | C: dependent steps 0.190n on Wikidata (2.65 per run), 0.811n on text, 0.0024n on Thue-Morse; copies 0.68-1.58n bytes | PF:`docs/journal/2026-11-10-interval-exchange.md`; PF:`examples/iet_census.rs` |

**How the recorded candidates ended:** the closing records cite the following as the deciding costs. (List only; no generalisation is drawn from them.)
- `u`: F recovery 16-28% of the walk.
- FOR-14: 38 instructions per step vs 16.
- E: 22-26 instructions per step.
- D4: build 14.45 vs 7.35 Ir/row.
- ψ²: build about 3× production's.

### 5.2 Bounds and impossibility claims, by strength

- **P, short arguments:**
  - The inherent work is n bytes stored. n *dependent* loads are not inherent: Ψ^k cells with A_2k = A_k ∪ Ψ^(−k)A_k, built by a merge (PF:hypothesis-register §0).
  - LF is a translation on each run of L (backward-table §1; interval-exchange §2).
  - At most 255 bucket boundaries fall in a 32-row window, and 0.15% of rows step past one (compact-psi-feasibility §2; the 0.15% is M or C on its corpus).
- **Reference point (C; not a proved bound):** interval-exchange §3 calls r steps, one per run, a trivial floor, under the assumption that each run costs at least one dependent step. Against it, the row walk takes 14× as many steps on Wikidata and the IET tower 2.7×.
- **C plus witness:** tall-tunnel stored-word construction costs 1.24-1.73 bytes made per byte delivered. This closes the construction, not the advantage; an earlier over-claim was withdrawn (tunnels §0a).
- **Mod, i9 only:**
  - The compact-psi-feasibility model (§5) predicted at most about 1.25× of inversion and 1.1-1.15× of the whole decoder against four teams, for compact tables of the kinds it considered.
  - It predates the three-team and pipelined plans and assumes a 60-65% inversion share.
- **Proved in code (P, with tests and Miri):**
  - one-pass ≡ two-pass build;
  - fused build ≡ `build`;
  - every walk is exact to `walk` on any table;
  - the #324 fill's tiling implies a valid table (branch only).
- **Withdrawn:** every law-based throughput price (compact30 2,350 MB/s; D4 3,570; chunked 3,120/2,500; N1 3.5-5.3 GB/s), because the law fails on the whole decoder (register §1a).
- **Conditions the closing records named as reasons to revisit (the team's notes; MONODROMY may take up any line regardless):**
  - Design E: a step plus skeleton counted at ≤1.44× the room walk's instructions, about 15 per row.
  - FOR-14: rule 1 was conditional on the one-block premise.
  - Pair walk: past 8 workers on the i9.

**Other sources:**
- Literature: `docs/journal/2026-10-29-literature-study.md` (with `/04-rlbwt-tricks`, `/05-partitioned-elias-fano`, `/09-cacheline-ef`) and `2026-10-27-bwt-inversion-literature.md`.
- Blog: `docs/site/blog/2026-11-08-inverse-bwt-navigation.md`, `2026-11-08-bwt-runs-from-mtf-codes.md`, `2026-11-07-why-mibz2-stops-scaling.md`.
- The hypothesis register: PF:`docs/journal/2026-11-10-hypothesis-register/` (sections a-r; 420 rows: 156 open, 76 rejected, 31 shipped). Its rows are pointers, not re-checked findings.

## 6. Open questions as currently recorded

These are reproduced as recorded, attributed to their sources. Their listing is not an endorsement.

- **Mechanism of the walk's co-run excess:** "core-side versus lines in flight", contested (binding-census).
- **Why throughput peaks at 3 teams:** labelled "inferred, not established" (knee-mechanism).
- **Causes of "prep starvation"** under the service core: open hypotheses (service journals).
- **Tunnels:** "construction closed; advantage open" (tunnels §0a).
- **Interval exchange** (PF interval-exchange):
  - The record estimates that an IET kernel would beat the row walk if each piece-step cost less than about 5 row-steps. This is untested.
  - Class-induced IET is untested.
- **#324:**
  - the advantage was "not disproved" (run-front-production);
  - a successor with `Runs` boxed is untested;
  - admission by bound rather than length is an open concern (short high-entropy blocks +10-11% Ir).
- **Share-nothing:** premise test pending (`decisions.md:305-307`).
- **Also flagged untested in the record:**
  - u16 byte+flag descriptor;
  - the `e` arm (byte inside the link's own line);
  - the complete chunked-link kernel;
  - N1 at k = 4;
  - compact30 with 6-10 plain workers on the i9;
  - tall-tunnel copying from output, uncounted in decode order;
  - the staged cooperative traverse (D083 pending);
  - the D4 pair-cursor census;
  - the STEPS diagnostic (draft).
- **Pair walk:** "an i9 question at 8 or more workers" (nogo.md:197). The entry records an i9 result of 1.107-1.117 at 8-10 workers and 0.816 at 4 (nogo.md:193, citing expert-review.md:262-291).

## 7. How to work here

### 7.1 Measurement paths

**CI paired bench** (`.github/workflows/bench.yml`).
- Triggers: PRs touching `src/**`, `Cargo.*`, `examples/{ab,windows,phases}.rs`, `tests/common/**`, `bench/**` or the workflow.
- Method: head vs merge-base as two arms of `examples/ab.rs` (`ab run`), with the order alternating each round.
- Settings: ROUNDS=10, SCALE=4, SETTLE=0.25 s (bench.yml:83-88).
- Threads: `1,2,nproc` (bench.yml:218), with the 2-thread run `--spread`. Runners have 4 vCPUs on 2 cores × SMT: EPYC 7763/9V74/9V45, Xeon 8370C/8573C.
- Corpora: primary `wikidata` (12 MB of real blocks, pinned in `bench/wikidata.json`) and `text`; secondary `records`, `random`, `pattern`, `onebyte`, `runs` (from `tests/common/mod.rs`).
- Extras: `bench/stages.py` (cycles/B per stage) and `alts/kernels` (inverse BWT against bz2ngrs and libsais).
- Verdicts (`bench/report.py`): geometric mean of paired ratios with a 95% t-interval (:173-179). Faster or slower only when the interval excludes 1 (:1046-1055). The resource verdict (:684) covers speed, CPU/byte, contention amplification and peak RSS.
- Acceptance (PERFORMANCE-POLICY.md §2):
  - a primary corpus is faster beyond noise at some thread count;
  - no primary corpus is slower beyond noise;
  - secondary losses ≤5% are listed; 5-10% need ≥2× primary gain plus a justification; >10% needs Louis;
  - peak RSS grows ≤10% on any corpus, or the PR justifies the growth for the target machine;
  - one run decides, and only a flagged primary regression gets one re-run.
- **CI never runs a team path.**

**Counting.**
- Callgrind Ir, e.g. `valgrind --tool=callgrind --toggle-collect='*measured*' [--cache-sim=yes --D1=32768,8,64 …]`.
- Lackey validated a T-vector trace replica: 900,444 loads on CI block 1, with the simulator within 0.5% of callgrind D1/LL (`2026-11-05-partitioned-walk/results.json`).
- Counts, cache models and traces locate work; only wall time decides integration (OWNERSHIP.md §3, line 44).

**Workstation (i9-10900K).**
- Hardware: i9-10900K, 10 cores / 20 CPUs, 20 MB L3, 4×32 GB DDR4-2666 (dual rank, two DIMMs a channel, 42.7 GB/s theoretical) (`2026-10-28-workstation-scaling.md:13-14`). The 256 KB L2 per core is the vendor specification, not stated there.
- A/B template: `just bench-release-scaling NEW OLD ROUNDS THREADS` (.just/bench.just:225). A standing template for any two commits (RULES C5, 2026-11-10). Examples: `bench-324-i9` (:1147), `bench-team-ports` (threads 8,10,12,16,20).
- Queue: `bench/workstation-queue.txt` (empty; PROTOCOL.md §9). Admission takes a frozen recipe, an untimed dry run, CI green and a `ready/<recipe>` tag via `.github/workflows/run-signal.yml`.
- Louis runs `just workstation-queue` or `just submit-when-ready`. `just submit` archives the bundle with its SHA-256 on the `evidence` branch and posts it to #145. Records go to `bench/release-scaling/<date>-<machine>-<label>/`.

**Local timing** on the shared machine is indicative only (RULES C5).

### 7.2 Testing and B5

**Correctness bar:** byte-identical output on every bench corpus and the same error on corrupt input as before the change (PERFORMANCE-POLICY.md §1).

**Tests:**
- `tests/{decode,corpus,cooperative,pipelined,in_place,mtf_checkpoints,mtf_kernels,reader_panic}.rs`.
- `cargo test`, `cargo test --example ab`, `timeout 600 cargo test --example par_scan`, and `just test` (nextest).

**B5** (RULES.md:58), for new `unsafe` code:
- a written safety argument with its proof scope;
- Miri, differential, fuzz and mutation testing;
- an adversarial review by a different agent.

A B5 objection blocks a merge. B7: a Miri timeout is never a PASS.

**Miri:** `.github/workflows/miri.yml` runs `cargo miri test --lib -- miri_` on changes to `src/{bwt,coop,parallel,read}.rs` and `src/parallel/**`. Examples: `bwt::tests::miri_build_one_pass` (bwt.rs:1750) and `miri_room_walk_handoffs`. Each asserts that its handoff transitions were reached.

**Fuzz:** `examples/fuzz.rs`, built with `cargo build --profile fuzz --example fuzz`.
- Run: `target/fuzz/examples/fuzz SECONDS SEED OUTDIR|- [mixed|classic|pipeline]`.
- It checks against libbzip2 and the plain parallel decoder.
- CI smoke run: 30 s at seed 1 (`.github/workflows/test.yml:45-56`).

**Mutation:** hand-made invariant mutants in `mutants/mutants.py` and `M*.patch` (`2026-11-09-pipeline-fuzz-and-mutation.md` §3).

### 7.3 Governance

**OWNERSHIP.md §4a (this lane):**
- MONODROMY owns its programme and may open PRs, including in `src/` (RULES O1).
- It is exempt from registration, Researcher/Guardian review, Optimiser pricing, the 48 h expiry and the 24 h merge clock.
- OWNERSHIP.md §4a sketches a loop for this lane (conjecture → derivation → counterexample or prototype → advance or abandon) and asks that proved, conjectured and measured results be kept apart. How MONODROMY works within that is its own call.
- Production adoption still meets §2 and PERFORMANCE-POLICY.md.
- The Coordinator decides merges, and the work never delays shipping.
- Diagnostics that feed any decision need a 3-line pre-data post on #145 (RULES C1a). The exemption covers theory and prototypes only.

**RULES §O** (RULES.md:104):
- O1: any role may open a PR; its author owns it to merge.
- O2: one reviewer, one round, four outcomes.
- O3: a HOLD binds only under B1, B5, B6 and B7.

### 7.4 Channel and peers

**Channel and protocol:**
- #145 is the team channel. The message header is FROM/TO/RE/TYPE, with TYPE ∈ CHECKPOINT | PROPOSAL | CORRECTION | QUEUED | REVIEW | DECISION | ACK | LOG | ESCALATION (PROTOCOL.md §3).
- One assignment is in flight per role (§2), and review is bounded (§2a).

**Routing** goes through `.github/coordination/coord.py`.
- Inboxes: optimizer #160, guardian #161, researcher #162, observer #222, pathfinder #314, monodromy #346.
- Commands: `route --to R --type T`, `verify --comment-id N --to R --type T`, `audit`, `head-sha --pr N --sha S [--merged]`.
- MONODROMY's cursor: `team-state/monodromy:state/cursor.json` (`processed_comment_ids`).
- PROTOCOL.md §1 still lists only #160-#162 and #222. #314 and #346 are defined in coord.py's ROLES (pathfinder :70-75, monodromy :78-83) and in OWNERSHIP.md:103.

**Records:** `docs/journal/monodromy/` holds only `README.md` today.

**Pathfinder:** a peer, not a superior or a subordinate. Exchange is direct, #346 ↔ #314. Its #313 record is the PF branch cited throughout, including the hypothesis register, interval-exchange, tunnels, chunked-link, backward-table and two-modes journals.

### 7.5 Uncertainty in this file

- Line numbers marked "~" are approximate.
- `2026-10-28-cooperative-inversion.md` and `2026-11-10-backward-table.md` are cited from master but exist only on branches.
- Figures from the PF branch and the research branches were reported by the readers, not re-checked against those branches for this file.

Source checkout: /tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/mono

Written 10 November 2026 from master; describes, does not prescribe.
