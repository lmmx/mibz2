#!/usr/bin/env bash
# A competitor study (research only; docs/journal/2026-10-29-bz2ngrs.md): mibz2 against
# bz2ngrs (Softleif's safe-Rust port of bz2-ng, at a pinned commit), on one physical core,
# with bz2-ng's C, libbzip2 and libbz2-rs as controls. Writes compete-<UTC time>.md, with
# the whole decoders' runs raw in compete-<UTC time>.jsonl and the kernels' passes in
# compete-<UTC time>.kernels.jsonl: the files to share.
#
#   bash bench/compete.sh [--rounds R] [--passes P] [--no-perf]
#
# 1. The whole decoders (examples/ab.rs in bench/alternatives' build): each run one
#    process decoding corpora/wikidata.bz2 from disk, pinned to one CPU, its output hashed
#    and compared; the arms' order rotated and reversed by round.
#    - streaming: mibz2 (`BzReader::new`), bz2ngrs (`DecompressReader`), libbzip2,
#      libbz2-rs, and bz2-ng's C `bzip2 -dc` through a pipe (its CLI's own I/O);
#    - one-shot (the file read whole, decoded into one growing Vec): mibz2-oneshot
#      (`mibz2::decode`) and bz2ngrs-oneshot (`Decoder::decompress_all_to_vec`), so a
#      reader's cost shows apart from its decoder's.
# 2. The inverse BWT alone (bench/alternatives/src/kernels.rs): mibz2's, bz2ngrs's and
#    libsais's kernels on the same blocks' last columns, build, walk and assembly timed
#    apart where a kernel has them, after a check that they give every block the same bytes.
# 3. With perf: one run of each of mibz2, bz2ngrs and bz2-ng's C sampled, their hottest
#    symbols, to place a whole-decoder difference outside the inverse BWT.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

rounds=14 passes=20 perf=1
while [ $# -gt 0 ]; do
  case "$1" in
    --rounds) rounds="$2"; shift 2 ;;
    --passes) passes="$2"; shift 2 ;;
    --no-perf) perf=0; shift ;;
    *) echo "bench/compete.sh: unknown argument $1"; exit 1 ;;
  esac
done

# One CPU of each physical core; the runs go on the second core's (the first takes more
# interrupts), or the only one
cpus=$(for c in $(taskset -pc $$ | sed 's/.*: //' | tr ',' '\n' | while IFS=- read a b; do seq "$a" "${b:-$a}"; done); do
  cut -d, -f1 /sys/devices/system/cpu/cpu$c/topology/thread_siblings_list | cut -d- -f1
done | sort -nu | paste -sd,)
pin=$(echo "$cpus" | tr ',' '\n' | sed -n 2p)
pin=${pin:-$(echo "$cpus" | cut -d, -f1)}

target=$(pwd)/target/compete
mkdir -p "$target"

# bz2-ng's C library and CLI at a pinned commit (CMake's Release build, its defaults)
ng=517948ed836d4c52d68d8e633887aaa21c17ba27
src=$target/bz2-ng-$ng
if [ ! -x "$src/build/bzip2" ]; then
  rm -rf "$src" && mkdir -p "$src"
  git -C "$src" init -q
  git -C "$src" fetch -q --depth 1 https://github.com/tfenne/bz2-ng "$ng"
  git -C "$src" checkout -q FETCH_HEAD
  [ "$(git -C "$src" rev-parse HEAD)" = "$ng" ] || { echo "bz2-ng: not at $ng"; exit 1; }
  cmake -S "$src/c" -B "$src/build" -DCMAKE_BUILD_TYPE=Release -DBZ2NG_BUILD_SHARED=OFF \
    -DBZ2NG_BUILD_TESTS=OFF -DBZ2NG_BUILD_BENCH=OFF -DBZ2NG_BUILD_FUZZ=OFF > /dev/null
  cmake --build "$src/build" -j "$(nproc)" --target bzip2 > /dev/null
fi
export AB_BZ2NG=$src/build/bzip2

# The harness with the alternatives (bz2ngrs at its pinned commit: bench/alternatives/Cargo.toml)
# (and libsais 2.12.1, single-threaded, for the kernels' third: bench/libsais.sh)
LIBSAIS_DIR=$(bash bench/libsais.sh "$target/libsais")
LIBSAIS_DIR=$LIBSAIS_DIR CARGO_TARGET_DIR=$target cargo build --release --manifest-path bench/alternatives/Cargo.toml --bin ab --bin kernels
ab=$target/release/ab
kernels=$target/release/kernels

out=compete-$(date -u +%Y%m%dT%H%M%SZ).md
runs=${out%.md}.jsonl
arms=mibz2,bz2ngrs,mibz2-oneshot,bz2ngrs-oneshot,bz2ng,libbzip2,libbz2-rs
{
  echo "# mibz2 against bz2ngrs, one core ($(git rev-parse --short HEAD), $(date -u +%FT%TZ))"
  echo
  echo "- **Machine:** $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs); every run pinned to CPU $pin; L2 $(cat /sys/devices/system/cpu/cpu0/cache/index2/size 2> /dev/null), L3 $(cat /sys/devices/system/cpu/cpu0/cache/index3/size 2> /dev/null)"
  echo "- **Versions:** mibz2 at this commit; bz2ngrs 6a92590; bz2-ng C $(echo $ng | cut -c1-7) ($("$AB_BZ2NG" --version 2>&1 < /dev/null | head -1 | sed 's/.*Version //'))"
  echo "- **Load:** $(cut -d' ' -f1-3 /proc/loadavg)"
  echo
  echo "## 1. The whole decoders ($rounds rounds)"
  echo
  "$ab" run "$runs" --arms "$arms" --threads 1 --pin "$pin" --rounds "$rounds" corpora/wikidata.bz2 > /dev/null
  "$ab" summary "$runs" --ref mibz2
  echo
  echo "## 2. The inverse BWT alone ($passes passes)"
  echo
  taskset -c "$pin" "$kernels" corpora/wikidata.bz2 "$passes" "${out%.md}.kernels.jsonl"
  if [ "$perf" = 1 ] && perf stat -e cycles:u true > /dev/null 2>&1; then
    echo
    echo "## 3. The hottest symbols (perf, one run each, user cycles)"
    for a in mibz2 bz2ngrs bz2ng; do
      echo
      echo "### $a"
      echo
      echo '```'
      if [ "$a" = bz2ng ]; then
        cmd=("$AB_BZ2NG" -dc corpora/wikidata.bz2)
      else
        cmd=("$ab" child "$a" 1 corpora/wikidata.bz2)
      fi
      perf record -q -e cycles:u -o "$target/$a.perf" -- taskset -c "$pin" "${cmd[@]}" > /dev/null 2>&1
      perf report -i "$target/$a.perf" --no-children --sort sym --stdio -q 2> /dev/null | grep -v '^$' | head -25
      echo '```'
    done
  fi
} 2> >(grep -v '^r[0-9]\|^  waiting' >&2) | tee "$out"
echo "wrote $out, $runs and ${out%.md}.kernels.jsonl"
