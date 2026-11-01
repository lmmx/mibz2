#!/usr/bin/env bash
# P87's four-core diagnostic, one command (docs/journal/2026-11-01-p87-four-core-protocol.md):
#   bash bench/p87_run.sh HOST_LABEL       (or: just bench-p87 HOST_LABEL)
# Steps: the pinned clean checkout and frozen sources; rustc 1.97.0; three builds (the
# trial's harness with `p87`; the phase instrument, examples/occupancy, with `trace,p87`;
# the default-feature examples/attribution_decode); the tests (the arms' correctness, the
# harness's, the analysis's on synthetic trials); E3 (the default-feature decoder is the
# base commit's) and the machine-code audit E1-E2 of the exact binary timed; the walk
# trial's frozen corpora, every hash checked, each decoded by both arms and the shipped
# path at w = 1 and 4 and sequentially; the allocation (four physical cores, their
# siblings idle); a numbers-only environment; the harness's self-test with synthetic
# figures; the counters probed (perf; what is unavailable is recorded, never replaced);
# then S1, the uninstrumented paired trial (20 rounds and a warm-up, arms A, B, A2, at
# w = 1 and 4); S2, the phase attribution (trace level 2, paired with level 0); S3, DRAM
# traffic and core counters over whole decodes; the analysis; the archive.
# Env: P87_DRY_RUN=1 (everything but S1's timed trial, which is replaced by the synthetic
# self-test; S2 and S3 at 2 repetitions; into LABEL.dry-run; nothing interpreted);
# P87_CORPORA (default the walk trial's work directory); P87_RESULTS_DIR (default
# target/p87-results); P87_WORK (default target/p87-work); PERF (default `perf` as root,
# else `sudo -n perf`, asked for once); CARGO_TARGET_DIR (default target/).
set -euo pipefail
label=${1:?usage: p87_run.sh HOST_LABEL}
[[ $label =~ ^[A-Za-z0-9._-]+$ ]] || { echo "label: letters, digits, . _ - only" >&2; exit 2; }
repo=$(git rev-parse --show-toplevel)
cd "$repo"
dir=docs/journal/2026-11-01-p87-four-core
frozen=$dir/frozen.json
work=${P87_WORK:-$repo/target/p87-work}
results_dir=${P87_RESULTS_DIR:-$repo/target/p87-results}
dry=${P87_DRY_RUN:-}
out=$results_dir/$label
[[ $dry == 1 ]] && out=$results_dir/$label.dry-run
die() { echo "p87: $*" >&2; exit 1; }
want() { python3 -I -c "import json,sys; print(json.load(open('$frozen'))$1)"; }

# 1. The pinned commit, a clean tree, a fresh output directory, the frozen sources.
[[ $(uname -sm) == "Linux x86_64" ]] || die "Linux x86_64 only"
[[ -z $(git status --porcelain --untracked-files=no) ]] || die "working tree not clean"
pin=$(cat bench/p87-pin)
git cat-file -e "$pin^{commit}" 2>/dev/null || die "pinned commit $pin not present (fetch the branch)"
git diff --quiet "$pin" HEAD -- . ":(exclude)bench/p87-pin" ":(exclude)$dir" || die "HEAD differs from pinned $pin"
if [[ $dry == 1 ]]; then rm -rf "$out" "$out.tar.gz"; fi
[[ ! -e $out ]] || die "$out exists: one run per label, no reruns"
mkdir -p "$out" "$work"
exec > >(tee "$out/run.log") 2>&1
echo "p87: label=$label head=$(git rev-parse HEAD) pin=$pin out=$out dry_run=${dry:-0}"
python3 -I - "$frozen" <<'EOF'
import hashlib, json, sys
f = json.load(open(sys.argv[1]))
for p, h in f["sources_sha256"].items():
    assert hashlib.sha256(open(p, "rb").read()).hexdigest() == h, f"source changed: {p}"
print(f"p87: {len(f['sources_sha256'])} sources match the frozen hashes")
EOF
reps=$(want '["schedule"]["S2"]["reps"]')
[[ $dry == 1 ]] && reps=2

# 2. rustc 1.97.0, empty RUSTFLAGS.
unset RUSTFLAGS CARGO_BUILD_RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$repo/target}
rustup toolchain list | grep -q '^1\.97\.0' || rustup toolchain install 1.97.0 --profile minimal
export RUSTUP_TOOLCHAIN=1.97.0
cargo="cargo +1.97.0"
echo "p87: $(rustc +1.97.0 -V); CARGO_TARGET_DIR=$CARGO_TARGET_DIR"

