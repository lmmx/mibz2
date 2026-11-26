#!/usr/bin/env bash
# Proposed independent run. Select the tracked candidate when freezing this recipe.
set -euo pipefail
mode=${1:?usage: run.sh dry|measure path/to/candidate.rs}
candidate=${2:?provide the frozen repository-relative candidate path}
[[ $mode == dry || $mode == measure ]]
matched=${3:-}
[[ -z $matched || ( $matched == --edge-matched && $candidate == research/astra-e/edge-budget/candidate.rs ) ]]
edge_revision=badb2aec3624bd65606c429b2e6f79e9d9d0050f
measured_revision=8c9640d8ab186f81a0500f48e606e0c21c2a6669
predecessor_entry=research/astra-e/narrow-short/ahead.rs
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
# The Coordinator's registered Justfile stays in its checkout. Only the new,
# detached measurement worktree receives the live master path refresh.
git ls-files --error-unmatch -- "$candidate"
git diff --quiet HEAD -- "$recipe/run.sh"
frozen=$(git rev-parse HEAD)
scratch=$(mktemp -d "$out/.build-XXXXXXXX")
tree=$scratch/source
other=$scratch/predecessor
cleanup() {
    git -C "$repo" worktree remove --force "$tree" >/dev/null 2>&1 || true
    if [[ -n $matched ]]; then git -C "$repo" worktree remove --force "$other" >/dev/null 2>&1 || true; fi
    rm -rf -- "$scratch"
}
finish() {
    status=$?
    cd "$repo"
    cleanup
    if (( status != 0 )); then
        printf 'Recipe exited %s. Review the console log and partial evidence.\n' "$status" > "$out/outcome.txt"
        git show "$frozen:bench/bundle.sh" | bash -s -- "$out" || true
    fi
}
trap finish EXIT
if [[ -n $matched ]]; then
    # Keep every decoder and generated kernel input at the source actually compared.
    git worktree add -q --detach "$tree" "$edge_revision"
    git worktree add -q --detach "$other" "$measured_revision"
    git -C "$tree" checkout "$frozen" -- "$recipe" research/astra-e/ci/paired.py
    git -C "$other" checkout "$frozen" -- "$recipe/refresh.py" "$recipe/test_protocol.py" "$recipe/assess.py"
    (cd "$other" && bash research/astra-e/prepare.sh "$predecessor_entry")
    {
        echo "recipe $frozen"
        echo "edge $edge_revision $candidate"
        echo "preceding combined source 80d8409359214b4e5abfd4430213921e82e9cd30"
        echo "measured combined $measured_revision $predecessor_entry"
        git -C "$tree" ls-tree HEAD -- src Cargo.toml Cargo.lock
        git -C "$other" ls-tree HEAD -- src Cargo.toml Cargo.lock
        sha256sum "$tree/$candidate" "$other/$predecessor_entry"
        git diff --exit-code 80d8409359214b4e5abfd4430213921e82e9cd30 "$measured_revision" -- "$predecessor_entry"
        git diff 80d8409359214b4e5abfd4430213921e82e9cd30 "$measured_revision" -- src Cargo.toml Cargo.lock > "$out/predecessor-library-against-source.diff"
    } > "$out/matched-sources.txt"
else
    git worktree add -q --detach "$tree" "$frozen"
fi
cd "$tree"
release=530ed404a014a414ef0b62adfd947fa14d4533e6
git cat-file -e "$release^{commit}" || git fetch origin tag v0.5.6
git diff "$release" HEAD -- src Cargo.toml > "$out/src-against-v0.5.6.diff"
git ls-tree -r "$frozen" -- "$recipe" research/astra-e/ci/paired.py > "$out/source-blobs.txt"
git ls-tree -r HEAD -- "$candidate" >> "$out/source-blobs.txt"
export RUSTUP_TOOLCHAIN=1.99.1
export PYTHONDONTWRITEBYTECODE=1
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
import os
from pathlib import Path
if 'i9-10900K' not in Path('/proc/cpuinfo').read_text() or len(os.sched_getaffinity(0)) != 20:
    raise SystemExit('Measured run requires the unpinned i9-10900K with all 20 CPUs available.')
PY
fi
export CARGO_TARGET_DIR=$scratch/target
export MOONSHOT_CANDIDATE=$tree/$candidate
bash research/astra-e/prepare.sh "$candidate"
if [[ -n $matched ]]; then
    python3 "$recipe/edge.py" select "$out" "$other"
else
    python3 "$recipe/refresh.py" "$out"
fi
git rev-parse HEAD origin/master "$release" > "$out/commits.txt"
selected=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["selected_build"])' "$out/scorer-refresh.json")
predecessor_target=$CARGO_TARGET_DIR/predecessor/$selected
export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/$selected"
git ls-files -s -- "$score" .just Justfile > "$out/scorer-blobs.txt"
sha256sum "$candidate" "$score/moonshot.py" "$recipe/assess.py" > "$out/sources.sha256"
bin=$CARGO_TARGET_DIR/release/moonshot
"$bin" info | tee "$out/scorer-info.json"
if [[ $mode == measure ]]; then
    # A 20-CPU affinity mask can coexist with a restrictive container CPU quota.
    python3 -c 'import json, sys; sys.exit(0 if json.load(open(sys.argv[1]))["cpus"] == 20 else "Available parallelism below the 20-CPU target.")' "$out/scorer-info.json"
