#!/usr/bin/env bash
# Two frozen decoder checkouts; all target timing is conditional on the actual 10900K.
set -euo pipefail
mode=${1:?expected dry or measure}
[[ $mode == dry || $mode == measure ]]
repo=$(git rev-parse --show-toplevel)
cd "$repo"
reg=$(git rev-parse HEAD)
recipe=research/astra-e/workstation
score=bench/moonshot
candidate=research/astra-e/front-fusion/cap-batch/worker/candidate.rs
old=research/astra-e/edge-budget/candidate.rs
scalar=d0a42980e896211906234c7679b38760ae44a46e
edge=badb2aec3624bd65606c429b2e6f79e9d9d0050f
release=530ed404a014a414ef0b62adfd947fa14d4533e6
git diff --quiet HEAD -- "$recipe/scalar-run.sh" "$recipe/scalar.py"
base=$repo/target/astra-e-scalar
mkdir -p "$base"
out=$base/run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir "$out"
exec > >(tee -a "$out/console.log") 2>&1
echo "$mode" > "$out/mode.txt"
scratch=$(mktemp -d "$out/.build-XXXXXXXX")
headtree=$scratch/scalar
oldtree=$scratch/edge
cleanup() {
    git -C "$repo" worktree remove --force "$headtree" >/dev/null 2>&1 || true
    git -C "$repo" worktree remove --force "$oldtree" >/dev/null 2>&1 || true
    rm -rf -- "$scratch"
}
finish() {
    status=$?
    cd "$repo"
    cleanup
    if (( status != 0 )); then echo "Protocol exited $status. Review the partial evidence." > "$out/outcome.txt"; fi
    git show "$reg:bench/bundle.sh" | bash -s -- "$out"
}
trap finish EXIT
git worktree add -q --detach "$headtree" "$scalar"
git worktree add -q --detach "$oldtree" "$edge"
for tree in "$headtree" "$oldtree"; do
    git -C "$tree" checkout "$reg" -- "$recipe/scalar.py" "$recipe/edge.py" "$recipe/refresh.py" "$recipe/assess.py" "$recipe/test_protocol.py" research/astra-e/ci/paired.py
done
git cat-file -e "$release^{commit}" || git fetch origin tag v0.5.6
{
    echo "recipe $reg"; echo "scalar $scalar $candidate"; echo "measured edge $edge $old"
    git -C "$headtree" ls-tree HEAD -- src Cargo.toml Cargo.lock
    git -C "$oldtree" ls-tree HEAD -- src Cargo.toml Cargo.lock
    sha256sum "$headtree/$candidate" "$oldtree/$old"
} > "$out/matched-sources.txt"
git ls-tree -r "$reg" -- "$recipe/scalar.py" "$recipe/scalar-run.sh" "$recipe/edge.py" "$recipe/refresh.py" "$recipe/assess.py" "$recipe/test_protocol.py" research/astra-e/ci/paired.py > "$out/protocol-blobs.txt"
git diff "$release" "$scalar" -- src Cargo.toml Cargo.lock > "$out/scalar-against-v0.5.6.diff"
git diff "$release" "$edge" -- src Cargo.toml Cargo.lock > "$out/edge-against-v0.5.6.diff"
export RUSTUP_TOOLCHAIN=1.99.1 PYTHONDONTWRITEBYTECODE=1
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS PYTHONOPTIMIZE
rustc -Vv > "$out/toolchain.txt"; cargo -Vv >> "$out/toolchain.txt"; python3 -V >> "$out/toolchain.txt"
lscpu > "$out/lscpu.txt"; lscpu -e > "$out/lscpu-e.txt"
cat /sys/devices/system/cpu/online > "$out/cpus-online.txt"; awk '/^Cpus_allowed_list:/' /proc/self/status > "$out/affinity.txt"
if [[ $mode == measure ]]; then
    python3 - <<'PY'
