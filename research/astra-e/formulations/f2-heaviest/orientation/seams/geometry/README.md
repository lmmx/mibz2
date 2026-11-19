# What if spelling probes are priced by their actual original runs?

A single spelling state can touch many original LF runs. Conversely, a forward cohort can have disjoint fragments on one original translation and can reuse its displacement. Price one event for each distinct (direction, consumed spelling context, original maximal run). Grant free sharing across all participating fragments of that state and run. Charge input/output ordering and fragment manipulation separately. An event is a unit of this explicit translation scheme. It is not a required memory miss or an implementation-independent decoding lower bound.

**The forward and backward edge use the same original run.** If an original L run maps by LF to an interval in F, its inverse Ψ maps that F interval back to the run. The two moves across an occurrence of a given byte use this one run ID, regardless of which end discovered the byte. Genuine full affine return bands can therefore export the sets of run IDs at each byte offset by walking one representative per band and checking that its entire width remains in the same translation interval. This gives the complete incidence sets without one walk per text occurrence.

**Optimizing the real incidences.** Give each word-position a binary cut variable. A forward choice forces its preceding position forward, so a word has a single seam. For each run ID that actually occurs there, a forward choice forces the event for the word's current prefix and that run, and a backward choice forces the event for its current suffix and that run. The two event sets live on the appropriate sides of a source/sink cut; each activated node has unit cost. Infinite arcs enforce the implications. Every feasible seam choice and minimum cut determine equal optimum values by closure. The recovered assignments were independently repriced against the run incidence sets. Setting all run IDs to one recovers the spelling-state model. [Solver and band export](optimize.py) · [713 exhaustive checks](small.txt)

| Ten systematic blocks | All backward | All forward | Alternating seam | Largest-mass seam | Best original-run seam shared by each word |
|---|---:|---:|---:|---:|---:|
| Quote returns | 1,068,084 | 887,556 | 870,697 | 819,916 | 815,694 |
| Frequent quote-return section | 1,423,936 | 1,390,788 | 1,004,977 | 874,402 | 866,587 |

Largest mass came within 0.52% and 0.90% of the respective actual-run incidence optima. A minimum cut priced only by spelling states used 817,790 and 871,503 events. Conversely the assignments returned by the run-cost cut used 641,125 and 714,221 states, against minima of 538,338 and 603,170. Different cuts can tie on their primary price. The observed assignments illustrate why neither state count nor distinct-run count alone measures native work. [All real inputs and both prices](wikidata.csv) · [Paid original-rank constructor](../async/native/)

**Different seams for different occurrences of one spelling.** Occurrences with the same word and complete ordered run trace have identical event requirements at every possible seam. Given an optimal per-occurrence assignment, choosing any seam it uses in that class for all members only removes events from the union. Thus a separate variable sequence per distinct trace suffices to compute the unrestricted per-occurrence optimum. This bound also allows earlier termination of some occurrences within a word cohort. Across 84,317 distinct quote trace classes its optimum was 815,689, just five events below the single-seam-per-word result. Across 66,373 deeper trace classes the saving was nine events, to 866,578. The 31-byte cyclic witness in the linked file has respective prices 20 and 19, disproving equality as a universal theorem. On these ten real sections, redistributing individual occurrences offers almost no further saving at this price. The physically indexed constructor still pays for disjoint packets, live ownership and output. [Per-trace solver](packets.py) · [339 exhaustive checks](packet-small.txt) · [All real minima](packets.csv) · [Constructive counterexample](witness.json)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams/geometry
python3 "$d/optimize.py"
python3 "$d/packets.py"
python3 "$d/optimize.py" research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
python3 "$d/packets.py" research/astra-e/dense/target/wikidata | diff - "$d/packets.csv"
python3 "$d/witness.py" | diff - "$d/witness.json"
```
