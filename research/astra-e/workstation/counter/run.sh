#!/usr/bin/env bash
set -euo pipefail
mode=${1:?expected dry or measure}
[[ $mode == dry || $mode == measure ]]
repo=$(git rev-parse --show-toplevel)
cd "$repo"
reg=$(git rev-parse HEAD)
protocol=research/astra-e/workstation/counter
common=research/astra-e/workstation
release=530ed404a014a414ef0b62adfd947fa14d4533e6
arms=(edge scalar both)
declare -A pins entries
pins[edge]=badb2aec3624bd65606c429b2e6f79e9d9d0050f
pins[scalar]=d0a42980e896211906234c7679b38760ae44a46e
pins[both]=50b8ecb8946a498601e3fd894ed84a2687df6cc8
entries[edge]=research/astra-e/edge-budget/candidate.rs
entries[scalar]=research/astra-e/front-fusion/cap-batch/worker/candidate.rs
entries[both]=research/astra-e/front-fusion/cap-counter/both.rs
fixed=("$protocol" "$common/assess.py" "$common/refresh.py" "$common/test_protocol.py" bench/bundle.sh .just/bench.just)
git diff --quiet HEAD -- "${fixed[@]}"
base=$repo/target/astra-e-cap-counter
mkdir -p "$base"
out=$base/run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir "$out"
exec > >(tee -a "$out/console.log") 2>&1
echo "$mode" > "$out/mode.txt"
echo "$reg" > "$out/registration.txt"
scratch=$(mktemp -d "$out/.build-XXXXXXXX")
finish() {
    status=$?
    cd "$repo"
    for arm in "${arms[@]}"; do git worktree remove --force "$scratch/$arm" >/dev/null 2>&1 || true; done
    rm -rf -- "$scratch"
    if (( status )); then echo "Protocol exited $status. Partial evidence." > "$out/outcome.txt"; fi
    git show "$reg:bench/bundle.sh" | bash -s -- "$out"
}
trap finish EXIT
export RUSTUP_TOOLCHAIN=1.99.1 PYTHONDONTWRITEBYTECODE=1
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS PYTHONOPTIMIZE
rustc -Vv > "$out/toolchain.txt"
cargo -Vv >> "$out/toolchain.txt"
python3 -V >> "$out/toolchain.txt"
lscpu > "$out/lscpu.txt"
lscpu -e > "$out/lscpu-e.txt"
cat /sys/devices/system/cpu/online > "$out/cpus-online.txt"
awk '/^Cpus_allowed_list:/' /proc/self/status > "$out/affinity.txt"
if [[ $mode == measure ]]; then
    python3 - <<'PY'
from pathlib import Path
import os
assert "i9-10900K" in Path("/proc/cpuinfo").read_text() and len(os.sched_getaffinity(0))==20, "measurement requires the 10900K with all 20 logical CPUs"
PY
fi
git cat-file -e "$release^{commit}" || git fetch origin tag v0.5.6
git ls-tree -r "$reg" -- "${fixed[@]}" > "$out/protocol-blobs.txt"
for arm in "${arms[@]}"; do
    tree=$scratch/$arm
    git worktree add -q --detach "$tree" "${pins[$arm]}"
    git -C "$tree" checkout "$reg" -- "$protocol" "$common/assess.py" "$common/test_protocol.py" "$common/refresh.py"
    git diff "$release" "${pins[$arm]}" -- src Cargo.toml Cargo.lock > "$out/$arm-against-v0.5.6.diff"
    (cd "$tree"
     bash research/astra-e/prepare.sh "${entries[$arm]}"
     git ls-tree HEAD -- src Cargo.toml Cargo.lock > "$out/$arm-tree.txt"
     sha256sum "${entries[$arm]}" > "$out/$arm-entry.sha256")
    echo "$arm ${pins[$arm]} ${entries[$arm]}" >> "$out/sources.txt"
done
cd "$scratch/both"
export CARGO_TARGET_DIR=$scratch/target
python3 "$protocol/panel.py" select "$out" "$scratch"
selected=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["selected_build"])' "$out/scorer-refresh.json")
for arm in "${arms[@]}"; do
    target=$CARGO_TARGET_DIR/$arm/$selected
    bin=$target/release/moonshot
    sha256sum "$bin" > "$out/$arm-binary.sha256"
    "$bin" info > "$out/$arm-info.json"
    git -C "$scratch/$arm" ls-files -s -- bench/moonshot .just Justfile > "$out/$arm-projection.txt"
    cmp "$out/edge-projection.txt" "$out/$arm-projection.txt"
    python3 "$protocol/sources.py" "$scratch/$arm" "$target/release/moonshot.d" "$out/$arm-compiled.json"
done
corpus=$scratch/wikidata.bz2
if [[ -f $repo/corpora/wikidata.bz2 ]]; then cp --reflink=auto "$repo/corpora/wikidata.bz2" "$corpus"; fi
bash bench/moonshot/corpus.sh "$corpus"
x4=$scratch/wikidata-x4.bz2
cat "$corpus" "$corpus" "$corpus" "$corpus" > "$x4"
sha256sum "$corpus" "$x4" > "$out/inputs.sha256"
[[ $(cut -d' ' -f1 "$out/inputs.sha256") == $'82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915\n0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e' ]]
python3 "$protocol/panel.py" plan > "$out/roster.json"
for arm in "${arms[@]}"; do
    target=$CARGO_TARGET_DIR/$arm/$selected
    entry=$scratch/$arm/${entries[$arm]}
    python3 bench/moonshot/moonshot.py score "$entry" --corpus "$corpus" --target-dir "$target" --no-build --no-timing --no-proxy --out "$out/gate-$arm" | tee "$out/report-$arm.txt"
    cmp <(grep '^scorer v' "$out/report-edge.txt") <(grep '^scorer v' "$out/report-$arm.txt")
    cases=$(python3 "$common/test_protocol.py" --gate "$out/gate-$arm/check.jsonl" candidate)
    (( cases >= 338 ))
done
for arm in lazy opend; do
    "$CARGO_TARGET_DIR/edge/$selected/release/moonshot" check --quick --arm "$arm" --wikidata "$corpus" > "$out/gate-$arm.jsonl"
    cases=$(python3 "$common/test_protocol.py" --gate "$out/gate-$arm.jsonl" "$arm")
    (( cases >= 98 ))
done
python3 "$protocol/panel.py" dry "$CARGO_TARGET_DIR" "$corpus" "$out"
if [[ $mode == measure ]]; then
    python3 "$protocol/panel.py" sweep "$CARGO_TARGET_DIR" "$x4" "$out"
    echo "Completed 800 paired measurements. See decision.json for each prespecified contrast." > "$out/outcome.txt"
else
    echo "Untimed rehearsal complete for three separately compiled kernels and 20 explicit configurations." > "$out/outcome.txt"
fi
