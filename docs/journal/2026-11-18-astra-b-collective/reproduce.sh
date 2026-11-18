#!/bin/sh
set -eu
# Run from the repository root with Python assertions enabled and rustc on PATH.
study=docs/journal/2026-11-18-astra-b-collective
corpus=$1
out=$2
mkdir -p "$out"
sh "$study/extract.sh" "$corpus" "$out/records"
python3 "$study/check.py" > "$out/small.json"
cmp "$study/small.json" "$out/small.json"
for group in frozen sample; do
    if [ "$group" = frozen ]; then
        set -- 0
    else
        set -- 24 48 72 96 120 144 168 192 216
    fi
    for probe in screen residual queue_ranges; do
        case "$probe" in
            screen) file=$group.csv ;;
            residual) file=$group-residual.csv ;;
            queue_ranges) file=$group-queues.csv ;;
        esac
        python3 "$study/$probe.py" "$out/records" --blocks "$@" > "$out/$file"
        cmp "$study/$file" "$out/$file"
    done
done
for probe in packets ledger; do
    python3 "$study/$probe.py" "$out/records" > "$out/$probe.csv"
    cmp "$study/$probe.csv" "$out/$probe.csv"
done
python3 "$study/verify_real.py" "$corpus" "$out/records" > "$out/verified.csv"
cmp "$study/verified.csv" "$out/verified.csv"
echo "All structural tables, marked outputs and input digests reproduced."
