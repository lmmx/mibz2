#!/usr/bin/env bash
# The dispatch-causality diagnostic, one command (docs/journal/2026-11-01-dispatch-causality-protocol.md):
#   bash bench/dispatch_run.sh HOST_LABEL       (or: just bench-dispatch HOST_LABEL)
# Steps: the pinned clean checkout; rustc 1.97.0; the two builds (the occupancy instrument
# with `trace`, the uninstrumented decode without); C6 (the default-feature decoder is the
# base commit's); the tests; the walk trial's frozen corpora, every hash checked; each
# corpus decoded at 1 and 4 workers against its frozen hash; the allocation; a numbers-only
# environment; the tracepoints probed (missing ones recorded); then the capture: paired
# runs (uninstrumented, then instrumented, the order alternating), three at w = 4 to choose
# the instrumentation tier by its perturbation (tier A; tier B without futex events if A
# perturbs more than 3%), three at w = 1, and seven more at w = 4 only if the quality
# checks pass on the first three; the analysis; the summary; the archive.
# Env: DISPATCH_DRY_RUN=1 (the whole path, one repetition and one expansion, into
# LABEL.dry-run; nothing interpreted); DISPATCH_CORPORA (default the walk trial's work
# directory); DISPATCH_RESULTS_DIR (default target/dispatch-results); DISPATCH_WORK
# (default target/dispatch-work); PERF (default `perf` as root, else `sudo -n perf`).
set -euo pipefail
label=${1:?usage: dispatch_run.sh HOST_LABEL}
[[ $label =~ ^[A-Za-z0-9._-]+$ ]] || { echo "label: letters, digits, . _ - only" >&2; exit 2; }
repo=$(git rev-parse --show-toplevel)
cd "$repo"
dir=docs/journal/2026-11-01-dispatch-causality
frozen=$dir/frozen.json
work=${DISPATCH_WORK:-$repo/target/dispatch-work}
results_dir=${DISPATCH_RESULTS_DIR:-$repo/target/dispatch-results}
dry=${DISPATCH_DRY_RUN:-}
out=$results_dir/$label
[[ $dry == 1 ]] && out=$results_dir/$label.dry-run
die() { echo "dispatch: $*" >&2; exit 1; }
want() { python3 -I -c "import json,sys; print(json.load(open('$frozen'))$1)"; }

# 1. The pinned commit, a clean tree, a fresh output directory, the frozen sources.
[[ $(uname -sm) == "Linux x86_64" ]] || die "Linux x86_64 only"
[[ -z $(git status --porcelain --untracked-files=no) ]] || die "working tree not clean"
pin=$(cat bench/dispatch-pin)
git cat-file -e "$pin^{commit}" 2>/dev/null || die "pinned commit $pin not present (fetch the branch)"
git diff --quiet "$pin" HEAD -- . ":(exclude)bench/dispatch-pin" ":(exclude)$dir" || die "HEAD differs from pinned $pin"
if [[ $dry == 1 ]]; then rm -rf "$out" "$out.tar.gz"; fi
[[ ! -e $out ]] || die "$out exists: one run per label, no reruns"
mkdir -p "$out" "$work"
exec > >(tee "$out/run.log") 2>&1
echo "dispatch: label=$label head=$(git rev-parse HEAD) pin=$pin out=$out dry_run=${dry:-0}"
python3 -I - "$frozen" <<'EOF'
import hashlib, json, sys
f = json.load(open(sys.argv[1]))
for p, h in f["sources_sha256"].items():
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == h, f"source changed: {p}"
print(f"dispatch: {len(f['sources_sha256'])} sources match the frozen hashes")
EOF
initial=$(want '["schedule"]["initial_w4"]'); w1reps=$(want '["schedule"]["w1"]'); total=$(want '["schedule"]["expanded_w4"]')
if [[ $dry == 1 ]]; then initial=1; w1reps=1; total=2; fi

# 2. rustc 1.97.0, empty RUSTFLAGS.
unset RUSTFLAGS CARGO_BUILD_RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo/target}
rustup toolchain list | grep -q '^1\.97\.0' || rustup toolchain install 1.97.0 --profile minimal
export RUSTUP_TOOLCHAIN=1.97.0
cargo="cargo +1.97.0"
echo "dispatch: $(rustc +1.97.0 -V); CARGO_TARGET_DIR=$CARGO_TARGET_DIR"

# 3. The builds, and the tests.
mkdir -p "$out/bin"
$cargo build --locked --release --features trace --example occupancy
cp "$CARGO_TARGET_DIR/release/examples/occupancy" "$out/bin/occupancy"
$cargo build --locked --release --example attribution_decode
cp "$CARGO_TARGET_DIR/release/examples/attribution_decode" "$out/bin/attribution_decode"
(cd "$out/bin" && sha256sum occupancy attribution_decode > ../binaries.sha256)
occ=$out/bin/occupancy; dec=$out/bin/attribution_decode
$cargo test --locked --release --features trace --test trace_dispatch --test trace_phases > "$out/tests-trace.log" 2>&1
$cargo test --locked --release --features trace --example occupancy > "$out/tests-occupancy.log" 2>&1
echo "dispatch: the dispatch and phase marks' tests, and the instrument's, pass"

