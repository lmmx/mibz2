#!/usr/bin/env bash
# The unrolled room walk's full-decoder trial, one command, the same on every host
# (protocol: docs/journal/2026-10-31-walk-unroll-decoder-protocol.md). Usage:
#   bash bench/walk_decoder_run.sh HOST_LABEL      (or: just bench-walk-decoder HOST_LABEL)
# Ported from the P74-C decoder trial's runner at v1.1 (bench/p74c_decoder_run.sh,
# research/p74c-decoder 3870c12). Steps: the pinned clean checkout; rustc 1.97.0 (the
# registered stage PASS's toolchain, CI run 41222735026); build; every corpus reproduced
# and hash-checked (range-guarded fetches), the shared ones identical to the P74-C trial's;
# correctness (bzip2 -t and -dc, both kernels' equivalence at every w, the arm's tests);
# the structural audit W1-W4 of the exact binary; the CPU allocation; a harness self-test
# with synthetic times; then ONE timed trial in the allocation; the summary; the archive.
# Env: WALK_RESULTS_DIR (results root, default bench/results/walk-decoder, git-ignored),
# WALK_WORK (input cache, default target/walk-decoder/work), WALK_WORKERS (override the
# worker counts, comma-separated; recorded), WALK_STOP_BEFORE_TIMING=1 (everything up to
# the timed trial, into LABEL.dry-run), CARGO_TARGET_DIR (default target/; recorded).
set -euo pipefail
label=${1:?usage: walk_decoder_run.sh HOST_LABEL}
[[ $label =~ ^[A-Za-z0-9._-]+$ ]] || { echo "label: letters, digits, . _ - only" >&2; exit 2; }
repo=$(git rev-parse --show-toplevel)
cd "$repo"
dir=docs/journal/2026-10-31-walk-unroll-decoder
frozen=$dir/frozen.json
work=${WALK_WORK:-$repo/target/walk-decoder/work}
results_dir=${WALK_RESULTS_DIR:-$repo/bench/results/walk-decoder}
dry=${WALK_STOP_BEFORE_TIMING:-}
out=$results_dir/$label
[[ $dry == 1 ]] && out=$results_dir/$label.dry-run
die() { echo "walk-decoder: $*" >&2; exit 1; }
want() { python3 -I -c "import json,sys; print(json.load(open('$frozen'))$1)"; }

# 1. The pinned commit, a clean tree, a fresh output directory.
[[ $(uname -sm) == "Linux x86_64" ]] || die "Linux x86_64 only"
[[ -z $(git status --porcelain) ]] || die "working tree not clean"
pin=$(cat bench/walk-decoder-pin)
git cat-file -e "$pin^{commit}" 2>/dev/null || die "pinned commit $pin not present (fetch full history)"
git diff --quiet "$pin" HEAD -- . ':(exclude)bench/walk-decoder-pin' || die "HEAD differs from pinned $pin"
if [[ $dry == 1 ]]; then rm -rf "$out" "$out.tar.gz"; fi
[[ ! -e $out ]] || die "$out exists: one timed run per label, no reruns"
mkdir -p "$out" "$work"
exec > >(tee "$out/run.log") 2>&1
echo "walk-decoder: label=$label head=$(git rev-parse HEAD) pin=$pin out=$out dry_run=${dry:-0}"
python3 -I - "$frozen" <<'EOF'
import hashlib, json, sys
f = json.load(open(sys.argv[1]))
for p, h in f["sources_sha256"].items():
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == h, f"source changed: {p}"
print(f"walk-decoder: {len(f['sources_sha256'])} harness sources match the frozen hashes")
EOF

# 2. rustc 1.97.0 (pinned: the stage PASS's toolchain), empty RUSTFLAGS.
unset RUSTFLAGS CARGO_BUILD_RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo/target}
if ! command -v rustup >/dev/null; then
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
fi
rustup toolchain list | grep -q '^1\.97\.0' || rustup toolchain install 1.97.0 --profile minimal
export RUSTUP_TOOLCHAIN=1.97.0
cargo="cargo +1.97.0"
echo "walk-decoder: CARGO_TARGET_DIR=$CARGO_TARGET_DIR; $(rustc +1.97.0 -V)"
$cargo build --locked --release --example walk_decoder
cp "$CARGO_TARGET_DIR/release/examples/walk_decoder" "$out/walk_decoder"
bin=$out/walk_decoder   # the exact binary audited and timed
sha256sum "$bin" | cut -d' ' -f1 > "$out/binary.sha256"

