# Exact same-run arithmetic skipping bound

For a unary interval [x,x+w) contained in a non-delimiter run [s,e), LF acts by adding a constant signed displacement d. While the translated interval stays inside [s,e), its next unary symbol is the same byte and its next run response is already known. For d>0 the number of further consecutive unary steps in the same run is floor((e-x-w)/d). For d<0 the count is floor((x-s)/(-d)). A zero-displacement non-delimiter interval would stay in a quote-free cycle forever and cannot be reached by this delimiter-rooted expansion; the observer checked every reachable case. Each internal transition counted at the preceding step corresponds to exactly one subsequent rank query a perfect repeated-step shortcut could avoid. Counts of eligible transitions therefore give an upper bound without overlap inflation.

An observer on the frozen eight-slot compact preload at 6758831 computed the exact criterion on every actual first-level dynamic lane, outside timed comparisons. It charged no setup, no division, no extra reads of previous ends and no cost for recognizing the opportunity. On all 228 canonical real admissions, all reconstructed bytes agreed with Segmented. There were 19,750,839 queries, including 18,763,157 unary steps. Exactly 47 transitions (0.000238% of queries) stayed in the same original run. All 47 moved forward; at most three further steps were possible from a single interval. This rules out a throughput-relevant gain on these inputs from this restricted arithmetic shortcut even at zero recognition cost. Whole-run successors between distinct runs are a different candidate already studied in F2d.

```sh
d=research/astra-e/formulations/f2-pages/payload/self-run
bash "$d/census.sh" /tmp/astra-e-same-run
```
