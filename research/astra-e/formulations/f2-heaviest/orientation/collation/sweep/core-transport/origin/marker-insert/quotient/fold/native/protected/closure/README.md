# Exact protected rank-query optimum on the ten frozen quote roots

The maximum-weight row policy already removed the **maximum possible 7,716 out of 69,999 stock discovery queries** on each tape in aggregate, and reached the exact per-tape optimum on all ten. The 137 additional individually removable states from the compatibility relaxation cannot be removed simultaneously. The two policies can disagree on other inputs. A 13-row single-cycle stable BWT has a unique row-optimal choice deleting two rows and zero queries, and a different unique query-optimal choice deleting one row and one query. [Exact solver and per-tape values](solver.csv) · [Counterexample](witness.txt) · [Original query theorem](../THEOREM.md)

Minimizing the copied first-level spelling bytes at the exact maximum gives a materially different feasible policy. It selects 16,265 classes over the ten tapes against 32,098 for the row optimum, appends 763,903 spelling bytes against 1,116,807 (31.60% less), and deletes 151,160 original visits against 215,523. Native application of every selected mask yielded the predicted 62,283 actual stock queries, the identical canonical second dictionary of 1,926,457 bytes, the identical marked second incoming tape and 245,445 second links. The solver also checked 6,826 small graph/reward instances, including 1,108 rewarded odd components, against brute force on both objectives. [Chosen old source bases](policy.csv) · [Checked marked native applications](apply.csv) · [Brute receipt](solver-small.txt)

## Sparse geometry and optimization

Suffix-state intervals are disjoint and partition the original d-reachable rows. Overlaying the Q intervals with the old R run tape has at most R+Q−1 nonempty pieces: each state contributes its first piece, and every additional piece contains a distinct interior old run boundary. This holds even if cycles outside d leave holes. Each state consumes its corresponding incoming runs and uses stable LF ranges to form its distinct child intervals. A state whose incoming labels all belong to the eligible pool yields one positive conjunction. The native traversal uses the old stock second rank index and per-class stamps, and visits each nonempty intersection exactly once. It emits only distinct labels within a support. The ten roots had 144,782 intersections, 108,637 distinct state-label incidences, 7,853 individually eligible states and 24,174 eligible incidences; grouping repeated sets reduced the instance to 1,832 supports with 18,095 incidences. [Sparse Rust traversal](second.rs) · [Original type/support inputs](features.csv.gz)

Every support has one common successor parent. Bipartite coloring transforms each exclusion edge into a directed implication; all members of each support have the same color. An odd component requires conditioning on at most one binary vertex. A positive conjunction then takes one auxiliary node and one implication per member, with a constant and opposite orientation for the complemented color. Each component scales a saved query above the sum of all its candidate spelling costs to implement the exact two-objective order. Python's integer-capacity Dinic solver returns a minimum cut and a compatible mask. The ten cases contained 134 rewarded components, all bipartite, and the constructed flow networks used 17,513 variable/auxiliary vertices and 17,554 implication arcs. [Maximum-closure proof](../QUERY-OPTIMAL.md) · [Sparse exact reference](solve.py)

## Fresh policy pricing

Twelve warmed geometry rounds per tape rebuilt the original named quote root and all support tables. For ten blocks the existing first root averaged 13.409 ms thread CPU. Fresh eligibility cost 2.682 ms, original stock index construction .923 ms, sparse interval geometry 2.531 ms and type-table materialization .140 ms. These stages alone added 6.276 ms before solving any cut or publishing the folded root. The prior protected maximum-mass experiment measured a .223 ms arithmetic mean saving in second discovery per ten tapes. Changing the protected class selection cannot eliminate another query beyond that already attained value. Different masks can still change merge geometry, cache cost and construction. [Native census rounds](census-price.csv) · [Matched original protected stages](../summary.txt)

The complete live reference rebuilt a fresh table from original caps, solved each sparse instance in a persistent Python process, passed the resulting source bases back to Rust, reapplied fixed eligibility checks, republished the root, ran the stock second and verified every marked output. Eight complete ten-block rounds alternated named/fresh order on each input. Average elapsed time was 204.776 ms fresh and 23.122 ms named, paired ratio 8.943 [7.845, 10.195]. Across the process boundary, the runner measured 34.795 ms caller CPU and 168.174 ms solver CPU per ten. The sum was 8.955 [7.880, 10.177] of named CPU. End-to-end timing includes binary policy framing, repeated classification and return/application of the mask. The Python solver's measured stage is specific to this executable reference; a different implementation could reduce that cost. All native support construction and the limit on removable queries remain independently measurable. [Frozen reference design](PAID.md) · [All paired calls](fresh.csv) · [Phase report](summary.txt)

## Reproduction

The input/support proof and exact native application are at `e304faf03d3c8eb34752d6f3236632d3252e79b3`. The complete live-policy binary source is `94369a9e666dc73a4c7965a168eb81396e832f70`, following preregistrations `096bfc5` and `5026b64`. The input is the same frozen ten-cap export as the preceding native studies. [Input, library, binary, language and solver identities](environment.txt)

```bash
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/native/protected/closure
out=/tmp/astra-closure-replay
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/fold" export /tmp/astra-clipped/ten.bin > "$out/features.csv"
gzip -cd "$d/features.csv.gz" | diff - "$out/features.csv"
python3 -B "$d/solve.py" | diff - "$d/solver-small.txt"
python3 -B "$d/witness.py" | diff - "$d/witness.txt"
python3 -B "$d/solve.py" "$out/features.csv" "$out"
diff "$out/policy.csv" "$d/policy.csv"
"$out/fold" apply /tmp/astra-clipped/ten.bin "$out/policy.csv" > "$out/apply.csv"
diff "$out/apply.csv" "$d/apply.csv"
nice -n 12 taskset -c 3 "$out/fold" price /tmp/astra-clipped/ten.bin > "$out/census-price.csv"
nice -n 12 taskset -c 3 "$out/fold" fresh /tmp/astra-clipped/ten.bin "$d/server.py" > "$out/fresh.csv"
python3 -B "$d/report.py" "$out"
```
