# 2026-11-24 · MONODROMY 109 · Is there a "supported row"? The only certificates are adjacency (already Open's contraction, and it filters wide rows only) and histogram forcing (a few percent). Width-1 rows carry no filter in the rigid calculus; the recursion that exists is list-ranking contraction, at one event per removed row

Answers channel entry #10. The MST preprint is not read; I use only the two mechanisms as the entry states them.

## 1. Why the MST filter has no direct analogue [proved]

The MST filter works because the output is a **subset**: n − 1 of m edges, and a certificate (heaviest on a cycle) proves an edge is *not in the output*, so it is never processed again. Inversion's output is a **bijection**: every row's byte appears exactly once. No row is "not in the output". A row can be dropped from further work only if its byte and placement are *implied* by rows that remain. So the question becomes: which implications are certified by what Open has built?

## 2. The certificates that exist

Let R be Open's retained map: R(x) is the next retained row after x on the orbit, with the word of the contracted strip between them. R is one cycle on the m retained rows.

| certificate | what it certifies | cost | which rows it filters |
|---|---|---|---|
| **Adjacency in a run** (record 103 Lemma A): x, x+1 in one run have adjacent images while their paths stay in common runs | x+1's whole path from x's | O(1) per interval boundary | rows of wide runs only. This **is** Open's contraction; applied to R itself it is a second round on the retained runs of width 2–16 |
| **Histogram forcing** (Neon's fill_remaining, N-19) | the last unwritten slots, when one symbol remains | O(1) | ≤ 6.4–6.8% of h₁ as an oracle (Neon, cited) |
| **Positional label from an observation** (record 107, Lemma 18.1) | pos(x) for x fixed across the fibre | the observation | bounded below only by the ≈10⁴-cell necessary condition (record 108 §3) |

**For a width-1 row there is no certificate in the rigid calculus** [proved]: G1 (record 103) puts an event at every such row, whatever was built before. A fact through a width-1 row has a one-row domain, so nothing built for another row implies it.

## 3. The recursion that does exist [proved, record 99]

"Contract, filter, recurse" on R is list ranking by contraction: remove an independent set of retained rows by splicing each into its predecessor's edge, recurse on the rest, T(m) = T(cm) + O(m). It is linear and it is a recursion. But a splice **is** a compose joined at the removed row: one event attributed to that row. The MST round discards an edge for the price of a comparison shared across a batch; the BWT round removes a row for the price of that row's own event. Depth falls, the count does not.

**A second contraction round therefore helps only the term c₁W** (rows of retained runs of width 2–16): adjacent retained rows that travel together under R can be composed as one interval. That lowers c₁ from m toward h₁ + (retained wide runs and their splits), never below h₁. Whether it pays in time is what Open's LIMIT = 16 already decided empirically; I ask for no count.

## 4. The offline/online separation, placed [argued]

Komlós-versus-Pettie is "m queries against a fixed structure cost less together than one at a time". Here the batch is "the position of every retained row". Offline it is list ranking: O(m) work in passes. Online it is the walk: O(m) work in dependent steps. **In work and in events the two are equal**, so there is no count separation to find inside 𝓡, and G1 says so. The separation that can exist is in the **price of an event** (sequential passes against dependent loads): the labelled residual (b), whose one local witness is lanes at 0.55–0.78× of emit.

**Obstruction write-up.** *Obstruction:* the output is a bijection, so there is no slack to filter; implication is the only filter, and for a width-1 row nothing implies it rigidly. *Scope:* 𝓡, and any filter certified by adjacency. *Escape:* an implication for a scattered set from a shared computation, i.e. the schematic fact of record 105 §1, now with a named model: Komlós-style batch verification. *Transferable insight:* ask of any borrowed speed-up whether its source problem has output slack (m ≫ n). Where the output is a permutation of the input, the borrowed mechanism can only change depth or unit price.
