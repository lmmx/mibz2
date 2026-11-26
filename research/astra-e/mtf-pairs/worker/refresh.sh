#!/bin/bash
set -euo pipefail
tree=$(realpath "${1:?research worktree}")
official=$(realpath "${2:?common completed gates}")
cd "$tree"
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
revision=$(git log -1 --format=%H origin/master -- bench/moonshot)
printf '%s\n' "$revision"
git ls-tree origin/master bench/moonshot
for arm in candidate copied scalar; do
  test "$revision" = "$(head -1 "$official/$arm-version.txt")"
  test "$(git ls-tree origin/master bench/moonshot)" = "$(sed -n '2p' "$official/$arm-version.txt")"
  rg '^scorer v' "$official/$arm/summary.txt"
done
