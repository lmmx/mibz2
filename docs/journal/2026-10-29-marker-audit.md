# The marker audit: do segment ends need per-row bits in the hot lookup? (#100)

Part of #79. The compact ψ descriptor proposed there is 64 bits per aligned 32-row group:
32 marker bits, a 20-bit base, the group's output byte (8 bits) and a 4-bit tag. Without
the marker bits, a two-plateau group (two 20-bit bases, a 5-bit split, the byte and the
tag: 57 bits) fits in one word. This audit asks whether the walks need those 32 bits. It is
analysis only: nothing in `src/` changed.

Read at master 918f788 (`src/bwt.rs`) and `research/coop-fused` cccdac6
(`examples/coop.rs`, PR #95). `research/coop-checkpoint` (#78) has the same walk: its
`START` sites are `examples/coop.rs:597, 635, 773, 821, 857` there.

Labels:
- **verified in code:** read in the source at those commits;
- **simulated:** an untimed Python model of the segmentation on this VM (method in §6);
- **hypothesis:** untested, and to be measured.

## Verdict: Pass

Segment ends can be found with **1 bit per group** (a fixed slot for the whole block) or
**6 bits per group** (a flag and a 5-bit slot). Neither adds a load, a per-step
shared-state update or any cross-lane synchronisation. The test reads the descriptor the
step loads anyway, as the flag in bit 31 does today.

A two-plateau group with the 6-bit form is 57 + 6 = 63 bits, so it still fits one word.
The census (#99) decides whether that is worth a prototype.

**The invariant.** Every segment start is the slot-s row 32·g + s of a flagged group g,
with at most one start per group. r0 is among the starts. A lane that arrives at row r
ends its segment if and only if `desc[r >> 5].flag` is set and `r & 31 == slot`. The slot
is the block's s = r0 & 31 in the 1-bit form, and the group's own 5-bit field in the
6-bit form.

## 1. Can the starts be aligned? What `random_rows` gives the walk

**Verified in code:**

- **Correctness needs only a set of distinct rows that contains r0.**
  - The argument is `walk_segments`'s doc, `src/bwt.rs:234-245`. Every segment ends,
    because its own start is on its own cycle. The lanes take at most n steps in all. The
    join from r0's segment yields the first n steps of `walk`. Starts that are not on
    r0's cycle are walked but never joined.
  - Nothing in that argument uses how the starts were chosen. The tests run with 1
    segment (r0 alone) and with 2 (`src/bwt.rs:969, 1191`), and they agree with `walk`.
- **Distinctness comes from sort and dedup, not from `random_rows`** (`src/bwt.rs:620-624`).
  - `ns` can be below `segments`.
  - The buffers' capacity follows `ns`: `cap = 2 * n / ns + 64` (`:629`).
- **r0 is pushed explicitly** (`:622`). The join needs r0 as a start: `s.first =
  seg_of(r0)` (`:794`), and `assemble` begins there (`:210-222`).
- **What `random_rows` itself provides** (`src/bwt.rs:874-884`):
  - The rows are xorshift64 output modulo n, seeded by n, so a block gets the same rows
    every time.
  - The doc's reason for random rows (`:234-238`; bz2k's `random_rows` doc,
    `docs/journal/2026-10-27-compute-frontier/kernels/bz2k/src/lib.rs:733-735`): a
    uniformly random row is a uniformly random text position, so the segments are about
    equal in expectation. Evenly spaced rows clustered in the text
    (`2026-10-27-compute-frontier.md:300`).
  - **It guarantees nothing more.** There is no spacing bound and no coverage of r0's
    cycle. In a block of several cycles, r0's cycle gets starts only in proportion to its
    length, in expectation.
- **Two things depend on the exact rows, as tests rather than invariants:**
  - The Miri test's seeds were "searched for natively" against today's starts, so that
    every handoff transition is reached (`src/bwt.rs:1219-1227`, with `assert_reached`).
  - Any change to the start set needs that search run again.

**The candidate forms:**

| form | starts | r0 | bits a group |
|---|---|---|--:|
| **B1** | slot 0 of a random group | the one unaligned start, tested by `r == r0` | 1 |
| **B2** | slot s = r0 & 31 of a random group, s fixed for the block | its group's slot-s row: no special case | 1 |
| **B3** | `random_rows` as today, at most one a group, the slot stored | its group's slot | 6 |

**B3 keeps today's distribution by construction** (verified in code that nothing else
constrains the starts). The only change is that two random rows in one group keep one.
- At n = 900k with 511 rows over 28,125 groups, about 4.6 starts are lost, against about
  0.15 lost today to equal rows.
