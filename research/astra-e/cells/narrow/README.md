# Width-specialized eager links

This arm tests the slot-index representation at the five-byte immediate grammar's original cutoff. The initial census already supplies the exact number of letters after splitting the marked row. If that number is at most 65,536, every index fits in sixteen bits. A bottom slot therefore needs four bytes, and the top metadata becomes two ordinary sixteen-bit indices and a 32-bit row width. The rest of the top slot still contains its full word descriptor. Per-slot length loads through an arena were the defect of the earlier 16-byte top design. The current layout keeps both the word's length and payload together. [Earlier experiment](https://github.com/lmmx/mibz2/blob/9c31461df7bef24cecf79348545b27ee26846835/docs/journal/2026-11-14-cells-slot16/rejected.md)

The dispatch occurs once after the checked census. A narrow bottom pair stores the predecessor in bits 0–15 and the top partner in bits 16–31. A top metadata word uses bits 0–15 and 16–31 for those indices and bits 32–63 for width. Width can exceed 65,535 in this arm and remains exact. After construction an index can only name an earlier initial slot or a promised front slot, so every later index fits as well. Zero is an ordinary slot index. Large blocks with more letters use the full-width instantiation. There is one reusable top table, one output allocation and one rule vector across the two instantiations.

On 176 of 228 admitted large Wikidata blocks (158,399,093 rows and 9,692,429 letters), the narrower layout applies. It removes 38,769,716 logical bytes from bottom initialization across those blocks. This is an exact sum of table extents, with no automatic credit for address-formation instructions, transfers or peak RSS. A reused destination vector can extend to the full output even on a compact block. The other 52 admissions use the wide layout. All 228 outputs matched Segmented; all slots, cuts, rules, gate outcomes and top/rule capacities matched the original control on each of the 233 large inputs. [Per-block raw census](geometry.csv) · [Geometry check](geometry-test.log) · [Small oracles and boundary fields](property-test.log)

The arm can be scored directly:

```sh
bash research/astra-e/score.sh narrow-links-round1 research/astra-e/cells/narrow/candidate.rs --with-opend
```

To regenerate it and verify its identity, run:

```sh
research/astra-e/cells/narrow/prepare.sh target/astra-e-narrow packed
cmp target/astra-e-narrow/candidate.rs research/astra-e/cells/narrow/candidate.rs
```

The candidate's SHA-256 is `0889129d7120386a246bb89b674786224a18b72ffdb6c5d09095c0b0dcd735a8`. The same index patch can be applied to the 6–16-byte literal-table arm with `research/astra-e/cells/narrow/prepare.sh target/astra-e-narrow-hybrid hybrid`. Any combined benefit needs its own check and paired score.
