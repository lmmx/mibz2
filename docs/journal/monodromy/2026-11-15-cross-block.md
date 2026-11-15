# 2026-11-15 · MONODROMY 13 · No cross-block self-similarity in the Rauzy path: what one block's τ shares with the next it shares with any block

Thread 5 of the thought experiment (#145, 6900062843). The question is whether the induction
itself repeats across Wikidata blocks. If a block's Rauzy path τ coded more cheaply given the
previous block's than given an unrelated one, it would.

**Labels:** **C** counted (exact for its input).

**Inputs.** All 233 blocks of at least 65,536 rows of `corpora/wikidata.bz2` (SHA-256
`82f65ace…c29915`), in file order.
- **τ:** the right-end word of step types, as in `2026-11-15-rauzy-path-code.md`. Every run's
  final word is still checked to be a rotation of the block.
- **The program:** `taux.rs.txt`, here.

## Context statistics (C, 232 blocks)

Adaptive KT code length, order 6. Each model starts with the counts shown and adapts as it
codes.

| warm start | bits a run, min / median / max | ÷ starting fresh, min / median / max |
|---|--:|--:|
| none (fresh) | 0.5473 / 1.2296 / 1.3758 | 1 |
| the previous block's counts | 0.5551 / 1.2109 / 1.3715 | 0.980 / 0.984 / 1.014 |
| a block half the file away | 0.5498 / 1.2127 / 1.3716 | 0.981 / 0.985 / 1.015 |
| every earlier block's counts | 0.5694 / 1.2034 / 1.3926 | 0.975 / 0.978 / 1.040 |

The previous block helps by 1.6%, and a block half the file away by 1.5%. So the gain is a
model's warm-up on stationary statistics, not similarity between neighbours.

## Exact shared segments (C, 232 blocks)

The share of a block's τ covered by 64-symbol windows that occur anywhere in another block's τ:

| | min / median / max |
|---|--:|
| windows from the previous block | 0.288 / 0.337 / 0.536 |
| windows from a block half the file away | 0.281 / 0.336 / 0.509 |

The coverage is the same either way, so what is shared is generic. The shared windows are
segments common to every block's τ, such as long stretches dominated by equal steps, and not
inherited from the block before.

## Verdict

- On this corpus the right-end induction has no cross-block structure beyond stationary
  statistics.
- A cross-block dictionary of induction segments would buy nothing over a warm model, and a
  warm model buys about 2%.
- Thread 5, as framed (repeating loops of the Rauzy path across blocks), is **closed**.

**Not tested:** loops within a block, that is, the same local configuration recurring
somewhere in the same block. The order-6 context statistics already capture what short-range
recurrence there is.

## Reproduce

```
cp docs/journal/monodromy/2026-11-15-cross-block/taux.rs.txt examples/taux.rs
cargo run --release --example taux -- corpora/wikidata.bz2 > taux.jsonl 2> cover.jsonl
```

The outputs are `taux_wikidata.jsonl` (bits: [fresh, previous, half-way, every earlier]) and
`taux_cover_wikidata.jsonl`.
