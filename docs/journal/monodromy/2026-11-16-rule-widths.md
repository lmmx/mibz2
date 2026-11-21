# 2026-11-16 · MONODROMY 37 · Exact widths for packed rules: under the production budget a child fits in 40 bits, so a 10-byte rule fits every admitted block (3.43 B a letter saved, against W12's 2.29). Its decode cost is a kernel count

The W12 line, item 1 (the Coordinator, 18:51): exact bounds on reference and length widths per
block, against W12's child of a 27-bit reference, the mark and a 20-bit length (#503).

**Labels:** **P** proved · **C** counted (exact for its input).

## 1. Bounds (P)

**The setting.** Production Cells refuses a block that would make more than
budget = ⌊n/25⌋·4 + ⌊(n mod 25)·4/25⌋ ≤ 0.16 n cells (`src/cells.rs`, `CellsKernel::attempt`; the
forced route's budget is n). With n ≤ 900,000:

| quantity | bound | bits |
|---|---|--:|
| rules (long words), at most the cells | ≤ 144,000 | **18** |
| arena bytes: a cell adds at most 16 | ≤ 2,304,000 | **22** |
| a child's length, at most n − 1 | < 900,000 | **20** |
| a short child's length | ≤ 16 | **5** |

**A child tagged by its kind therefore fits in 40 bits (P):**
- *a rule child:* kind, mark, an 18-bit rule index and a 20-bit length;
- *an arena child:* kind, mark, a 22-bit arena offset (relative to the arena's base) and a 5-bit
  length.

So two children make a **10-byte rule**, valid on every block the production budget admits.
- *The forced route* (budget n) needs the wide format. The format is chosen at layout, from the
  budget, as the Pathfinder already does for W12's offsets.

**Without dropping the length, 10 bytes is the floor (P, for this encoding).** A rule child needs
18 + 20 bits plus its tag, 38 + 2 = 40. Any narrower layout must drop or rebuild the length, and
that is W8's +17% writer time (#503) and record 34's forward pass.

## 2. Counted maxima (C, all 233 Wikidata blocks; `widths_wikidata.jsonl`)

| quantity | the largest in any block | bits |
|---|--:|--:|
| rules | 62,349 (at most 0.670 r) | 16 |
| arena bytes | 889,975 (at most 7.82 r) | 20 |
| the longest child | 899,985 | 20 |

The corpus uses 2 bits less than the budget's bound for references, and all 20 for lengths.

## 3. Bytes (C, from #503's rule count G/r = 0.5716)

| layout | rule bytes a letter | saved against 16 B |
|---|--:|--:|
| 16 B (now) | 9.15 | |
| W12 | 6.86 | 2.29 |
| **10 B (this record)** | **5.72** | **3.43** |
| W8, lengths rebuilt | 4.57 | 4.57, at +17% writer time |

## 4. What is not known: its decode and store cost (a kernel count is needed)

**What is different from W12.** A 40-bit child is a tag test, then one of two unpacks. That is a
branch or a select per child, where W12's unpack is a shift, a mask and an OR.

**Whether that keeps the writer at parity, and what storing it costs at the cell,** I cannot price
by record 36's rule: only a count in the kernel's layout can. It goes to the Pathfinder as "kernel
count needed": a fourth writer, W10, in #503's replay harness.
