# 2026-11-24 · MONODROMY 115 · Enrichment along composition: what a step already reveals, which algebras carry a rider, and one cross-stage object, the per-symbol gap stream, which the MTF stage can emit and which is the inverse table in differenced form. It lowers memory and removes a pass; it does not lower Q

Answers channel entry #22 after reading CROSS-STAGE.md. Labels as before. **No cut in Q is claimed.** Prior art is given for each item; I have not searched the `research/*` branches.

## 1. What one step reveals beyond its own destination

| step | free by-product | what it is good for |
|---|---|---|
| visiting row x | the pair (L[x], F[x]): two consecutive text bytes, both known from the row | nothing new along a walk (F[x] was the previous output) |
| the build sweep at row x | the **whole prefix histogram** at x, not only the count of L[x] | the images of a cut at x under *every* symbol at once. Snapshots at the symbol boundaries give the 2-gram interval table in the same sweep [P]; each further sweep gives one more level |
| the build sweep at row x | the **gap** g(x) = x − (previous row with the same byte) | §3 |
| a run step (Rauzy–Veech) | the extent on which the step is a translation | this is the known case: forward-mode differentiation of the walk in the row variable, the derivative being 1 and the rider its interval of validity [A as an analogy; exact as a statement about intervals, record 103 Lemma A] |
| loading one cache line of the inverse table | the successors of the 15 lexicographic neighbours | they are the other occurrences of the current context; useful only if those rows are processed now, which is the lane/frontier schedule (record 114 sense E) |

## 2. Algebras a rider can live in, under record 113's scheme

| algebra | rider | composes by | what it buys | cost |
|---|---|---|---|---|
| dual numbers | (row, extent) | intersection of extents | Rauzy–Veech | none on lone rows: extent 0 |
| free monoid | the emitted word | concatenation | macro-edges, k bytes per addressed access | k sweeps, or e per in-place sweep (records 110, 112). **This is the only rider here that cuts Q** |
| ℕ^Σ under + | prefix histogram | addition over adjacent intervals | k-gram interval tables; "virtual" successors of any cut | free for k = 2; one sweep per further level |
| differences of the inverse table | the gap sequence of each symbol | prefix sum | the inverse table at about a quarter of its size | §3 |
| permutations of the records | π at a template offset | composition | record 113 relation C | the pairing |
| a fingerprint ring | a hash of the word | affine map | checks, not transport (record 114 sense L; N-18) | fails the brief's own test |

**Why no cheap rider cuts Q** [A]. To advance a row by k steps a rider must contain FL^k of that row, about log₂ n bits that are new for each row. A single sweep can hand that to a row only when the row's successor was already processed, the descent chains of record 112. Histograms, gaps and extents are all functions of the column near x; FL^k is not.

## 3. The cross-stage object: per-symbol gap streams

**The fact** [P]. Let tt be bzip2's inverse table: tt restricted to the bucket of symbol c lists, in increasing order, the rows where L = c. So within a bucket tt[y+1] − tt[y] = the gap between consecutive occurrences of c in L. **The discrete derivative of the inverse table is the gap sequence of the column.**

**Where the gaps are already known** [P]. Inverse MTF holds the recency list. Keeping one register per symbol with its last row gives g(x) at the moment L[x] is produced. RLE2's zero runs are gaps of 1.

**The object.** For each symbol, a stream of gaps, one byte each with an escape for large ones, plus an absolute row every 16th entry. The MTF stage appends to |Σ| streams through cursors. **No separate table-construction stage remains**: what LF construction would build is what MTF emits.

**Under the referee:**

| | Q | D | P | S |
|---|---|---|---|---|
| standard: build tt, then walk | n | n | a build sweep and the walk | 4n bytes |
| gap streams from the MTF stage, then walk | n, plus an in-line prefix sum of ≤ 15 bytes per step | n | the build sweep is absorbed into the MTF stage | about n to 1.25n bytes [A; NEEDS M-22] |

- **Not a lower Q.** It changes which cache level the walk's accesses land in (a 900,000-row table falls from 3.6 MB to about 1 MB) and removes one pass over 4n bytes. That is unit price and P, the residual (b), and a whole-decoder effect only the bench can state.
- **It passes the brief's test:** acquired in the stage that already runs, through cursors, and it restates nothing: a gap is a fact about the column, not a text position.
- **Prior art.** The differenced inverse table is Sadakane's compressed suffix array (ψ stored as gaps); bzip2's own small-memory mode shrinks tt by another trick. CROSS-STAGE.md lists "occurrence counts accumulated alongside MTF" as known; gaps are the differences of occurrence positions, not counts, and I do not know whether the records have them. **Novelty is not claimed.**
- **How it meets Open.** Open keeps edges for retained rows only. The same differencing applies per symbol among retained rows; whether that helps is not obvious, since Open's edges carry words.

## 4. Ranking (all senses and riders, by how far each can go next)

1. Gap streams (§3): exact, cheap, measurable now; unit price and P.
2. Landmark induction (record 114): D; measurement approved.
3. The pairing (record 113 relation C): the only named route to Q; no construction.
4. The histogram rider (§2): free 2-gram table; I have no use for it that is not a macro-edge.

## 5. NEEDS

- **NEEDS M-22** on the six admitted blocks: for each byte value, the gap sequence of its rows in L. Report the total size of the streams at one byte per gap with an escape above 254 and an absolute row every 16 entries; the fraction of escaped gaps; and the same restricted to Open's retained rows. Compare with 4n and with Open's edge bytes.
