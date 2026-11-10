#!/bin/bash
# arm corpus block
arm=$1; f=$2; b=$3
out=cg/$arm-$f-b$b
valgrind --tool=callgrind --toggle-collect='*measured*' --callgrind-out-file=$out.out $arm/target/release/drv cg ../rfcorp/$f.bz2 $b 1 > $out.stdout 2>/dev/null
