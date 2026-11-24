# Record 120: thin-strip acquisition without searches, and the depth-2 schedule costed (channel #28)

*MONODROMY, 24 November 2026. Labels: proved / argued / measured / conjectured. Nothing was run on the admitted blocks; §4 is arithmetic on figures others reported and is labelled conjectured. No cut of total Q below h₁/2 is claimed.*

## 1. Where the acquisition cost actually sits

Record 57 verified Open's own constructor: right-end induction on the wide strips, 11,215 endpoint steps for 3,448 strips on its block. That is the scale of the strip count, far under h₁. So "acquisition is 0.9–4·h₁" is not Open's cost. It is the cost of contracting **thin** strips eagerly with a binary search per endpoint (Neon CL3, CL4). The question is whether thin runs can be removed without that.

## 2. Contraction by label order needs no search (proved; checked)

Let the strips be disjoint row intervals, each inside one run, and G(y) = y on retained rows, G(y) = G(LF(y)) on strips. A **piece** is an interval sitting inside one strip.

**Proposition A.** Keep the pieces in one stream per label of the strip they sit in, each stream in row order. Process the labels in increasing order, and each stream in order. Then:
- the images of one stream's pieces are increasing, so one forward cursor over the layout of that label's bucket finds every image: no search;
- the parts of the images that fall in strips, appended to the stream of that strip's label, arrive in row order.

*Proof.* LF is increasing on the rows of one label, which gives the first claim. Images of label c lie in bucket c. Buckets are disjoint and ordered by label, so appending in label order, and in row order inside a label, appends in row order. ∎

So each level of contraction is one streamed pass over the live pieces, with one cursor per label. These are the cursors of the successor schedule (B_P).

**Checked** on 1,800 synthetic systems (`.strips.py.txt`): G exact against brute force, every stream sorted, every cursor forward only. Deepest cases: 35 rounds on templated text, 87 on periodic text.

**Cost.** Addressed accesses: none. Passes: one per level of depth, each over the live pieces only. Depth is the length of the longest repeated context, so eager contraction to the end costs many passes. That is the price CL4 paid in searches instead.

**Prior art.** Finding the run that holds a run's image by merging, not searching, is the move structure (Nishimoto and Tabei 2021; cited from memory). The label-ordered rounds are that merge iterated.

## 3. Lazy rule: contract thin strips one level, retain what is deeper (proved)

Contract the wide strips as Open does. For thin strips, set

S* = { thin-strip rows y : LF(y) is retained or lies in a wide strip }.

Remove S* and retain every other thin-strip row.

- **Exact:** for y in S*, G(y) is LF(y) or the wide-strip map at LF(y). Nothing in S* lands in S*, so depth is one.
- **Acquisition:** round one of Proposition A alone. One streamed pass over the run records, no addressed access, no search.
- **Payload:** a retained row landing in S* emits two labels; the second is the strip's label, read off the same cursor.
- **What it keeps:** thin-strip rows that land in thin strips. A second round would take those that land in S*, at one more pass.

This is the "lazy and shared" acquisition asked for: shared because all strips use the label cursors, lazy because depth is cut off by retaining rows, not chased.

## 4. What the lever is worth (conjectured; arithmetic only)

Removing rows lowers m, and the emit links are a fixed fraction of m (about 1/3 at the best order on record, record 118).

- **Floor of the lever:** every run keeps at least one row, so m ≥ r, the number of runs, and r ≥ h₁.
- **Block 0, from the figures in chatgpt-closure.md:** m = 89,874 and h₁ = 35,116. If thin runs average 2.5 rows and four in five of their interior rows fall in S*, then m falls to about 65k and links to about 0.6·h₁, from 0.83–0.93·h₁ now.
- **At the floor,** with r about 1.65·h₁, links are about 0.55·h₁.

So this thread can move total Q from about h₁ towards 0.6·h₁ with acquisition near zero. It does not reach h₁/2 at a failing fraction of 1/3.

## 5. The depth-2 schedule, costed (argued)

- **Building R²** reads the record at R(x). That is the byte schedule: 2,148 restarts on block 0, within 256 cursors. The pair schedule is not needed to build R².
- **The (byte, successor-byte) schedule** (6.3k segments) serves only a further composition, R⁴ = R²∘R². By Lemma W (record 119) it is exact, and its classes are the label pairs present. Those exceed 256 cursors, so in Π each gather is addressed. With a cursor table held in first-level cache the restarts are the only irregular part.
- **Payloads:** flat words of 2 bytes a row for R², 4 for R⁴.

| route, block 0 | build, addressed | emit links | total, in h₁ |
|---|---|---|---|
| walk R | 0 | 89,874 | 2.56 |
| R², byte schedule | 2,148 | 44,937 | 1.34 |
| R⁴, cursor table allowed | 2,148 + 6.3k | 22,469 | 0.88 |
| R⁴, strict Π | 2,148 + 89,874 | 22,469 | 3.26 |
| one in-place sweep (M-17) | 2,148 | 32,833 | 1.00 |

R⁸ needs four-label classes, which record 119's law puts in the sparse regime. **The depth-2 schedule does not beat the in-place sweep under strict Π, and beats it by about a tenth with a cursor table.** Its use is in combination with §3: both scale with m.

## NEEDS

`NEEDS M-27` On the six admitted blocks, at Open's width rule: thin runs and their interior rows; the share of those rows landing on retained rows, in wide strips, in thin strips; m after removing S*; emit links after one in-place sweep on the reduced map. Rule, fixed before data: m after S* above 0.85 of m on at least four blocks closes §3.
