# 2026-11-18 · MONODROMY 54 · Codex's bispecial-factor result verified independently: all four claims hold. Its support lives on at most r − 1 contexts. A periodic block is as far from dendric as Cassaigne's identity allows, and the extension graphs are the Rauzy–Veech induction read by contexts

**The request.** The Coordinator's CHECKPOINT 6974410090 (QUEUED via the #346 inbox), for draft PR
#553 at 111ff9ac: `docs/journal/2026-11-18-bispecial-factors.md` and its directory.
- *What I did not read:* the Researcher's verdict.
- *What I did not post:* anything on #553.

**Labels:** **P** proved · **C** counted (by me, here) · **R** reproduced from Codex's artifacts ·
**M** · **Conj**.

## 1. Verdicts

| claim | verdict | basis |
|---|---|---|
| 1. Separator dependence | **VERIFIED** | P, and C: all 70 texts at m = 8, h = 4 |
| 2. Support-only sufficiency | **VERIFIED** | P; C: my own encoder/decoder, 12,840 round trips; R: the frozen file |
| 3. The 4,744 / 4,745 witness | **VERIFIED** | C: independent hashing and an independent LCP count |
| 4. The measured claims | **VERIFIED where re-run; the corpus is not re-run** | R on the frozen block |
| 5. Cost | **What it shows is stated below; it doesn't show a cheap path** | C: one timing |

## 2. The claims

