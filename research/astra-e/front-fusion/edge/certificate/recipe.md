# Gate CPU and corpus contribution

The five large dense blocks are the full set on which the frozen edge policy scans records for admission. Measure four functions on these five blocks in 32 paired rotations, four repetitions per sample. Arms 0 and 1 invoke the identical frozen noninlined marked record walk. Arm 2 uses AVX2 to classify first capped records and falls back to the marked walk on an ambiguous charge. Arm 3 applies the same decision using the preexisting symbol masks. Its timed region starts with valid masks; supplying valid masks to a stock scalar worker would require separate work or the ordinary fallback. The AVX2 record method starts from stock capped records and needs no extra representation.

Use CLOCK_THREAD_CPUTIME_ID, CPU 3 and niceness 10. Each measured invocation reruns the whole certificate, including exact fallback if the envelope straddles the threshold. Preserve individual rounds and unrounded values. Report paired log means with Student 95% intervals. Interpret arms 0 versus 1 as an empirical check of close results. The warm isolated gate comparison measures the stages alone.

Three additional full sequential passes decode the complete Wikidata corpus from in-memory compressed bytes, including scan/Huffman decoding, stock run front, ordinary/fallback inverse, RLE1 and each block CRC. Record the time spent in the original edge selector at its natural position during those decoded blocks. Each pass must return the independently checked 209,700,196 bytes over 431 blocks. The observed fraction of total thread CPU sets the maximum possible saving from eliminating that selector on this single-worker execution, conditional on all other costs staying fixed. Worker overlap, cache side effects and i9 timings are independent questions.

```sh
d=research/astra-e/front-fusion/edge/certificate
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
python3 "$d/panel.py" "$scratch/source"
rustc -O --edition 2021 -A unused -C codegen-units=1 "$scratch/source/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/panel"
nice -n 10 taskset -c 3 "$scratch/panel" corpora/wikidata.bz2 > "$d/panel.csv" 2> "$d/full.csv"
python3 "$d/summarize.py"
```
