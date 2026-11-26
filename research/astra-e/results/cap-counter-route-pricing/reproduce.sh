#!/usr/bin/env bash
# Historical reproduction with the original Rust 1.99.1 toolchain and Valgrind 3.24.0 on PATH.
set -euo pipefail
repo=$(git rev-parse --show-toplevel)
study="$repo/research/astra-e/results/cap-counter-route-pricing"
scratch=$(realpath -m "${1:?new scratch directory}")
corpus=$(realpath "${2:?original compressed Wikidata corpus}")
# MODE=build recompiles the counter arms. Archived mode checks all four original hashes.
mode=${MODE:-archive}
case "$mode" in
    archive) flags=(--archived-new) ;;
    build) flags=() ;;
    *) echo "Set MODE to archive or build" >&2; exit 2 ;;
esac
if [[ "${CHECK_ONLY:-0}" == 1 ]]; then flags+=(--check-only); fi
projection=3df195b10e9fb6f86f0c06ad9fad1feb992406d5
mkdir "$scratch"
git worktree add --detach "$scratch/scorer" d05577e22526c5880fc6f683f4189b5b48377dc5
git -C "$scratch/scorer" checkout "$projection" -- bench/moonshot .just Justfile
nice -n 15 taskset -c 4 bash "$study/build.sh" "$scratch/build" "$projection" "$mode"
# The debugger embeds absolute build paths. Reuse the original hashed tool when available.
vlib=${CHAINGRIND_DIR:-/workspace/work/astra-scalar-route-build/chaingrind}
if [[ ! -f "$vlib/chaingrind-amd64-linux" ]]; then
    vlib="$scratch/chaingrind"
    export REPLAY_SOURCE="$scratch/scorer" REPLAY_CWD="$PWD" REPLAY_CC="${CC:-gcc}"
    cat > "$scratch/debug-path-cc" <<'SH'
#!/usr/bin/env bash
exec "$REPLAY_CC" "-ffile-prefix-map=$REPLAY_CWD=/workspace/work/astra-e" \
    "-ffile-prefix-map=$REPLAY_SOURCE=/workspace/work/astra-scalar-route-scorer" "$@"
SH
    chmod +x "$scratch/debug-path-cc"
    CC="$scratch/debug-path-cc" nice -n 15 taskset -c 4 bash "$scratch/scorer/bench/moonshot/chaingrind/build.sh" "$vlib"
fi
nice -n 15 taskset -c 4 python3 -B "$study/run.py" "$scratch/scorer" "$scratch/build" "$vlib" "$corpus" "$scratch/results" --historical-master "$projection" "${flags[@]}"
if [[ "${CHECK_ONLY:-0}" == 1 ]]; then exit 0; fi
python3 -B "$study/report.py" "$scratch/scorer" "$scratch/results"
