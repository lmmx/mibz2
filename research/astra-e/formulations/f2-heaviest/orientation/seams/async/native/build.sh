#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-heaviest/orientation/seams/async/native"
out=${1:?absolute scratch output directory}
deps=${2:?absolute Rust release deps directory with libmibz2.rlib}
mkdir -p "$out"
cp "$root/research/astra-e/formulations/f2c/second.rs" "$out/second.rs"
sed 's#../../../f2c/second.rs#second.rs#' "$root/research/astra-e/formulations/f2-pages/payload/slots/candidate.rs" > "$out/compact.rs"
cat "$d/compact_ffi.rs" >> "$out/compact.rs"
cat > "$out/wrapper.rs" <<'RS'
pub use mibz2::{bwt,runs,Kernels};
mod compact;
RS
rustc --edition=2021 --crate-type=cdylib -A dead_code -A unexpected_cfgs -C opt-level=3 -C codegen-units=1 "$out/wrapper.rs" --extern mibz2="$deps/libmibz2.rlib" -L dependency="$deps" -o "$out/libcompact.so"
g++ -std=c++20 -O3 -DNDEBUG -Wall -Wextra -Wno-unused-variable -Wno-unused-parameter -Wno-stringop-overread "$d/match.cpp" -L"$out" -lcompact -Wl,-rpath,"$out" -o "$out/match"
g++ -std=c++20 -O3 -DASTRA_PAGED -Wno-stringop-overread "$d/ranks.cpp" -o "$out/check"