import os
from pathlib import Path
if "i9-10900K" not in Path("/proc/cpuinfo").read_text() or len(os.sched_getaffinity(0))!=20:
    raise SystemExit("Only the unpinned i9-10900K with all 20 CPUs may measure this recipe.")
PY
fi
(cd "$oldtree" && bash research/astra-e/prepare.sh "$old")
cd "$headtree"
bash research/astra-e/prepare.sh "$candidate"
export CARGO_TARGET_DIR=$scratch/target
export MOONSHOT_CANDIDATE=$headtree/$candidate
python3 "$recipe/scalar.py" select "$out" "$oldtree"
git rev-parse HEAD origin/master "$edge" "$release" > "$out/commits.txt"
selected=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["selected_build"])' "$out/scorer-refresh.json")
other_target=$CARGO_TARGET_DIR/predecessor/$selected
export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/$selected"
bin=$CARGO_TARGET_DIR/release/moonshot
other=$other_target/release/moonshot
sha256sum "$bin" "$other" > "$out/binaries.sha256"
git ls-files -s -- "$score" .just Justfile > "$out/scorer-blobs.txt"
git -C "$oldtree" ls-files -s -- "$score" .just Justfile > "$out/predecessor-scorer-blobs.txt"
cmp "$out/scorer-blobs.txt" "$out/predecessor-scorer-blobs.txt"
"$bin" info > "$out/scorer-info.json"; "$other" info > "$out/predecessor-info.json"
if [[ $mode == measure ]]; then
    python3 -c 'import sys,json; assert all(json.load(open(p))["cpus"]==20 for p in sys.argv[1:])' "$out/scorer-info.json" "$out/predecessor-info.json"
fi
corpus=$scratch/wikidata.bz2
x4=$scratch/wikidata-x4.bz2
bash "$score/corpus.sh" "$corpus"
cat "$corpus" "$corpus" "$corpus" "$corpus" > "$x4"
sha256sum "$corpus" "$x4" > "$out/inputs.sha256"
[[ $(sha256sum "$x4" | cut -d' ' -f1) == 0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e ]]
python3 "$recipe/scalar.py" plan > "$out/roster.json"
python3 "$score/moonshot.py" score "$candidate" --corpus "$corpus" --target-dir "$CARGO_TARGET_DIR" --no-build --no-timing --no-proxy --out "$out/candidate-gate" | tee "$out/scorer-report.txt"
python3 "$score/moonshot.py" score "$oldtree/$old" --corpus "$corpus" --target-dir "$other_target" --no-build --no-timing --no-proxy --out "$out/predecessor-gate" | tee "$out/predecessor-report.txt"
cmp <(grep '^scorer v' "$out/scorer-report.txt") <(grep '^scorer v' "$out/predecessor-report.txt")
python3 "$recipe/test_protocol.py" --gate "$out/candidate-gate/check.jsonl" candidate
python3 "$recipe/test_protocol.py" --gate "$out/predecessor-gate/check.jsonl" candidate
for arm in lazy opend; do
    "$other" check --quick --arm "$arm" --wikidata "$corpus" > "$out/gate-$arm.jsonl"
    python3 "$recipe/test_protocol.py" --gate "$out/gate-$arm.jsonl" "$arm"
done
if [[ $mode == dry ]]; then
    python3 "$recipe/scalar.py" dry "$bin" "$other" "$corpus" "$out"
    echo "Untimed gate, two full binaries and all sixteen target settings exercised." > "$out/outcome.txt"
else
    python3 "$recipe/scalar.py" sweep "$bin" "$other" "$x4" "$out"
    python3 - "$out/decision.json" > "$out/outcome.txt" <<'PY'
import json,sys
d=json.load(open(sys.argv[1]))
print("Matched incremental criterion met =",d["incremental_primary"]["criterion_met"])
print("Separate per-worker stretch and memory criterion met =",d["i9_criterion_met"])
PY
fi
