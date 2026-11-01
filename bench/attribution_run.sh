#!/usr/bin/env bash
# The shipped decoder's one-to-four-worker resource attribution, one command
# (docs/journal/2026-11-01-decoder-attribution-protocol.md). Usage:
#   bash bench/attribution_run.sh HOST_LABEL      (or: just bench-attribution HOST_LABEL)
# Steps: the pinned clean checkout; rustc 1.97.0; the two builds (the occupancy instrument
# with `trace`, the uninstrumented decode without); C6, the default-feature decoder the
# same as the base commit's; the walk trial's frozen corpora, every hash checked; each
# corpus decoded at 1 and 4 workers against its frozen hash; the allocation (four
# physical cores, their siblings idle); a numbers-only environment; the event plan
# (`perf` probed: what counts without multiplexing, what is unavailable); the analysis's
# self-test on synthetic counts; then M1 (the timeline), M2 (counters), M3 (samples); the
# summary; the archive. Nothing else is run.
# Env: ATTR_DRY_RUN=1 (the whole path at 2 rounds and 1 repetition, into LABEL.dry-run:
# a check of the machinery, its figures not interpreted); ATTR_CORPORA (default the walk
# trial's work directory, target/walk-decoder/work/corpora); ATTR_RESULTS_DIR (default
# target/attribution-results); ATTR_WORK (default target/attribution-work); PERF (default
# `perf` as root, else `sudo -n perf`, asked for once); CARGO_TARGET_DIR (default target/).
set -euo pipefail
label=${1:?usage: attribution_run.sh HOST_LABEL}
[[ $label =~ ^[A-Za-z0-9._-]+$ ]] || { echo "label: letters, digits, . _ - only" >&2; exit 2; }
repo=$(git rev-parse --show-toplevel)
cd "$repo"
dir=docs/journal/2026-11-01-decoder-attribution
frozen=$dir/frozen.json
work=${ATTR_WORK:-$repo/target/attribution-work}
results_dir=${ATTR_RESULTS_DIR:-$repo/target/attribution-results}
dry=${ATTR_DRY_RUN:-}
out=$results_dir/$label
[[ $dry == 1 ]] && out=$results_dir/$label.dry-run
die() { echo "attribution: $*" >&2; exit 1; }
want() { python3 -I -c "import json,sys; print(json.load(open('$frozen'))$1)"; }

# 1. The pinned commit, a clean tree, a fresh output directory, the frozen sources.
[[ $(uname -sm) == "Linux x86_64" ]] || die "Linux x86_64 only"
[[ -z $(git status --porcelain --untracked-files=no) ]] || die "working tree not clean"
pin=$(cat bench/attribution-pin)
git cat-file -e "$pin^{commit}" 2>/dev/null || die "pinned commit $pin not present (fetch the branch)"
git diff --quiet "$pin" HEAD -- . ":(exclude)bench/attribution-pin" ":(exclude)$dir" || die "HEAD differs from pinned $pin"
if [[ $dry == 1 ]]; then rm -rf "$out" "$out.tar.gz"; fi
[[ ! -e $out ]] || die "$out exists: one run per label, no reruns"
mkdir -p "$out" "$work"
exec > >(tee "$out/run.log") 2>&1
echo "attribution: label=$label head=$(git rev-parse HEAD) pin=$pin out=$out dry_run=${dry:-0}"
python3 -I - "$frozen" <<'EOF'
import hashlib, json, sys
f = json.load(open(sys.argv[1]))
for p, h in f["sources_sha256"].items():
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == h, f"source changed: {p}"
print(f"attribution: {len(f['sources_sha256'])} sources match the frozen hashes")
EOF
rounds=$(want '["schedule"]["rounds"]'); stat_reps=$(want '["schedule"]["stat_reps"]'); rec_reps=$(want '["schedule"]["record_reps"]')
if [[ $dry == 1 ]]; then rounds=2; stat_reps=1; rec_reps=1; fi

