# Returning interval packets on the modular exchange

In a residue-conjugated exchange, launch the retained prefix once in physical order. Split a packet where its current image crosses a source-strip boundary. Translate the child, concatenate its carried word and stop it if its image is retained. Push the right remainder before following the left child to obtain final bands in source order. An array of m′ ordinal slots at the retained destination bases supplies their bottom order; both initialization and scanning are payable. The only predecessor index here is a page directory and bounded binary search on the source strips. [Implementation](packets.hpp)

Every emitted packet fragment represents at least one distinct pair of starting selected rank and macro step. Before the next return these pairs map injectively to the whole current population: any covered row has a unique preceding retained rank, including itself at step zero. Thus the sum of the traversed widths is at most m, and the fragment count at most m on a stage with m input rows. This proof includes cycles disjoint from the retained set. The same partition of original atom itineraries yields the final affine bands as tail induction, though the binary word parse can differ. The complete marked spelling was checked independently against stable unit LF at every original position.

| q | Packet fragments | Joins | Outputs across stages | Binary search probes | Longest real macro return |
|---:|---:|---:|---:|---:|---:|
| 2 | 3,586,917 | 2,074,533 | 1,794,323 | 17,188,208 | 25 |
| 4 | 2,994,221 | 2,458,210 | 749,675 | 13,260,019 | 54 |
| 8 | 3,302,219 | 3,051,156 | 413,823 | 13,511,612 | 107 |
| 16 | 4,107,171 | 3,973,384 | 257,872 | 15,554,380 | 203 |

At q=2 the prior endpoint tower had 2,001,159 joins. Direct tracing generated 2,074,533, despite a maximum of six pending right fragments. Even at the cheapest packet radius by fragment count the directory paid 13.26 million binary-search probes. The exact construction succeeds without endpoint surgery. This particular schedule produced more joins at every radix on every aggregate and has no independently measured time advantage. All 85,254 tiny marked/radix inversions and all forty original real inversions passed. The remaining geometric question is the number and discoverability of *semantic* return words and canonical strips after discounting redundant discovery from noncanonical parents. [All trajectories](levels.csv) · [Finite checks](small.txt) · [Paid endpoint control](../native/)

```sh
d=$(dirname "$(rg --files | rg '/modular/packets/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/check.cpp" -o /tmp/astra-modular-packets
/tmp/astra-modular-packets | diff - "$d/small.txt"
python3 -B "$d/panel.py" /tmp/astra-modular-packets /tmp/astra-clock-wires | diff - "$d/levels.csv"
```
