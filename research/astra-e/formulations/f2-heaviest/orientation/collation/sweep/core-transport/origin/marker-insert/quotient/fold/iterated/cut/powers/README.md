# Reconstructing the original marked byte BWT with powers and delimiter cuts

A protected delimiter cut can now guarantee at least one quarter of the current rows disappear on **every** nonterminal input. First try the ordinary cut against its exact entering mass. If it falls short of one quarter, collapse complete equal-type runs into weighted power tokens and cut their zero-diagonal column. This gives at most ceil(log base 4/3 of m) rounds, at most 4m cumulative ordinal rows and at most m new spelling nodes from an initial m-row byte or token column. One direct construction therefore starts at the **original byte BWT**, retains neither a named delimiter dictionary nor a dense original LF walk as a required phase, and finishes with a grammar for the marked cycle. [Stable aliases and work bound](PROOF.md) · [Exact cut gate](GATE.md)

On all ten Wikidata original byte columns this construction produced the correct final bytes and block CRCs. On the frozen block its native implementation took **1.384 [1.121, 1.708]** times the CPU time of a simple full dense byte inverse in the same 24 pairs. Fresh-process peak RSS was **6,368 versus 7,512 KiB**. This is a lower-memory stage with a time regression. The engineering best-so-far and the i9 throughput result are unaffected. The original type graph, cut publication and grammar expansion are fully charged in this stage. [Matched native results](gated/summary.json) · [Raw pairs](gated/times.csv)

## Why normalization works

The source positions supporting d occurrences of type a form nested intervals inside its old source bucket. Positions exiting after exactly d steps have a left- or right-exit interval. Restricting those intervals to actual run starts keeps each alias contiguous and preserves strict destination order. The retained set has m−e rows when there are e same-type edges. All of its edges go to a different type. Constant-type cycles are fixed points by stable source order. The protected singleton either lives at a run start or is already a fixed point whose byte word can be repeated directly.

The finite proof checks include all 9,225 weighted byte marks in small surjective columns, including 7,616 marks with other cycles and 5,819 nonintegral numbers of marked laps. The two power constructors solved the identical cases, with 6,884 normalization steps and 3,870 power nodes. Separate writer checks cover 560 spans inside positive literal, concatenation and nested-power words. On the previous two-run obstruction of length N+1, even N=2³²+7 gives only two retained rows, three compressed point lookups and one cut. The native byte harness checks mandatory guarded fallbacks up to N=16,384. [Marked checks](interval-panel/small.json) · [Native full gates](gated/native-check.json)

## Constructing the power BWT without a row scatter

A forward constructor follows compressed affine successor pieces, then scatters a label at each of m−e destinations. A backward constructor begins instead at each mismatch interval in destination order. It intersects the corresponding predecessor interval with old incoming RLE records. A different-type predecessor closes the word. A same-type piece continues by its affine LF translation, bulk-advancing an entire interval whenever possible.

Across all roots and all depths the physically visited source intervals are disjoint, since a source has one first future different-type exit and one distance to it. Each old incoming run seam can split at most one visited interval. Consequently the number of terminal fragments before even coalescing is at most T+R−1 for T starting mismatch intervals and R old maximal runs. The output needs O(R+K) working words apart from its payload, for K types. The lookup count is bounded both by m and by O(R²), independently of large uniform multiplicities. [Proof and complexity](RUN-BOUND.md) · [Compressed implementation](backward.py)

On the first original frozen byte column, 15,635 of 899,931 rows had same-byte transitions. Eager normalization emitted 48,980 runs of 259 aliases from 884,296 retained rows and 66 power nodes. The pointwise algorithm made 899,931 successor lookups; the backward algorithm used 51,639 interval lookups and no destination scatter. Over the complete eager marked grammar there were three normalized passes, 114,644 backward interval lookups, 24,141 old seam splits, 135,985 terminal fragments and 83 newly allocated power nodes. The full 25-round output contained 14,259 literal bytes, 885,532 bytes copied from earlier complete grammar nodes and 140 directly copied repetitions. Its 899,931-byte marked lap matched the original stable traversal, and RLE1 yielded 900,000 bytes with CRC d4a25a5c. [Complete eager original](interval-panel/original.json) · [All eager passes](interval-panel/depths.csv)

| Frozen native first-phase arm | Median thread CPU | Fresh VmHWM | Charged successor/interval lookups |
|---|---:|---:|---:|
| Compressed backward intervals | 12.930 ms | 5,556 KiB | 51,639 |
| Dense forward scatter | 17.957 ms | 14,588 KiB | 899,931 |
| Compressed forward scatter | 59.057 ms | 8,188 KiB | 899,931 |
| Complete dense original byte inverse control | 19.835 ms | 7,516 KiB | Separate full inverse |

