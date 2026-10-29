# The libsais rematch: the shipped cooperative inverse BWT against libsais's 2,356 MB/s

**Status:** pre-registered on 2026-10-29, before any run. The results go below, under
their own heading.

## Where 2,356 MB/s comes from

The figure is libsais's `s3x3` arm in the cooperative-inversion journal's §7.1
(research/coop-fused, `docs/journal/2026-10-28-cooperative-inversion.md`):
- **Machine and placement:** the i9-10900K, every thread pinned to its own physical core.
- **Harness:** `examples/coop.rs grid` through `bench/coop.sh`, 38 rounds.
- **Workload:** the bench corpus's 431 Wikidata blocks (209.7 MB of BWT).
- **Boundary:** the T-vector phase only, build + walk. Each block's last column is made
  before the timing (the scanner and MTF are outside it).
- **libsais:** 2.12.1 at c284ba8, `libsais_unbwt_cyclic_ctx` on an OpenMP context of k
  threads, given the block's byte counts.

§7.1's medians at that boundary: p4 1,412, 4x2 1,736, 3x3 1,912, **s3x3 2,356** MB/s.

The ~2 GB/s remembered for mibz2 is the research kernel's 3x3 at 1,912 there.
- **That is not the decoder:** v0.2.0's full-decoder figures (1,050–1,256 MB/s on the i9)
  include the scanner, MTF, RLE1, CRC and the output.
- **Neither result scales the other:** the two figures measure different boundaries.

## What is compared

The same harness, at v0.2.0 (1140793) plus this branch's harness changes. One new arm kind:
- **`lNxK`** is the library's cooperative inverse BWT as v0.2.0 ships it, called through
  `InverseBwt::invert_checkpointed`. That is `bwt::Cooperative<K>`, with:
  - its own pool of K − 1 helpers;
  - the fused build with its window checks and end check;
  - the room walk, 512 × K segments.

  Its worker is pinned to its k CPUs together, as libsais's is. The library spawns its
  helpers on the worker's thread, so they inherit those CPUs. The harness checks, untimed,
  that the checkpoints it gives an `l` arm are exactly the library's own MTF's on every
  block, so its build never falls back.

The research arms are unchanged:
- `p` is the production one-core inversion, the old decoder's kernel (build +
  `walk_segments_room`);
- `NxK` is §7.1's cooperative research kernel (two-pass build, plain cooperative walk);
- `x` is the research port of the fused build (#81), checkpoints given;
- `y` is `x` with its checkpoints made in the MTF stage (`mtf_cp2p`), with `COOP_WITH_MTF=1`
  only;
- `s` is libsais.

**Pass 1, §7.1's boundary** (build + walk; the MTF and the checkpoints made outside the
timing): `p4, 4x2, 3x3, x4x2, x3x3, l4x2, l3x3, s4x1, s4x2, s3x3`.

**Pass 2, the decoder's boundary** (`COOP_WITH_MTF=1`: each arm's MTF stage in its timing):
`p4, y4x2, y3x3, l4x2, l3x3, s4x1, s4x2, s3x3`.
- **The arms' MTF stages:**
  - p and s arms run production's `mtf::mtf`. libsais takes no checkpoints, so it isn't
    charged for any.
  - y arms run `mtf_cp2p`.
  - l arms run the library's `mtf::mtf_checkpoints`.
- **How this pass serves libsais:** the harness used to refuse s arms here. It now gives
  them production's MTF, the fastest one-core MTF we have.

Both passes: 20 rounds, the arms' order rotated and reversed by round, every arm's every
block checked by its hash in an untimed first round.

## How it will be read (fixed now)

1. **Reproduction first.** Pass 1's p4, 4x2, 3x3 and s3x3 are §7.1's arms, unchanged.
   - **The bar:** their ratios to p4 within about ±5% of §7.1's (4x2 1.226, 3x3 1.354,
     s3x3 1.629).
   - **Absolute MB/s can differ:** today's runs on this machine have been slower than the
     morning's (p4 at 863 against 1,007 in the full decoder). Ratios to p4 are the
     comparison across runs; absolute MB/s only within a run.
   - **If the ratios don't reproduce,** that is the finding, and nothing below is read as a
     rematch.
2. **Matched budgets, within a pass, paired by round:**
   - **8 cores:** l4x2 against s4x2 (and x4x2).
   - **9 cores:** l3x3 against s3x3 (and x3x3).
   - **4 cores:** p4 against s4x1.
   - Each pair's paired ratio and its 95% interval.
3. **Each implementation's best:** the fastest arm of each kind in the pass, by median MB/s,
   with its cores. It is reported beside the matched pairs, never in place of them, and the
   cores are always named.
4. **The question, "have we caught 2,356?":**
   - **Within the run:** the best l arm's ratio to s3x3 in pass 1, with its interval.
   - **Against the historical figure:** the best l arm's median MB/s next to 2,356, marked
     as across runs.
5. **The library against its research twin:** l against x at the same N×k, pass 1. This
   is the price of the library's soundness: the window checks, the end check and its own
   pool (about 7–8% of the build on the VM, unmeasured on the i9). It is pass 1's other
   result.
