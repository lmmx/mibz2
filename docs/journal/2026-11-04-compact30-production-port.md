# compact30 in production: the port, and CI's verdict

**Result: negative. PR #155 was closed unmerged.** Assembly 1's constructor saving carries into
production. The record walk costs more than that saving, and on CI's hardware time follows
instructions.

## What was tried

The research kernel (the Ctor2c constructor and the record room walk) became the default inverse
BWT, with no feature flag and master's `Segmented` as the fallback. Branch
`perf/compact30-default` @ b4786f66, PR #155 against master c4b69de3. The tests, Miri on the
`c30` cases, clippy and the mutation checks all passed. Outputs were identical in every bench
arm.

## Instructions (Callgrind, one Wikidata block, 899,971 rows)

| scope | master | port | Δ |
|---|---:|---:|---:|
| whole `decode_block` | 25,751,794 | 29,530,459 | +3,778,665 (+14.7%) |
| build | 5,429,062 | 4,369,204 | −1,059,858 (−19.5%) |
| walk | 15,426,892 | 20,207,700 | +4,780,808 (+31.0%) |
| rest (MTF, CRC, assembly) | 4,895,840 | 4,953,555 | +57,715 |

The walk goes from 17.14 to 22.45 instructions per row. The raw profiles, the driver, the
command and the input hash are in `compact30-channel/evidence/port-b4786f66/` (c493a45d).

## Time (CI paired bench: EPYC 7763, 2 physical cores, 10 rounds, bench 3.2.0)

| `wikidata` | wall | CPU | peak RSS |
|---|--:|--:|--:|
| 1 thread | +18.7% (+17.6, +19.7) | +18.6% | −5.7% |
| 2 threads | +11.0% (+8.6, +13.5) | +15.1% | −10.6% |
| 4 threads | +7.3% (+6.2, +8.5) | +8.2% | −10.3% |

Every corpus is slower at 1 and 2 threads except `onebyte`. Amplification at 4 threads
improves by 8.8%.

## What was learned

- **The shared VM misled.** It showed −3 to −4% wall (intervals excluding 1), and the
  cache-residency hypothesis (2.1 MB of records against a 3.6 MB T-vector) explained it. On
  CI's hardware that hypothesis is falsified: CPU per byte rose about 18% for +14.7%
  instructions. Local timing stays indicative only, as CLAUDE.md says.
- **The research ledger measured construction only** (gate 3, D015). In production the walk
  outweighs the build about 3:1, and that term was never priced.
- **The revival condition** is that whole `decode_block` instructions be no more than master's. On
  this block that means a cut of at least 3.78 M. With the build held, the record walk must
  reach at most about 18.26 instructions per row (from 22.45), a cut of at least 18.7% of
  `C30::room`.
- **This is a failed implementation, not a failed possibility** (charter §8.4). The decision
  now rests on the Optimiser's per-component pricing of the walk, and on the Researcher's
  lower bound for the record walk per row (CHECKPOINTs 6641512290 and 6641672441).
