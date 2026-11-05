# 2026-11-05: What a walk segment costs; cross-chain lockstep closed

**The question.** The partitioned-walk entry (`2026-11-05-partitioned-walk.md`) parked
cross-chain lockstep, which needs dense, row-sorted starts, behind a revival gate. The gate
asked for a lean per-segment path of at most about 10 cycles a segment, against today's
230–340 cycles per added segment implied by CI's sweeps. It also asked why a segment costs that
much.

**How it was tested.** All of it was untimed: callgrind with cache and branch simulation, and the
trace simulator of the partitioned-walk entry. The subject is master's room walk with prefetch at
32 lanes and L1 prefetch.
- **Blocks:** bench-corpus block 3 (n 900,000) with a warm scratch, and block 41 as a check.
- **Simulator inputs:** the collision's pinned held-out T-vectors (blocks 1 and 233), with every
  access traced (`sim/pwx.c`, mode `mfull`).
- **A scratch lean prototype** (`drv/src/lean.rs`), byte-identical to the plain walk on blocks 1,
  2, 3, 40 and 41.

Everything is in `2026-11-05-segment-cost/`, pinned in `PINS.sha256` and `INPUTS.sha256`.

## Why a segment costs 260–340 cycles

**Steady per-segment work is about 457 instructions** (OBSERVED, 512 → 2048 segments, blocks 3
and 41 agree):

| part | Ir a segment |
|---|--:|
| binary search mapping a start back to its segment, at finish | 138 |
| sorting the starts | 108 |
| two `memcpy` calls (end-of-segment flush, assembly) | about 82 |
| assembly | 33 |
| handover | 26.5 |
| reserve and `set_len` | 21–29 |
| the rest (random rows, take, flags, flush, bookkeeping) | about 45 |

**What else moves with the segment count.** From 512 to 2048 segments:
- **The tail** (the lanes still walking once no segment is left to take) shrinks. At 512 segments
  it walks 5.7–6.0% of a block at about 17 Ir a byte. Net of the busy phase's rise, it gives
  back 252–276 Ir per added segment.
- **Buffer growth** is stochastic: −159 Ir a segment on block 3, +601 on block 41.
- **A cold scratch** adds 573 Ir a segment. Production keeps its scratch, so this is the bench's
  first block per worker.

**Where the cycles go** (INFERRED, solved against CI's net for the 9V45 and 9V74):
- the steady instructions at the walk's own IPC: 123–181 cycles;
- about 7 mispredicts, mostly `memcpy`'s size dispatch, the search and the handover: about 105;
- about 6.4 extra misses at about 24–26 cycles each. That is half an L3 latency, because they sit
  in dependent chains: the search's probes, the take's start entry, and assembly's
  next → header → buffer;
- the tail's credit: −147 cycles.

Gross is 380–450 cycles a segment: about a third instructions, a quarter mispredicts and two
fifths exposed misses. Net of the tail, 262–338 cycles, which matches CI.

**A harness artefact, not production cost.** The sweep (`examples/sweep.rs`) segments the bench
corpus's 198 blocks under 64 Ki rows. Production walks those plainly (`MIN_SEGMENTED`,
`src/bwt.rs:444`). Block 4 (n 62) costs 78,814 Ir at 512 segments and 226,772 at 2048. That is
about 29 M Ir over the corpus, roughly a tenth of the sweep's measured 512 → 2048 difference. A
sweep row that compares segment counts carries this bias.

## The lean path, and the gate

**The prototype.**
- Starts are set by the build at zero cost: row r starts a segment when r & (K−1) == 0. There is
  no flag pass, no sort and no search.
- Segments are taken in row order (the dense, row-sorted starts).
- A lane tests the link it just read.
- Output goes to one arena in 8 KiB tile dumps.
- Each segment gets an 8-byte record.
- The join is a second-level walk over the records.

**Density is forced to K ≤ 8** (OBSERVED, in simulation). Lockstep reuse needs adjacent lanes
within one 64-byte line of entries. T-vector misses a byte, SMT pair, 1 MiB L2:

| K | T misses/B |
|---|--:|
| 8 | 0.476 |
| 16 | 0.822 |
| 32 | 0.836 |
| master, 512 segments | 0.863 |

At K = 16 the lean walk is already worse than master in total misses: 0.997 against 0.919.

**Its cost** (OBSERVED, callgrind, block 3):

| | Ir/B |
|---|--:|
| K = 4 | 39.93 |
| K = 8 | 25.37 |
| K = 16 | 18.13 |
| K = 32 | 14.59 |
| master, 512 segments | 10.74 |

The fit is 10.9 Ir/B + 108 Ir a segment (0.4% misfit): the walk 37.3 and the join 70.6.

**The gate fails on both terms.**
- **Per segment.** It costs 33–64 cycles: 108 Ir at 0.27–0.53 cycles each, plus 1.9 join misses
  at 2.2–3.7 cycles. The gate asked for about 10, so it is 3.3–6.4× over. An idealised floor of
  about 30–35 Ir a segment still gives 12–25 cycles. The miss saving pays for only about 7–15 Ir
  a segment.
- **Whole walk, SMT-pair cell.** At K = 8, misses fall 0.157 a byte, worth 0.35–0.58 c/B.
  Instructions rise 13.7 a byte, costing 3.7–7.2 c/B. Net: 48–107% slower than the four-copy
  walk. Even the best case, lean2's misses with the idealised instruction count, gives −0.16 to
  +0.38 c/B, which is not a reliable win.
- **Why it cannot be repaired.** The join is a random permutation of n/K pieces, so about one
  random line a segment returns in the join. And K ≤ 8 makes that one per 8 bytes or fewer.

**Cross-chain lockstep is closed on CI.** The partitioned-walk entry's revival gate is not met.
Partitioning stays rejected.

## What the anatomy opens

**The tail's cost is latency, not instructions.** Tail passes (#173) cut the tail's instructions,
about 2% of decoder Ir, and their effect was not shown. The anatomy's tail model (`tailsim.py`,
the census fit t = a + lat/min(L, M), L the live lanes) prices the tail differently:
- at 512 segments the tail costs 0.20–0.45 c/B beyond the busy phase's rate. Against the walk
  alone at 2.89–4.27 c/B, that is 5–16% of it (INFERRED). The model was not split by CPU;
- the cost is lost memory-level parallelism, as live lanes drain from 32 towards 1;
- it is largest alone and at one thread a core, where M is largest. That is wikidata-pq's
  configuration.

Today's only lever on the tail is more segments, at 262–338 cycles each. A construction that
keeps the lanes full in the tail, at a lower price than a segment, is the next front.
