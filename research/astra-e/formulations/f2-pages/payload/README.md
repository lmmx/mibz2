# A compact single-run response to a predecessor query

The guarded hardware bit-count candidate is fixed at 3651470cb0ef5458e75cf97d3de363a1c73f2f76. Its first-level directory returns the index of the coalesced input run containing the queried row. That record originally held a u32 start, a u32 LF image of the start and a byte, occupying twelve bytes with padding. The unary path also loaded the next record's start to test interval containment. The packed successor field in the previous F2d frontier study retained twelve bytes to learn whole-run successors; that is a different payload.

For a run on [s,e), let y=LF(s) and store d=y-s modulo 2^32. Stable LF preserves the run's contiguous offset, so for any x in its interval, wrapping_add(x,d) is y+x-s. The desired rank lies below the block length. Thus this representation is exact for either sign of the displacement. Each new arrival knows s, its byte and y from the front histogram and the current per-byte position. The following run's start supplies e, including after any number of cap boundaries. Finishing the final run with e=n gives the last proper record.

The eight-byte endpoint record stores ((e << 8) | byte, d) in two u32 words aligned to eight bytes. At most 900,000 rows fit comfortably in the upper 24 bits of the first word. For a query [lo,hi), end < hi is equivalent to the first word < (hi << 8) for any byte. A unary transition can test containment, append the byte, and obtain the mapped row from the single indexed record. At a published branch the first selected row is lo, already inside the run supplied by the directory. With cursor a, take [a,min(e,hi)), map a by the stored displacement, emit that width, and move a to the end. If a<hi the next run starts exactly at a. This advances from the old selector to the identical stable ordering by induction and needs no sentinel record. The marked transfer uses the same mapping formula. The code retains all existing recursion, output reconstruction, original admission and fallback behavior.

A separately compiled eight-byte start record stores ((s << 8) | byte,d). It retains the original use of the next record's start and a terminal sentinel. Comparing this arm directly to the endpoint arm distinguishes capacity reduction from the replacement of adjacent-record queries. Both arms retain the original split64 directory and guarded POPCNT setup and discovery.

The exact checks independently recover LF from a fresh exclusive byte histogram and occurrence counts, test both sides of every run, run dense alternating columns and long capped runs, and compare marked outputs with the stable-sort oracle at short lengths. The full geometry and timing harness compares with the same 3651470 reconstruction. Each worker retains independent run storage.

```sh
d=research/astra-e/formulations/f2-pages/payload
bash "$d/run.sh" /tmp/astra-e-payload
python3 "$d/report.py" /tmp/astra-e-payload/{first,repeat}.csv
```

## Measurements

Both payloads matched the independent original-row LF oracle including every row of the dense and capped adversaries. Every forced marked output agreed on all 11,264 accepted small marks per arm. Both whole inverse implementations also agreed with Segmented for every one of the 205,197,794 admitted real inverse bytes. The 228 blocks contained 14,427,155 proper runs. The start-based arrays used one extra sentinel per block. Under retained worker reuse the original twelve-byte run vector peaked at 1,572,864 B; both compact vectors peaked at 1,048,576 B. Other buffers and allocator or process overhead are additional.

Two new three-arm, 60-lap processes priced ready-run inversion on the nine systematic blocks; all six arm orders occurred per block and the block order changed on every pass. The direct control was the guarded original population-count kernel from 3651470. Each phase and the complete inverse were timed. The observations and both clocks are in first.csv, repeat.csv and summary.csv.

| Variant relative to twelve-byte control | CPU4 setup+mark | CPU2 setup+mark | CPU4 discovery | CPU2 discovery | CPU4 complete CPU | CPU2 complete CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Start8 | 1.0293 | 1.0438 | 0.9211 [0.8736,0.9712] | 0.9293 [0.8977,0.9619] | 0.9750 [0.9371,1.0144] | 0.9686 [0.9433,0.9945] |
| End8 | 1.2537 | 1.2521 | 0.8774 [0.8372,0.9196] | 0.8933 [0.8564,0.9318] | 0.9746 [0.9349,1.0159] | 0.9751 [0.9450,1.0062] |

The endpoint lookup accelerated discovery beyond the start-based lookup. Its current construction buffers a pending run until the next distinct byte, adding about 25% to setup+mark on both pins. The compact start record has the stronger measured complete ratio on CPU2 and a smaller timed setup penalty on both. Separate direct full end8/start8 estimates included parity. Both variants are preserved at 112c5c17f952db8385af5d6962c5e0787cc2d0c2. The next constructor can avoid the pending-record state by reading each complete original run before emission.
