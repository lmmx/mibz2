# Direct old-column rank stream

Build the marker quotient's complementary index and root children without publishing its token BWT or storing one event per hazard run. The first old-root scan determines word counts, old bases, coincident bucket, selected mass and marked source owner. The transformed histogram is the same except for the chosen class label, whose count becomes the entire selected mass. All bases after its old bucket shift by the added row count. A selected suffix mark can require another ordered scan to find its old occurrence. Update dictionary slices for selected noncoincident old words; the marker reuses its entire existing spelling.

Two monotone cursors on the old compressed root then form a lazy stream of (word,width). One supplies the ordinary substituted column. The other advances until a noncoincident selected run, saving only the next event's word, width, and the number of preceding coincident occurrences. This places the event at the correct gap in the ordinary background. If several events occupy that gap, they are yielded in chronological order before resuming the background. Every background run is consumed once by each cursor, and any event splits at most one background run. The iterator has O(1) working words in addition to the ordinary O(K) type flags and histograms. The new decoder learns the true heaviest label before indexing, including nonmarker winners. It streams that label's root and marks, retains only complementary rank records, and decodes weighted words exactly. The no-Z, identity, disjoint and bit-budget fallbacks retain their explicit or marked-byte paths. The unfused column is materialized only in the untimed inspect mode to check its exact RLE and spellings.

Compare named original, explicit quotient h4, explicit quotient h4 with streamed root, direct h4, explicit quotient h8, direct h8. Use the six-arm 60-round ten-cluster rotor, paid cap-to-byte output, and independent little/dense adversarial oracle from the native control. Assess direct h4 against each of its direct controls and named, and direct h8 against explicit h8. Use the direct-native index+discovery total including old-root census, word classification and iterator work; the full cap-to-byte timer also includes all mark/fallback/preparation work. Repeated fresh memory processes verify 228 output hashes independently. Ratios from this single-core phase experiment do not establish a production worker speedup.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/virtual/direct
out=/tmp/astra-virtual-direct
python3 -B "$d/build.py" "$out" "$PWD/target/release/deps"
"$out/direct"
"$out/direct" geometry /tmp/astra-clipped/ten.bin
"$out/direct" admissions /tmp/astra-token-all/admissions.bin
nice -n 12 taskset -c 3 "$out/direct" price /tmp/astra-clipped/ten.bin > "$out/phase.csv"
python3 "$d/../native/report.py" "$out"
```