6. **Pass 2's question:** whether including the MTF stage changes the order. The MTF is
   one core per block for every arm, so the cooperative arms' helpers idle through it.
   That idling is the cost the batched team (overlapping MTF with inversion) would
   remove.

**Not read from this run:**
- the decoder's throughput;
- the counters (instructions, fill buffers). Those come from `bench/coop_counters.py` on
  the l arms, if asked for;
- any machine but the i9.

## The command

On the i9, at this branch's head, with the browser and other busy programs closed:

```sh
git fetch origin research/libsais-rematch
git checkout --detach origin/research/libsais-rematch
just bench-libsais-rematch
```

It writes two `coop-<UTC time>.md` files (one a pass) and their `.jsonl`. Upload all four.

## The result (measured, the i9, 192fca6, 2026-10-29)

**The run:**
- **Command:** `just bench-libsais-rematch` on the i9-10900K, at this branch's head 192fca6.
- **Pass 1** (build + walk): [`coop-20261029T183303Z.md`](2026-10-29-libsais-rematch/coop-20261029T183303Z.md), load 1.6.
- **Pass 2** (MTF + build + walk): [`coop-20261029T183353Z.md`](2026-10-29-libsais-rematch/coop-20261029T183353Z.md), load 4.5 at its start.
- **Raw files:** both `.jsonl`, beside the reports.
- **Correctness:** every arm's every block matched the room walk's bytes, and the library's
  checkpoints were the harness's on every block. No thread was misplaced in any round.

All ratios below were recomputed from the raw rounds, paired by round over 20 rounds: the
geometric mean, its 95% t-interval, the median ratio, and the rounds won.

### 1. Reproduction: passes

| arm against p4 | §7.1 | today | within ±5% |
|---|--:|--:|---|
| 4x2 | 1.226 | 1.213 (1.189–1.238) | yes |
| 3x3 | 1.354 | 1.356 (1.329–1.383) | yes |
| s3x3 (libsais) | 1.629 | 1.602 (1.480–1.733) | yes |

- **Absolute figures:** p4 ran at 1,355 MB/s against §7.1's 1,412 (−4%), the slower machine
  state seen all afternoon.
- **One bad libsais round:** s3x3 had one round at 1,079 MB/s against its usual 2,200–2,430,
  hence its wide interval. The median ratios below are untouched by that round.

### 2. Pass 1, §7.1's boundary (build + walk): the shipped kernel beats libsais at every budget

| matched budget | pair | paired ratio (95%) | median | won |
|---|---|--:|--:|--:|
| 9 cores, 3 blocks | **l3x3 / s3x3** | **1.114 (1.027–1.208)** | **1.081** | 19/20 |
| 8 cores, 4 blocks | l4x2 / s4x2 | 1.343 (1.303–1.385) | 1.368 | 20/20 |
| 4 cores, 4 blocks | p4 / s4x1 (one core a block) | 0.930 (0.909–0.952) | 0.929 | 2/20 |

- **Each implementation's best:**
  - the library at **l3x3, 2,430 MB/s** (median, 9 cores);
  - libsais at **s3x3, 2,260 MB/s** (9 cores).
- **Against the historical 2,356 MB/s** (across runs, so indicative only): l3x3's 2,430 is
  above it, on a machine whose p4 ran 4% slower than then.
- **On one core a block, libsais still wins:** s4x1 beats our one-core inversion (p4) by
  7.5%. That is where its ψ² walk's fewer reads per byte show (coop journal §10.4). The
  library's lead comes from cooperation, not from a faster one-core kernel.

### 3. The library against its research twin

| pair | paired ratio (95%) | median |
|---|--:|--:|
| l4x2 / x4x2 | 1.001 (0.976–1.025) | 0.999 |
| l3x3 / x3x3 | 1.072 (1.051–1.094) | 1.082 |

- **The predicted cost doesn't show:** the library's soundness checks (window checks, end
  check, its own pool) were predicted to cost about 7–8% of the build. At 4x2 the library
  equals the research kernel; at 3x3 it is 7% faster.
- **Why 3x3 is faster isn't measured.** Candidates are the pools (the library's caller and
  helpers park; the research pool spins between jobs), and the library taking each block's
  pool and segment scratch from its own worker state. This is a lead, not a finding.

### 4. Pass 2, the decoder's boundary (MTF + build + walk): a tie with libsais at 9 cores

| matched budget | pair | paired ratio (95%) | median | won |
|---|---|--:|--:|--:|
| 9 cores | **l3x3 / s3x3** | **1.022 (0.986–1.059)** | **1.003** | 11/20 |
| 8 cores | l4x2 / s4x2 | 1.218 (1.182–1.255) | 1.203 | 20/20 |
| 4 cores | p4 / s4x1 | 0.939 (0.924–0.954) | 0.933 | 2/20 |
| | l3x3 / y3x3 | 1.065 (1.051–1.080) | 1.070 | 19/20 |
| | l4x2 / y4x2 | 1.008 (0.992–1.025) | 1.000 | 10/20 |

