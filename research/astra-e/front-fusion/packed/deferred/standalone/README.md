# Preregistered whole-corpus serial comparison

Four fixed binaries decode two fresh full copies of the compressed Wikidata corpus per sample, yielding 419,400,392 bytes over 862 returned blocks and validated block CRCs. Each process loads the compressed file once before starting the wall and thread-CPU clocks. It reuses the scanner's symbol Vec and the worker's scratch across blocks and copies. Unlike the previous stage panel, this requires no symbol cloning or resetting between measurements. This single-thread sequential scanner and worker does not model the parallel scanner/worker pipeline on the i9.

The comparator rotates five labels for 24 rounds on nice CPU3. Two labels execute the very same frozen-edge binary. The other three use, respectively, the first in-place source that also supported separate output, the later in-place-specific source, and deferred n/admission. All have fixed treatment at compile time. The comparison is designed to distinguish persistent ownership/algorithm effects from the contradictory mixed-binary local results. A local CPU or wall advantage requires a paired 95% log interval under one against edge together with a same-code interval containing one. Report all treatments, both clocks and whole-process peak RSS. These are local serial results. Any performance promotion requires independent complete-worker CI or i9 evidence.

```sh
d=research/astra-e/front-fusion/packed/deferred/standalone
scratch=$(mktemp -d)
cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
python3 "$d/prepare.py" "$scratch/bin" "$scratch/build/release/deps"
python3 "$d/paired.py" run "$scratch/bin" corpora/wikidata.bz2 "$scratch/runs.csv"
python3 "$d/paired.py" summary "$scratch/runs.csv"
```
