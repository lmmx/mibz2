#!/usr/bin/env bash
# Builds chaingrind, an out-of-tree Valgrind tool, against the installed Valgrind's tool
# headers and static libraries (Debian/Ubuntu: the `valgrind` package ships them), into OUT, a
# directory Valgrind can run it from: VALGRIND_LIB=OUT valgrind --tool=chaingrind ...
# Usage: build.sh OUT
set -euo pipefail
out=${1:?usage: build.sh OUT}
here=$(cd "$(dirname "$0")" && pwd)
inc=${VALGRIND_INC:-/usr/include/valgrind}
lib=${VALGRIND_STATIC:-}
if [ -z "$lib" ]; then
  for d in /usr/lib/x86_64-linux-gnu/valgrind /usr/lib64/valgrind /usr/lib/valgrind /usr/libexec/valgrind; do
    [ -f "$d/libcoregrind-amd64-linux.a" ] && lib=$d && break
  done
fi
run=${VALGRIND_RUNTIME:-}
if [ -z "$run" ]; then
  for d in /usr/libexec/valgrind /usr/lib/x86_64-linux-gnu/valgrind /usr/lib64/valgrind /usr/lib/valgrind; do
    [ -f "$d/vgpreload_core-amd64-linux.so" ] && run=$d && break
  done
fi
[ -f "$inc/pub_tool_basics.h" ] || { echo "chaingrind: no Valgrind tool headers in $inc"; exit 1; }
[ -n "$lib" ] || { echo "chaingrind: no libcoregrind-amd64-linux.a (install valgrind's static libraries)"; exit 1; }
[ -n "$run" ] || { echo "chaingrind: no Valgrind runtime directory (vgpreload_core-amd64-linux.so)"; exit 1; }
mkdir -p "$out"
cc=${CC:-gcc}
$cc -c -o "$out/chaingrind.o" "$here/chaingrind.c" -I"$inc" \
  -DVGA_amd64=1 -DVGO_linux=1 -DVGP_amd64_linux=1 -DVGPV_amd64_linux_vanilla=1 \
  -m64 -O2 -g -std=gnu99 -fno-strict-aliasing -fno-builtin -fno-stack-protector \
  -fno-pic -fno-pie -Wall -Wno-unused-parameter -Wno-pointer-sign
$cc -o "$out/chaingrind-amd64-linux" "$out/chaingrind.o" -m64 -static -nodefaultlibs -nostartfiles \
  -u _start -Wl,--build-id=none -Wl,-Ttext-segment=0x58000000 -no-pie \
  "$lib/libcoregrind-amd64-linux.a" "$lib/libvex-amd64-linux.a" "$lib/libgcc-sup-amd64-linux.a" -lgcc
# Valgrind looks for the tool and its preloads, suppressions and XML beside each other
for f in "$run"/*; do
  name=$(basename "$f")
  [ -e "$out/$name" ] || ln -s "$f" "$out/$name"
done
echo "$out/chaingrind-amd64-linux"
