# 2026-11-16 · MONODROMY 25 · Stop-and-walk reopened again (PROTOCOL §11a): the i9 found the single-lane walk's latency to be the cost. Eight staged chains remove the serial dependency for +13 to +16 instructions a letter, byte-exact on 233 blocks; two chains halve it for +12 with no other change

Divergence report: the Coordinator's NOT ADOPTED (LOG on #145; relayed 16:45 on 16 November).
- *Where it lost:* the production plan, one worker a core, was 1.0 to 1.5% slower at 8, 9 and
  20 threads and level at 10 and 12.
- *What was real:* the memory relief, a dose 0.947 of lazy's.
- *Where it paid:* only with two workers a core, at 0.986 at 12 and 0.965 at 20.
- *The cost:* one thread was 1.064, which is the single-lane walk's latency. Reading the stopped
  exchange was 18.9 a letter.
This is the reopening round on those two terms.

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled · **Conj**.

## 0. Result

1. **Two chains (record 23, P; C: byte-exact on 233 blocks; +11.9 a letter).**
   - *What:* up and down from x_μ, the one anchored point.
   - *Effect:* the critical path halved. No other change to the decoder.
2. **Eight staged chains (this record, P; C: byte-exact on 233 blocks).**
   - *Seeds:* the points with y mod 2^b = 0. A landing is a mask test, and a seed's segment index
     is y >> b, so there is no table.
   - *How the chains run:* eight interleave in one thread. Each walks T⁻¹ up from its seed, writing
     words left to right into its own chunks, carved from one buffer by a bump pointer. It ends
     on landing at a seed and records that seed as its successor.
   - *Why nothing is overwritten:* no chain writes outside its own chunks, and a word goes into a
     chunk only with |w| + 16 bytes free.
   - *The places:* not computed at all. RLE1 and the CRC read the segments' runs in cycle order,
     starting at x_μ's segment.

   | seeds every | segments a block | writer, a letter | against one chain | staging buffer ÷ n |
   |---|--:|--:|--:|--:|
   | 2^8 points (1 KB chunks) | 165 | 51.6 | **+13.3** | 1.132 |
   | 2^10 (512 B chunks) | 43 | 53.0 | +14.7 | 1.091 |
   | 2^11 (512 B chunks) | 22 | 53.9 | +15.6 | 1.086 |

   *How it was counted:* callgrind, hardware popcount, β = 1.5, every 8th block.
   - *The waste:* nearly all of it is the |w| + 16 left unused at each chunk's end, and it falls
     with bigger chunks.
   - *The memory:* the staging buffer stands in for the output buffer, so its overhead, 0.05 to
     0.13 n or about 0.8 to 2 B a letter, comes out of the stop's 5.0 B.
3. **The reader (record 23, C; M).** The stopped exchange at β = 1.5 is 0.121 r live letters, of
   which 0.0001 r are untouched, and 0.672 r points. A reader linear in that should cost about
   5 a letter, not 18.9.

## 1. Why eight chains need staging and two do not (P)

**Places along the cycle are a prefix sum, a list ranking.** The only point whose place is known
without walking is x_μ (Lemma 2, record 19).
- *Two chains need nothing more:* a walk from x_μ in each direction knows its places, and the two
  write only into the gap between them (record 23).
- *Any third chain starts at a point whose place is unknown.* So it either learns it from a
  counting walk first, which is the eight-lane design's pass 1, or does not need it: it writes
  where its place is irrelevant, its own chunks, and leaves the order to the reader.

**RLE1 can be that reader.** It consumes the block's bytes once, in order, so it can follow
(segment, run) lists as well as one array. The order is the seeds' successor chain, which the
chains write as they finish.

**Lemma (staged chains).** Let K = {y ∈ D : y ≡ 0 mod 2^b} ∪ {x_μ}. Each seed's segment, walked
upward to the next seed, is written contiguously within runs of its own chunks. Concatenating
the segments along the successor chain from x_μ's gives the block from x_μ's place.
- *Each segment is a contiguous stretch of the text:* places along the upward walk are
  consecutive.
- *The segments partition the cycle:* this is record 19's Lemma 1, with K as the starts.
- *No write leaves its chunk:* a word's 16-byte stores reach at most 15 bytes past it, and it is
  placed only with |w| + 16 free. ∎

A first-place copy may read another chain's chunk. The source is a whole word written earlier in
the single thread, so it is complete.

## 2. What would decide it (M; the clock is the Pathfinder's)

**At one thread** the arm lost 6.4%, for the walk's latency. Eight chains turn the walk from
latency-bound to throughput-bound. Two chains halve the latency.

| | instructions, a letter | latency | change elsewhere |
|---|--:|--:|---|
| arm now | +12.3 (measured) | one chain | none |
| two chains | about +12 more | half | a top-order rank for the down chain |
| eight staged chains | about +13 to +16 more | hidden behind the other chains | RLE1 and the CRC read runs; the buffer is about 1.05 to 1.13 n |

**The reader** would take about −14 a letter back if it is linear in what it reads.
- *With two chains:* about +10 against shipped.
- *With staged chains:* about +12.

Both are within the old +20 gate, with the latency halved or hidden.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-staged-chains/stopwalk.rs.txt examples/stopwalk.rs
BETA=1.5 cargo run --release --example stopwalk -- corpora/wikidata.bz2 1   # pass2s, emit_two, emit_k checked on every block
SEEDBITS=10 CHUNK=512 BETA=1.5 valgrind --tool=callgrind --toggle-collect=stopwalk::emit_k ./target/release/examples/stopwalk corpora/wikidata.bz2 8
```

**Field** `hybrid`: [K, m, ns a point of pass 1, ns a point of pass 2 (local), long words, bytes
equal, staging buffer bytes, n].