**1. Separator dependence (P, C).**
- *Cut after `#`:* the single phrase's spelling carries the choice. 70 distinct dictionaries.
- *Cut after `|`:* the phrase multiset is fixed, {#|, 0|×4, 1|×4}, a single multiset across all 70.
  The token BWT carries the choice: 70 distinct token BWTs.
- *Both cuts satisfy the RLE1 condition:* no `##`, no `||`.
- *The point stands:* the split between spelling and order depends on the section, so no invariant
  "spelling information" exists to be priced.

**2. Support-only sufficiency (P; C).** The protocol is a breadth-first refinement of classes of
futures, with Weiner links (left extensions).
- *Each class gets its right children when created:* from a predecessor's table row, or all of them
  when the context has one left extension.
- *A unique right extension:* reuses the class, since I(aw) = I(awb).
- *The invariant:* stable LF preserves the order of the a-preceded occurrences.
- *Termination:* any two distinct futures differ at a finite depth, and the split tree has at most
  2D − 1 nodes.
- *Every terminal class has exactly one predecessor:* an infinite periodic future determines its
  necklace.

**My implementation (C).** Written from the journal's prose, not from Codex's code. The encoder
answers "does a·w·b occur"; the decoder sees only the alphabet, n, the mark and the bit tape.
- *Round trips:* exact on all 12,840 words: every ternary word up to n = 8, and 3,000 random words up
  to n = 60, 30% of them powers.
- *Collisions:* no two distinct necklaces of length 8 share an (alphabet, tape).
- *The one edge case:* a one-letter alphabet. It needs a special case and outputs aⁿ.

**Periodicity.** One primitive cycle of period d gives each rotation n/d times. LF maps I(f) onto
I(pred f) bijectively, so the multiplicities are equal along a cycle.

**Several cycles.** Columns with this support correspond to the positive vectors (m_j) with
Σ d_j m_j = n. Distinct vectors give distinct columns by the Gessel–Reutenauer and extended-BWT
correspondence: a column is a multiset of primitive necklaces. **Only the multiplicities remain.**

**The mark.** It is the distinct-future rank, an integer below D, and it carries no counts.
- *From bzip2's primary,* for a block that is one necklace (every valid bzip2 block): rank =
  ⌊p / (n/D)⌋, where D is known after decoding. So bzip2's primary suffices.
- *For several cycles:* the primary depends on the multiplicities (the journal's `aab` / `abb`
  example is correct). The rank does not.

**Two outputs from identical inputs?** Not for one necklace: the output is unique. With several
cycles the *marked output* is unique, but the full column is not (the multiplicities).

**3. The ambiguity witness (C).**
- *Rebuilt independently:* the reference column at rows 138,927–138,929 holds `6d 6e 65`.
  Reordering to `65 6d 6e` and inverting at primary 264,045 gives one 899,931-cycle, a primitive
  text that is no rotation of the original.
- *Compared by my own double rolling hash (mod 2⁶¹ − 1 and 2³¹ − 1):*

  | length | distinct substrings in the union | counts that differ | total difference |
  |---|--:|--:|--:|
  | 4,744 | 899,853 | 0 | 0 |
  | 4,745 | 899,856 | 4 | 4 |

  These are Codex's numbers exactly.
- *Their comparator is also exact:*
  - the pair IDs come from one shared HashMap, so equality is checked on keys;
  - two spans of 2^j ≥ length/2 cover each substring.
- *The threshold, counted independently by suffix order and Kasai LCP:*

  | spectrum length | mixed context intervals |
  |---|---|
  | 4,744 | one: 3 rows, `6d6e65` |
  | 4,745 | one: 2 rows, `6e65` |
  | 4,823 | none |

  The maximum LCP is 4,821. Swapping the 4,745 interval gives cycles of 442,958 + 456,973. So
  **4,745 is the first length that identifies the block up to rotation, verified.**

**4. The measured claims.**
- *Reproduced (R), rustc 1.99.1:*
  - `support.rs` on the 67,024-byte file reports 899,931 terminal futures and writes raw and column
    with the stated SHA-256s (f87e5caa…, db876422…);
  - both are byte-equal to the reference at bf88365b;
  - `contexts.rs support` rebuilds the file byte-identically from the published column.
- *The file's size checks:* 535,773 bits / 8 + 52 = 67,024.
- *Not re-run by me:* the 431-block corpus (209,700,196 bytes through RLE1 and CRC), the
  504,258 / 22,261 counts, and the prose control.
- **Size against the compressed block:** 67,024 B against the 41,618 B bzip2 block, **1.61×
  larger**, and EXT1 is 92,119 B (2.2×). This is a re-presentation, not compression: it holds
  about 10.5 bits a BWT run (r = 51,184).

**5. Cost (C, M).** The evidence shows identifiability. It does not show a cheap path from the
column to the support, which in bzip2 decoding is decoder work.
- *The encoder side:* the encoder builds a rank index over L and explores 504,258 right-special
  contexts. No timing of it is reported.
- *The decoder side:* `support.rs` took **0.675 s** on the frozen block here (one run, this
  container). A plain inverse of 0.9 MB at the measured 255 MB/s is about 3.5 ms. So the
  support-only *decoder alone* is about 190× slower, before any encoder cost.
- *As the journal says,* decoder throughput is unmeasured. These numbers rule out nothing about a
  future algorithm. They only say the present one is an identifiability tool.

## 3. Dendric shifts and the induction records (M; Conj where marked)

**Cassaigne's identity is the journal's cycle-rank equation.** For a bispecial w, the bilateral
multiplicity m(w) = E − L − R + 1 satisfies Σ_{|w|=k} m(w) = p(k+2) − 2p(k+1) + p(k) (Cassaigne
1997). Summing over k for a *periodic* language, where p stabilises at D, gives Σ_w m(w) = −(s − 1).
With m(w) = β_w − (c_w − 1), that is the journal's Σ(c_w − 1) = Σ β_w + s − 1, and on the frozen
block 32,328 = 32,240 + 88.

**Why a block is never dendric.** A dendric (tree-set) language has every extension graph a tree:
β_w = 0 and c_w = 1, so m(w) = 0 throughout and p(k) = (s − 1)k + 1 grows forever.
- *A periodic block must lose s − 1 in total,* through disconnected (weak) bispecials, so it is
  never dendric.
- *The frozen block is far from it:* 20,296 forest-supported graphs, but 32,240 chords. Each chord
  is a free bit of support that a dendric language would not need.

**What dendric shifts would give (Conj).**
- *The planar dendric case:* a regular interval exchange's coding, which is exactly the dynamics
  record 19 et seq. study for LF.
- *There, the extension trees are fixed by the two orders:* the induction's top and bottom
  permutations. The support tables would be redundant given the Rauzy–Veech path.

So the support file measures how far the block is from an IET coding, bispecial by bispecial.

**How it relates to my induction records (M).**
1. **The same r.** The tables sit at bispecial contexts, and every left-special context's interval
   contains an L run boundary. The deepest such contexts number at most r − 1. Here there are
   22,261 bispecials against r − 1 = 51,183.
   - *The same r appears in:* #553's phrase-BWT bound (q ≤ r) and in my run-length IET's letter
     count. It is the r-index measure.
   - *So:* the unweighted description costs O(r) tables, and it removes exactly the weights that
     the induction's lengths carry.
2. **The extension graphs are the induction read by contexts.** A Rauzy–Veech step on LF's IET
   resolves one coincidence of interval ends. In the context picture that is one split of a class
   at a bispecial. The winner/loser type is the shape of the extension graph. My induction runs
   on run-length *letters* (r of them, about 1.8 r steps, record 21). The support decoder runs on
   distinct *futures* (D = n classes, 504,258 splits).
   - *Conj:* the support decoder is the induction without lengths, unfolded to full depth, which is
     why it is 190× slower.
3. **The return theorem as a diagnostic.** In a dendric shift every factor has exactly s return
   words (Berthé et al.). #553's phrase dictionary is the return words to the delimiter: 2,716 for
   `"`, against s = 89. That excess is a direct measure of non-dendricity on the delimiter.
