#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
d="$root/research/astra-e/formulations/f2-heaviest/orientation/seams/async/native/token-writer"
out=${1:?absolute scratch output}; deps=${2:?absolute release dependency directory}
mkdir -p "$out"
cp "$root/research/astra-e/formulations/f2c/second.rs" "$out/second.rs"
cat "$d/second_cap.rs" >> "$out/second.rs"
sed -e 's#../../../f2c/second.rs#second.rs#' -e '/^struct Piece {/i #[repr(C)]' "$root/research/astra-e/formulations/f2-pages/payload/slots/candidate.rs" > "$out/compact.rs"
cat "$d/interface.rs" >> "$out/compact.rs"
cat > "$out/wrapper.rs" <<'RS'
pub use mibz2::{bwt,runs,Kernels};
mod compact;
RS
rustc --edition=2021 --crate-type=cdylib -A private_interfaces -A dead_code -A unexpected_cfgs -C opt-level=3 -C codegen-units=1 "$out/wrapper.rs" --extern mibz2="$deps/libmibz2.rlib" -L dependency="$deps" -o "$out/libtoken.so"
g++ -std=c++20 -O3 -Wall -Wextra -Wno-stringop-overread "$d/writer.cpp" -L"$out" -ltoken -Wl,-rpath,"$out" -o "$out/writer"
g++ -std=c++20 -O3 -DNDEBUG -DASTRA_RANK_PANEL -Wall -Wextra -Wno-unused-variable -Wno-unused-parameter -Wno-stringop-overread "$d/writer.cpp" -L"$out" -ltoken -Wl,-rpath,"$out" -o "$out/rank"
rustc --edition=2021 -C opt-level=3 "$d/export.rs" --extern mibz2="$deps/libmibz2.rlib" -L dependency="$deps" -o "$out/export"
g++ -std=c++20 -O3 -Wno-stringop-overread "$d/coverage.cpp" -L"$out" -ltoken -Wl,-rpath,"$out" -o "$out/coverage"
g++ -std=c++20 -O3 -DNDEBUG -DASTRA_ENVELOPE_PANEL -DASTRA_RANK_PANEL -Wno-stringop-overread "$d/writer.cpp" -L"$out" -ltoken -Wl,-rpath,"$out" -o "$out/envelope"
