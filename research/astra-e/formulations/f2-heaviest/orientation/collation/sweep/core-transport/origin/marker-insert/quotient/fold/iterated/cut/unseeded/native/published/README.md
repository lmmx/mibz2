# Publishing the next source geometry as a marked cut produces it

On the same ten unseeded histories, publishing source bases eliminated all 2,203,538 noninitial full-RLE histogram visits. The first ten original byte columns still required 690,484 such visits. Each new mark was located among 1,065,610 emitted pieces in aggregate. The eager arm updated retired per-alias rank counters through the mark. The delayed arm counted the single marked predecessor symbol during 1,017,929 earlier RLE visits in the following mandatory overlay. On rows where the marked position was already known to fix itself, delayed verification could create temporary prefix slices before stopping. The published geometry, full RLE, every current grammar node, ordinal, offset and full output agreed with the original histogram route across all 250 cuts. The same checks covered 66,712 tiny marks, 21,772 forced powers and 15 long-run cases. [All charged events](counts.csv) · [Binary and gate identities](check.json) · [Rank cost and proof](POINT.md)

| Original inverse comparison | Paired thread CPU, 95% cluster interval | Paired wall time, 95% cluster interval |
|---|---:|---:|
| Eager prefix / original histogram | 0.9984 [0.9684, 1.0293] | 1.0021 [0.9747, 1.0304] |
| Delayed / original histogram | 0.9885 [0.9518, 1.0265] | 0.9888 [0.9503, 1.0289] |
| Delayed / eager prefix | 0.9901 [0.9781, 1.0022] | 0.9867 [0.9694, 1.0043] |
| Delayed / dense stable inverse | 0.9554 [0.9362, 0.9749] | 0.9561 [0.9394, 0.9730] |

This 40-round, 1,600-output panel establishes the eliminated work. Its mark-strategy intervals cross parity. Maximum fresh-process RSS was 10,256 KiB for original histogram, 10,292 KiB for each published variant and 8,020 KiB for dense. No full-stream production worker, CI or i9 results are attached to this original inverse stage. [Raw pairs](pairs.csv) · [Fresh peaks](rss.csv) · [All ratios](summary.json) · [Preregistered comparisons](PLAN.md)

```bash
d=$(dirname "$(rg --files | rg '/cut/unseeded/native/published/build.py$')")
tmp=$(mktemp -d)
python3 -B "$d/../build.py" "$tmp/base"
python3 -B "$d/../check.py" "$tmp/base/run" research/astra-e/dense/target/wikidata corpora/wikidata.bz2 "$tmp/inputs" "$tmp/ordinary-gates"
python3 -B "$d/build.py" "$tmp/base" "$tmp/native"
taskset -c 3 nice -n 14 python3 -B "$d/gate.py" "$tmp/native" "$tmp/inputs" "$tmp/gates" > "$tmp/check.json"
diff "$tmp/check.json" "$d/check.json"
diff "$tmp/gates/counts.csv" "$d/counts.csv"
taskset -c 3 nice -n 12 python3 -B "$d/measure.py" "$tmp/native/run" "$tmp/inputs" "$tmp/gates"
python3 -B "$d/report.py" "$tmp/gates"
python3 -B "$d/report.py" "$d" | diff - "$d/summary.json"
```

The implementation is frozen at cdc1de3. Gate binaries are compiled with and without the additional snapshot and independent marked-LF checks. The header injected at build time contains the exact observer operations in source order. [Publication observer](events.hpp) · [Specialization](build.py)
