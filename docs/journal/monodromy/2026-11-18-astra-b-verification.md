# 2026-11-18 · MONODROMY 73 · Astra B (#602, collective boundary dynamics) verified independently. Its residual constructions, rebuilt by direct first-return walking, reproduce every residual count exactly: the equal-byte contraction saves 39,549 cuts (3.20%) and width-2 tiles save 45,338, on eight blocks. Both are exact against LF inversion. The proofs hold. The negative cost finding holds on every block, at 2.5–2.9× the credit

**The request.** The Coordinator's CHECKPOINT 6993052513 (QUEUED 6992960979): an independent verification
of Astra B on #602 at c42e1c69, `docs/journal/2026-11-18-astra-b-collective.md`.
- *Conduct:* I did not read the other Astra branches.

**Labels:** **P** · **C** · **I**.

**Method.** I extended my model (`mixedcheck.rs.txt` beside this record) with a *generic* residual
induction, independent of the study's queue and range constructors.
1. **Choose a section** (a row mask). For each section row x, walk T = LF until it returns to the
   section. That gives the target rank and the LF-departure word, with the mark's offset inside the
   word.
2. **Coalesce** consecutive ranks that have consecutive targets and equal words.
3. **Run my Cells induction on the resulting bands,** a band-based version of record 66's model, and
   compare the rotated marked word with my independent LF inversion.

The sections:
- **Equal-byte:** all non-internal rows, L ≠ F.
- **Width-2 clean tiles:** remove each aligned tile that lies inside one constant-byte run, has an
  aligned image and isn't the primary's tile.

## Verdicts

| claim | verdict | basis |
|---|---|---|
| The first-return identity R = N∘S, and its word | **EXACT** | P (checked) |
| The marked-spelling convention | **EXACT** | C: exact on 14,209 small columns, 12,192 of them short-period, with marks inside multi-byte words |
| Rotation: L = 1ⁿ⁻¹0 at primary n − 2 has T(x) = x + 1 mod n, output 1ⁿ⁻²01, and n − 1 Cells cuts | **EXACT** | C at n = 4,096: 4,095 cuts, output reproduced |
| Equal-byte formulas: the split; the internal image is one interval | **EXACT** | P; C: the residual exact everywhere |
| The equal-byte contraction saves 39,549 cuts (3.20%) over ten blocks | **EXACT** | C |
| 664,730 range intersections | **Not rebuilt** (a constructor-specific count). Its *output* matches mine exactly | C |
| The staged arrays cost more than the credited eager and lazy savings on every sampled block | **EXACT** (the arithmetic) | C |
| Width-2 tiles save cuts on eight blocks | **EXACT** | C |
| Correctness | **EXACT** against independent LF inversion. The libbz2 digests were not rerun | C |

## 1. Proofs (P, C)

**The first return.** S = Q⁻¹T moves exactly the departures where T ≠ Q, and E is those plus the
primary, so S(E) = E. From e, the first step is Q(S(e)). Off E, T = Q. So the first return is
N(S(e)), with gap g(S(e)), and the word reads F along Q.

**The tile-lane order and the clean-tile paths.** These are P by the stated arguments, and their
outputs are confirmed below by the exactness of the residual.

**The equal-byte split.** For an internal row of bucket c, T(b + j) = b + k + j − e(j). So the
internal images form one interval, arrows on either side of the k-th exit point the same way, and
the queue invariant follows.

**Counterexamples.**
- *`11000`:* the zero bucket's entrances map out of order. This is why the residual must carry each
  entrance's identity. My walk does so directly.
- *`1101000` at primary 3, width 2:* support falls 7 → 5 while cuts rise 2 → 3. This is a case of the
  26 small columns where tiles *raise* cuts; 3,156 lower them.

**Exactness (C).** Both residual constructions reproduce LF inversion on **all 14,209** small marked
columns, short periods and marks inside multi-byte words included.
- *Orientation:* this needs the words in the induction's own orientation, the reverse of LF departure.
  That is a convention: the study writes words in LF departure order, and either is exact once
  applied consistently.

