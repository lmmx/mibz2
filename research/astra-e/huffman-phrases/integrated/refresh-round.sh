#!/usr/bin/env bash
set -euo pipefail
base=$(realpath "${1:?two frozen build worktrees}")
official=$(realpath "${2:?full matched scorer gate receipts}")
for arm in stock phrase; do
    tree=$base/$arm
    (cd "$tree"
     git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
     path=$(git log -1 --format=%H origin/master -- bench/moonshot)
     projected=$(git rev-parse origin/master:bench/moonshot)
     printf '%s %s %s %s\n' "$arm" "$(git log -1 --format=%h origin/master -- bench/moonshot)" "$path" "$projected"
     [[ "$path" == "$(sed -n '3p' "$official/refresh-$arm.txt")" && "$projected" == "$(sed -n '4p' "$official/refresh-$arm.txt")" ]] || {
         echo "Scorer changed. Rebuild both binary gates on one compatible projection before pricing." >&2
         exit 1
     }
     grep '^scorer v' "$official/$arm.log")
done
