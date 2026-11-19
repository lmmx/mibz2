# Independent audit of synchronized root descent

Audited the single-byte delimiter construction and inverse merge at `0bebdcfece931da18ddcd431f47e236883a1fa0a`. The tests here use dense row spelling, projection, and binary searches of cumulative selector ends. They share no Rust routines with the candidate.

## Proof and charge

A live node has one ordered selector tape with positive integer widths. Its distinct choices are the node's own terminal and its distinct child subtrees. Adjacent equal choices are coalesced. A child tape has exactly the total width of its parent's selections of that child. Terminal labels are unique to their nodes. Let the expansion of a node replace each child selection of width q with the next q labels of that child's expansion.

At every node, the consumed prefix equals the projected consumed root prefix. This starts at zero. Following the current choice down to a terminal reveals a path on which the next b rows all have the same label, where b is the minimum remaining choice width on that path. Consuming b at every visited node preserves the prefix invariant. The root consumes a positive prefix each time and ends with all child tapes consumed.

For a nonfinal emission, take the highest node u attaining the minimum. All its strict ancestors still select their same child. Node u must have a next selector. Otherwise the parent would have further occurrences of a child whose full tape was already consumed, contradicting width conservation. If u is root its next selector exists by nonfinality. This new selector has a different choice. Its terminal support is disjoint from the preceding choice, irrespective of other simultaneous cuts. Thus the very next label differs. One descent emits exactly one *maximal* root run.

The boundary charges to the internal selector transition at u. These charges are distinct by monotonicity. For S selector pieces, V nonempty nodes and T root runs, **T ≤ S−V+1**. The bound is attained for a root alternating its own terminal with its single terminal child's label. Without coalescing or disjoint terminal supports, this maximality implication fails. `exhaust.json` contains two minimal examples, including the case of distinct children bearing a shared external label.

## Exact costs and obstruction

Let h(j) be the number of physical nodes from the root through the terminal of root run j. Let R(u) count runs in the full expansion of u. Node visits are **H = Σ h(j)**. Restriction to a subtree preserves order. Distinct visits to u may split one local run around intervening visits to other subtrees, giving **H ≥ Σ R(u)**. The original bottom-up merge writes Σ R(u) pieces, and every source emission is one local run by the same distinct-choice argument. The root visitor stores T pieces and a temporary path of maximum h(j). In the audited Rust, each visit reads a selector while extending the path and uses its width again when advancing. The implementation additionally initializes V cursors, updates path storage, writes T pairs and retains the original selector and node arrays. These are source-level operations. Machine instructions, transferred cache lines and RSS require measurement.

In the single-byte delimiter trie, each contracted edge carries at least one source byte, including the root edge carrying the delimiter. Therefore h(j) ≤ the length of its token word. Summing over runs counts each word at most once per occurrence. The section-return words partition the cycles that meet the delimiter, giving **H ≤ covered bytes ≤ n** even if other cycles never meet it. Inputs without the delimiter take another path in the decoder.

This byte bound is sharp. Give the ordered token alphabet j=0,…,D the prefix-free code 1^j 0. Choose token last column (D,0)^M followed by 1,…,D−1, with D≥2 and M≥1. Expand each stable token cycle into its codewords, marking sections at their final zero. Sorting byte suffixes at these zeros orders the subsequent token sequences exactly as the original token suffixes, because codewords are ordered and prefix-free. The preceding word is the prescribed token last-column symbol. Periodic equal rotations have equal predecessors. The delimiter tree has one terminal at each depth 0,…,D and an edge to the next terminal. The induced column has a distinct label on every adjacent row. Direct counting gives

```text
T = 2M + D − 1
S = 2M + 3D − 2
Σ R(u) = T + D(D+1)/2
H = n = M(D+2) + (D−1)(D+2)/2.
```

At M=D², H grows cubically while S+T and Σ R(u) grow quadratically. Neither expression bounds the work to a constant factor for this visitor. `lift.py` constructs the byte columns by sorting cyclic suffixes, independently recomputes stable byte LF, and re-reads every token before checking the formulas. At D=48, M=2,344 the byte input has a single cycle, 118,375 visits, 4,830 selector pieces, 4,735 root runs and 5,911 total local runs. This obstruction concerns deep repeated reopening; it does not lower-bound other inverse merges.

## Evidence and reproduction

Project each root terminal sequence onto its ancestral paths to generate selector tapes. Every valid tree of positive widths with ordered child projections and coalesced choices can be recovered this way from its expansion. `exhaust.py` tests all parent-before-child shapes with one through five nodes using root sequences up to lengths 7, 7, 7, 6 and 5 respectively, omitting unused nodes. It compares literal bottom-up expansion with cumulative-end descent and checks the charge at every boundary. All 55,694 tree/tape pairs passed, including 35,860 nonterminal bottlenecks and 148,047 simultaneous cuts. The tests cover 252,638 root runs and 196,944 nonfinal boundaries.

`real.py` reconstructs all quote-section words by dense byte LF, builds and contracts a literal reverse trie of the resulting dictionary, projects terminal labels to selector tapes, and checks both expansions. All ten archived Wikidata inputs passed. On all nine admitted inputs the independently computed dictionary size, section size, tree nodes, selector pieces, root runs and node visits match the native geometry. For frozen block zero, 135,716 token rows have 6,133 root runs, 18,994 total local runs and 27,156 visits. `real.csv` records every input hash and includes the tenth block under forced structural analysis.

From this directory in the research worktree, using the archived Wikidata inputs prepared by the campaign:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 exhaust.py | diff - exhaust.json
PYTHONDONTWRITEBYTECODE=1 python3 lift.py | diff - family.csv
PYTHONDONTWRITEBYTECODE=1 python3 real.py | diff - real.csv
```
