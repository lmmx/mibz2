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
