# Native construction on #714

Direct manual assignment from Louis Maddox. Base `26a5f6aed23a430c7447616782c0a2b6db946e2f`. Wall reference `8ff1187af6a920f7d57fd338680db3a5633a3ac5`.

Against master, the six-block one-thread kernel sum was **0.872 [0.869, 0.875]**. The four-worker six-block pass was **0.880 [0.877, 0.883]**, and the two-worker pass was **0.882 [0.879, 0.886]**. Retained Open scratch after the six blocks fell from 1,200,839 to 968,484 bytes per worker (**−19.3%**). After all 431 blocks it fell from 1,377,699 to 1,128,960 bytes (**−18.1%**).

The #714 base measured 0.973 [0.970, 0.977] of master on the same six-block one-thread sum and 0.977 [0.973, 0.982] on the four-worker pass. The assembled stack ran at 0.835 [0.827, 0.842] on the dense five at one thread and 0.838 [0.834, 0.843] on their four-worker pass. Intervals throughout are paired 95% intervals.

Kitchen Sink reported 0.8552× one-thread p10, 0.8499× two-worker p50 and −19.28% scratch on its six-block panel. Three of the six native one-thread block ratios fell below 0.90, compared with all six in cycle 29. Its 48 KiB reserve saved 36 KiB on that panel here. After the whole corpus the reserve added 12 KiB against the immediately preceding commit.

## Per-commit contribution

| Idea | Six 1T / previous [95% CI] | Six 4W / previous [95% CI] | Six 1T / master | Six 4W / master | Δ scratch after six / 431 (bytes) |
| --- | --- | --- | --- | --- | --- |
| L4 bottom coordinates | 0.998 [0.993, 1.002] | 0.998 [0.995, 1.001] | 0.972 [0.968, 0.977] | 0.973 [0.970, 0.976] | +0 / +0 |
| Thin + thick row totals | 0.992 [0.988, 0.997] | 0.993 [0.990, 0.995] | 0.962 [0.957, 0.966] | 0.967 [0.964, 0.970] | +0 / +0 |
| Strips in edge prefix | 1.006 [1.002, 1.011] | 1.004 [1.001, 1.007] | 0.967 [0.964, 0.969] | 0.968 [0.965, 0.971] | -131,072 / -131,072 |
| 64-bit bands | 1.004 [0.998, 1.010] | 1.004 [1.001, 1.007] | 0.972 [0.968, 0.976] | 0.976 [0.973, 0.979] | -32,768 / -32,768 |
| Short rule payloads | 0.970 [0.968, 0.973] | 0.964 [0.961, 0.967] | 0.940 [0.936, 0.943] | 0.939 [0.937, 0.940] | +20,416 / -45,120 |
| Long ranges in inline slots | 1.001 [0.995, 1.006] | 1.003 [1.000, 1.007] | 0.938 [0.935, 0.941] | 0.942 [0.939, 0.945] | -16,384 / -16,384 |
| Interned dictionary | 0.989 [0.984, 0.993] | 0.986 [0.983, 0.990] | 0.926 [0.922, 0.930] | 0.931 [0.928, 0.934] | -35,683 / -35,683 |
| 48 KiB spelling reserve | 1.004 [1.000, 1.009] | 1.001 [0.997, 1.004] | 0.924 [0.922, 0.927] | 0.931 [0.929, 0.934] | -36,864 / +12,288 |
| Two-ended emission | 0.941 [0.937, 0.945] | 0.943 [0.940, 0.946] | 0.872 [0.869, 0.875] | 0.880 [0.877, 0.883] | +0 / +0 |

## Authenticated and dense blocks

Times are µs. S is the six authenticated blocks and D is the five dense blocks.

