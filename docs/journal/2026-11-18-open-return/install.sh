#!/bin/sh
set -eu
study=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
tree=$1
for file in open_return switches endpoint checks segments; do
    cp "$study/$file.rs" "$tree/src/$file.rs"
done
cp "$study/probe.rs" "$tree/examples/open_return.rs"
cp "$study/heap.rs" "$tree/examples/return_heap.rs"
if ! grep -qxF 'pub mod open_return;' "$tree/src/lib.rs"; then
    echo 'pub mod open_return;' >> "$tree/src/lib.rs"
fi
