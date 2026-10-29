#!/usr/bin/env bash
# libsais 2.12.1 (Ilya Grebnov, Apache-2.0) at a pinned commit, its source checked by SHA-256,
# built single-threaded (no OpenMP) as a static library into DIR, for bench/alternatives'
# kernels binary (its inverse BWT beside mibz2's). Prints DIR, for LIBSAIS_DIR.
#
#   bash bench/libsais.sh DIR
#
# The same commit and sums as the cooperative pilot's (research/coop-unbwt, bench/coop.sh),
# built with the same flags but without OpenMP: -O3 -march=native.
set -euo pipefail
dir=${1:?usage: bench/libsais.sh DIR}
commit=c284ba843dd16c159acb976f17bc19382793ea75
mkdir -p "$dir"
for f in src/libsais.c include/libsais.h; do
  [ -f "$dir/$(basename "$f")" ] || curl -sSfL -o "$dir/$(basename "$f")" "https://raw.githubusercontent.com/IlyaGrebnov/libsais/$commit/$f"
done
(cd "$dir" && sha256sum -c --quiet - <<'SUMS'
aa84f63278f7cb401196a1a795c03d4d96be8b571d72b6b0901425e781ad1508  libsais.c
a18b46efcd632491acfcc696606b5455a6c8cc0b014ebd67ae29cfabba20774a  libsais.h
SUMS
)
[ -f "$dir/liblibsais.a" ] || (cd "$dir" && gcc -O3 -march=native -c libsais.c -o libsais.o && ar rcs liblibsais.a libsais.o)
cd "$dir" && pwd
