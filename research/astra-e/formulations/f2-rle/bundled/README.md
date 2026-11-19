# Trace only the initial word cylinders

Let A(x) be the preceding retained source on x's Ψ cycle, on all components covered by the original Open section S. For a nonempty set H of event starts and D = H ∪ ΨH ∪ … ∪ Ψ⁴H, the set of affected whole return edges has the following exact description.

    A(D) = A(H) ∪ (D ∩ S)

Along a window of forward visits the current owning source changes precisely on entering a retained row. Each possible owner is therefore either the owner at the start or a retained position within the window. Every member of the right-hand side belongs to D's owner set. This applies to overlapping windows and short cycles. Components disjoint from S have no owners and cannot contribute to the marked output. The constructor recovers D∩S by one monotone sweep of sorted danger rows and the old thick strips. It only rewinds the starts H.

H is a union of at most 256 FM intervals. To label their leftmost remaining source a, carry its remaining interval width w and trace x=a by LF. Whenever x is in a thick strip reduce w to the strip's right slack. Each of the w consecutive sources has the same translated itinerary by induction over the steps. At the first retained gap truncate w to its right slack and append the contiguous range of w owners. A return to a before reaching a gap is a closed cycle disjoint from S, uniformly for that width. Continue at a+w. The mark remains independently retained in S. The proof certifies each entire translated interval; it makes no timing assumption about batch width or output volume.

The native implementation passed all 18,434 arbitrary marked cases and checked every valid real output and all qualifying FM events against the independent full-run index. On 228 density admissions, 227 completed sparse reconstruction, with only one rejecting the event-count cap. All 66 previous owner-budget fallbacks disappeared. Total accounted thick-strip translations fell from 296,275 (already stopped after 4,097 on any old block) to 46,241. On frozen all 34 starts are retained and zero rewinds suffice, with exactly the same 95 tagged edges. Stream 134 needs 829 bundled steps and stream 397 needs 971. Across 21 paired, rotated repetitions the CPU ratios were 0.968 [0.901,1.041] vs opend and 0.941 [0.866,1.023] vs the previous solver. The wall ratios were 1.112 [0.932,1.327] and 1.014 [0.825,1.246]. These fixed-input stage results resolve the redundant owner work; the small elapsed-time effects remain uncertain. [Outcomes](native.csv) · [All samples](phases.csv) · [Ratios](summary.txt)

```sh
d=research/astra-e/formulations/f2-rle/bundled
scratch=$(mktemp -d)
python3 "$d/price.py" "$scratch"
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/probe"
"$scratch/probe" small
"$scratch/probe" corpora/wikidata.bz2 "$scratch/phases.csv"
bash research/astra-e/score.sh bundled-rle-repeat "$d/candidate.rs" --workers 1,4 --rounds 9 --repeat 2 --settle 2 --with-opend --no-proxy
```
