#!/usr/bin/env bash
set -eu
out=${1:?provide a temporary binary directory}
mkdir -p "$out"
pin=8f1be9d4067b9e0eb4d1bb627bf1c764a2786218
control=research/astra-e/formulations/f2-heaviest/orientation/seams/async/native/ranks.cpp
git show "$pin:$control" | sed '/^int main(/,$d' > "$out/control.hpp"
g++ -std=c++20 -O2 -Wall -Wextra -Wpedantic -I"$out" "$(dirname "$0")/history.cpp" -o "$out/history"
