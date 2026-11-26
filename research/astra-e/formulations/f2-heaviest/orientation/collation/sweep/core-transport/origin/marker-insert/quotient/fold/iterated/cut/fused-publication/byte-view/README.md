# One physical marked cap and a bounded first-cut matrix

A first contraction can consume the original capped byte records directly. Scanning the frequencies and locating the physical image of the marked source determines its singleton class everywhere in the input. The exception lies in one cap. A dense matrix counts each original seeded source/destination pair for voting; the same slots then store each outgoing alias's final rank. On successful initial cuts the constructor allocates the new weighted column and grammar. Its old maximal-run column, seeded column, overlaid slice list and ancestry sequence each have zero entries. The live matrix has 4K² bytes. Two suspended run coordinates per target suffice for reading skipped predecessors. [Seed theorem](PLAN.md) · [Construction bound](GEOMETRY.md) · [Native constructor](matrix.hpp)

All 109,227 cap partition/primary combinations from ternary columns of lengths one through seven matched the explicit seed pointwise, including 4,779 cases with two splits inside the marked cap. The native gate checked every new root and grammar on 62,326 ordinary cuts, took the mandatory-power route 15,464 times and identified 31,437 terminal fixed points. The complete three-entry ABI independently reproduced the exact stable marked bytes for 23,211 arbitrary cap partitions with primaries through length six and twelve long-power/mark cases. The ordinary grammar completes mandatory-power cases after paying for the matrix's decision. The scalar input contract remains in the unmodified donor front end. [Pointwise receipt](seed.json) · [Native gate](native-small.csv) · [Full ABI gate](abi.json) · [Exact output verifier](abi.py)

All ten original Wikidata inputs admitted the first contraction. Their 740,790 original capped records produced 567,627 weighted incoming runs on 5,974,644 surviving rows, removing 3,025,271 old rows and preserving 8,999,915 weighted marked bytes. The single exceptional physical cap added 17 total partition boundaries across ten blocks. Actual K ranged from 87 to 98; the matrix occupied 30,276–38,416 bytes per original first cut. Every newly published run and grammar index matched the previously verified direct publisher. [Per-block geometry](geometry.csv)

| Local same-caps paired quantity | Ratio and 95% interval |
|---|---:|
| New first cut / previous explicit first cut, both followed by grammar | 0.624 [0.583, 0.668] |
| New first cut / complete scalar-cap, later grammar | 0.838 [0.806, 0.871] |
| New first cut / complete scalar-cap, later direct weighted traversal | 0.869 [0.836, 0.904] |
| Whole new grammar path / whole previous sliced grammar | 0.930 [0.884, 0.978] |
| Whole new grammar path / complete scalar-cap | 6.235 [5.889, 6.602] |
| Whole weighted direct path / complete scalar-cap | 4.853 [4.757, 4.952] |

The changed representation moves the first paid cut below the scalar-cap inverse's cost. The geometry of every surviving row stayed identical. In this same panel the explicit first cut alone took 1.344 [1.284, 1.406] of scalar-cap. Across the 24 matched ten-block rounds, the direct residual arm had a median 12.49% of complete scalar CPU left after its prefix. Its measured remaining work took 3.982 [3.868, 4.100] times the entire scalar inverse. Matching scalar on this route would require removing a median 96.99% of the measured continuation cost after the first root. Peak high water over 30 fresh one-block processes was 6,848 KiB with weighted residual, 9,036 with grammar, 9,320 with the ordinary sliced grammar and 6,100 with scalar. The measurements cover the run-input inverse with matched worker count one on a shared quota host. They carry no independent CI or workstation verification. [Preregistered protocol](PAID.md) · [1,440 full observations](capped/capped.csv) · [Prefix clocks](capped/stamps.csv) · [180 memory calls](capped/fresh.csv) · [Per-block residual budgets](budget.json) · [Paired intervals](capped/summary.json)

The previous lower bound concerned a particular initial representation. This first-cut implementation falsifies any broader claim that constructing the exact first root must already cost more than a production inverse. The remaining 5.97 million physical survivors and 567,627 RLE entries still require an efficient marked continuation. Skipping publication of the surviving column entirely, or using an output operation that can consume the suspended old-bucket cursors directly, would change that cost. Estimating the number of queried old intervals and the price of ordering the marked output is an appropriate next falsifier for such a consumer. The two measured completions have substantial local time and peak-memory gaps.

## Reproduction

Use the exact old source generation and unchanged capped Rust reference from the parent README. The compiled C++ shim links at runtime using the verified DSO. Fresh-process RSS runs load the independently recovered compressed block before measuring the inverse. The complete cap panel validates output bytes for every treatment, then checks each input cap and raw output SHA-256 against the original libbzip2-qualified receipt. The affine proof and C++ scripts operate on research objects only. [Parent build](../README.md) · [Price build hashes](price-manifest.json) · [Frozen Rust binary](../capped/manifest.json)

```bash
b=$(dirname "$(rg --files | rg 'fused-publication/byte-view/PAID.md$')")
d=$(dirname "$b")
w=$(mktemp -d)
ref=/workspace/work/astra-powers-original/capped
python3 "$d/raw-build.py" "$w/raw" > "$w/raw-manifest.json"
python3 "$b/build.py" "$w/raw" "$w/view" check
python3 "$b/seed.py"
"$w/view/check" small
python3 - "$d" "$ref" > "$w/view/inputs.txt" <<'PY'
import json, pathlib, sys
d,ref=map(pathlib.Path,sys.argv[1:])
for q in json.loads((d/'../powers/xor/capped/receipt.json').read_text())['inputs']:
    print(q['block'],q['mark'],ref/f"block-{q['block']}.caps")
PY
"$w/view/check" "$w/view/inputs.txt"
python3 "$b/build.py" "$w/raw" "$w/view" price
python3 "$b/abi.py" "$w/view/libgrammar.so"
taskset -c 0 nice -n 16 python3 "$d/caps-run.py" "$w/raw" "$ref/panel" corpora/wikidata.bz2 "$w/view" price joint byte_grammar byte_residual
taskset -c 0 nice -n 16 python3 "$d/caps-rss.py" "$ref/rss" corpora/wikidata.bz2 "$w/view" joint byte_grammar byte_residual
python3 "$d/caps-report.py" "$w/view"
```
