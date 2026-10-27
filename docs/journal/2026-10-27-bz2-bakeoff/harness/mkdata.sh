#!/usr/bin/env bash
# mkdata.sh DATA_DIR: the bake-off's synthetic and standard workloads, compressed with the
# reference bzip2 (libbzip2 1.0.8) at -9. The real-dump workload is mkstream.py's.
set -euo pipefail
D=$1; mkdir -p "$D/dl"; cd "$D/dl"
# (c) the Silesia corpus as one tar (211,968,000 bytes), names sorted
[ -f "$D/silesia.bz2" ] || [ -f silesia.zip ] || curl -sS -o silesia.zip https://sun.aei.polsl.pl/~sdeor/corpus/silesia.zip
[ -f "$D/silesia.bz2" ] || { python3 -I -c "import zipfile; zipfile.ZipFile('silesia.zip').extractall('silesia')"
tar --sort=name --owner=0 --group=0 --mtime=@0 -cf - -C silesia . | bzip2 -9 > "$D/silesia.bz2"; }
rm -rf silesia silesia.zip
# (d) highly repetitive text: one 64-byte line, 3,276,800 times (200 MiB; every block a
#     repeated string, so its inverse BWT has many cycles)
python3 -I -c "
import sys
line = b'{\"id\":\"Q42\",\"type\":\"item\",\"labels\":{\"en\":\"Douglas Adams\"}}'.ljust(63) + b'\n'
sys.stdout.buffer.write(line * 3276800)" | bzip2 -9 > "$D/repetitive.bz2"
# (e) 100 MiB of pseudo-random bytes (seeded), incompressible
python3 -I -c "
import random, sys
r = random.Random(20261027)
for _ in range(100): sys.stdout.buffer.write(r.randbytes(1 << 20))" | bzip2 -9 > "$D/random.bz2"
cd "$D"; rmdir dl; ls -l "$D"