# 2. rustc 1.97.0, empty RUSTFLAGS.
unset RUSTFLAGS CARGO_BUILD_RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo/target}
rustup toolchain list | grep -q '^1\.97\.0' || rustup toolchain install 1.97.0 --profile minimal
export RUSTUP_TOOLCHAIN=1.97.0
cargo="cargo +1.97.0"
echo "attribution: $(rustc +1.97.0 -V); CARGO_TARGET_DIR=$CARGO_TARGET_DIR"

# 3. The two builds of the same source, either side of the tracing split.
mkdir -p "$out/bin"
$cargo build --locked --release --features trace --example occupancy
cp "$CARGO_TARGET_DIR/release/examples/occupancy" "$out/bin/occupancy"
$cargo build --locked --release --example attribution_decode
cp "$CARGO_TARGET_DIR/release/examples/attribution_decode" "$out/bin/attribution_decode"
(cd "$out/bin" && sha256sum occupancy attribution_decode > ../binaries.sha256)
occ=$out/bin/occupancy; dec=$out/bin/attribution_decode
$cargo test --locked --release --features trace --test trace_phases > "$out/tests-trace.log" 2>&1
$cargo test --locked --release --features trace --example occupancy > "$out/tests-occupancy.log" 2>&1
echo "attribution: the trace's phase and feeder tests, and the instrument's, pass"

# 4. C6: the default-feature decoder is the base commit's (the same example built
#    against each library); the run stops on any difference.
base=$(want '["source_base"]')
rm -rf "$work/base"
git worktree remove --force "$work/base" 2>/dev/null || true
git worktree add --detach "$work/base" "$base" > /dev/null
cp examples/attribution_decode.rs "$work/base/examples/"
(cd "$work/base" && CARGO_TARGET_DIR=$work/base-target $cargo build --locked --release --example attribution_decode)
python3 -I bench/attribution_same_code.py "$work/base-target/release/examples/attribution_decode" "$dec" --out "$out/same-code.json"
git worktree remove --force "$work/base"

# 5. Corpora: the walk trial's frozen files, every hash checked; rebuilt by that trial's
#    own preparation (its frozen head, stopped before timing) where they are missing.
cdir=${ATTR_CORPORA:-$repo/target/walk-decoder/work/corpora}
check_corpora() {
    python3 -I - "$frozen" "$1" "$out/manifest.tsv" <<'EOF'
import hashlib, json, os, sys
f, d, out = json.load(open(sys.argv[1])), sys.argv[2], sys.argv[3]
rows = []
for name in f["order"]:
    c = f["corpora"][name]
    p = os.path.join(d, name + ".bz2")
    if not os.path.exists(p) or hashlib.sha256(open(p, "rb").read()).hexdigest() != c["compressed_sha256"]:
        sys.exit(1)
    rows.append("\t".join([name, c["role"], os.path.abspath(p), c["decoded_sha256"], str(c["decoded_bytes"])]))
open(out, "w").write("\n".join(rows) + "\n")
EOF
}
if ! check_corpora "$cdir"; then
    echo "attribution: corpora not in $cdir: rebuilding them with the walk trial's preparation"
    walk_head=$(want '["corpora_from"]["head"]')
    git cat-file -e "$walk_head^{commit}" 2>/dev/null || git fetch origin "$(want '["corpora_from"]["branch"]')"
    rm -rf "$work/walk"; git worktree remove --force "$work/walk" 2>/dev/null || true
    git worktree add --detach "$work/walk" "$walk_head" > /dev/null
    (cd "$work/walk" && WALK_WORK=$work/walk-work WALK_RESULTS_DIR=$work/walk-results WALK_STOP_BEFORE_TIMING=1 \
        CARGO_TARGET_DIR=$work/walk-target bash bench/walk_decoder_run.sh corpora > "$out/corpora-preparation.log" 2>&1)
    git worktree remove --force "$work/walk"
    cdir=$work/walk-work/corpora
    check_corpora "$cdir" || die "corpora: a file differs from its frozen hash"
fi
echo "attribution: $(wc -l < "$out/manifest.tsv") corpora match the frozen SHA-256 ($cdir)"

# 6. Correctness, untimed: each corpus decoded at 1 and 4 workers equals its frozen hash.
while IFS=$'\t' read -r name role path sha n; do
    for w in 1 4; do
        got=$("$dec" "$path" "$w" --stdout | sha256sum | cut -d' ' -f1)
        [[ $got == "$sha" ]] || die "$name at w = $w: decoded bytes differ from the frozen hash"
    done
