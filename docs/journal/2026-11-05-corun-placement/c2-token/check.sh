#!/bin/bash
# the token probe: type-check, then the paired-worker tests and the decoder's stream tests
# WT: a worktree of master bab0d603 with token-probe.patch applied
cd "${WT:?set WT to the patched worktree}" || exit 1
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$WT/target}"
echo "== start $(date -u +%T)"
cargo test --offline --no-run 2>&1 | grep -E "^(warning|error)|-->|Finished" | head -60
echo "== lib pairs tests"
cargo test --offline --lib pairs:: 2>&1 | grep -E "test |test result|panicked|error" | head -20
echo "== tests/pairs.rs"
cargo test --offline --test pairs 2>&1 | grep -E "test |test result|panicked|error" | head -20
echo "== tests/decode.rs + in_place.rs (StreamDecoder::new now routes Workers through with_pairs)"
cargo test --offline --test decode --test in_place 2>&1 | grep -E "test result|panicked|FAILED" | head -20
echo "== done $(date -u +%T)"
