#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$d" rev-parse --show-toplevel)
out=$(realpath -m "$2")
wiki=$(realpath "$1")
cpu=${3:-3}
mkdir -p "$out"
cargo build --manifest-path "$root/Cargo.toml" --target-dir "$out/target" --locked --release --features moonshot --lib
python3 "$d/prepare.py" "$out/kernel.rs"
export EDGE_TABLE_SOURCE="$out/kernel.rs"
args=(--edition=2021 -O --extern mibz2="$out/target/release/deps/libmibz2.rlib" -L "dependency=$out/target/release/deps")
rustc "${args[@]}" "$d/main.rs" -o "$out/landmark"
rustc -Awarnings "${args[@]}" "$d/property.rs" -o "$out/property"
rustc -Awarnings "${args[@]}" "$d/count.rs" -o "$out/count"
"$out/property" > "$out/property.txt"
"$out/landmark" "$wiki" "$out/all-geometry.csv" census
taskset -c "$cpu" "$out/landmark" "$wiki" "$out/panel-geometry.csv" panel > "$out/stage.csv"
python3 "$d/report.py" "$out"
