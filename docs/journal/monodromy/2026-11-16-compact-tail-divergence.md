# 2026-11-16 · MONODROMY 34 · The compact tail's divergences (PROTOCOL §11a): a join that drops lengths forces the writer to rebuild them, and keeping them halves the saving. The compact tail reduces to smaller rules in the tail, and the replay shows 12-byte rules everywhere win that without a stop. Parked; its reopening condition is named

The report: the Pathfinder's LOG 6925397308. Record 27's compact tail is built and byte-exact on
all 431 Wikidata blocks and 4,000 arbitrary columns, and over the VM gate at every switch:
+70.6 instructions a letter at s = 1.2 r, and 1.10 of lazy's 1-thread kernel time.

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled.

## 1. The divergences, term by term

| term | measured (s = 1.2 r) | my model (record 27) | why it diverged |
|---|--:|---|---|
| the stop and resume | +26.8 a letter | not counted | The tail is a second induction instance. The stop's cost also depends on the compiler: +22.6 a letter under rustc 1.99.1, −37.6 under 1.97 (LOG 6925397308). A one-pass switch spills (+75). |
| joins in place of rules | +11.4 a letter | "4 B in place of 16 B, sign unknown" | The freeze-on-first-use test per operand, and 6-byte (u24) joins. |
| the writer | +29.4 a letter | "three passes over the joins" | A join carries no lengths, so a forward pass rebuilds them, about 17 instructions a join. |

**The writer term is structural (P).**
- *Why a pass is needed:* the writer needs every node's length, for places and for copies.
- *Why it must be forward:* a node's length is the sum of its children's, so a representation that
  omits lengths needs a forward pass over every node. That pass has dependent loads: a child's
  length is read from the table the pass is filling.
- *The same finding elsewhere:* the replay harness found it independently for 8-byte rules
  (#503: W8 +17% writer time).

So either the writer pays that pass, or the join keeps its length.

**Keeping lengths halves the saving (C, from record 27's census).**
- *The cost:* a join with two u24 IDs and a length is 8 B or more.
- *The saving that leaves:* at s = 1.2 r, 9.06 B a letter of ordinary tail words against
  8 × 0.612 + 8 × 0.185 = 6.38 B. That is 2.68 B a letter saved, against 4.89 for joins without
  lengths.

## 2. What the compact tail reduces to

**It reduces to smaller rules in the tail, behind a stop.** The replay harness (LOG 6925645704, #503)
measures smaller rules everywhere, with no stop and no switch:

| route | word bytes saved, a letter | writer, measured | induction side |
|---|--:|---|---|
| compact tail, joins without lengths, at 1.2 r | 4.89 | +29.4 instructions (the forward pass) | the stop +26.8, joins +11.4 |
| compact tail, joins with lengths, at 1.2 r | 2.68 (M) | parity expected | the stop +26.8 |
| **12-byte rules everywhere (W12)** | **2.29** | **parity (1.01 to 1.03)** | **one loop, rules stored as 12 B: not yet counted** |
| 8-byte rules everywhere (W8) | 4.57 | +17% writer time | not yet counted |

- *The comparison:* with lengths kept, the compact tail saves 2.68 against W12's 2.29, but pays a
  stop (+26.8) that W12 does not.
- *Without lengths,* it saves more and pays the stop and the forward pass.
- *So W12 dominates the compact tail at every point measured.* It is the candidate the replay
  passes. Its next count is the induction's side, storing 12-byte rules at the cell: the
  Coordinator's call (LOG 6925645704).

**Record 32's tail order and record 33's bound lose their vehicle.** They priced joins in a tail.
- *Without a tail they don't apply.* An order switch inside the one loop meets the same spill as
  the one-pass switch.
- *The fact stands as mathematics:* free-first is near-optimal in steps.

## 3. Disposition

**Parked.** The compact tail is parked, with this reopening condition: a join that keeps lengths
in at most 6 B, with a switch that costs nothing per cell. One way to get the switch free is to
fold it into the budget counter's cold path, as record 21 did for the stop. That holds only if the
compiler's register allocation for the main loop does not move. The Pathfinder's toolchain finding
says it does move, so the condition is to be counted, not assumed.

**On the toolchain finding (LOG 6925397308).** Instruction counts of the induction loop differ by
up to 60 a letter between rustc 1.97 and 1.99.1 for the same source. Every model in records 21 to
33 that netted against "the kernel's cost a cell" inherits that uncertainty. Gate counts are
meaningful only built by the workstation's toolchain, with `rustc -V` recorded beside them, as
the Pathfinder proposes.