- **Medians:** l3x3 1,731, s3x3 1,724, l4x2 1,707 MB/s.
- **The MTF stage takes away the 3x3 lead.**
  - l3x3 against l4x2 is 1.158 (1.128–1.190) in pass 1, and 1.014 (1.000–1.029) in pass 2.
  - The MTF is one core a block. While it runs, a team's other threads wait: 0.19 of a
    3x3 team's core-time and 0.11 of a 4x2's (the workers' MTF and inversion seconds in
    the raw rounds).
  - libsais pays the same in its s3x3 arm (0.19). Neither implementation overlaps the MTF
    with inversion.
- **What would remove that wait** is the batched team, overlapping one block's MTF with the
  inversion of another. That item now has a measured size: up to 0.19 of a 3x3 team's
  core-time at this boundary, and 0.11 of the 4x2 the policy ships. It is an upper bound
  on the cores' idle share, not a throughput gain.

### What it answers, and what it doesn't

- **Yes, at the boundary 2,356 MB/s was measured at:** v0.2.0's cooperative inverse BWT
  beats libsais 2.12.1's multi-threaded inversion on the same machine, blocks and core
  budget. At 9 cores the median paired ratio is 1.081 (geometric mean 1.114, 1.027–1.208);
  at 8 cores it is 1.343.
- **A tie at the decoder's boundary, at 9 cores.** At 8 cores, the plan the policy ships,
  the library leads libsais by 1.218.
- **libsais still has the faster one-core kernel** (s4x1 over p4: 1.075).
- **Not shown here:**
  - the decoder's throughput: this is the inversion with or without the MTF stage, never
    the scanner, RLE1, CRC or output;
  - other machines;
  - the counters on the shipped kernel. The instruction-bound reading in the coop
    journal §10.4 is for the earlier research kernel (two-pass build), not this one.

**What follows:**
- **The batched team (MTF overlap) is the next experiment, sized here.**
- **The policy question:** at the decoder level, 3x3 and 4x2 tie (1.014). The full-decoder
  A/B's 3 × 3 lead (lower RSS at equal throughput) is consistent with that.
- **The one-core gap to libsais** (ψ², two bytes a step) stays open, conditional on an
  instruction budget (coop journal §10.4).

## Follow-up, pre-registered: why l3x3 beats x3x3, and the shipped kernel's counters

**1. The 7% between the library and its research twin** (l3x3 / x3x3 = 1.072,
1.051–1.094). What differs, from the source, with no cause assumed:

| | research `x3x3` | library `l3x3` |
|---|---|---|
| placement | each thread pinned to its own CPU | the worker pinned to its 3 CPUs together; its helpers, spawned on its thread, inherit them and float within them |
| pool waits | 2,000 spins, then park | the same |
| segments | 512 × k | 512 × K |
| build | `coop_build_fused` | `coop::build_fused`, plus window checks and the end check |
| instrumentation | build and walk timed apart; per-thread start and end times; walked bytes | none inside the call |
| compilation | in the example's crate | in the library's, called across the crate boundary |

**The test:** a new arm kind, `g`. It is the x kernel placed as the library places its
threads: the worker and every helper pinned to the group's k CPUs together, unchecked.
Run with `bash bench/coop.sh --arms x3x3,g3x3,l3x3,x4x2,g4x2,l4x2,s3x3`, 20 rounds.

**The reading, fixed now:**
- **Placement explains it** if g3x3 / x3x3 is above 1 (its interval excluding 1) and
  l3x3 / g3x3 is within 1 ± 0.02 (its interval including 1).
- **The code explains it** if g3x3 / x3x3 includes 1 and l3x3 / g3x3 excludes it. The
  symbol profile (below) then says where: build, walk or assembly.
- **Anything between is a split,** reported as measured.
- **The 4x2 arms are the control:** l4x2 and x4x2 tied in the rematch (1.001), so g4x2
  should tie too.

**2. The counters on the shipped kernel.** The coop journal's §10.4 counters (09:30 today)
were for the research harness's earlier kernel (the two-pass build with its recount). So
"the teams are instruction-bound, not fill-buffer-bound" is a hypothesis for v0.2.0's
kernel, not a finding.

**The run:** `bench/coop_counters.py` as registered in the coop journal's §8.2 and §10:
- each arm alone under `perf stat`, 1 and 21 rounds, 3 repeats, one counter pair a pass,
  the memory controller machine-wide;
- arms p4, x3x3, g3x3, l3x3, l4x2 and s3x3;
- then `perf record` of l3x3, g3x3 and s3x3, by symbol in user cycles. That splits the
  library's build from its walk, which its arm otherwise times as one.

**The reading:**
- **The hypothesis holds for the shipped kernel** if its arms meet the fill buffers full
  well below p4's rate (§10.4: p4 0.24 a cycle, the old teams 0.07–0.08), at a similar
  wait per read past L2, with instructions per byte above p4's.
- **Then the levers are instructions,** not more misses in flight: AMAC-style scheduling
  is not indicated for the 32-bit table (docs/journal/2026-10-29-amac.md).
- **If instead the fill buffers are full** as often as p4's, the walk is memory-parallelism
  bound, and lane count and scheduling come back in.
