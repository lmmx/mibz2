#!/bin/bash
# build.sh NAME (REV | --tree [WALK] [LIMIT])
set -e
S=${SCRATCH:?set SCRATCH to a scratch directory}
name=$1; shift
d=$S/build/src-$name
rm -rf $d; mkdir -p $d
if [ "$1" = "--tree" ]; then
  (cd ${TREE:-$REPO} && git ls-files -co --exclude-standard | grep -v '^target/' | tar -cf - -T -) | tar -x -C $d
  if [ -n "$2" ]; then sed -i "s/^const WALK: Walk = Walk::[A-Za-z]*;/const WALK: Walk = Walk::$2;/" $d/src/open.rs; fi
  if [ -n "$3" ]; then sed -i "s/^pub(crate) const LIMIT: u32 = [0-9]*;/pub(crate) const LIMIT: u32 = $3;/" $d/src/open.rs; fi
  grep -n "^const WALK\|^pub(crate) const LIMIT" $d/src/open.rs || true
else
  (cd ${REPO:?set REPO to a mibz2 checkout} && git archive $1) | tar -x -C $d
fi
python3 -c "import sys;p=sys.argv[1];s=open(p).read().replace('    pub(crate) fn records(&self)','    pub fn records(&self)',1);open(p,'w').write(s)" $d/src/runs.rs
cp $REPO/docs/journal/2026-11-20-open-stage-attribution/attrib.rs $d/examples/attrib.rs
(cd $d && CARGO_TARGET_DIR=$S/build/target cargo build --release --example attrib --example ab -q 2>&1 | grep -E "^error" -A5 || true)
cp $S/build/target/release/examples/attrib $S/out/bin/attrib-$name
cp $S/build/target/release/examples/ab $S/out/bin/ab-$name
echo built $name
