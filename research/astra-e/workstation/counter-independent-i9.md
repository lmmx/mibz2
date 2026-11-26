# Independent i9 result for scalar-cap with queue-counter recovery

**The combined worker passed the registered pooled comparison against edge and missed it against scalar-cap.** The two W7–10 pooled wall ratios were 0.993791 [0.989841,0.997756] against edge and 0.999805 [0.994519,1.005118] against scalar. All eight per-worker incremental guards and all twelve Lazy memory guards passed. Every one of the 160 paired candidate peaks was lower than Lazy. The registered joint incremental and stretch decisions were both false. Scalar-cap stays the independently verified speed reference.

The Coordinator's Intel Core i9-10900K used 40 balanced rounds, five arms, W7–10, 4W slots and a two-second settle. Each of the 800 timed processes produced 838,800,784 bytes from four copies of the 431-block Wikidata corpus. Candidate 50b8ecb8946a498601e3fd894ed84a2687df6cc8 used the same scalar first-cap specialization as d0a42980e896211906234c7679b38760ae44a46e, plus retained-queue counter recovery. Edge was badb2aec3624bd65606c429b2e6f79e9d9d0050f. Each implementation passed all 338 full libbzip2, CRC and adversarial records. Both target invocations passed the 20 explicit worker settings and all 20 affinity-specific gates. The scorer path was 75297a1d57b2bdf262c3a19f3064848dedf2c493 and it printed v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04. [Frozen rule](counter/README.md) · [Timed archive](evidence/cap-counter-i9-1b8d2a3.tar.gz) · [Independent dry](evidence/cap-counter-dry-9eb5782.json)

| W | Combined / Lazy wall [95%] | Throughput vs Lazy | Combined / edge wall [95%] | Combined / scalar wall [95%] | Combined / scalar CPU [95%] |
|---:|---|---:|---|---|---|
| 7 | 0.86565 [0.85993,0.87141] | 1.1552× | 0.99253 [0.99022,0.99483] | 0.99645 [0.99443,0.99847] | 0.99638 [0.99390,0.99887] |
| 8 | 0.88243 [0.87660,0.88829] | 1.1332× | 0.99580 [0.99037,1.00126] | 1.00120 [0.99410,1.00835] | 0.99773 [0.99337,1.00211] |
| 9 | 0.88264 [0.87459,0.89077] | 1.1330× | 0.99167 [0.97937,1.00413] | 0.99749 [0.98461,1.01054] | 0.99169 [0.98561,0.99781] |
| 10 | 0.86828 [0.85946,0.87719] | 1.1517× | 0.99517 [0.98338,1.00709] | 1.00410 [0.99233,1.01601] | 0.99765 [0.98617,1.00928] |

| W | Paired RSS / Lazy upper | Combined / Lazy median peak KiB | Median ratio | Combined / Lazy maximum KiB | Maximum scalar KiB |
|---:|---:|---:|---:|---:|---:|
| 7 | 0.70278 | 191,918 / 275,270 | 0.69720 | 195,552 / 298,956 | 195,676 |
| 8 | 0.69237 | 202,810 / 295,078 | 0.68731 | 205,540 / 332,332 | 206,836 |
| 9 | 0.68477 | 210,914 / 310,024 | 0.68032 | 214,188 / 329,804 | 214,592 |
| 10 | 0.67521 | 219,664 / 327,588 | 0.67055 | 222,524 / 356,512 | 223,704 |

The native counter specialization demonstrably reduced frontend work, and at W4 independent AMD CI its wall ratio versus scalar was 0.9843 [0.9760,0.9927]. The target's pooled interval permits effects ranging from a 0.55% reduction to a 0.51% increase in time. Only W7's individual combined/scalar interval is wholly below one. The target sweep collected wall, process CPU and peak RSS. Hardware instruction, cache, IPC and DRAM observations are available from different experiments with different scope. [Direct contrasts across all ten arm pairs](counter-independent-i9.csv) · [Independent statistics and all 40 round ratios](counter-independent-i9.json) · [Independent CI](../ci/evidence/cap-counter-44460943655.md) · [Three-block proxy](../results/cap-counter-route-pricing/)

Archive SHA-256 is d041b544977aef51ffd8377466fa4108bc6e779d0a470385a6436b0ccbb23d66. Each direct pooled test first averages the four same-round log ratios, then uses the two-sided df39 Student interval. The independent implementation reproduces the official interval endpoints to within 2×10⁻¹² and all decisions exactly. The receipt also verifies both original public archives, their published digests, source entries, generated Rust dependency closure, baseline release changes, scorer selection, separate builds and target machine. The workstation's numbered release additions were checked against both pinned Git trees. [Source and machine authentication](evidence/cap-counter-i9-1b8d2a3.json) · [Original publication](https://github.com/lmmx/mibz2/tree/1b8d2a33a9976e0f8bc4d7cda2e6188dd6ab8ab5/astra-e-cap-counter)

## Reproduction

From the repository root, replay the archived measurements with the following commands. The measurement recipe itself is frozen at a7e314a6f433c3ed0b1fb83ec5d9052f56be0f09.

```bash
d=research/astra-e/workstation
p=$d/evidence/cap-counter-i9-1b8d2a3.tar.gz
python3 -B "$d/counter/verify.py" "$p" a7e314a6f433c3ed0b1fb83ec5d9052f56be0f09
python3 -B "$d/counter_metrics.py" --self-test
python3 -B "$d/counter_metrics.py" "$p" --csv /tmp/astra-e-counter-target.csv
python3 -B "$d/counter_receipt.py" "$p" 1b8d2a33a9976e0f8bc4d7cda2e6188dd6ab8ab5 astra-e-cap-counter/run-20261119T061036Z.tar.gz
```