fi
sha256sum "$bin" > "$out/binaries.sha256"
base_corpus=$scratch/wikidata.bz2
x4=$scratch/wikidata-x4.bz2
bash "$score/corpus.sh" "$base_corpus"
cat "$base_corpus" "$base_corpus" "$base_corpus" "$base_corpus" > "$x4"
sha256sum "$base_corpus" "$x4" > "$out/inputs.sha256"
[[ $(sha256sum "$x4" | cut -d' ' -f1) == 0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e ]]
controls=(lazy opend open)
control_bin=$bin
predecessor_bin=
if [[ -n $matched ]]; then
    predecessor_bin=$predecessor_target/release/moonshot
    control_bin=$predecessor_bin
    controls=(lazy opend)
    "$predecessor_bin" info > "$out/predecessor-info.json"
    if [[ $mode == measure ]]; then
        python3 -c 'import json, sys; sys.exit(0 if json.load(open(sys.argv[1]))["cpus"] == 20 else "Measured predecessor requires 20 CPUs.")' "$out/predecessor-info.json"
    fi
    sha256sum "$predecessor_bin" >> "$out/binaries.sha256"
    git -C "$other" ls-files -s -- "$score" .just Justfile > "$out/predecessor-scorer-blobs.txt"
    cmp "$out/scorer-blobs.txt" "$out/predecessor-scorer-blobs.txt"
    python3 "$recipe/edge.py" plan > "$out/roster.json"
    python3 "$score/moonshot.py" score "$other/$predecessor_entry" --corpus "$base_corpus" --target-dir "$predecessor_target" \
        --no-build --no-timing --no-proxy --out "$out/predecessor-gate" | tee "$out/predecessor-report.txt"
    python3 "$recipe/test_protocol.py" --gate "$out/predecessor-gate/check.jsonl" candidate
else
    python3 "$recipe/assess.py" --plan > "$out/roster.json"
fi
# Full, untimed differential gate for the candidate. Capture the scorer report and raw cases.
python3 "$score/moonshot.py" score "$candidate" --corpus "$base_corpus" --target-dir "$CARGO_TARGET_DIR" \
    --no-build --no-timing --no-proxy --out "$out/candidate-gate" | tee "$out/scorer-report.txt"
python3 "$recipe/test_protocol.py" --gate "$out/candidate-gate/check.jsonl" candidate
for arm in "${controls[@]}"; do
    "$control_bin" check --quick --arm "$arm" --wikidata "$base_corpus" > "$out/gate-$arm.jsonl"
    python3 "$recipe/test_protocol.py" --gate "$out/gate-$arm.jsonl" "$arm"
done
if [[ $mode == dry ]]; then
    # The pinned gate tests 1, 2 and available_parallelism() workers. Restrict only
    # these untimed checks so its third count covers every measured W on the i9.
    python3 - "$bin" "$base_corpus" "$out" "$predecessor_bin" <<'PYDRY'
import json, os, subprocess, sys
from pathlib import Path
bin_path, corpus, out, predecessor = sys.argv[1:]
checks = [("candidate", "candidate", bin_path)]
if predecessor:
    checks.extend([("predecessor", "candidate", predecessor), ("lazy", "lazy", predecessor), ("opend", "opend", predecessor)])
cpus = sorted(os.sched_getaffinity(0))
is_target = "i9-10900K" in Path("/proc/cpuinfo").read_text() and len(cpus) == 20
exercised = []
for w in (7, 8, 9, 10):
    if len(cpus) < w:
        print(f"Cloud rehearsal lacks {w} CPUs for this untimed worker gate")
        continue
    prefix = ["taskset", "-c", ",".join(map(str, cpus[:w])), bin_path]
    info = json.loads(subprocess.check_output([*prefix, "info"]))
    if info["cpus"] != w:
        print(f"Available parallelism {info['cpus']} cannot exercise the W={w} untimed gate")
        if is_target:
            raise SystemExit(f"Target dry run cannot validate W={w}")
        continue
    for label, arm, binary in checks:
        cmd = [*prefix[:-1], binary, "check", "--quick", "--arm", arm, "--wikidata", corpus]
        gate = Path(out) / f"gate-{label}-w{w}.jsonl"
        with gate.open("w") as f:
            subprocess.run(cmd, stdout=f, check=True)
        subprocess.run([sys.executable, "research/astra-e/workstation/test_protocol.py",
                        "--gate", str(gate), arm], check=True)
    exercised.append(w)
(Path(out) / "dry-coverage.json").write_text(json.dumps({
    "target_machine": is_target, "gate_arms": [label for label, _, _ in checks], "target_worker_counts_tested": exercised,
    "all_target_worker_counts_tested": exercised == [7, 8, 9, 10],
}, indent=2) + "\n")
PYDRY
    echo 'Untimed. Candidate and control gates passed. Roster recorded.' > "$out/outcome.txt"
else
    if [[ -n $matched ]]; then
        python3 "$recipe/edge.py" sweep "$bin" "$predecessor_bin" "$x4" "$out"
    else
        python3 "$score/moonshot.py" sweep --target-dir "$CARGO_TARGET_DIR" --corpus "$x4" --out "$out" \
            --arms lazy,opend,open,candidate --workers 7,8,9,10 --rounds 16 --settle 2 | tee "$out/sweep.txt"
        python3 "$recipe/assess.py" "$out/runs.jsonl" > "$out/decision.json"
    fi
    python3 -c 'import json,sys; print("i9 criterion met =",json.load(open(sys.argv[1]))["i9_criterion_met"])' \
        "$out/decision.json" > "$out/outcome.txt"
fi
cd "$repo"
cleanup
trap - EXIT
git show "$frozen:bench/bundle.sh" | bash -s -- "$out"
