# Deterministic residue orbits on exact returned rows

For each current permutation, construct a directed graph on occupied residues modulo q. An edge means at least one physical source row has this destination residue. Following singleton-outdegree residues from the marked residue until it returns proves that every source in this class first returns in exactly the same number of steps. On this orbit the group maps are bijections. Injectivity makes successive class sizes nondecreasing, and going around the cycle forces equality. Its union is invariant. For a one-cycle current permutation, this union contains every active class. When n≥q this forces q to divide n. Determinism can also fail when divisibility holds. On a nonprimitive input the same reasoning applies per closed union and other marked components must be treated separately.

This check uses dense stable sorting of the original physical last column followed by exact dense first-return permutations. It checks every image, row traversal count, selected size and full marked lap at every level, then joins the results to the original endpoint operations in the parent's levels.csv. All ten real original permutations and all derived sections were one-cycle.

|q|examined layers with n≥2q|closed at marked residue|qualifying joins / all joins on these layers|
|---:|---:|---:|---:|
|2|186|3|7 / 2,001,145|
|4|90|0|0 / 2,233,780|
|8|60|0|0 / 2,524,573|
|16|40|0|0 / 2,931,145|

The only nondegenerate matches had sizes 4, 6, and 4 on blocks 85, 134, and 397. Of all 418 layers including the final singleton-row classes, no closed residue cycle excluded the marked residue. levels.csv records even the partial deterministic prefixes and rows belonging to singleton-outdegree classes. A complete marked orbit at a final singleton stage can only save that tiny stage. The measured construction remains dominated by layers without this property. Replacing their endpoint work would require a weaker invariant or a different partition. No wall time was measured.

Reproduce from the checkout root with the ten capped originals listed in powers/xor/capped/receipt.json. The output directory must be fresh.

```sh
here=$(dirname "$(rg --files | rg '/modular/theory/residues/panel.py$')")
python3 "$here/panel.py" /workspace/work/astra-powers-original/capped /tmp/residue-orbits-replay
diff "$here/levels.csv" /tmp/residue-orbits-replay/levels.csv
diff "$here/summary.csv" /tmp/residue-orbits-replay/summary.csv
```