| Block | Master 1T µs | Head 1T µs | 1T / master [95% CI] | Master 4W µs | Head 4W µs | 4W / master [95% CI] |
| --- | --- | --- | --- | --- | --- | --- |
| block-0.L | 781.2 | 714.0 | 0.911 [0.907, 0.916] | 792.9 | 731.7 | 0.923 [0.921, 0.925] |
| block-2.L | 963.7 | 869.1 | 0.901 [0.897, 0.904] | 974.8 | 884.4 | 0.908 [0.906, 0.911] |
| block-85.L | 1,009.2 | 917.7 | 0.905 [0.899, 0.912] | 1,022.4 | 933.0 | 0.914 [0.910, 0.919] |
| block-143.L | 1,828.2 | 1,532.2 | 0.835 [0.831, 0.839] | 1,846.5 | 1,557.4 | 0.846 [0.843, 0.850] |
| block-210.L | 1,606.2 | 1,378.7 | 0.857 [0.853, 0.862] | 1,618.0 | 1,400.5 | 0.867 [0.864, 0.871] |
| block-207.L | 1,167.2 | 1,014.1 | 0.869 [0.865, 0.874] | 1,185.8 | 1,036.9 | 0.876 [0.873, 0.880] |
| S pass | — | — | — | 7,475.2 | 6,571.0 | 0.880 [0.877, 0.883] |
| dense-41.L | 1,821.4 | 1,527.3 | 0.842 [0.835, 0.849] | 1,845.0 | 1,563.3 | 0.847 [0.842, 0.853] |
| dense-56.L | 1,795.3 | 1,499.0 | 0.836 [0.830, 0.842] | 1,808.2 | 1,534.4 | 0.844 [0.838, 0.850] |
| dense-165.L | 1,813.7 | 1,509.0 | 0.834 [0.826, 0.841] | 1,830.6 | 1,535.6 | 0.837 [0.834, 0.840] |
| dense-270.L | 1,519.5 | 1,223.8 | 0.803 [0.791, 0.815] | 1,555.0 | 1,256.1 | 0.807 [0.801, 0.812] |
| dense-278.L | 1,532.7 | 1,319.9 | 0.857 [0.852, 0.862] | 1,556.8 | 1,343.0 | 0.862 [0.858, 0.865] |
| D pass | — | — | — | 8,681.6 | 7,274.9 | 0.838 [0.834, 0.843] |

## Whole Wikidata

| Workers | Master wall ms | Head wall ms | A/A [95% CI] | Head / master [95% CI] | Head / previous [95% CI] |
| --- | --- | --- | --- | --- | --- |
| 1 | 339.69 | 311.07 | 1.024 [0.983, 1.067] | 0.925 [0.882, 0.971] | 0.988 [0.939, 1.040] |
| 4 | 142.25 | 129.58 | 1.052 [0.964, 1.148] | 0.912 [0.857, 0.970] | 0.990 [0.927, 1.057] |

Phase times below sum Open's measured time across the stream. Bottom is nested within S3a.

| Phase | 1W master / head (ms) | 4W master / head (summed ms) |
| --- | --- | --- |
| S1 first pass | 42.83 / 41.79 | 51.98 / 49.31 |
| S2a layout and nodes | 8.25 / 8.99 | 12.07 / 10.84 |
| S2b elimination | 26.73 / 27.31 | 29.67 / 30.54 |
| S3a coordinates bands and words | 28.93 / 17.50 | 32.26 / 20.47 |
| S4 wire | 15.53 / 26.83 | 17.10 / 29.87 |
| S3b separate inline table | 2.41 / 0.00 | 2.68 / 0.00 |
| S5 emit | 106.45 / 84.35 | 116.68 / 90.43 |
| S3a bottom only | 1.72 / 0.91 | 1.80 / 0.95 |

## Stage attribution

