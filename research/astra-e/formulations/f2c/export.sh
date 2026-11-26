#!/usr/bin/env bash
# Add a test-only observer to the immutable F2b source in a temporary directory.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
dest=${1:?output directory}
mkdir -p "$dest/src" "$dest/export"
dest=$(cd "$dest" && pwd)
cp "$root/research/astra-e/formulations/f2b/"{candidate,checks}.rs "$dest/src"
cp "$root/research/astra-e/formulations/f2c/export.rs" "$dest/src/export.rs"
cat >> "$dest/src/candidate.rs" <<'RS'
#[cfg(test)]
#[path="export.rs"]
mod f2_export;
RS
export F2_EXPORT="$dest/export" F2_CORPUS="$root/corpora/wikidata.bz2"
export MOONSHOT_CANDIDATE="$dest/src/candidate.rs"
cargo test --manifest-path "$root/bench/moonshot/Cargo.toml" f2_export::export -- --ignored --nocapture
