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

All 120 fresh processes completed and validated their 419,400,392 returned bytes through every block CRC. Wall ratios against the frozen edge control were 1.0326 [0.9972,1.0692] for the original in-place source, 1.0243 [0.9961,1.0533] for the in-place-specific source, and 1.0069 [0.9851,1.0291] deferred. The identical edge binary gave 1.0132 [0.9823,1.0450] against itself. The deferred CPU ratio was 1.0073 [0.9852,1.0298]. Median process high water fell from 18,684 KiB frozen to 18,382–18,384 KiB for all three compact treatments. The previously observed seven percent timing advantage failed to reproduce across full input and fixed process binaries. The preregistered complete-worker speed screen failed. The representation remains an exact fixed-storage result.

Five-second samples of live processes covered 158 seconds. CPU2 included 503 ticks of C++ compilation, sampled Python processes accounted for 93 ticks last observed on CPU2, and CPU3 had 10 Python and seven dockerd ticks outside the paid binaries. Last-observed CPU is a sample, and short-lived processes can start and finish between polls. [All 120 samples](runs.csv) · [Paired replay](summary.txt) · [Observed co-runners](co-runners.csv)
