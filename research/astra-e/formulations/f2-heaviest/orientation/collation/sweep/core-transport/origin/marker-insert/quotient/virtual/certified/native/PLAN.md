# Compile the full deterministic root tree into actual second-stage selectors

Accept a whole source bucket for precompilation when every occurrence enters an already certified bucket in the forward graph. Start with the actual heaviest class, including source-order ties. Collect each original token class's count, base, first and last incoming row in a real scan; sort positive classes by source base, binary-search the first endpoint, and test the last. Build reverse adjacency lists and breadth-first certified node/parent order. Each certified class owns exactly one trie node. Preassign its edge, then use a sequential run/bucket intersection sweep to fill children, coalesced selectors and terminal weighted dictionaries directly from the known parent histories. Ordinary boundary children receive normalized intervals in the concatenation of uncertified source buckets. Certified nodes precede every ordinary child, retaining the stock bottom-up merge. The remaining trie discovery uses the ordinary compressed-page search on the complement.

Recover any marked probe in a precompiled class directly from its selector list. Determine the owning selector and within-selector byte-independent ordinal, then add the widths of previous selectors for that child. The child's prepared interval start recovers the exact LF target. Terminal selectors have the existing certified node's d ordinal. The strict depth increase proved in the parent study ensures at most one probe per certified class. The regular charged rewind and bit-budget fallbacks still apply. The fresh capacity report includes all eight new reusable u32 vectors, including sorted types, endpoints, adjacency, projection and BFS ownership.

Test all dense marked adversaries, the ten exact structural snapshots and all real dense outputs. Price six source-matched arms in 60 balanced repetitions on ten inputs. The direct comparisons are compiled/ordinary on named Compact, explicit coincident h4 and explicit coincident h12. Include the full new census and order construction, all certified selectors, complement, old first-word/root publication, mark, final links and actual output. Report complete inverse-stage CPU and summed second index/discovery because root/tree work moves between phase categories. The three quotient candidates retain explicit first-to-second quotient publication in this experiment. Fresh complete 228-input arm processes also check the frozen expected FNVs. Stage timing here will decide whether this precompilation merits further engineering.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/virtual/certified/native
out=/tmp/astra-virtual-forest
python3 -B "$d/build.py" "$out" "$PWD/target/release/deps"
"$out/forest"
"$out/forest" geometry /tmp/astra-clipped/ten.bin
"$out/forest" admissions /tmp/astra-token-all/admissions.bin
nice -n 12 taskset -c 3 "$out/forest" price /tmp/astra-clipped/ten.bin > "$out/phase.csv"
python3 "$d/../../native/report.py" "$out"
```
