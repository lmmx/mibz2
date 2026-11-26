# One-word predecessor pages inside F2 dynamic refill

The reference is the first-level eight-lane dynamic scheduler at `87d41eb0e1cdd3b82268ea91ac736d150207188e`. Its page map uses a separate u32 containing-run number and a u64 run-start mask per 64 original rows. This experiment retains the same marked transfer, refill schedule, spelling, merge and second-level inverse. It changes the first-level predecessor index.

For width W, the base is the run containing the page's first row. For any run starting at offset j > 0 in that page, record bit j-1. An offset i therefore lies in run base + popcount(bits & ((1 << i)-1)). A start exactly on a boundary already belongs to the base. This counts every later start through i once, including on partial final pages and pages entirely covered by one run. At W=32, 40 or 44, all cut bits fit below bit 44. The top 20 bits store the base because a bzip2 block has at most 900,000 distinct runs. Lookup reads one u64. The interleaved64 control stores the mask and base in one 16-byte aligned-layout entry, including four padding bytes. Its struct alignment is eight bytes. The candidate scorer entry selects width 44.

The original map takes twelve logical bytes per 64 rows. At 900,000 rows the logical footprints are 168,756 B split64, 225,000 B packed32, 180,000 B packed40, 163,640 B packed44 and 225,008 B interleaved64. These precede vector capacity rounding, other worker arrays and whole-process RSS. The first implementation retains separate passes to write run cuts and page bases. The page count, base loop, shift or constant quotient and cache density all change with W.

## Exact checks and construction price

All 11,264 accepted small marked outputs agreed for each of the four representations. The direct rank test compared each queried index with a separately scanned capped column for 156 synthetic run/page geometries per width, including alternating 900,000-row columns and cap splits. Each layout then agreed at every original row on all 228 real admissions and reconstructed the complete 205,197,794 inverse bytes returned by the independent Segmented kernel. There were 14,427,155 original runs. The index-only allocated high-water marks observed across retained worker reuse were 449,968 B, 359,984 B, 327,248 B and 449,984 B at widths 32, 40, 44 and 64 respectively. The `native/geometry.csv` file records each block.

Two 30-pass, five-arm processes compared the complete inverse on the nine systematically sampled large admissions used in the dynamic scheduler study. Every block occupied all five positions six times per arm, after a warmup. Each output matched Segmented outside the clock. The Linux thread-clock phases measure preparation and primary transfer, tree discovery, merge, and recursive second-level inverse and output. Paired ratios sum the same nine blocks within each pass. The first pinned CPU was much slower during this shared-machine measurement. Both files retain the observations and both clocks.

| Packed or interleaved layout | CPU3 complete CPU ratio | CPU0 complete CPU ratio | CPU0 prep+mark ratio | CPU0 discovery ratio |
| --- | ---: | ---: | ---: | ---: |
| Packed32 | 0.9605 [0.9048,1.0196] | 1.0139 [0.9707,1.0591] | 1.0883 | 1.0031 |
| Packed40 | 0.9963 [0.9372,1.0591] | 1.0238 [0.9508,1.1025] | 1.0608 | 1.0428 |
| Packed44 | 0.9630 [0.9068,1.0226] | 1.0312 [0.9598,1.1078] | 1.1139 | 1.0357 |
| Interleaved64 | 0.9770 [0.9186,1.0392] | 0.9995 [0.9472,1.0546] | 1.0310 | 1.0008 |

The reference was split64 in the same linked executable. The price of packed preparation did not produce a reproducible complete inverse speedup in this series. The native build used the pinned 0.5.6 library preserved from the queue study at `7998c953ab4e0de27508ab76e6c8f1018c6225d0`. Its digest is in `native/manifest.txt`. The initial shared-worktree build is kept separately in `pilot/`. The native data provide inverse-stage measurements on ready run records. They do not establish whole-worker timing, peak RSS or target i9 speed.

```sh
d=research/astra-e/formulations/f2-pages
bash "$d/run.sh" /tmp/astra-e-page-stage
python3 "$d/report.py" /tmp/astra-e-page-stage/{first,repeat}.csv
```

Use PAGE_DEPS pointing to a pinned release/deps directory to select an independently built library. The study used CPUs 3 and 0; alternate CPU IDs are the second and third arguments to run.sh.
