#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
[ "$#" -eq 0 ] || { echo 'argument-free registered recipe' >&2; exit 2; }
mkdir -p target/cells-selective-i9
DIR="target/cells-selective-i9/$(date -u +%Y%m%dT%H%M%S%NZ)"
mkdir "$DIR"
exec > >(tee -a "$DIR/console.log") 2>&1
finish() {
    local result=$?
    trap - EXIT
    printf '%s\n' "$result" > "$DIR/exit"
    if [ "$result" -ne 0 ]; then
        printf 'INCOMPLETE: recipe exit %s; retain partial evidence, no automatic retry\n' "$result" > "$DIR/outcome.txt"
    fi
    bash bench/bundle.sh "$DIR" || exit 125
    exit "$result"
}
trap finish EXIT
python3 bench/bounded.py 7200 5 python3 bench/cells-selective-i9/workstation.py prepare "$DIR"
python3 bench/bounded.py 7200 5 python3 bench/cells-selective-i9/workstation.py measure "$DIR"
