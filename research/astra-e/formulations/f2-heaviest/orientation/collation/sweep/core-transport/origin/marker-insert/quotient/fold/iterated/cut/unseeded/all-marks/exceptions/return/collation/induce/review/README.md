# Independent audit of the conjugated section

Moving an arbitrary retained physical section to a prefix on both axes gives its exact marked first-return permutation and backward itineraries. A unique ordinary ordinal per returned strip must follow **LF-target order**. If the starting physical atom cuts are fixed, complement order can change the number of endpoint steps, while the final partition into full initial-atom itineraries is invariant after transporting the kept row coordinates back. An explicit retained boundary prevents a clipped final subtraction. [Proofs, orientation witness and worst-case bounds](THEOREMS.md)

One complementary ordering reduced measured structural work on all ten run-head sections. Sorting the contiguous removed components by ascending physical width, stably within a width, reduced total endpoint steps **13.29%**, from 843,191 to 731,142. Stable physical order was best on every one of the ten shifted K8 exception sections. The same ascending-width rule increased work there by 10.59%. These comparisons count induction steps on the supplied geometry, without native elapsed time or an assumption that the cost of each step is equal.

| Section across the ten blocks | Kept rows | Original stable | Reverse order of complement chunks | Long chunks first | Short chunks first | Both axes reflected | Final bands in every unmerged ordering |
|---|---:|---:|---:|---:|---:|---:|---:|
| Shifted K8 exceptions plus primary | 409,441 | 922,499 | 1,075,790 | 1,028,691 | 1,020,157 | 1,071,292 | 145,707 |
| Original run heads plus primary | 690,484 | 843,191 | 946,498 | 985,974 | 731,142 | 946,380 | 690,484 |
| Prefix of K8 exception mass plus primary | 409,441 | 1,023,612 | 1,070,598 | 1,060,964 | 1,033,246 | 1,034,122 | 82,212 |

Natural source-atom coalescing has a sharply limited effect on these stable cases. For K8 it reduced starting atoms from 703,241 to 699,388, steps from 922,499 to 920,611, and final bands from 145,707 to 145,489. For the run-head section it removed 3,127 initial atoms but only **ten** steps (843,191 to 843,181). It can matter greatly on other inputs: an explicitly constructed 512-row rotation dropped from 511 to 52 steps after a costly orbit-based reordering and affine coalescing. [All 300 model rows](orders.csv) · [Totals](totals.csv) · [Rotation family](long-laps.csv)

For the run-head policy, sorting operates on 194,023 complementary chunks in total. The maximum across blocks was 419 distinct widths and a longest chunk of 27,068 rows. A dense width-counting directory can charge O(chunks+maximum width) per block, including its initialization and output grouping. A sparse sort instead pays to find and order the distinct widths and append actual component IDs. Publishing the final run-head inverse still involves 690,484 singleton-width bands. The reduction in tail steps therefore establishes a structural opportunity with substantial acquisition and publication costs left to price. [Per-block ordering inputs](sort-cost.csv)

The direct-edge interval hitting set gives a tail-step lower bound of 1,308 across the ten stable K8 models. Their actual total is 922,499. The proof supplies an O(A log A+H) calculation on affine atoms for H required cuts, checked against the independent two-sided dense-edge scan in all 300 models. The stronger observed atom-conservation lower bound is 703,241−145,707 = **557,534 steps** for this fixed physical refinement. That term is unchanged by permuting its complement chunks. On the two-run singleton-section family, the edge lower bound attains n−1 in stable order and still forces at least n−k after rearranging k intact monotone chunks. The family demonstrates a linear obstruction even when there is only one published return. [Lower bound derivation](THEOREMS.md) · [Atom and dense implementations](orders.cpp)

The exact ordering problem admits another reduction. At each outer step the cost of removing one whole complementary component depends only on that component and the subset of components still live. The rest of their order and the history of components already removed can be discarded. A subset DP gives an exact small oracle, checked against 49,220 complete permutations. It is exponential in the number of components. [Proof and exact verification](ORDER.md)

## Verification and reproduction

The independent dense model forms each smaller permutation by deleting simultaneous suffix rows and explicitly composing all edges that entered them. On 33,840 small layouts it checked first returns, byte words, ordinary reduced LF and 92,145 complete marked outputs, including nondividing marked periods. Another 63,840 layouts checked all 214,875 kept adjacencies against independently collected original atom itineraries. At fixed membership cuts these included 30,000 randomized larger layouts with a fixed seed. Every real stable instance in the three sections also matched the independent linked-list implementation at cd35d80851be5c6127345aa8f2d25556bfee8ecf, including initial and final atom counts and all induction steps. The real-order program checks geometry and implements no decoder. [Finite marked test](small.json) · [Cylinder test and counterexamples](cylinders.json) · [Pinned real reconciliation](audit.json) · [Supplied input hashes](wires.json)

The first real audit used the existing checked K8 last-column/exception mask files. From the authenticated repository and a disposable output directory, run

```bash
review=$(dirname "$(rg --files | rg '/return/collation/induce/review/PLAN.md$')")
out=/tmp/astra-conjugate-review-replay
mkdir -p "$out"
taskset -c 1 nice -n 14 python3 -B "$review/finite.py" "$out"
taskset -c 1 nice -n 14 python3 -B "$review/cylinders.py" "$out/cylinders.json"
taskset -c 1 nice -n 14 python3 -B "$review/order_dp.py" "$out/order-dp.json"
g++ -std=c++20 -O3 -Wall -Wextra "$review/orders.cpp" -o "$out/orders"
taskset -c 1 nice -n 14 python3 -B "$review/real.py" "$out/orders" /tmp/astra-clock-wires "$out"
taskset -c 1 nice -n 14 python3 -B "$review/sort_cost.py" /tmp/astra-clock-wires "$out/sort-cost.csv"
cmp "$out/orders.csv" "$review/orders.csv"
python3 -B "$review/verify_real.py" "$review"
```

This is structural evidence. It establishes neither a full worker improvement nor a target-machine memory result. A decisive engineering follow-up would have to build the chosen section, group all cuts in its measured order, execute every induction and spell the entire marked stream against the same scalar-cap input. A further mathematical opportunity is to prove sharp order-dependent bounds on the unequal steps left after the invariant A−B term. [Registered protocol](PLAN.md)
