#!/bin/sh
set -eu
study=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
tree=$1
for unit in switch_elimination network switch_audit coincidence coincidence_audit forest forest_audit; do
    cp "$study/$unit.rs" "$tree/src/$unit.rs"
done
cp "$study/probe.rs" "$tree/examples/switches.rs"
cp "$study/heap.rs" "$tree/examples/switch_heap.rs"
if ! grep -qxF 'pub mod switch_elimination;' "$tree/src/lib.rs"; then
    echo 'pub mod switch_elimination;' >> "$tree/src/lib.rs"
fi