# 3. The builds and the tests.
mkdir -p "$out/bin"
$cargo build --locked --release --features p87 --example p87_decoder
cp "$CARGO_TARGET_DIR/release/examples/p87_decoder" "$out/bin/p87_decoder"
$cargo build --locked --release --features trace,p87 --example occupancy
cp "$CARGO_TARGET_DIR/release/examples/occupancy" "$out/bin/occupancy"
$cargo build --locked --release --example attribution_decode
cp "$CARGO_TARGET_DIR/release/examples/attribution_decode" "$out/bin/attribution_decode"
(cd "$out/bin" && sha256sum p87_decoder occupancy attribution_decode > ../binaries.sha256)
bin=$out/bin/p87_decoder; occ=$out/bin/occupancy; dec=$out/bin/attribution_decode
$cargo test --locked --release --features p87 --test p87 --test p87_parallel > "$out/tests-arms.log" 2>&1 || die "the arms' tests fail (tests-arms.log)"
$cargo test --locked --release --features p87 --example p87_decoder > "$out/tests-harness.log" 2>&1 || die "the harness's tests fail (tests-harness.log)"
$cargo test --locked --release --features trace,p87 --test trace_phases --test trace_dispatch > "$out/tests-trace.log" 2>&1 || die "the trace's tests fail (tests-trace.log)"
python3 -I bench/test_p87.py > "$out/tests-analysis.log" 2>&1 || die "the analysis's tests fail (tests-analysis.log)"
echo "p87: the arms', the harness's, the trace's and the analysis's tests pass"

# 4. E3, the default-feature decoder the base commit's; E1-E2, the exact binary's machine
#    code (one compiled path for both arms; the shipped kernels). The run stops on either.
base=$(want '["source_base"]')
git worktree remove --force "$work/base" 2>/dev/null || true
rm -rf "$work/base"
git worktree add --detach "$work/base" "$base" > /dev/null
cp examples/attribution_decode.rs "$work/base/examples/"
(cd "$work/base" && CARGO_TARGET_DIR=$work/base-target $cargo build --locked --release --example attribution_decode)
python3 -I bench/attribution_same_code.py "$work/base-target/release/examples/attribution_decode" "$dec" --out "$out/same-code.json"
git worktree remove --force "$work/base"
rm -rf "$work/base-target"
python3 -I bench/p87_code_audit.py "$bin" "$dec" --same-code "$out/same-code.json" --out "$out/code-audit.json"
# the phase instrument's, recorded (its marks are compiled in; the gate is the timed binary's)
python3 -I bench/p87_code_audit.py "$occ" "$dec" --same-code "$out/same-code.json" --out "$out/code-audit-instrument.json" || true
objdump -d -C -Mintel --no-show-raw-insn "$bin" | gzip -9 > "$out/p87_decoder.asm.gz"

# 5. Corpora (the walk trial's frozen files), and correctness: both arms and the shipped
#    path at w = 1 and 4 through the trial's decode, and sequentially.
cdir=${P87_CORPORA:-$repo/target/walk-decoder/work/corpora}
python3 -I bench/p87.py manifest --frozen "$frozen" --dir "$cdir" --out "$out/manifest.tsv" \
    || die "corpora: missing or differing from the frozen hashes in $cdir (run the walk trial's preparation, or set P87_CORPORA)"
primary=$(awk -F'\t' '$2=="primary"{print $3}' "$out/manifest.tsv")
pbytes=$(awk -F'\t' '$2=="primary"{print $6}' "$out/manifest.tsv")

# 6. The allocation and the environment (numbers only).
python3 -I bench/attribution.py allocation --cores 4 > "$out/allocation.json"
alloc=$(python3 -I -c "import json; print(','.join(map(str, json.load(open('$out/allocation.json'))['alloc'])))")
[[ $(tr ',' '\n' <<< "$alloc" | wc -l) == 4 ]] || die "fewer than four physical cores allowed"
taskset -c "$alloc" "$bin" --check "$out/manifest.tsv" 1,4 > "$out/check.tsv"
echo "p87: $(wc -l < "$out/check.tsv") corpora decode to their frozen hashes in arms A and B and the shipped path, at w = 1 and 4 and sequentially"
# the phase instrument's own output hash (not SHA-256) of the primary, through the shipped
# path: every S2 and S3 run, in either arm, must give it and the frozen length
ref=$(env -u MIBZ2_P87_ARM taskset -c "$alloc" "$occ" --child "$primary" 1 2 0 auto)
refhash=$(tr ' ' '\n' <<< "$ref" | awk -F= '$1=="hash"{print $2}')
[[ $(tr ' ' '\n' <<< "$ref" | awk -F= '$1=="out"{print $2}') == "$pbytes" && -n $refhash ]] || die "the phase instrument's reference decode: $ref"
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
                cur_khz=rd(f"/sys/devices/system/cpu/cpu{c}/cpufreq/scaling_cur_freq"),
                max_khz=rd(f"/sys/devices/system/cpu/cpu{c}/cpufreq/scaling_max_freq"),
                siblings=rd(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list"))
        for c in a["alloc"]}
