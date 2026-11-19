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
