# Which end of a return word should the constructor discover?

Fix the actual return dictionary at a single marked section. Imagine a constructor that can discover a word either forward from its source or backward from its destination. In one direction, charge once for each distinct proper oriented prefix of the words assigned to that direction, including the empty prefix when used. For backwards discovery these are the ordinary proper suffixes. This prices the spellings under an optimistic representation. Identifying their endpoints, building the index, conveying the primary, ordering the words and expanding output still have to be paid. The cold-word subset below uses one occurrence per dictionary word, which can include a word discovered directly by a wider branching node.

**Exact mixed-direction optimum.** Make one vertex for each word. Its side of a cut chooses forward or backward. Give every prefix-trie node its unit activation price as an arc to the sink. An infinite arc from a word to its forward leaf, and from a prefix node to its parent, forces the entire spelling path onto the forward side when the word is there. Reverse all these constraints for the suffix trie and give its nodes a unit arc from the source. Every orientation has a cut costing exactly its number of distinct active nodes. A finite cut must pay at least that cost for the directions it chooses. Consequently minimum cut is the exact optimum in this model. The implementation contracts every unary edge into its length, accounts for terminal proper-prefix keys that end at internal nodes, and recovers the actual selected words from the residual graph. Each real cut was independently repriced from its assigned strings. [Solver and exhaustive assignment check](cut.py) · [1,470 small optimality checks](small.txt)

**Achievable separation on cyclic BWTs.** Take m distinct fixed-length binary codes on two symbols separate from the delimiter, a and b. Make m words delimiter·aᴸ·codeᵢ and m words delimiter·codeᵢ·bᴸ. Concatenating them on a cycle gives precisely this dictionary. Every all-forward trie pays at least m(L−1) for the disjoint b tails and the all-backward trie pays at least mL for the a tails. Assigning the first group forward and the second backward costs at most 2L + O(m log m). With L large compared with m log m the improvement factor over either fixed direction grows proportionally to m, even on a fixed five-symbol byte alphabet. For m=32 and L=512 the exact optimum is 1,119 queried spellings against 16,958 and 17,021. [Family, cyclic BWT verification and exact cuts](family.py) · [Results](family.txt)

| Section and words over ten systematic blocks | Forward | Backward | Optimal mixture | Words assigned forward / total |
|---|---:|---:|---:|---:|
| Quote, all | 672,559 | 908,546 | 649,685 | 32,849 / 35,117 |
| Quote, occurring once | 561,473 | 751,142 | 548,587 | 25,961 / 27,229 |
| Most frequent quote return, all | 1,168,644 | 1,296,362 | 987,087 | 21,658 / 38,648 |
| Most frequent quote return, occurring once | 976,665 | 1,103,008 | 856,803 | 19,032 / 31,665 |

On frozen, all quote returns cost 70,560 backwards, 49,806 forwards and 48,631 with optimal orientation. Across the ten blocks the forward-only quote estimate already recovers much of the apparent gain. A forward step on an original union of ranks can split its image into many fragments. The cut also assumes that the words and their direction assignments are known. The decisive experiment is to recover the words by original forward Ψ with every split, select, merge and emitted interval charged. A low cut value alone gives no decoder timing. [All 40 exact real dictionaries and cuts](wikidata.csv) · [Coarser width-one cold cut](../cold-links/)

The replay uses SciPy's integer maximum-flow solver, NumPy and the ten [frozen source tapes](../../../dense/README.md).

```sh
d=research/astra-e/formulations/f2-heaviest/orientation
python3 "$d/cut.py"
python3 "$d/family.py"
python3 "$d/cut.py" research/astra-e/dense/target/wikidata > /tmp/astra-orientation.csv
cmp /tmp/astra-orientation.csv "$d/wikidata.csv"
```
