# Compact Ψ after #112: a feasibility review (research only; no code, no runs)

- **The question** (the user's and the researcher's, 2026-10-30): could a smaller ψ table
  keep the cache benefit #112 measured, without the per-byte cost that failed it? In
  particular, could per-32-row F-symbol descriptors replace the hot loop's symbol
  recovery? The two-tier formats with staged (AMAC-style) exception loads are a separate,
  more speculative question.
- **The target stays the complete decode of the existing full-size Wikidata `.bz2`,** at
  level 9. It is compared with the shipped cooperative decoder, CPU and peak RSS included.
- **Method:** reading #112's code, journal and grid; the succinct-structures review and its
  ψ census; the cooperative decoder's code and journals; and wikidata-pq's own use of the
  decoder. Nothing was run or built.
- **Sources** (most are on research branches, not master):
  - #112: [the journal](https://github.com/lmmx/mibz2/blob/research/compact-psi/docs/journal/2026-10-29-compact-psi.md)
    and [`examples/coop/compact.rs`](https://github.com/lmmx/mibz2/blob/research/compact-psi/examples/coop/compact.rs);
  - [the succinct review](https://github.com/lmmx/mibz2/blob/research/psi-census/docs/journal/2026-10-29-succinct-review.md)
    (§6–§9, the census);
  - the rematch and knee journals (#107);
  - `2026-10-28-workstation-scaling.md` §9 and `2026-10-29-cooperative-decoder.md`.

## Verdict

1. **The 32-row symbol descriptor is what #112 measured, not an alternative to it.**
   - #112's link-only arms recover the byte from `win[row >> 5]`: one byte per 32-row
     window, 28 KB a block. A step past a bucket boundary is needed for 0.15% of rows.
   - So the measured cost is the descriptor's own always-paid part, not the rare boundary
     case.
   - The variants still untested are narrower, and #112's own §5.4 already lists them.
2. **On the i9 the payoff is bounded and small.**
   - The shipped decoder's four teams already keep their tables in the cache. A smaller
     table pays only by adding a fifth team, on the last two physical cores.
   - That bounds the inversion at about 1.25×, and the complete decoder at about 1.1–1.15×,
     before the per-step cost any compact format adds (§5).
3. **Not approved for implementation.**
   - The remaining compact-Ψ science is #130's `e` and `h` arms, already designed.
   - The largest gains for Wikidata found during this review are elsewhere, and are
     larger (§6):
     - wikidata-pq was still on mibz2 0.1.4;
     - it caps the decoder at 4 threads, so 0.2.0's cooperative decoder has never run in
       the release;
     - the shipped decoder has a measured idle share it doesn't yet use.

## 1. What #112 established (measured, the i9, 20 paired rounds)

**The science held.**
- On the same blocks, a 24-bit link-only table (2.7 MB against 3.6) moved the knee from 4
  blocks to 6.
- The L3 hit share followed live table bytes within 0.031 at every N.
- The control `v` (the same bytes as production, the byte recovered as `u` does) had
  production's hit share, not `u`'s. So the move belongs to the bytes.

**The costs** (#112 §5.3, the walk and build split):

| comparison | what it isolates | walk | build | whole |
|---|---|--:|--:|--:|
| v1/o1 | the byte from F (and a store-only build) | **0.721** | 1.279 | 0.829 |
| u1/v1 | the 24-bit packing | 0.830 | 0.786 | 0.821 |
| v4/o4 | the byte from F, 4 blocks | **0.839** | 1.490 | 0.950 |
| u4/v4 | the packing, 4 blocks | 0.969 | 0.823 | 0.940 |

- **Under memory pressure** (4 blocks), recovering the byte costs 16% of the walk, and the
  packing 3%.
- **The build is a second front.** Storing the link alone made `v`'s build 1.28–1.49×
  faster. That gain shipped as #114. The 24-bit layout's narrow store at each bucket end
  and its 3-byte stride gave it back.
- **Unexplained:** `u` makes 26% more reads past L2 per byte than `p` at 4 blocks. Loads
  that cross a line explain about 5 points of that.

## 2. Q1–Q2: does a 32-row symbol descriptor remove the hot-loop work?

**What #112's arms already do** (`compact.rs`, `f_byte`):

```
c = win[row >> 5]                  // the window's first byte: n/32 bytes, 28 KB a block
while row >= cstart[c + 1] { c++ } // past a bucket boundary inside the window
```

- **The descriptor is aligned, per 32 rows, and holds the window's symbol.** That is the
  researcher's proposal, already built.
- **The boundary case is rare, as the theory says.**
  - The census (succinct review §9, the bench corpus's 233 full blocks): 0.29% of rows
    are in mixed windows, and 0.15% take the step.
  - 93% of mixed windows hold one boundary.
  - The ≤255-boundary argument is right, and it was already being exploited.
- **What every step pays:** a load of `win`, then a load of `cstart[c + 1]` that depends on
  it, a compare and a branch.
  - That is two L1 loads in series and a few µops.
  - It is all off the link chain, as `row` is known before its own entry is loaded.
- **So the measured 16–28% walk cost is this always-paid part.**

**Why off-chain work costs at all** (analysis, not measured):
- The walk interleaves 32 lanes (`LANES = 32`, the room walk).
- Its throughput at a block in L3 is set by how many lanes' loads the core can keep in
  flight.
- Extra µops per step use the reorder window (224 entries on this core), and fewer misses
  then overlap.
- This is the leading reading, not a finding. #112's counters didn't separate the
  pipeline's stall classes.

**What a cheaper descriptor could still remove.** These are the untested variants, all
already in #112 §5.4:

| variant | per-step work off the chain | extra bytes a block | removes |
|---|---|--:|---|
| #112 (`win` u8 + `cstart`) | 2 dependent L1 loads, compare, branch | 28 KB | — |
| `win` u16: byte + "boundary in window" flag | 1 L1 load, test, branch (predicted not taken in 99.7% of windows) | 56 KB | the `cstart` load in the common case |
| 64- or 128-row windows | as above | 14 or 7 KB | L1 pressure |
| the byte in the link's own line (`e`, #130) | none | 0 | the separate structure |

- **Footprint is not the issue.** Any descriptor here is 7–56 KB against a 2.7 MB table:
  under 2% of the working set.
- **The per-step work is the whole question.**
- **The floor is set by production.**
  - Production's entry (`link << 8 | byte`) gets the byte from the same load as the link,
    at zero extra cost.
  - A separate descriptor can't beat that. It can only come closer, at one load and a test
    per step.
  - The only designs with no extra load put the byte in the link's own entry or line:
    production's 32 bits, or a record format (§3).

## 3. Q3: the formats already studied (sizes for a 900 K-row block)

| design | size | per step | status |
|---|--:|---|---|
| production, 32-bit (link + byte) | 3.60 MB | 1 load, a shift | shipped |
| `u`: 24-bit link + F window | 2.70 + 0.03 MB | 1 load (unaligned, masked) + 2 L1 loads off the chain | **measured: fails stage B** (u1/o1 0.681) |
| 20/21-bit dense packing | 2.25–2.36 MB | unaligned extract | **measured in #91's `c` arm: 0.15× the build; ruled out** |
| 64-byte line records (FOR-14 / EF-12 / exceptions), 16.12 bits a row | 1.81 MB | 1 line fetch, a decode on the chain (about 15 cycles, the guardian's estimate); byte from the header | census: 98.1% of rows FOR-14, 0.49% exceptions; **untimed (#130 `h`)** |
| two-level: 8-byte descriptor per group + payload for non-run groups, about 5 bits a row | about 0.56 MB | 1 load on the 77.6% of steps in run groups; a **second, dependent** load on about 22% | **hypothesis only** (succinct review §9) |
| blocked rank over L | L + 1–4 B/row | rank, LF-oriented | **ruled out for the ψ walk** (§6: the walk needs select, not rank) |

**What the census gives the record formats:**
- 77.6% of rows sit in "run" groups (span 31: 32 consecutive positions), where
  ψ(j) = base + (j & 31). That is one add after the fetch, cheaper than today's shift.
- The cost that remains is a format test and branch on every step, and the decode latency
  added to the chain on the rest.
- Neither has been timed. That is what #130's `h` arm, with its `e` control, is for.

## 4. Q4: AMAC, and the cooperative decoder's fit

**The walk already overlaps independent lanes.**
- The room walk round-robins 32 lanes per block, each step prefetching its lane's next
  entry to L1. That is a static form of what AMAC generalises.
- AMAC's extra lever is a per-lane state machine for chains of varying length. Only the
  two-level format has those: a second dependent load on about 22% of steps.
- For #112's design and the line records, every step has the same shape, so AMAC adds
  nothing.
- So, as the researcher said, AMAC doesn't repair #112. It is relevant only after a
  two-level format exists and its stall profile is measured.

**The cooperative decoder imposes no hard incompatibility, but it makes a format's build a
different algorithm:**
- **The walk:** teams split a block's segments among threads (`coop::walk_room`). A
  format change lives inside the step, so the walk carries over.
- **START flags:** the walk ends segments at flagged rows. A record format needs a bit per
  row for them; the EF line's 32-bit start mask is that bit.
- **The parallel build** (`coop::build_fused`) scatters rows into (thread, bucket) ranges
  that aren't aligned to anything.
  - A 32-row record must be written by one owner, so ranges would be cut at line
    boundaries. Groups crossing buckets are already the exception class.
  - Since a bucket's rows are filled in increasing order, each bucket cursor can buffer
    its 32 links and encode a record when full: about 32 KB of buffers per thread.
  - This "direct construction" is the succinct review's §15 hypothesis. It is untested.
  - The build is where `u` lost `v`'s gain (§1), so it would need its own measurement.

## 5. Q5: could it compete with the shipped decoder on Wikidata?

**Measured (the i9):**

| figure | value | source |
|---|--:|---|
| shipped kernel, 4 teams × 2, build + walk, pinned | **2,345 MB/s** | #112 grid, `l4x2` |
| best compact arm, 6 blocks × 1 core | 1,615 MB/s (0.687× `l4x2`) | #112 grid, `u6` |
| 5 teams × 2 of full tables | 0.746× `l4x2` (cache overflow) | #112 grid, `l5x2` |
| shipped kernel with its MTF stage timed, 4 × 2 | 1,707 MB/s | rematch pass 2 |
| a 4 × 2 team's core-time idle while its caller runs MTF | 0.11 (upper bound) | rematch |
| the complete decoder, the default at 20 threads (4 × 2) | **1,256 MB/s** | release run, 0.2.0 |
| 10 teams × 2 on SMT siblings, at 20 threads | 0.838× the 0.1.6 4-worker reference (the policy: 1.257×) | cooperative-decoder journal |

**What bounds a compact table here:**

1. **The four teams' tables already fit.**
   - 4 × 3.6 = 14.4 MB sits at a hit share of about 0.93 on the measured curve.
   - A smaller table barely speeds up the blocks already running. In #112, `u4` ran at
     0.899× `p4`, at a higher hit share (0.971).
   - Its gain is more blocks at once, not faster blocks.
2. **The gain stops at five teams on the i9.**
   - Teams of 2 use 8 of the 10 physical cores.
   - Teams on SMT siblings lose (0.838× the reference, against the policy's 1.257×), as the
     walk's memory parallelism is per core.
   - A fifth team is the most a smaller table can add: at most 5/4 = 1.25× on the
     inversion.
   - That is before any per-step cost, and before the second limit near 10 cores. At -3,
     the knee turned at 8–10 blocks with the hit share still at 0.94.
3. **The complete decoder dilutes it** (modelled).
   - The inversion is part of a team's time. The MTF stage runs on the team's caller while
     its helper waits (`decode_block` calls `mtf_checkpoints`, then `invert_checkpointed`).
   - The decoder runs at 1,256 MB/s against the kernel-with-MTF's 1,707.
   - So a 1.25× inversion is about 1.1–1.15× of the whole. This assumes the inversion is
     60–65% of a team's wall, an estimate from these figures, not a measurement.
4. **A cross-check from block size.**
   - At -3, with 1.2 MB tables (the regime a radical shrink would imitate), the whole
     decoder's best was 1,729 MB/s, and -1 plateaued at about 1,460 where the scanner
     binds (workstation §9, 0.1.x workers).
   - Taken at face value, even small tables at no cost per step bound the gain at about
     1.38× over today's 1,256.
   - Those blocks also carry advantages a compact -9 table wouldn't. So this is a loose
     upper bound, from older code.

**Elsewhere** (not measured; a reason to keep the science, not to build now):
- The bound is the i9's: a 20 MB L3 shared by 10 cores.
- On a machine with less L3 per core, the knee falls below 4 tables, and a smaller table
  matters more. Below 6 physical cores the plan is workers, so that knee binds directly.
- An ordinary laptop is the release's target machine, and no such machine has been
  measured.

## 6. Found during the review: what limits the existing Wikidata decode now

1. **wikidata-pq's reshape extension pinned mibz2 0.1.4.**
   - None of 0.1.5–0.2.1 reached the release.
   - lmmx/wikidata-pq#213 bumps it to 0.2.1. Its bench decides.
2. **wikidata-pq runs the decoder on at most 4 threads** (`bz2_threads()`: one per
   physical core, capped at 4; wikidata-pq #212).
   - At 4 threads mibz2 plans workers, so 0.2.0's cooperative decoder has never run in the
     release.
   - The cap was measured against 0.1.x's workers, whose extra threads thrashed the L3.
     Teams keep only one table per team live, so the trade-off is new.
   - In the release-shaped run on the i9, the split was 25 s of 99 s, and its saving
     carried through to the end of the release (wikidata-pq
     `2026-10-28-bz2-threads.md`).
   - The question is whether 8 threads (4 teams of 2) beat 4 workers there, beside the
     split's parse workers. It is a wikidata-pq measurement, not a mibz2 change.
3. **Inside the shipped decoder,** two measured gaps need no change of representation:
   - **The helper's idle share during MTF:** up to 0.11 of a 4 × 2 team's core-time. The
     rematch journal's "batched team" overlaps one block's MTF with another's inversion.
   - **The kernel-with-MTF at 1,707 MB/s (pinned) against the decoder's 1,256
     (unpinned):** not attributed. The scanner, RLE1 and CRC on the caller, placement and
     the reader are the candidates.
   - The tool that would attribute it is the occupancy trace. It collects no complete
     round under a team plan: it waits for one thread's marks per decoder thread, plus
     one (the level-aware journal's diagnostics). Fixing that collector is small, can be
     checked on CI by forcing a team plan, and is a precondition for attributing either
     gap.

## 7. What is established, unknown and ruled out

- **Established:** a smaller table alone moves the knee on Wikidata's full blocks (#112).
  #112's descriptor-based byte recovery costs 16–28% of the walk. Footprint is not the
  descriptor's problem; its per-step work is.
- **Unknown:**
  - whether a one-load descriptor (u16 byte + flag) or a byte embedded in the link's line
    (`e`) comes close to production's step;
  - how fast the line records decode (`h`);
  - whether off-chain µops limit the walk's memory parallelism (counters with the stall
    classes would say);
  - all of it on a laptop-class cache.
- **Ruled out:** dense 20/21-bit packing (measured); blocked rank for the ψ walk (wrong
  direction); `u` as built (stage B gate); AMAC as a repair for #112 (nothing to overlap).
- **Order, if this is resumed:**
  1. the wikidata-pq bump and its thread-cap question, and the trace collector under
     teams. These are cheap and could move the release's decode the most;
  2. then the batched team;
  3. #130's `e` and `h` arms only after those, pre-registered against `l4x2` and the
     complete decoder, not against `p4`.
