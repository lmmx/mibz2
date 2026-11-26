#!/usr/bin/env bash
set -euo pipefail
build=${1:?new scratch directory}
corpus=$(realpath "${2:?canonical Wikidata}")
generated=$(realpath "${3:?frozen generated singletiny.rs}")
out=$(realpath -m "${4:?receipt directory}")
repo=$(git rev-parse --show-toplevel)
dir=$repo/research/astra-e/huffman-phrases/integrated
pin=d0a42980e896211906234c7679b38760ae44a46e
worker=research/astra-e/front-fusion/cap-batch/worker/candidate.rs
[[ $(sha256sum "$corpus" | cut -d' ' -f1) == 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915 ]]
mkdir -p "$build" "$out"
build=$(realpath "$build")
export CARGO_BUILD_JOBS=1 RUSTUP_TOOLCHAIN=1.99.1 PYTHONDONTWRITEBYTECODE=1
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
for arm in stock phrase; do
    tree=$build/$arm
    git -C "$repo" worktree add -q --detach "$tree" "$pin"
    cd "$tree"
    cat bench/moonshot/README.md > /dev/null
    git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
    {
        git rev-parse origin/master
        git log -1 --format=%h origin/master -- bench/moonshot
        git log -1 --format=%H origin/master -- bench/moonshot
        git rev-parse origin/master:bench/moonshot
        git rev-parse HEAD
    } > "$out/refresh-$arm.txt"
    bash research/astra-e/prepare.sh "$worker"
    if [[ $arm == phrase ]]; then python3 "$dir/prepare.py" "$tree" "$generated" > "$out/prepare.txt"; fi
    export CARGO_TARGET_DIR=$build/$arm-target
    python3 bench/moonshot/moonshot.py score "$worker" --corpus "$corpus" --target-dir "$CARGO_TARGET_DIR" --no-timing --no-proxy --out "$out/$arm" > "$out/$arm.log" 2>&1
    cargo test --release --locked --lib block::tests::symbols_read_as_in_memory -- --exact > "$out/$arm-generated-test.txt" 2>&1
    sha256sum "$CARGO_TARGET_DIR/release/moonshot" src/block.rs "$worker" > "$out/hashes-$arm.txt"
    git diff -- src/block.rs > "$out/scanner-$arm.patch"
done
cmp "$out/refresh-stock.txt" "$out/refresh-phrase.txt"
cmp <(grep '^scorer v' "$out/stock.log") <(grep '^scorer v' "$out/phrase.log")
