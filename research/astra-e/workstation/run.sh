#!/usr/bin/env bash
# Proposed independent run. Select the tracked candidate when freezing this recipe.
set -euo pipefail
mode=${1:?usage: run.sh dry|measure path/to/candidate.rs}
candidate=${2:?provide the frozen repository-relative candidate path}
[[ $mode == dry || $mode == measure ]]
cd "$(git rev-parse --show-toplevel)"
repo=$PWD
recipe=research/astra-e/workstation
score=bench/moonshot
base=$repo/target/astra-e
mkdir -p "$base"
out=$base/run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir "$out"
exec > >(tee -a "$out/console.log") 2>&1
printf '%s\n' "$mode" > "$out/mode.txt"
# This must be a clean disposable research checkout at the Coordinator's frozen SHA.
git diff --quiet HEAD --
git ls-files --error-unmatch -- "$candidate"
if [[ -n $(git ls-files --others --exclude-standard -- src "$score" research/astra-e) ]]; then
    echo 'Untracked research or source inputs are present in the frozen checkout.' >&2
    exit 1
fi
git fetch origin master
if ! git diff --quiet HEAD origin/master -- "$score" .just Justfile; then
    echo 'The scorer or recipes changed after freeze. Return to local calibration and refreeze.' >&2
    exit 1
fi
git checkout origin/master -- bench/moonshot .just Justfile
release=530ed404a014a414ef0b62adfd947fa14d4533e6
git cat-file -e "$release^{commit}" || git fetch origin tag v0.5.6
git diff "$release" HEAD -- src Cargo.toml > "$out/src-against-v0.5.6.diff"
git rev-parse HEAD origin/master "$release" > "$out/commits.txt"
git log -1 --format=%h origin/master -- bench/moonshot > "$out/scorer-revision.txt"
git rev-parse HEAD:bench/moonshot > "$out/scorer-tree.txt"
git ls-tree -r HEAD -- "$score" "$recipe" "$candidate" > "$out/source-blobs.txt"
sha256sum "$candidate" "$score/moonshot.py" "$recipe/assess.py" > "$out/sources.sha256"
export RUSTUP_TOOLCHAIN=1.99.1
export PYTHONDONTWRITEBYTECODE=1
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
rustc -Vv > "$out/toolchain.txt"
cargo -Vv >> "$out/toolchain.txt"
python3 -V >> "$out/toolchain.txt"
lscpu > "$out/lscpu.txt"
lscpu -e > "$out/lscpu-e.txt"
cat /sys/devices/system/cpu/online > "$out/cpus-online.txt"
awk '/^Cpus_allowed_list:/' /proc/self/status > "$out/affinity.txt"
if [[ $mode == measure ]]; then
    python3 - <<'PY'
import os
from pathlib import Path
if 'i9-10900K' not in Path('/proc/cpuinfo').read_text() or len(os.sched_getaffinity(0)) != 20:
    raise SystemExit('Measured run requires the unpinned i9-10900K with all 20 CPUs available.')
PY
fi
scratch=$(mktemp -d "$out/.build-XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
export CARGO_TARGET_DIR=$scratch/target
export MOONSHOT_CANDIDATE=$repo/$candidate
cargo build --release --locked --manifest-path "$score/Cargo.toml"
bin=$CARGO_TARGET_DIR/release/moonshot
"$bin" info | tee "$out/scorer-info.json"
sha256sum "$bin" > "$out/binaries.sha256"
base_corpus=$scratch/wikidata.bz2
x4=$scratch/wikidata-x4.bz2
bash "$score/corpus.sh" "$base_corpus"
cat "$base_corpus" "$base_corpus" "$base_corpus" "$base_corpus" > "$x4"
sha256sum "$base_corpus" "$x4" > "$out/inputs.sha256"
[[ $(sha256sum "$x4" | cut -d' ' -f1) == 0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e ]]
python3 "$recipe/assess.py" --plan > "$out/roster.json"
# Full, untimed differential gate for the candidate. Capture the scorer report and raw cases.
python3 "$score/moonshot.py" score "$candidate" --corpus "$base_corpus" --target-dir "$CARGO_TARGET_DIR" \
    --no-build --no-timing --no-proxy --out "$out/candidate-gate" | tee "$out/scorer-report.txt"
for arm in lazy opend open; do
    "$bin" check --quick --arm "$arm" --wikidata "$base_corpus" > "$out/gate-$arm.jsonl"
done
if [[ $mode == dry ]]; then
    # The pinned gate tests 1, 2 and available_parallelism() workers. Restrict only
    # these untimed checks so its third count covers every measured W on the i9.
    python3 - "$bin" "$base_corpus" "$out" <<'PYDRY'
import json, os, subprocess, sys
from pathlib import Path
bin_path, corpus, out = sys.argv[1:]
cpus = sorted(os.sched_getaffinity(0))
for w in (7, 8, 9, 10):
    if len(cpus) < w:
        print(f"Cloud rehearsal lacks {w} CPUs for this untimed worker gate")
        continue
    prefix = ["taskset", "-c", ",".join(map(str, cpus[:w])), bin_path]
    info = json.loads(subprocess.check_output([*prefix, "info"]))
    if info["cpus"] != w:
        print(f"Available parallelism {info['cpus']} cannot exercise the W={w} untimed gate")
        continue
    with (Path(out) / f"gate-candidate-w{w}.jsonl").open("w") as f:
        subprocess.run([*prefix, "check", "--quick", "--arm", "candidate", "--wikidata", corpus],
                       stdout=f, check=True)
PYDRY
    echo 'Untimed. Candidate and control gates passed. Roster recorded.' > "$out/outcome.txt"
else
    python3 "$score/moonshot.py" sweep --target-dir "$CARGO_TARGET_DIR" --corpus "$x4" --out "$out" \
        --arms lazy,opend,open,candidate --workers 7,8,9,10 --rounds 16 --settle 2 | tee "$out/sweep.txt"
    python3 "$recipe/assess.py" "$out/runs.jsonl" > "$out/decision.json"
    python3 -c 'import json,sys; print("i9 criterion met =",json.load(open(sys.argv[1]))["i9_criterion_met"])' \
        "$out/decision.json" > "$out/outcome.txt"
fi
rm -rf -- "$scratch"
trap - EXIT
bash bench/bundle.sh "$out"
