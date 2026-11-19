# Three complete induced inverses on original capped blocks

The best of the three sections, original L runs of width at most four plus the primary, used **2.645 [2.577, 2.716] times scalar-cap's inverse CPU**. It took 58.80 ms for ten blocks, against scalar's 21.71 ms, LazyCells' 22.33 ms and OpenGated's 21.87 ms. Its largest fresh process reached 14,192 KiB, against 5,976 KiB for scalar. All three complete marked reconstructions passed the byte checks. None qualifies for a whole-worker benchmark in its current implementation. [Registered scope](PLAN.md) · [All paired results](summary.json)

| Original row section plus primary | Rows selected over ten blocks | Induced records | Grammar joins | Ten-block CPU ms | Paired ratio vs scalar [95%] | vs Lazy | vs opend | Maximum process KiB |
|---|---:|---:|---:|---:|---|---:|---:|---:|
| First floor(n/22) rows, at least one | 409,094 | 80,819 | 1,029,865 | 62.41 | 2.810 [2.664, 2.964] | 2.713 | 2.802 | 11,668 |
| Every original L run of width ≤4 | 784,776 | 617,326 | 445,729 | 58.80 | 2.645 [2.577, 2.716] | 2.554 | 2.638 | 14,192 |
| 16 least frequent present F-byte buckets | 6,745 | 4,110 | 1,227,593 | 63.51 | 2.979 [2.648, 3.351] | 2.876 | 2.970 | 12,464 |

This ten-block corpus contains 8,999,915 original rows. Reducing the section from 784,776 to 6,745 selected rows reduced the number of return records but increased grammar joins substantially. Across the three arms, the median summed times for coalescing/choosing the section were 2.59/0.01, 2.59/2.26 and 2.49/0.05 ms. Induction used 22.05, 24.80 and 23.73 ms. Building the selected-row map used 1.13, 2.98 and 0.70 ms. Marked emission used 36.59, 25.94 and 35.70 ms. Charging zero for the smaller stages would still leave these implementations behind the measured scalar. The phases are medians of separate per-round totals and need not add exactly to median total CPU. The outside clock also includes return-map teardown and bridge/clock costs. [All raw phases and immutable donor clocks](paired.csv) · [Complete control observations](times.csv)

The gate checked 60 original six-arm calls from the same donor's true run frontend, with full byte equality and matching cap/raw hashes from the earlier libbzip2-backed original input receipt. An independent stable sort produced Ψ from each cap tape and verified every marked byte via F and Ψ. The worker's separate exact gate checked 30,603 arbitrary arm/mark calls and all 30 deployed selector outputs before pricing. The ten real marked laps have full length n. Every subsequently timed call was checked outside its clock against the same raw inverse. The stage ends at inverse output, before RLE1 expansion and block CRC; the independently authenticated fixture supplies the original plaintext hashes and block CRCs. [Donor and dense gate](gate.json) · [Native arbitrary-column gate](../EXACT.md)

All arms used the same frozen six-position Rust timing driver and compiled scalar-cap source at `d0a42980e896211906234c7679b38760ae44a46e`. The worker at `758c0abd` and compact constructor at `cd35d808` rebuild the section, induced record table, grammar, marked row map and output on every call. Only reusable scratch capacity persists. Measurements used one untimed rotation and 24 complete six-arm rotations, each arm occupying every position four times within each block. Ninety-five percent paired log-t intervals use four chronological groups of six rounds. The 1,440 native samples used thread CPU on CPU0, nice 12, on a Xeon 8573C under a four-equivalent shared-host quota. There is no whole-worker, official scorer, i9, or memory-transfer result in this experiment. [Library, compiler and compressed input identities](build.json) · [Host receipt](runtime.json)

The 180 separate RSS processes each opened exactly one authentic cap tape before measurement. No symbol vector, source column, plaintext, prepared geometry or reference output was loaded. A detached copy of the frozen Rust library received one RSS-only constructor for `Runs`, populating exactly its original records, per-byte record counts and per-byte widths. Every arm ran in this harness. The oracle raw buffer was opened only after reading `VmHWM`. The measured CPU executable remained the immutable Rust donor with the original scanner and run frontend. Median fresh process sizes were 6,052/6,686/6,058 KiB for prefix/thin/rare and 4,594/4,330/4,756 KiB for scalar/Lazy/opend. Maxima on the three controls were 5,976/9,056/9,052 KiB. Vector capacities in the stamps report retained arrays after each call and a separate value before releasing the induced result. Construction temporaries and allocator high water enter process HWM. [RSS-only build](rss-build.json) · [All fresh processes](rss.csv)

From this directory, the local reference path contains the frozen Rust driver and cap inputs from the original capped benchmark. Its tracked builder and source attestations are located under `powers/xor/capped`. The separate component gate beside this directory should pass before the price.

```sh
ref=/workspace/work/astra-powers-original/capped
out=$(mktemp -d)
taskset -c 0 nice -n 12 python3 -B run.py "$ref" "$out" build
taskset -c 0 nice -n 12 python3 -B run.py "$ref" "$out" gate
taskset -c 0 nice -n 12 python3 -B run.py "$ref" "$out" price
taskset -c 0 nice -n 12 python3 -B rss_build.py "$ref" "$out"
taskset -c 0 nice -n 12 python3 -B run.py "$ref" "$out" rss
python3 -B report.py "$out"
```
