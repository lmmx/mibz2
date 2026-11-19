# Selective materialization inside the delimiter trie

Choose an antichain of cut nodes. Below each cut, construct the terminal streams bottom-up using the original stable merge. Above it, descend the selector tapes and treat a cut node's current precomputed run as a terminal choice. The projection invariant in the [audit](../audit/) applies to this substituted tree. Child consumption preserves rank order, and the minimum remaining width still yields exactly one maximal root run.

Write G(v) for the number of root runs whose terminal lies below or at v, and R(v) for the number of runs after projecting the root labels to that subtree. G(v) can exceed R(v) when outside labels interrupt a local run. Active internal nodes incur G(v) routed visits each. A cut node incurs G(v) precomputed-run lookups. Materializing a subtree at v costs A(v) = Σ R(u) over its full subtree in local emissions. Every bottom-up emission produces one local run because neighboring selector choices have disjoint terminal supports. With a single append-only arena a nonroot cut requires A(v) additional pairs. For a root cut the root result is already available, so no extra cut lookup is required and its extra pieces are A(0)−R(0). The candidate's `Piece` has two u32 fields.

Use one unit per routed visit, cut lookup or bottom-up emission. Choosing a proper subtree v as a cut in a pure cursor tree saves exactly Σ G(u) over its **strict** descendants minus A(v), and spends A(v) pairs. These units have different memory-access and instruction costs. Root writes, the selector and node arrays, mark transfer, construction, dictionary, residual inversion and final byte output still apply. The numbers below measure the merge geometry; any production claim requires timed decoding and peak RSS.

For memory price λ, the optimal penalized subtree recursion takes the smaller of the cut's work plus λ times its storage, and G(v) plus the sum of independently optimized children. Root gets the special cut price above. Slope queries find all vertices of the supported lower convex hull. Integer memory budgets can also have unsupported optima. The oracle requires the root runs before choosing the cut and serves as a construction limit in this model.

## Ten input blocks

All statistics were computed from independent dense LF spelling and reverse-trie projection. The frozen block's cursor tree takes 27,156 routed visits. Materializing all nodes takes 18,994 local emissions and 12,861 extra pairs. An intermediate supported point takes 24,309 normalized actions and 1,188 extra pairs (9,504 logical bytes). Across ten blocks, spending at most a tenth of the full extra-pair budget achieves an 8.46–10.81% normalized reduction, capturing 28.1–35.0% of the endpoint gap. On frozen input the first three extra pairs save 133 normalized actions. A uniform depth-two cut spends 8,749 extra pairs to save 436. Every deeper uniform cut increases the count on that block. [Oracle frontier](frontier.csv) · [Fixed-depth costs](depth.csv)

An input-only policy can guarantee a positive count reduction. Let s(v) count local selectors, i(v) the incoming distinct selector pieces in its parent and t(v) its own terminal occurrence width. A subtree has R(v) ≤ min(subtree occurrence width, 1 + Σ(s(u)−1)). Lower-bound G(v) by max(s(v),i(v)) and propagate parent lower bounds after subtracting the mass of all other choices. Sum child lower bounds to bound the cost of strict descendant visits. Each of the guaranteed runs beyond t(v) must also visit at least the shortest possible path through a child to a terminal. Use the larger lower bound. Subtract the sum of the upper bounds on R to obtain a certified saving for a cut. A bottom-up antichain DP maximizes certified penalized savings. The inputs to this decision are existing tree geometry and local selector counts. The code consults measured G only in assertions. [Decision rule](policy.py) · [Measured outcomes](policy.csv)

With zero memory price the rule selects 24 frozen subtrees, costing 401 extra pairs and saving 1,699 actions (6.26%, with 672 certified from local data). Across all ten blocks it selects 15–48 cuts per block, saves 4.51–6.82% of normalized work and builds 210–1,328 extra pairs. Recognition needs incoming-piece and terminal-width accounting, whole-tree forward and reverse passes, DP bookkeeping and selection. Frozen has 3,689 nodes and 14,992 selectors; recognition costs can exceed the saved actions. A constant-radius certificate selects only parents whose children are terminal leaves. Here A(v)=s(v)+number of children and strict descendant visits are at least i(v)−t(v). On ten blocks this much simpler rule saves 0.40–1.46%; 148 actions on frozen. [Fringe-only results](fringe.csv)

The next independent simplification absorbs each terminal-only child directly into its parent selector. Width conservation places that leaf's remaining mass above or equal to the remaining parent selection, so no leaf cursor can constrain the jump. This removes a dependency even when none of the mixed-cut certificates fire.

## Reproduction

From this directory, with the campaign's archived Wikidata input records available, the following commands reproduce the structural evidence. `nodes.csv.xz` includes per-node true and pre-output statistics for all ten blocks, also covering the density-refused control.

```sh
export PYTHONDONTWRITEBYTECODE=1
python3 data.py /tmp/f2-cut-nodes.csv.xz
cmp /tmp/f2-cut-nodes.csv.xz nodes.csv.xz
mkdir -p /tmp/f2-cut-frontier
python3 frontier.py nodes.csv.xz /tmp/f2-cut-frontier
diff frontier.csv /tmp/f2-cut-frontier/frontier.csv
diff depth.csv /tmp/f2-cut-frontier/depth.csv
python3 policy.py nodes.csv.xz | diff - policy.csv
python3 fringe.py nodes.csv.xz | diff - fringe.csv
python3 check.py | diff - check.json
```

The small-tree check exhausts 990 tree/tape pairs and their 5,970 antichain decodes. It independently counts each hybrid traversal, checks exact root output and local writes, brute-forces the scalarized optimum and checks both selector-only certificates at five memory prices.