**The rotation family (C).** At n = 4,096:
- T is exactly x + 1 mod n;
- the forward output is 1⁴⁰⁹⁴01;
- Cells makes **4,095** cuts.

The modular shortcut replaces those cuts with three fills.

## 2. The real blocks (C)

Ten prespecified blocks, every 24th of the large blocks from ordinal 0. Exact against LF inversion on
all ten, for both constructions.

| | frozen | ten blocks |
|---|---|---|
| Cells cuts | 92,570 | 1,236,053 |
| Internal equal-byte rows | 15,756 | 157,654 |
| **Equal-byte residual cuts** | **89,708** | **1,196,504** (−39,549, **3.20%**; per block 3.05–3.55%) |
| Equal-byte residual rules | 28,683 | 359,396 |
| Equal-byte residual records | 48,931 | (every block equals `queue_residual_records`) |
| Width-2 support rows | 439,965 | |
| **Width-2 residual cuts** | **93,746** (+1,176) | **1,190,715** (−45,338) |
| Width-2 bands | 53,430 | 715,805 |
| Width-2 rules | 35,211 | 384,872 (the study: 384,871; a 1-rule convention difference) |

- *Where the tiles save:* 8 of 10 blocks. Frozen (+1,176) and ordinal 168 (+1,895) are worse;
  ordinal 144 saves 13,462. All exactly as stated.
- *The study's queues:* I computed the same residual by walking, not with their FIFO-and-division
  queues. Identical residual cuts, rules and records on every block confirm their construction's
  output, though not its constructor counts (fragments, intersections).

**The screens.**
- *Best global rotation:* agrees on **26,413** rows (exact).
- *Tile disagreement lower bounds,* widths 2–64:

  | | B = 2 | B = 4 | B = 8 | B = 16 | B = 32 | B = 64 |
  |---|---|---|---|---|---|---|
  | Mine, rigid lane shifts | 219,982 | 252,043 | 247,804 | 266,576 | 274,324 | 295,765 |
  | The study's `shift_min_errors` | same | same | same | same | same | same |

  The journal quotes the weaker *arbitrary-permutation* column (250,289 … 284,516), as its wording
  says. Both are valid bounds; mine reproduces the stricter one.

## 3. The cost finding (C)

Recomputed from the study's `ledger.csv`. Staged accesses = 2 × (source + residual records), so
frozen is 2 × (48,853 + 48,931) = 195,568.

| population | staged accesses | credited saving | per block |
|---|---|---|---|
| Eager-admitted (8) | 1,661,348 | 635,595 | 2.46–2.75× |
| Lazy-admitted (9) | 2,140,008 | 792,511 | 2.55–2.88× |

- **The staged arrays exceed the credit on every block, in both populations, by 2.5–2.9×.**
- *The credit is generous:* it counts geometry and word operations plus a setup credit per removed
  band, and grants the residual its bottom order for free.
- *So the negative result is robust:* the representation loses before any queue or index cost is
  charged.

## 4. Reading (I)

**What the contraction saves.** It removes about 3% of cuts: all equal-byte excursions, which are
Cells' cheapest cuts, the same-byte self-joins. The writer is unchanged: 9,628 copies of 735,582 bytes
on frozen.

**What tiles do.** They help where long parallel aligned runs exist (ordinal 144) and hurt elsewhere.

**Why neither pays.** Both residuals have to be *materialised* before induction, and that alone costs
2.5–2.9× the saving.

**Where this sits.** This agrees with the pattern of records 70–72. Cells' ordered linked fronts are
an efficient implicit schedule. Collective shortcuts are exact, but their constructors cost more than
the cuts they remove, except in degenerate families like the pure rotation, which Wikidata does not
contain (best rotation agreement 26,413 of 899,931 rows).

**What this decides:** nothing in production.
