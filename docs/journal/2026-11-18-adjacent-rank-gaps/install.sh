#!/bin/sh
# Install the journal sources in an experimental worktree passed as the argument.
set -eu
study=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
tree=${1:?pass the destination mibz2 worktree}
earlier="$study/../2026-11-18-bispecial-factors"
for module in extrema forest valleys; do
    cp "$study/$module.rs" "$tree/src/$module.rs"
    if ! grep -Fxq "pub mod $module;" "$tree/src/lib.rs"; then
        printf '\npub mod %s;\n' "$module" >> "$tree/src/lib.rs"
    fi
done
cp "$study/probe.rs" "$tree/examples/extrema_probe.rs"
cp "$study/heap.rs" "$tree/examples/extrema_memory.rs"
price="$tree/examples/support_price"
mkdir -p "$price"
cp "$study/support_price.rs" "$price/main.rs"
cp "$earlier/contexts.rs" "$earlier/support.rs" "$price/"
cat >> "$price/contexts.rs" <<'RUST'

pub fn through_support(last: &[u8], primary: usize) -> Vec<u8> {
    let (weighted, _, _, rank) = pack(last, primary);
    support::read(&support_tape(&weighted, rank)).marked_output().0
}
RUST
