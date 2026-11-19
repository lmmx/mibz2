# Forced growth of the ordinary incoming run column

**Even on a primitive, zero-self BWT, every cut meeting the quarter rule can increase the number of incoming runs.** One 32-row marked circular word is 02524525232310310303103103031031. Its last column is 11111133333333355500000000222224. Each of its six types has one incoming run. There are no self transitions. Of all 64 orientations, 44 remove at least 8 rows. Every one leaves at least seven runs. This is the full ordered source-by-destination refinement described in the cut proof. Coarser source alphabets and different parses may behave differently.

For each orientation let q be its ingress and r′ its resulting number of runs. The 64 exact outcomes all satisfy **5r′ ≥ 23+q**. The two masks 45 and 5 attain equality at (q,r′)=(7,6) and (12,7). Take h disjoint copies using ordered, disjoint type alphabets and concatenate their incoming columns in the same order. Their stable permutations and RLEs remain disjoint; their run counts add. A globally productive cut has total q≥8h, hence at least 31h/5 output runs from 6h initial runs. For h a multiple of five, using four copies of mask 45 per one copy of mask 5 attains the bound. This proves a sharp factor **31/30** for this family. The many-component family is a valid multiple-cycle stable column. The single primitive witness above already forces strict growth on its own. Replacing each row of that primitive circular BWT by t identical consecutive copies also gives a legitimate periodic word BWT of unbounded length with the same six-run obstruction.

A small exhaustive search initially gave no forced-growth zero-self witness. Across 11,106 productive zero-self columns through nine rows and four types there were 118 individual cuts where both orientations grew, yet another productive cut always preserved or reduced runs. Drawing 16,000 columns from clustered balanced cycle graphs also found no forced-growth primitive example. The subsequent one-run-per-type interval exchanges did: 7 among 3,196 primitive zero-self draws. The independent Python verifier reconstructs the actual marked word by stable LF, rebuilds its sorted circular BWT, prices every possible coloring, and checks the supporting line and its equality cases using exact rational arithmetic. [Exhaustive counts](observations.csv) · [Original clustered draws](sparse.txt) · [One-run-per-type draws](exchange.txt) · [Witness and all exact costs](witness.csv)

## A general run-sensitive grammar bound

Let m and r be the initial number of positive-weight rows and incoming runs. Write Rᵢ for the incoming runs before each geometric contraction. Rᵢ≤mᵢ≤m(3/4)ⁱ, independently of any alphabet-order decision. A power publication has at most 2Rᵢ+Kᵢ−2≤3Rᵢ marked interval leaves, and both its distinct types and emitted runs fit this bound. Ordinary source refinement of a column with R runs and K types gives at most R+K−1≤2R incoming records. Folded output has at most the number of refined runs plus the number of selected incoming records, hence at most twice the refined runs. Together these give Rᵢ₊₁≤12Rᵢ, including steps where normalization is skipped.

Thus Rᵢ≤min(r·12ⁱ,m·(3/4)ⁱ). Split the sum at the crossover log₁₆(m/r). Each side is a geometric series. With α=log(12)/log(16)≈0.89624 we obtain **Σ Rᵢ = O(mᵅ r¹⁻ᵅ)**. New distinct ordered products, new grammar nodes, published runs and per-pass alphabet sizes are all O(Rᵢ) by the same bounds. Consequently the terminal grammar obeys this sublinear-in-m bound for fixed r, along with its independent ≤m−1 internal-node bound. When no power pass is used the run growth multiplier can be replaced by 4, giving α=log(4)/log(16/3). Neither exponent is asserted optimal. One-round forced growth rules out a monotone-run proof of O(r log m); it leaves a sharper amortized bound open. These construction counts exclude reading and writing the requested output.

```bash
d=$(dirname "$(rg --files | rg '/work-bound/run-growth/PLAN.md$')")
for t in scan sparse exchange; do
    g++ -std=c++20 -O2 -Wall -Wextra -Werror "$d/$t.cpp" -o "/tmp/astra-cut-$t"
    "/tmp/astra-cut-$t"
done
python3 -B "$d/witness.py" | diff - "$d/witness.csv"
```