| Addition | Phase | Six 1T µs previous → candidate | Phase ratio [95% CI] |
| --- | --- | --- | --- |
| L4 bottom coordinates | S3a bottom only | 51.4 → 27.5 | 0.536 [0.535, 0.536] |
| Thin + thick row totals | S1 first pass | 1,267.2 → 1,251.0 | 0.991 [0.987, 0.995] |
| Strips in edge prefix | S2a layout and nodes | 209.0 → 222.3 | 1.065 [1.061, 1.069] |
| 64-bit bands | S4 wire | 470.4 → 485.6 | 1.039 [1.034, 1.044] |
| Short rule payloads | S2b elimination | 621.5 → 749.9 | 1.213 [1.199, 1.227] |
| Short rule payloads | S3a coordinates bands and words | 875.8 → 512.1 | 0.583 [0.580, 0.587] |
| Long ranges in inline slots | S5 emit | 3,728.1 → 3,674.3 | 0.986 [0.978, 0.994] |
| Interned dictionary | S3a coordinates bands and words | 513.7 → 519.8 | 1.010 [1.005, 1.015] |
| Interned dictionary | S5 emit | 3,655.2 → 3,628.8 | 0.992 [0.983, 1.001] |
| 48 KiB spelling reserve | S3a coordinates bands and words | 513.1 → 523.0 | 1.015 [1.013, 1.018] |
| Two-ended emission | S4 wire | 501.1 → 860.7 | 1.721 [1.713, 1.729] |
| Two-ended emission | S5 emit | 3,599.0 → 2,859.2 | 0.791 [0.784, 0.798] |
| Packed queue cursor | S1 first pass | 1,246.0 → 1,264.4 | 1.013 [1.008, 1.018] |

The following cells give master / head in µs.

### One thread

| Block | S1 | S2a | S2b | S3a | S4 | S3b | S5 | Bottom (inside S3a) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| block-0.L | 137.2 / 134.9 | 35.6 / 38.0 | 106.7 / 111.4 | 121.0 / 70.0 | 55.2 / 89.8 | 8.2 / 0.0 | 313.1 / 271.6 | 6.5 / 3.3 |
| block-2.L | 149.4 / 147.1 | 39.6 / 42.4 | 132.8 / 137.0 | 145.0 / 86.6 | 71.5 / 113.7 | 12.3 / 0.0 | 415.8 / 348.2 | 8.8 / 4.6 |
| block-85.L | 192.9 / 186.7 | 14.7 / 16.0 | 66.2 / 66.3 | 72.4 / 48.7 | 53.5 / 134.6 | 7.5 / 0.0 | 617.1 / 463.8 | 4.6 / 2.5 |
| block-143.L | 277.4 / 269.2 | 50.3 / 53.4 | 204.9 / 208.4 | 227.2 / 135.1 | 118.1 / 198.5 | 21.6 / 0.0 | 935.6 / 675.1 | 14.3 / 7.8 |
| block-210.L | 315.8 / 296.5 | 27.8 / 29.5 | 115.7 / 116.6 | 125.8 / 80.2 | 83.6 / 185.3 | 13.0 / 0.0 | 935.4 / 673.2 | 8.5 / 4.6 |
| block-207.L | 218.0 / 213.0 | 33.9 / 36.4 | 134.5 / 135.9 | 144.4 / 87.6 | 79.7 / 136.3 | 13.3 / 0.0 | 556.5 / 414.1 | 9.2 / 5.0 |
| dense-41.L | 317.3 / 306.0 | 24.5 / 25.7 | 116.0 / 114.5 | 128.6 / 82.1 | 92.9 / 213.3 | 15.4 / 0.0 | 1124.2 / 774.7 | 9.3 / 4.8 |
| dense-56.L | 323.7 / 308.1 | 21.6 / 23.0 | 103.5 / 102.5 | 116.7 / 75.0 | 83.3 / 206.5 | 14.0 / 0.0 | 1127.1 / 770.6 | 7.9 / 4.1 |
| dense-165.L | 325.8 / 312.7 | 26.0 / 27.4 | 121.7 / 122.1 | 140.7 / 86.7 | 91.5 / 204.1 | 15.8 / 0.0 | 1086.8 / 740.8 | 9.3 / 4.8 |
| dense-270.L | 396.0 / 371.4 | 5.6 / 6.2 | 25.9 / 23.6 | 26.0 / 23.1 | 28.4 / 152.5 | 2.9 / 0.0 | 1055.6 / 634.8 | 1.9 / 1.0 |
| dense-278.L | 315.6 / 296.9 | 20.3 / 21.7 | 96.5 / 96.6 | 108.3 / 69.4 | 74.2 / 178.5 | 11.7 / 0.0 | 921.7 / 654.7 | 7.1 / 3.7 |

### Four concurrent workers

