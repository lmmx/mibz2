# Fully decoded pages for straight open-return tracing

The 431-block route census puts 4,508,379 of 209,700,196 final bytes on the density-refused run route and 465 on the column route. The dense run blocks account for 0.014329 s of 0.648943 s of summed historical Lazy kernel medians. Those separate single-thread measurements concern inversion only. Optimising the dense route in isolation has limited scope on this workload. An earlier full-table pair walk increased Wikidata CI wall time by 40.6% at one thread and 24.3% at four against 0.1.4. Sources are the [route census](https://github.com/lmmx/mibz2/blob/4f7d0235025940bd3d744463152515d189eb13dd/docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv), [phase observations](https://github.com/lmmx/mibz2/blob/2b53453e124ffdc75cc4d2141b833a319e3cfd73/docs/journal/2026-11-18-open-return/evidence/phases-wikidata.csv) and [pair-walk study](https://github.com/lmmx/mibz2/blob/f398966ed88301609c543c4df47fb46379c1c783/docs/journal/2026-10-27-pair-walk.md).

The experimental candidate targets all width-16 open-eligible blocks. It uses the straight tracer from the open-return study. A homogeneous 32-row page carries its entire tile descriptor in eight bytes. The descriptor holds the end, displacement and symbol. A mixed page instead holds the index of its first tile and the tracer searches forward. When a trace reaches a terminal tile, the constructor wires the newest band to its slice of the relevant byte's queue immediately. This eliminates the temporary list of all bands. The inherited admission, mark splitting, whole-cycle repetition and column fallback are as in opend. All indexing and copying use safe Rust.

For each page wholly inside a tile, the direct descriptor is exactly the tile a search would find. Its end is the tile end, including when that is beyond the page. Taking the minimum remaining width along each trace finds the interval on which every advance and terminal result agree. Its exit ranks are consecutive. LF stable sorting maps the next consecutive subrange of the queue for that byte to the entrances just traced. Inverting these edges immediately writes each retained rank once and assigns the same word as deferred wiring. A trace starting at the LF image of a retained row must return to a retained row by bijectivity. A component entirely in the removed set requires no trace.

On the frozen block the tracer makes 50,709 wide steps and 3,204 terminal probes. Of these 53,913 tile lookups, 44,401 are direct; the rest use the mixed path. The page directory costs 224,984 bytes. Across ten archived large Wikidata blocks, 426,414 of 524,517 lookup sites are direct. The paid work also includes the initial scans and queues, constructing all 281,248 page descriptors across those ten blocks, the mixed-page tile loads, reversing each nonempty traced word, wiring retained rows and interpreting the final marked cycle. The competing endpoint constructor makes only 11,215 cuts on frozen. These structural counts do not establish comparative performance.

Reproduce from the repository root with the scorer already built. The architecture and compiler must match the scorer rlib. The standalone source checker uses stable row sorting as an independent marked inverse. It forces the tracer at widths 0, 1, 3 and 16 on arbitrary small columns, including short cycles. The ten larger archives and three synthesised run-heavy columns use width 16. The scorer adds the stream, CRC and malformed-input checks.

```sh
d=research/astra-e/dense
mkdir -p "$d/target"
data=2e4b34e086ac95e8218f1ee822a4652c8db5bb19
git fetch --no-tags origin "$data"
git archive "$data" wikidata | tar -x -C "$d/target"
python3 "$d/routes.py"
python3 "$d/screen.py" "$d/target/wikidata"
deps=bench/moonshot/target/release/deps
rustc --edition=2021 -O --extern "mibz2=$deps/libmibz2.rlib" -L "dependency=$deps" "$d/check.rs" -o "$d/target/check"
python3 "$d/check.py" "$d/target/wikidata" "$d/target" "$d/target/check"
```

The next distinct construction can store two LF steps in a homogeneous page when the whole page translates into a single next tile. The admissible range must use the smaller of the two right slacks. Both wide bytes, or the first byte and terminal tag, must be retained. Measure the actual missed tile lookups and the cost of building these composed descriptors before implementing the walker.

## Compose the two-step partition once

The page-only follow-up at 32 rows would spend 216,270 setup target lookups to remove 206,772 dependent probes on the ten blocks. A source-order refinement yields a better construction. For each wide tile, intersect its LF image with the one-step target tiles. The image starts for each byte increase. One cursor per byte therefore suffices. The 256 starting cursors are found by scanning the row boundaries once. Across bytes an original boundary can cross the interior of at most one wide image because the images are disjoint. For T original tiles, the refined list contains at most 2T−1 entries. The additional cursor movement costs O(T+256), including transitions through portions of the F buckets omitted by the retained rows.

Each refined tile finishes one of three ways. An original terminal has an empty word and points to retained coordinates. A wide tile followed by a terminal carries the wide byte and finishes at a retained rank. Two wide tiles carry both bytes in traversal order and point to the original row after both moves. Intersecting the source interval with the translated-back target ensures that the end is the exact two-move right bound. The additive displacement modulo 2²⁰ is exact because the true destination lies in [0,900000). A 64-bit entry holds that displacement, the end, both byte slots and the length. The compiler retains the original and refined arrays, and the refined page directory identifies homogeneous or mixed pages as before. Other construction and writing stages use the earlier implementation.

| Ten systematic archives, width 16 | One-step | Refined two-step |
| --- | ---: | ---: |
| Stored tile entries in the lookup partition | 56,046 | 71,192 |
| Dependent page probes during tracing | 524,517 | 275,562 |
| Probes requiring a mixed-page tile search | 98,103 | 73,032 |
| Return bands | 35,953 | 35,953 |

The refinement makes 55,567 target cursor advances after the initial bucket scan. On frozen its additional tile tape requires 58,024 logical bytes. The census checks the same exit and width for each actual traced band. The native candidate matches independent inversion on all 56,836 small forced trials and 11,699,915 larger input rows. The implementation uses only safe Rust. Measure complete decode time and peak RSS to price the extra preparation, descriptor footprint, bit extraction and spelling work. Routine commands for this variant are below.

```sh
python3 "$d/compose.py" "$d/target/wikidata"
python3 "$d/two_counts.py" "$d/target/wikidata"
rustc --cfg two_step --edition=2021 -O --extern "mibz2=$deps/libmibz2.rlib" -L "dependency=$deps" "$d/check.rs" -o "$d/target/check2"
python3 "$d/check.py" "$d/target/wikidata" "$d/target" "$d/target/check2"
```

## How far to double

A k-hop tile either finishes at a retained rank before consuming k wide bytes, or emits exactly k wide bytes and reaches another original row. Keep the former. For each latter tile intersect its translated image with the entire k-hop partition, concatenating spellings and adding displacements. All continuing images are disjoint by injectivity of LF raised to k, giving at most 2T−1 new tiles from T old. The census checks every width, exit and full spelling against the one-step tracer on each marked input. The search and write columns below charge *cumulative* setup. A search locates the destination of a continuing source before intersecting intervals. The Python census uses binary search. The Rust two-step builder uses the per-byte monotone sweep priced above.

| Length k | Final tiles | Cumulative compiled descriptor writes | Cumulative continuing source searches | Traced page probes | Mixed-page probes |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 56,046 | 0 | 0 | 524,517 | 98,103 |
| 2 | 71,192 | 71,192 | 32,475 | 275,562 | 73,032 |
| 4 | 89,847 | 161,039 | 71,747 | 152,210 | 57,341 |
| 8 | 112,661 | 273,700 | 118,290 | 91,092 | 47,270 |
| 16 | 139,097 | 412,797 | 171,225 | 61,356 | 41,210 |
| 32 | 172,497 | 585,294 | 229,454 | 47,153 | 37,935 |
| 64 | 209,500 | 794,794 | 291,737 | 40,342 | 35,784 |

All rows cover the same ten input archives and exactly 35,953 return bands. Four hops remove 123,352 additional page probes and require 89,847 additional compiled writes. Eight hops remove 61,118 further probes and require 112,661 further writes. The table omits the additional bytes per wider descriptor, spelling stores and page-directory construction, each of which must be priced in a native prototype. It also assigns no time equivalence to a dependent probe and a setup store. Four hops is the next structural candidate if two hops pays for its construction on hardware. Reproduce with `python3 "$d/doubling.py" "$d/target/wikidata"`.
