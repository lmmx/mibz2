# N1's s = 1 upgrade and the capped variable-length walk: a brute-force correctness check

**Status:** read-only, an offline Python model. Nothing was built in Rust, benchmarked or
run on CI. This closes the step that `2026-10-30-n1-construction-economics.md` §4.2 note
(f1)/(f2) and §8 left argued but unchecked: the s = 1 upgrade of unsplit parents in
P^{2|4} and P^{4|8}, and the segmented walk over the mixed-length records it produces.

**Labels:** *measured* means from the runs listed in §10 (file named). *Deduced* means it
follows from the definitions in §2. There are no model figures in this note.

## 1. Summary

1. **The s = 1 upgrade is exact (deduced, §5; measured, §3).**
   - **The claim.** Let a level-a interval I have the record (D_I, w_I), and let its
     whole Ψᵃ image [s + D_I, e − 1 + D_I] lie in one level-a interval J. Then
     Ψ^{2a}(x) = x + D_I + D_J on all of I, with the word w_I ‖ w_J.
   - **The proof** needs only that the two level-a records are exact. It needs neither
     minimality nor the union identity.
   - **Unsplit means the same thing.** With the stable Ψ, the image test passes exactly
     when the parent is unsplit at level 2a, so P^{a|2a} upgrades exactly the parents
     `sample_sim.py` counts.
2. **The capped variable-length walk reproduces the block exactly (deduced, §5;
   measured, §3).**
   - A lane stores 8 bytes, advances by len and stops on landing at a start.
   - With caps from every start, origPtr's row included, it emits each segment's bytes
     exactly and ends on the decoder's next start.
   - The decoder's join then gives the inverse BWT byte for byte, for any start set
     containing origPtr and any lane count.
