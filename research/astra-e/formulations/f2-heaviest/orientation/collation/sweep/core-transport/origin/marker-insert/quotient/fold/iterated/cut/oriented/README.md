# Either side of a delimiter cut can carry the primary

Fix a cut T of the old word types and a frequency-one anchor a. If a is outside T it survives as its unique refined outgoing type. If a is inside and its incoming old predecessor k is inside too, the anchor survives. In the remaining case the source subclass (k,a) has frequency one since only one row belongs to the target bucket a. Its surviving occurrence spells Wₖ Wₐ. Move the anchor there and add ∣Wₖ∣ to the previous intra-word offset. That sum remains strictly less than the new word length. This covers all possibilities, preserves the marked orbit at every requested horizon and permits either orientation. The old number of deleted rows is identical because ingress and egress have equal mass.

Let dᵢ be the number of distinct destinations for source i, and bᵢ the number of original F-bucket boundaries in the interior of an L-run of type i. For the original full-by-destination refinement, the exact new type count is K + Σᵢ∉T(dᵢ−1), and the exact refined incoming run count is R + Σᵢ∉T bᵢ. Old run boundaries always survive in the refinement. An old source-bucket boundary within a run changes its alias exactly when that run's type is outside T. The two orientations' type counts sum to K + Σdᵢ. Their refined run counts sum to 2R + Σbᵢ. The cheaper orientation can therefore always achieve at most ⌊(K+Σdᵢ)/2⌋ types. The orientation minimizing run count can instead achieve at most R + ⌊Σbᵢ/2⌋ runs. The selected lexicographic criterion prioritizes type count. These formulas precede composition, so all post-composition costs are measured separately.

The exhaustive set covers 27,675 marked weighted reconstructions across ordinary, flipped and lower-type orientations, including 17,457 nondividing horizons. It moved the marked row 13,586 times and verified every surviving partner against a dense independently built map. On ten original-byte columns both complete schedules passed recovered libbzip2 output and all ten block CRCs. The lower-type schedule flipped 51 of 250 cuts and moved the anchor 40 times. Summed refined IDs fell from 1,222,263 to 1,164,273 (4.74%), joins from 596,427 to 572,038 (4.09%), and refined runs from 2,974,603 to 2,957,606 (0.57%). The two complete schedules needed 251 and 250 rounds. Only 10,440 of the 57,990 fewer IDs are the immediate within-round orientation difference along the changed history. Following the new induced column changes later graphs. These interpreter measurements establish exact structure. [All phases](passes.csv) · [Independent CRCs and totals](real.jsonl) · [Tiny exhaustive receipt](small.json)

```bash
d=$(dirname "$(rg --files | rg '/cut/oriented/PLAN.md$')")
python3 -B "$d/panel.py" small | diff - "$d/small.json"
tmp=$(mktemp -d)
python3 -B "$d/panel.py" real --passes "$tmp/passes.csv" > "$tmp/real.jsonl"
diff "$tmp/passes.csv" "$d/passes.csv"
diff "$tmp/real.jsonl" "$d/real.jsonl"
```
