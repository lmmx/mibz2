#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$d" rev-parse --show-toplevel)
out=$(realpath -m "$2")
mkdir -p "$out"
cd "$root"
cargo build --locked --release --features moonshot --lib --target-dir "$out/target"
args=(-O --edition=2021 "$d/locality.rs" --extern mibz2="$out/target/release/deps/libmibz2.rlib" -L "dependency=$out/target/release/deps")
rustc --test "${args[@]}" -o "$out/test-locality"
"$out/test-locality"
rustc "${args[@]}" -o "$out/locality"
"$out/locality" "$1" "$out"
