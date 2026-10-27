# Memory-level parallelism, prefetch scheduling and run-length permutations, read against the room walk

A literature round, written after the room walk (#37) merged and shipped in 0.1.5. It reads
three sources and the vendor documents, and asks one question of our own: what else stops
the hardware from keeping more misses in flight? §5 adds the user's leads, as open questions for
the next experiment. Its outputs are issues #39, #40 and #42.
Nothing here changes `src/`.

Every finding is labelled as one of three kinds:
- **published:** what a source states or measured;
- **ours, measured:** a figure from our CI or a local run;
- **interpretation** or **hypothesis:** ours, and untested.

No paper's speedup below is a claim about mibz2.

## The baseline the sources are read against (ours, measured)

**The room walk (#37), on the CI bench's final head `6c645ef`** (Intel Xeon 6973P-C, 10 paired
rounds, bench 1.2.0):
- **Whole decode, Wikidata:** −5.8% at 1 thread and −2.7% at 4 threads.
- **Stage table:** walk 4.37 → 3.81 c/B (−14.2%), 47.7% of a one-thread decode; build 1.71 c/B.

**The CI runner varies between bench runs.** #37's earlier head `db3593d` was benched on an AMD
EPYC 9V74 (Zen 4) at −10.3% / −9.1%. Each verdict is paired on one runner, but a
hardware-specific tuning can rank differently on the other runner type.

**The walk's mechanics** (`src/bwt.rs`):
- 32 lanes and 512 pseudo-random segment starts, flagged with START (bit 31) in their own
  entries;
- `_MM_HINT_T0` on each lane's next entry, issued as soon as the row is known (`prefetch`,
  bwt.rs:456–468);
- the room walk's passes, which run `room` rounds over all lanes. `room` is the fewest spare
  bytes in any lane's buffer.

