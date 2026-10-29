# Expert materials for the next inverse-BWT stage, read against what is now measured

[External report 3](2026-10-29-external-report-3.md) asked for an expert-materials review:
- the sources' mechanisms, extracted for bzip2's blocks on the i9;
- each mapped onto this journal's history;
- a ranked experiment matrix.

This entry does that. Most of the report's sources were already read in full here, so it
re-ranks them against what has been measured since:
- the i9's knee (`2026-10-28-workstation-scaling.md`);
- the cooperative grid (`2026-10-28-cooperative-inversion.md` §7);
- the kernels' table on CI (`2026-10-29-bz2ngrs.md` §3b, §7).

One source is read afresh: libsais's own cyclic inverse BWT, the code path our harness
calls.

Labels: **source** (read at the version or commit given), **measured** (said where),
**hypothesis** (ours, untested).

## 1. Where each source already stands here

| source | read where | what it gave | status |
|---|---|---|---|
| Kärkkäinen, Kempa & Puglisi, "Slashing the Time for BWT Inversion", DCC 2012, pp. 99–108, [pdf](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf) | `2026-10-27-literature-reading.md` (read in full) | interleaved traversals; the super-alphabet (LF², the pair counting sort); Precopy; 2.3–4× over Seward's mergedTL, in their setting | interleaving shipped (the room walk); LF² ported and **rejected on CI** (#6, `2026-10-27-pair-walk.md`); Precopy **rejected** twice (#9, `2026-10-27-precopy-forward-chains.md`) |
| Kocberber, Falsafi & Grot, "Asynchronous Memory Access Chaining", PVLDB 9(4) 2015 (VLDB 2016), [pdf](https://www.vldb.org/pvldb/vol9/p252-kocberber.pdf) | `2026-10-27-mlp-scheduling-and-run-structures.md`, lead 3 | per-lookup state in a circular buffer; refill as a lookup completes; about 10 in flight a core is enough | the room walk **is** an AMAC-style loop (a lane takes the next segment when its own ends); #39 holds the open window hypothesis |
| Ainsworth & Jones, "Software Prefetching for Indirect Memory Accesses", CGO 2017 / TOCS | same entry, §2 | prefetch the next indirect target a distance ahead; one chain's own next hop can't be prefetched | shipped as each lane's next entry, prefetched to L1 (#5, v0.1.3); the sweep found no better setting (`2026-10-27-walk-sweep.md`) |
| Brown & Langmead, "Bounding the Average Move Structure Query for Faster and Smaller RLBWT Permutations", SEA 2026, LIPIcs 9 (doi:10.4230/LIPIcs.SEA.2026.9; arXiv:2602.11029) | same entry, §3 | length capping bounds the average query | built for ψ and **rejected** (#42, `2026-10-27-move-structure-walk.md`): +90–108% under the room walk. A reopening condition is named (the walk sweep, "What it leaves open"): under four copies, move build + walk at least 10% under build + room walk |
| Edwards & Vishkin, "Parallel algorithms for Burrows–Wheeler compression and decompression", TCS (doi:10.1016/j.tcs.2013.10.009) | `2026-10-28-cooperative-inversion.md` §5b.2 | O(log n) depth, O(n) work, by list ranking | theory. The cooperative pilot's segments are its practical form, and their coordination measured about 4% of a block's walk (§7.3 there) |
| libbzip2's low-memory decoder (`BZ2_decompress` with `small`: ll16 + ll4, 2.5 B a row) | `2026-10-28-cooperative-inversion.md`; the compact pilot (unpublished branch) | 20-bit links instead of 32, 31% less table | a pilot built on the VM: the walk is 0.7–0.8 × the T-vector's, and the packed build is far slower (85 against 1,052 MB/s). It is parked behind the cooperative work |
| Intel's top-down method: A. Yasin, "A Top-Down Method for Performance Analysis and Counters Architecture", ISPASS 2014, pp. 35–44; Intel's VTune memory-access guidance | not before (bench-imc used single events) | a hierarchy: retiring, bad speculation, front-end bound, back-end bound (memory: L1, L2, L3, DRAM, store bound; core) | **adopted as the method for §4's counters** (E3). Its events are confirmed on the i9 before relying on them |
| Intel Performance Counter Monitor | — | memory-controller traffic | already measured through perf's `uncore_imc` events (`bench/imc.py`); PCM adds nothing needed |
| libsais 2.12.0 / 2.12.1 (Ilya Grebnov) | §2 below, and [its source reading](2026-10-29-expert-review/libsais-unbwt-path.md) | the cyclic inverse BWT that our harness calls | read here, at c284ba8 |

## 2. libsais's cyclic inverse BWT, from its source (c284ba8 = 2.12.1)

The full reading, with a line number for every claim, is
[`2026-10-29-expert-review/libsais-unbwt-path.md`](2026-10-29-expert-review/libsais-unbwt-path.md). It
was made by a delegated reader and spot-checked here against `libsais.c`:
- the size thresholds, L55–70;
- the skipped histogram, L11493–11496;
- the parallel build's recount, L11572;
- the decode's prefetch, L12496;
- the chain-path test, L13284.

**Its changelog** (verbatim, [kept](2026-10-29-expert-review/libsais-CHANGES-c284ba8.txt)):
- 2.12.0 (October 18, 2026): "Support for cyclic Burrows-Wheeler transform and inverse BWT
  construction (e.g. as used by bzip2)" and "Significantly improved performance of inverse
  BWT, both single- and multi-threaded".
- 2.12.1 (October 21, 2026): MSVC ARM64 and big-endian fixes only, with no x86 or inverse-BWT
  change.

**The path for a 900,000-byte block, counts given** (source):

- **Packed or symbol rows.** n is above 65,535, so not the packed path. It is at most
  8,388,607 (`UNBWT_SYMBOL_ROWS_MAX_SIZE`), so the **symbol-rows** path: `init_single` (or
  `init_parallel`), then `decode_chains_symbol_rows`.
- **Not the fastbits path.** The fastbits + bucket2 decode (`decode_1..16`) runs only for
  larger blocks, or as a fallback.
  - This corrects the coop journal's §5b.1, which described the fastbits lookup and
    bucket2 search: that is libsais's path for other sizes, not bzip2's.
- **The table, P (4 B a row, 3.6 MB).** Each entry is `(ψ²(row) << 8) | the symbol between`,
  with bit 31 for chain marks.
- **One step: one dependent random load of P gives two output bytes.**
  - The second byte comes from a 28 KB `first_symbols` table (one byte per 32 rows) and a
    1 KB `symbol_ends` correction.
  - bucket2 (256 KB) is a write cursor in the build only. The decode never reads it.
- **The build:**
  - the counts are skipped when given, single-threaded only;
  - a bigram histogram, one per first symbol: a pass over L;
  - prefix sums and a transpose of 64 K bigram counters;
  - a scatter of P into up to 65,536 bigram streams (a plain T-vector build scatters into
    256), with two read-modify-write counters per row;
  - a page touch of P, and chain setup.
- **The OpenMP build (`init_parallel`) ignores the counts given**, and each thread recounts
  its slice of L (L11572).
- **The walk:**
  - 24 lanes a thread, in lockstep (64 only for blocks of 16 MiB and up);
  - chains start at marks, one per 256-row window (about 3,500 chains of about 256
    bytes);
  - each step prefetches the lane's next P entry, one full round ahead;
  - a new chain's first load isn't prefetched.
- **The output:**
  - chains write into 16,320-byte chunks;
  - a serial pass links them;
  - a per-chain copy stitches them into P's own memory;
  - a final copy moves them to U.
  - That is two copies of n bytes after the walk.
- **The OpenMP coordination:** six parallel regions a block, four barriers in the build, a
  master-only section, and chains handed out under `omp critical`.
- **The fallback:** if the chains fail to link, the build is redone and one serial chain
  walks the block, with no prefetch.
  - This is a hypothesis for the 13 ms straggler seen once in s3x3 (coop §7.5). It can be
    checked by counting fallbacks.

**What this says about our own record:**
- **#6's port was a port of this same decode** ("libsais symbol rows"). It lost on CI's
  runners at 1 and 4 threads.
- **libsais itself loses on CI's runner:** 1.44 × our kernel's time on an AMD EPYC (`kernels`,
  `2026-10-29-bz2ngrs.md` §7).
- **It wins on the i9:** 1.13 × faster on one core, 1.08 × at 4 workers, 1.20 × our 3x3 at
  s3x3.
- So two symbols per load is not a property of the code alone; it depends on the machine.
  **The ψ² walk was never run on the i9, by us or as libsais at 6–10 workers.**

**Why it might win on the i9 and lose on the EPYC (hypotheses, each with its test):**
1. **Prefetch reach.** One round of 24 lanes is about 500 instructions. That covers an L3 hit
   on the i9 (the table fits in its 20 MB L3) but may not cover a DRAM or remote-CCX
   latency on the EPYC.
   - **The test:** fill-source counters (`mem_load_retired.l3_hit` against `l3_miss`, or
     AMD's data-fabric events) on both.
2. **Work per byte against clock.** About 10.5 instructions a byte in its step loop (a
   local disassembly). More work per byte pays where the clock is higher and the latency
   per load dominates, as on the i9.
   - **The test:** instructions, IPC and GHz for both kernels on both machines.
3. **The two copies after the walk, and the bigram scatter's 65,536 streams.**
   - These cost bandwidth and TLB reach, which a VM or a shared EPYC socket may charge
     more for.
   - **The test:** the build's and the copies' share of its time, by perf symbols.

## 3. Corrections to the report

- **"libsais beats production p4 by about 8% at four independent workers".**
  - It does on the i9 (s4x1 1.083, `2026-10-28-cooperative-inversion.md` §7.1). On CI's EPYC
    its single-core kernel is 1.44 × slower than ours.
  - The ranking's priority 1, "libsais cyclic bigram LF²", therefore cannot go in as a kernel
    for every machine. It is a per-machine question first (E1, E2 below).
- **"Some of the 23% may be explained by the research walk's regression".** Measured, and
  most of it is.
  - Cooperation lifts both implementations by about the same factor (1.57 against 1.50 over
    their own 4 workers).
  - The kernels differ by 1.25 (§7.5 there).
  - The regression of our cooperative kernel splits evenly between its phases, on one i9
    core (block mode): build 0.98 against production's 0.83 ns/B (+18%), walk 1.71 against
    1.48 (+16%).
- **"Rigid lockstep traversal can lose when chains differ in length" (AMAC).**
  - The room walk and the cooperative walk already refill: a lane takes the next segment
    from a shared counter when its own ends (`taken.fetch_add`, `examples/coop.rs`).
  - In 3x3 the threads end within about 11 µs of each other, and the busiest walks 1.36 × the
    mean bytes. Coordination is about 4% of a block's walk.
  - That bounds what dynamic scheduling can recover to about that 4%.
- **"Intel Performance Counter Monitor for DRAM traffic"** is already measured here through
  perf's `uncore_imc` events (`bench/imc.py`).
- **The move structure** "revisit only if run-structure measurements justify it": its named
  reopening condition is in the walk sweep (§1). It is the four-copy case, which the
  workstation can now run.
- **"Software prefetching"**: shipped, then swept (#5, `2026-10-27-walk-sweep.md`). The
  remaining prefetch question is libsais's reach on the EPYC (§2, hypothesis 1), not ours.

## 4. The experiment matrix, ranked by what it can decide for its cost

The two pre-registered cooperative confirmations (`2026-10-28-cooperative-inversion.md`
§8.1) come before all of these, unchanged. Each row's criterion is fixed before it runs.

| # | experiment | mechanism | code | cost (workstation) | correctness risk | counters | pass / fail |
|--:|---|---|---|---|---|---|---|
| E1 | **The pair walk on the i9**: `just bench-variants --variants v0.1.4,pairwalk --threads 1,4,6,8,10 --rounds 10` | two bytes per dependent load, under pressure | none: the kept `perf/pair-walk` branch (v0.1.4 + one commit) as a `bench-variants` variant | ~15 min | none: byte-checked against this build | — | **pass:** pairwalk ≥ 1.05 × v0.1.4 at its best and at 8–10 workers, interval above 1, in paired rounds. **fail:** ≤ 1.0 at every count. ψ² then closes on the i9 too, and libsais's lead is its own code's. |
| E2 | **libsais against production, by worker count** (queued: `p4…p10, s4x1…s10x1`, 16 rounds; `p4,4x1,3x3,s4x1,s3x3`, 20 rounds) | the same, as libsais | none (PR #73 at c2620b8) | ~10 min | none | — | **informs E1:** sNx1 / pN rising with N says ψ²'s gain is memory pressure. Flat at ~1.08 says it's a constant kernel difference. |
| E3 | **Why 3x3 wins**, and libsais's phases: each arm alone under `perf stat`, top-down level 1–2 plus memory events plus IMC; libsais's symbols by `perf record` | residency against overlap (coop §8.2); libsais's build and copies against its walk | a script, after the confirmations | ~30 min | none | top-down (Yasin 2014; events confirmed on the i9 first), `mem_load_retired.l3_hit/miss`, `offcore_requests_outstanding.*`, `uncore_imc` reads and writes; repeats kept raw, multiplexing checked | as coop §8.2's predictions. For libsais: the share of `calculate_biPSI*`, `decode_chains_symbol_rows` and the copies in its time |
| E4 | **The cooperative kernel at production's speed** | the build and walk's +18% and +16% on one core | research harness (#73's successor branch) | VM, then ~10 min | the shared table's handoffs: segment oracle, Miri | instructions per byte against production's | **pass:** 4x1 ≥ 0.97 × p4 in the grid, with 3x3 ≥ its confirmed ratio. **fail:** either misses |
| E5 | **The parallel build without its count pass**: MTF already counts the block's bytes, so let it also snapshot the counts at the k cut points as it writes L. A team's threads then scatter at once. | the build is 42% of 3x3's time; libsais's own parallel build also recounts (L11572) | MTF and the coop build (research) | VM, then ~10 min | the cut points' counts must equal a recount: checked per block | — | **pass:** 3x3's build ≥ 1.25 × its current (4511 MB/s) and 3x3 ≥ 1.45 × p4. **fail:** below either |
| E6 | libsais's walk parameters in our ψ² port (24 lanes, a round of prefetch lead, 256-row chains) | as libsais | the pair-walk branch | VM, then ~15 min | as #6's (byte checks) | — | **only if E1 passes:** ≥ libsais's s4x1 / p4 on the i9, and no worse than 1.0 × production on CI's 2-core runners. **Otherwise the per-machine choice is recorded instead.** |
| E7 | The compact 20-bit ψ (ll16 + ll4 or packed) under pressure | 31% less table | the parked pilot | ~10 min | its self-test | L3 hit share | **pass:** its best absolute at 6–10 workers > p4's best. **fail:** below |
| E8 | The move structure under four copies (#42's named reopening) | a sixth of the T-vector's working set | the #42 branch | ~10 min | as #42's | — | as named: move build + walk ≥ 10% under build + room walk per copy at four copies; **otherwise it closes for good** |
| E9 | The walk's output in L's freed buffer | memory reuse; its effect on the knee is unknown (`2026-10-29-bz2ngrs.md` §4) | the room walk (research) | ~10 min | as the room walk's oracle and Miri | DRAM write bytes per output byte | **pass:** occupancy curve at 6–10 workers ≥ 1.03 × today's |
| E10 | Dynamic scheduling beyond the shared segment counter (AMAC proper) | refill at finer grain | — | — | — | — | **not run.** Its ceiling is the ~4% coordination share (§3). It reopens only if E3 finds the walk's end spread or start lag above 10% of a block. |

**Why this order:**
- E1 and E2 need no code and decide the report's own first priority.
- E3 decides which mechanism E4–E9 should serve.
- E4 and E5 are the measured costs of the arm that already wins (3x3).
- E6–E9 are conditional.

**What the report's ranking would change:**
- It put "libsais LF²" first. The record says first establish whether LF² pays on the i9 at
  all (E1). On CI it lost both as our port and as libsais.

**The complementarity the report names** (fewer live tables; more bytes per lookup; cheaper
construction) is a fair frame:
- E4 and E5 are cheaper construction and a faster kernel inside fewer live tables.
- E1, E2 and E6 are more bytes a lookup.
- Their gains need not add: a ψ² table changes the build (65,536 bigram streams) and its
  cache behaviour.

## 5. The method for counters (E3)

- **Confirm first.** Before relying on an event on the i9, check that it exists and counts
  there (`perf list`, and `perf stat -e` on a known kernel). Use top-down level 1, then the
  memory-bound breakdown, as Yasin's method orders them.
- **Run each arm alone, pinned as in the grid, repeated three times.**
  - Keep the raw counts, and drop a reading whose time enabled differs from its time
    running (coop §8.2's guard).
  - A difference between two runs counts only if its spread over the repeats is small
    against it.
- **Label what is measured against what is modelled:** an L3 hit share is measured; "fewer
  misses because fewer tables" is the hypothesis it tests.

## 6. Deferred directions, filed

Filed as issues, so none depends on the outside session that produced them.

**New issues:**
- **E3**, the mechanism and libsais's phases: [#76](https://github.com/lmmx/mibz2/issues/76).
- **E4**, the cooperative kernel's parity: [#77](https://github.com/lmmx/mibz2/issues/77).
- **E5**, the build without its count pass: [#78](https://github.com/lmmx/mibz2/issues/78).
- **E7 and E9**, footprint under pressure: [#79](https://github.com/lmmx/mibz2/issues/79).

**Comments on existing issues:**
- **E1 and E6** on [#6](https://github.com/lmmx/mibz2/issues/6#issuecomment-6490650295) (closed), with the reopening criterion.
- **E8** on [#42](https://github.com/lmmx/mibz2/issues/42#issuecomment-6490651624) (closed), with its named condition.
- **E10** on [#39](https://github.com/lmmx/mibz2/issues/39#issuecomment-6490652998), with its ceiling.

**Already queued** (`2026-10-28-cooperative-inversion.md` §8): E2, and the two confirmations that come first.

**What this entry adds to `bench/variants.py`:** the variant `pairwalk`, the kept `perf/pair-walk` commit, so E1 is one recipe.
