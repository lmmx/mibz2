# 2026-11-24 · MONODROMY 104 · The ordered-frontier lemma placed: it is the compose rule on scattered singletons under a round schedule, with c₁ unchanged, and by the team's own cost model it does not lower the unit price. The transformation walk → Cells → Open, characterised

## 1. The ordered-frontier lemma (channel entry #5)

**Lemma F** [proved; standard, it is the step of batched backward search]. For rows x < y with L[x] = L[y], LF(x) < LF(y); rows of a smaller character map below rows of a larger. So for a sorted set S, listing LF(S) by (character, order in S) is sorted.

**Iterated form.** LF^j is order-preserving on any set of rows that emit the same j-word. This is the scattered-set version of record 103's Lemma A: Lemma A is Lemma F plus adjacency.

### 1a. Not a yes [proved, by a cost-preserving simulation]

A frontier record after j steps is the rigid fact ({a}, j, t, w) on one row. One record-step composes it with the primitive fact at LF^j(a): **one compose with a one-row result, joined at LF^j(a).** The map (record, step) ↦ event is a bijection: nothing is unfolded, nothing merged. For G fields of d rows: G·(d−1) composes and G assembly links, **G·d events = the number of field rows = the walk's count on those rows.** c₁ is unchanged, and G1 (record 103) says it must be: every width-1 field row is some record's join row or anchor.

So the frontier is in 𝓡, by a simulation that preserves cost. This closes **this** bulk use of E1 (the class fact at k = 1, iterated); it does not close E1's bulk uses in general.

**Anchors are not the obstacle** [proved]. The anchors are the rows in the F-range of the delimiter character whose own L byte is hex: one contiguous range scan. A record retires when it emits a non-hex byte, so field lengths need not be known.

### 1b. A witness for the unit-price residual? [argued: predicted no]

Per event, against the walk:

| | walk | lanes (G independent walks) | ordered frontier |
|---|---|---|---|
| load of the row table | dependent | independent | independent, in ascending address order |
| bytes moved per event | 1 (the output byte) | ≈ 10 (toy-transfer) | ≥ 2 passes over a record of ≥ 8 bytes (count, scatter), plus the payload byte |

- The frontier differs from lanes only by address order and by the partition. Address order pays only when the row table is not cache-resident; the Coordinator's entry #4 cites toy-transfer (lanes: 5–14%) and the T² kill as showing misses are not the emit's cost on this machine at n ≤ 900k.
- Under emit ≈ α·events + β·bytes + construct, the frontier keeps α's count, cannot beat lanes on the load, and adds ≥ 16 bytes of β per event.
- **Prediction, with its falsifier:** frontier ≤ lanes − (partition cost), so no gain over master. Falsified if a frontier step measures cheaper per record than a lane step on an admitted block. Not requested: the team's data already bounds lanes.

### 1c. Is it Open's contraction on width-1 paths? [proved]

Yes. Open contracts thick strips by composing interval facts; the frontier contracts thin paths by composing one-row facts, G at a time in d rounds. The fact language and the rule are the same. "Ordered" is a property of the schedule (which memory the rounds touch), not of the facts.

**Obstruction write-up.**
- *Obstruction:* a scattered set transports its order, not its adjacency. A rigid fact is cheap because adjacency turns |I| destinations into one translation.
- *Scope:* every schedule of one-row composes, whatever its order.
- *What would escape it:* a description of LF^h on a scattered S using o(|S|) evaluations.
- *Transferable insight:* for S inside one character class, LF(x_{i+1}) − LF(x_i) = the number of that character in [x_i, x_{i+1}). So LF on a scattered set is exactly a set of **gap counts**: the aggregate input of record 101 with one block per element. Theorem 3 there says such a rule places a row only with at least ν′ blocks. The frontier and the block-histogram cell are the same object seen from two sides.

## 2. The transformation between the witnesses (the "24th problem" question)

**A witness** is a tiling of the cyclic orbit by paths of valid facts, plus the chain that reads it. Walk: n one-step tiles. Cells / Rauzy–Veech: tiles are first-return paths of intervals. Open: thick strips contracted, thin rows kept as one-step tiles.

**The transformation** [proved for these three]: *contraction along compose*. Replace two consecutive tiles by their composite, and do it for all rows of an interval at once whenever Lemma A keeps the interval inside runs.

| question | answer |
|---|---|
| what it preserves | the emitted cyclic word (T1); the orbit's cyclic order; that the tiles cover every row once |
| which obligation disappears | "evaluate LF once per row" on the interior rows of an interval: w rows discharged by one translation. Walk → Cells removes it on every wide interval; Cells → Open stops inducing where width ≤ 16, trading events for less construction |
| its invariant | the set J₁ of width-1 rows: each stays a join row or a link start under every contraction (G1). In interval-exchange terms, induction never removes a discontinuity; a width-1 interval is two adjacent ones |
| does it act on scattered ordered sets | it acts (Lemma F: order is carried) but discharges nothing: LF on scattered S costs one gap count per element. An obligation disappears only where LF^h on S has an o(|S|) description |

**The generative form of the question:** find a family of row sets S and a description class in which LF^h|_S costs o(|S|). Known members: intervals with translations (𝓡); classes with a common word (E1), which shatter to singletons past k ≈ 5 on random fields (record 96 G3). On text-random fields (record 103 §4) the carried words are independent at 4 bits a row, so any such description has ≥ 4|S| bits. **That bounds bits, not events** [proved; the gap is real]: it does not forbid a description with few, wide evaluations. That is the fine-input global residue again, now stated as a search target rather than a cell.
