#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
tree=$(realpath "${1:?prepared 01e44cbe MTF checkout}")
deps=$(realpath "${2:?matching release/deps directory with libmibz2.rlib}")
test "$(git -C "$tree" rev-parse HEAD)" = 01e44cbe011c1f965b0e54fa724a738083512976
b=$(mktemp -d)
trap 'rm -rf -- "$b"' EXIT
w="$tree/research/astra-e/mtf-pairs/worker"
cp "$w"/{candidate.rs,common.rs,kernel.rs,gate.rs,stock.front.rs,wide.front.rs} "$b"
cp "$here/review-decline.rs" "$b"
rustc -O --edition=2021 "$b/review-decline.rs" -L "dependency=$deps" --extern "mibz2=$deps/libmibz2.rlib" -o "$b/check"
"$b/check" "$b/raw" | tee "$b/receipt"
crc=$(sed -n 's/.* crc=\([[:xdigit:]]\{8\}\) .*/\1/p' "$b/receipt")
python3 -B "$here/review-decline.py" "$b/raw" "$crc" "$b/input.bz2"

if [[ $# == 3 ]]; then
    for arm in scalar copied candidate; do
        "$3/$arm" run --arm candidate --workers 1 --file "$b/input.bz2" |
            python3 -c 'import json,sys; r=json.load(sys.stdin); assert r["out_bytes"]==271164; print(sys.argv[1],r["out_bytes"])' "$arm"
    done
fi
