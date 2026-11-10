#!/usr/bin/env bash
set -u
# (run from a directory holding ab, walk_count, in/{b1,b41,b233,wikidata}.bz2, and empty cg/, out/, log/)
VG="valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes"
for inp in b1 b41 b233 wikidata; do
  f=in/$inp.bz2
  $VG --callgrind-out-file=cg/$inp-old.%p ./ab child mibz2-pipe3-both 9 $f > out/$inp-old.json 2> log/$inp-old.log
  AB_STAGED_TRAVERSE=1 $VG --callgrind-out-file=cg/$inp-staged.%p ./ab child mibz2-pipe3-both 9 $f > out/$inp-staged.json 2> log/$inp-staged.log
  for m in old staged room; do
    $VG --callgrind-out-file=cg/$inp-ctl-$m.%p ./walk_count control $f $m 1024 > out/$inp-ctl-$m.txt 2> log/$inp-ctl-$m.log
  done
  $VG --callgrind-out-file=cg/$inp-ctl-room512.%p ./walk_count control $f room 512 > out/$inp-ctl-room512.txt 2> log/$inp-ctl-room512.log
  echo "$inp done"
done
echo ALL DONE