# 3. Corpora: reproduced, every hash checked (the fetches behind the range guard).
c=$work/corpora
mkdir -p "$c"
rebuild() { [[ ! -s $c/$1.bz2 || $(sha256sum "$c/$1.bz2" | cut -d' ' -f1) != "$(want "[\"corpora\"][\"$1\"][\"compressed_sha256\"]")" ]]; }
if rebuild wikidata-fitted; then
    git fetch --depth 1 origin bench-data
    rm -rf "$work/fitted" && mkdir -p "$work/fitted"
    git archive FETCH_HEAD windows/wikidata-20261019 | tar -x -C "$work/fitted"
    python3 -I - "$work/fitted/windows/wikidata-20261019" bench/wikidata.json <<'EOF'
import hashlib, json, sys
for w in json.load(open(sys.argv[2]))["windows"]:
    p = f"{sys.argv[1]}/{w['name']}.bin"
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == w["sha256"], p
EOF
    "$bin" --windows "$c/wikidata-fitted.bz2" "$work"/fitted/windows/wikidata-20261019/w*.bin
fi
if rebuild wikidata-heldout; then
    python3 -I bench/walk_decoder.py fetch --spec $dir/heldout-spec.json --dir "$work/heldout"
    "$bin" --windows "$c/wikidata-heldout.bz2" "$work"/heldout/x*.bin
fi
if rebuild wikidata-20261026; then
    python3 -I bench/walk_decoder.py fetch --spec $dir/different-dump/windows.json --dir "$work/diffdump"
    "$bin" --windows "$c/wikidata-20261026.bz2" "$work"/diffdump/d*.bin
fi
# text: from the v1.0 pin's sources (amendment 1), so harness edits leave it unchanged
text_src=$work/text-src
rm -rf "$text_src" && mkdir -p "$text_src"
git archive "$(want '["text_source_commit"]')" src tests examples README.md | tar -x -C "$text_src"
"$bin" --prepare "$c" "$text_src"
for n in wikidata-fitted wikidata-heldout wikidata-20261026 random records mixed-independent text \
    text-l1 text-l5 zeros period-3; do echo "$c/$n.bz2"; done > "$out/files.txt"
python3 -I bench/walk_decoder.py manifest --frozen "$frozen" --files "$out/files.txt" --out "$out/manifest.tsv"

# 4. Correctness, untimed.
python3 -I - "$out/manifest.tsv" <<'EOF'
import hashlib, subprocess, sys
for l in open(sys.argv[1]).read().splitlines():
    name, role, path, _, decoded, n = l.split("\t")
    subprocess.run(["bzip2", "-t", path], check=True)
    d = subprocess.run(["bzip2", "-dc", path], check=True, capture_output=True).stdout
    assert len(d) == int(n) and hashlib.sha256(d).hexdigest() == decoded, f"{name}: bzip2 -dc differs from the frozen hash"
print("walk-decoder: bzip2 -t and bzip2 -dc match the frozen output hashes")
EOF
workers=${WALK_WORKERS:-}
alloc_json=$out/allocation.json
python3 -I bench/walk_decoder.py allocation > "$alloc_json"
alloc=$(python3 -I -c "import json; print(','.join(map(str, json.load(open('$alloc_json'))['alloc'])))")
if [[ -z $workers ]]; then
    workers=$(python3 -I -c "import json; n = len(json.load(open('$alloc_json'))['alloc']); print(','.join(map(str, range(1, min(4, n) + 1))))")
fi
echo "walk-decoder: allocation CPUs $alloc; workers $workers"
taskset -c "$alloc" "$bin" --check "$out/manifest.tsv" "$workers" > "$out/check.tsv"
$cargo test --locked --release --lib unrolled > "$out/tests-lib.log"
$cargo test --locked --release --test segmented_unrolled > "$out/tests-decoder.log"
echo "walk-decoder: the arm's tests pass"

# 5. The structural audit of the exact binary: abort on failure.
python3 -I bench/walk_decoder_audit.py "$bin" --base "$(want '["source_base"]')" --out "$out/asm-audit.json"
objdump -d -C -Mintel --no-show-raw-insn "$bin" > "$out/walk_decoder.asm"
# W4 (c): an inlining difference in the per-block driver needs the maintainer's recorded
# review (frozen.json "w4_review", the same signature) before any timing
python3 -I - "$out/asm-audit.json" "$frozen" "${dry:-0}" <<'EOF'
import json, sys
audit, frozen, dry = json.load(open(sys.argv[1])), json.load(open(sys.argv[2])), sys.argv[3] == "1"
r = audit["report"]["W4c_review_required"]
if not r["required"]:
    print("walk-decoder: W4 (c): no driver inlining difference")
    sys.exit(0)
accepted = frozen.get("w4_review", {})
if accepted.get("signature") == r["signature"]:
    print(f"walk-decoder: W4 (c): the driver inlining difference was reviewed and accepted: {accepted.get('record')}")
elif dry:
    print(f"walk-decoder: W4 (c): REVIEW REQUIRED before any timing: {r['signature']}")
else:
    sys.exit(f"walk-decoder: W4 (c): review required and not recorded as accepted: {r['signature']}")
EOF

# 6. The environment.
python3 -I - "$out/environment.json" "$label" "$alloc_json" <<'EOF'
import json, platform, subprocess, sys
out, label, alloc = sys.argv[1:]
def sh(*a):
    try: return subprocess.check_output(a, text=True, stderr=subprocess.STDOUT).strip()
    except Exception as e: return f"unavailable: {e}"
def rd(p):
    try: return open(p).read().strip()
    except Exception as e: return f"unavailable: {e}"
a = json.load(open(alloc))
cg = rd("/proc/self/cgroup").split("::")[-1]
cpus = {}
for c in a["alloc"] + a["idle_siblings"]:
    d = f"/sys/devices/system/cpu/cpu{c}"
    cpus[c] = dict(siblings=rd(f"{d}/topology/thread_siblings_list"), governor=rd(f"{d}/cpufreq/scaling_governor"),
                   driver=rd(f"{d}/cpufreq/scaling_driver"), cur_khz=rd(f"{d}/cpufreq/scaling_cur_freq"),
                   max_khz=rd(f"{d}/cpufreq/scaling_max_freq"))
json.dump(dict(label=label, allocation=a, cpus=cpus, lscpu=sh("lscpu"), topology=sh("lscpu", "-e"),
    smt_control=rd("/sys/devices/system/cpu/smt/control"),
    intel_no_turbo=rd("/sys/devices/system/cpu/intel_pstate/no_turbo"),
    boost=rd("/sys/devices/system/cpu/cpufreq/boost"),
    cgroup=cg, cpu_max=rd(f"/sys/fs/cgroup{cg}/cpu.max"), root_cpu_max=rd("/sys/fs/cgroup/cpu.max"),
    kernel=platform.uname()._asdict(), rustc=sh("rustc", "+1.97.0", "-vV"),
    head=sh("git", "rev-parse", "HEAD"), loadavg=rd("/proc/loadavg"),
    ps=sh("ps", "-eo", "pid,psr,pcpu,comm", "--sort=-pcpu")), open(out, "w"), indent=1)
EOF

# 7. The harness self-test: the whole trial path (children, wait4, affinity, summary) on
#    the four smallest corpora's files under every corpus's name, every figure synthetic.
st=$out/selftest
mkdir -p "$st"
python3 -I - "$out/manifest.tsv" "$st/manifest.tsv" <<'EOF'
import sys
rows = [l.split("\t") for l in open(sys.argv[1]).read().splitlines()]
small = sorted(rows, key=lambda r: int(r[5]))[:4]
with open(sys.argv[2], "w") as f:
    for i, r in enumerate(rows):
        f.write("\t".join([r[0], r[1], *small[i % len(small)][2:]]) + "\n")
EOF
WALK_SYNTHETIC_TIMES=1 taskset -c "$alloc" "$bin" --trial "$st/manifest.tsv" "$workers" "$alloc" > "$st/raw-synthetic.tsv"
python3 -I bench/walk_decoder.py summary --manifest "$st/manifest.tsv" --raw "$st/raw-synthetic.tsv" \
    --allocation "$alloc_json" --out "$st/summary-synthetic.json" > /dev/null
echo "SYNTHETIC: every time and CPU figure here is a fixed value, not a measurement" > "$st/README"
echo "walk-decoder: harness self-test (synthetic times) passed"

if [[ $dry == 1 ]]; then
    echo "walk-decoder: WALK_STOP_BEFORE_TIMING=1: preparation and checks only, no timing"
    echo "walk-decoder: results: $out"
    exit 0
fi

# 8. The one timed trial, the whole process tree in the allocation.
snap() {
    cat /proc/stat > "$out/stat-$1.txt"
    for p in cpu memory io; do cat /proc/pressure/$p > "$out/pressure-$p-$1.txt" 2>/dev/null || echo unavailable > "$out/pressure-$p-$1.txt"; done
    cat "/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)/cpu.stat" > "$out/cpu.stat-$1.txt" 2>/dev/null || true
}
snap before
start=$(date +%s.%N)
WALK_APPROVED_RUNNER="$label" taskset -c "$alloc" "$bin" --trial "$out/manifest.tsv" "$workers" "$alloc" \
    > "$out/raw.tsv" 2> "$out/stderr.txt"
end=$(date +%s.%N)
snap after
echo "{\"wall_s\": $(python3 -I -c "print($end - $start)"), \"alloc\": \"$alloc\", \"workers\": \"$workers\"}" > "$out/timing.json"

# 9. Summary and archive.
python3 -I bench/walk_decoder.py summary --manifest "$out/manifest.tsv" --raw "$out/raw.tsv" \
    --allocation "$alloc_json" --stat-before "$out/stat-before.txt" --stat-after "$out/stat-after.txt" \
    --out "$out/summary.json" | tee "$out/verdict.json"
gzip -k "$out/raw.tsv"
tar -czf "$out.tar.gz" -C "$(dirname "$out")" "$label"
echo "walk-decoder: results: $out"
echo "walk-decoder: archive: $out.tar.gz"
