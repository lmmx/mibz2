# Guarded POPCNT for the second token column

The fixed eight-slot preload at 6758831 still uses generic rank/popcount on the second phrase alphabet. Prior structural observations count 1,424,280 second-level discovery rank queries across the 228 canonical admissions, compared with 19,750,839 first-level queries. The second stage also constructs a dictionary, merges selectors and writes the final inverse through token links. [Operation counts](../../../../f2-rootstream/frames/cut/account.txt)

Two independent changes to the second column were priced. The guarded arm dispatches once on CPU support before its full discovery loop. The fused arm also sets the mask for every token-run start except the first as it appends that run. A mask scan then encounters starts in increasing row order. At a page boundary it increases the carried run ordinal by bit zero, stores the resulting ordinal, clears bit zero, and carries forward the cardinality of the remaining bits. The first start is omitted because its ordinal is zero. Thus the stored page base is precisely the ordinal containing the page's first row. The remaining inclusive mask prefix counts precisely the internal starts through a queried row. Zero-start interior pages inherit the preceding ordinal. No extra vectors are required. Runtime x86 POPCNT detection has a portable fallback. The recorded binary contains hardware POPCNT in the specialized discovery and fused directory scan. The CPU reused the compiled discovery body between guarded variants.

Four arms shared the same first-level eight-response implementation, with the frozen unmodified candidate and a refactored generic second column as controls. All arms agreed with Segmented on 228 admitted Wikidata blocks (205,197,794 inverse bytes). The stable-sort marked tests and 900,000-row first-run geometry test passed. The two independently pinned processes each ran 64 balanced four-arm laps on nine admissions. The second-stage time includes dictionary materialization, merge, links, and writing the actual bytes. The complete time is thread CPU for both levels, including setup and output.

| Native ratio vs frozen preload | CPU 4 | CPU 2 |
|---|---|---|
| Guarded second discovery, residual | 0.9829 [0.8445,1.1439] | 0.9870 [0.9556,1.0196] |
| Guarded second discovery, complete | 1.0886 [0.9547,1.2412] | 0.9807 [0.9410,1.0221] |
| Guarded and fused directory, residual | 0.9311 [0.8137,1.0654] | 0.9806 [0.9414,1.0214] |
| Guarded and fused directory, complete | 1.0183 [0.8941,1.1597] | 0.9801 [0.9400,1.0220] |

The CPU 4 process had wide intervals on the shared host. Neither run resolves a complete advantage. Keep the frozen preload for worker and CI comparison. These arms have no whole-worker, RSS, or i9 result.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/second-rank
bash "$d/run.sh" /tmp/astra-e-second-rank
python3 "$d/report.py" /tmp/astra-e-second-rank/{first,repeat}.csv
```

The corpus hash and toolchain are recorded in the runner and manifest. Set PAGE_DEPS to the release deps of the counted moonshot library from 252646a to reproduce the frozen library. The wrappers guarded.rs and fused.rs are standalone scorer entries.
