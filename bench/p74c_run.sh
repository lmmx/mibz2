#!/usr/bin/env bash
# The P74-C confirmatory run, one command, the same on every host (protocol:
# docs/journal/2026-10-31-p74-c-confirmatory-protocol.md). Usage:
#   bash bench/p74c_run.sh HOST_LABEL        (or: just bench-p74c HOST_LABEL)
# Steps: pinned clean checkout; rustc 1.90.0; build; reproduce and hash-check every corpus;
# --check (all five arms); the handoff audit; the structural assembly audit of the exact
# binary; topology; one timed five-arm run pinned to one CPU; summary; tarball.
# Env: P74C_WORK (input cache, default ~/.cache/mibz2-p74c), P74C_OUT (results root,
# default ~/p74c-results), P74C_CPU (override the chosen CPU; recorded),
# P74C_STOP_BEFORE_TIMING=1 (preparation and checks only).
set -euo pipefail
label=${1:?usage: p74c_run.sh HOST_LABEL}
[[ $label =~ ^[A-Za-z0-9._-]+$ ]] || { echo "label: letters, digits, . _ - only" >&2; exit 2; }
repo=$(git rev-parse --show-toplevel)
cd "$repo"
frozen=docs/journal/2026-10-31-p74-c-confirmatory/frozen.json
work=${P74C_WORK:-${XDG_CACHE_HOME:-$HOME/.cache}/mibz2-p74c}
out=${P74C_OUT:-$HOME/p74c-results}/$label
die() { echo "p74c: $*" >&2; exit 1; }
hash_of() { sha256sum "$1" | cut -d' ' -f1; }
want() { python3 -c "import json,sys; print(json.load(open('$frozen'))$1)"; }

# 1. The pinned commit, a clean tree, a fresh output directory.
[[ $(uname -sm) == "Linux x86_64" ]] || die "Linux x86_64 only"
[[ -z $(git status --porcelain) ]] || die "working tree not clean"
pin=$(cat bench/p74c-pin)
git cat-file -e "$pin^{commit}" 2>/dev/null || die "pinned commit $pin not present (fetch full history)"
git diff --quiet "$pin" HEAD -- . ':(exclude)bench/p74c-pin' || die "HEAD differs from pinned $pin"
[[ ! -e $out ]] || die "$out exists: one timed run per output directory, no reruns"
mkdir -p "$out" "$work"
exec > >(tee "$out/run.log") 2>&1
echo "p74c: label=$label head=$(git rev-parse HEAD) pin=$pin out=$out"
python3 - "$frozen" <<'EOF'
import hashlib, json, sys
f = json.load(open(sys.argv[1]))
for p, h in f["sources_sha256"].items():
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == h, f"source changed: {p}"
print("p74c: harness sources match the frozen hashes")
EOF

# 2. rustc 1.90.0 (pinned), empty RUSTFLAGS.
unset RUSTFLAGS CARGO_BUILD_RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
export CARGO_TARGET_DIR=$repo/target
if ! command -v rustup >/dev/null; then
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi
rustup toolchain list | grep -q '^1\.90\.0' || rustup toolchain install 1.90.0 --profile minimal
cargo="cargo +1.90.0"
$cargo build --locked --release --example p74_experiment --example p74_suite
bin=$CARGO_TARGET_DIR/release/examples/p74_experiment
cp "$bin" "$out/p74_experiment"
bin=$out/p74_experiment   # the exact binary audited and timed

# 3. Corpora, every hash checked.
c=$work/corpora
mkdir -p "$c"
if [[ ! -s $c/wikidata-fitted.bz2 || $(hash_of "$c/wikidata-fitted.bz2") != "$(want '["corpora"]["wikidata-fitted"]')" ]]; then
    git fetch --depth 1 origin bench-data
    rm -rf "$work/fitted" && mkdir -p "$work/fitted"
    git archive FETCH_HEAD windows/wikidata-20261019 | tar -x -C "$work/fitted"
    python3 - "$work/fitted/windows/wikidata-20261019" <<'EOF'
import hashlib, json, sys
for w in json.load(open("bench/wikidata.json"))["windows"]:
    p = f"{sys.argv[1]}/{w['name']}.bin"
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == w["sha256"], p
EOF
    "$bin" --windows "$c/wikidata-fitted.bz2" "$work"/fitted/windows/wikidata-20261019/w*.bin
