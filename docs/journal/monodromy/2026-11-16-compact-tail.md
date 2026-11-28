# 2026-11-16 · MONODROMY 27 · A compact tail grammar keeps the whole induction and the Cells writer, with no walk: switch at 1.2 r to joins of two u16 IDs (u24 where they overflow) and 32% of the word bytes go, 4.9 B a letter. And the Ghrist relay's §3, §5 and §11, answered from records 24 and 26

Source: LOG 6923818459, Louis's ChatGPT Pro answer drawn from Ghrist, *Elementary Applied
Topology* §7.8 and §10.9, routed to me as §3, §5, §6 and §11. Its claims are ChatGPT's; below each
is checked against the records and counted.

**Labels:** **P** proved · **C** counted (exact for its input) · **M** modelled.

## 0. Result

**§6, the compact tail grammar: confirmed, and the strongest of the four (C, all 233 Wikidata
blocks).**
- *The scheme:* at cell s, every word that is already made becomes a frozen terminal. Each later
  cell stores its join as two IDs, with no bytes and no rule.

| switch at s | tail joins t | frozen terminals referenced | ordinary tail words | saved, 4-byte joins (u16) | blocks with IDs > 65,536 | saved, u16 or else u24 |
|---|--:|--:|--:|--:|--:|--:|
| 1.0 r | 0.812 r | 0.268 r | 11.14 B a letter | 5.75 (38%) | 90 | **4.98 (33%)** |
| **1.2 r** | 0.612 r | 0.185 r | 9.06 | 5.13 (34%) | 30 | **4.89 (32%)** |
| 1.3 r | 0.512 r | 0.143 r | 7.77 | 4.58 (30%) | 15 | 4.47 (30%) |
| 1.4 r | 0.413 r | 0.132 r | 6.45 | 3.74 (25%) | 3 | |
| 1.5 r | 0.315 r | 0.120 r | 5.00 | 2.79 (18%) | 1 | |

- *Bytes counted:* 4 B a join (two u16), or 6 B (two u24) where a block's IDs do not fit, plus 8 B
  a referenced terminal (its descriptor). A byte terminal is one of 256 reserved IDs.
- *Against ChatGPT's model at 1.5 r (2.81):* 2.79, counted.
- *Against stop-and-walk at 1.5 r (5.00 B, record 21):* 4.89 at 1.2 r. It needs no walk, no reader
  and no second emitter, and every rule is still written whole by the existing writer.

**§3, excursion certificates: this is record 26 (P), and the feasible traversal is counted (C).**
- *The equivalence:* J = ⋂ (I_{i_j} − s_j) is the set of starts sharing a k-step itinerary.
  Record 26's run lemma is the same statement for a pair of visits at offset δ: δ stays in J
  while the shifted points stay in their intervals.
- *The singleton risk, measured as ChatGPT asks (a feasible traversal, not certificate widths),
  at β = 1.5 with sources walked:*
  - runs cover 67% of points, with a mean length of 3.2 points;
  - calls fall to 0.53 of the points, at 39 B a call;
  - the unconstrained ceiling is 0.32 calls a point.
- *So the certificates mostly die within about 3 steps.* That is the singleton risk, realised.
  Long certificates exist but are rare: 1.1% of run points lie in runs of more than 64.

**§11, shared-factor groups along winner streaks: this is record 24 (lazy tags).**
- *The ceiling:* 3.3 B a letter. A rule already shares the winner's word by pointer, so only
  single-use rules can fold (2.44) and only the copied winner bytes in short words can go (0.92).
- *Against §6:* below it.

**§5, the forest writer over a logged walk (M).** It keeps the walk, and the walk's cost is what
failed on the i9 (record 25 and the Pathfinder's finding). §6 reaches the same memory with no walk
at all, so I rank §5 below §6. Not counted here.

## 1. The compact tail (P)

**What changes at cell s.** Nothing in the induction's order, letters or lengths. Only how a new
word is stored:
- *Before s:* a 16-byte rule, or arena bytes, as now.
- *From s on:* a join (id_A, id_B) appended to a table. The new word's handle in its letter
  becomes the join's ID.

**IDs without enumerating anything.** An operand whose handle is still an ordinary descriptor is
frozen when first used: it takes the next terminal ID, its descriptor goes to the dictionary, and
its letter's handle is rewritten to the ID. A word instance belongs to one letter (record 18
§2), so later uses of the same word read the ID from the letter, and no table from descriptors to
IDs is needed. Original letters' byte words take the 256 reserved IDs.

**The writer (P, as a statement about the same grammar).** The tail grammar is the same grammar,
re-encoded. Its joins form exactly the subtree above the frozen terminals.

The writer's order:
1. **Lengths:** one forward pass over the joins, from the terminals' descriptors.
2. **The mark:** a forward pass marks a join iff a child is marked. The marked terminal is the
   one-row letter's word (Lemma 2, record 19).
3. **Places:** the reverse pass over the joins from the final root, as `write_cells`' first-place
   pass. This seeds each terminal's first place.
4. **The prefix grammar:** its existing reverse pass seeded from those terminals, then its forward
   copy pass.
5. **The tail's bytes:** the joins' forward pass, each child's bytes copied from its first place.

The per-join scratch (length and first place, 8 B) can live in the dead top slots after the
induction, where it fits.

**u16, and the overflow (C, then P).**
- *How often IDs pass 65,536* (256 + frozen + t): in 30 of 233 blocks at 1.2 r, 3 at 1.4 r, 1 at
  1.5 r.
- *Recovery without a second pass over the block:* when the counter reaches 65,536, re-encode the
  joins so far into 6 B (u24) once, a cold path of at most 65,536 joins, and continue in u24.

## 2. What the kernel must count (M; the Pathfinder's)

**Per tail cell:**
- *Gone:* the 16-byte rule store, or the up-to-16-byte arena copy.
- *Added:* a 4-byte join store, and a test of each operand's handle for an ordinary descriptor,
  which freezes it.

Whether the induction loses or gains instructions here is the kernel's question.

**At the end:** three passes over t joins, plus the dictionary, in place of the full grammar's
rules from s on.

**Memory at the peak:** the prefix grammar to s, the joins, and the dictionary. At 1.2 r that is
4.9 B a letter fewer, 32% of the words.

## Reproduce

```
cp docs/journal/monodromy/2026-11-16-compact-tail/stopwalk.rs.txt examples/stopwalk.rs
BETA=1.2 cargo run --release --example stopwalk -- corpora/wikidata.bz2 1   # field tailc
```

**Fields** (`tail_wikidata.jsonl`):
- `tailc` = [tail joins t, frozen terminals referenced, distinct byte terminals, 256 + frozen + t];
- `made_at_switch`: ordinary word bytes made by cell s.