json.dump(dict(label=label, allocation=a, cpus=cpus, lscpu=sh("lscpu"), kernel=platform.uname()._asdict(),
    rustc=sh("rustc", "+1.97.0", "-vV"), head=sh("git", "rev-parse", "HEAD"), loadavg=rd("/proc/loadavg"),
    intel_no_turbo=rd("/sys/devices/system/cpu/intel_pstate/no_turbo"), smt_control=rd("/sys/devices/system/cpu/smt/control"),
    thp=rd("/sys/kernel/mm/transparent_hugepage/enabled"), perf_event_paranoid=rd("/proc/sys/kernel/perf_event_paranoid"),
    ps=sh("ps", "-eo", "pid,psr,pcpu", "--sort=-pcpu")), open(out, "w"), indent=1)
EOF
echo "p87: allocation CPUs $alloc"

# 7. The harness's self-test: S1's whole path (children, wait4, affinity, rows, analysis) on
#    the smallest corpus's file under every corpus's name, every figure synthetic.
st=$out/selftest
mkdir -p "$st"
python3 -I - "$out/manifest.tsv" "$st/manifest.tsv" <<'EOF'
import sys
rows = [l.split("\t") for l in open(sys.argv[1]).read().splitlines()]
small = min(rows, key=lambda r: int(r[5]))
with open(sys.argv[2], "w") as f:
    for r in rows:
        f.write("\t".join([r[0], r[1], *small[2:]]) + "\n")
EOF
P87_SYNTHETIC_TIMES=1 taskset -c "$alloc" "$bin" --trial "$st/manifest.tsv" 1,4 "$alloc" > "$st/raw-synthetic.tsv"
python3 -I bench/p87.py summary --manifest "$st/manifest.tsv" --raw "$st/raw-synthetic.tsv" \
    --allocation "$out/allocation.json" --out "$st/summary-synthetic.json" > /dev/null
echo "SYNTHETIC: every time and CPU figure here is a fixed value, not a measurement" > "$st/README"
echo "p87: harness self-test (synthetic figures) passed"

# 8. perf, for S3: the counters probed, what is unavailable recorded (never replaced).
if [[ -z ${PERF:-} ]]; then
    if [[ $(id -u) == 0 ]]; then
        export PERF=perf CHOWN=""
    else
        echo "p87: the memory controller's counters need root: sudo, once"
        sudo -v
        ( while true; do sudo -n true; sleep 60; done ) & keep=$!
        trap 'kill $keep 2>/dev/null || true' EXIT
        export PERF="sudo -n perf" CHOWN="sudo -n chown $(id -u):$(id -g)"
    fi
fi
echo "p87: perf = $PERF ($($PERF --version 2>&1 | head -1))"
imc="uncore_imc/data_reads/,uncore_imc/data_writes/"
core="{cycles,ref-cycles,instructions}"
s3=$out/counters
mkdir -p "$s3"
python3 -I - "$out/counter-probe.json" "$s3" "$alloc" "$occ" "$primary" <<'EOF'
import json, os, shlex, subprocess, sys
out, d, alloc, occ, primary = sys.argv[1:]
sys.path.insert(0, "bench")
import p87
perf = shlex.split(os.environ["PERF"])
probe = dict(unavailable={}, available=[])
for name, spec, events, wide in (("imc", "uncore_imc/data_reads/,uncore_imc/data_writes/", p87.IMC, True),
                                  ("core", "{cycles,ref-cycles,instructions}", p87.CORE, False)):
    f = f"{d}/probe-{name}.csv"
    cmd = perf + ["stat", "-x,", "-o", f, "-e", spec] + (["-a"] if wide else []) + ["--", "env", "MIBZ2_P87_ARM=A", "taskset", "-c", alloc, occ, "--child", primary, "1", "2", "0", "auto"]
    p = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    vals = p87.parse_stat(f) if os.path.exists(f) else {}
    if p.returncode == 0 and p87.counted(vals, events):
        probe["available"].append(name)
    else:
        probe["unavailable"][name] = dict(rc=p.returncode, values={e: vals.get(e) for e in events}, stderr=p.stderr[-300:])