# 4. C6: the default-feature decoder is the base commit's; the run stops on any difference.
base=$(want '["source_base"]')
git worktree remove --force "$work/base" 2>/dev/null || true
rm -rf "$work/base"
git worktree add --detach "$work/base" "$base" > /dev/null
cp examples/attribution_decode.rs "$work/base/examples/"
(cd "$work/base" && CARGO_TARGET_DIR=$work/base-target $cargo build --locked --release --example attribution_decode)
python3 -I bench/attribution_same_code.py "$work/base-target/release/examples/attribution_decode" "$dec" --out "$out/same-code.json"
git worktree remove --force "$work/base"

# 5. Corpora (the walk trial's frozen files), and correctness at w = 1 and 4.
cdir=${DISPATCH_CORPORA:-$repo/target/walk-decoder/work/corpora}
python3 -I - "$frozen" "$cdir" "$out/manifest.tsv" <<'EOF' || die "corpora: missing or differing from the frozen hashes in $cdir (run the walk trial's preparation, or set DISPATCH_CORPORA)"
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
while IFS=$'\t' read -r name role path sha n; do
    for w in 1 4; do
        got=$("$dec" "$path" "$w" --stdout | sha256sum | cut -d' ' -f1)
        [[ $got == "$sha" ]] || die "$name at w = $w: decoded bytes differ from the frozen hash"
    done
done < "$out/manifest.tsv"
echo "dispatch: $(wc -l < "$out/manifest.tsv") corpora match their frozen hashes and decode to them at w = 1 and 4"
primary=$(awk -F'\t' '$2=="primary"{print $3}' "$out/manifest.tsv")

# 6. The allocation and the environment (numbers only).
python3 -I bench/attribution.py allocation --cores 4 > "$out/allocation.json"
alloc=$(python3 -I -c "import json; print(','.join(map(str, json.load(open('$out/allocation.json'))['alloc'])))")
[[ $(tr ',' '\n' <<< "$alloc" | wc -l) == 4 ]] || die "fewer than four physical cores allowed"
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
cpus = {c: dict(governor=rd(f"/sys/devices/system/cpu/cpu{c}/cpufreq/scaling_governor"),
                idle_driver=rd("/sys/devices/system/cpu/cpuidle/current_driver"),
                idle_states=sh("sh", "-c", f"cat /sys/devices/system/cpu/cpu{c}/cpuidle/state*/name 2>/dev/null | tr '\\n' ' '"))
        for c in a["alloc"]}
json.dump(dict(label=label, allocation=a, cpus=cpus, lscpu=sh("lscpu"), kernel=platform.uname()._asdict(),
    rustc=sh("rustc", "+1.97.0", "-vV"), head=sh("git", "rev-parse", "HEAD"), loadavg=rd("/proc/loadavg"),
    perf_event_paranoid=rd("/proc/sys/kernel/perf_event_paranoid"), sched_features=rd("/sys/kernel/debug/sched/features"),
    ps=sh("ps", "-eo", "pid,psr,pcpu", "--sort=-pcpu")), open(out, "w"), indent=1)
EOF
echo "dispatch: allocation CPUs $alloc"

# 7. perf, and the tracepoints: probed, the missing ones recorded (never replaced).
if [[ -z ${PERF:-} ]]; then
    if [[ $(id -u) == 0 ]]; then
        export PERF=perf CHOWN=""
    else
        echo "dispatch: the scheduler tracepoints need root: sudo, once"
        sudo -v
        ( while true; do sudo -n true; sleep 60; done ) & keep=$!
        trap 'kill $keep 2>/dev/null || true' EXIT
        export PERF="sudo -n perf" CHOWN="sudo -n chown $(id -u):$(id -g)"
    fi
