# W1, the cold start (docs/journal/2026-11-10-cold-start.md, the rule registered before data;
# the spec is W1 of docs/journal/2026-11-10-next-experiments.md). #318's gap-block harness with
# five more arms: G gap, Z no gap, P taskset one CPU a core, W a 20 ms all-CPU spin ending at
# launch, B one spinner on the last CPU throughout, L the input x4 by 100 ms window. 30 runs an
# arm in a seeded order, the archived ab's own child (default plan, 10 threads), the Wikidata
# sample. About 3 minutes.
#   just submit bench-cold-start
# A DRY check (N runs an arm, optionally a CORPUS; not evidence):
#   just bench-cold-start 2 [CORPUS]

# W1: the cold start's arms (gap, no gap, pinned, start-up spin, busy CPU, input x4)
bench-cold-start DRY="" CORPUS="":
    #!/usr/bin/env bash
    set -euo pipefail
    B=$PWD/target/cold-start
    W=$B/$(date -u +%Y%m%dT%H%M%SZ)
    mkdir -p "$B"
    mkdir "$W" || { echo "bench-cold-start: $W already exists; not reusing an attempt's directory"; exit 2; }
    exec > >(tee -a "$W/console.log") 2>&1
    outcome="INCOMPLETE: stopped before the judge (see console.log)"
    finish() {
      [ -s "$W/outcome.txt" ] || echo "$outcome" > "$W/outcome.txt"
      echo "bench-cold-start: OUTCOME: $(head -1 "$W/outcome.txt")"
      bash bench/bundle.sh "$W"
    }
    trap finish EXIT
    fail() { outcome="REFUSED: $*"; echo "bench-cold-start: $*"; exit 2; }
    each=30; dry="{{DRY}}"; corpus="{{CORPUS}}"
    if [ -n "$dry" ]; then
      each=$dry; echo "bench-cold-start: DRY RUN: $each runs an arm; not evidence" | tee "$W/DRY-RUN.txt"
    elif [ -n "$corpus" ]; then
      fail "a CORPUS is for a DRY check only"
    fi
    [ -z "$(git status --porcelain --untracked-files=no)" ] || fail "the checkout has uncommitted changes"
    git rev-parse HEAD | tee "$W/commit.txt"
    sha() { sha256sum "$1" | cut -d' ' -f1; }
    # the harness that produced the two modes: the run-fed leader's archived ab, by digest
    ev=65ab4b95fd659e9d910736233d95479f6f3b452e; src=run-fed-leader/20261109T201013Z.tar.gz
    bundle_sha=7e36a5a75903e31be1c1722382ef0098a9505033a5831b5b3e92f25a811be188
    ab_sha=ce1eea5306487a0797b00645ba341bbd6712e6a34a2d93e6f5edc67cb841b1bb
    win_sha=cf5d74a6173d0cbdc91b5b366e1cdb6888755b7523f534124ba0ff6d1d884fd2
    git cat-file -e "$ev^{commit}" 2>/dev/null || timeout 120 git fetch -q origin evidence || fail "the evidence branch could not be fetched"
    S=$B/source-${bundle_sha:0:12}
    if [ ! -f "$S/bin/ab" ]; then
      rm -rf "$S.tmp"; mkdir -p "$S.tmp/bin"
      git show "$ev:$src" > "$S.tmp/bundle.tar.gz"
      [ "$(sha "$S.tmp/bundle.tar.gz")" = "$bundle_sha" ] || fail "the archived bundle's digest is not the registered one"
      top=$(basename "$src" .tar.gz)
      tar -xzf "$S.tmp/bundle.tar.gz" -C "$S.tmp" "$top/bin/ab" "$top/bin/windows"
      mv "$S.tmp/$top/bin/ab" "$S.tmp/$top/bin/windows" "$S.tmp/bin/"
      rm -rf "$S.tmp/$top" "$S.tmp/bundle.tar.gz"; mv "$S.tmp" "$S"
    fi
    mkdir "$W/bin"
    cp "$S/bin/ab" "$S/bin/windows" "$W/bin/"
    [ "$(sha "$W/bin/ab")" = "$ab_sha" ] || fail "the archived ab is not the registered executable"
    [ "$(sha "$W/bin/windows")" = "$win_sha" ] || fail "the archived windows is not the registered executable"
    cc -O2 -Wall -Wextra -pthread -o "$W/bin/cold_start_spin" bench/cold_start_spin.c || fail "the spinner did not compile"
    if [ -z "$corpus" ]; then
      dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
      if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
        timeout 120 git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
          && git archive origin/bench-data "$dir" | tar -x || true
      fi
      python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null || fail "the corpus's windows could not be fetched"
      python3 bench/wikidata.py build bench/wikidata.json "$dir" "$W/bin/windows" "$B/wikidata.bz2" || fail "the corpus could not be built"
      corpus=$B/wikidata.bz2
      want=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["corpus_sha256"])')
      [ "$(sha "$corpus")" = "$want" ] || fail "the corpus built is not bench/wikidata.json's $want"
    fi
    corpus=$(realpath "$corpus")
    X=$B/$(basename "$corpus" .bz2)-x4.bz2
    cat "$corpus" "$corpus" "$corpus" "$corpus" > "$X"
    { sha256sum "$W/bin/ab" "$W/bin/cold_start_spin" "$corpus" "$X"; cc --version | head -1; uname -a
      echo "layout (P CPUs, B CPU): $(python3 bench/cold_start.py layout)"; } | tee "$W/inputs.txt"
    set +e
    timeout 900 python3 bench/cold_start.py run "$W/bin/ab" "$W/bin/cold_start_spin" "$corpus" "$X" "$W/runs.jsonl" --each "$each"
    echo "run: exit $?" | tee "$W/exits.txt"
    set -e
    python3 bench/cold_start.py judge "$W" || outcome="REPORT ERROR: the judge failed"