json.dump(probe, open(out, "w"), indent=1)
print(f"p87: counters available: {probe['available'] or 'none'}; unavailable: {sorted(probe['unavailable']) or 'none'}")
EOF
[[ -n ${CHOWN:-} ]] && $CHOWN "$s3"/probe-*.csv
has() { python3 -I -c "import json,sys; sys.exit(0 if '$1' in json.load(open('$out/counter-probe.json'))['available'] else 1)"; }

snap() {
    cat /proc/stat > "$out/stat-$1.txt"
    for p in cpu memory io; do cat /proc/pressure/$p > "$out/pressure-$p-$1.txt" 2>/dev/null || echo unavailable > "$out/pressure-$p-$1.txt"; done
}

# 9. S1: the paired trial, the whole process tree in the allocation, nothing else running.
if [[ $dry == 1 ]]; then
    echo "p87: DRY RUN: S1's timed trial not run (its path is the self-test's, above)"
else
    snap before
    start=$(date +%s.%N)
    P87_APPROVED_RUNNER="$label" taskset -c "$alloc" "$bin" --trial "$out/manifest.tsv" 1,4 "$alloc" \
        > "$out/raw.tsv" 2> "$out/stderr.txt"
    end=$(date +%s.%N)
    snap after
    echo "{\"wall_s\": $(python3 -I -c "print($end - $start)"), \"alloc\": \"$alloc\"}" > "$out/timing.json"
    echo "p87: S1 done ($(grep -c '^ROW' "$out/raw.tsv") rows)"
fi

# 10. S2: the phases. Per repetition, w = 1 then 4; arms AB on even repetitions, BA on odd;
#     modes off-full on even, full-off on odd; the placement the repetition's parity.
kv() { tr ' ' '\n' <<< "$1" | awk -F= -v k="$2" '$1==k{print $2}'; }
s2=$out/phases
mkdir -p "$s2"
pairs=$out/phase-pairs.tsv
echo -e "# w\trep\tarm\tmode\twall\tcpu\tout\trss_kb\tplacement\tmarks" > "$pairs"
one() { # w rep arm mode
    local w=$1 rep=$2 arm=$3 mode=$4 line marks=- level=0 pl=$(( $2 % 2 ))
    if [[ $mode == full ]]; then
        level=2; marks=w$w-$arm-r$(printf %02d "$rep").tsv
        line=$(MIBZ2_P87_ARM=$arm MIBZ2_P87_PLACEMENT=$pl OCCUPANCY_MARKS="$s2/$marks" taskset -c "$alloc" "$occ" --child "$primary" "$w" $((2 * w)) $level auto)
    else
        line=$(MIBZ2_P87_ARM=$arm MIBZ2_P87_PLACEMENT=$pl taskset -c "$alloc" "$occ" --child "$primary" "$w" $((2 * w)) $level auto)
    fi
    [[ $(kv "$line" err) == "-" && $(kv "$line" hash) == "$refhash" && $(kv "$line" out) == "$pbytes" ]] || die "S2 w=$w rep=$rep $arm $mode: the decode failed or differs: $line"
    echo -e "$w\t$rep\t$arm\t$mode\t$(kv "$line" wall)\t$(kv "$line" cpu)\t$(kv "$line" out)\t$(kv "$line" rss_kb)\t$pl\t$marks" >> "$pairs"
}
for ((r = 0; r < reps; r++)); do
    for w in 1 4; do
        arms="A B"; (( r % 2 )) && arms="B A"
        for arm in $arms; do
            if (( r % 2 == 0 )); then one "$w" "$r" "$arm" off; one "$w" "$r" "$arm" full; else one "$w" "$r" "$arm" full; one "$w" "$r" "$arm" off; fi
        done
    done
done
echo "p87: S2 done ($(grep -vc '^#' "$pairs") runs)"

