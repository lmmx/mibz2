# Price path-capacity checks with the same growth sequence

The eight-slot constructor appends a byte to a lane's private spelling on each unary extension. This is 18,763,157 ordinary appends over the 228 admitted real blocks. A newly assigned job separately resets one lane and writes its seed. Three variants kept the same per-lane Vec and its normal growth policy.

The frame arm checks every eighth wave that each current capacity has eight bytes beyond max(current length,1). A continuing lane grows by at most one byte per wave. Reassigning a lane resets it to one seed before ordinary growth can resume. Thus all eight waves can fill existing capacity directly after a successful check. The slack variant uses the minimum current slack by the same bound; when zero, a checked wave can grow a tight lane naturally. The cursor arm keeps an initialized write pointer and one-past-capacity end for each live lane. On actual exhaustion it commits the initialized length and uses an ordinary push to grow. The start/refill and retirement paths reestablish the base or publish the initialized length. All pointer writes are confined to byte buffers with live ownership by their corresponding Vec. The pointer is replaced on every reallocation.

A separate checked observer validated the bounds of each direct write and compared all eight retained capacities against unmodified preload after each block. All four arms produced the Segmented output over the 205,197,794 real inverse bytes and passed the marked-column tests; the eight private paths reached 9,216 retained bytes in total. An independent counted build traced the exact opportunities.

| Arm | Ordinary path writes using the fast store | Ordinary writes through checked/growing path | Actual growth |
|---|---:|---:|---:|
| 8-wave frame | 18,760,641 | 2,516 | 57 |
| Minimum slack | 18,762,826 | 331 | 57 |
| Cached cursor | 18,763,100 | 57 | 57 |

The price binary omitted both counters and extra bound assertions. In each of two balanced four-arm, 64-lap processes, all timed outputs matched. The complete inverse thread-CPU ratios were as follows.

| Arm vs unchanged preload | CPU 4 | CPU 2 |
|---|---|---|
| 8-wave frame | 1.0273 [1.0039,1.0512] | 1.0140 [0.9705,1.0595] |
| Minimum slack | 0.9990 [0.9664,1.0326] | 1.0057 [0.9566,1.0573] |
| Cached cursor | 0.9946 [0.9571,1.0336] | 0.9743 [0.9385,1.0114] |

The successful slack proofs save almost all per-Vec capacity tests yet yield no resolved complete gain in this implementation. Cursor discovery measured 0.9845 [0.9350,1.0367] / 0.9876 [0.9468,1.0302]. The extra pointer/capacity management offers no supported reason to promote this research implementation or queue an independent run. Whole-process RSS and instructions were not sampled.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/path-writes
bash "$d/run.sh" /tmp/astra-e-path-writes
bash "$d/count.sh" /tmp/astra-e-path-count
python3 "$d/report.py" /tmp/astra-e-path-writes/{first,repeat}.csv
```

Set PAGE_DEPS to release deps of the frozen counted library from 252646a. Every timing run verifies the canonical corpus hash before processing it. The complete implementation and separate observation can be rebuilt without applying changes to the decoder.
