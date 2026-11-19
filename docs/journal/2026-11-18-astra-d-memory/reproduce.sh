#!/usr/bin/env bash
# Run from this journal's checkout. Supply an empty scratch directory.
set -euo pipefail
study=$(cd -- "$(dirname -- "$0")" && pwd)
repo=$(git -C "$study" rev-parse --show-toplevel)
mkdir -p "$1"
scratch=$(cd -- "$1" && pwd)
pin=1edb29fdc3d42dfddfc3d60e46456b611323760c
data=2e4b34e086ac95e8218f1ee822a4652c8db5bb19
windows=a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a
export RUSTUP_TOOLCHAIN=1.90.0
mkdir -p "$scratch"/{data,bin,verify,evidence}
git -C "$repo" fetch --no-tags origin "$data" "$windows"
git -C "$repo" archive "$data" wikidata | tar -x -C "$scratch/data"
git -C "$repo" archive "$windows" windows | tar -x -C "$scratch/data"
g++ -O2 -std=c++17 -Wall -Wextra "$study/capture.cc" -o "$scratch/bin/capture"
g++ -O3 -std=c++17 -Wall -Wextra "$study/replay.cc" -o "$scratch/bin/replay"
"$scratch/bin/replay"
for arm in stock direct tiles; do
    tree="$scratch/$arm-tree"
    out="$scratch/verify/$arm"
    mkdir -p "$out"
    git -C "$repo" worktree add --detach "$tree" "$pin"
    cd "$tree"
    # Each build target belongs to exactly one source tree, including test binaries.
    export CARGO_TARGET_DIR="$scratch/$arm-target"
    if test "$arm" = stock; then
        cargo build --release --locked --example windows
        python3 bench/wikidata.py build bench/wikidata.json "$scratch/data/windows/wikidata-20261019" \
            "$CARGO_TARGET_DIR/release/examples/windows" "$scratch/data/wikidata.bz2"
        python3 "$study/extract.py" "$scratch/data/wikidata.bz2" "$scratch/data/wikidata" \
            "$scratch/data/blocks" > "$scratch/inputs.csv"
        cmp "$study/inputs.csv" "$scratch/inputs.csv"
    fi
    python3 "$study/install.py" "$tree"
    if test "$arm" = direct; then python3 "$study/candidate.py" "$tree"; fi
    if test "$arm" = tiles; then python3 "$study/tile_install.py" "$tree"; fi
    cargo rustc --release --locked --example astra_trace -- -C link-arg=-no-pie
    cp "$CARGO_TARGET_DIR/release/examples/astra_trace" "$scratch/bin/$arm"
    python3 "$study/collect.py" "$(command -v valgrind)" "$scratch/bin/$arm" "$scratch/bin/capture" \
        "$scratch/data/blocks" "$scratch/$arm-runs"
    python3 "$study/summarise.py" "$scratch/bin/replay" "$scratch/$arm-runs" "$scratch/evidence" "$arm"
    python3 "$study/validation.py" "$tree" "$arm"
    rustc --crate-type rlib --edition=2021 --crate-name mibz2 -C opt-level=3 src/lib.rs -o "$out/libmibz2.rlib"
    rustc --edition=2021 --extern "mibz2=$out/libmibz2.rlib" -C opt-level=3 examples/astra_corpus.rs -o "$out/corpus"
    rustc --edition=2021 --extern "mibz2=$out/libmibz2.rlib" -C opt-level=3 "$study/pool.rs" -o "$out/pool"
    extra=()
    if test "$arm" = stock; then extra=(save); fi
    "$out/corpus" "$scratch/data/wikidata.bz2" "$out/output" "$scratch/verify" "${extra[@]}" > "$out/rows.csv"
    "$out/pool" "$scratch/data/wikidata.bz2" > "$out/pool.csv"
    if test "$arm" != stock; then cargo test --release --locked --lib astra_ > "$out/tests"; fi
done
python3 "$study/screen_finish.py" "$scratch/stock-runs" > "$scratch/edits.csv"
python3 "$study/tiles.py" "$scratch/data/wikidata" > "$scratch/tile-screen.csv"
cmp "$study/edits.csv" "$scratch/edits.csv"
cmp "$study/tile-screen.csv" "$scratch/tile-screen.csv"
python3 "$study/dependencies.py" "$scratch/data/wikidata" "$scratch/stock-runs" "$scratch/verify" "$scratch/evidence"
python3 "$study/attest.py" "$scratch/data/wikidata.bz2" "$scratch/verify" "$scratch/evidence" \
    "$scratch/stock-runs" "$scratch/direct-runs" "$scratch/tiles-runs"
python3 "$study/crc_checks.py" "$scratch/data/blocks/001.bz2" "$scratch/bin/"{stock,direct,tiles} > "$scratch/evidence/crc-checks.txt"
python3 "$study/report.py" "$scratch/evidence"
