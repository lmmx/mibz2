# Predict the next run from the last transition actually observed

The 796c2bf learned eight-lane method certifies that an entire source L run translates inside a target run. This experiment allows the target to contain only the actual next row and explicitly rejects an invalid prediction. For each source run i keep the most recent observed target run. At retirement its lane discards the previous source. At an ordinary response it carries the predicted target for that run into the next wave. If the target's half-open row interval contains the new query row, its ordinal is exact. Otherwise a true directory rank gives the target and updates the source. The range check works for arbitrary merged L runs and LF cycles. It needs both boundaries unless additional stable information excludes one. Simultaneous lane misses update the same source in ascending lane order. A guess carried into the next wave is frozen after all those updates.

A separate observer charged all transitions and checked every prediction against direct rank while reconstructing all canonical admitted inverses. Across 228 blocks and 205,197,794 inverse bytes it saw 19,750,839 rank queries. First steps of fresh jobs take 987,682 original ranks. Of the 18,763,157 continuing steps, 10,778,280 lacked a prediction, 6,169,472 were right (31.24% of all queries), and 1,815,405 were wrong. Exactly 12,573,500 changed-table writes were needed. Only 58,271 waves could skip all eight ranks. Of the hits, 5,441,071 already place the whole source image inside the predicted run. This form of prediction captures 728,401 further hits on partial images, 3.69% of all queries. Half the wrong guesses missed by exactly one run (922,017). Checking an adjacent guess still costs extra work and was counted separately.

A full u32 side table adds four bytes per original run. The largest observed table would occupy 514,928 bytes before capacity rounding. A compact implementation can encode a 16-bit prediction in the original eight-byte record using the upper twelve bits of the modulo-2^20 displacement and four unused endpoint bits. A row crossing the first 65,535 runs selects the upper range using a block threshold; the largest observed block has 128,732 runs. The implementation must still validate every prediction, provide a fallback on other legal blocks, account for packing/masking, and charge failed guesses and changed writes.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/successor
bash "$d/census.sh" /tmp/astra-e-last-successor
```

The historical whole-image eight-lane classifier removed 5,404,952 ranks and cost 1.2266 [1.1172,1.3468] in complete inverse CPU against its own baseline. Its 12-byte records and older scheduler prevent transferring that timing result to the present eight-byte preloaded kernel. [Original whole-image account](../../../../f2d/frontier/README.md)

## Paid implementation

The side-table candidate kept the original eight-byte record and reserved one u32 target per run. The embedded candidate stored the cached ordinal modulo 65,535 plus one in the existing record, with zero meaning unobserved. A comparison with the row immediately after run 65,534 selects the lower or upper ordinal range. For a block with more than 131,070 runs, a guessed ordinal may fail in later ranges. Testing the candidate run's two row boundaries always rejects such an alias, so an ordinary rank recovers the answer. The layout control packed an identical modulo-2^20 displacement without learning. Queries retained the previous full eight-slot response and ordinary fallback paths. Actual eight-byte records always cover every legal original row.

For every block the side candidate reproduced the observer's query, hit, and write counts. Totals for the embedded format agreed as well. All arms produced the canonical 205,197,794 inverse bytes against Segmented. An additional property test checked all 65,536 possible codes against positive and negative LF translations, both end tests, and ordinary and delimiter bytes. A 131,101-run alternating column exercised indices beyond both encoding boundaries and all nine chosen marks agreed with an independent stable-sort inverse. The side vector reached 818,944 retained bytes after normal growth. The embedded arm needed zero side-table bytes and kept the original payload capacity.

| Native CPU ratio vs unchanged preload | CPU 4 | CPU 2 |
|---|---|---|
| Packed layout only, complete | 1.0046 [0.9611,1.0501] | 1.0262 [0.9983,1.0549] |
| Side table, discovery | 2.1664 [2.0158,2.3282] | 2.0524 [1.9879,2.1190] |
| Embedded target, discovery | 2.3253 [2.1394,2.5274] | 2.3266 [2.2466,2.4094] |
| Side table, complete | 1.4313 [1.3416,1.5270] | 1.4250 [1.3917,1.4590] |
| Embedded target, complete | 1.5062 [1.4091,1.6100] | 1.5436 [1.5118,1.5761] |

The direct comparisons use 64 balanced four-arm laps in each process and check every output. Eliminating 31.24% of rank requests coexists with roughly doubled or higher discovery costs. Recognizing a cached target requires dependent boundary reads before a fallback rank, unchanged predictions still require checks, and advancing each lane updates its last-source state. The receipt quantifies total costs without attributing them to a single mechanism. Both variants are retired for worker-timing purposes. The preserved 6758831 preload remains the independent-CI representative for this family.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/successor
bash "$d/run.sh" /tmp/astra-e-last-successor-price
bash "$d/edge.sh" /tmp/astra-e-last-successor-encoding
python3 "$d/report.py" /tmp/astra-e-last-successor-price/{first,repeat}.csv
```

Set PAGE_DEPS to a release moonshot library from 252646a to reproduce the manifest. Corpus SHA-256 is checked by the runner. packed.rs, side.rs and embedded.rs are experimental scorer entries.