The three publication arms produced identical full spellings, ordered aliases, destination RLE and primary type. Their clocks include primary seeding, index construction, publication and destruction. The direct arm produces the complete original marked bytes. All four arms used the same original run tape and 24 balanced rotating/reversed rounds. Backward/dense publication was 0.646 [0.531, 0.785] in paired thread CPU; backward/full dense inverse was 0.611 [0.541, 0.690] before paying for cuts and output. The previous three-arm experiment is preserved separately. Its process-wide getrusage highs are sensitive to pre-exec memory inherited from the driver. The current panel reads post-exec VmHWM in dedicated fresh processes. [Four-arm results](interval-panel/summary.json) · [Previous three-arm data](first-panel/times.csv)

## The actual guarded route on original byte input

The ordinary cut already met the quarter threshold at every one of 251 passes across ten complete Wikidata blocks. It bypassed all 72 provisional nonzero-diagonal normalizations. The histories must be measured independently of eager normalization because taking a different cut changes later aliases. The guarded paths touched 2,916,661 old maximal runs, created 315,213 cut aliases and built 596,427 joins across the ten blocks. All 40,303,491 checked stable-partner relations passed. The native ending spellings matched the independently checked original marked byte output on every block. After RLE1 the 8,999,785 final bytes matched libbzip2. The source corpus SHA-256 is 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915. Each input tape, pre-RLE1 marked output, final output and block CRC has its own receipt. [All original checks](gated/python.jsonl) · [Whole native validation](gated/native-check.json) · [All 251 passes](gated/passes.csv)

The first frozen cut removed 297,151 rows directly. The complete path finished in 26 rounds with 47,434 joins. Its median of 23.132 ms includes 10.305 ms in graph work, 8.635 ms in cut publication and 1.750 ms in final expansion. Individual phase medians do not add exactly to whole-stage medians. The native dense full inverse control took 15.544 ms in this separate matched panel. The complete native guarded clocks start from original byte runs and the true mark, ending with the full inverse BWT bytes. Input parsing, Huffman/MTF, RLE1, CRC and verification are outside both native clocks. Oracle validation of the stored data includes RLE1 and the recovered block stream CRC. A multi-block production decoder and combined stream checks were not submitted to the scorer for this formulation. The published 0.5.6 target requires complete independent worker measurements. [Exact full native source](direct_cut.cpp) · [Frozen timing rules](GATE.md)

Future gains here would have to materially lower the 10.3 ms class-graph construction and 8.6 ms cut publication costs. Further tuning of power discovery cannot reduce either of those on the ten guarded histories, since it was bypassed in every one. The separate mathematical result supplies a bounded-run representation and a worst-case remedy on the inputs that actually need it.

## Reproduction

All runs use only the isolated research paths. For the full structural panel, Python's bz2 module uses libbzip2, and bzip2recover reconstructs independently decodable original blocks in a temporary directory. A fresh scratch path is necessary because raw files are created exclusively.

```bash
d=$(dirname "$(rg --files | rg 'quotient/fold/iterated/cut/powers/PROOF.md$')")
tmp=$(mktemp -d)
tapes=research/astra-e/dense/target/wikidata
python3 -B "$d/check.py" --max-rows 6 --backward | diff - "$d/interval-panel/small.json"
python3 -B "$d/gate_panel.py" "$tapes" corpora/wikidata.bz2 "$tmp/python" > "$tmp/gate.jsonl"
g++ -std=c++20 -O3 -g0 -Wall -Wextra "$d/direct_cut.cpp" -o "$tmp/full"
python3 -B "$d/verify_native.py" "$tmp/full" "$tapes" "$tmp/gate.jsonl" \
  "$d/gated/passes.csv" "$tmp/checked" > "$tmp/native.json"
taskset -c 1 nice -n 12 "$tmp/full" "$tmp/checked/0000/input.txt" panel 24 > "$tmp/full-times.csv"
taskset -c 1 nice -n 12 "$tmp/full" "$tmp/checked/0000/input.txt" gated 1
taskset -c 1 nice -n 12 "$tmp/full" "$tmp/checked/0000/input.txt" direct 1
python3 -B "$d/real.py" "$tapes/0000.irec.xz" corpora/wikidata.bz2 \
  --backward --differential --trace "$tmp/eager-depths.csv" > "$tmp/eager.json"
taskset -c 1 nice -n 10 python3 -B "$d/native.py" "$tapes/0000.irec.xz" "$tmp/eager.json" "$tmp/phase" > "$tmp/phase.json"
mkdir "$tmp/phase-summary"
cp "$tmp/phase/phase.csv" "$tmp/phase-summary/times.csv"
cp "$tmp/phase/"*-rss.csv "$tmp/phase-summary/"
python3 -B "$d/report.py" "$tmp/phase-summary"
```

The recorded summaries and raw pairs can be rechecked directly without new timings using the interval-panel directory with report.py. Interpreter CPU and VmHWM fields in the structural JSON include environment-specific measurements; compare deterministic hashes, counts, flags and per-depth rows when reproducing. [Native environment and hashes](interval-panel/manifest.json) · [Compressed and full native stage summaries](gated/summary.json)
