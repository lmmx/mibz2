# Measure private path width after compact response preloading

The control is the frozen eight-slot preload at 6758831810b6a913c88ff079dede881b578ca03a. This study also compiles an independent instantiation of eight and four alternatives using one const-parameterized source. Each complete block has one configured width. All variants preload every response when all slots are active. Partial waves use the live mask. Parents publish their children before assignment, each lane mutates only its own row and spelling, and each retiring job partitions the same interval in stable order. Changing lane count can change publication order, intermediate node identities, and the numbering of terminal words. Original F order and marking still determine the same inverse. The forced checks verified 11,264 admitted small marks for each of 4, 8, 12, 16 and 24 lanes. The real check verified every candidate against Segmented for 205,197,794 bytes on all 228 admitted blocks.

Every width issued 19,750,839 dynamic first-level queries. The capacity observer reuses the existing F2 scalar/first/second vector accounting from f2-scheduler/storage. It processed the same ordered corpus with one retained worker per width. The first-level vector and second-level vector high waters were 3,450,448 B and 3,214,016 B respectively at all five widths. These exclude the separately charged private paths, fallback storage, input/output buffers, allocator overhead and process residency. The distinct real wave counts and maximum actual capacities follow.

| Lanes | Waves | Private path capacity | First + second + path high water | CPU4 complete relative to same-source 8 | CPU2 complete relative to same-source 8 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4 | 4,940,898 | 5,120 B | 6,669,584 B | 1.0614 [0.9979,1.1291] | 1.0509 [1.0224,1.0801] |
| 8 | 2,476,362 | 9,216 B | 6,673,680 B | 1.0000 | 1.0000 |
| 12 | 1,656,658 | 12,288 B | 6,676,752 B | 1.0316 [0.9800,1.0859] | 1.0406 [1.0082,1.0740] |
| 16 | 1,247,707 | 16,896 B | 6,681,360 B | 1.0083 [0.9513,1.0687] | 1.0480 [1.0188,1.0781] |
| 24 | 840,796 | 22,016 B | 6,686,480 B | 1.0696 [0.9888,1.1570] | 1.0760 [1.0462,1.1067] |

Every 60-lap process included the frozen eight and the five width instantiations, with six arms each occupying each position ten times per block. The independently instantiated eight divided by frozen eight measured 0.9593 [0.8908,1.0330] and 0.9848 [0.9579,1.0124] on complete paired thread CPU. Against both controls the experiment provided no gain from widening. On the CPU2 repeat each alternate width was slower than its same-source eight on the complete inverse. The number of waves decreased substantially at higher widths while private paths grew. Width eight remains the judged-throughput entry; the wider source is retained to reproduce the tests.

```sh
d=research/astra-e/formulations/f2-pages/payload/slots/width
bash "$d/run.sh" /tmp/astra-e-slot-width
bash "$d/capacity.sh" /tmp/astra-e-slot-capacity
python3 "$d/report.py" /tmp/astra-e-slot-width/{first,repeat}.csv
```
