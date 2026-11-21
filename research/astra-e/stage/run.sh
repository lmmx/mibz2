#!/usr/bin/env bash
# stdout holds the raw CSV. Retain stderr with it for provenance.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
data=${1:-corpora/wikidata.bz2}
rounds=${2:-11}
expected=82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915
digest=$(sha256sum "$data")
test "${digest%% *}" = "$expected"
cargo build --release --locked --features moonshot --example astra_e
bin="${CARGO_TARGET_DIR:-target}/release/examples/astra_e"
{
  git rev-parse HEAD
  rustc -Vv
  sha256sum "$data" "$bin"
  echo "single-thread invert_runs on warmed state; one untimed warm pass; $rounds recorded passes"
  echo "ordinals 1,19,61,108,150,194,240,282,329,371,407; large eligible decile midpoints plus first"
  cat /sys/fs/cgroup/cpu.max
} >&2
"$bin" "$data" "$rounds"