done < "$out/manifest.tsv"
echo "attribution: every corpus decodes to its frozen hash at w = 1 and 4"

# 7. The allocation: four physical cores, one CPU each, their siblings idle.
python3 -I bench/attribution.py allocation --cores 4 > "$out/allocation.json"
alloc=$(python3 -I -c "import json; print(','.join(map(str, json.load(open('$out/allocation.json'))['alloc'])))")
ncpu=$(python3 -I -c "import json; print(len(json.load(open('$out/allocation.json'))['alloc']))")
[[ $ncpu == 4 ]] || die "fewer than four physical cores allowed"
echo "attribution: allocation CPUs $alloc"

# 8. The environment (numbers only: no process names or command lines).
python3 -I - "$out/environment.json" "$label" "$out/allocation.json" <<'EOF'
import json, platform, subprocess, sys
out, label, alloc = sys.argv[1:]
def sh(*a):
    try: return subprocess.check_output(a, text=True, stderr=subprocess.STDOUT).strip()
    except Exception as e: return f"unavailable: {e}"
def rd(p):
    try: return open(p).read().strip()
    except Exception as e: return f"unavailable: {e}"
a = json.load(open(alloc))
cpus = {}
for c in a["alloc"] + a["idle_siblings"]:
    d = f"/sys/devices/system/cpu/cpu{c}"
    cpus[c] = dict(siblings=rd(f"{d}/topology/thread_siblings_list"), governor=rd(f"{d}/cpufreq/scaling_governor"),
                   driver=rd(f"{d}/cpufreq/scaling_driver"), max_khz=rd(f"{d}/cpufreq/scaling_max_freq"))
json.dump(dict(label=label, allocation=a, cpus=cpus, lscpu=sh("lscpu"), topology=sh("lscpu", "-e"),
    smt_control=rd("/sys/devices/system/cpu/smt/control"), intel_no_turbo=rd("/sys/devices/system/cpu/intel_pstate/no_turbo"),
    nmi_watchdog=rd("/proc/sys/kernel/nmi_watchdog"), perf_event_paranoid=rd("/proc/sys/kernel/perf_event_paranoid"),
    kernel=platform.uname()._asdict(), rustc=sh("rustc", "+1.97.0", "-vV"), head=sh("git", "rev-parse", "HEAD"),
    loadavg=rd("/proc/loadavg"), ps=sh("ps", "-eo", "pid,psr,pcpu", "--sort=-pcpu")), open(out, "w"), indent=1)
EOF

# 9. perf: as root, or through sudo (asked for once, kept alive while the run lasts).
if [[ -z ${PERF:-} ]]; then
    if [[ $(id -u) == 0 ]]; then
        export PERF=perf CHOWN=""
    else
        echo "attribution: the counters need root (machine-wide memory-controller counts): sudo, once"
        sudo -v
        ( while true; do sudo -n true; sleep 60; done ) & keep=$!
        trap 'kill $keep 2>/dev/null || true' EXIT
        export PERF="sudo -n perf" CHOWN="sudo -n chown $(id -u):$(id -g)"
    fi
fi
echo "attribution: perf = $PERF ($($PERF --version 2>&1 | head -1))"
primary=$(awk -F'\t' '$2=="primary"{print $3}' "$out/manifest.tsv")
python3 -I bench/attribution.py events --exe "$dec" --file "$primary" --cpus "$alloc" --out "$out/events.json"

