#!/usr/bin/env bash
# #78's stage 2 (the coop journal's §14; research only): whether the build's saving survives
# making its count checkpoints in the MTF stage. Two measurements, each thread pinned to its
# own physical core as bench/coop.sh pins them:
#   1. the MTF stage alone, one core: production's kernel against `mtf_cp`, which also makes
#      the checkpoints, paired by round (`coop mtf`), with what the checkpoints occupy;
#   2. the grid with every worker making its block's last column from the symbols in the
#      timing (COOP_WITH_MTF=1): r3x3, q3x3 (checkpoints free), n3x3 (checkpoints made by
#      its own MTF), r4x1, n4x1 and p4, then bench/coop_ab.py's pairs (n against r: the net;
#      n against q: the checkpoints' cost; q against r: the bound with the MTF timed).
# Writes coop-net-<UTC time>.md (both), its grid's .jsonl, and coop-ab-<UTC time>.md.
# With --spill, #78's spill fix (§15): the same two measurements, the MTF stage alone now
# also timing `mtf_cp2` and `mtf_cp2p`, the grid's default arms adding m3x3 and f3x3 (and
# f4x1, the null), written as coop-spill-<UTC time>.*.
#
#   bash bench/coop_net.sh [--spill] [--rounds R] [--arms LIST]
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
rounds=20 arms="" label=net
while [ $# -gt 0 ]; do
  case "$1" in
    --spill) label=spill; shift ;;
    --rounds) rounds="$2"; shift 2 ;;
    --arms) arms="$2"; shift 2 ;;
    *) echo "coop-net: unknown argument $1" >&2; exit 2 ;;
  esac
done
if [ -z "$arms" ]; then
  if [ "$label" = spill ]; then
    arms="r3x3,q3x3,n3x3,m3x3,f3x3,r4x1,f4x1,p4"
  else
    arms="r3x3,q3x3,n3x3,r4x1,n4x1,p4"
  fi
fi
if [ "$label" = spill ]; then
  title="#78's spill fix: the checkpoints made by \`mtf_cp2\` and \`mtf_cp2p\` (journal §15)"
else
  title="#78's stage 2: the checkpoints made in the MTF stage"
fi
eval "$(bash bench/coop.sh --build-only --no-libsais | tail -1)"
out=coop-$label-$(date -u +%Y%m%dT%H%M%SZ).md
jsonl=${out%.md}.jsonl
{
  echo "# $title ($(git rev-parse --short HEAD), $(date -u +%FT%TZ))"
  echo
  echo "- **Machine:** $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs); CPUs used $cpus"
  echo "- **Load:** $(cut -d' ' -f1-3 /proc/loadavg)"
  echo
  echo "## 1. The MTF stage alone, one core ($rounds rounds)"
  echo
  COOP_CPUS=$cpus taskset -c "${cpus%%,*}" "$bin" mtf corpora/wikidata.bz2 "$rounds"
  echo
  echo "## 2. The grid with the MTF stage timed in every arm ($rounds rounds)"
  echo
  COOP_WITH_MTF=1 COOP_CPUS=$cpus COOP_JSONL=$jsonl taskset -c "$cpus" "$bin" grid corpora/wikidata.bz2 "$rounds" "$arms"
} 2> >(grep -v '^round' >&2) | tee "$out"
python3 bench/coop_ab.py "$jsonl"
echo "wrote $out, $jsonl and the coop-ab file: upload all three"
