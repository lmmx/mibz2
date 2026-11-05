#!/bin/bash
S="$(cd "$(dirname "$0")" && pwd)"  # place the bench corpus (sha 82f65ace) at $S/corp/wikidata.bz2
O=$S/out; D=$S/data-indep; mkdir -p $O; cd $S/sim
nice ./pwx 512,8,32,8 $D/tt-b1.bin $D/tt-b233.bin < jobs.txt > $O/sim-smt-512K-b1b233.txt
nice ./pwx 1024,16,32,8 $D/tt-b1.bin $D/tt-b233.bin < jobs.txt > $O/sim-smt-1M-b1b233.txt
nice ./pwx 512,8,32,8 $D/tt-b1.bin < jobs.txt > $O/sim-alone-512K-b1.txt
nice ./pwx 1024,16,32,8 $D/tt-b1.bin < jobs.txt > $O/sim-alone-1M-b1.txt
nice ./pwx 512,8,32,8 $S/data/tt-b1.bin $S/data/tt-b233.bin < jobs.txt > $O/sim-smt-512K-bench-b1b233.txt
echo SIM DONE
