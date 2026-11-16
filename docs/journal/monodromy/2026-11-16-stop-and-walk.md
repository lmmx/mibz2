# 2026-11-16 · MONODROMY 19 · Stop the induction at n/32 and walk the rest with ranked starts: 26% fewer word bytes for 0 to 9 more instructions a letter; at n/16, 37% fewer for 7 to 20 more. Byte-exact on all 233 blocks. A forced arm by the ring's own price of bytes

Programme: QUEUED 6914587350 ("Beyond Rauzy–Veech"), its second item: an induction that learns
output positions earlier. The words entry (`2026-11-16-words.md` §4) proved the right end holds
the whole writer's grammar to the end, for any order whose positions come from the walk's row
alone, and left walking as the one way out (Conj there). This prices that way out for memory.
§6.4 of the Rauzy–Veech entry priced it for time only ("closed as priced", under 3%).

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled · **Conj**.

## 0. Result

- **The curve (C, 233 blocks).** By m = n/8, 73% of the right end's steps are made, but only 51%
  of its word bytes; every original letter is touched; 14% of the letters are alive. The last 27%
  of the steps are Euclid on few long letters: 0.27 S steps for 0.14 r deletions.
- **Positions without the marked word (P).** Stop at domain [0, m). Choose K starts in it. Walk
  each start's segment of the induced cycle to the next start, summing word lengths. Rank the K
  starts along the contracted cycle. Then every point's word has its place. No grammar is needed
  past the stop, and nothing past it is ever made.
- **Built and checked (C).** Pass 1 (segment lengths, 8 lanes), ranking, and pass 2 (each word
  at its place, 8 lanes) reproduce the block, byte for byte, on all 233 Wikidata blocks:
  at m = n/8 with the single-lane pass 2, and at n/16 and n/32 with both. The lean pass 2 is
  also checked on 10 blocks at n/8 to n/256.
- **Price (C for the counts, M for the sum).** The cells not made and Cells' writer are traded
  for pass 2, and for pass 1 unless segments are staged.

  | stop | word bytes saved | instructions a letter, without pass 1 | with pass 1 |
  |---|--:|--:|--:|
  | m = n/16 | 37% (5.7 B a letter) | +7 to +20 | +29 to +42 |
  | m = n/32 | 26% (4.0 B) | 0 to +9 | +12 to +20 |
  | m = n/64 | 17% | −3 to +3 | +3 to +9 |

  The range runs from the lazy kernel's average cell to the eager one's. The tail touches no
  stream letter, so the eager figure is the likelier.
- **Against the record.** The ring of eight saved 4.5 B a letter for +38% induction instructions,
  about +69 a letter, and lost at 8 to 10 threads (`docs/journal/2026-11-16-lazy-ring.md`). Its
  entry's conclusion: "A cut of this size has to come nearly free in instructions to win here."
  This one is 88% of the ring's bytes at n/32, for about one eighth of its instructions.
  - *Why the writer may help further (Conj):* Cells' writer stores in rule order, scattered over
    the block. Pass 2 stores in place order within each segment.

## 1. The curve (C)

`stopwalk.rs.txt`, here: the right-end induction, with words as a DAG, checked at the end against
the block's length.
- *Checkpoints:* each time the domain falls below n · 2^(−k/2).
- *Recorded there:* steps, live letters, untouched letters, word bytes made so far, and word bytes
  reachable from the live letters. A word longer than 16 bytes costs 16 B; any other costs its
  length; an original letter's byte costs nothing.

Wikidata, medians over 233 blocks:

| k | m / n | steps / S | (S − steps) / m | live / r | untouched / r | word bytes made / final | reachable / final | m / r |
|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 2 | 1/2 | 0.107 | 0.206 | 0.896 | 0.811 | 0.046 | 0.023 | 8.0 |
| 4 | 1/4 | 0.550 | 0.216 | 0.313 | 0.048 | 0.261 | 0.106 | 4.0 |
| 6 | 1/8 | 0.732 | 0.251 | 0.140 | 0.000 | 0.515 | 0.264 | 2.0 |
| 8 | 1/16 | 0.794 | 0.388 | 0.130 | 0 | 0.612 | 0.337 | 1.0 |
| 10 | 1/32 | 0.852 | 0.554 | 0.105 | 0 | 0.723 | 0.449 | 0.50 |
| 12 | 1/64 | 0.900 | 0.748 | 0.085 | 0 | 0.813 | 0.536 | 0.25 |
| 16 | 1/256 | 0.978 | 0.692 | 0.027 | 0 | 0.960 | 0.678 | 0.06 |
| 20 | 1/1024 | 0.993 | 0.905 | 0.011 | 0 | 0.986 | 0.705 | 0.016 |

