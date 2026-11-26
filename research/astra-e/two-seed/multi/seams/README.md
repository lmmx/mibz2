# RLE1 across four positive residual fragments

On every one of the 233 real admissions the stock RLE1 decoder could consume the four physical fragments without the final rotations. This removed 141,536,791 bytes of rotated ranges across 209,697,662 pre-RLE1 bytes. Of the 699 fragment seams, 671 gave direct parser restarts and 28 needed 65 combined input bytes in short bridges. All final output bytes and block CRCs agreed with the independent whole-stream decoder and libbzip2. The actual coded and final output buffers had identical growth histories in the stage experiment. [All per-block seams, CRCs and capacities](all.csv) · [Corpus gate](gate.txt)

The aggregate local emitter + RLE1 + CRC result on 14 fixed blocks was 0.9600 [0.9298, 0.9911] of the unmodified four-forward plus ordinary contiguous RLE1. The matched contiguous-output research code measured 1.0023 [0.9518, 1.0556] of that same stock path. The direct sliced/contiguous-research ratio was 0.9577 [0.9061, 1.0123]. These paired 95% intervals span 36 complete laps, with two position-balanced labels per implementation. The closer implementation comparison leaves the size and direction unresolved. Upstream inverse construction, scanner work and worker scheduling lie outside these clocks. [All 3,024 raw samples](paid.csv) · [All paired results and block strata](summary.json)

A globally known RLE1 boundary follows different adjacent bytes provided the four raw bytes immediately preceding the cut are not all the previous byte. When the logical cut is before the fourth position of the entire block, a count cannot be due there. Applying this exception inside an arbitrary physical slice would be incorrect. The parser can cross a cut only through a literal quartet or between a quartet and its count. The two parts of the condition exclude those cases. Different adjacent bytes alone would incorrectly split `9,9,9,9 | 0`. The proof checks arbitrary incoming parser states, clipped periodic sequences, unfinished counts and error rollback. [Independent proof, witnesses and exhaustive results](proof/README.md)

The implementation determines all seam cuts before touching its caller output. It hands each contiguous interior and each necessary small bridge to the stock vectorized or scalar RLE1 routine. If the full positive marked return is unavailable, or if no safe cut exists inside the width-32 window, it completes the original fragment rotation, period repetition, and ordinary RLE1. The research-only output hook uses the normal block-CRC checker and output rollback. Its coded scratch is owned by the experimental worker, so whole-process measurement must include that scratch and existing stock fallback capacities. The normal crate and its allocator are used as shipped. [Checked parser](slices.rs) · [Matched worker builder](build_worker.py)

The F5 hazard work previously treated individual dictionary words as parser actions and classified unsafe boundary cylinders. This construction reuses the ordinary parser across whole first-return fragments. The eliminated operation is the physical permutation of four already-spelled fragments. [F5 word results](../../../formulations/f5/README.md)

Native reproduction with Rust 1.99.1 and the frozen Wikidata corpus:

```sh
d=research/astra-e/two-seed/multi/seams
tmp=$(mktemp -d)
python3 research/astra-e/edge-displacements/build.py "$tmp/support"
python3 "$d/build.py" "$tmp/native" "$tmp/support/libmibz2.rlib"
bzip2 -dc corpora/wikidata.bz2 > "$tmp/oracle"
"$tmp/native/panel" synthetic
"$tmp/native/panel" gate corpora/wikidata.bz2 "$tmp/oracle" > "$tmp/all.csv"
cmp "$tmp/all.csv" "$d/all.csv"
taskset -c 3 nice -n 16 "$tmp/native/panel" price corpora/wikidata.bz2 "$tmp/oracle" > "$tmp/paid.csv"
python3 "$d/report.py" | cmp - "$d/summary.json"
```

## Complete worker

Both isolated hook implementations passed all 338 full scorer cases at source `85025c903eed22e2df130b6f345613a4ddfaf5d1`. These include malformed and corrupt input, exact real Wikidata bytes and the stock stream CRC path. Both used refreshed scorer path `75297a1d57b2bdf262c3a19f3064848dedf2c493`, printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The new executables were paired with the exact original scalar and scalar-four binaries. Every one of 160 complete local process observations checked its original block and stream CRCs and produced 209,700,196 bytes. [Correctness gate](official/candidate/summary.json) · [Joined control gate](official/joined/summary.json) · [Direct raw observations](official/direct/runs.jsonl)

| Direct ratio | W1 wall [95%] | W4 wall [95%] | W1/W4 peak RSS |
|---|---:|---:|---:|
| Sliced / joined hook | 1.1872 [0.9840, 1.4323] | 1.0641 [0.9592, 1.1805] | 1.0179 / 1.0127 |
| Sliced / original four-forward scalar | 0.9693 [0.7412, 1.2674] | 1.0385 [0.9185, 1.1741] | 1.0108 / 1.0073 |
| Sliced / i9-verified scalar | 1.0978 [0.9354, 1.2885] | 1.0228 [0.8748, 1.1960] | 1.0059 / 1.0038 |

The local complete-process comparison leaves the elapsed-time change unresolved. Its paired point estimate against the closest control was slower at both worker counts. The geometric, proof and output milestones stand independently of system throughput. Use `python3 research/astra-e/two-seed/multi/seams/official/verify.py` to reconcile the receipts. A live scoring round must refresh both scorer trees and build with `python3 research/astra-e/two-seed/multi/seams/build_worker.py` before invoking each entry through the scorer. [Frozen full-worker plan](official/PLAN.md) · [Paired and clustered intervals](official/summary.json) · [Exact new worker executables](official/workers.tar.xz)
