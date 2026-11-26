# Induced return in retained F coordinates

Open-Return at `9c31461` builds one queue per byte. It puts each retained L rank in a queue, then copies packed ranks from queue slices to the residual table. The i9 record 76 attributes the original kernel's advantage to fewer instructions per byte, with a lower IPC and more lower-cache stalls. This experiment asks whether the queues can be removed by changing coordinates without lengthening the residual chain.

Let S send a retained L rank j to its stable-sort rank within the retained letters. Let P take a retained entry k in F order across the open complement to its first exit j in retained L order. The usual residual forward permutation is P S. Use the coordinate S(j) instead. Each interval of the induced endpoint exchange provides a translation P(k+t) = j+t with one reversed spelling W, including the initial thin letter. Write an edge at S(j+t) whose address is k+t and whose label is W. These are exactly the edges of (S P) inverse. Stable sort is bijective, so walking from S(mark) yields the same labels in the same order through the first return to the mark. Short marked cycles repeat to the requested length and other cycles stay disconnected. The existing thick-primary split and fallback handle the same input contract as opend.

To build the table, record each top-order node's first band while attaching the retained letter and expanding its dictionary word. Traverse the same nodes in bottom order to obtain bands ordered by exit j. A second scan of the capped records supplies retained L ranks in that order. For each thin run its S destinations form a consecutive slice of one byte bucket. Intersect this slice with the current band and write increasing packed ranks. The new work is a second capped-record scan, a second pass over bands and short range stores into up to 256 independently advancing destinations. Peak scratch exchanges a u32 per retained row plus the 256 queues' excess capacity for an extra 12 bytes per induced band. The walk makes the same number of dependent edge reads and can have different cache locality in the new coordinate. RSS and wall time require measurement.

The isolated candidate uses opend's original density policy, 16-row cutoff, column fallback and scalar residual writer. It derives endpoint elimination, maximal-run splitting and dictionary emission from `docs/journal/2026-11-18-open-return` at `9c31461`. The shipping dispatch and allocator are untouched.

## Exact coordinate test

```sh
MOONSHOT_CANDIDATE=research/astra-e/open/candidate.rs cargo test --manifest-path bench/moonshot/Cargo.toml exhaustive_conjugacy -- --nocapture
```

All 55,302 triples passed. The oracle constructs inverse LF by stable sorting the L row indices and follows it from the mark for exactly n steps. The test enumerates every binary column of lengths 1 through 10, all marks and thresholds 1, 2 and 3 to force thick strips on small inputs. The raw test log is in `exhaustive.txt`. The official scorer covers the full stream, RLE1 and CRC.

## Wikidata arrow audit and successor-table locality

`bash research/astra-e/open/audit.sh` verifies the corpus SHA-256 and scans every 24th of its 233 large blocks, including frozen stream ordinal 1. On all 10 it compares the complete pre-RLE1 sequence with Plain and checks each of the 1,188,146 retained inverse edges and every byte of its dictionary spelling against independent dense forward LF. The paths account for all 8,999,915 input rows. One block is outside opend admission and is inspected forcibly. See `wikidata.csv` and `audit-log.txt`.

On the nine admitted blocks the extra top bands' logical sizes total 9.85% of the old queues' row payloads. 76.49% of the 531,443 band/run intersections require writing one row. These counts exclude capacity rounding, other buffers and competing construction costs. For those blocks, trace the same 956,546 marked successor reads in old and conjugate coordinates. With 64-byte lines the F/L ratios of cold fully associative LRU misses are 1.003 at 32 KiB, 1.010 at 256 KiB and 1.000 at 2 MiB. These count the u32 successor array alone. They exclude the dictionary, construction, allocator alignment, other streams and other workers. The traces give little support to a successor-locality explanation for a large performance difference. The extra capped-record scan and many one-row stores remain a construction cost to address if paired timings warrant further work.
