#!/usr/bin/env bash
set -euo pipefail
d=$(cd "$(dirname "$0")" && pwd)
root=$(git -C "$d" rev-parse --show-toplevel)
dest=$(realpath -m "${1:?scratch directory}")
oracle=${2:-"$dest/oracle"}
cpu=${3:-3}
mkdir -p "$dest"
python3 "$d/prepare.py" "$dest"
if [[ ! -f "$oracle" ]]; then
  python3 - "$root/corpora/wikidata.bz2" "$oracle" <<'PY'
import bz2,shutil,sys
with bz2.open(sys.argv[1],'rb') as f,open(sys.argv[2],'wb') as g:
    shutil.copyfileobj(f,g)
PY
fi
{
  rustc --version --verbose
  sha256sum "$root/corpora/wikidata.bz2" "$oracle" "$dest/portfolio" "$dest/libmibz2.rlib"
  cat /sys/fs/cgroup/cpu.max
  lscpu | awk '/^Model name:/'
  echo "CPU $cpu; 24 paired rounds; phase scope 00efe9b; no worker timing"
} > "$dest/manifest.txt"
nice -n 12 taskset -c "$cpu" "$dest/portfolio" "$root/corpora/wikidata.bz2" "$oracle" 24 "$dest" 2> "$dest/check.txt"