3. **Measured: 0 failures** in about 12 minutes of one-core CPU over the §3 suites
   (14 minutes with §4's mutation runs).
   - **Exhaustive records:** every text over {a, b, c} up to n = 10 (88,572), and every
     L up to n = 9.
   - **Exhaustive walks:** every text up to n = 7 with every origPtr, plus 8 kinds of start
     set.
   - **Every start set:** every start set containing origPtr for every text up to
     n = 6 over three letters (n = 8 over two). That is 497,325 segmentations.
   - **Random:** 103 adversarial texts up to n = 4,000, and 200 arbitrary L.
   - **Real data:** six real Wikidata slices.
   - **In all:** 5.65 million checked walks (one plan, one start set, one lane count
     each).
   - Each walk's output was compared with the original text, or for an arbitrary L with
     the naive inverse. Each was also compared segment by segment with a mirror of
     mibz2's `segment_oracle`.
4. **The checker catches every non-equivalent injected bug (measured, §4).** That covers
   upgrading split parents, the image test at the last row, a wrong J, in place, four
   cap and advance bugs, reversed words, assembly order and rank off by one. Each first
   fails on a text of three bytes or fewer.
   - **Output-only checks miss some record bugs** that the per-row record check catches.
     For in place: 419 failing cases against 551.
5. **In place against separate is not a detail: in place is wrong (measured, §6).**
   - The smallest counterexample is T = `ab` at level 1. At level 2, T = `aab` gives a
     dependency 3-cycle, so no processing order works.
   - On real slices, in place would corrupt 34–43% of the records.
   - The economics note's §1, item 3 ("in place") should read "into a separate array".
6. **Caps on real data reach their bound.** The maximum is 14 cap records per start at
   K = 8, with a mean of 8.4–12.0 (measured, §3). That is above the "at most 5" seen on
   random strings, and still negligible per block.
7. **What is not covered (§8):**
   - full-size blocks;
   - 32-bit field widths;
   - the len field, which the 12 B walk record has no room for as specified;
   - the bitvector rank directory;
   - the merge-order construction;
   - the room walk's pass mechanics.

## 2. What exactly was checked: the definitions

### 2.1 The reference (written apart from the candidate)

The reference and the candidate are two separate files that import nothing from each
other (`ref_naive.py`, `n1core.py`). The candidate is given only L, origPtr and the start
set. It never sees Ψ, F, the T-vector or the original text.

- **The input and the expected output (text-driven suites).** Start from a string T.
  Sort its n rotations as strings, with ties by start index. L is the byte before each
  rotation, rot[row] is its start, and origPtr is the row of rotation 0. Decoding (L, j)
  must give rotation rot[j] of T. This is taken from T itself, so for j = origPtr the
  expected output is T.
- **Ψ for the record-level check.**
  - For a primitive T, Ψ is read off the rotation order: the row of rotation i maps to
    the row of rotation i + 1 (mod n).
  - For a periodic T, that map can be unstable at the wrap, and the decoder's Ψ is the
    stable one. There the reference uses the stable map, written naively: the q-th
    occurrence of byte c in F maps to the q-th occurrence of c in L, from two position
    lists.
  - On every primitive text, the two were asserted equal.
  - Ψᵏ(x) and the k bytes F[x], F[Ψx], …, F[Ψ^{k−1}x] are then computed by iterating Ψ.
    The direct record partition M_k breaks between j and j + 1 wherever the k-words or
    the successors Ψᵏ(j + 1) ≠ Ψᵏ(j) + 1 differ.
- **Arbitrary last columns (L-driven suites).** Most L are not the BWT of anything at
  hand, and their Ψ has several cycles. There the expected output is F[Ψⁱ(j)] for
  i < n, from the naive stable map. This is the inverse BWT's definition, and it is what
  `bwt::walk` computes for any L and origPtr.
- **The decoder mirror,** to pin the segment semantics: the T-vector exactly as
  `bwt::build` makes it, `bwt::walk`, `tests::segment_oracle` (each start's bytes, the first
  from its own entry, until the next row is a start, and that start's index), the join
  of `walk_segments` / `SegScratch::assemble`, and `bwt::random_rows`. The mirror is
  itself checked, on every case: its links equal the naive stable Ψ, and its walk and
  segmented join equal the expected output.

**Code shared between the reference and the candidate:** none of the reference logic.
- Both run on Python's `bytes`, `sorted` and `bisect`.
- The harness (`n1run.py`) reads the candidate's tables to *choose inputs*: it places
  the "straddle" starts inside the candidate's long hops. It also reads a row's uncapped
  length to state what the capped length must be. Neither can hide a mistake: the
  expected values (output, Ψᵗ, words, the capped length's minimum over t) all come from
  the reference.
- `n1extra.py`'s cross-check against `sample_sim.py` (§7), and `cap_stats.py`, use a
  numpy prefix-doubling rotation sort to *generate* the input at n = 20,000, where the
  naive sort's keys would take n² bytes. The cross-check's expected output is still T
  itself.
- The length-mode walker is given each segment's length from the reference's Ψ. That is
  its model's premise (a walker that knows lengths), not a value it is checked
  against.

### 2.2 Conventions, and where they differ from the decoder

- **Rows are F-convention,** as in `sample_sim.py`: the walk at row u emits F[u] and moves
  to Ψ(u). The record partition is portfolio §3.1's M_k. B includes the F-bucket
  boundaries, and row n − 1 is never a break (no row follows it).
- **The decoder is L-convention:** `bwt::walk` at row x emits L[x] = F[LF(x)] and moves to
  Ψ(x). The two are one walk under u = LF(x). So:
  - a decoder segment start x is the N1 start LF(x);
  - the decoder's first row r0 = Ψ(origPtr) is the N1 row origPtr;
  - the decoder's segment from x and N1's from LF(x) emit the same bytes and end at
    corresponding starts. This is checked segment by segment against
    `segment_oracle` (§2.4).
- **Shipped start sets** are taken exactly as `walk_segments_flagged` takes them:
  `random_rows(n, 511)` plus r0, sorted and deduplicated, then mapped by LF. On blocks
  this small, most rows become starts.

### 2.3 The candidate

- **Level 1** comes from L's runs (portfolio §3.1(c′)1): a run (c, p, ℓ) gives source
  rows [C[c] + q_c, + ℓ), with D = p − (C[c] + q_c) and the word (c).
- **Level 2a from level a** is built by image splitting (§3.1(c′)2), with J found by rank.
- **The s = 1 upgrade, P^{a|2a}.** For each level-a interval I = [s, e) with record
  (D_I, w_I):
  - let lo = s + D_I and hi = e − 1 + D_I;
  - if rank(lo) = rank(hi) = J, write (D_I + D_J, w_I ‖ w_J, length 2a);
  - otherwise write (D_I, w_I, length a).

  The output goes to a separate array, and D_J and w_J are read from the unmodified
  level-a array.
- **Caps** are a post-split of the final table (portfolio §3.1(g)2).
  - For each start e and t = 1 … K − 1 (K the longest record), take y = LF^t(e). LF
    comes from the level-1 records alone, since their images tile the rows.
  - If t < len(y), row y gets a singleton record of length t: D = e − y, and the word's
    first t bytes. Where several starts reach y, the smallest t wins.
- **The walker** is a scalar model of the lane walk.
  - L lanes go round-robin in lockstep. Segments are queued in ascending start order,
    and a lane takes the next queued one when its own ends.
  - Each hop ranks the lane's row and stores 8 bytes (the word, padded with the junk
    byte 0xFE). It then advances its write position by len and moves the row by D.
  - The segment ends when the new row is a start. It records its byte count and the
    index of the start it reached.
  - The output joins segments from origPtr's, each followed by the one it ran into, and
    truncates at n. That is `walk_segments`' join.
- **A second walker, the length mode,** is `sample_sim.py`'s hop rule. The walker knows
  each segment's length, hops min(len, remaining), and its records have no caps. It
  must give the same bytes and exactly the same lookups as the flag walker.

### 2.4 The checks applied to every case

**Records** (every row x, every plan P¹, P², P⁴, P⁸, P^{1|2}, P^{2|4}, P^{4|8}):
- x + D = Ψ^{len}(x), and the stored word equals the len bytes of F along the walk from x;
- the doubling's level-a partition equals the direct M_a;
- an upgraded parent is never split by the direct M_{2a}, and every unsplit parent is
  upgraded (the s = 1 cut, exactly).

**Caps** (every row x, every start set):
- len(x) = min(the uncapped len, the first t in [1, len) with Ψᵗ(x) a start);
- D and the word are exact for that length;
- the capped table has at most 2(K − 1)·|S| more records than the uncapped one.

**The walk** (every start set, every plan):
- the output equals the expected output, byte for byte;
- every segment's bytes and successor equal the decoder mirror's `segment_oracle`;
- the length-mode walker gives the same output and the same number of lookups;
- for uniform Pᵏ, the lookups are Σ⌈ℓ_s/k⌉ over every walked segment.

**Lanes:** 32 (the shipped `FLAGGED_LANES`), plus 2, 3 or 5 lanes in some suites.

## 3. Coverage and results (measured)

**Every run passed with 0 failures.** Counts are summed over the plans checked in each
suite (P¹, P², P⁴, P⁸, P^{1|2}, P^{2|4}, P^{4|8}, unless the plans column says otherwise).
"Upgrades" are summed over P^{1|2}, P^{2|4} and P^{4|8}. CPU is one core.

| Suite (file) | Inputs | Start sets and lanes | Plans | Checks run | Fails | CPU |
|---|---|---|---|---|--:|--:|
| `rec` (`out_rec_texts_n10.json`) | every text over {a, b, c}, n = 1–10: 88,572 texts. Ψ from rotations for 88,161 primitive texts, the stable map for 411 periodic | — | all 7 | 5,890,143 row-records, 5,078,070 records; 1,878,468 upgrades, 240,882 split parents, 222,080 upgrades at the block's last row, 1,829,737 singleton upgrades | 0 | 49 s |
| `recL` (`out_rec_L_n9.json`) | every L over {a, b, c}, n = 1–9: 29,523 | — | all 7 | 1,756,713 row-records; 576,065 upgrades, 154,137 of them with the image in the parent itself (Ψ with short cycles) | 0 | 15 s |
| `walk` (`out_walk_texts_n7.json`) | every text, n = 1–7: 3,279 texts; every origPtr (21,324) | shipped (512 and 8 segments), single, all rows, 9 traversal-step patterns (lengths 1, 2, 3, 7, 8, 9, mixed), 2 random, adjacent rows, straddle; 32 and 3 lanes | all 7 | 3,011,414 walks; 1,477,606 segments, 819,510 of length 1; 797,894 cap records | 0 | 202 s |
| `walkL` (`out_walk_L_n6.json`) | every L, n = 1–6: 1,092; every origPtr (6,015); 896 with a multi-cycle Ψ | as `walk` | all 7 | 557,158 walks | 0 | 31 s |
| `allseg` (`out_allseg_abc_n6.json`) | every text over {a, b, c}, n = 1–6; every origPtr; **every start set containing origPtr** (162,363) | 2 lanes | P^{1\|2}, P^{2\|4}, P^{4\|8}, P⁸ | 649,452 walks | 0 | 42 s |
| `allseg` (`out_allseg_ab_n8.json`) | every text over {a, b}, n = 1–8; every origPtr; every start set (334,962) | 2 lanes | as above | 1,339,848 walks | 0 | 98 s |
| `rand` (`out_rand.json`, seed 12345) | 103 texts, n = 64, 300, 1,000 and 4,000 (136,325 bytes); the genuine origPtr plus 1 random | as `walk` less "all rows"; 32 lanes | all 7, then P^{2\|4} and P^{4\|8} | 11,319 walks; 2,701,556 cap records | 0 | 73 s |
| `randL` (`out_randL.json`, seed 777) | 200 arbitrary L, n ≤ 1,000, σ ∈ {1, 2, 3, 4, 8, 64}, half run-structured; 196 with a multi-cycle Ψ; 2 origPtrs each | as `rand`; 32 and 5 lanes | all 7 | 83,300 walks; 26.7 M bytes walked | 0 | 180 s |
| `real` (`out_real.json`) | 6 Wikidata slices: census blocks 0 (2 slices), 145, 90, 33, 220; 2,000–6,000 bytes, 20,000 in all; genuine origPtr | as `rand` | all 7 | 756 walks; 571,331 cap records | 0 | 16 s |

**The random families** (each at every size): a^n; a^{n/2} b^{n/2}; Fibonacci and
Thue–Morse words; periodic strings with periods 1, 2, 3, 5, 8, 9 and 16, both cut and
exact (so Ψ has several cycles); uniform random strings over 2, 3, 4, 16 and 256 symbols;
random-length runs; Wikidata-like token soup. De Bruijn sequences B(2, 9), B(3, 6) and
B(4, 5) were added once.

**What these cases exercise, by construction:**
- singleton intervals, which are 89% of records in `rec`;
- intervals ending at row n − 1;
- an image ending exactly on J's last row;
- a parent whose image is the parent itself;
- multi-cycle Ψ, where segments off r0's cycle are walked but not joined, and r0's cycle
  is joined round more than once;
- segments of length 1, where every row is a start;
- a single segment, which is the whole cycle;
- a start one step after another;
- starts placed inside a would-be 2a-byte hop ("straddle");
- `random_rows`' exact start sets, both 512 and 8.

**Also measured:**
- **The cap records per start**, on three 20,000-byte real slices with 512 random starts
  (`out_cap_stats.json`):
  - the maximum reaches the bound 2(K − 1) exactly: 14 for P⁸ and P^{4|8}, 6 for P⁴ and
    P^{2|4};
  - the mean is 10.0–12.0 for P⁸, 8.4–10.2 for P^{4|8} and 3.6–4.1 for P^{2|4}.
  - The portfolio's "at most 5 per start observed" (§3.1(g)2) came from random strings,
    whose intervals are mostly singletons, so it doesn't carry to Wikidata. On real data
    almost every cap row lies inside a longer interval and adds two records.
  - **Deduced:** at most 14 × 512 = 7,168 records per block, against about 110,000 at
    level 8. The bound holds, and the cost is still small. It is per-start work the
    economics note costs at 150–300 instructions per start.
- **Lookups match the hop-rule model exactly.** On every walk, the flag walker (caps in
  the records) made exactly as many lookups as the length-mode walker (`sample_sim.py`'s
  rule). For uniform Pᵏ, that was exactly Σ⌈ℓ_s/k⌉.

## 4. Mutation tests: the checker fails when it should (measured, `out_mut.json`)

**Each mutant was run against five reduced suites:**
- every text with n ≤ 5, every origPtr, every start-set kind (363 texts);
- every L with n ≤ 4 (120);
- every text with n ≤ 4 against every start set (120);
- the random families at n = 40 and 150 (53 texts);
- 40 arbitrary L with n ≤ 120.

**Two modes:**
- **full:** every check in §2.4;
- **output only:** only the output and per-segment comparisons, which is what a decoder
  test would see.

A case counts once, at its first failure. The unmutated candidate failed 0 of 696 cases
in both modes (110 s for all of §4).

| Mutant (the bug a real implementation could have) | Cases failing, full (of 696) | Output only (of 696) | Smallest failing case, and the first check to fail |
|---|--:|--:|---|
| `upgrade_split`: upgrade every parent | 442 | 441 | text `aab`: P^{1\|2} row 1, D gives 3, Ψ² = 0 |
| `upgrade_lastrow`: the unsplit test omits the image's last row | 394 | 396 | `aab`, as above |
| `upgrade_wrongJ`: J taken at the parent's start s, not its image s + D | 216 | 211 | `abb`: P^{1\|2} row 1, D wrong |
| `inplace_fwd`: upgrade in place, ascending | 551 | 419 | `ab`: P^{1\|2} row 1, D 0, Ψ² = 1 |
| `inplace_rev`: in place, descending | 551 | 418 | `ab`: P^{1\|2} row 0 |
| `cap_miss_last`: no cap for t = len − 1 | 696 | 696 | `a`: capped row 0, len 2, want 1 |
| `cap_len_plus1`: a capped record keeps t + 1 bytes | 696 | 696 | `a`, as above |
| `cap_skip_r0`: origPtr's row left out of the cap sources | 685 | 685 | `aa`: capped row 0, len 2, want 1 |
| `word_reversed`: the word stored in reverse byte order | 618 | 618 | `ab`: output ≠ T |
| `advance_a`: a lane advances by a, not len | 696 | 696 | `a`: segment bytes differ |
| `advance_W`: a lane advances by 8 | 696 | 696 | `a` |
| `assemble_queue`: segments joined in start order, not by successor | 524 | 524 | `abb`: output ≠ T |
| `rank_left`: rank off by one at interval starts | 685 | 685 | `aa`: segment bytes differ |
| `len_hop_plus1`: the length-mode last hop is min(len, remaining + 1) | 606 | 606 | `aab`: output ≠ T |
| `upgrade_conservative`: the test also requires the row after the image in J | 0 | 0 | equivalent: correct output, fewer upgrades |
| `cap_le_len`: caps also at t = len | 120 | 0 | equivalent: same bytes; caught only by the 2(K − 1)·\|S\| bound (`aabab`) |

**The reading:**
- **Every non-equivalent mutant is caught,** and each first fails on a text of length
  3 or less.
- **Output-only checks are weaker on the record bugs.**
  - In place: 419 cases against 551.
  - A wrong record is not always executed at full length, because caps or the landing
    pattern can skip it. So a decoder-level byte comparison alone can let a wrong
    upgrade through on some inputs. The per-row record check is what closes that.
- **Crashes count as caught.** In output-only mode, many of the upgrade mutants' failures
  are a walker row leaving [0, n). In Rust that would be an out-of-bounds read, or
  silent garbage with unchecked indexing.
- **The two equivalent mutants are the expected ones.**
  - `upgrade_conservative` is caught only by the completeness count. On every text with
    n ≤ 8 it leaves 122,376 unsplit parents unupgraded and makes 41,994 upgrades, against
    0 and 164,370 for the design.
  - `cap_le_len` adds redundant singleton records, which the record bound sees.

## 5. The composition, deduced

**Notation.**
- A level-a table is a partition of the rows [0, n) into intervals. On each interval I it
  is *exact*: for every x ∈ I, Ψᵃ(x) = x + D_I and w_a(x) = w_I, where
  w_a(x) = F[x] F[Ψx] … F[Ψ^{a−1}x].
- rank(y) is the interval containing row y.

**Lemma 1 (composition).**
- *Statement.* Let I = [s, e) and J = [s′, e′) be intervals of an exact level-a table,
  with Ψᵃ(I) ⊆ J. Then for every x ∈ I:
  - Ψ^{2a}(x) = x + D_I + D_J;
  - w_{2a}(x) = w_I ‖ w_J.
- *Proof.* Take y = Ψᵃ(x) = x + D_I, which lies in J. Then
  Ψ^{2a}(x) = Ψᵃ(y) = y + D_J. Also w_{2a}(x) = w_a(x) ‖ w_a(Ψᵃx) = w_I ‖ w_J. ∎
- *What it needs:* only that the two level-a records are exact on I and J. It needs
  neither minimality of the partition, nor stability of Ψ, nor portfolio §3.1(a). It
  holds for any level a, so P^{1|2}, P^{2|4} and P^{4|8} alike.
- *What it uses:* J's **level-a** record (D_J, w_J). If J itself was upgraded, its new
  record (D_J + D_K, w_J ‖ w_K) is a Ψ^{2a} record, and composing with it gives Ψ^{3a}
  with a 3a-byte word. Lemma 1 then fails (§6).

**Lemma 2 (what "the image lies inside J" must mean).**
- *Statement.* Ψᵃ(I) ⊆ J if and only if rank(s + D_I) = rank(e − 1 + D_I). Equivalently,
  no level-a interval starts in (s + D_I, e − 1 + D_I].
- *Proof.* Ψᵃ is the translation x ↦ x + D_I on I, so Ψᵃ(I) is the contiguous rows
  [s + D_I, e − 1 + D_I]. A contiguous range lies in one interval exactly when its
  first and last rows do. ∎
- **The interval's last row.** The test must include the image of I's last row,
  e − 1 + D_I. It must not include the row after it, e + D_I, which is the image of
  nothing in I.
  - Dropping the last row is unsound. Mutant `upgrade_lastrow` is caught (§4).
  - Adding the row after is sound but loses upgrades. Mutant `upgrade_conservative`
    produces correct output and is not caught by any output check. Only the
    completeness count "every unsplit parent is upgraded" sees it.
  - Measured: the distinction matters often. An upgraded parent's image ends exactly on
    J's last row in 1,686,214 of 1,878,468 upgrades (90%) over every text with n ≤ 10
    (`out_rec_texts_n10.json`), and in 15,208 of 19,369 (79%) on the real slices
    (`out_real.json`).
- **Wrap.** Row space has no wrap: Ψᵃ maps rows to rows in [0, n), and a translation of
  a contiguous range is contiguous. D may be negative. The text's cyclic wrap (T[n−1]
  followed by T[0]) is in the traversal, not in row space. The records don't refer to
  origPtr at all.
- **The last row of the block.** Row n − 1 has no successor row, so B, and with it every
  M_k, never breaks after it. An interval ending at n, or an image ending at n − 1, needs
  no special case.

**Lemma 3 (the test is "unsplit at the next doubling").**
- *Statement.* With the stable Ψ, the image test passes exactly when I is not split by
  M_{2a}. So P^{a|2a} upgrades exactly the parents that `sample_sim.py` counts as having
  one child, `kids == 1`.
- *Proof.*
  1. Portfolio §3.1(a) gives M_k = C_k, where C_k = ⋃_{t<k} Ψ^{−t}(B).
  2. So C_{2a} = C_a ∪ Ψ^{−a}(C_a).
  3. Take j ∈ [s, e − 2], an interior pair of I, so j ∉ C_a. Then j ∈ C_{2a} exactly
     when Ψᵃ(j) = j + D_I ∈ C_a.
  4. Such a break lies in [s + D_I, e − 2 + D_I] exactly when an interval starts in
     (s + D_I, e − 1 + D_I]. ∎
- *Measured:* the check "upgraded ⇔ not split by the direct M_{2a}" held on every case
  (§4). It is the only place the upgrade's plan depends on §3.1(a). Its correctness
  (Lemma 1) doesn't.

**Lemma 4 (caps).**
- *Statement.* For a start set S and a row y, let t_y be the first t in [1, len(y)) with
  Ψᵗ(y) ∈ S. The capped record (D = Ψ^{t_y}(y) − y, the word's first t_y bytes) is exact
  on the singleton {y}.
- *Proof.*
  - Every prefix of an exact record is exact on a singleton: its word is the first t
    bytes of the walk from y, and Ψᵗ(y) is the start reached.
  - Each start e and each t give one row, y = LF^t(e). So a start adds at most K − 1 cap
    rows.
  - Each cap row splits its interval into at most three pieces, so it adds at most two
    records. That is at most 2(K − 1) records per start. ∎
- *What it needs:* LF only through level 1, because the level-1 images tile the rows.

**Lemma 5 (the capped walk emits each segment exactly).**
- *Setup.* Take a lane on row x of the segment from u. Let r = min{t ≥ 1 : Ψᵗ(x) ∈ S},
  the steps to the segment's end. The segment ends because u's own cycle contains u.
- *By Lemma 4,* the record at x has length ℓ = r if r < len(x), and ℓ = len(x)
  otherwise. So ℓ ≤ r.
- *The hop* emits w_ℓ(x), which is the walk's next ℓ bytes, and lands on Ψ^ℓ(x).
  - If ℓ = r, it lands on the next start, and the segment ends.
  - If ℓ < r, then Ψ^ℓ(x) is not a start, by the definition of r, and the lane goes on.
- *So:*
  - the lane never passes a start;
  - it emits exactly the segment's bytes, in order;
  - it ends on the start the decoder's segment ends on;
  - its lookups are those of the hop rule min(len, remaining). That rule is
    `sample_sim.py`'s, which is why its lookup counts transfer (§7).
- *The 8-byte store.*
  - A hop writes [p, p + 8) and advances p by ℓ ≤ 8. The next hop's store, at p + ℓ,
    overwrites the junk. So after the last hop, bytes [0, p) are exactly the segment.
  - It needs a stored width of at least the longest record, 8 ≥ 2a (a ≤ 4), and 7 bytes
    of buffer slack past each segment.
- *The join* follows the successors from origPtr's segment and truncates at n. That is
  the decoder's join, on segments equal to the decoder's, so the output is equal.
- *Two consequences:*
  - **origPtr must be a cap source.** The decoder's r0, which is the N1 row origPtr, is
    always a start. The text's wrap point is the step into it, so no hop crosses the
    wrap. Mutant `cap_skip_r0` shows that a builder which leaves r0 out of the caps
    crosses it.
  - **Multi-cycle Ψ** (periodic blocks, or arbitrary L). Starts off r0's cycle are walked
    and never joined. r0's cycle is joined round again, as in `walk_segments`. A cycle
    shorter than a record is handled by the caps: a fixed point Ψ(x) = x with S = {x}
    caps to length 1.

**Together:** P^{a|2a}'s records are exact (Lemmas 1–2), and they upgrade exactly the
unsplit parents (Lemma 3). With caps from every start, origPtr's included (Lemma 4), the
segmented walk reproduces the inverse BWT byte for byte for any start set containing
origPtr, any lane count and any L (Lemma 5).

## 6. In place against a separate array

**It matters.** An upgrade written into the array it reads from is wrong in general, and
no processing order rescues it.

**Why (deduced).**
- Upgraded parent I reads (D_J, w_J) for J = rank(s_I + D_I). If J has already been
  upgraded in place, I reads D_J + D_K, and its record becomes a Ψ^{3a} displacement
  under a 2a-byte length (§5, Lemma 1).
- In place is therefore safe only if every upgraded I is processed before each upgraded
  J(I) ≠ I. That needs the dependency graph I → J(I), restricted to upgraded parents, to
  be acyclic. A self-loop J(I) = I is harmless if the record is read before it is written.
- Each I has one J, so the graph is functional. A cycle of length ≥ 2 rules out every
  order.

**The smallest counterexample (measured, `out_inplace.json`).** T = `ab`, so L = `ba`,
at level a = 1 (P^{1|2}).
- Ψ = [1, 0], and both level-1 records are singletons and unsplit: row 0 has (+1, `a`),
  row 1 has (−1, `b`).
- The dependencies are 0 → 1 → 0.
- In ascending order, row 0 becomes (0, `ab`), which is correct. Row 1 then reads row 0's
  new D = 0 and gets D = −1. Ψ²(1) = 1 needs D = 0.

**The smallest at the levels that matter (P^{2|4}).** T = `aab`, so L = `baa`.
- Ψ = [1, 2, 0], with level-2 records (+2, `aa`), (−1, `ab`) and (−1, `ba`).
- The dependencies form one 3-cycle: 0 → 2, 1 → 0, 2 → 1.
- With a separate array: (+1, `aaba`), (+1, `abaa`), (−2, `baab`), all exact.
- In place ascending gets row 1's D = 0 and row 2's D = −1. Descending gets row 0's D = 0.
- Wrong either way.

**How common it is (measured).**

| | a = 1 | a = 2 (P^{2\|4}) | a = 4 (P^{4\|8}) |
|---|--:|--:|--:|
| Every L over {a, b, c}, n ≤ 8 (9,840): in place ascending is wrong | 6,121 | 7,897 | 7,569 |
| … wrong in both orders | 4,906 | 7,477 | 7,526 |
| … with a dependency cycle of length ≥ 2 (no order works) | 2,622 | 6,725 | 7,464 |

On the six real slices (`out_real.json`):
- 73–92% of the upgraded parents read a J that is itself upgraded;
- in place ascending would corrupt 34–43% of the records, for example 310 of 779 level-2
  records on block 0's first slice.

**For the economics note:**
- §4.2 note (f1)/(f2) and this check agree: the upgrade writes a separate array.
- Its §1, item 3, says the P^{4|8} upgrade works "in place". That wording should be
  corrected.
- The traffic model already costs a separate output (read S_a 8 B, gather S_a[J] 8 B,
  write 12 B).
- Whether the P^{4|8} peak (2.76 MB at the median, the same as P⁴) includes both S_4 and
  the upgraded array is not settled here (§9).

## 7. The cross-check against sample_sim.py (measured)

**The cross-check.** Three 20,000-byte real slices: block 0 at offset 0, block 145 at
300,000, block 90 at 600,000. `sample_sim.analyse` was run on each, exec'd unchanged.
This checker's builder and capped flag walker were run against the same 512
traversal-step starts, from the same numpy seed (`out_xsim.json`).
- **Interval counts:** m₁, m₂, m₄ and m₈ are equal on all three slices, for example
  1,862 / 2,247 / 2,672 / 3,264 on block 0.
- **Lookups per byte:** equal to the last digit for P², P⁴, P⁸, P^{2|4} and P^{4|8}, on all
  three. For example P^{2|4} is 0.30185 and P^{4|8} is 0.15595 on block 0.
- **The output** was exact (equal to T) in all 15 walks.
- **So:** the walk the economics note measured (0.296 and 0.150 lookups per byte on full
  blocks) is the walk this note proves correct, with the caps in the records rather than
  in the walker.

**One definitional difference: "long hops".**
- `sample_sim.py` counts a lookup on an upgraded row as long even when the hop is capped.
  This checker counts only hops that emit 2a bytes.
- At n = 20,000 with 512 segments, the shares are 0.65–0.69 here against 0.71–0.74
  there.
- **Deduced:** on a 900 kB block, the gap is at most one capped hop per segment, which is
  512 of about 135,000 lookups (0.4%). The note's 67–69% stands as a share of lookups on
  upgraded rows.

## 8. Limits: what this coverage doesn't exercise

- **Block sizes.**
  - Exhaustive coverage stops at n = 10 for records, n = 7 for walks and n = 6 (n = 8 over
    two letters) for every segmentation.
  - Random texts go up to n = 4,000, and real slices up to 6,000 bytes (20,000 in the
    cross-check).
  - No full 900 kB block was walked, which Python's speed rules out. Nothing in the
    lemmas depends on n.
  - What size does change is the mix: at n = 900,000 a segment is about 1,760 bytes,
    against tens here, so caps are rarer. The small cases are the harder ones for caps,
    not the easier.
- **Lane count and scheduling.** 32 lanes (the shipped count), with 2, 3 and 5 in places.
  The model is round-robin lockstep with per-lane buffers. Not modelled: the room walk's
  passes (`room`), buffer growth and rebasing, or prefetch. Those are exercised by the
  decoder's own tests, and they only move bytes, which Lemma 5 doesn't involve.
- **Field widths.** D is a Python integer: signed and unbounded.
  - A real record would hold D in 32 bits: two's complement, or u32 with wrapping add,
    which works mod 2³². |D| < n ≤ 900,000 < 2²⁰.
  - Not exercised: overflow, sign extension, or the T-vector's 24-bit link limit.
- **The walk-format record packing.**
  - The note's 12 B {D: u32, word: 8 B} has **no room for len**, and P^{a|2a} and caps
    need len (1–8) per record.
  - It must go somewhere: in D's spare high bits (D needs about 21 bits signed, leaving
    11), in a 13th byte, or in a parallel bit array.
  - This is not modelled, and the 12 B in the economics note's traffic figures should
    be read as 12 B plus len, packed into D.
  - For P^{2|4} the walk record is {D, 4-byte word, len}: 8 B with len in D's high bits.
- **The rank structure.**
  - Rank is `bisect` over interval starts, not the 64-row {count, bits} directory.
  - The `rank_left` mutant shows that a rank off by one is caught. The directory's own
    failure modes are not exercised: popcount masks, the last partial word, a break
    bit at row n − 1.
- **The construction order.**
  - The doubling finds J by rank, not by the image-order merge with strict-interior
    popcount counts. That construction was checked separately (portfolio §3.1(k), 8,160
    levels). Only its result, the partition and its records, is compared here.
  - The cap rows are found by LF through level 1 with a binary search, as the economics
    note costs them. They are not fused into the last merge.
- **The byte order of the 8-byte store.**
  - It is modelled as a byte string written at p: word byte 0 first. A real
    little-endian u64 store needs the word packed with byte 0 in the low byte.
  - The `word_reversed` mutant shows the checker catches the opposite order. It does
    not test a specific packing.
- **What isn't covered at all:** the MTF/RLE2 side, RLE1 decoding after the walk, the
  CRC, origPtr validation (the caller checks origPtr < n), multi-block streams and timing.

## 9. Unresolved, and what the economics note should change

- **The economics note's §1, item 3,** says the P^{4|8} upgrade is done "in place". It
  must write a separate array (§6). §4.2 (f1) already says so, and the traffic model
  costs it that way.
- **The peak for P^{2|4} and P^{4|8}.**
  - The note gives both the same median peak as P² and P⁴ (2.34 and 2.76 MB).
  - A separate output array means S_a (12 B construction records) and the upgraded walk
    array coexist during the upgrade pass. S_a can be dropped afterwards.
  - Whether the peak figure counts both arrays is not checked here.
- **The walk record needs a len field.**
  - The 12 B {D, 8-byte word} format has none, and P^{a|2a} and caps need one per
    record.
  - Packing it into D's high bits keeps 12 B. That is a design choice to make
    explicitly, and it costs a mask and a shift per lookup, which the walk model's
    16–24 instructions do not list.
- **Caps per start on real data reach the bound** (14 at K = 8), not the "at most 5"
  observed on random strings in portfolio §3.1(g)2. The count is still small per block
  (§3). The portfolio's sentence should carry the real-data figure.
- **Not shown here:**
  - that the image-order merge construction (§3.1(c′)3, (k)) gives these same tables at
    full size. It was checked separately, on random levels;
  - anything about cycles, which are Gate B and Gate C.
- **Nothing found contradicts N1's projected advantage.** The economics note's §8 lists
  "the upgrade's correctness is argued, not brute-force checked" as an assumption. On the
  coverage above, it is now checked and deduced. That assumption can no longer reverse
  the plan choice.

## 10. Files

Read-only scripts and outputs, kept in the scratchpad's `n1-check/` (not committed):
- `ref_naive.py`: the reference (the naive BWT, both naive Ψ, the expected outputs, and
  the decoder mirror of `bwt.rs`);
- `n1core.py`: the candidate (level 1, doubling, upgrade, caps, the two walkers, and the
  mutants);
- `n1run.py`: the suites and checks;
- `n1extra.py`: real slices, the `sample_sim.py` cross-check, the in-place analysis and
  the mutation tests;
- `cap_stats.py`: cap records per start on real slices;
- `run_all.sh`: the run, one job at a time (`cap_stats.py` was run after it);
- `out_*.json` and `log_*.txt`: each suite's results.

**Inputs:**
- `data/blocks.npz` (real slices only);
- `n1-analysis/sample_sim.py` (exec'd up to `main`, for its `analyse`);
- mibz2 `src/bwt.rs`, read for the semantics.

**Run:** `sh run_all.sh` from that directory.
