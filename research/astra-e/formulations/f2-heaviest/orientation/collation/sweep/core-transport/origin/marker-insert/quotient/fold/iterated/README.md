# Saturating the original quote-word column

The deterministic-class fold reduced all ten original quote columns to **one row each**, with the marked word spelling the input block. The schedule applied a maximum-weight compatible set at each step, without introducing a new primary ID. Nine of the ten inputs have 900,000 byte rows or close to that size. The ten together contain 8,999,915 bytes. The program compared the complete native second-or-fallback output with original byte LF after every round, checked each changed permutation independently, and propagated its marked rank and byte offset. The final word sizes sum to exactly the input byte mass. This is a constructive terminal result for the ten sampled blocks. The [singleton theorem](GRAMMAR.md) gives a modified universally terminating marked schedule even when an arbitrary original column has no eligible class.

| Rounds applied on each block | Rows | Input runs at that depth | Distinct word bytes | Subsequent second lookups | Subsequent second literal bytes |
|---:|---:|---:|---:|---:|---:|
| 0 | 1,357,067 | 84,317 | 1,077,638 | 69,999 | 1,926,457 |
| 1 | 954,191 | 83,280 | 1,161,701 | 247,877 | 3,847,233 |
| 4 | 544,740 | 69,952 | 1,793,662 | 152,404 | 4,685,331 |
| 8 | 347,934 | 57,199 | 3,104,759 | 97,520 | 6,016,436 |
| 16 | 144,031 | 32,444 | 5,306,021 | 53,007 | 7,324,346 |
| 32 | 30,671 | 8,253 | 8,380,196 | 12,378 | 8,839,791 |

The final depths ranged from 61 to 255. The cumulative incoming run count over changed passes was 1,484,250, and the cumulative number of old IDs was 609,259. These count input sizes, before multiplying by the number of scans in the MWIS and streaming constructor. The recorded second-lookup column simulates the stock contracted discovery for comparison, including its root. The actual native consumer bypassed that discovery at the terminal singleton. Finishing with the current heaviest word after one fold more than tripled rank queries. Saturation has a different end condition and incurs repeated construction. A weighted grammar avoids writing every intermediate byte dictionary. Native timings separate these alternatives. [Every depth and actual native disposition](wikidata.csv)

```bash
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated
python3 "$d/trace.py" /tmp/astra-token-all/libtoken.so research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
```
