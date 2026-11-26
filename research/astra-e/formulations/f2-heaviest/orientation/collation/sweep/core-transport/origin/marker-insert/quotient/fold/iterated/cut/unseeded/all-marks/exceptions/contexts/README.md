# Exact physical predecessor exceptions from the original RLE

The constructor takes the original width/byte runs, the bzip2 primary ordinal and a requested forward context length k. It returns the maximal physical intervals of equal k-byte cyclic futures, their modal incoming bytes with exact successor states, the initial state, and maximal physical ranges of nonmodal bytes with their actual poststates. A single extension step intersects the current boundaries with original incoming runs and appends cumulative endpoints per byte. The native entry is construct(raw,k,charge) at source commit 3ddf456f. [Native header](contexts.hpp) · [Proof, worst-case family and binary ABI](PROOF.md)

On ten true Wikidata columns, all eight depths matched an independent dense LF/Ψ oracle at every row. Each uninstrumented model serialized identically to its audited counterpart. A separate process parsed these binaries, supplied the ordered exception clocks from a dense stable row traversal and reconstructed all 80 complete original byte blocks through the default transitions and ordered exceptions. The ten RLE1-decoded results matched independent bzip2recover/libbzip2 output and each stored block CRC. The bytes cover 8,999,785 final output bytes across the distinct blocks. The constructor alone supplies the event payloads in physical row order. The replay validator pays for a separate oracle to order them. [Wire SHA-256 and geometry](blocks.csv) · [Conditional full-output checks and block CRCs](replay.csv) · [Clock-order verifier](clock_oracle.cpp)

| k | States, all ten | Exceptional rows | Physical exception ranges | Wire MiB, all ten | Full construction ms per block¹ | Highest fresh RSS KiB |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 893 | 5,694,534 | 647,332 | 8.03 | 0.658 | 4,812 |
| 2 | 18,054 | 3,276,395 | 609,061 | 7.71 | 1.095 | 4,792 |
| 3 | 136,549 | 1,865,100 | 510,708 | 7.50 | 2.102 | 4,880 |
| 4 | 471,228 | 1,122,585 | 261,833 | 7.29 | 2.543 | 4,648 |
| 6 | 704,601 | 664,503 | 134,002 | 7.71 | 3.696 | 4,952 |
| 8 | 815,844 | 409,432 | 121,003 | 8.50 | 5.047 | 5,124 |
| 12 | 1,007,353 | 285,437 | 100,730 | 9.90 | 8.381 | 5,808 |
| 16 | 1,170,188 | 229,013 | 94,260 | 11.21 | 11.887 | 6,244 |

¹ Mean of the ten tape-specific median thread-CPU times over twelve rotating/reversed passes. Each timed invocation starts afresh from resident original RLE, reconstructs all intervening partitions and returns the entire model in memory. Clocks exclude original tape parsing, destruction of the returned model, disk serialization, clock acquisition and output production. Temporary construction storage is destroyed inside the clock. The native harness checks all four output cardinalities on every timed sample, and the uninstrumented binary's complete outputs are checked separately. Fresh RSS is the post-exec process peak for one original tape and one k, including the RLE input, model, executable and allocator. The local CPU is Intel Xeon Platinum 8573C, pinned CPU 3 at nice +12. [All 960 observations](times.csv) · [80 RSS observations](rss.csv) · [Exact denominators and structural prices](summary.csv)

The ten initial frequency passes scan 690,484 raw records at each independently requested k. At k=4 there are 2,858,488 primary run/partition intersections through all levels, 1,259,487 positive histogram entries to consume and clear, and 390,657 final intersections reread to emit physical exceptions. At k=16 the respective costs are 15,580,723, 12,940,911 and 130,970. Dynamic retained high-water is at most 1,680,948 bytes at k=4 or 2,869,293 at k=16, excluding the original input vector, fixed-size objects and allocator transients. Mixed intervals alone need the final second read. Producing the deeper state reduced physical exception rows by 79.6% against k=4 and took 4.68 times its local construction CPU. The smaller event count must pay for its more expensive states, ordered coordinates and writer in a complete decoder. [Every depth's actual charges](layers.csv)

The finite gate checks all 48,120 combinations of a ternary column through six rows, a primary ordinal, and a requested k, including 39,792 multiple-cycle combinations and 16,560 nondividing horizons. Each model also replays a second horizon n+1 with the correctly phased reverse start. A two-run compressed example has 2³²−1 rows and takes exactly j+2 intersections at every level j≤1024; the final model has one physical exceptional row. On a homogeneous column of the same size, an arbitrarily large k stabilizes after the first refinement. [Exact gate counts](small.json) · [Dense test implementation](oracle.hpp)

## Reproduction

Run from the repository root with g++ 14.2.0. The named source and this directory's tight geometry program are independent of the shipping path. The raw original tapes and their checksums are in the input manifest. The separate clock census used for cross-checking state/edge/exception totals is frozen at 50f89ed8. [Input identities](inputs.csv) · [Census](../panel/README.md)

```bash
d=$(dirname "$(rg --files | rg '/all-marks/exceptions/contexts/contexts.hpp$')")
scratch=$(mktemp -d)
g++ -std=c++20 -O3 -g0 -Wall -Wextra "$d/main.cpp" -o "$scratch/model"
g++ -std=c++20 -O3 -g0 -Wall -Wextra "$d/clock_oracle.cpp" -o "$scratch/replay"
"$scratch/model" small | diff - "$d/small.json"
python3 -B "$d/real.py" "$scratch/model" research/astra-e/dense/target/wikidata "$scratch"
diff "$scratch/blocks.csv" "$d/blocks.csv"
diff "$scratch/layers.csv" "$d/layers.csv"
diff "$scratch/inputs.csv" "$d/inputs.csv"
python3 -B "$d/replay.py" "$scratch/model" "$scratch/replay" "$scratch" corpora/wikidata.bz2 | diff - "$d/replay.csv"
taskset -c 3 nice -n 12 python3 -B "$d/price.py" "$scratch/model" "$scratch"
python3 -B "$d/report.py" "$scratch"
python3 -B "$d/report.py" "$d" | diff - "$d/summary.csv"
```

The measured native executable SHA-256 is 260dbada6608aae674a3622b66700cd9af0189e223e881c7d606f33112bcb1aa. The separate conditional replayer is b5af0f5c7aed3f1bcf905dd3ed5e35530818648bc7cb2cf184b2ce691faa1098. The executable accepts dump ORIGINAL.txt FOLDER K to write a complete uninstrumented wire. A worker receiving caps in another representation must charge conversion to the ordered original RLE or provide a measured equivalent iterator. The decisive next measurement is a same-input whole inverse with direct construction, independently acquired ordered clocks, output, and jointly live memory against the original inverse control. Scorer, CI and i9 evidence are needed for production throughput.
