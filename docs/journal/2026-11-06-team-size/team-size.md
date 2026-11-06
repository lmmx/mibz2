# Team size against live tables: wikidata

Runs: 320; errors: 0; output hashes: 1 (same bytes). Reference: `mibz2@8`. Paired ratio: the reference's wall over the cell's, by round (above 1 is faster), its geometric mean and 95% interval.

| teams × k | arm @ threads | tables | MB/s | vs ref | busy cores | leader CPU | helper CPU | scanner CPU | peak RSS | L3 hit share | L3 misses /KB | cycles /B | IPC | L3-miss stall share |
|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 2 × 3 | `mibz2-team3@8` | 7.2 MB | 1,116 | 0.863 (0.846–0.881) | 4.4 | 0.165 | 0.085 | 0.050 | 52 MB | – | – | – | – | – |
| 2 × 4 | `mibz2-team4@8` | 7.2 MB | 1,159 | 0.905 (0.891–0.919) | 5.2 | 0.155 | 0.073 | 0.050 | 52 MB | – | – | – | – | – |
| 2 × 4 | `mibz2-team4@10` | 7.2 MB | 1,166 | 0.915 (0.904–0.926) | 5.2 | 0.155 | 0.073 | 0.050 | 52 MB | – | – | – | – | – |
| 3 × 3 | `mibz2-team3@10` | 10.8 MB | 1,382 | 1.075 (1.058–1.093) | 6.4 | 0.127 | 0.065 | 0.050 | 67 MB | – | – | – | – | – |
| 3 × 4 | `mibz2-team4@12` | 10.8 MB | 1,399 | 1.097 (1.083–1.111) | 7.5 | 0.123 | 0.059 | 0.060 | 68 MB | – | – | – | – | – |
| 4 × 2 | `mibz2@8` | 14.4 MB | 1,276 | 1.000 (1.000–1.000) | 6.6 | 0.135 | 0.080 | 0.060 | 82 MB | – | – | – | – | – |
| 4 × 2 | `mibz2-team2@8` | 14.4 MB | 1,274 | 1.001 (0.989–1.012) | 6.6 | 0.136 | 0.080 | 0.055 | 82 MB | – | – | – | – | – |
| 4 × 2 | `mibz2@10` | 14.4 MB | 1,265 | 0.992 (0.983–1.002) | 6.6 | 0.136 | 0.083 | 0.050 | 82 MB | – | – | – | – | – |
| 4 × 2 | `mibz2@12` | 14.4 MB | 1,274 | 0.996 (0.985–1.008) | 6.6 | 0.138 | 0.080 | 0.055 | 82 MB | – | – | – | – | – |
| 4 × 2 | `mibz2@16` | 14.4 MB | 1,264 | 0.987 (0.976–0.997) | 6.6 | 0.135 | 0.083 | 0.050 | 82 MB | – | – | – | – | – |
| 4 × 3 | `mibz2-team3@12` | 14.4 MB | 1,300 | 1.017 (1.008–1.026) | 8.5 | 0.133 | 0.073 | 0.065 | 82 MB | – | – | – | – | – |
| 4 × 4 | `mibz2-team4@16` | 14.4 MB | 1,279 | 1.003 (0.993–1.014) | 10.1 | 0.135 | 0.068 | 0.070 | 82 MB | – | – | – | – | – |
| 5 × 2 | `mibz2-team2@10` | 18.0 MB | 1,152 | 0.899 (0.891–0.908) | 8.4 | 0.154 | 0.100 | 0.070 | 96 MB | – | – | – | – | – |
| 5 × 3 | `mibz2-team3@16` | 18.0 MB | 1,147 | 0.899 (0.891–0.907) | 10.4 | 0.152 | 0.083 | 0.080 | 96 MB | – | – | – | – | – |
| 6 × 2 | `mibz2-team2@12` | 21.6 MB | 1,051 | 0.823 (0.816–0.831) | 10.0 | 0.165 | 0.109 | 0.080 | 110 MB | – | – | – | – | – |
| 8 × 2 | `mibz2-team2@16` | 28.8 MB | 882 | 0.690 (0.683–0.697) | 13.2 | 0.203 | 0.135 | 0.100 | 138 MB | – | – | – | – | – |

## The rule's predicates

- Every run of every cell the same output hash and length, no run failed: **yes** (1 hash(es), 1 length(s), 0 failed).
- The reference `mibz2@8`: rounds 0–19 once each, every run 4x2: **yes**.
- The whole grid, 16 cells × 20 rounds, each round once, no failed run: **yes**.
- The control `mibz2-team2@8` 1.001 (0.989–1.012), 20 pairs: **within ±2%**.

| primary | plan in every run | 20 unique pairs | mean ≥ 1.03 | lower bound > 1.00 | raises the ceiling | peak RSS / ref (≤ 1.15) | CPU / ref (≤ 1.5) |
|---|---|---|---|---|---|---|---|
| `mibz2-team3@12` (4x3) | yes | yes | NO (1.017) | yes (1.008) | **no** | 1.00 | 1.26 |
| `mibz2-team4@16` (4x4) | yes | yes | NO (1.003) | NO (0.993) | **no** | 1.00 | 1.52 |

A pass here is not a qualification: the same cell must pass the same predicates, within the adoption limits, in one predeclared second run of 20 rounds. The adoption limits are ratios of medians (the cell's median peak RSS and CPU-seconds over the reference's) and must hold in both runs.