- **Points removed a step.** From k = 2 to 4: about 5. From k = 4 to 8: 6 to 9. From k = 10 to 12:
  about 3. Continuing past m is worth it while a step removes more points than
  a cell costs in walk points.
- **(S − steps)/m** is the most a walk point may cost, in cells, for walking from m to break even
  on the induction alone. That is §6.4's rule: steps(m) + m never rises (P, a step removes at
  least one point).
- **Live state is unchanged.** The touched letters peak at 0.318 r, by k ≈ 4. A stop at k ≥ 6
  does not lower the slots' peak, only the words'.
- **Prose** (`stopwalk_prose.jsonl`) is outside Cells' admission (r/n ≈ 0.37). Its curve is
  recorded and not used.

## 2. Positions from ranked starts (P)

**Setting.** The induction stopped with domain D = [0, m).
- *The induced map:* T, an exchange of the live letters; x ∈ I_α goes to T x ∈ J_α.
- *Invariant I* (§2 of `2026-11-10-rauzy-veech.md`): rotation(T x) = w_α · rotation(x).
- *So, in the text:* place(T x) = place(x) − |w_α| (mod n), and w_α occupies
  [place(T x), place(x)).
- *The cycle:* LF is one n-cycle, so T is one m-cycle on D.

**Lemma 1 (segments).** Let K ⊂ D be nonempty. For s ∈ K, its segment is s, T s, …, up to but not
including the next element of K. Its bytes: len(s) = Σ |w_α(x)| over the segment.
- (i) The segments partition D.
- (ii) next_K(s) is the first element of K that T reaches from s.
- (iii) place(next_K(s)) = place(s) − len(s) (mod n).
- (iv) Σ_s len(s) = n.

*Proof.* T restricted to D is one cycle, so K cuts it into |K| arcs. (iii) and (iv) follow by
summing invariant I along an arc and around the cycle. ∎

**So all places follow from one.** Fix one start's place. Then walking next_K over K, |K| steps,
places every start, and walking each segment places every point.

