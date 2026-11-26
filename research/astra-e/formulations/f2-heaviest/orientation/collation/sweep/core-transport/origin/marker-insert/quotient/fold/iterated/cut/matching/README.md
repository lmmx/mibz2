# How much can one delimiter cut remove?

Write Eᵢⱼ for the number of source rows of type i whose successor has type j. Its row and column sums agree because the successor is a permutation. A cut's ingress q therefore equals half the number of directed edges crossing the cut. Let A be the total nonself mass. For each unordered pair give edge ij weight Eᵢⱼ+Eⱼᵢ.

## A constructive certificate

Choose any disjoint matching of total weight M. Put the two ends of each pair on opposite sides, then flip each pair and each unmatched vertex independently with probability one half. The matched edges all cross. Every other edge crosses with probability one half. Some assignment therefore admits **4q ≥ A+M**. Choose each pair's orientation to maximize the weight crossing previously assigned pairs. This preserves the conditional expected lower bound until it is deterministic. A global flip puts the protected anchor outside without changing ingress.

The structural prototype sorts the h nonzero unordered edges by decreasing mass. In O(h log h + K) time and O(h+K) storage it finds a matching, fixes its signs and counts the actual ingress. The conditional guarantee holds for a cheaper matching too. If M exceeds or equals the self mass the new certificate alone proves that a power normalization can be skipped. Computing an ordinary cut and checking its actual ingress can already establish the same conclusion.

On the ten original marked Wikidata byte columns the previous chooser removed 3,025,271 of 8,999,915 rows in the first pass. Forced pairs removed 3,170,769, about 1.62% more of the original mass. It lost to the ordinary chooser on two tapes. Matched weight covered the self mass on all ten. The new guaranteed numerator totaled 10,555,919, against 8,843,063 from nonself mass alone. Division by four gives the respective lower bounds. Both realized cuts exceeded one quarter on every tape. Building the weighted graph, sorting records and publishing aliases are real costs. This test measures no time gain. [First-pass counts](first-cuts.csv) · [Exhaustive arbitrary anchored columns](small.txt)

## A realizable upper bound

Use an Euler tour of the loopless complete directed graph on k symbols as a cyclic word. It contains each ordered unequal pair once. The word is primitive because a nontrivial repetition repeats every pair occurrence. Its real circular BWT, reconstructed by stable occurrence rank at its primary, has exactly that adjacency graph. A type bipartition of sizes t and k−t therefore has ingress t(k−t), at most ⌊k²/4⌋. There are k(k−1) rows. For these words the matching certificate attains the optimum exactly, with M=2⌊k/2⌋. Inserting a new unique protected symbol along one existing edge changes the maximum by at most one upwards, since at most one of the two replacement transitions enters a given side. The limiting removable fraction is one quarter. A universal bound strictly above one quarter by a fixed constant cannot apply to single cuts on arbitrary weighted token alphabets. A raw bzip2 byte column has at most 256 symbols. Its finite limits remain explicit in the formula.

Enumeration for k=2 through 12 checks every allowable cut, uses the true sorted cyclic rotations and reconstructs every witness. Equal-adjacent-byte expansion is irrelevant to this family. The inserted singleton also supplies a canonical primary. [Finite cliques and exact optima](cliques.csv) · [Witness generator](certify.py)

## Exact optimal selection is NP-hard for general token alphabets

Given a connected simple undirected graph G, attach a new leaf s to any existing vertex u. Replace every edge by its two directed orientations. The directed graph is Eulerian. Reading symbol IDs in an Euler tour starting at s gives a cyclic word with a unique s, no self transitions and each directed edge used once. Hence the word is primitive and its marked BWT realizes exactly this graph. Each crossing undirected edge contributes one directed ingress. Any cut of G extends to place s opposite u and a global flip can always fix s outside. Conversely removing the leaf edge can decrease any cut by at most one. The optimal admissible ingress is therefore **MAXCUT(G)+1**. This is a polynomial reduction, including construction of the true ordered last column. Connected simple MAX-CUT is NP-hard, giving a complexity obstruction to globally optimizing this one pass. It leaves efficient approximate selection and multi-stage constructions available. The argument concerns token alphabets that can grow with the instance.

The independent exhaustive certificate reconstructs marked BWTs for every labeled connected simple graph of order 2 through 5 and compares both optima. Counts by size are 1, 4, 38 and 728. A unit-weight input and a unique marked symbol suffice. [Reduction and finite check](graph_reduction.py) · [Raw counts](graph-reduction.txt)

Reproduce from the repository root with Python 3.

```bash
d=$(dirname "$(rg --files | rg '/cut/matching/PLAN.md$')")
python3 -B "$d/certify.py" finite | diff - "$d/cliques.csv"
python3 -B "$d/certify.py" small | diff - "$d/small.txt"
python3 -B "$d/certify.py" real research/astra-e/dense/target/wikidata | diff - "$d/first-cuts.csv"
python3 -B "$d/graph_reduction.py" | diff - "$d/graph-reduction.txt"
```
