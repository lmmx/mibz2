#!/usr/bin/env bash
set -euo pipefail
base=$(realpath -m "${1:?new trace work directory}")
packed=$(realpath "${2:?verified singletiny module}")
projection=${3:?same scorer and just projection selected for the official pair}
root=$(git rev-parse --show-toplevel)
d=$root/research/astra-e/huffman-phrases/integrated
mkdir -p "$base"
export CARGO_BUILD_JOBS=1 RUSTUP_TOOLCHAIN=1.99.1 PYTHONDONTWRITEBYTECODE=1
worker=research/astra-e/front-fusion/cap-batch/worker/candidate.rs
for arm in stock phrase; do
    tree=$base/$arm
    git -C "$root" worktree add -q --detach "$tree" d0a42980e896211906234c7679b38760ae44a46e
    cd "$tree"
    git checkout "$projection" -- bench/moonshot .just Justfile
    bash research/astra-e/prepare.sh "$worker"
    if [[ $arm == phrase ]];then python3 "$d/prepare.py" "$tree" "$packed";fi
    python3 - "$d/trace-out.inc.rs" <<'PY'
from pathlib import Path
import sys
path=Path("bench/moonshot/src/main.rs")
s=path.read_text()
line="            let (wall, cpu, bytes) = by_arm!(arm.as_str(), timed(data, workers), unknown());"
assert s.count(line)==1
s=s.replace(line,'''            let trace=std::env::var_os("ASTRA_SCAN_MARKS").unwrap();
            mibz2::trace::set_level(2);
'''+line+'''
            write_trace(std::path::Path::new(&trace),workers);''')
path.write_text(s+"\n"+Path(sys.argv[1]).read_text())
PY
    export CARGO_TARGET_DIR=$base/$arm-target MOONSHOT_CANDIDATE=$tree/$worker
    cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml --features mibz2/trace
done
