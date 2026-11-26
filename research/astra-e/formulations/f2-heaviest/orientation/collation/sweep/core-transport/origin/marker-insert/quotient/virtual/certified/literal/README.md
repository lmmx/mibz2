# A producer-supplied complete child on unseen real blocks

The selected word ending in 7d2c22 is the original native right-brace, comma, quote. Its noncoincident retained word spells just the right brace. In the quotient's incoming column every one of its occurrences is an inserted hazard inside the marker source. Consequently it is always a complete child when the actual pivot equals Z. The producer passes those two IDs during its existing dictionary classification. The second stage can prepare that source bucket without sorting candidates or measuring the first and last incoming occurrence of every type. [Source and held-out protocol](PLAN.md)

Nine of the ten training caps/primary payload hashes overlap the 228 admitted inputs, leaving 219 independent held-out blocks. The fixed class was present and fully inside the actual pivot on every held-out block at both horizons. It captured a median 87.48% of eligible first-generation record savings at h4 (minimum 68.18%), and 85.84% at h12 (minimum 67.60%). Both targets passed on all 219 blocks. All 438 dense marked outputs and independently computed actual rank-record counts matched, as did all 24,906 tiny quotient cases. [Identity manifest](manifest.csv) · [All held-out rows](heldout.csv) · [Preregistered decision replay](decision.json) · [Small cases](small.txt)

| Policy | Ten-block ordinary / fixed rank records | Complete inverse CPU, fixed / ordinary | Maximum retained bytes, ordinary / fixed | Single fresh peak KiB, ordinary / fixed |
|---|---:|---:|---:|---:|
| Named, root-only control | 84,317 / 78,647 | 1.00296 [0.98451, 1.02175] | 8,473,542 / 8,507,990 | 8,248 / 8,440 |
| Quotient h4 | 84,427 / 28,940 | 1.02452 [1.00755, 1.04177] | 7,896,294 / 7,631,294 | 8,168 / 8,420 |
| Quotient h12 | 84,641 / 22,897 | 1.00520 [0.98490, 1.02592] | 7,934,870 / 7,669,246 | 8,300 / 8,260 |

The canonical trie node counts agree in each matched pair. The h4 and h12 fixed variants used 1.05410 [1.03324, 1.07538] and 1.06674 [1.04057, 1.09356] of named Compact's complete inverse CPU in this same rotor. The held-out finding is a reliable rank-storage property. The paired complete measurements supply no throughput gain for this source against its ordinary policy. In particular, h4 reusable allocation fell while its single-process RSS increased. [Native shape](metrics.csv) · [3,600 raw complete calls](phase.csv) · [Cluster comparisons](summary.txt) · [1,368 fresh-process hashes and peaks](rss.csv) · [Fully implicit factorial](combined/PLAN.md)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/virtual/certified/literal
out=/tmp/astra-producer-literal-final
python3 -B "$d/build.py" "$out" "$PWD/target/release/deps"
python3 "$d/corpus.py" /tmp/astra-clipped/ten.bin /tmp/astra-token-all/admissions.bin /tmp/astra-literal-219.bin | diff - "$d/manifest.csv"
"$out/literal" held /tmp/astra-literal-219.bin | diff - "$d/heldout.csv"
python3 "$d/decide.py" "$d" | diff - "$d/decision.json"
nice -n 12 taskset -c 3 "$out/literal" price /tmp/astra-clipped/ten.bin > "$out/phase.csv"
python3 "$d/../../native/report.py" "$out"
```
