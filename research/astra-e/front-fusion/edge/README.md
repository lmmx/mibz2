# Direct maximal-run front with the independently measured 1 MiB edge admission

The frozen control is edge-budget at `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. It admits the original sparse route, and additionally dense blocks with n≥65,536 whose width-certified residual table is at most 1 MiB. Its dense gate scans the actual cap records and computes retained m and post-primary thick-fragment count h. Both fused variants collect these same values naturally as the front feeds geometry. The eager variant also calculates exact virtual cap count from the bitmaps up front. The conditional variant first asks whether the block passes the large edge gate. On success its OR-policy answer is settled regardless of cap density; it entirely omits the cap calculation. For the remaining blocks it consults the retained symbol bitmaps and applies the exact original density alternative. A partial nonfitting block may delegate before finishing the speculative geometry; the stock edge worker then determines the final result.

**Exactness and bounds.** The packed label upper bound is 511+3h. The link field consumes max(1,bit_length(m−1)) bits. Comparing the label bound with the remaining field determines the 4- or 8-byte charge. The fused geometry classifies the complete run and then splits the marked thick run with identical fragment semantics. Queue occupancy m is monotone as runs arrive. At a bitmap transition, m>262,144 proves that even the four-byte edge table cannot fit; abandoning speculation and replaying the stock control also preserves any *density*-admitted input that would otherwise be lost by treating this as a final rejection. Between checks ≤64 runs can add ≤1,024 thin rows. Vector capacities can exceed live widths. This bounds speculative work on nonqualifying text without predicting a small process RSS.

**All 431 real blocks matched** the frozen edge control's raw marked inverse, decoded bytes and block CRC. All 233 large blocks fit the edge budget, including all five newly admitted at the i9 milestone. This allowed conditional counting to complete all 233 without evaluating cap density or materializing cap records. The other 198 used the ordinary routes. The scalar/detected adversarial differential agreed on all 11,200 comparisons. Thirty additional dense cases exercised the 1 MiB boundary, the 4/8-byte field transition and marks at strip interiors/ends: 13 qualified and 17 delegated. [Corpus observations](checked.csv) · [Error outcomes](adversarial.txt) · [Edge-boundary tests](boundaries.txt)

The 24 paired rotations ran on CPU 3 at niceness 10, with parsed symbols as input and the route choice, front, inverse and RLE1 inside thread CPU. Every round included all five changed dense routes, nine ordinary systematic admissions, all 150 short run blocks and all 48 column blocks. Relative to frozen edge, deferred counting was 0.988 [0.910,1.072] on the five, 1.041 [0.976,1.110] on the nine, and 1.038 [0.977,1.104] after weighting the nine by 228/9. The eager variant was 1.063 [0.906,1.248] on the five and 1.103 [1.017,1.197] weighted. The recombined CPU results are unresolved for conditional counting and unfavorable for eager counting; neither advances the verified speed reference. [All paired observations](panel.csv) · [All strata and controls](panel.txt)

```sh
d=research/astra-e/front-fusion/edge
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
python3 "$d/prepare.py" "$scratch/source"
rustc -O --edition 2021 -A unused -C codegen-units=1 "$scratch/source/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/native"
"$scratch/native" adversarial
nice -n 10 taskset -c 3 "$scratch/native" corpora/wikidata.bz2 24 > "$scratch/panel.csv"
python3 "$d/summarize.py" "$scratch/panel.csv"
```
