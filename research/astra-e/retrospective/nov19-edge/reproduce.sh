#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
wt=${1:?new detached worktree path}
d=research/astra-e/retrospective/nov19-edge
git -C "$root" worktree add --detach "$wt" 6eaf1e8bace868fe8f8bba5f73e1f6ea9e83afca
git -C "$wt" checkout be74d5763d212864607d46521d74ebc0827f9f36 -- bench/moonshot .just Justfile
mkdir -p "$wt/$d"
cp "$root/$d/receipts.py" "$root/$d/audit.py" "$wt/$d/"
cd "$wt"
python3 -B "$d/receipts.py"
LC_ALL=C just moonshot-eval research/astra-e > "$d/official.txt" 2> "$d/eval-command.txt"
python3 -B "$d/audit.py"
for path in ci-receipts.json verified-inputs.json official.txt eval-command.txt joins.csv coverage.csv conditional-edge.csv audit.json; do
  cmp "$d/$path" "$root/$d/$path"
done
