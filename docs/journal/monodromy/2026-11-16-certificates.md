# 2026-11-16 · MONODROMY 28 · Itinerary certificates on real stopped exchanges: a third of the starts share a column of 256 bytes or more; the coarser induced exchange's letters are exactly those columns; and the "cheap conjugacy" for weighted ranking is the induction itself, continued as the compact tail

Source: LOG 6924013198, the Applied Dynamical Systems relay (Ghrist, ADS Vol. I ch. 1 p. 3; ch. 5
pp. 27 and 29). It reaches the same certificate as the topology relay and record 26:
J = [max(a_j − D_j), min(b_j − D_j)). Three questions for me:
- J's width as k grows, and whether a tower view predicts long shared columns;
- §7, the common-child groups;
- a cheap conjugacy for weighted ranking.

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled.

## 1. How J shrinks (C, every 8th Wikidata block)

**What is counted.** For every point p_{k0} of the stopped walk, J_k is the set of starts that share
its k-step itinerary, an interval of offsets about p_{k0}. Means are over starts, so a wide letter
weighs by its points.

| β | mean width, k = 1 / 2 / 4 / 8 / 16 / 32 / 64 | share with width ≥ 2, k = 1 / 4 / 8 / 16 / 32 / 64 |
|---|---|---|
| 1.4 | 826 / 445 / 193 / 81 / 25 / 4.8 / 1.6 | 0.92 / 0.76 / 0.63 / 0.47 / 0.24 / 0.08 |
| 1.5 | 624 / 258 / 82 / 25 / 6.3 / 2.0 / 1.4 | 0.86 / 0.64 / 0.48 / 0.27 / 0.11 / 0.05 |
| 1.6 | 355 / 77 / 26 / 5.8 / 1.9 / 1.3 / 1.2 | 0.78 / 0.49 / 0.27 / 0.11 / 0.05 / 0.03 |

**By bytes, the question's 64 to 256:** the share of starts whose J still holds another start once
the itinerary has covered 64, 128 and 256 bytes.

| β | 64 B | 128 B | 256 B |
|---|--:|--:|--:|
| 1.4 | 0.64 | 0.50 | 0.32 |
| 1.5 | 0.61 | 0.49 | 0.33 |
| 1.6 | 0.55 | 0.44 | 0.31 |

**What it says.**
- *The singleton risk is real for most starts but not for all.* A third of them share a column of
  256 bytes or more. Record 26's tracked parse used only partners already visited and walked, so
  it took fewer.
- *The bound in the relay* (the k-step partition has at most 1 + k(L − 1) cells) gives a mean
  width over cells of at least m/(kq) ≈ 5.5/k at β = 1.5. That is true but uninformative: the long
  columns sit in the wide letters, which the start-weighted means show.

## 2. Which bases share columns (P): the coarser induced exchange's letters

**Setting.** Let D' = [0, m') ⊂ D = [0, m) be a later stop of the same induction, so the first
return of T_D to D' is T_{D'}.

**Lemma.** For x ∈ D', let k(x) be its excursion's length in D-steps, the return time to D'. Let
β be x's letter in D', with interval I'_β. Then I'_β ∩ D is contained in the certificate
J_{k(x)}(x). Every start in x's D'-letter follows x's D-itinerary for the whole excursion, which
covers |w'_β| bytes.

*Proof.* T_{D'} is a translation on I'_β, and it is the composition of the D-steps along the
excursion, so every point of I'_β follows the same D-letters for k(x) steps (record 22's tower
lemma, read at the level of points). ∎

**So the Kakutani–Rokhlin tower over D' predicts the columns:** bases in one D'-letter share a
column of height k and |w'| bytes, and the column's width is λ'_β.

**Counted, from record 22's two-level runs** (words at 1.5 r; the coarser exchange at β₂):

| β₂ | top calls a letter | bytes a call |
|---|--:|--:|
| 1.6 | 0.318 | 43 |
| 1.7 | 0.144 | 88 |
| 1.8 | 0.045 | 193 |

The 64 to 256 bytes asked for are the letters of the exchange at about 1.7 r to 1.8 r.

**Getting those letters costs the induction from β to β₂, lengths only.** That is §6's compact tail
(record 27), which continues the same induction and stores joins in 4 B.

## 3. The conjugacy remark (P)

**What weighted ranking is.** Places along the stopped cycle are prefix sums of word lengths in
cycle order.

**Neither order the exchange holds is that order.** The top and bottom orders are orders of
intervals, while the cycle is the orbit; a map giving each point's cycle position *is* the
ranking.

**The conjugacy is the induction.** Each Rauzy–Veech step conjugates the first return on a smaller
domain to the larger map, with weights (word lengths) composing additively along excursions.
Continued to one point, it reduces the ranking to the root. The Cells writer's first-place pass
then propagates places down the hierarchy, last rule to first: weighted ranking by the existing
passes.

**So the cheap conjugacy asked for is to continue the induction past the stop.** It costs no
rule bytes if the tail is stored compactly. That is §6 (record 27): at a 1.2 r switch, 32% of the
word bytes go, and the existing passes rank.

## 4. §7, the common-child groups

These are record 24, lazy tags. A rule already shares the winner's word by pointer, so grouping
can remove at most 3.3 B a letter. That is below §6.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-certificates/stopwalk.rs.txt examples/stopwalk.rs
BETA=1.5 cargo run --release --example stopwalk -- corpora/wikidata.bz2 8
```

**The field.** `hybrid[8][0]` = [mean width at k = 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64; the
share with width ≥ 2 at those k; the share still ≥ 2 after 64, 128, 256 bytes]. The output is
`certificates_wikidata_every8.jsonl`.
