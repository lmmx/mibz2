# Direct retained-row append screen

This follow-up keeps the original direct u32 destination and the vectorizable band projection. Single handles width one by Vec::push. Four writes each run in padded four-rank chunks, reserving any last unused slots and publishing just the true width. Hybrid combines singleton push with four-rank chunks for wider runs. The initialized contents and published lengths are identical. No allocator settings change. The reference per-row queue and the preserved narrow/short/ahead kernel remain separate arms.

Each process pinned one CPU and verified every complete output in 30 position-balanced passes on the original eleven admitted blocks. The hybrid's complete inverse thread-CPU ratios to ordinary rows were 0.9683 [0.9188,1.0205] on CPU 3 and 0.9657 [0.9427,0.9893] on CPU 0. Both wall-time intervals included parity. The other alternatives had CPU intervals crossing parity in both runs. This is a tentative inverse-stage lead only. The compact-queue experiment has closed.

From the same checkout as the queue-stage driver, use Rust 1.99.1, its release crate and the generated narrow/short/ahead source. The following runs the independently marked-column properties and both timed replicas.

```sh
d=research/astra-e/queue-spans/direct
scratch=$(mktemp -d)
python3 "$d/prepare.py" "$scratch"
for name in check stage; do
  rustc --edition=2021 -C opt-level=3 -C codegen-units=1 -C lto=fat \
    -L dependency=target/release/deps --extern mibz2=target/release/deps/libmibz2.rlib \
    "$scratch/$name.rs" -o "$scratch/$name"
done
"$scratch/check"
nice -n 12 taskset -c 3 "$scratch/stage" corpora/wikidata.bz2 30 > "$scratch/first.csv"
nice -n 12 taskset -c 0 "$scratch/stage" corpora/wikidata.bz2 30 > "$scratch/repeat.csv"
```
