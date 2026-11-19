# Explicit marked output owns both levels' complete spellings

The predecessor stores first words in the final second dictionary. This construction instead records both levels' spellings in the output itself. Both tries still exist. Each completed first word retains its leaf and byte depth; each completed second word retains its leaf and the sum of the first-word byte lengths along its trie path. The second token BWT, first-column ranks, packed token links and transported mark are unchanged. Neither construction stage needs a contiguous first or second byte dictionary on an accepted two-level path.

**Output invariant.** An uncached descriptor points to its terminal node. A cached descriptor contains the integer offset of a complete, earlier occurrence of the same word in the emitted prefix. At a complete uncached second word, follow its nodes to the root, spelling their reversed edge segments of first-level tokens. Handle each complete first token by the same rule using byte edge segments. Only after all of a word's bytes have been emitted is its starting offset a reusable full owner. For a cached word the indicated output interval is already initialized, and copying it to the end with `extend_from_within` preserves the invariant. Storing offsets handles reallocation.

The weight passed through both LF mark projections is measured in original bytes. Therefore the initial returned second word can begin in its middle, and the final word can end inside an arbitrary first word. On a partial second return, skip whole first tokens until the first intersecting token and clip at the final requested byte. A cached first word can supply the requested slice. An uncached, partially visible first word emits only the intersection with its reversed edge segments. Neither partially emitted word becomes a full owner. Fully emitted first tokens inside a partial second word are eligible for ownership. There are at most two clipped boundaries in the entire marked output, with possible coincidence. A covered short cycle repeats until n bytes have been delivered, using exactly the same bounds and ownership invariant. If second induction declines, the predecessor's one-level materialization remains the fallback. [Full program](candidate.rs) · [Clipped first words and marked writer](second.rs)

## Verification and cost

An exhaustive test over every primary of binary columns through length 10 and ternary columns through length 7 exercised 21,409 actual two-layer marks, 10,263 exact one-layer fallbacks and 8,086 section-free refusals. Of the two-layer successes, 8,784 required clipping an uncached first word. Every byte and every coverage outcome agreed with a dense stable-sort inverse oracle. [Small marked receipts](partial.log)

All 228 admitted large Wikidata blocks matched Segmented. Standalone first/second byte dictionaries were empty on every two-level admission. The algorithm still retains both compressed tries, the second token row table, the output itself and any inherited fallback capacities. Before this change F2c's contiguous dictionaries totalled 23,393,313 and 41,433,719 byte writes across these admissions. The new decoder delivered the 205,197,794 marked bytes as follows.

| Source of emitted bytes | All admissions | Frozen |
|---|---:|---:|
| Earlier full second word in output | 163,764,074 | 753,748 |
| Earlier full first word in output | 18,040,399 | 62,729 |
| Full first trie spelling | 23,388,559 | 83,454 |
| Partial first trie spelling | 4,762 | 0 |

Earlier output now supplies potentially long-distance copy reads; the algorithm still visits the first-token spans inside uncached return words. It created 805,808 complete second-word owners over the admitted corpus. Frozen needed 2,949, while the full second dictionary contains 2,950 distinct words. A word straddling the marked boundary can have no complete occurrence in the retained output. [All native observations](geometry.csv)

A 24-round four-arm phase test measured the same batched discovery schedule in full-dictionary F2c, one-dictionary, and output-owned variants against the LazyCells inverse. On the nine admitted systematic blocks, output-owned/batched was 1.1990 [1.0815,1.3293], output-owned/one-dictionary was 1.0270 [0.9192,1.1475], and output-owned/Lazy was 1.1421 [0.9744,1.3385]. The host produced large per-phase variation. These kernel timings and exact dictionary counts have separate scopes; process high-water RSS also includes reused buffers and worker slots. [Raw phases](phase-cpu.csv) · [Paired summaries](phase-summary.txt)

```sh
study=research/astra-e/formulations/f2d
scratch=$(mktemp -d)
cargo build --locked --release --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/base"
deps="$scratch/base/release/deps"
rustc --edition=2021 -O --test "$study/output-owned/property.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/property"
"$scratch/property" candidate::checks::ownership_on_arbitrary_marked_cycles --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV="$scratch/geometry.csv" "$scratch/property" candidate::checks::spelling_ownership --ignored --nocapture
mkdir -p "$scratch/price"
bash "$study/output-owned/price.sh" "$scratch/price" "$deps" > "$scratch/price/phase-cpu.csv"
python3 "$study/summary.py" "$scratch/price"
```
