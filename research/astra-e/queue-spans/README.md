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
