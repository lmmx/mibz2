# Optimal sharing of default spellings on deterministic physical intervals

A smooth state has only one incoming byte and hence one deterministic predecessor state. Its entire physical source interval maps by translation into that destination component. If a directed cycle consists entirely of smooth states, their populations are nondecreasing around the cycle, hence equal. Every edge therefore covers its entire successor, excluding other incoming states. The translation around that cycle returns each physical row to itself. A mark in such a component can spell its lap straight from the state cycle. The remaining smooth states form forests directed toward branch states.

For each smooth destination v choose at most one smooth predecessor u to be stored consecutively with it. Write one byte per smooth state, arranging each resulting chain from the end nearest a branch toward its leaves. A backwards walk traversing one chain writes its byte slice with one normal forward memcpy into descending output space. Every transition u→v left out of the layout forces one extra segment when a smooth stretch crosses that edge. Conversely there is no other internal split. On an entire primitive lap its weight is exactly the number of physical rows in u. Maximizing saved copies can therefore be solved independently at each v by continuing its greatest-mass smooth child. On a finite cut of the lap subtract the possible one light-edge crossing cut by the terminal seam. For general multi-cycle marked horizons these physical population weights give the all-row objective. A particular marked orbit need not have the same weights.

Disjoint smooth-child images imply Σ mass(child)≤mass(v). A child not chosen as largest has at most half the parent's mass. A stretch beginning at u therefore crosses at most floor(log₂(n/mass(u))) light edges, regardless of its length. This proves the logarithmic bound while minimizing total copies over static disjoint-chain layouts. The proof allows branches to have other incoming, exceptional ranks.

On ten real k=8 models, 708,399 maximal smooth stretches required 1,021,865 chain segments and at most seven segments on any stretch. Each participating smooth type needs one stored byte (792,348 total). Storing every smooth state's complete path independently would use 16,246,823 bytes. At k=16, the corresponding counts are 376,879 stretches, 575,909 chain segments, 1,151,722 shared spelling bytes and 36,229,715 naive bytes. These figures exclude transitions, coordinates, starts and the branch representation. Every counted extra split matched the independent sum of rejected child weights with the finite seam adjustment. [Exact per-block cost](blocks.csv) · [Computation](chains.cpp)

```sh
d=$(dirname "$(rg --files | rg '/jumps/smooth/chains/PLAN.md$')")
g++ -std=c++17 -O3 -Wall -Wextra -pedantic "$d/chains.cpp" -o /tmp/astra-mass-chains
python3 -B "$d/panel.py" /tmp/astra-mass-chains /tmp/astra-clock-wires | diff - "$d/blocks.csv"
```