**Prior negative results** that every hypothesis below is checked against:
- **ψ² (#6, #29, #34; [the pair walk](2026-10-27-pair-walk.md)):** half the steps, each about
  twice the cost, +21–51%.
- **Forward-chain Precopy (#9, #30, #36; [Precopy](2026-10-27-precopy-forward-chains.md)):**
  removing cheap, cache-resident steps saved nothing, +6–20%.
- **The lesson both left:** with prefetch keeping 32 misses in flight, the walk is bound by
  instructions per step. Per-step work added loses.

## 1. Kohout, Choi, Kim & Yeung, "Multi-Chain Prefetching" (PACT 2001)

N. Kohout, S. Choi, D. Kim, D. Yeung. *Multi-Chain Prefetching: Effective Exploitation of
Inter-Chain Memory Parallelism for Pointer-Chasing Codes.* PACT 2001, Barcelona.
- doi:10.1109/PACT.2001.953307
- PDF: http://maggini.eng.umd.edu/pub/yeung_multichain.pdf
- Journal version: S. Choi, N. Kohout, S. Pamnani, D. Kim, D. Yeung, *A General Framework for
  Prefetch Scheduling in Linked Data Structures and its Application to Multi-Chain
  Prefetching*, ACM TOCS 22(2), 214–280, 2004 (doi:10.1145/986533.986536). Read for its
  citation only.

**Published:**
- **The mechanism** (§2.1, §3.2): a *hardware prefetch engine* beside the core traverses
  several independent pointer chains ahead of the program, to a schedule computed offline.
  - The schedule has two numbers per chain descriptor: the pre-traversal time PT (how early a
    chain's traversal must start) and the prefetch distance PD = ⌈PT / w⌉, where w is the work
    per outer-loop iteration.
  - PD is the number of chains in flight at once.
  - For one list with l-cycle misses and w₁ cycles of work a link, the stall is 3l − 2w₁ for
    three links (Figure 1a). A single chain gives no memory parallelism; independent chains do.
- **Evaluation:** simulated, not real hardware.
  - an L1 with 8 MSHRs and an L2 with 16;
  - a 1/10/70-cycle hierarchy and a 32-entry prefetch buffer.
- **Results:** 40% lower execution time on six Olden kernels and 8% on four SPECint2000 programs.
  The technique "loses its effectiveness when the pointer-chain traversal order is
  unpredictable".

**Applicability to standard .bz2 streams:**
- The hardware engine doesn't exist on our CPUs.
- The scheduling model transfers as arithmetic. mibz2's lanes are its software equivalent: chains
  known to be independent (segments from pseudo-random starts), each prefetched one hop ahead,
  the only hop whose address is known.
- **Terminology:** Kohout's "chain" is a lane's pointer chain. KP 2011's "chain", the one #36's
  Precopy used, is a run of parallel adjacent rows. The two are unrelated.

**Interpretation:**
- **The lead:** in a round of L lanes at s cycles a step, a lane's prefetch leads its load by
  about L·s cycles. At 32 lanes and 3.8–7.4 c a step (CI and local room walk), that is about
  120–240 cycles.
- **Against L3 latency:** the documented L3 latencies are 46–50 cycles on Zen 3/4 (AMD 56665,
  57647) and about 77 cycles on Skylake-SP (AnandTech, one source). PD = ⌈PT / w⌉ says latency
  alone would need about 8–12 lanes.
- So 32 lanes are set by something other than latency: miss-buffer capacity (§4), and in the
  room walk, the pass length.

**Hypothesis:** with fewer instructions per step after #37, the best lane count can move. It
interacts with `room`, since the minimum spare over 32 buffers is smaller than over 16. That is
#39.

**Limits:**
- The results are simulations, of 2001 cores, with a hardware engine.
- None of the speedups bears on a software-prefetched walk.

## 2. Ainsworth & Jones, "Software Prefetching for Indirect Memory Accesses" (CGO 2017; TOCS)

- S. Ainsworth, T. M. Jones. *Software Prefetching for Indirect Memory Accesses.* CGO 2017,
  pp. 305–317 (doi:10.1109/CGO.2017.7863749).
  https://www.cl.cam.ac.uk/~sa614/papers/Software-Prefetching-CGO2017.pdf
- The extended version: *Software Prefetching for Indirect Memory Accesses: A Microarchitectural
  Perspective*, ACM Transactions on Computer Systems 36(3), pp. 1–34
  (doi:10.1145/3319393). Preprint: https://www.cl.cam.ac.uk/~tmj32/papers/docs/ainsworth19-tocs.pdf
- **Which is "2019":**
  - the preprint's ACM reference reads "March 2019";
  - Crossref dates volume 36 issue 3 to 2018-08-31.

  So "2019" is the TOCS article's publication, in an issue dated 2018. The 2019 version adds four
  systems and two configurable benchmarks, Kangaroo and Camel. The TOCS version was read in full
  text, the CGO version only as cited by it.

**Published (TOCS):**
- **The target class** (§4): stride-indirect accesses `A[B[i]]` traced back to an induction
  variable.
  - §4.8 excludes linked structures explicitly: "there is no induction variable to move ahead in
    the computation".
  - The introduction says linked-list prefetching has "limited performance improvement due to
    the lack of memory-level parallelism inherent in linked structures".
- **The schedule** (§4.4, eq. 1): `offset = c(t − l)/t`, for load l of t in a dependent sequence.
  Dependent prefetches are spaced equally.
  - c = 64 is "close to optimal for every benchmark and microarchitecture combination" (§6.2,
    Figure 6), on Haswell, Kaby Lake, Cortex-A57, A72, A53, Kryo and Xeon Phi.
  - Arriving late hurts more than arriving early.
- **The costs of prefetching** (§6.2, §7.7):
  - Prefetching every level of a dependent sequence costs O(n²) extra code, and on all four
    systems in Figure 7 it was best to leave the last level out.
  - When the data fits in the LLC (Kangaroo at 0.5 MiB arrays), prefetching "is usually reduced
    compared to the baseline due to the code overhead, reaching as low as 0.4×".
  - On the out-of-order A72, CG (data in the LLC) lost from prefetching.
- **TLB** (§6.2, Figure 11):
  - The A57 handles one page-table walk at a time, which limited its gains.
  - With transparent huge pages, the gains were slightly lower for simple benchmarks and higher
    for page-walk-bound ones.
- **Speedups** (abstract): 1.1–1.35× on out-of-order cores, 2.1× and 2.7× on in-order ones.

**Applicability:**
- The walk is a pointer chain per lane, outside the paper's class. There is no look-ahead array,
  so the distance can't be tuned as an offset.
- In our walk, "distance" is the lane count times the step time (§1). The knobs left are the hint
  and the placement.
- The O(n²) staggering result is the same wall ψ² met (#6): ψ(ψ(x)) can't be prefetched without
  first loading ψ(x). That is why ψ² changed the table instead, and paid for it per step.

**Interpretation:**
- The T-vector (3.6 MB) is LLC-resident on every machine we run on, which is the case where the
  paper found prefetch overhead can dominate.
- Our prefetch was a win all the same (#21). A plausible reason, unmeasured: without it, the next
  round's load of a lane is 32 lane-steps away, beyond what the reorder window covers.
- So prefetch placement and hint are worth a measured sweep after an instruction-mix change,
  rather than assumed. That is #39.

**Limits:**
- Their codes are not pointer chains, and their cores are 2014–2017 designs.
- The c = 64 finding doesn't transfer as a number.

## 3. Run-length permutations: the move structure and length capping

- **Brown & Langmead, SEA 2026:** N. K. Brown, B. Langmead. *Bounding the Average Move Structure
  Query for Faster and Smaller RLBWT Permutations.* SEA 2026, LIPIcs, article 9
  (doi:10.4230/LIPIcs.SEA.2026.9; arXiv:2602.11029). Library: Orbit,
  https://github.com/drnatebrown/orbit (the v1 preprint names it RunPerm).
- **Nishimoto & Tabei, ICALP 2021, the antecedent:** T. Nishimoto, Y. Tabei. *Optimal-Time Queries
  on BWT-Runs Compressed Indexes.* ICALP 2021, LIPIcs 198, 101:1–101:15 (arXiv:2006.05104), §3.
- **Practical balancing, also built on:**
  - N. Bertram, J. Fischer, L. Nalbach, *Move-r: Optimizing the r-index*, SEA 2024, LIPIcs 301, 1;
  - Brown, Gagie & Rossi, *RLBWT tricks*;
  - Zakeri et al., *Movi*, iScience 2024.

  These were read for their role in the SEA paper only.

**Published:**
- **The move structure** (Nishimoto & Tabei §3.2): a permutation consecutive on k input intervals
  is stored as `Dpair[x] = (start p_x, image q_x)` and `Dindex[x]`, the interval holding q_x
  (3k words).
  - A query computes `i' = q_x + (i − p_x)`, then scans forward from `Dindex[x]` to the interval
    holding i'.
  - *Balancing* splits intervals until each output interval has at most three incoming. That
    gives ≤ 2k intervals and ≤ 3 fast-forwards per query.
  - For LF on a BWT, k = r, the number of runs.
- **Length capping** (Brown & Langmead):
  - split every interval longer than L = c·(n/r);
  - that adds at most r/c intervals, and the build is O(r);
  - over n queries on a single-cycle permutation the fast-forwards total at most c·n + n, so the
    average is ≤ c + 1;
  - the worst case is O(n/r), or O(log(n/r)) with exponential search.
- **Their experiments:**
  - 16–1,000 copies of human chromosome 19 (n/r 29.2–1,287.4), on a Xeon Gold 6248R, one
    thread;
  - BWT inversion took 83–98 ns per LF query (Table 2, d = 16), as one dependent chain;
  - "up to ∼17 times faster when compared to an unbalanced representation of ϕ";
  - "at least a ∼40% disk size decrease for LF … when compared to a balanced/unbalanced move
    structure".
- **What they don't compare:** every comparison is between move-structure variants. None is
  against an explicit array of LF or ψ, nor against an interleaved walk.

**Applicability:**
- **Format:** none needed. ψ is consecutive on the runs of the last column within each F-bucket,
  so a bzip2 block's ψ is a run-length permutation like LF.
- **The emitted byte** is constant per interval, so it can be stored with it.
- **The build:** a sequential scan of L for runs, plus O(r) to link intervals.

**Ours, measured** (local, one build of an uncommitted scratch counter on the bench's
`wikidata.bz2`, SHA-256 `82f65ace…`; its ψ checked against `bwt::walk`'s output on every block):

| Measure | Value |
|---|---|
| Blocks | 431. The 233 of ≥ 65,536 rows (188 of exactly 900,000, 45 of 899,809–899,999) hold 209,697,662 rows, 99.996%. The other 198 hold 8,895 rows (median 29). |
| BWT runs, r / n (the 233 blocks) | **0.0722**, n/r 13.85. Per block: min 0.0533, p10 0.0568, median 0.0623, p90 0.1100, max 0.2000 |
| ψ intervals / n | 0.0722: 15,147,913 intervals against 15,148,086 runs, all blocks |
| Rows by the length of their run | 1: 5.18%; 2–3: 2.23%; 4–7: 1.89%; 8–15: 3.08%; 16–31: 3.57%; 32–63: 3.78%; 64–255: 8.92%; ≥ 256: 71.35% |

The move structure is bit-packed at minimum widths: start and image 20 bits each, the rank
⌈log₂ r'⌉ bits, and the byte 8 bits. The mean fast-forward is over all rows. A walk visits
every row once, so that is the mean per walk step, whatever the lane order.

| Capping | Intervals / n | Mean fast-forwards a step | Max | Size | A 900,000-row block |
|---|--:|--:|--:|--:|--:|
| none | 0.0722 | 7.475 | 3,160 | 0.581 B/row | 523 kB |
| c = 8 | 0.0787 | 0.534 | 150 | 0.635 B/row | 572 kB |
| c = 4 | 0.0859 | 0.416 | 75 | 0.695 B/row | 626 kB |
| c = 2 | 0.1007 | 0.342 | 37 | 0.821 B/row | 739 kB |
| c = 1 | 0.1308 | 0.297 | 18 | 1.067 B/row | 960 kB |

The capped averages sit well under the paper's c + 1 bound. Without capping, the long runs (71%
of rows in runs of 256 or more) make the fast-forward 7.5 a step.

**Interpretation:**
- **n/r:** at 13.85, Wikidata's n/r is below the paper's smallest dataset (29.2).
- **Size:** the structure is still 0.6–1.1 B/row against 4, about 0.5–1 MB a block against
  3.6 MB. That is near a 1 MB L2 (Zen 4, Cascade Lake) and within a 2 MB one (Golden Cove server).
- **What it would change:** the walk's misses would move from L3 toward L2, with fewer lines in
  flight per step. That is the opposite trade to ψ² (#34), which kept the misses and changed the
  steps.
- **The cost:** each step would add a packed-record decode, a fast-forward test and a second piece
  of lane state (the interval index). The fast-forward branch is taken on 30–53% of steps at
  c = 1–8 and is data-dependent, so it likely mispredicts often.
- **Against the prior results:**
  - #34 lost at twice the per-step cost with half the steps.
  - The move walk keeps every step, so it gains only if L2 residency removes more cycles than the
    extra instructions add.
  - #36 says that removing cheap steps saves nothing. The move walk removes none. It makes the
    expensive scattered steps cheaper, which is the question #36 left open.
- **Other corpora:** for `random` and `records`, r/n is near 1 (about 8 B/row). A run-length ψ
  could only be a per-block choice by r/n. That is cheap to compute, but #27's per-block choice
  was rejected, for its own reasons.

**Hypothesis:** a length-capped run-length ψ beats the explicit T-vector on Wikidata's run-heavy
blocks, build and per-step lookup included. Its first experiment (above) passes the size gate.
The next is a scratch 32-lane walk over the c = 4 structure (#42).

**Limits:**
- The paper's timings are one chain, a billion-row DRAM-resident n, and genomes. None of them
  transfers.
- Our size estimate leaves out alignment padding, which a fast decoder may want.

## 4. What else limits the misses in flight (the coordinator's question)

### Miss buffers

**Published (vendor documents):**
- **Zen 3 and Zen 4:** "The LS unit can track up to 24 outstanding in-flight cache misses in the
  Miss Address Buffer (MAB)" (AMD 56665 rev 3.00, AMD 57647 rev 1.00).
  - L2: 1 MB, ≥ 14 cycles.
  - L3: an average of 46 (Zen 3) or 50 (Zen 4) cycles.
- **Golden Cove:** "16 FB", with MLC miss queues deepened from 32 to 48 (Intel 248966-046,
  January 2023).
- **Nehalem:** 10 fill buffers (the same manual, §E.4.5).
- **Skylake Server / Cascade Lake** (the local machine, model 85 stepping 7): the manual read here
  gives no fill-buffer count. The commonly cited 12 is unverified.
- **The Intel manual on software prefetch:** "A software prefetch into L1 will consume critical
  hardware resources (fill buffer) until the cacheline fill completes. A software prefetch into L2
  does not hold those resources."

**Interpretation:**
- The briefing's 10–16 holds for Intel cores. AMD's Zen 3/4 have 24.
- **Little's law floor:** steps per cycle ≤ buffers / latency.
  - With 24 buffers and a 50-cycle L3, the floor is about 2 c a step even if every step went to
    L3, well under the CI walk's 3.8 c/B.
  - With 16 buffers, the floor depends on the Granite Rapids L3 latency, which no source here
    gives. Of the order of 100 cycles would put it near 3 c/B, close to the measured 3.81. That is
    unverified.
- So the walk can be bound by instructions on one runner and by fill buffers on another.
- 32 lanes exceed every buffer count above. Lanes past the buffer count add lead, not
  concurrency.

**Hypothesis:** on fill-buffer-bound Intel cores, `T1` (into L2) in place of `T0` frees fill
buffers, at the cost of an L2 hit at the demand load. The Zen 3 and Zen 4 guides don't say what
the hints do (§5, lead 5). That is the hint arm of #39 and #41's factorial, which must win on both
runner types or be gated by vendor.

### The L2 hardware prefetcher

**Published:**
- Intel's L2 spatial prefetcher "strives to complete every cache line fetched to the L2 cache
  with the pair line that completes it to a 128-byte aligned chunk" (248966-046).
- Zen 4's L2 Up/Down prefetcher "uses memory access history to determine whether to fetch the
  next or previous line for all memory accesses" (57647).
- Both vendors control the prefetchers by MSR only.

**Hypothesis (no experiment):**
- For the walk's pseudo-random misses, a pair or next-line fetch doubles the L3 → L2 line traffic
  and takes L2 queue entries and L2 capacity.
- A library has no lever over this, and turning prefetchers off is machine configuration. It is
  recorded so it isn't rediscovered as an idea.

### TLB reach

**Published (vendor documents, and one measurement):**
- **Zen 4:** L1 DTLB 72 entries; L2 DTLB 3,072, 24-way; six page-table walkers (57647). Chips and
  Cheese gives "288 KB … with no latency overhead, or 12 MB with a slight 7–8 cycle penalty".
- **Skylake / Cascade Lake** (248966-046, Table C-1):
  - L1 DTLB: 64 4K entries;
  - STLB: 1,536 entries, 12-way, shared by 4K and 2M pages;
  - an STLB hit costs 7 cycles: `%STLB.HIT.COST = 100 × DTLB_LOAD_MISSES.STLB_HIT × 7 /
    CPU_CLK_UNHALTED.THREAD` (248966-046).
- **Golden Cove:** a 96-entry load DTLB for 4K pages, and four D-side page walks in parallel
  (248966-046).
- **Sandy Bridge / Ivy Bridge** (Bloom's i7-3770; the same Table C-1): STLB 512 4K entries, 4-way.
  That is 2 MB of reach.
- **Bloom, measured** ([Bloom](2026-10-27-bloom-inverse-bwt.md)): large pages took enwik7's 4×8
  decoder from 36.1 to 28.2 c/B.

**Interpretation:**
- A 900,000-row T-vector is 879 4K pages. That is beyond every L1 DTLB above (256–384 kB of reach),
  so nearly every walk load misses it.
- 879 pages fit the Skylake / Cascade Lake and Zen 4 STLBs (6 and 12 MB of reach), so the walk
  pays STLB hits, overlapped across lanes, and almost no page walks. Golden Cove's STLB size isn't
  given in the manual read here.
- At 4 threads each core walks its own block, so per-core reach is unchanged, unless SMT siblings
  share a core.
- Bloom's tables were tens of MB on a 2 MB-reach STLB, so his gain was from page walks and doesn't
  transfer.
- #29's 8-byte pair table (7.2 MB a block) exceeded Skylake's reach and not Zen 4's. That fits
  #29's unmeasured "TLB pressure" explanation (#6) on the local machine only.

**Needs the user's decision, not proposed:**
- Huge pages for the T-vector (transparent huge pages, or `madvise(MADV_HUGEPAGE)`; the local THP
  mode is `madvise`) change how memory is allocated. The project's rules forbid allocator changes.
- So the idea is recorded for the user to decide on, and #40 is limited to measuring the STLB cost.
- **What blocks the measurement locally:** there is no PMU locally (`/sys/bus/event_source/devices`
  has no `cpu`), and `perf` isn't built for this kernel.

## 5. Open questions that could redirect the next experiment (the user's leads)

These five leads came from the user. Each primary source was read here, and what it does or
doesn't say is recorded. A lead whose source doesn't support it is marked as such, not dropped
silently.

The questions are carried on #39. The instrument is #41's sweep tool, which the coordinator is
extending to the factorial below.

### Lead 1. Software prefetch of one dependent chain, and what the gain is made of

**Published:** confirmed in substance (Ainsworth & Jones, TOCS, §1 and §4.8; see §2).
- Linked-list prefetching has "limited performance improvement due to the lack of memory-level
  parallelism inherent in linked structures".
- Their pass "does not attempt to generate prefetches in these cases, as there is no induction
  variable to move ahead in the computation".
- AMAC (lead 3) says the same: "prefetching within the traversal of single pointer chain is not
  possible".

**Ours, measured (prior evidence, PR #21):** the flagged walk alone, local, at 2.8 GHz nominal,
512 segments, before the room walk:

| Lanes | No prefetch | `T0` prefetch |
|--:|--:|--:|
| 8 | 16.42 | 14.63 |
| 12 | 15.41 | 11.97 |
| 16 | 15.31 | 11.17 |
| 24 | 16.24 | 11.22 |
| 32 | 16.39 | 11.51 |

- With four copies at once on four CPUs, the candidates' order changed. 12 lanes, 256 segments,
  no prefetch was 50.99 c/B, against 20.35 at 32 lanes and 512 segments with prefetch.
- At one walk, 16–32 lanes with prefetch were within noise of each other.

**Interpretation:**
- **Without the prefetch,** lanes past about 12 add nothing or lose. The reorder window then
  caps the misses in flight, which is AMAC's claim (lead 3).
- **With it,** 16–32 lanes gain 26–28% over 12 lanes without.
- **The split:** interleaving supplies the parallelism, and the prefetch is what lets more than
  about 12 lanes' misses overlap.
- **After the room walk,** the split is unknown. Fewer instructions a step let the window span
  more lanes, so the prefetch's share may have shrunk.

**Open question:** how much of 0.1.5's walk is interleaving and how much is the prefetch? The
factorial's prefetch-off arm answers it.

### Lead 2. Intel's data-dependent hardware prefetchers

**Published:** partly confirmed, with one correction.
- **Where the AOP prefetcher appears:** Intel's optimization manual (248966-051, October 2026,
  §2.1.1.5) introduces the Array of Pointers (AOP) prefetcher with **Redwood Cove**, not
  Crestmont. Its Crestmont chapter lists next-line, IP-stride and next-page L1 prefetchers and
  L2 streamers, and no AOP.
- **What it detects:** the prefetcher "treats the data prefetched for a constant stride load as a
  pointer and may issue prefetch requests to the memory addresses corresponding to the pointer's
  value".
- **Its conditions:**
  - the first load is "a regular integer load, naturally aligned";
  - the second load "depends upon the first load through the base operand and has no index
    operand";
  - it doesn't trigger when the offset exceeds 32 bytes;
  - it can be turned off by bit 7 of MSR 0x1A4. The manual calls it a data-dependent prefetcher.
- **The other data-dependent prefetchers:** Intel's data-dependent prefetcher page
  (software-security guidance) describes DDPs that "may also examine data values in memory" to
  pick prefetch addresses, including a pointer or "a computed address, such as a base plus an
  index".
  - They don't recurse.
  - They are enumerated by CPUID.(7,2):EDX[3] and disabled for user mode by IA32_SPEC_CTRL bit 8.
  - The page names no microarchitectures, and Intel's hardware-prefetch control guide for Atom
    cores (357930-001) covers cores up to Gracemont only.

**Interpretation:**
- The CI's Xeon 6973P-C is a Xeon 6 P-core part, Granite Rapids, which uses Redwood Cove cores.
  Intel's product pages aren't cited here.
- The walk's load `t[(q >> 8)]` isn't a constant-stride load followed by a base-only dependent
  load. The lanes' state loads are strided, but the T-vector load uses an index operand. So AOP's
  stated conditions don't match the walk.
- Whether a more general DDP matches it, the documents don't say.

**Clue, not evidence:** #37 gained −10.3% at one thread on the EPYC 9V74 (head `db3593d`) and
−5.8% on the Xeon 6973P-C (head `6c645ef`).
- The two runs differ in head, bench version and runner.
- A hardware prefetcher doing part of the software prefetch's work on the Intel runner is one
  possible reading. Fill-buffer limits (§4) are another.

**Open question:** does software prefetch overlap or interfere with a data-dependent prefetcher
on the Intel runner? That would show as a smaller prefetch-on gain there than on the AMD runner,
in the factorial's prefetch arm. Its cause can only be named with counters.

### Lead 3. Kocberber, Falsafi & Grot, "Asynchronous Memory Access Chaining" (AMAC)

O. Kocberber, B. Falsafi, B. Grot. *Asynchronous Memory Access Chaining.* PVLDB 9(4), 252–263,
December 2015 (doi:10.14778/2856318.2856321), https://www.vldb.org/pvldb/vol9/p252-kocberber.pdf.
The venue is PVLDB volume 9, VLDB 2016's volume.

**Published:**
- **The window limit:** inter-lookup parallelism "is beyond the reach of modern out-of-order cores
  due to the limited size of their instruction window" (abstract).
- **The mechanism:** AMAC keeps each lookup's state separately in a software circular buffer, so
  a new lookup starts as soon as any in-flight one completes, with a prefetch at each stage.
- **What bounds it:**
  - one thread's MLP "is constrained by the number of L1 data cache misses that can be in-flight
    at once";
  - on a Xeon x5670 (10 L1-D MSHRs), ten in-flight lookups were best;
  - beyond about eight, performance showed "little sensitivity";
  - more than 32 in flight "can be harmful due to TLB thrashing on large datasets";
  - across cores, the 32-entry LLC miss queue bounded six threads.
- **Results:** up to 2.3× over group and software-pipelined prefetching on irregular lookups.

**Interpretation:**
- The room walk is an AMAC-like loop with uniform stages: one hop a step, every lane in
  lockstep.
- The hypothesis passed on: fewer instructions a step also let the window hold more lanes' loads,
  so part of #37's gain may be MLP rather than issue slots.
- **The test:** instructions per byte against stall or miss counters, before and after #37.
  - Instructions falling modestly while stalls fall disproportionately supports it.
  - Stalls tracking instructions argues against it.
- It needs a PMU, which the local machine lacks. #40's first step checks the CI runners.

### Lead 4. Lemire, testingmlp

https://github.com/lemire/testingmlp (README, read here).

**Published:**
- "The answer seem to vary between 1 and 25, more or less", for the concurrent memory requests a
  core supports. The benchmark adds pointer-chasing "lanes" until throughput stops growing.
- In the README's one sample output (the CPU isn't named), efficiency over the previous lane
  count falls to 88% at 9 lanes, 64% at 11 and 40% at 12, with a 10.1× speedup at 12.
- The array is RAM-sized (2²⁵ words), and the README warns that "the translation lookaside
  buffer (TLB) is put to work", advising transparent huge pages.

**Interpretation:**
- 32 software lanes don't mean 32 misses in flight. The buffer counts in §4 (16–24) cap them
  lower.
- The README's diminishing returns past 10–12 are one sample machine, DRAM-bound, and TLB-bound.
  The T-vector is L3-resident and fits the STLB (§4), so its knee can be elsewhere.
- The per-processor range is the README's summary. The blog posts behind it weren't read here.

### Lead 5. Prefetch hints, distance, and 1 against 4 threads

**Published:**
- **AMD:** only the Zen 5 guide (58455 rev 1.00, §2.13.1, Table 8) documents the hints.
  - `PREFETCHT0` moves data "into all data cache levels";
  - `PREFETCHT1` into all levels "except 0th level (L1)";
  - `PREFETCHT2` into all levels "except 0th level (L1) and 1st level (L2) caches".

  The Zen 3 (56665) and Zen 4 (57647) guides describe only `PREFETCHNTA`. The lead's "T1/T2 fill
  L2" therefore holds for T1 on Zen 5, while T2 targets L3. No hint semantics are documented for
  the Zen 4 runner.
- **AMD on software prefetch interfering between threads that share caches:** not found in any of
  the three AMD guides. The lead is unsupported by these sources.
- **Intel (248966-051):**
  - on distance (§20.13.1.2, in the AMX GEMM chapter): too close and "the CPU will stall", too
    distant and "the data might be evicted";
  - on saturation (the floating-point tuning procedure): "adding PREFETCH instructions will not
    help if the bus is already saturated";
  - on fill buffers: L1 prefetches hold them, L2 prefetches don't (248966-046, §4 above).

**Ours, measured (prior):** #21's four-copies run reordered the lane counts relative to one walk
(lead 1's table).

**Open question:** does the best hint, or the best lane count, differ between a 1-thread decode
and 4 concurrent ones? At 4 threads the L3 and its miss queues are shared. The factorial runs
both.

### The next experiment, as a small factorial (the coordinator's framing)

It separates three mechanisms:

| Factor | Levels |
|---|---|
| chain interleaving | the lane count, e.g. 8 / 16 / 32 / 48 |
| software prefetch | off, `T0`, `T1` |
| shared-offset scheduling | the room walk (0.1.5) against the flagged walk (0.1.4) |

- **Measured:** the walk alone, at 1 thread and with 4 concurrent copies, every arm byte-checked
  against `bwt::walk`, on as many runner types as CI hands out.
- **Optionally:** instructions and stall or miss counters where a runner exposes a PMU (lead 3,
  #40).
- **Decided by:** the paired whole-decoder A/B at 1 and 4 threads. The walk-only figures pick the
  candidates.
- **Prior negatives:** none of the arms adds per-step work, the cost that sank ψ² (#6, #34) and
  Precopy (#9, #36).

## Mapped onto the issues

| Issue | Finding | Status |
|---|---|---|
| #5 (closed) lane count and prefetch | §1, §2, §4, §5: the lead, buffer counts, the hint, #21's on/off data | the question is reopened on the room walk as **#39**, not on #5 |
| #6 (closed) ψ² | §2: staggering costs O(n²); §4: #29's table against STLB reach | comment with the TLB arithmetic; stays closed |
| #8 memory | §3: a run-length ψ would cut ≈3 MB a slot | noted in #42; no change to #8 |
| #9 (closed) Precopy | §3: a run-length ψ keeps every step and makes the scattered ones cheaper | linked from #42; stays closed |
| new | lanes × hint × placement on the room walk; §5's open questions | **#39** holds the questions; **#41** (the coordinator's sweep tool, being extended to §5's factorial) is the instrument |
| new | the walk's STLB cost, measured; huge pages flagged for the user's decision | **#40** |
| new | a length-capped run-length ψ; feasibility measured above | **#42** |

## Not done, and why

- **Hardware-counter evidence:** no PMU is exposed locally (§4). #40 asks whether the CI runners
  expose one.
- **The CGO 2017 text:** read only through the TOCS version, which extends it.
- **Lemire's per-processor figures:** only the testingmlp README was read, not the blog posts
  behind its "between 1 and 25".
- **The 2004 TOCS framework paper (Choi et al.):** cited, not read.
- **Move-r, RLBWT tricks and Movi:** read only for their role in the SEA paper.
- **`text`, `records`, `random`:** the r/n counter ran only on Wikidata. The synthetic corpora are
  generated by `ab gen` and weren't built here.
- **The scratch counter:** kept out of the repository, and its build output deleted.