- B3 caps `ns` at about n/32 groups. That doesn't bind in production: blocks below
  `MIN_SEGMENTED` (65,536 rows, 2,048 groups) are walked plainly (`:566-569`). It does
  bind in the small-block tests and the Miri test.

**Balance, simulated (§6):**

- **One cycle (every encoder's ordinary block): B1, B2 and random are level.** The ratio
  is the modelled makespan over the ideal at 32 lanes. On Python source (400k rows),
  random is 1.155 and B2 1.146. On a JSON corpus shaped like Wikidata (400k), random is
  1.147, B2 1.227 and evenly spaced 1.30. On random bytes (200k), random is 1.21 and B2
  1.116. Starts lost to shared groups: 512 → 502 at 400k rows.
- **Several cycles: B1 fails on balance (it stays correct).**
  - In periodic blocks the rows ≡ 0 mod 32 gathered on one or two cycles. With r0 on any
    other cycle, r0's segment is its whole cycle, walked by one lane.
  - The modelled makespan was 10.6–16.0× the ideal (p = 1000, k = 64: 1,000 steps against
    random's 2,372; p = 65,536, k = 4: 65,536 against 9,475).
  - For s·s at 900k, that would be about 450k serial steps on that block. It is still
    faster than `walk`, but about 14× today's walk.
- **B2 showed no such case.**
  - With r0 placed on each of several cycles in turn, B2's makespan was 73, 37, 122 and
    2,474 steps, against random's 2,232, 1,183, 234 and 10,443. B2 also walks less,
    because few starts fall off r0's cycle.
  - The one shortfall was p = 999, k = 33: 1,390 against 1,280 (1.09×).
- **Hypothesis:** B2's balance holds on real encoders' orders for equal rotations and on
  crafted permutations. The model orders equal rotations by index. A crafted permutation
  can put r0's residue class on other cycles, and B2 then degrades like B1 (it stays
  correct). B3 has no such exposure.

## 2. Who reads the START flag

**Verified in code.** Every site:

| site | where | what it does |
|---|---|---|
| definition | `src/bwt.rs:318-321` | bit 31; links are below 2^20 |
| set | `:381` (flagged), `:627` (room) | a read-modify-write per start before the walk |
| `take`'s first step | `:403`, `:647` | masks the flag off (`& LINK`), so the loop sees only the next start's flag (`:334-336`) |
| hot test, full phase | `:452` (flagged), `:712` (room) | `q & START` on the entry just loaded |
| hot test, tail | `:492` (flagged), `:772` (room) | the same |
| clear | `:512`, `:792` | `&= LINK` per start |
| tests | `:947, 967, 986, 1214` | assert that every flag has been cleared |
| coop-fused | `examples/coop.rs:85-86, 676, 713, 751, 797, 834, 889, 937, 973, 1019` | the same roles: `traverse`, `traverse_room`, their set, mask and clear |

**What does not read the flag** (verified in code):

- `SegScratch::assemble` (`:210-222`) reads `bufs`, `next`, `first` and `n` only.
- `seg_of` is a binary search over `starts` (`:638`), by row.
- `tests::segment_oracle` (`:894-912`) marks the starts in its own `index` table, built
  from the starts list. It runs after the walk, on the cleared table.
- `check_segments` (`:915-920`) compares buffers and successors.
- The bitmap walk `walk_segments` (`:246-317`) uses its own bitmap.
- `block.rs`'s `BAD = 1 << 31` (`src/block.rs:65`) belongs to the Huffman table and is
  unrelated.

**The cyclic case.** No code path special-cases several cycles. The tests that cover it,
`room_walk_short_cycles` (`:1049`, cycles of 1 and 2), the crafted permutations (`:1079`,
periodic with p ≤ 5) and `flagged_walk_equals_plain` (`:1147`, "abc"), check output
against `walk` and segments against the oracle. Neither check depends on how a start is
encoded.

## 3. The cooperative walk: flag, count or row?

**Verified in code: by flag, as production.**

- **Ending a segment:** `traverse` tests `q & START` (`examples/coop.rs:713`), and so does
  `traverse_room` (`:889`, tail `:937`). The starts are `random_rows` plus r0 (`:746-752`,
  `:968-974`).
- **The teams' cut points are not walk cuts.**
  - The build's ranges are L-position ranges: n·j/k rounded to the 16 KiB `CHECKPOINT`
    (`:89-92, 468-473, 565-570`).
  - They split the build's scatter between threads. The walk never sees them.
- **The checkpoints (#78) are counts of L's bytes for the build, not walk positions.**
  - They are cumulative byte counts every 16 KiB of L, made by `mtf_cp` and `mtf_cp2`
    (`:1073-1190`, `:1194-1349`), and read by `coop_build_given` and `coop_build_fused`
    (`:454, 554`).
  - They place each range's slots in every bucket.
- **Cross-thread synchronisation is once per segment:** `taken.fetch_add`
  (`:701, 715, 868, 892, 939`). It never happens per step.
- **The counts that do exist sit off the chain and end nothing.**
  - `walked` is a statistic: `+= 1` a step in `traverse` (`:706, 718, 732`) and a
    segment's length at its end in `traverse_room` (`:891, 938`).
  - The room pass's `room` (`:877-885`; production `src/bwt.rs:692-707`) is a buffer
    bound, computed once a pass.
- **No lane knows its segment's length in advance.**
  - A length is the gap in text position between a start and the next one on its cycle.
  - Starts are chosen as rows, and nothing in the format or the build gives a row's text
    position: the entry holds the L position, `src/bwt.rs:147`, the bz2ngrs entry §1.
  - So a count could only come from walking.

## 4. One step's dependency path, by candidate

**Today** (`room_segments`, `src/bwt.rs:711-745`: load `:711`, test `:712`, `l.row = q >> 8` `:737`), row r known:

```
r ──► q = load t[r] ──► r' = q >> 8 ──► prefetch t[r'], next load
              └──► test q, START ──► jnz finish   (predicted not taken; off the r→r' path)
```

- There is one load. After `q` arrives there is one operation and one branch for the end
  test. The next address doesn't wait for the branch: it is speculated.

**The descriptor step**, common to every candidate:

```
r ──► g = r >> 5, j = r & 31 ──► D = load desc[g] ──► r' = decode(D, j) ──► next load
```

- `j` is ready before D arrives.

| candidate | the end test (★ = before D arrives, off every path) | after D arrives | loads added | branches | sync | bits a group | result |
|---|---|---|--:|--:|---|--:|---|
| **A: per-row markers** | — | `bt D.markers, j` → `jc` | 0 | 1 (as today) | none | 32 | passes on cost; it is the outcome to beat |
| **B1: slot 0, r0 apart** | ★ `m = (j == 0) ? FLAG : 0`, ★ `e = (r == r0)` | `test D, m`; `or e` → branch | 0 | 1 | none | 1 | passes on cost, **fails on balance** with several cycles (§1) |
| **B2: slot r0 & 31 for the block** | ★ `m = (j == s) ? FLAG : 0` | `test D, m` → `jnz` | 0 | 1 | none | 1 | **pass**; balance off one cycle is a hypothesis |
| **B3: flag + 5-bit slot** | ★ `w = (32 \| j) << SH` | `xor D, w`; `test 63 << SH` → `jz` | 0 | 1 | none | 6 | **pass**; today's distribution |
| **C: per-lane remaining count** | `dec c[k]` → `jz`, independent of D | nothing | 0 | 1 | none | 0 | **fails**: the length isn't known (§3) |
| **D: side table (bitmap or hash of starts)** | — | `load bm[r >> 6]` in parallel with D; `bt`; `jc` | **1** | 1 | none | 0 | **fails**: one added load a step |

**Per candidate:**

- **A** is what the descriptor has without this audit.
- **B1** also needs r0 in a register. Its balance failure is the several-cycles case in
  §1; it stays correct.
- **B2 matches today's post-arrival cost.** It is one `test` plus a branch, with the
  mask built from `r` while the load is in flight. s is a block constant in a register.
  r0 needs no special case.
- **B3 adds one operation after arrival over today's:** two ALU operations (or
  shift–and–compare, three) after D, in parallel with `decode`. They feed only the
  branch. The start selection becomes sort, then dedup by `r >> 5` with r0 kept.
- **C has the best path of all,** fully off the chain. It still fails, because the
  lengths are unknown (§3). Cutting segments by count instead would make each segment's
  start the previous segment's end row, which is a serial handoff between lanes.
  - In the room loop, a lane counter would also bring back a per-step
    read-modify-write of lane state. #77 removed that for the output pointer
    (`docs/journal/2026-10-28-cooperative-inversion.md` §12.1, on `research/coop-fused`).
- **D's load is indexed by the chain's row.** It is off the r→r' path, but it is one
  more load every step, and a miss holds retirement. It is the 112 KB bitmap that #12
  removed (`2026-10-27-bz2-bakeoff.md:300-303`, `CHANGELOG.md:357`; the old form is at
  `src/bwt.rs:296`). A 512-entry hash set is the same shape.

**What this leaves to measurement (hypothesis):**

- B3's one or two extra ALU operations per step are throughput, not latency. The room
  loop is 16 instructions a step (coop journal §12.1).
- Whether a two-plateau `decode` plus B3's test stays within the step's slack is for the
  dependent-successor throughput benchmark that #79 lists after the census.

## 5. What stays the same

**Verified in code; these hold for B2 and B3:**

- **Set and clear:** the flags are set before the walk and cleared after it, about 512
  read-modify-writes a block, as at `:627/:792`.
- **`take`:** it decodes the start's own step and ignores its flag, as `& LINK` does.
- **Ownership, spare room, handoffs and lifetimes** (`:578-592`) are unchanged. None of
  them mentions how a start is encoded.
- **Tests:**
  - `segment_oracle` keeps working, since it takes the starts list.
  - The assertions that the flags were cleared become checks on the descriptor's flag.
  - The Miri seeds and `assert_reached` need searching again for any new start set.
    B3's group cap also lowers `ns` on the tests' small blocks (n ≈ 200-4,200 rows,
    256 segments).

## 6. The simulation (simulated, untimed, this VM)

- **The table:** a T-vector built as `build` does (row j holds the L position i with
  LF(i) = j). Rotations were sorted by cyclic prefix doubling, equal rotations by index.
- **The walk:** segments walked to the next start, and r0's cycle joined.
- **The balance model:** list scheduling of the segments over 32 lanes in sorted-start
  order, the way `queued` hands them out. The ratio is the makespan over ⌈steps/32⌉.
- **The inputs:** 511 starts plus r0, on:
  - 400k bytes of Python source;
  - a 400k JSON corpus shaped like Wikidata;
  - 200k random bytes;
  - periodic s^k for (p, k) = (1000, 64), (1000, 32), (3000, 2), (7, 20000), (999, 33),
    (65536, 4) and (5, 40000), with r0 also placed on each of up to three other cycles.
- **What it doesn't capture:** the room walk's passes, cache behaviour or real encoders'
  orders for equal rotations. It shows how the segment lengths spread, nothing about
  speed.
