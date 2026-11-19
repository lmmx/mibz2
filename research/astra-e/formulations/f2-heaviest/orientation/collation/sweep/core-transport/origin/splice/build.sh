#!/usr/bin/env bash
set -euo pipefail
self=$(cd "$(dirname "$0")" && pwd)
dest=$(realpath -m "${1:?scratch directory}")
lib=$(realpath "${2:?directory containing libtoken.so}")
mkdir -p "$dest"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$self/small.rs" -o "$dest/small"
rustc --edition=2021 -C opt-level=3 -C codegen-units=1 "$self/native.rs" -L "native=$lib" -l dylib=token -C "link-arg=-Wl,-rpath,$lib" -o "$dest/panel"