fi
if [[ ! -s $c/wikidata-heldout.bz2 || $(hash_of "$c/wikidata-heldout.bz2") != "$(want '["corpora"]["wikidata-heldout"]')" ]]; then
    mkdir -p "$work/heldout"
    python3 - "$work/heldout" <<'EOF'
import hashlib, json, subprocess, sys
spec = json.load(open("docs/journal/2026-10-31-p74-c-confirmatory/heldout/spec.json"))
for w in spec["windows"]:
    p = f"{sys.argv[1]}/{w['name']}.bin"
    try:
        ok = hashlib.sha256(open(p, "rb").read()).hexdigest() == w["sha256"]
    except FileNotFoundError:
        ok = False
    if not ok:
        r = f"{w['offset']}-{w['offset'] + w['length'] - 1}"
        subprocess.check_call(["curl", "-sSfL", "--retry", "5", "-r", r, "-o", p, spec["url"]])
        assert hashlib.sha256(open(p, "rb").read()).hexdigest() == w["sha256"], p
EOF
    "$bin" --windows "$c/wikidata-heldout.bz2" "$work"/heldout/x*.bin
fi
"$bin" --prepare "$c"
rm -rf "$c/suite"
"$CARGO_TARGET_DIR/release/examples/p74_suite" "$c/suite" 2> "$out/suite-generation.log"
{
    for n in wikidata-fitted wikidata-heldout random records mixed-independent text; do echo "$c/$n.bz2"; done
    cat "$c/suite/files.txt"
} > "$out/files.txt"
{
    echo wikidata-fitted wikidata-heldout random records mixed-independent text | tr ' ' '\n'
    sed 's/.*/adversarial/' "$c/suite/files.txt"
} > "$out/roles.txt"
python3 - "$frozen" "$out/files.txt" <<'EOF'
import hashlib, json, sys
f = json.load(open(sys.argv[1]))
want = dict(f["corpora"], **{s["name"]: s["compressed_sha256"] for s in f["suite"]})
paths = open(sys.argv[2]).read().split()
names = [p.rsplit("/", 1)[1][:-4] for p in paths]
assert names == f["order"], "corpus order differs from the frozen order"
for p, n in zip(paths, names):
    h = hashlib.sha256(open(p, "rb").read()).hexdigest()
    assert h == want[n], f"{n}: {h} != frozen {want[n]}"
print(f"p74c: {len(paths)} corpora match the frozen SHA-256")
EOF

# 4. Correctness, untimed: all five arms on every block; the actual handoffs.
for f in $(cat "$out/files.txt"); do bzip2 -t "$f"; done
"$bin" --check $(cat "$out/files.txt") > "$out/check.tsv"
P74_AUDIT_FILES="$out/files.txt" $cargo test --locked --release --lib audit_manifest_corpora \
    -- --ignored --nocapture > "$out/audit.log"
python3 bench/p74_experiment.py suite --dir "$c/suite" --files "$c/suite/files.txt" \
    --audit <(python3 - "$out/audit.log" <<'EOF'
import sys
lines = [l.split("\t") for l in open(sys.argv[1]) if l.startswith("AUDIT\t")]
for l in lines:
    if int(l[1]) >= 6:
        l[1] = str(int(l[1]) - 6)
        print("\t".join(l), end="")
EOF
) --out "$out/suite.json"

# 5. The structural assembly audit of the exact binary: abort on failure.
python3 bench/p74c_asm_audit.py "$bin" --out "$out/asm-audit.json"
objdump -d -C -Mintel --no-show-raw-insn "$bin" > "$out/p74_experiment.asm"

# 6. Topology, chosen from sysfs: a CPU other than 0 (and not 0's sibling) with an SMT
#    sibling, which stays idle. Without SMT the first such CPU, and that is recorded.
sys=/sys/devices/system/cpu
allowed=$(python3 -c "import os; print(' '.join(map(str, sorted(os.sched_getaffinity(0)))))")
siblings0=$(cat $sys/cpu0/topology/thread_siblings_list 2>/dev/null || echo 0)
cpu=${P74C_CPU:-}
if [[ -z $cpu ]]; then
    for x in $allowed; do
        s=$(cat $sys/cpu$x/topology/thread_siblings_list 2>/dev/null || echo "$x")
        [[ $x == 0 || $s == "$siblings0" ]] && continue
        if [[ $s == *[,-]* ]]; then cpu=$x; break; fi
        [[ -z ${fallback:-} ]] && fallback=$x
    done
    cpu=${cpu:-${fallback:-}}