| Block | S1 | S2a | S2b | S3a | S4 | S3b | S5 | Bottom (inside S3a) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| block-0.L | 139.2 / 136.8 | 36.0 / 38.1 | 107.6 / 112.1 | 121.3 / 71.2 | 56.1 / 90.8 | 8.9 / 0.0 | 317.5 / 275.7 | 6.5 / 3.4 |
| block-2.L | 150.1 / 148.1 | 40.0 / 42.6 | 133.2 / 137.4 | 146.9 / 87.3 | 72.6 / 115.0 | 12.8 / 0.0 | 419.5 / 356.9 | 8.8 / 4.6 |
| block-85.L | 195.9 / 188.3 | 15.0 / 16.0 | 66.6 / 66.5 | 73.0 / 49.3 | 54.7 / 135.6 | 7.7 / 0.0 | 612.5 / 483.7 | 4.7 / 2.5 |
| block-143.L | 280.1 / 270.4 | 51.8 / 53.6 | 204.5 / 209.6 | 225.4 / 136.1 | 118.7 / 202.2 | 22.1 / 0.0 | 944.9 / 688.3 | 14.3 / 7.8 |
| block-210.L | 318.9 / 305.6 | 27.8 / 29.6 | 115.8 / 117.3 | 125.9 / 81.1 | 85.1 / 188.8 | 13.1 / 0.0 | 949.4 / 693.1 | 8.5 / 4.6 |
| block-207.L | 219.9 / 213.1 | 33.9 / 36.3 | 133.9 / 136.2 | 143.7 / 88.2 | 80.3 / 136.5 | 13.3 / 0.0 | 559.3 / 425.8 | 9.2 / 5.0 |
| dense-41.L | 326.7 / 313.9 | 24.4 / 25.8 | 116.2 / 115.1 | 127.6 / 85.7 | 95.4 / 219.7 | 15.4 / 0.0 | 1150.3 / 815.2 | 9.3 / 4.8 |
| dense-56.L | 330.3 / 317.3 | 21.7 / 23.3 | 104.0 / 103.2 | 115.6 / 79.9 | 85.7 / 209.1 | 14.0 / 0.0 | 1154.1 / 808.6 | 7.9 / 4.1 |
| dense-165.L | 329.1 / 319.9 | 26.0 / 27.8 | 122.0 / 122.7 | 140.8 / 90.5 | 93.4 / 209.1 | 15.9 / 0.0 | 1110.8 / 771.1 | 9.3 / 4.8 |
| dense-270.L | 400.5 / 381.8 | 5.8 / 6.4 | 26.4 / 24.1 | 27.4 / 25.4 | 33.1 / 155.8 | 3.0 / 0.0 | 1064.4 / 652.0 | 1.9 / 1.0 |
| dense-278.L | 318.4 / 301.8 | 20.4 / 22.0 | 96.9 / 97.1 | 108.4 / 71.8 | 75.5 / 180.0 | 11.7 / 0.0 | 927.4 / 665.3 | 7.1 / 3.7 |

## Alternatives

The C29 packed cursor cost 1.016 [1.013, 1.019] on the six-block sum and 1.015 [1.011, 1.019] on the four-worker pass relative to two arrays. Retained bytes were equal. The cursor reduced temporary stack use by 1 KiB. The PR retains the two arrays and tests the rollover in both representations through the measured snapshots.

The two geometry implementations were tested with the same short rules, interning, 48 KiB reserve and XOR walk. Both geometry arms shared the trial's packed cursor and four-way wiring, so these intervals isolate geometry in that configuration. Wormhole L5 uses a single `Vec<[u32; 3]>` for strips and bands. Neon uses an edge-arena prefix for strips and image order, then an eight-byte band with two absolute 20-bit coordinates and a dictionary code. L5 / Neon was 0.998 [0.996, 1.001] on the six-block sum and 0.995 [0.992, 0.998] on the four-worker pass. On the five dense blocks the corresponding ratios were 0.995 [0.989, 1.002] and 0.993 [0.986, 1.001]. The Neon layout retained 32,768 fewer bytes after each full pass over the six, the dense five and the whole corpus. L5's four-worker six-block pass was slightly faster. The stacked port takes the 32 KiB memory saving because the dense and whole-stream comparisons include parity. Wormhole's handoff reported a 23–125 KiB working-set reduction for L5 against its own earlier layout. That range has a different denominator from the two assembled variants here. The measured 1T and 4W whole-stream intervals for L5 / Neon were 0.955 [0.912, 1.001] and 0.995 [0.923, 1.072].

