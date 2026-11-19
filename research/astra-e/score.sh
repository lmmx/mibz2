#!/usr/bin/env bash
# Run in this branch's research worktree. All pristine-candidate commits are recorded
# separately from subsequent score commits. Use a new label for each invocation.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
label=$1
candidate=$2
shift 2
root=research/astra-e/results
dest=$root/$label
test ! -e "$dest"
# The campaign's modifications are outside these scorer-owned paths.
git diff --quiet HEAD -- bench/moonshot .just Justfile
git fetch origin master
git checkout origin/master -- bench/moonshot .just Justfile
rev=$(git log -1 --format=%h origin/master -- bench/moonshot)
# Preserve each imported scorer in branch history. A change starts a new
# comparison series with the baseline and the best preserved candidate.
if ! git diff --cached --quiet -- bench/moonshot .just Justfile; then
  git commit --only -m "research(astra-e): refresh moonshot scorer to $rev" -- bench/moonshot .just Justfile
fi
bash research/astra-e/prepare.sh "$candidate"
mkdir -p "$dest"
{
  git rev-parse HEAD origin/master
  echo "scorer path revision $rev"
  sha256sum "$candidate" corpora/wikidata.bz2
  rustc -Vv
  python3 --version
  cat /sys/fs/cgroup/cpu.max
  cat /sys/fs/cgroup/cpuset.cpus.effective
  awk -F ': ' '/^model name/{print $2; exit}' /proc/cpuinfo
  printf 'invocation: bash research/astra-e/score.sh %q %q' "$label" "$candidate"
  printf ' %q' "$@"
  printf '\n'
} > "$dest/manifest.txt"
record=()
if python3 bench/moonshot/moonshot.py score --help | rg -q -- '--record'; then
  record=(--record "$dest/official-register.jsonl")
fi
python3 bench/moonshot/moonshot.py score "$candidate" --out "$dest/score" "${record[@]}" "$@" \
   > "$dest/stdout.txt" 2> "$dest/stderr.txt"