fi
perf1=${PERF##* }
echo "dispatch: perf = $PERF ($($PERF --version 2>&1 | head -1))"
essential="sched:sched_switch sched:sched_wakeup sched:sched_wakeup_new sched:sched_waking syscalls:sys_enter_sched_yield"
optional="syscalls:sys_enter_futex syscalls:sys_exit_futex power:cpu_idle"
have=""
: > "$out/tracepoints.tsv"
for e in $essential $optional; do
    if $PERF record -q -k CLOCK_MONOTONIC -C "$alloc" -e "$e" -o "$work/probe.data" -- sleep 0.05 > /dev/null 2> "$work/probe.err"; then
        echo -e "$e\tsupported" >> "$out/tracepoints.tsv"; have="$have $e"
    else
        echo -e "$e\tunavailable\t$(tail -1 "$work/probe.err")" >> "$out/tracepoints.tsv"
        [[ " $essential " == *" $e "* ]] && die "essential tracepoint $e unavailable"
    fi
done
tierA=$have
tierB=$(tr ' ' '\n' <<< "$have" | grep -v futex | tr '\n' ' ')
echo "dispatch: tracepoints: $(grep -c supported "$out/tracepoints.tsv") supported, $(grep -c unavailable "$out/tracepoints.tsv") unavailable"

# 8. The capture.
caps=$out/capture
mkdir -p "$caps"
pairs=$out/pairs.tsv
echo -e "# w\trep\tmode\twall\tcpu\tout\trss_kb\ttier" > "$pairs"
kv() { tr ' ' '\n' <<< "$1" | awk -F= -v k="$2" '$1==k{print $2}'; }
one() { # w rep mode tier
    local w=$1 rep=$2 mode=$3 tier=$4 line evs="" e
    if [[ $mode == off ]]; then
        line=$(taskset -c "$alloc" "$occ" --child "$primary" "$w" $((2 * w)) 0 auto)
    else
        for e in $([[ $tier == A ]] && echo "$tierA" || echo "$tierB"); do evs="$evs -e $e"; done
        local m=$caps/w$w-r$(printf %02d "$rep")
        # shellcheck disable=SC2086
        line=$($PERF record -q -k CLOCK_MONOTONIC -C "$alloc" $evs -o "$m.data" -- \
            env OCCUPANCY_MARKS="$m.tsv" taskset -c "$alloc" "$occ" --child "$primary" "$w" $((2 * w)) 1 auto)
        [[ -n ${CHOWN:-} ]] && $CHOWN "$m.data" "$m.tsv" "$m.tsv.meta.json"
        "$perf1" script -f -i "$m.data" --ns -F comm,tid,cpu,time,event,trace 2> /dev/null > "$m.perf.txt"
        python3 -I bench/dispatch.py run --marks "$m.tsv" --perf "$m.perf.txt" --workers "$w" --cpus "$alloc" --out "$m.json" > /dev/null
    fi
    [[ $(kv "$line" err) == "-" && $(kv "$line" hash) != "" ]] || die "w=$w rep=$rep $mode: the decode failed: $line"
    echo -e "$w\t$rep\t$mode\t$(kv "$line" wall)\t$(kv "$line" cpu)\t$(kv "$line" out)\t$(kv "$line" rss_kb)\t$tier" >> "$pairs"
}
pair() { # w rep tier: the order alternates by repetition
    if (( $2 % 2 == 0 )); then one "$1" "$2" off "$3"; one "$1" "$2" instr "$3"; else one "$1" "$2" instr "$3"; one "$1" "$2" off "$3"; fi
}
cat /proc/stat > "$out/stat-before.txt"
tier=A
for ((r = 0; r < initial; r++)); do pair 4 "$r" A; done
# quality: exit 0 (every check holds), 3 (a check fails); anything else is the analysis failing
quality() { local rc=0; python3 -I bench/dispatch.py quality --runs "$caps" --pairs "$pairs" > "$1" || rc=$?
    (( rc == 0 || rc == 3 )) || die "the quality analysis failed (exit $rc)"; return "$rc"; }
quality "$out/quality-A.json" || true
if ! python3 -I -c "import json,sys; sys.exit(0 if json.load(open('$out/quality-A.json'))['perturbation']['ok'] else 1)"; then
    echo "dispatch: tier A perturbs by more than 3%: tier B (no futex events), from the start"
    mkdir -p "$out/tier-A" && mv "$caps" "$pairs" "$out/tier-A/" && mkdir -p "$caps"
    echo -e "# w\trep\tmode\twall\tcpu\tout\trss_kb\ttier" > "$pairs"
    tier=B
    for ((r = 0; r < initial; r++)); do pair 4 "$r" B; done
fi
for ((r = 0; r < w1reps; r++)); do pair 1 "$r" "$tier"; done
if quality "$out/quality.json"; then
    echo "dispatch: the quality checks pass on the first $initial: expanding to $total"
    for ((r = initial; r < total; r++)); do pair 4 "$r" "$tier"; done
elif [[ $dry == 1 ]]; then
    echo "dispatch: DRY RUN: a quality check fails on the first $initial; the expansion forced to exercise its path (a real run stops here)"
    for ((r = initial; r < total; r++)); do pair 4 "$r" "$tier"; done
else
    echo "dispatch: a quality check fails on the first $initial: not expanded (quality.json says which)"
fi
cat /proc/stat > "$out/stat-after.txt"

# 9. The summary, then the archive.
python3 -I bench/dispatch.py summary --runs "$caps" --pairs "$pairs" --out "$out/result.json"
[[ $dry == 1 ]] && echo "DRY RUN: the machinery checked at one repetition and one expansion; nothing here is interpreted" > "$out/DRY-RUN"
rm -f "$caps"/*.data
gzip -9 "$caps"/*.tsv "$caps"/*.perf.txt 2> /dev/null || true
[[ -d $out/tier-A ]] && { rm -f "$out"/tier-A/capture/*.data; gzip -9 "$out"/tier-A/capture/*.tsv "$out"/tier-A/capture/*.perf.txt 2> /dev/null || true; }
tar -czf "$out.tar.gz" -C "$results_dir" "$(basename "$out")"
echo "dispatch: results: $out"
echo "dispatch: archive: $out.tar.gz"
