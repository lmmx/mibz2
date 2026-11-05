# The walk round of 5 November: results, and what binds the walk

Six decoder PRs ran through the CI bench overnight (#168–#173), and seven local experiments
were run on #171's staged walk. This entry gathers the results and what they imply. Each PR's
own result is appended to its registration:
- `2026-11-05-deferred-flag-walk.md` (#168);
- `2026-11-05-build-run-vectors.md` (#169);
- `2026-11-05-mtf-short-runs.md` (#170);
- `2026-11-05-staged-walk.md` (#171);
- `2026-11-05-mtf-byte-list.md` (#172);
- `2026-11-05-tail-passes.md` (#173).

The bench's comment is overwritten by each new run. `bench-runs/` holds a one-line summary of
every run, transcribed from the comment when the run finished.

## The PRs: instructions against time

| PR | change | `decode_block` Ir | CI, `wikidata` | outcome |
|---|---|--:|---|---|
| #168 | the walk tests its flags once an offset | −2.6% | no gain; 1-thread wall slower in one run | closed, carried into #171 |
| #169 | the build writes a run's 16 entries as four vectors | −2.4% | build stage −3.1%, no decoder gain | closed, not shown |
| #170 | MTF's short runs as two vector stores | −1.5% | wall −1.4% and −2.1% at 1 thread, −1.9% at 4 | merged |
| #171 | the walk's bytes staged in one tile (with #168) | −5.7% | wall −5% to −14% at 4 threads, five runs | merged |
| #172 | MTF's list holds bytes | −0.6% | 4-thread CPU −2.2% in one run | merged |
| #173 | the walk's tail in deferred-flag passes | −2.0% | no gain | closed, not shown |

Three PRs changed no decoder code: #174 (a test, and a guard of one comparison per block),
#175 (the bench's links store) and #176 (the history page). Their benches showed no change.
#175's run is an A/A run on the decoder, and it still flagged two rows by chance: `wikidata`
at 1 thread, +0.8% (+0.0, +1.6), and `text` at 4 threads, +2.3% (+0.9, +3.7). About 42
intervals are read per run.

**Instructions predicted time in two places only:**
- **On MTF's serial chain.** Each block's MTF is one chain of dependent steps, so its
  instructions are on the critical path. Both MTF changes shown faster here were on it.
- **Where the change relieved the memory system.** #171 is the one case.

**Instructions saved inside the walk's loop, or in the build, bought nothing measurable:** #168
and #173 in the walk, #169 in the build.

**The local attribution of #171** (staging alone, `research/staging-only` 7c6bfd30, against
#171; 32 paired rounds a condition):
- staging alone removes 2.24% of block 1's instructions, and the deferred flags a further
  3.45%;
- staging alone carries the whole of the wall-time gain within local resolution. Against #171
  it is 1.0075 (0.973, 1.023) at 1 thread and 1.0026 (0.962, 1.050) at 4;
- on `pattern`, instructions fall 2.8% with staging alone and wall time 25%.

So the gain is a memory-system effect of where the walk's bytes go, not of how many
instructions it runs.

## What binds the walk

The local experiments below point at one resource. This is an inference: neither the VM nor
CI's runners expose hardware counters.

- **Lanes:** from 24 to 64 lanes the walk's time is flat within noise, and 16 lanes is slower.
  With the tail in passes, 48 and 64 lanes run 0.3–0.5% fewer instructions than 32, and
  were still no faster.
- **Prefetch:** dropping the prefetch saves 5% of instructions and costs about 6% of wall
  time at 1 thread. `prefetchnta` costs +33% at 4 threads, most likely because on this Xeon
  it skips L2.
- **Staging:** before #171, the 32 lanes wrote to 32 buffers, so the loop carried 32 store
  streams, each missing a line every 64 bytes. Staging puts those stores in one 8 KiB tile
  that stays in L1, and copies it out in bursts.

The reading that fits all three: the walk is bound by the number of misses a core can keep in
flight, and by their latency. Staging freed the miss buffers the stores held, for the table's
loads. That is also why #171 gained most at 4 threads under SMT, where two walks share one
core's buffers. More lanes add no misses in flight once the buffers are full. Fewer
instructions don't help a loop that waits on misses.

## The local experiments on #171

Each ran on the shared VM against #171's head 23011895, interleaved and paired. These are
indicative only.

| experiment | result | disposition |
|---|---|---|
| tile 128, 512, 1024 (`walk/tile-size`) | Ir flat. 512: −1.9% at 1 thread and −2.0% at 4. 1024: −0.6% and −3.2%. A checker found it weak: per-run means move 2.6 points, and there is a layout confound | CI A/B as #177 (512) and #178 (1024), registered in `2026-11-05-tile-size.md` |
| lanes 16 to 64 (`walk/lanes`) | Ir +0.6% to +5%. Wall flat from 24 to 64; 16 slower | rejected; 32 stays |
| prefetch L2, NTA, off, late | L1 fastest at 1 thread; at 4, L1, L2 and off tie. NTA +33%; late +4.5% Ir and slower | rejected; `PF_L1` stays |
| flush: fixed-size, SIMD and non-temporal copies (`walk/flush`) | the flush is about 1% of Ir. f3 −1.46% Ir, mostly incidental register allocation. No time resolved | neutral; `walk/flush` a28792f5 kept on its branch |
| `runs` at 1 thread on #171: a staging cost? | none found. Aligned layouts give +0.0% (−1.3, +1.4) over 500 rounds; a layout-only rebuild moves it by up to 2.7% | noise at the bench's resolution |
| attribution: staging alone, #171, tail | staging alone ≈ #171; the tail adds nothing detectable | above |

## Also found

- **Half of a block's 142 chunks are a pass's short last chunk.** The room is the smallest
  spare among 32 lanes, so passes end early. Fewer, longer passes would be a structural
  change. It hasn't been tried.
- **A test sized for one tile.** #174's chunk-edge test failed at TILE 1024, because its
  blocks were too short. It now scales with the tile (#177, #178).
- **Shared target directories reuse stale test binaries** across worktrees of the same
  package and version. Every tile-size run reported 256 until `src/bwt.rs` was touched. Give
  each worktree its own `CARGO_TARGET_DIR`.

## Pending the maintainer

Huge pages for the T-vector are an allocator-adjacent change, so they wait for Louis.
