# Price an optimal physical-rank partition with arbitrary modular slopes

Let P(x) be the byte coordinate of original source rank x. On a physical interval the formula P(x)=a·x+b modulo n is exact precisely when all adjacent modular differences on that interval agree. Singletons and pairs always qualify. Taking the longest feasible prefix minimizes the number of disjoint intervals. Indeed the greedy prefix ends at or after the first interval of any optimal partition. Removing that prefix leaves a suffix of the remaining intervals, still feasible under restriction. Induction finishes the proof. A dynamic program gave the same minimum on all 5,913 permutations through length seven. [Checker](partition.py) · [Exhaustive receipt](small.json)

The oracle supplied P by stable sorting each original last column and following its marked cycle. The optimal disjoint partition still needs **4,472,732** intervals on 8,999,915 positions. Only **54,456 positions (0.605%)** follow from an interval's first two coordinates, below the preregistered 50% threshold on every block. Requiring source intervals to stay inside a single F-byte bucket gives 4,472,941 intervals and 54,446 implied positions. Even the union of *all* exact monochromatic intervals of length at least four covers only 62,770 positions. Length eight covers 8,345 and length sixteen covers none. These are optimistic construction-free counts and the normalized primary is immaterial to differences. Contiguous rank intervals with arbitrary slopes give little compression of the composed position permutation on these inputs. The per-round sparse transport maps remain individually valid and exact. [Preregistration](PLAN.md) · [Ten complete histograms and counts](blocks.jsonl)

For reproduction run the following from the repository root.

```sh
d=$(dirname "$(rg --files | rg '/backwards/slopes/PLAN.md$')")
python3 -B "$d/partition.py" | diff - "$d/small.json"
python3 -B "$d/partition.py" research/astra-e/dense/target/wikidata | diff - "$d/blocks.jsonl"
```
