# Width-one queries require no endpoint test

Every first-level directory query returns the unique run containing lo. For w=1, the interval [lo,lo+1) is wholly contained in that same run. Its next-byte test needs only the run byte and its stable LF displacement. A delimiter still causes retirement. At a published branch the same argument applies to a final selected piece with only one row remaining. For w>1 the original boundary test is required.

On the frozen eight-slot preload (6758831), the separate native observer measured 15,325,702 singleton intervals in 19,750,839 actual queries on 228 admitted Wikidata blocks. The 547,404 delimiter hits among them retire; the rest are unary. All 886,075 fully singleton waves had eight live slots. Summing the interval widths over every query gives the full 205,197,794 admitted rows. The census also checked the complete inverse against Segmented on every block. Its bins group widths [1,2), [2,4), ... through [256,infinity). The wave column at index i counts waves with exactly i singleton queries.

This permits an exact 32-bit hot word containing (delta mod 2^20) and the byte. Every legal translated row lies below 900,000, so modular addition with a final 20-bit mask recovers LF(x) for positive or negative displacements. A separate u32 array supplies exclusive endpoints when required. Splitting retains eight bytes per run overall, doubles the number of independently appended vectors, and can reduce the hot working set on singleton-heavy discovery. An ordinary interleaved control can also omit the logical endpoint test. These effects require direct stage pricing.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/singleton
bash "$d/census.sh" /tmp/astra-e-singleton-census
```

## Paid split-payload test

Four arms shared the frozen first-level eight-response preload. Each constructed its own published records, marked permutation, paths and output. Each produced the Segmented output for all 228 admitted Wikidata blocks (205,197,794 bytes). Six tests for marked columns and record geometry passed. Both separate u32 allocations retained precisely as many total payload bytes as the interleaved allocation for every observed block. `geometry.csv` records the cold allocation separately. This comparison does not measure whole-process RSS.

Two pinned processes ran 64 fully balanced four-arm laps on nine frozen blocks. Each period used all four arms in each position 16 times per block. Whole inverse thread-CPU ratios against the unchanged eight-byte interleaved preload were as follows.

| Variant | CPU 4 | CPU 2 |
|---|---|---|
| Interleaved record, skip singleton endpoint | 1.0142 [0.9876,1.0416] | 1.0169 [0.9956,1.0387] |
| Split hot/cold, always check endpoint | 1.0663 [1.0372,1.0962] | 1.0641 [1.0338,1.0953] |
| Split hot/cold, skip singleton endpoint | 1.0908 [1.0603,1.1223] | 1.0706 [1.0502,1.0914] |

The split singleton variant spent 15.2% and 18.2% more CPU in construction and marking, and 13.4% and 10.7% more in discovery. The high singleton rate did not pay for the separate allocation, conditional access and masked delta arithmetic together. The experiment cannot assign the slowdown to a single one of those costs. Keep `6758831` as the fixed-response candidate. This native experiment supplied no reason to schedule the loser for whole-worker or workstation comparison.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/singleton
bash "$d/run.sh" /tmp/astra-e-singleton-payload
python3 "$d/report.py" /tmp/astra-e-singleton-payload/{first,repeat}.csv
```

`manifest.txt` hashes the generated source, binary and frozen moonshot-enabled library. `PAGE_DEPS` selects the release `deps` directory of a compatible build from `252646a`. The initial construction census and the priced experiment use separate raw receipts.
