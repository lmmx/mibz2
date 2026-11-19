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
