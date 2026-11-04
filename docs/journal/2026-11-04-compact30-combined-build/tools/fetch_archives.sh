#!/bin/sh
# Fetch the components' archived listings and raw profiles from their pins (git show only;
# nothing is checked out or modified) into OUT/{b2,scanner,descriptor,v3}.
#   sh fetch_archives.sh REPO OUT
set -e
R=$1; O=$2; mkdir -p $O/b2 $O/scanner $O/descriptor $O/v3
T=docs/journal/2026-11-04-compact30-scanner-u1u2-trial
P=docs/journal/2026-11-04-compact30-descriptor-prototype
V=docs/journal/2026-11-04-compact30-epoch-tagged-finaliser/listings
B=docs/journal/2026-11-03-compact30-constructor-v2-prototype
for f in $(git -C $R ls-tree --name-only ba261eff245f45e1c07fdfb05fa2fe6ff1ced835 $T/listings/ $T/callgrind/ | grep -E '\.asm$|\.gz$'); do git -C $R show ba261eff245f45e1c07fdfb05fa2fe6ff1ced835:$f > $O/scanner/$(basename $f); done
for f in $(git -C $R ls-tree --name-only 7620c0c667018772a0ba3be28eb62263b55037f8 $P/listings/ $P/callgrind/ | grep -E '\.asm$|\.gz$'); do git -C $R show 7620c0c667018772a0ba3be28eb62263b55037f8:$f > $O/descriptor/$(basename $f); done
for f in $(git -C $R ls-tree --name-only 46c32221 $V/); do git -C $R show 46c32221:$f > $O/v3/$(basename $f); done
for f in $(git -C $R ls-tree --name-only cd14fa90 $B/ | grep '\.asm$'); do git -C $R show cd14fa90:$f > $O/b2/$(basename $f); done
echo fetched
