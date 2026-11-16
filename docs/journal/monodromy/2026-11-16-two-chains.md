# 2026-11-16 · MONODROMY 23 · The stopped exchange's walk splits in two for free: from the one anchored point, up by T⁻¹ and down by T, meeting once. Byte-exact on 233 blocks, +12 instructions a letter, half the chain. More than two chains need a counting pass or staging. The exchange read at β = 1.5 is 0.12 r letters: a reader should cost about 5 a letter, not 18.9

Question: the Coordinator's, after Gate 1 passed (+10.8, then +12.3; LOG 6922045659). The built
kernel walks the stopped exchange as one dependent chain, about 10% of one thread's time, and
reads the exchange at 18.9 instructions a letter against the model's 10.
- Can the walk be split into k independent chains without the boundary overwrites that sank the
  eight-lane design?
- Is there a cheaper representation of the exchange to read?

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled · **Conj**.

## 0. Answers

1. **Two chains, free of any counting pass (P; C byte-exact).**
   - *The anchor:* x_μ, the one-row letter's point (Lemma 2 of record 19), has a known place.
     Write it as place(0) ≡ place(n).
   - *Chain up:* walk T⁻¹ from it, writing each word at [pu, pu + |w|) with left-aligned 16-byte
     stores. They reach only above.
   - *Chain down:* walk T, writing each word at [pd − |w|, pd) with right-aligned 16-byte stores,
     reading 16 bytes that end at the word's end. They reach only below.
   - *Why there is one boundary:* both chains reach only into the gap [pu, pd). They meet once,
     at a word boundary, because places of points are word boundaries. A word is written exactly
     only when the gap is shorter than the word plus 16.
   - *First places:* a letter's word is expanded once, wherever it first lies, and either chain
     copies it from there. Down-chain copies go chunk by chunk from the end. An over-read lands in
     the gap.
   - *The cost:* **+11.9 instructions a letter** over the one-chain writer (50.2 against 38.3 at
     β = 1.5, every 8th block, callgrind). The dependent chain is half as long. The
     alternation and the gap test are most of the +11.9.
2. **More than two chains need a counting pass or staging (Conj, with the reason).**
   - *No other free anchor:* the stopped exchange gives the cycle only as a map. A point's place is
     the sum of word lengths along the cycle from x_μ, and I find no other point whose place
     follows without that walk. This is list ranking.
   - *Why x_μ gives two chains:* it anchors both directions.
   - *Beyond two, either:*
     - a counting pass, as before (16 a letter on the kernel at n/32); or
     - staging: each chain writes its segment into its own chunk, and RLE1 and the CRC follow
       the chunks in cycle order. Seeds can be the points y with y mod 2^b = 0, so the end test
       is a mask and no seed bitmap is needed. There are no overwrites, since no chain writes
       outside its own chunk.
3. **The object to read is small (C), so a reader should cost about 5 a letter (M).** At β = 1.5,
   over 232 stopped blocks:
   - *Live letters:* 0.121 r. Still untouched in the streams: 0.0001 r, 0.1% of the live ones.
   - *Points:* 0.672 r. The rank bitmap and its directory are 0.19 × m bytes, 6 to 13 KB a block,
     and the records about 0.12 r × 12 B.
   - *A reader linear in these:* about 30 instructions a live letter plus m/64 bitmap words is
     about 4 to 5 a letter. **The kernel's 18.9 says its reader does work in proportion to
     something larger.** A likely candidate is the streams' consumed extent: up to r entries
     scanned to find 0.0001 r untouched. That is for the Pathfinder to profile.
4. **A per-point successor table is not clearly cheaper (M).**
   - *What it saves:* one load a step instead of rank's two loads and a popcount.
   - *What it costs:* the table is 8 B × 0.672 r, about 270 KB on a 51 k-letter block, so it sits
     in L2. The rank structure is about 6 KB of bitmap and 25 KB of records, in L1.
   - *Latency:* about 14 cycles a step either way. Not recommended.

## 1. The two chains (P)

**Setting.** As in record 21: D = [0, m) is the stopped domain and T its exchange. For x ∈ I_α,
w_α lies at [place(T x), place(x)) with place(T x) = place(x) − |w_α| (invariant I). x_μ is the
anchor, with place(x_μ) = 0 ≡ n after the rotation by `SHIFT`.

**Up.** y₀ = x_μ, p₀ = 0. Take x = T⁻¹(y), found by the bottom rank. Then w_{α(x)} lies at
[p, p + |w|), the next point is y ← x, and p ← p + |w|.

**Down.** x₀ = x_μ, q₀ = n. Take α = α(x), found by the top rank. Then w_α lies at
[q − |w|, q), the next point is x ← T x, and q ← q − |w|.

**Lemma.** The two chains together write every point's word exactly once. The bytes either chain
writes outside its words lie in [p, q), and are overwritten before the chains meet.
- *Why the words are disjoint:* the up chain covers [0, p) and the down chain [q, n), so they
  are disjoint while p ≤ q.
- *Why they meet once:* each step moves p up or q down by a word, and both are places of
  points of D. Every point of D is visited once, by one chain, because the cycle from x_μ read
  forwards and backwards meets once. This holds when T is one cycle on D, the hypothesis of
  record 19 §2: a block whose text is a power (abab) has several cycles, and its blocks fall back
  (correction, 16 November).
- *Why the stray bytes are safe:* up stores reach at most 15 bytes above p + |w|, and down
  stores at most 15 below q − |w|. Both lie in [p, q) whenever q − p ≥ |w| + 16, which the
  writer tests. Otherwise it writes the word exactly. ∎

**Copies.** A first place is recorded when its word is written, so a copy's source is written
before it is read. A copy reads 16-byte chunks: the up chain from the start, the down chain from
the end. An over-read falls before or after the source word, and its bytes land in the gap.

## 2. Counts (C)

- **Byte-exact:** all 233 Wikidata blocks at β = 1.4, 1.5 and 1.6, against the general writer,
  which is itself checked against the block's text.
- **Instructions:** callgrind, hardware popcount, β = 1.5, every 8th block, 0.638 points a
  letter.

  | writer | a letter | a point |
  |---|--:|--:|
  | one chain (`pass2s`, record 21) | 38.3 | 60.1 |
  | two chains (`emit_two`) | 50.2 | 78.7 |

- **The stopped exchange at β = 1.5** (232 of 233 blocks stop):

  | | per letter |
  |---|--:|
  | live letters | 0.1212 r |
  | untouched (still in a stream) | 0.0001 r (0.1% of the live; at most 0.5% in a block) |
  | points m | 0.672 r |

## 3. What would change the time (M)

**Halving the chain.** The walk's 10% of one thread is latency, a rank and a record load each
step. Two chains halve the critical path. At 10 threads every worker has its own walk, so the
saving is each worker's own, about 5% of a thread, against +12 instructions a letter, about 2% of a
decode.

**Cheaper alternation, an untested estimate.** A lockstep loop could take one up step and one down
step each turn while the gap is long, then finish exactly. That would remove the per-step choice,
which is most of the +12.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-two-chains/stopwalk.rs.txt examples/stopwalk.rs
BETA=1.5 cargo run --release --example stopwalk -- corpora/wikidata.bz2 1   # emit_two checked on every block
BETA=1.5 valgrind --tool=callgrind --toggle-collect=stopwalk::emit_two ./target/release/examples/stopwalk corpora/wikidata.bz2 8
```

**Field:** `bstop_live` = [live letters, untouched letters] at the β stop.
