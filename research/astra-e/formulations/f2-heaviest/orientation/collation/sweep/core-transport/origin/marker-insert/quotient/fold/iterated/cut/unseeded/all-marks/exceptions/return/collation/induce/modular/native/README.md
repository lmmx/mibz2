# Shared spelling and full cost of modular induction

The grammar really reused spelling on all ten Wikidata blocks. For q=2, cached occurrences copied 7,758,796 of 8,999,915 marked bytes (86.2%). The full inverse still took **8.966 [8.802, 9.134] times scalar-cap's CPU**. At q=4 it took 8.960 [8.780, 9.143], and at q=8 it took 10.142 [9.914, 10.375]. The stage includes the whole modular induction from original caps, all allocations and teardown, and writing the output. No residual traversal, prepared index or dense LF input is supplied. [Paid protocol](PLAN.md) · [Reconciled results](summary.json) · [Implementation](../modular.hpp)

| q | Grammar joins | Cached copy calls | Bytes delivered by those copies | Induction ms | Spelling ms | Whole inverse ms | Maximum fresh-process KiB |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 2 | 2,001,159 | 760,050 | 7,758,796 | 105.30 | 93.52 | 201.19 | 17,440 |
| 4 | 2,233,806 | 771,382 | 7,537,481 | 94.37 | 103.41 | 198.92 | 18,548 |
| 8 | 2,524,599 | 739,708 | 7,215,014 | 105.24 | 117.84 | 225.90 | 20,852 |

The matched Rust scalar/LazyCells/OpenGated medians were 22.08/23.64/21.87 ms. Each median sums the ten blocks per round. Coalescing original 127-capped records into maximal runs cost 2.09/2.10/2.21 ms. The initial run builder populated start, end and byte. The unused per-run LF base stayed zero. At q=4, 2.14 million residue fragments and 2.23 million grammar joins remained; the actual spelling pass visited 4.47 million stack entries. Either induction or spelling alone cost over four times the whole scalar inverse. The results close these tested implementations as throughput candidates and give separate budgets for any replacement emitter or constructor. [Raw Rust calls](times.csv) · [Per-call stage and sharing data](stamps.csv) · [Joined records](paired.csv)

The component gate at `5d240588` verified 85,254 marked radix cases, including 38,076 whose marked period did not divide the output length. The compiler dependency digests recorded by the six-arm donor matched that source. All 30 real treatment outputs matched the original cap and libbzip2-backed marked-byte hashes; an independent stable-sort Ψ checked every marked byte and the lap length on each block. Every subsequent timed call was also checked byte for byte after its clock, and its join, visit, cached copy and copied-byte counts were reconciled against the separate modular geometry gate. The measured stage finishes before RLE1 expansion and CRC. [Component gate](../small.txt) · [Real source and oracle hashes](gate.json) · [Geometry](../levels.csv)

The pinned Xeon 8573C process used CPU0, nice 12, with a four-equivalent local quota. The three controls were instantiated in the immutable earlier Rust donor at scalar source `d0a42980e896211906234c7679b38760ae44a46e`. Twenty-four complete six-arm rotations put each arm in every position four times on each block, yielding 1,440 raw observations. Two-sided 95% paired log-t intervals use four consecutive six-round groups. A separate arm/block process loaded only the cap tape and counts before running one kernel. It opened the reference output after sampling high water. Three replicates per cell yielded 180 observations. Control maxima were 5,908/9,044/9,112 KiB. The same authenticated detached cap-only RSS binary as the preceding induced-section panel linked this study's modular plugin at execution. There is no i9 or whole-decoder timing here. [Build receipts](build.json) · [RSS build](rss-build.json) · [All fresh HWM](rss.csv) · [Host](runtime.json)

From this directory, the following replays the complete experiment; the standalone modular finite/real gate in the parent must have passed first.

```sh
ref=/workspace/work/astra-powers-original/capped
out=$(mktemp -d)
for phase in build gate price rss-build rss; do
    taskset -c 0 nice -n 12 python3 -B run.py "$ref" "$out" "$phase"
done
python3 -B report.py "$out"
```