Cycle 24's four-way epoch-filtered XOR wiring ran at 1.006 [1.002, 1.010] of ordinary reversible wiring on the six-block sum and 1.013 [1.010, 1.017] on the four-worker pass. Both implementations passed the 431-block parity test. Only the ordinary wiring ships.

Cycle 28 compared left-end and two-ended Rauzy–Veech elimination with BiOp48 and rejected them. Its reported two-ended construction took 174.3 µs more on six blocks. BiOp48 kept right-end elimination. Wormhole L4 instead reads both ends of the completed side-one list to assign bottom coordinates. It preserves #714's graded handles and tail-pop logic.

Cycles 22, 23, 26 and 27 also rejected separate XOR scatter, alternate queue chunk sizes, tiled wiring and length-in-slot Inline15. The cycle 29 champion used the original maximal-run scanner and the 16-rank iota store.

## Measurement and memory accounting

AMD EPYC 9V45, Linux x86-64, four-CPU cgroup quota, rustc 1.99.1, Cargo release with fat LTO and one codegen unit. CPU 2 ran the single-thread kernels. CPUs 0–3 ran the four-worker passes and both stream configurations. The optional two-concurrent-worker pass used CPUs 0–1.

All candidate and reference kernels came from their respective revisions. One binary contains master's unmodified kernel for wall measurements, its A/A copy, #714, each candidate, and separate instrumented copies. Each candidate ran in direct pairs with both master and its immediate predecessor. Every reported timing has 12 fresh-process rounds. Arms rotate forward and reverse across rounds. Within each process the block decodes alternate ABBA. Ratios use the geometric mean of paired candidate/reference elapsed-time ratios, with Student's 95% interval on the logs (11 degrees of freedom). Absolute times are medians across rounds.

The one-thread kernel measurement takes p10 of 41 decodes per arm per block after warming the kernel. Four workers share a barrier and start at staggered block offsets. Each has its own warmed kernel and output. Four warmups precede 30 measured passes per arm. The four-worker pass time is a median worker's elapsed time to decode every block in the specified set. The sum at one thread adds the individual block p10s within each round.

The six-block A/A control measured 1.005 [1.002, 1.008] at one thread and 1.001 [0.998, 1.005] at four workers. The one-thread A/A interval excludes parity by a few tenths of a percent. L4's complete-kernel interval against #714 includes parity, while its own coordinate phase falls from 6.4 to 3.3 µs on block 0. The stage table reports the six-block sum.

The single-block measurements use #714's `Kernel::run::<true,true>` on prebuilt records. This measures the admitted kernel body. Whole-stream measurements use `BzReader::parallel_workers_with::<OpenCells>`, two slots per worker, and each block's actual admission or fallback. They include decoder creation and draining 209,700,196 bytes to a sink. A/B or B/A order alternates by round. The five dense block IDs are corpus indices 41, 56, 165, 270 and 278. The six manifest names use the large-block index. Each stage sample uses 15 decodes per arm after four warmups. Four-worker stage samples divide summed worker time by four. Whole-stream stages sum time spent in Open across workers. They exclude upstream decoding and time spent in fallback.

S1 is pass one. S2a covers initial layout and node lists, S2b elimination, S3a bottom coordinates, bands and dictionary creation, S4 wiring, S3b the separate inline table pass on master, and S5 emission. The bottom clock nests inside S3a. A zero in S3b on #714 and the candidates records the removed phase.

The block scratch rows sum the capacities of the kernel's retained vectors, including spill chunks and the endpoint stack. They exclude the caller's output buffer. Each block adds to one worker's running high-water sequence. For whole-corpus scratch the heap-tracing binary ran every block in order on one reused OpenCells. It separately recorded the Open kernel's capacities and live heap including the fallback. After all 431 blocks the latter was 1,382,471 bytes for master and 1,133,732 for the selected stack.

