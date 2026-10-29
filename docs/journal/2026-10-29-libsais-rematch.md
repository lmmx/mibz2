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
