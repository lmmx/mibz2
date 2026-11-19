# Queue retained L ranks as contiguous spans

The combined narrow endpoint, short words and one-edge-ahead writer enqueues a u32 retained L rank for every kept row. A byte bucket is later consumed in stable order while a band writes consecutive packed edges. Another existing schedule scans the capped records a second time and intersects each retained run with that bucket's open bands. The experiment keeps the combined construction and emission while replacing the temporary row queues with per-run spans. Its sources derive from the combined constructor at 80d8409359214b4e5abfd4430213921e82e9cd30. The original generated constructor digest is c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f. The narrow endpoint module is byte-identical to the one generated at that revision.

The packed word stores the first retained L rank in its low 20 bits and length minus one in its high 12. Every legal length fits after splitting spans at 4,096 rows, and every rank fits because a block has at most 900,000 rows. A newly retained run appends the exact consecutive ranks the original row loop enumerated. In merged mode, a bucket joins any fitting contiguous prefix onto its last span and enqueues any remainder. The primary retained inside an eliminated strip follows the original singleton classification. By induction over run arrivals, expanding each bucket's spans gives its former sequence of ranks exactly.

The producer of induced bands and their spelling codes is the combined kernel's original code. On each bucket a cursor stores the next span index and the current retained row and remaining length. Taking the minimum of the remaining span and remaining band consumes exactly the requested stable rank prefix. Consecutive output cells receive the same packed successor values. When codes require the wide representation the separate code array is filled in the original band order. The final edge table, word code table, mark and output loop therefore have their original semantics on arbitrary cycles. The candidate's density gate, column fallback and end-of-stream handling remain those of the combined kernel.

The native study selects four routes in one source. Mode 0 rescans the input, mode 1 enqueues each row, mode 2 stores each run span, and mode 3 joins adjacent spans. The scorer entry is mode 3. Astra B's range queue compresses rotating entrances to internal bucket paths, and the F-coordinate conjugacy removes the retained queues by a second pass and altered scatter. Those algorithms differ from compressing the original retained-row queue in place. The original open comparison tested queued rows and a record rescan. Its record is in docs/journal/2026-11-18-open-return at 9c31461.

## Verified geometry

The forced-kernel property check compared each route with independently stable-sorted marked Ψ on 56,843 marked column/limit cases. It exercises join lengths over 4,096, split bands, capped input records, thick primaries and all primary positions for binary columns through length 9 and ternary columns through length 6. The real ledger also compared the emitted inverse bytes on all 228 density-admitted Wikidata blocks, totaling 205,197,794 rows.

| Across the 228 blocks | Row queues | Run spans | Joined spans |
| --- | ---: | ---: | ---: |
| Queued u32 entries | 25,484,201 | 13,682,519 | 13,589,952 |
| Max total retained bucket capacity (bytes) | 1,563,328 | 840,960 | 840,896 |
| Band/span intersections | — | 13,732,298 | 13,650,293 |
| One-row intersections | — | 10,336,588 | 10,259,864 |

The row queue can zip a flat input slice directly against each destination band. The packed routes unpack a descriptor for every span and test exhaustion on either side of every intersection. The joined route absorbed 315,082 retained rows into an earlier descriptor and saved 92,567 entries relative to raw spans. No joined span reached the encoding cap on this corpus. The bucket capacity counts exclude the outer vector and all other kernel, worker and output buffers.

## Reproduction

With the scorer's checksum-verified corpus, a fresh output directory and Rust 1.99.1, run the driver below. Its release build excludes the ledger counters for timing. The timed region is the complete inverse on already prepared capped records. Scanner, front end, RLE1, CRC and final stream I/O lie outside this stage measurement. The driver preserves the original combined kernel and opend as independent control arms.

```sh
bash research/astra-e/queue-spans/run.sh /tmp/astra-e-queue-stage
```

## Native stage result

The source for this measurement is `300160cf9f1ab38b0a1206eb1800c42bd26b7b7f`. Two independent processes checked all eleven sample blocks in all seven arms with 28 rotated rounds and a warmup. Positions balance exactly per block in each process. The shared VM was busy during the first run, and its paired CPU intervals are wide. On the repeat, thread CPU time relative to the per-row mode measured 1.0478 [1.0292, 1.0667] for raw spans, 1.1386 [1.1053, 1.1729] for joined spans and 1.1888 [1.1659, 1.2120] for the input rescan. The direct repeat comparison with the independent preserved constructor gave 1.0137 [0.9745, 1.0544] for raw spans and 1.1015 [1.0587, 1.1461] for joined spans. Both runs' sums of per-block CPU medians placed raw spans behind row queues and joined spans behind raw spans.

The nearly identical high-water capacities make the unjoined variant the stronger compact-queue alternative on these inputs. Its scorer entry is `raw.rs`. Neither whole-worker peak RSS nor target performance follows from the measured bucket capacity. See `stage/first.csv`, `stage/repeat.csv` and `stage/summary.csv` for the independent wall and CPU measurements and the preserved controls.

## Bulk u16 rank queues

A fourth alternative, mode 4, keeps one low u16 per retained rank. During the initial run scan it snapshots the 256 per-byte retained counts each time the global retained rank reaches a multiple of 65,536, including the part of a straddling run before that boundary. For page j, a bucket's slice between successive snapshots consists precisely of ranks with high part j. Empty pages of a byte are allowed. Inside the slice, each edge is reconstructed with a shared code and page prefix OR-ed with the low u16. The maximum original rank is below 2²⁰, so at most thirteen page boundaries are crossed. This retains the original rank order across all band splits and eliminates the per-run span decoder.

All five routes agreed on 56,851 forced marked cases, including a 458,779-row multiple-page input with empty per-byte pages and an artificial 170,035-row retained run crossing two page boundaries. On the 228 admitted Wikidata blocks the low-u16 path needed 273 global cuts, and 823,841 page/band intersections versus 810,224 bands. Its bucket and page-array capacity peaked at 785,760 B. The per-row u32 control peaked at 1,563,328 B. The input and whole-worker buffers are additional.

The lower-capacity representation cost more CPU on both independent 28-round runs. Relative to row queues the complete-inverse CPU ratios were 1.0799 [1.0242, 1.1386] and 1.0730 [1.0530, 1.0933]. Against the original preserved implementation they were 1.1036 [1.0511, 1.1588] and 1.0561 [1.0202, 1.0933]. The wall ratios against row queues were 1.0055 [0.8725, 1.1588] and 1.0516 [0.9261, 1.1942]. This alternative spends extra work testing page boundaries and widening stored u16 values during band filling. The stage clocks do not isolate the contributions of these two changes.

The scorer entry is `low16.rs`. The new ledger and native timing sources are reproduced from the original harness using `low-harness.patch`. Raw measurements, the build receipt and the two-clock summaries are in `low-stage/`. The frozen v0/v1 scoreboard and i9 comparisons are separate from this native-stage result.

```sh
QUEUE_SUITE=low16 bash research/astra-e/queue-spans/run.sh /tmp/astra-e-u16-stage
```
