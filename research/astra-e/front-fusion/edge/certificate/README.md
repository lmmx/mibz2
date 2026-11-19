# Certifying the dense edge route before reconstructing marked run geometry

Let S be the sum of the widths of maximal L-runs of width ≤16, and H the number of wider maximal runs, before splitting on the mark. On valid MTF input each nonzero-rank literal selects a different byte from the current head. Its true width, with t following run symbols of little-endian RUNB value u, is 2ᵗ+u. At the opening it is 2ᵗ−1+u. Thus S and H are determined entirely by the literal and RUNB bitmaps, independently of all literal ranks and the alphabet permutation.

For each literal, the first subsequent literal or EOB at distance d=t+1 determines its span. For d≤4, add 2ᵗ times the population of these starts, plus the populations of their masked RUNB bit planes with weights 1,2,4. For d=5 only the all-RUNA span contributes to S, with width 16. All remaining nonopening literals contribute one to H. A two-word read accommodates every span across a machine-word boundary. The opening term is handled separately. The result takes one pass over bitmap words, constant scratch and a bounded number of operations per word for this fixed cutoff. For another integer cutoff K, compare 2ᵗ+u to K on the last potentially thin level and account for at most floor(log₂ K) run symbols per start.

After splitting, m=S and h=H for a thin marked run. A thick marked run contributes one retained row. Its remaining fragments give h=H at an endpoint or h=H+1 in the interior. Consequently the edge charge lies between charge(S,H) and charge(S+1,H+1), by monotonicity of both the link width and label bound. If both endpoints of this interval yield the same admission, the marked run need never be identified. The two explicit constructions in witness.rs show why ambiguity can matter. For one pair of marks the retained size crosses 1 MiB by one row. For another the width switches from four to eight bytes. Both require the original exact marked decision at the boundary. [Witness outcomes](witness.txt)

For a stock worker that has already emitted capped records, an equivalent test reads each record once. Consecutive cap records have the same high byte exactly when they describe the same maximal run. At the first record, its length is min(127, true width), enough to classify width ≤16 and recover its exact thin contribution. Summing a vector of first-record flags times length and counting the first heavy records gives the identical unmarked S,H. Adjacent vector loads compare sixteen records at a time with no mark or variable-length coalescing dependency. Either certificate preserves stock fallback on ambiguous intervals. The record certificate uses the existing buffer and needs no access to the stock front's private bitmaps.

All 383 real run-route blocks matched the independently coalesced, marked reference using both bitmap and capped-record calculations. Seven real primaries changed the retained mass and six changed the strip count. The charge interval decided all 383; the existing edge gate only pays for the five dense blocks of at least 65,536 rows. Their 737,585 total cap records correspond to 15,341 bitmap words. Their exact upper charges were 926404, 920140, 884560, 796760 and 805676 bytes. Exhaustive checks cover every bit alignment, including EOB, opening runs and all RUNA/B patterns through length 12. [All real inputs](real.csv) · [524,224 exhaustive checks](small.txt) · [Independent oracle](audit.rs) · [Bitmap certificate](geometry.rs) · [AVX2 record certificate](simd.rs)

```sh
d=research/astra-e/front-fusion/edge/certificate
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc -O --edition 2021 "$d/audit.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/audit"
"$scratch/audit" small
"$scratch/audit" corpora/wikidata.bz2 > "$scratch/real.csv"
cmp "$scratch/real.csv" "$d/real.csv"
rustc -O --edition 2021 "$d/witness.rs" -o "$scratch/witness"
"$scratch/witness"
```