In a separate run of the production four-worker reader, the following were each worker's retained Open capacities after the whole stream, sorted within each run. The scheduler assigns each block to the next available worker, so adjacent rows can have different work distributions.

| Kernel | Smallest worker (bytes) | Second | Third | Largest |
| --- | --- | --- | --- | --- |
| master | 1,236,176 | 1,299,711 | 1,303,115 | 1,360,189 |
| #714 | 1,255,147 | 1,263,596 | 1,303,952 | 1,305,899 |
| L4 bottom coordinates | 1,234,663 | 1,285,163 | 1,288,440 | 1,312,163 |
| Thin + thick row totals | 1,219,627 | 1,300,199 | 1,302,762 | 1,305,899 |
| Strips in edge prefix | 1,106,507 | 1,170,152 | 1,170,262 | 1,194,524 |
| 64-bit bands | 983,206 | 1,070,823 | 1,143,648 | 1,207,595 |
| Short rule payloads | 1,003,760 | 1,046,187 | 1,151,280 | 1,162,475 |
| Long ranges in inline slots | 1,072,628 | 1,073,096 | 1,139,903 | 1,152,355 |
| Interned dictionary | 1,005,348 | 1,090,416 | 1,110,408 | 1,116,672 |
| 48 KiB spelling reserve | 977,968 | 1,085,068 | 1,122,696 | 1,128,960 |
| Two-ended emission | 1,054,376 | 1,085,068 | 1,122,696 | 1,128,960 |
| Packed queue cursor | 973,244 | 1,057,648 | 1,122,696 | 1,128,960 |

## Exactness

The six last-column files matched the byte counts and SHA-256 digests in manifest.json at `origin/data/admitted-bwt` commit `41c03a5b22d630adb171231f5225da348b8055af`. The harness uses the primary index from each manifest entry. The 12 corpus windows rebuilt to 11,854,933 compressed bytes, SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. Each window check compared the window with libbzip2 and lbzip2.

A separate byte comparison exercised every one of the 431 run- or column-route blocks against master through each candidate's OpenCells. The six authenticated columns and the five dense columns also matched the independent plain inverse. The production `examples/bzcat` at 1, 4, 8 and 10 threads and `bzip2 -dc` all emitted 209,700,196 bytes with SHA-256 `3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`.

`taskset -c 0-3 cargo test --release` passed 261 tests and reported three ignored. `cargo clippy --all-targets -- -D warnings` passed. The full moonshot gate with `Candidate = mibz2::bwt::OpenCells` passed all 338 reported cases including Wikidata and the 600-mutant batch. Each hand mutation was compiled independently against its commit, after the unmodified targeted test passed.

