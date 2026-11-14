#!/usr/bin/env bash
# Builds the memtrace valgrind tool out of tree against the system valgrind (3.22.0's
# headers and static libraries), into $1/lib: a VALGRIND_LIB directory holding the tool
# beside links to the system's valgrind files. Then: VALGRIND_LIB=$1/lib valgrind --tool=memtrace
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=$1
sys=/usr/libexec/valgrind
libs=/usr/lib/x86_64-linux-gnu/valgrind
mkdir -p "$out/lib"
for f in "$sys"/*; do ln -sf "$f" "$out/lib/"; done
gcc -m64 -O2 -g -Wall -Wno-unused-parameter -fno-strict-aliasing -fno-builtin \
  -fno-stack-protector -fno-pie -no-pie -fomit-frame-pointer \
  -I/usr/include/valgrind \
  -DVGA_amd64=1 -DVGO_linux=1 -DVGP_amd64_linux=1 -DVGPV_amd64_linux_vanilla=1 \
  -o "$out/lib/memtrace-amd64-linux" "$here/mt_main.c" \
  -static -nodefaultlibs -nostartfiles -u _start -Wl,--build-id=none \
  -Wl,-Ttext-segment=0x58000000 \
  "$libs/libcoregrind-amd64-linux.a" "$libs/libvex-amd64-linux.a" \
  "$libs/libgcc-sup-amd64-linux.a" -lgcc
echo "$out/lib/memtrace-amd64-linux"