fi
[[ -n $cpu ]] || die "no CPU other than CPU 0 available"
sib=$(cat $sys/cpu$cpu/topology/thread_siblings_list 2>/dev/null || echo "$cpu")
python3 - "$out/environment.json" "$cpu" "$sib" "$label" <<'EOF'
import json, os, platform, subprocess, sys
out, cpu, sib, label = sys.argv[1:]
def sh(*a):
    try: return subprocess.check_output(a, text=True, stderr=subprocess.STDOUT).strip()
    except Exception as e: return f"unavailable: {e}"
def rd(p):
    try: return open(p).read().strip()
    except Exception as e: return f"unavailable: {e}"
cg = rd("/proc/self/cgroup").split("::")[-1]
d = f"/sys/devices/system/cpu/cpu{cpu}"
json.dump(dict(label=label, cpu=int(cpu), smt_siblings=sib, smt_sibling_idle_by_policy=sib != cpu,
    no_smt_exposed=sib == cpu, lscpu=sh("lscpu"), topology=sh("lscpu", "-e"),
    smt_control=rd("/sys/devices/system/cpu/smt/control"),
    governor=rd(f"{d}/cpufreq/scaling_governor"), driver=rd(f"{d}/cpufreq/scaling_driver"),
    cur_khz=rd(f"{d}/cpufreq/scaling_cur_freq"), max_khz=rd(f"{d}/cpufreq/scaling_max_freq"),
    intel_no_turbo=rd("/sys/devices/system/cpu/intel_pstate/no_turbo"),
    boost=rd("/sys/devices/system/cpu/cpufreq/boost"),
    cgroup=cg, cpu_max=rd(f"/sys/fs/cgroup{cg}/cpu.max"), root_cpu_max=rd("/sys/fs/cgroup/cpu.max"),
    kernel=platform.uname()._asdict(), rustc=sh("rustc", "+1.90.0", "-vV"),
    head=sh("git", "rev-parse", "HEAD"), loadavg=rd("/proc/loadavg"),
    ps=sh("ps", "-eo", "pid,psr,pcpu,comm", "--sort=-pcpu")), open(out, "w"), indent=1)
EOF
echo "p74c: timing CPU $cpu (siblings $sib)"
python3 bench/p74_experiment.py manifest --files "$out/files.txt" --roles "$out/roles.txt" \
    --check "$out/check.tsv" --audit "$out/audit.log" --binary "$bin" --out "$out/manifest.json"

if [[ ${P74C_STOP_BEFORE_TIMING:-} == 1 ]]; then
    echo "p74c: P74C_STOP_BEFORE_TIMING=1: preparation and checks only, no timing"
    exit 0
fi

# 7. The one timed run.
snap() {
    cat /proc/stat > "$out/stat-$1.txt"
    for p in cpu memory io; do cat /proc/pressure/$p > "$out/pressure-$p-$1.txt" 2>/dev/null || echo unavailable > "$out/pressure-$p-$1.txt"; done
    cat "/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)/cpu.stat" > "$out/cpu.stat-$1.txt" 2>/dev/null || true
}
snap before
start=$(date +%s.%N)
P74_APPROVED_RUNNER="$label" taskset -c "$cpu" "$bin" --measure-approved $(cat "$out/files.txt") \
    > "$out/raw.tsv" 2> "$out/stderr.txt"
end=$(date +%s.%N)
snap after
echo "{\"wall_s\": $(python3 -c "print($end - $start)"), \"cpu\": $cpu}" > "$out/timing.json"

# 8. Summary and archive.
python3 bench/p74_experiment.py confirm --manifest "$out/manifest.json" --raw "$out/raw.tsv" \
    --out "$out/summary.json" | tee "$out/verdict.json"
tar -czf "$out.tar.gz" -C "$(dirname "$out")" "$label"
echo "p74c: done: $out.tar.gz"