# 10. The analysis's self-test: the counter and sample steps and the summary on synthetic
#     perf output (bench/attribution_fakeperf.py), the decodes real, every count SYNTHETIC.
st=$out/selftest
mkdir -p "$st"
small=$(sort -t$'\t' -k5,5n "$out/manifest.tsv" | head -1)
awk -F'\t' -v OFS='\t' 'NR==FNR{s=$3; next} {$3=s; print}' <(echo "$small") "$out/manifest.tsv" > "$st/manifest.tsv"
( export PERF=$repo/bench/attribution_fakeperf.py CHOWN=""
  python3 -I bench/attribution.py events --exe "$dec" --file "$(cut -f3 <<< "$small")" --cpus "$alloc" --out "$st/events.json" > /dev/null
  python3 -I bench/attribution.py stat --plan "$st/events.json" --exe "$dec" --manifest "$st/manifest.tsv" --cpus "$alloc" --workers 1,4 --reps 1 --out "$st/stat.json" > /dev/null
  python3 -I bench/attribution.py record --plan "$st/events.json" --exe "$dec" --manifest "$st/manifest.tsv" --cpus "$alloc" --workers 1,4 --reps 1 --dir "$st/perf" --out "$st/record.json" > /dev/null
  for name in $(cut -f1 "$st/manifest.tsv"); do
      mkdir -p "$st/occupancy/$name/marks"
      OCCUPANCY_MARKS_DIR=$st/occupancy/$name/marks "$occ" "$(cut -f3 <<< "$small")" --threads 1,4 --rounds 2 \
          --placements "$alloc" --out "$st/occupancy/$name/occupancy.json" > /dev/null 2>&1
  done
  python3 -I bench/attribution.py summary --manifest "$st/manifest.tsv" --occupancy-dir "$st/occupancy" --stat "$st/stat.json" \
      --record "$st/record.json" --events "$st/events.json" --out "$st/result-synthetic.json" > /dev/null )
echo "SYNTHETIC: every count here is a fixed value from bench/attribution_fakeperf.py, not a measurement" > "$st/README"
rm -rf "$st/perf" "$st/occupancy"/*/marks
echo "attribution: the analysis's self-test (synthetic counts) passed"

# 11-13. M1, M2, M3, inside the allocation, with the host's state around them.
snap() {
    cat /proc/stat > "$out/stat-$1.txt"
    for p in cpu memory io; do cat /proc/pressure/$p > "$out/pressure-$p-$1.txt" 2>/dev/null || echo unavailable > "$out/pressure-$p-$1.txt"; done
}
snap before
t0=$(date +%s)
while IFS=$'\t' read -r name role path sha n; do
    mkdir -p "$out/occupancy/$name/marks"
    OCCUPANCY_MARKS_DIR=$out/occupancy/$name/marks "$occ" "$path" --threads 1,4 --rounds "$rounds" --placements "$alloc" \
        --out "$out/occupancy/$name/occupancy.json" --meta "corpus=$name" > "$out/occupancy/$name/report.txt" 2> "$out/occupancy/$name/stderr.txt"
done < "$out/manifest.tsv"
t1=$(date +%s)
python3 -I bench/attribution.py stat --plan "$out/events.json" --exe "$dec" --manifest "$out/manifest.tsv" --cpus "$alloc" \
    --workers 1,4 --reps "$stat_reps" --out "$out/stat.json"
python3 -I bench/attribution.py record --plan "$out/events.json" --exe "$dec" --manifest "$out/manifest.tsv" --cpus "$alloc" \
    --workers 1,4 --reps "$rec_reps" --dir "$out/perf" --out "$out/record.json"
t2=$(date +%s)
snap after
echo "{\"timeline_s\": $((t1 - t0)), \"counters_s\": $((t2 - t1))}" > "$out/timing.json"

# 14. The summary: the registered rules, the validity checks, the ranking.
python3 -I bench/attribution.py summary --manifest "$out/manifest.tsv" --occupancy-dir "$out/occupancy" --stat "$out/stat.json" \
    --record "$out/record.json" --events "$out/events.json" --host-before "$out/stat-before.txt" --host-after "$out/stat-after.txt" \
    --allocation "$out/allocation.json" --out "$out/result.json"
[[ $dry == 1 ]] && echo "DRY RUN: the machinery checked at 2 rounds and 1 repetition; no figure here is interpreted" > "$out/DRY-RUN"

# 15. The archive (the perf.data files' reports are in record.json; the files are kept).
for f in "$out"/occupancy/*/marks; do (cd "$f" && gzip -9 -- *.tsv); done
tar -czf "$out.tar.gz" -C "$results_dir" "$(basename "$out")"
echo "attribution: results: $out"
echo "attribution: archive: $out.tar.gz"
