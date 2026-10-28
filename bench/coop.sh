#!/usr/bin/env bash
# The knee pilot's cooperative inversion (examples/coop.rs, research only;
# docs/journal/2026-10-28-cooperative-inversion.md): one block's T-vector built and walked
# by k cores, and N blocks in flight on k cores each against N production workers, on one
# CPU of each physical core. Writes coop-<UTC time>.md and its rounds, raw, in
# coop-<UTC time>.jsonl: the files to share.
#
#   bash bench/coop.sh [--rounds R] [--arms p4,p8,4x2,...] [--no-libsais]
#
# With gcc and OpenMP, libsais 2.12.1 (its cyclic inverse BWT, the multi-threaded reference)
# is built from its source at a pinned commit, checked by SHA-256, and timed beside.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

rounds="" arms="" libsais=1
while [ $# -gt 0 ]; do
  case "$1" in
    --rounds) rounds="$2"; shift 2 ;;
    --arms) arms="$2"; shift 2 ;;
    --no-libsais) libsais=0; shift ;;
    *) echo "bench/coop.sh: unknown argument $1"; exit 1 ;;
  esac
done

# One CPU of each physical core this process may use (the first of each sibling list)
cpus=$(for c in $(taskset -pc $$ | sed 's/.*: //' | tr ',' '\n' | while IFS=- read a b; do seq "$a" "${b:-$a}"; done); do
  cut -d, -f1 /sys/devices/system/cpu/cpu$c/topology/thread_siblings_list | cut -d- -f1
done | sort -nu | paste -sd,)
cores=$(echo "$cpus" | tr ',' '\n' | wc -l)

# libsais at a pinned commit, built with OpenMP into the example's own target directory
target=target/coop
flags=""
if [ "$libsais" = 1 ] && command -v gcc > /dev/null && echo 'int main(){return 0;}' | gcc -fopenmp -x c - -o /dev/null 2> /dev/null; then
  commit=c284ba843dd16c159acb976f17bc19382793ea75
  dir=$target/libsais-$commit
  mkdir -p "$dir"
  for f in src/libsais.c include/libsais.h; do
    [ -f "$dir/$(basename $f)" ] || curl -sSfL -o "$dir/$(basename $f)" "https://raw.githubusercontent.com/IlyaGrebnov/libsais/$commit/$f"
  done
  (cd "$dir" && sha256sum -c --quiet - <<'SUMS'
aa84f63278f7cb401196a1a795c03d4d96be8b571d72b6b0901425e781ad1508  libsais.c
a18b46efcd632491acfcc696606b5455a6c8cc0b014ebd67ae29cfabba20774a  libsais.h
SUMS
  )
  [ -f "$dir/liblibsais_omp.a" ] || (cd "$dir" && gcc -O3 -march=native -fopenmp -DLIBSAIS_OPENMP -c libsais.c -o libsais_omp.o && ar rcs liblibsais_omp.a libsais_omp.o)
  flags="--cfg libsais -L $(pwd)/$dir"
else
  libsais=0
fi
RUSTFLAGS="$flags" CARGO_TARGET_DIR=$target cargo build --release --example coop
bin=$target/release/examples/coop

# The arms that fit the cores: production workers at 4 and beyond, the cooperative kernel
# at 4 x 1 (its cost against production's at the same footprint), and N x k with N blocks
# live on N x k cores
if [ -z "$arms" ]; then
  arms="p4,p2,p6,p8,p10,4x1,2x2,3x2,4x2,5x2,3x3,2x4,w4x2,w5x2,w3x3"
  [ "$libsais" = 1 ] && arms="$arms,s4x1,s4x2,s5x2,s3x3"
fi
fit=""
for a in $(echo "$arms" | tr ',' ' '); do
  s=${a#[psw]}; n=${s%x*}; k=1; [ "$s" != "$n" ] && k=${s#*x}
  [ $((n * k)) -le "$cores" ] && fit="$fit,$a"
done
fit=${fit#,}
n_arms=$(echo "$fit" | tr ',' '\n' | wc -l)
rounds=${rounds:-$((2 * n_arms))}
ks="1,2,3,4"; [ "$cores" -lt 4 ] && ks="1,2"

out=coop-$(date -u +%Y%m%dT%H%M%SZ).md
export COOP_JSONL=${out%.md}.jsonl  # (every round of the grid, raw)
{
  echo "# The cooperative inversion pilot ($(git rev-parse --short HEAD), $(date -u +%FT%TZ))"
  echo
  echo "- **Machine:** $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs); $(nproc --all) CPUs on $cores physical cores used ($cpus); L3 $(cat /sys/devices/system/cpu/cpu0/cache/index3/size 2> /dev/null) shared by CPUs $(cat /sys/devices/system/cpu/cpu0/cache/index3/shared_cpu_list 2> /dev/null)"
  echo "- **libsais:** $([ "$libsais" = 1 ] && echo "2.12.1 at c284ba8, OpenMP" || echo "not built")"
  echo "- **Load:** $(cut -d' ' -f1-3 /proc/loadavg)"
  echo
  echo "## One block on k cores (block)"
  echo
  taskset -c "$cpus" "$bin" block corpora/wikidata.bz2 3 "$ks"
  echo
  echo "## N blocks in flight on k cores each (grid, $rounds rounds)"
  echo
  taskset -c "$cpus" "$bin" grid corpora/wikidata.bz2 "$rounds" "$fit"
} 2> >(grep -v '^pass\|^round' >&2) | tee "$out"
echo "wrote $out and $COOP_JSONL"
