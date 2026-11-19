# Direct matched comparison with run-front counts

The dynamic scheduler is pinned at 87d41eb, its page-major constructor at 84804dc, and the front-counts constructor at 252646a. A single binary builds against the moonshot-enabled library from 252646a, which exposes the existing run-front byte histogram. Five independent candidate states rotate through historical dynamic split64, sequential split64, sequential packed44, front-counted split64, and a fused counted split64. The three modified split64 implementations use the original two-buffer directory and query formula. Every input and native inverse phase shares one clock and one baseline in the matched run.

With known byte frequencies, the counted constructor can assign the final LF address on each coalesced run arrival and insert its start bit immediately. It then scans every page mask to count the starts, derive the base and clear its boundary bit. The fused constructor removes that remaining directory pass. It tracks one open mask and base; a run arriving in a subsequent page publishes the finished pair. Every wholly skipped page contains just the previous run. At the new page's first row the containing run is the arriving run if its offset is zero and the previous run otherwise. At the end it emits the open page and fills any trailing pages with the final run. This gives the original masks, bases and LF values on every legal capped stream. Retained directory storage still consists of one u32 and one u64 per page. Removing the final page sweep adds a page-transition comparison on each new run and separate vector writes at publication.

The native checks compare all five outputs on the 228 real admissions and include forced small arbitrary marks, direct byte-for-byte comparison of the counted runs and directories against historical preparation, and skipped/trailing pages inside capped runs much longer than 64 rows. Two 40-pass processes independently rotate all five arms on the nine systematic real blocks. Each arm occupies each position eight times per block. Per-pass samples sum those same nine blocks. The four phases include setup/mark transfer, discovery, merge and full recursive output. Wall time encloses that complete instrumented inverse call.

To reproduce with the pinned library available as PAGE_DEPS, use the following commands. The raw files and compiler receipt are beside this note.

```sh
d=research/astra-e/formulations/f2-pages/counted
bash "$d/run.sh" /tmp/astra-e-counted-pages
python3 "$d/report.py" /tmp/astra-e-counted-pages/{first,repeat}.csv
```

| Charged method relative to the front-counted split64 | CPU3 setup+mark | CPU0 setup+mark | CPU3 complete inverse | CPU0 complete inverse |
| --- | ---: | ---: | ---: | ---: |
| Page-major split64 | 1.1426 [1.0100,1.2925] | 1.5792 [1.3017,1.9158] | 1.0247 [0.9416,1.1152] | 1.0397 [0.9563,1.1303] |
| Fused counted stream | 1.1118 [0.9964,1.2405] | 1.2905 [1.0704,1.5559] | 0.9712 [0.8851,1.0655] | 0.9973 [0.9339,1.0650] |

The original counted candidate recorded the lowest summed per-block median setup on both pins, 3.399 and 1.728 ms respectively, compared with 4.107 and 2.440 ms for page-major, and 4.076 and 2.199 ms for the joint. Relative to the unmodified dynamic refill its complete CPU ratios were 1.0270 [0.9544,1.1051] and 0.9164 [0.8698,0.9655]. Joint/dynamic was 0.9973 [0.9078,1.0957] and 0.9139 [0.8592,0.9721]. The direct joint/counted measurements did not resolve an incremental overall speedup. The extra run-arrival branching and per-page publication in the joint increased the measured stage they were intended to accelerate. Retain the original 252646a implementation as the front-counts lead.

All five states returned the expected exact 205,197,794 bytes. The pinned host library was built from 252646a so both counted arms use the actual histogram returned by Runs::byte_counts(). The front end lies outside these native clocks and is common to all arms. Its existing production histogram accumulation is part of the end-to-end decoder. The first process on CPU3 ran at a markedly different clock cost from the CPU0 repeat. Whole-worker and target confirmation remain separate.
