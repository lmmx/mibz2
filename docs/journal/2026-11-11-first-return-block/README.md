# First-return real-block result: revise metadata

The single authorised run completed with exact RLE1 bytes, output, selected
origin, cycle length and CRC. The delimiter representation also passed its
token-BWT/LF assertion. It passed the traversal-count screen but failed the
metadata screen: **REVISE this configuration, not retain for a compiled-cost
experiment.** No speedup or production qualification follows.

Dispatch [363/6808027528](https://github.com/lmmx/mibz2/issues/363#issuecomment-6808027528),
rule [145/6793754589](https://github.com/lmmx/mibz2/pull/145#issuecomment-6793754589).
Sources/inputs frozen at `54622528abc7226e080a0c3d9e1b483c7407c512` and posted
in [145/6808415800](https://github.com/lmmx/mibz2/pull/145#issuecomment-6808415800)
before candidate execution. [PLAN.md](PLAN.md), [FREEZE.json](FREEZE.json) and
[identities.json](evidence/identities.json) remain unchanged. Attempt count 1,
retry count 0; no timeout or resource exhaustion, child exit 0, empty stderr.
All sources are research-only. No production/unsafe/default/team changes.

## Exact result

Corpus, native baseline and original prototype are pinned in the plan. Native
stream-order selection chose block 1, bit 336, n=899931, primary=264045,
delimiter=34. Baseline is unchanged Segmented; Cells was a separate comparator.
Independent row LF found one cycle of length n; libbzip2 matched all 900000
output bytes after the preceding 13 output bytes. This is one actual block,
not a new arbitrary-column or multiple-cycle census.

| Account | Actual count |
| --- | ---: |
| Capped run records read | 56,386 (112,772 bytes) |
| Maximal input runs / conversion visits | 51,184 |
| Coalesced capped records | 5,202 |
| Anchor intervals / rows | 679 / 135,716 |
| E, constructor interval visits | 124,080 |
| Weighted constructor rows | 899,931 |
| Origin steps / lookup comparisons | 0 / 15 |
| Residual token visits | 135,716 |
| E + origin steps + residual visits | **259,796 < 899,931** (0.288684n) |
| Boundary touches / splits | 130,027 / 5,947 |
| Frontier peak / pending+completed peak records | 5,303 / 6,626 |
| Completed pieces / merged Q | 6,626 / 6,133 |
| Word-pair queries / H created and live | 123,401 / 70,559 |
| Binary-search comparisons, all categories | 5,687,908 |
| Sorts / sort items / comparisons | 2 / 9,342 / 38,176 |
| Dictionary lexical byte comparisons | 123,473 |
| Distinct dictionary words / stored bytes | 2,716 / 83,454 |
| Dictionary expansion calls / grammar visits | 6,133 / 267,425 |
| Dictionary bytes expanded (includes repeated IDs) | 136,779 |
| Residual grammar visits / bytes emitted | 1,664,146 / 899,931 |
| Contiguous copy calls / bytes | 283,733 / 2,973,590 |
| Counted logical byte writes | 4,910,300 |
| Excess over one RLE1 write plus one final-output write | 3,110,369 |
| RLE1 decisions / CRC byte updates | 899,795 / 900,000 |

Zero origin steps means the selected primary was already an anchor; it does
not demonstrate the value of origin tracking on another block. Counter omits
untouched zero keys; the lookup and its comparisons are still charged.
Phrase lengths range 2–272. Full distinct-word length histogram and the
copy-length histogram are in [result.json](evidence/attempt-1/result.json).
Repeated dictionary IDs are expanded repeatedly as in the original prototype;
the counts expose that work rather than silently improving this candidate.

## Memory and cost interpretation

| Live candidate metadata sample | Bytes |
| --- | ---: |
| Run conversion, including live intermediate run records | 14,958,490 |
| Constructor complete, including completed pieces and hash cons | 22,896,147 |
| Expanded dictionary and token ordering | 23,458,573 |
| Final metadata, including residual source index | 23,511,653 |
| Frozen threshold, 4n | **3,599,724** |

These deduplicated Python object-graph samples prove peak metadata is **at
least 6.53 times the threshold**. They exclude the separately charged offline
oracle. Exact peak metadata is unavailable; a lower bound already suffices
to fail this gate. Cumulative record/field charge is 58,864,532 bytes and
growing-container charge 59,805,144 bytes; these are conservative logical
allocation accounts, not exact allocator totals, and should not be added to
the live samples as a peak estimate. Hash-cons storage is included in the
samples; dictionary payload and live records are not hidden behind counts.

Final live output buffers occupy 2,853,179 bytes. Whole-child peak RSS was
87,540 KiB, including interpreter, counters, audit sets/stacks and a 2 MiB
reporting reserve. The hard 256 MiB address-space limit and 600-second CPU/wall
bounds were enforced; RSS is neither exact metadata nor a native baseline
comparison. Instrumentation visited 1,389,231 metadata objects, separately
counted. Logical byte writes exclude opaque allocator/interpreter traffic;
they are not measured hardware writes or a complete memory-traffic estimate.

Offline validation separately wrote n row-LF entries and n inverse entries,
visited n rows for cycle discovery and n for inverse output. Its explicitly
listed column/LF/inverse/seen/decoded/baseline objects occupy 10,893,459 bytes;
the compressed input is 11,854,933 bytes and libbzip2 prefix 900,013 bytes.
These are named objects, not the oracle's exact peak. Native baseline scratch
peak was not measured. Candidate input was the actual run stream, not the
expanded column or oracle-built return paths.

The expected possible advantage was fewer interval/occurrence visits. That
survives this block's traversal heuristic: 71.13% fewer counted visits under
the registered sum. It closes **no measured decoder-time gap**. The sum does
not include 5.69 million search comparisons, grammar expansion, dictionary
ordering, copying or final RLE1/CRC. Fragmentation produces Q=6133, but the
70,559 live grammar nodes and Python records dominate representation cost.
The weighted constructor still covers every row. Representation compression
must not be confused with avoided construction or output work.

**Recommendation:** preserve the exact delimiter/token-LF correspondence and
this negative metadata verdict; revise representation/routing before any
compiled-cost request. Python object overhead does not prove a compact native
layout impossible. Conversely, a hypothetical compact layout does not satisfy
this run's frozen gate. No native layout or cost is measured here. The next
decision belongs to the Coordinator: use this evidence in already-queued
performance-opportunity pricing (6807501300), without rerunning or broadening
this probe. Stopped treap and deferred summary decisions stay unchanged.

## Reproduction and checks

Preparation and sole-attempt commands are in PLAN.md. Native test: 1 passed,
104 filtered; logs preserved. Original prototype blobs verified byte-for-byte
against their pinned commit. Scoped Ruff check/format and Python compilation
passed. No new unsafe code is present; no B5 certification is claimed.

The four gzipped artifacts reconstruct the exact exported inputs without
needing another native preparation. Their uncompressed hashes are in
identities.json; all retained evidence files have [SHA256.json](evidence/SHA256.json)
entries. Validate sources, artifacts, histograms, output identities and frozen
rule arithmetic **without executing the candidate again**:

```sh
python3 docs/journal/2026-11-11-first-return-block/verify_evidence.py
```

The frozen probe is deliberately unmodified after its attempt. The verifier
is a later artifact-consistency check, not a second experiment. Exact runtime
paths are in environment.json and supervisor.json; they are provenance, not
required installation paths. Native build logs contain incidental elapsed
times, which are not performance evidence. A raw log's terminal blank line is
preserved. Independent review and CI status must be assessed separately from
these local checks.
