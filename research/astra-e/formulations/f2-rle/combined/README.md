# Combine sparse exception recognition with the independent endpoint leader

The graph continues to store the same thick strips, image order and thin queues used in the rank/select proof. The narrower node type changes endpoint node links only, and literal 2–3 byte handles change how the internal rule grammar stores short spellings. Both constructions output the same residual return permutation and spelling. Apply the incumbent guard for overflowing 16-bit endpoint nodes and fall back to its wide short-word induction. While the sparse writer processes a word, load the next valid residual edge before copying, including for the closing edge and a clipped final pass. When a recognition budget or tag constraint uses stock finishing, the candidate uses the same ahead-of-copy walker as the incumbent.

The four-arm experiment compares ordinary opend, narrow-short/ahead, bundled sparse windows on ordinary opend, and this synthesis. The same exact 228 density admissions and their 205,197,794 raw inverse bytes were covered by every arm, with final decoded equality and the stored CRC. Nine systematic admitted blocks had 32 measured rounds. Four forward and four reversed complete rotations balance each arm across all positions. A paired observation is the sum over those same nine blocks within one repetition. The stage includes construction from capped records through final CRC. It excludes the common entropy front, the slot queue and process overhead.

The first run's thread CPU ratio was 0.938 [0.909,0.967] against opend and 0.986 [0.959,1.014] against narrow-short/ahead. Wall time against the latter was 0.795 [0.659,0.958], while the incumbent/opend ratio disagreed between wall and thread CPU. This fixed-input comparison cannot establish a small production gain. [Raw rounds](phases.csv) · [Statistics](summary.txt) · [Compiler input digests](inputs.sha256)

For each arm the probe counts the entire inverse kernel's inline structure, every retained dynamically allocated vector including per-byte queues and both sizes of endpoint scratch, and each private intermediate coded and final decoded vector. The native sparse fallback's coded scratch is part of its kernel. On these admissions the observed high waters were 7,910,380 B for opend, 7,558,316 B for narrow-short/ahead, 7,082,246 B for the ordinary bundled writer, and 6,730,182 B combined. Shared frontend buffers, allocator slack beyond vector capacities, and worker/process RSS are excluded. The stage holds distinct per-arm buffers, checking outputs outside each timed run. [All per-block capacities](capacity.csv)

```sh
d=research/astra-e/formulations/f2-rle/combined
scratch=$(mktemp -d)
python3 "$d/prepare_stage.py" "$scratch" > "$scratch/inputs.sha256"
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/probe"
"$scratch/probe" corpora/wikidata.bz2 "$scratch/phases.csv" "$scratch/capacity.csv"
cp "$d/adversarial.rs" "$scratch/"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/adversarial.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/adversarial"
"$scratch/adversarial"
```