# 11. S3: the counters over whole decodes of the phase instrument at level 0 (no marks).
#     The memory controller's, machine-wide, with an idle reading as long just before; the
#     core counters, the process's.
runs=$out/counter-runs.jsonl
: > "$runs"
idle_s() { python3 -I -c "
import statistics
r = [l.split('\t') for l in open('$pairs').read().splitlines() if l and not l.startswith('#')]
print(f\"{statistics.median(float(x[4]) for x in r if x[0] == '$1' and x[3] == 'off'):.3f}\")"; }
for ((r = 0; r < reps; r++)); do
    for w in 1 4; do
        arms="A B"; (( r % 2 )) && arms="B A"
        for arm in $arms; do
            pl=$(( r % 2 )); tag=w$w-$arm-r$(printf %02d "$r")
            if has imc; then
                len=$(idle_s "$w")
                rc0=0; $PERF stat -x, -o "$s3/$tag-imc-idle.csv" -a -e "$imc" -- sleep "$len" 2> /dev/null || rc0=$?
                t0=$(date +%s.%N); rc=0
                # (the arm through `env`: sudo does not pass the caller's environment on)
                line=$($PERF stat -x, -o "$s3/$tag-imc.csv" -a -e "$imc" -- \
                    env MIBZ2_P87_ARM="$arm" MIBZ2_P87_PLACEMENT="$pl" taskset -c "$alloc" "$occ" --child "$primary" "$w" $((2 * w)) 0 auto) || rc=$?
                t1=$(date +%s.%N)
                [[ $(kv "$line" hash) == "$refhash" && $(kv "$line" out) == "$pbytes" ]] || die "S3 $tag imc: the decode differs: $line"
                echo "{\"w\": $w, \"rep\": $r, \"arm\": \"$arm\", \"pass\": \"imc\", \"csv\": \"$tag-imc.csv\", \"idle_csv\": \"$tag-imc-idle.csv\", \"rc\": $((rc + rc0)), \"wall_s\": $(python3 -I -c "print($t1 - $t0)"), \"idle_wall_s\": $len, \"out\": $(kv "$line" out)}" >> "$runs"
            fi
            if has core; then
                rc=0
                line=$($PERF stat -x, -o "$s3/$tag-core.csv" -e "$core" -- \
                    env MIBZ2_P87_ARM="$arm" MIBZ2_P87_PLACEMENT="$pl" taskset -c "$alloc" "$occ" --child "$primary" "$w" $((2 * w)) 0 auto) || rc=$?
                [[ $(kv "$line" hash) == "$refhash" && $(kv "$line" out) == "$pbytes" ]] || die "S3 $tag core: the decode differs: $line"
                echo "{\"w\": $w, \"rep\": $r, \"arm\": \"$arm\", \"pass\": \"core\", \"csv\": \"$tag-core.csv\", \"rc\": $rc, \"wall_s\": $(kv "$line" wall), \"out\": $(kv "$line" out)}" >> "$runs"
            fi
        done
    done
done
[[ -n ${CHOWN:-} ]] && $CHOWN "$s3"/*.csv 2> /dev/null || true
echo "p87: S3 done ($(wc -l < "$runs") passes)"

# 12. The analysis, then the archive.
python3 -I bench/p87.py phases --pairs "$pairs" --runs "$s2" --out "$out/phases.json"
python3 -I bench/p87.py counters --runs "$runs" --dir "$s3" --probe "$out/counter-probe.json" --out "$out/counters.json"
if [[ $dry == 1 ]]; then
    echo "DRY RUN: S1 not run (the self-test's synthetic trial in its place); S2 and S3 at $reps repetitions; nothing here is interpreted" > "$out/DRY-RUN"
    python3 -I bench/p87.py verdict --trial "$st/summary-synthetic.json" --phases "$out/phases.json" --counters "$out/counters.json" --out "$out/verdict-synthetic.json" > /dev/null
    echo "p87: DRY RUN: the verdict's path checked on the synthetic trial (verdict-synthetic.json, not a result)"
else
    python3 -I bench/p87.py summary --manifest "$out/manifest.tsv" --raw "$out/raw.tsv" --allocation "$out/allocation.json" \
        --stat-before "$out/stat-before.txt" --stat-after "$out/stat-after.txt" --out "$out/trial.json" > /dev/null
    python3 -I bench/p87.py verdict --trial "$out/trial.json" --phases "$out/phases.json" --counters "$out/counters.json" --out "$out/verdict.json"
    gzip -k "$out/raw.tsv"
fi
gzip -9 "$s2"/*.tsv 2> /dev/null || true
tar -czf "$out.tar.gz" -C "$results_dir" "$(basename "$out")"
echo "p87: results: $out"
echo "p87: archive: $out.tar.gz"