**Lemma 2 (the walk's row stays one point).** Cells cuts the walk's row out as a one-row letter μ.
- *μ never wins.* A winner's length drops by the loser's, which is at least 1, so a winner has
  length at least 2.
- *An equal step keeps one row.* If μ is the top letter, it is deleted, and the bottom letter,
  also of length 1, takes the mark in its word w_μ · w_B. If μ is the bottom letter, its length
  stays 1.
- *So the mark is on a letter of length 1 at every stage.* It has exactly one point x_μ ∈ D, and
  the row's place is place(x_μ) − |w_μ| + j. Here j is the row's offset in w_μ, which Cells'
  writer already finds by walking the marked rules. Put x_μ in K.

**What the stop removes.** Nothing after the stop needs a word: no further rule, no arena byte, no
`pos` pass over the grammar.
- *Places:* they come from K's ranking, not from the marked word. This is the way out of the
  words entry's bound (its §4, Conj), now priced.
- *Within one block:* every segment is independent in both passes. So one block's writer splits
  over a team with no shared state but K places. This is the numeration item of the programme:
  places as prefix sums over a sampled cycle.

## 3. The prototype (C)

In `stopwalk.rs.txt`, `hybrid` builds the stopped exchange's tables in top order:
- a bitmap of top starts, with a rank directory;
- for each letter, the image offset and the word's length;
- the words: a short word right-aligned in a 16-byte slot, a long one a rule of two words.

**K = 256 starts,** evenly spaced, with 0 among them. The block's place 0 is rotation(0); the
mark's offset (Lemma 2) is a constant shift not exercised here.

**Pass 1, eight lanes.**
- *Each step:* rank → letter; next point = x + offset; bytes += |w|; stop on a start's bit.
- *Then:* the K starts are ranked, and their places summed.

**Pass 2, eight lanes, three variants.**
1. **Single lane in chain order.**
2. **Lean.**
   - *Stores:* a short word is one 16-byte store ending at its place. It reaches up to 15
     bytes below the word, into bytes not yet written in its own segment. A long word expands
     right to left through its rules.
   - *Segment boundaries:* a store can reach into the segment below, which another lane
     writes. So each segment's top 16 bytes are kept as written and put back at the end.
     (The first version kept them once 16 bytes were written. Another lane's store had
     reached them in between, which broke 2 bytes on one block at n/32.)
3. **Checked on all 233 blocks** (`stopwalk_wikidata_k8.jsonl`, `_k10.jsonl`, field
   `hybrid` = [K, m, ns a point pass 1, ns a point pass 2, long words emitted, bytes equal]):
   pass 1's lengths and successors, and both pass 2s, against the block's text.

**Instructions, callgrind, each pass alone** (`instructions.txt`; every 24th block, 690,484
letters; hardware popcount; production on the same tool: 115.5 a letter to induce, 54.0 to
write, over the corpus's 11,090,867 letters, exactly the Pathfinder's 115.5):

| stop | points a letter | tail's share of S | word bytes made | pass 1, a point / a letter | pass 2, a point / a letter |
|---|--:|--:|--:|--:|--:|
| n/8 | 1.63 | 0.254 | 0.529 | 26.8 / 43.6 | 77.6 / 126.3 |
| n/16 | 0.81 | 0.192 | 0.628 | 27.3 / 22.2 | 118.6 / 96.1 |
| n/32 | 0.41 | 0.133 | 0.743 | 28.9 / 11.8 | 191.5 / 78.0 |
| n/64 | 0.20 | 0.086 | 0.833 | 31.4 / 6.4 | 326.5 / 66.5 |

Over all 233 blocks: 0.861 points a letter at n/16 and 0.432 at n/32. The word bytes made are
0.623 and 0.738 of the final, which saves 5.69 and 3.96 B a letter of the 15.5.

**Net (M):** saved = (tail share × cell cost) + 54.0, against pass 2, plus pass 1 if kept.

| stop | saved, eager cells / lazy average | net without pass 1 | net with pass 1 |
|---|--:|--:|--:|
| n/8 | 83.4 / 100.0 | −43.0 / −26.3 | −86.6 / −69.9 |
| n/16 | 76.2 / 88.8 | −20.0 / −7.4 | −42.1 / −29.5 |
| n/32 | 69.3 / 78.0 | −8.6 / +0.1 | −20.4 / −11.7 |
| n/64 | 64.0 / 69.6 | −2.5 / +3.2 | −8.9 / −3.2 |

Negative means more instructions than Cells. The eager column is the likelier one for the tail
(§0).
- *Pass 1 can go.* Pass 2 can write each segment into chunks taken from a bump pointer, as it
  walks. RLE1 then reads the segments in chain order. That costs one bound test a word.
- *Pass 2's headroom is the long words.* At n/32 they are 191 instructions a point, because a
  long word is expanded at every occurrence. Cells copies a repeated long factor from its first
  place instead.

**Local clocks** (container Xeon; not evidence): pass 1 about 2 to 3 ns a point with eight lanes,
11 with one.

## 4. For the kernel (Conj, for the Pathfinder)

**The arm.** The lazy kernel, run to m ≤ n/32 (or n/16), then build the tables (top order of
about 0.1 r letters), rank 256 starts, and run pass 2 with eight lanes. Pass 1 stays, or
segments are staged and RLE1 follows the chain.

**What it changes at 10 threads.**
- 26% (37%) of the word bytes are never made: 4.0 (5.7) B a letter of lazy's 29.5.
- There is no grammar pass.
- Stores go in place order within a segment.
- Instructions rise by 0 to 9 a letter at n/32. That is about 2% of a whole decode, by the
  ring entry's own arithmetic (+69 a letter there was +13% of the decoder).

**Why the record says this is the right side of the line.** The ring bought 4.5 B a letter at
+13% of the decoder's instructions and lost by 1.3 to 3.0% of wall time at 8 to 10 threads, with its loss
shrinking as threads were added. Its entry's estimate: 4.5 B is worth about 11% of CPU at 10
threads, were instructions time, "an order of size and no more".

**Where it is safe to say nothing.** Wall time at 9 and 10 threads may not be the workers' to give
(REVIEW 6906255284). The stop threshold is a free parameter, and the curve says n/16 to n/32.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-stop-and-walk/stopwalk.rs.txt examples/stopwalk.rs
cargo run --release --example stopwalk -- corpora/wikidata.bz2 1 > curve.jsonl   # seconds
KSTOP=10 cargo run --release --example stopwalk -- corpora/wikidata.bz2 1 > k10.jsonl  # all three checks
python3 docs/journal/monodromy/2026-11-16-stop-and-walk/summary.py.txt k10.jsonl
# instructions: build with RUSTFLAGS="-C target-feature=+popcnt,+bmi1,+bmi2,+lzcnt", then
KSTOP=10 valgrind --tool=callgrind --toggle-collect=stopwalk::pass2l ./target/release/examples/stopwalk corpora/wikidata.bz2 24
```

**Fields, one line a block.**
- `S`: steps.
- `eq`: equal steps.
- `alloc`: final word bytes.
- `peak_touched`: the touched letters' peak.
- `curve`: rows [k, m, steps, live, untouched, word bytes made, word bytes reachable from live
  letters, long-word edges on the reachable DAG].
- `hybrid`: as in §3.
