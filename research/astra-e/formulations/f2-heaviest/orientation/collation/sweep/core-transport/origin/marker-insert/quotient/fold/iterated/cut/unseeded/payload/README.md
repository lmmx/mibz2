# Grammar sharing with the same source ranks

The cut needs separate ordered types to preserve stable partners. A spelling node can be shared across unrelated types because rank computations, class colors and mark offsets depend on the ordered types and word lengths. Replacing a node by any node with exactly the same byte string preserves every reconstructed byte, including a clipped start or end. Existing word-color pair compression in Astra investigation 3 already shared rules by child identities. This experiment freezes the full unseeded ordinary-column schedule and isolates reuse within that schedule. [Earlier explicit-band screen at its source revision](https://github.com/lmmx/mibz2/blob/9cd25610131b66924eaa75813d0f649db79f354f/docs/journal/2026-11-18-switch-elimination/pair_screen.py)

| Sharing key | Internal nodes | Emission node visits | Materialized bytes for exact text comparison |
|---|---:|---:|---:|
| Fresh node for every selected type | 566,895 | 1,134,006 | 0 |
| Exact ordered child identities | 523,724 | 1,047,664 | 0 |
| Child identity and equal texts up to length 16 | 523,417 | 1,047,010 | 2,122,086 |
| Child identity and equal texts of every length | 523,383 | 1,046,936 | 126,415,653 |

The counts sum ten complete original byte blocks. All thirty reconstructions reproduced the original pre-RLE1 hashes, the exact marked offsets and all cut geometry. Structural reuse removed 7.62% of original internal allocations. Fully canonical byte equality eliminated only 341 additional nodes. Within this fixed parse, exhaustive equal-text sharing can reduce node requests by only 728 more than structural sharing. Acquiring full exact spellings consumed 126.4 MB over the ten inputs. A native child-identity cache still has to repay its own lookup and storage cost on each candidate join. These figures do not price that cache. [All output hashes, potentials and counts](real.jsonl)

The potential materialization cost is bounded. At a layer, sum each new positive word length weighted by its frequency. Contraction partitions old cycle edges into disjoint positive groups. The total weighted byte mass cannot increase; removing a fixed foreign component decreases it. Charging one representative spelling per positive new type is therefore at most the original byte mass B per cut, and at most another B for a preceding power phase. Across logarithmically many rounds an exhaustive spelling oracle needs O(B log m) cumulative byte materialization, separately from cost of hashing or exact-equality checks.

```bash
d=$(dirname "$(rg --files | rg '/unseeded/payload/PLAN.md$')")
python3 -B "$d/census.py" research/astra-e/dense/target/wikidata | diff - "$d/real.jsonl"
```
