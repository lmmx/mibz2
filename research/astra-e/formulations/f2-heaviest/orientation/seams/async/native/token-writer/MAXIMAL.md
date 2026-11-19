# The physical constructor emits one band per maximal destination run

Fix a stable original column, maximal original L runs, a normalized union of section intervals, and the exact two-front interval/cohort algorithm in 8f1be9d. Label each section destination by the raw word of the positive return ending there. Define a physical run to be a maximal **consecutive original-row interval in the section** with a common destination label.

**Theorem.** The emitted hit bands are exactly the physical runs, with the true affine source owners and true spelling. No reinsertion of an advanced or surviving fragment coalesces two previously separate packets. These statements apply to the specified constructor for any schedule that selects and expands whole live directional cohorts. They need no minimum-cut assumption.

For two sources sharing the next F byte, Ψ is strictly increasing, hence noncontractive on integer rank differences. With two rows carrying the same L byte, their LF image separation is the number of occurrences of that byte in the corresponding half-open rank interval, and is at most their previous separation. These inequalities follow from stable LF independently of periodicity, number of cycles, or placement of the mark.

First consider two consecutive physical destination rows y,y+1 with equal raw return word W. Their sources are ordered because Ψ is strictly increasing at each step of a common word. Their initial separation is at least one and the sequence of separations along W is nondecreasing. Its final value is one. Hence their sources and **all** corresponding intermediate row pairs are consecutive. An adjacent forward pair with equal edge byte and adjacent images traverses exactly one maximal original translation piece: the image L rows are consecutive and have that same byte. In reverse the input L rows themselves are consecutive and equal. Equal-W occurrences have the same pair of cohort histories and retire on the same expansion.

At initialization each of these consecutive owner pairs occupies the same packet in its corresponding direction. Before a common retirement, neither newly translated row can hit another unfinished arc's endpoint. When the two arcs retire, both target rows belong to the same opposite packet by the induction hypothesis. Clipping by original pieces or opposite occupancy therefore never splits this pair. Its two owners are published in one hit band. Since a published band has one word and contiguous destination ranks, no hit band crosses a change of word or a section hole. This proves maximality, including self-return arcs and distinct periodic cycles.

For the coalescence claim suppose adjacent current-rank fragments would join with adjacent owner ranks and a common group. Both survivors have exactly the same directional prefix/suffix history. For forward transport their initial and current owner-pair separations are one, forcing all interim separations to equal one by noncontraction. For backward transport the same conclusion follows by nonexpansion and injectivity. Their neighboring endpoints therefore started in a common packet, used the same translation piece on every completed whole-cohort step, and survived every previous opposite intersection. Two adjacent surviving rows cannot be separated by an intervening opposite-rank integer. Thus no clip could have created the first split. Reinserting the remainder of an opposite hit only removes a nonempty physical rank interval; it cannot bridge an earlier gap. Normalization at initialization is the one condition on the section representation.

For a single interval the number of bands A equals the maximal RLE count R of the induced token column. For disconnected sections,

```text
A = R + number of section gaps whose two neighboring projected labels coincide.
```

For example, stable L=aaa with section ranks {0,2} has two physical self-return bands and one projected token run. Hence a final destination-ordered writer can omit all label-coalescence checks on an individual component and needs to compare at component boundaries when concatenating projected pieces. A rank producer still pays to locate the actual boundaries, assign words and establish destination order. This does not remove the expensive dynamic front discovery.

The earlier F2-dual gap correspondence identifies a delimiter's projected discontinuities by transporting adjacent gaps in the reverse direction; it then pays to recover word labels. The theorem here identifies every actual labelled two-front hit as an already maximal output run. It applies to arbitrary normalized section unions, accounts for cross-component equalities, and locates a redundant mutation in this particular constructor. [Prior gap correspondence](../../../../../../f2-dual/README.md)

The independent dense audit enumerates each source and walks until its first positive return, groups its words by **destination**, and compares against every unsorted physical hit after destination ordering. All binary columns through eight rows and ternary columns through six rows, all nonempty section subsets, passed. Across 141,764 tested column/section pairs the program checks the asserted word, source, destination and exact width, counts 19,416 projected equalities across holes, and observes zero dynamic packet coalescences. On all 456 real registered sections A=R exactly.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams/async/native/token-writer
g++ -std=c++20 -O3 "$d/maximal.cpp" -o /tmp/astra-maximal
/tmp/astra-maximal
```

[Exhaustive receipts](maximal.txt) · [Independent dense comparison](maximal.cpp) · [All 456 physical/root run counts](coverage.csv)
