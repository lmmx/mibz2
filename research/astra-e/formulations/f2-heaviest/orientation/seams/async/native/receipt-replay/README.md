# Which budget remains with free geometry?

A cold, exact paged run takes place before the timers. For each productive group it records child identities, spelling edges, masses, birth ordinals and positively identified whole source/destination hit bands. The first oracle arm receives all rank transport, opposite predecessor queries, fragment splitting and physical owner identification for free. It pays the actual global max-heap pushes, pops, stale priority changes and decreases of opposite mass. Every productive heap pop must name the exact recorded next group. It reconstructs the directional spelling nodes, publishes/sorts the terminal bands, indexes the source ranks and emits and verifies the full marked bytes. The second arm additionally receives the schedule for free and skips dynamic mass and heap maintenance. Neither oracle computes a valid inverse from the original column unaided.

In both arms the original forward index is already supplied for the initial marked fragment. The trace, interval selection, and physical construction of all observed geometry cost zero measured time; reading the effect stream still costs time. Thus these are optimistic executable controls for a particular data flow. Trace reads and the chosen writer prevent their ratios from serving as algorithm-independent lower bounds. The direct comparison also includes its own indexed output, while Compact/preload independently reconstructs the entire marked inverse.

| Measured oracle stage / full reference | Quote ratio [95% interval] | Frequent ratio [95% interval] |
|---|---:|---:|
| Free rank geometry, real heap / Direct | 2.016 [1.901, 2.137] | 2.038 [1.923, 2.160] |
| Free rank geometry, real heap / Compact/preload | 5.900 [5.523, 6.303] | 5.603 [5.402, 5.812] |
| Free geometry and schedule / Direct | 0.717 [0.645, 0.797] | 0.510 [0.475, 0.548] |
| Free geometry and schedule / Compact/preload | 2.097 [1.878, 2.342] | 1.402 [1.294, 1.519] |

Even after precomputing all rank intersections, exact heaviest-first queue maintenance with explicit spelling publication uses twice the complete Direct budget. Granting both geometry and order moves the fixed implementation below Direct but leaves it above complete Compact. In the independent physical panel, merely publishing and expanding already-complete terminal bands used 1.73 and 1.32 times Compact. Replacing only the heap or only the predecessor data structure cannot eliminate that measured component. A new candidate would need a different object handed to the writer, or would need to bypass explicit band indexing and per-band spelling publication. No CI or i9 claim follows.

The replay phase uses the same eight balanced repetitions, real marks, system CPU and controls as the bounded-rank panel. All 320 oracle observations matched real productive-expansion counts and all 160 real-heap oracle observations matched the physical number of lazy heap pops. All 640 paired outputs and 80 isolated process outputs matched the independent expected bytes. In these special arms the CSV field discovered_bands is zero because the column-driven discovery is prepaid; the full bands are supplied in the trace. Fresh-process VmHWM for an oracle includes the prior untimed recording step, both original directional indexes and oracle trace. Use the physical panel for candidate memory.

The separate paged-index stress test exercises sparse and dense rows around 64-, 4,096-, and 262,144-row hierarchy boundaries and the 900,000-row block limit. It performs 60,000 alternating inserts/removals against an ordered set, 600,000 predecessor/successor checks and 75,676 checks that pointers survive unrelated slot growth and recycling.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams/async/native
bash "$d/build.sh" /tmp/astra-async-build "$PWD/target/release/deps"
nice -n 10 taskset -c 4 python3 -B "$d/match.py" /tmp/astra-async-build/replay research/astra-e/dense/target/wikidata /tmp/astra-async-replay --replay
python3 -B "$d/summary.py" /tmp/astra-async-replay
g++ -std=c++20 -O2 "$d/page_test.cpp" -o /tmp/astra-page-test
/tmp/astra-page-test
```

[Paired observations](paired.csv) · [Fresh-process receipts](fresh.csv) · [Count audit](check.txt) · [Mask hierarchy property checks](../panel/page-property.txt)
