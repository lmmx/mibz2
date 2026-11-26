#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$d" rev-parse --show-toplevel)
tmp=${1:?temporary build and results directory}
input=${2:-"$root/corpora/wikidata.bz2"}
turns=${3:-10}
cpu=${4:-$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')}
mkdir -p "$tmp"
python3 "$d/prepare.py" "$tmp"
python3 - "$input" "$tmp/oracle" <<'PY'
import bz2, shutil, sys
with bz2.open(sys.argv[1], "rb") as source, open(sys.argv[2], "wb") as output:
    shutil.copyfileobj(source, output)
PY
{
  git -C "$root" rev-parse HEAD
  git -C "$root" rev-parse origin/master
  git -C "$root" log -1 --format=%h origin/master -- bench/moonshot
  rustc --version --verbose
  sha256sum "$input" "$tmp/oracle"
  uname -a
  cat /sys/fs/cgroup/cpu.max
  lscpu | awk '/^Model name:/'
  echo "fixed cpu $cpu, $turns measured rotations"
} > "$tmp/manifest.txt"
nice -n 10 taskset -c "$cpu" "$tmp/measure" "$input" "$tmp/oracle" "$turns" "$tmp" 2> "$tmp/check.txt"
