#!/usr/bin/env bash
# Usage: bash research/astra-e/open/audit.sh [verified corpus path].
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
export MOONSHOT_CORPUS=$(realpath "${1:-corpora/wikidata.bz2}")
echo "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915  $MOONSHOT_CORPUS" | sha256sum --check >&2
audit_tmp=$(mktemp -d)
trap 'rm -rf "$audit_tmp"' EXIT
cp research/astra-e/open/{candidate,endpoint,profile}.rs "$audit_tmp/"
printf '\n#[cfg(test)]\nmod profile;\n' >> "$audit_tmp/candidate.rs"
MOONSHOT_CANDIDATE="$audit_tmp/candidate.rs" cargo test --release --locked --manifest-path bench/moonshot/Cargo.toml audit_wikidata -- --ignored --nocapture