| Idea | Replacement in the trial | Failing test |
| --- | --- | --- |
| L4 bottom coordinates | `hi -= self.width(last);` → `hi -= self.width(first);` | `bottom_coordinates_at_odd_and_even_meetings` |
| L4 bottom coordinates | `self.set_bottom(last, hi);` → `self.set_bottom(last, lo);` | `bottom_coordinates_at_odd_and_even_meetings` |
| Thin + thick row totals | `to: f.thin[a] + f.thick_rows[a]` → `to: f.thick_rows[a]` | `thin_and_thick_sum_to_each_symbols_prefix` |
| Thin + thick row totals | `f.thick_rows[a] += w` → `f.thick_rows[a] += 1` | `thin_and_thick_sum_to_each_symbols_prefix` |
| Packed queue cursor | `const FILL_MASK: u32 = BYTES as u32 - 1` → `const FILL_MASK: u32 = CHUNK as u32 - 1` | `full_chunk_preserves_tail_and_overflow_link` |
| Packed queue cursor | `let at = pos as usize + f;` → `let at = pos as usize;` | `full_chunk_preserves_tail_and_overflow_link` |
| Strips in edge prefix | `(w << 8) \| u32::from(c)` → `(w << 7) \| u32::from(c)` | `strip_prefix_stops_at_dense_cap` |
| Strips in edge prefix | `self.ns += 1;` → `self.ns = 1;` | `strip_prefix_stops_at_dense_cap` |
| 64-bit bands | `u64::from(dst) << 20` → `u64::from(dst) << 19` | `maximum_band_fields_decode_independently` |
| 64-bit bands | `self.0 >> 40` → `self.0 >> 41` | `maximum_band_fields_decode_independently` |
| Short rule payloads | `8 * left_len` → `7 * left_len` | `short_rules_keep_all_fifteen_high_bytes` |
| Short rule payloads | `0x00ff_ffff` → `0x0000_ffff` | `short_rules_keep_all_fifteen_high_bytes` |
| Long ranges in inline slots | `slot[..4].copy_from_slice(&start` → `slot[..4].copy_from_slice(&end` | `long_inline_slot_keeps_both_offsets` |
| Long ranges in inline slots | `slot[4..8].copy_from_slice(&end` → `slot[4..8].copy_from_slice(&start` | `long_inline_slot_keeps_both_offsets` |
| Interned dictionary | `self.pairs[slot] == key` → `self.pairs[slot] != 0` | `dictionary_collisions_compare_complete_spellings` |
| Interned dictionary | `self.inline16[old] == entry` → `true` | `dictionary_collisions_compare_complete_spellings` |
| 48 KiB spelling reserve | `n >= 65_536` → `n > 65_536` | `full_block_reserves_spelling_once` |
| 48 KiB spelling reserve | `48 * 1024` → `47 * 1024` | `full_block_reserves_spelling_once` |
| Two-ended emission | `front == next` → `front != next` | `two_ended_emission_meets_on_odd_even_and_singleton_cycles` |
| Two-ended emission | `out.copy_within(hi..n, lo);` → `out.copy_within(hi..n, 0);` | `two_ended_emission_meets_on_odd_even_and_singleton_cycles` |
| Wormhole L5 (alternative) | `(w << 8) \| u32::from(c)` → `(w << 7) \| u32::from(c)` | `capped_runs_and_shifted_marks` |
| Wormhole L5 (alternative) | `dst.wrapping_sub(k), node_word` → `k.wrapping_sub(dst), node_word` | `capped_runs_and_shifted_marks` |
| Four-way XOR (alternative) | `edges[u0 + 2] ^= pack \| v2` → `edges[u0 + 2] ^= pack \| v1` | `eight_nonmonotone_ranks_in_an_unmarked_epoch` |
| Four-way XOR (alternative) | `edges[v3 as usize] ^= (u0 + 3)` → `edges[v3 as usize] ^= (u0 + 2)` | `eight_nonmonotone_ranks_in_an_unmarked_epoch` |

## Sources and evidence

[Kitchen Sink cycle 20, Operad](https://github.com/lmmx/mibz2/blob/research/carbon-kitchen-sink/docs/journal/2026-11-21-carbon-kitchen-sink-cycle20.md) · [Cycle 28, BiOp48 and BiRV](https://github.com/lmmx/mibz2/blob/research/carbon-kitchen-sink/docs/journal/2026-11-21-carbon-kitchen-sink-cycle28.md) · [Cycle 29, All and Pos](https://github.com/lmmx/mibz2/blob/research/carbon-kitchen-sink/docs/journal/2026-11-21-carbon-kitchen-sink-cycle29.md) · [WormholePreEmitCells handoff](https://github.com/lmmx/mibz2/blob/research/carbon-wormhole/docs/journal/2026-11-21-wormhole-cycle20-whole-kernel-bounds-and-handoff.md) · [Native harness and base PR](https://github.com/lmmx/mibz2/pull/714)

The complete time table includes per-block, per-stage and per-stream pairs against master and the previous commit, plus the A/A control. [Paired intervals](paired.csv) · [Retained bytes](scratch.csv) · [Verified column hashes and primary indices](block-inputs.csv) · [Overlap and rejection trials](ablations.csv) · [Raw ablation measurements](ablation-rounds.jsonl.gz) · [Raw round measurements](rounds.jsonl.gz) · [Reproduction](reproduce.md)
