# Seven-worker model for eager packed descriptors

At commit `ff36563`, the scorer's W=7 model used 2,925 KiB of LLC, 66 cycles for L3 and 549 cycles for DRAM. Each executable ran five arms so the candidate can be compared with a reference that shares its build. The model brackets the scorer's `decode_block` scope and primes scratch using the first selected input. For the two-block sample the second, wider input can therefore grow scratch in the modeled region. [Exact scorer and inputs](packed/manifest.json)

The frozen stream block is index 1, with 51,184 letters. The second screen pairs index 85 (large ordinal 48, 62,533 letters) with index 269 (large ordinal 144, 124,940 letters). The packed, hybrid and narrow checks all admitted all three. The mixed pair exercises both bottom-link representations in the narrow arm. Three chosen stream blocks and one worker-share setting make this a targeted advisory comparison. This experiment has no end-to-end wall result. [Selected block identities and symbols](narrow/manifest.json) · [All raw-derived metrics and reference-range checks](summary.csv)

Rates are per reconstructed final byte. Each reference row below is from the packed executable. Small variations in the other two reference builds are retained in the CSV.

| Input | Arm | Instructions | Model cycles | L1↔L2 B | L2↔L3 B | DRAM B | Dependent past-L2 links | Dependent DRAM links | DRAM MLP |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Frozen | LazyCells | 19.47 | 10.21 | 36.08 | 23.57 | 6.99 | 0.01466 | 0.000037 | 2.61 |
| Frozen | Packed | 27.12 | 13.23 | 35.65 | 26.38 | 8.06 | 0.01361 | 0.000037 | 2.61 |
| Frozen | Hybrid | 26.80 | 13.61 | 33.16 | 25.68 | 8.06 | 0.01366 | 0.000037 | 2.60 |
| Frozen | Narrow | 28.67 | 14.90 | 34.23 | 24.85 | 8.07 | 0.00861 | 0.000037 | 2.60 |
| Mixed | LazyCells | 33.53 | 19.74 | 62.41 | 48.39 | 10.12 | 0.04694 | 0.000063 | 1.63 |
| Mixed | Packed | 46.07 | 26.87 | 60.98 | 49.49 | 13.55 | 0.03599 | 0.000659 | 1.20 |
| Mixed | Hybrid | 45.16 | 23.81 | 55.78 | 46.94 | 12.88 | 0.03542 | 0.000634 | 1.39 |
| Mixed | Narrow | 46.85 | 28.14 | 60.09 | 48.54 | 13.53 | 0.03371 | 0.000609 | 1.19 |

All six candidate cells are outside the four-arm reference instruction envelope under the scorer's 10% rule. Frozen narrow is also outside the past-L2-link envelope. The modeled cycle ratios have correspondingly limited predictive value for the workstation. Open-Return meanwhile executes 14.55 instructions per frozen byte in these builds.

**Diagnosis.** The five-byte control reduced past-L2 dependence from 0.01466 to 0.01361 links/B on frozen and from 0.04694 to 0.03599 on the mixed pair. Its instruction penalty is respectively 39% and 37%. On the mixed input the three packed-family implementations issue roughly 9.7–10.4 times as many dependent DRAM links as Lazy, with less useful overlap. Frozen narrow cuts past-L2 dependent links by 37% relative to packed and transfers 6% fewer bytes across L2↔L3, yet costs 1.55 additional instructions/B. Hybrid's 19,478 fewer frozen parent rules remove only 0.32 instructions/B and the modeled cycle count rises. On the mixed pair the same hybrid lowers modeled cycles 11.4% against packed; its 23.81 cycles/B still exceed Lazy's 19.74. The aggregate dependency count, the memory level of a binding load and the work to issue it must be distinguished.

The three implementations remain available at their source commits. A whole-process, same-version paired win would supersede this model ranking. The two structural optimizations in isolation have given no proxy case for an i9 request. A successor to this family needs source-level instruction attribution before specializing more slot widths or grammar cutoffs. A frozen Callgrind breakdown for initial census, induction and writer would distinguish costly extra rule work from bounds checks and repeated packed-field extraction before another kernel is built.

## Reproduction

Use this report/driver checkout and a detached experiment checkout established at `ff36563`. Build the scorer's Valgrind tool using its own README and set `VALGRIND_LIB` to that tool runtime. A pinned scorer replay also requires master still to resolve to the recorded scorer version, or a preserved remote/ref replay of that state. The driver deliberately refreshes before each run. It records the resolved source and candidate versions rather than assuming a stable remote.

```sh
git worktree add --detach /tmp/packed-model-tree ff36563
for arm in packed hybrid narrow; do
  case "$arm" in
    packed) entry=research/astra-e/cells/packed.rs ;;
    *) entry=research/astra-e/cells/$arm/candidate.rs ;;
  esac
  python3 research/astra-e/cells/chain/screen.py \
    /tmp/packed-model-tree "$PWD/corpora/wikidata.bz2" "/tmp/packed-model-tree/$entry" \
    "$VALGRIND_LIB" /tmp/packed-model-build "/tmp/packed-model-results/$arm"
done
python3 research/astra-e/cells/chain/report.py /tmp/packed-model-tree /tmp/packed-model-results
```
