# Allocating the short-safe prefix budget

On Wikidata ordinal 0, varying the depth by literal prefix reduces 78,084 distinct core lookups to 67,396 at exactly the same 7,583 forward intersections and 1,715 terminal bands. The integer solver's lower bound is 67,396 for the registered search family. On nine held-out blocks the frozen policy saves 0.78% of pooled core lookups, with 43.83% fewer forward intersections and 23.53% fewer bands. Five of nine blocks improve all three quantities. The transfer threshold was 5% fewer lookups and seven individual wins. This ends native evaluation of this literal policy under the registered rule. [Registration](REGISTER.md) · [Fit](fit.json) · [Held-out receipts](heldout.csv)

## Two additive forward budgets and one closure objective

For each original word W, cut after t successful forward steps, with t at most its length minus one. An enabled literal prefix authorizes the corresponding forward lookahead. If it reaches U, the packet stops at its current physical row. Every descendant reached in the full depth-16 construction has exactly the same source-owned affine packets whenever all its ancestors are enabled. F therefore sums the intercepted original Ψ runs at the activated nodes. B starts at one and adds the count of outgoing packets and short stops minus incoming packets at each activated node. The accounting uses the uncoalesced atlas from 11cc7ee. A compilation can merge adjacent equivalent affine strips, which defines a different budget. [Frozen per-prefix charges](frozen-costs.csv.xz)

A nonempty suffix u contributes to backward discovery if some W has u as a *proper* suffix and t(W) is strictly less than d = |W|−|u|. A word equal to u contributes no such lookup. When any contributing d exceeds 16, u is mandatory. Otherwise, removing u requires every prefix W[:d−1] for its contributing words. Prefix closure supplies the ancestors, so comparable requirements reduce to their maximal elements. Equal requirement sets share a weighted reward. The empty root always remains. The solver imposes the two budgets and child-implies-parent. A shared reward is bounded by each required action. Its positive objective coefficient makes its optimum their conjunction. Single-requirement rewards enter the action objective directly. This model can express complementary actions whose separate marginal rewards are zero. [Grouped obligations](frozen-goals.jsonl.xz) · [Independent tiny proof and exhaustive checks](audit/README.md)

The optimizer sees the true original return vocabulary on the single training column. Stable dense Ψ and F bytes build that oracle from the original runs. No plaintext file is supplied. Applying the frozen trie uses only the current state and the byte on the intercepted Ψ strip. A state tests its sorted outgoing byte list. In breadth-first edge order, child edge i goes to state i+1. Its 2,393 states use 2,392 edge bytes and 2,394 four-byte offsets, totaling 11,968 bytes before container metadata. Direct application checks every source, original word, destination and terminal owner against the dense oracle. For the constant cap its physical leaves also equal the original atlas. These structural checks cover 24,165 training arcs and 221,280 held-out arcs.

| Nine-block sum | Cap 8 | Frozen literal policy |
|---|---:|---:|
| Distinct core lookups, Q | 927,906 | 920,648 |
| Forward affine intersections, F | 104,232 | 58,542 |
| Terminal bands, B | 23,788 | 18,190 |
| Distinct core spelling bytes | 1,420,132 | 1,483,666 |
| Core types | 31,286 | 34,424 |

Every held-out block stayed within both geometry budgets. Each visited just 496 to 860 of the 2,393 trained states. The byte trie performed 56,982 nonempty-child searches and 204,308 single-byte comparisons across nine blocks. The charged lookahead produced 58,542 successful child segments. A depth cap checks the current depth alone. These counts leave setup, the backward index, actual hardware costs, and the writer unmeasured.

The literal policy gains 55,538 suffix eliminations that cap eight misses across the held-out blocks. It loses 48,280 of cap eight's eliminations. For 26,559 of those lost suffixes every failed obligation first stopped at a prefix the optimizer deliberately disabled on the training block. For 21,611 each first failure encountered a previously unseen prefix. The remaining 110 have both causes across different word requirements. This accounts for the small net transfer despite a sizeable training gain and substantial forward-work savings. [Per-block decomposition](diagnosis.csv)

## Reproduction

Run in the research worktree, with the archived Wikidata corpus present. The rule was registered at d8faf43. The literal policy, model, solver receipt, and transfer evaluator were frozen at 203c9f3 before evaluating the nine tapes. Regenerate outside the research record.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/short-core/policy
t=research/astra-e/dense/target/wikidata
out=$(mktemp -d)
python "$d/model.py" "$t/0000.irec.xz" "$out"
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 python "$d/fit.py" "$out" "$t/0000.irec.xz" --seconds 90
python "$d/transfer.py" "$out" "$t" "$out/heldout.csv"
python "$d/diagnose.py" "$out" "$t" "$out/diagnosis.csv"
```

The numerical optimum bound is from SciPy 1.17.0's HiGHS MILP solve with the recorded tolerances. To evaluate the exact preserved policy independently of solver tie choices, use the committed directory as the policy argument to transfer.py. Local timing, worker throughput, peak decoder memory, CI, and i9 performance were outside this structural experiment.
