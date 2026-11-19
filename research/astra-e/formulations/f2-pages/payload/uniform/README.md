# Embed a complete run response in uniform pages

A split64 page's bit zero is always clear after counted preparation. A page with no internal run start has a zero mask and one containing original run. The fixed-size directory can instead store that run's entire compact response in the unused word and set bit zero. Its neighboring u32 base remains the containing run ID. No additional heap allocation is needed. A tagged answer from this page can serve the unary transition from the directory. If the job retires at a branch, its base still locates the original run for enumerating selectors.

The census instruments the actual dynamic eight-lane discovery at compact-source ab70765 and validates every final inverse independently. It classifies every original 64-row page and all 19,750,839 first-level dynamic queries on the 228 admitted blocks. A cut means a run starting strictly inside the page. The longest covered interval counts ties in favor of the earliest. The complete histogram and wave-level hit distribution are in census.csv and census-summary.txt.

Across 3,206,333 pages, 2,318,843 had zero cuts. Their 148,402,675 original rows comprise 72.3% of all 205,197,794 input rows. Only 2,972,659 dynamic queries (15.05%) landed on them. Of those, 2,582,255 were unary and could use the tag without recovering a run index for branch publication. The remaining 390,404 retired. Among 2,476,362 waves, 942,340 had no uniform hit and just 3,619 had eight. Encoding the longer side of every one-cut page could raise total query coverage to 16.69%, given extra tagged encodings and construction.

Filling the response once for each uniform page entails 2,318,843 extra payload reads in a sequential construction pass, with potential reuse of a run across adjacent pages. It could avoid 2,972,659 dependent, irregular payload loads during discovery, as well as the mask popcount on each hit. All searches pay a tag check and retirements may need a run index. The difference in load kind prevents their raw counts from deciding performance.

```sh
d=research/astra-e/formulations/f2-pages/payload/uniform
bash "$d/census.sh" /tmp/astra-e-uniform-census
python3 "$d/report.py"
```

## Price of two no-allocation tagged lookups

The mask-first route places the packed run's position/byte word in bits 1–28, its wrapping LF displacement in bits 32–63 and a tag in bit zero of the uniform page's mask. Mixed masks keep bit zero clear. A mixed reply is twice the ordinary located index. An odd reply is the embedded payload, which supplies the original byte, end and LF displacement without the dependent read. Only a retiring embedded job uses its page's u32 base to publish selectors. The base-first route additionally tags the high bit of the page base and returns an odd page ID on a hit. Its consumption reads the embedded payload from that page. Both encodings use the same two heap arrays and run table as the eager endpoint at ab70765.

All rows, marks and real outputs checked in the previous compact stage agree for both routes. Per-block array byte capacities matched the control. Two distinct three-arm 60-lap processes checked every output at all six arm permutations on the nine systematic blocks. The table shows paired Linux thread-CPU ratios to the untagged eager endpoint.

| Route | CPU4 setup+mark | CPU2 setup+mark | CPU4 discovery | CPU2 discovery | CPU4 complete inverse | CPU2 complete inverse |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Mask first | 1.1626 | 1.1376 | 1.3208 [1.2609,1.3834] | 1.2437 [1.1953,1.2941] | 1.1606 [1.1310,1.1910] | 1.1331 [1.0957,1.1718] |
| Base first | 1.1599 | 1.1268 | 1.3682 [1.3124,1.4265] | 1.3001 [1.2550,1.3467] | 1.1979 [1.1570,1.2402] | 1.1415 [1.1064,1.1777] |

The tagged discovery and its construction both cost more on each pin. The measured coverage and full-stage regression reject this shortcut for these inputs. The eager endpoint remains reproducible using its immutable source.

```sh
d=research/astra-e/formulations/f2-pages/payload/uniform
bash "$d/run.sh" /tmp/astra-e-uniform-price
python3 "$d/price-report.py" /tmp/astra-e-uniform-price/{first,repeat}.csv
```
