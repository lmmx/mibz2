#!/usr/bin/env bash
set -euo pipefail
# each attempt in a directory of its own, never overwritten
B=$PWD/target/three-team-policy
W=$B/$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$B"
mkdir "$W" || { echo "bench-three-team-policy: $W already exists; not reusing an attempt's directory"; exit 1; }
exec > >(tee -a "$W/console.log") 2>&1
# every attempt ends with one stated outcome (outcome.txt) and is packed, however it ends:
# a refusal, a failed build, a broken report or a full run
outcome="INCOMPLETE: stopped before the run finished (see console.log)"
finish() {
  status=$?
  echo "$outcome" > "$W/outcome.txt"
  echo "bench-three-team-policy: OUTCOME: $outcome"
  bash bench/bundle.sh "$W"
  # (VALID PASS and VALID FAIL exit 0, INVALID 1, anything that kept the run from a
  # verdict, a refusal or a report error among them, 2 or its own failing status)
  case "$outcome" in
    "VALID "*) exit 0 ;;
    INVALID*) exit 1 ;;
    *) [ "$status" -ne 0 ] && exit "$status"; exit 2 ;;
  esac
}
trap finish EXIT
fail() { outcome="REFUSED: $*"; echo "bench-three-team-policy: $*"; exit 1; }
# the commits the registration froze (bench/three-team-policy.frozen): the candidate, whose
# build and workload inputs HEAD must match exactly, and the base it is measured against
frozen=bench/three-team-policy.frozen
cand=$(awk '$1 == "candidate" {print $2}' "$frozen")
base=$(awk '$1 == "base" {print $2}' "$frozen")
[[ $cand =~ ^[0-9a-f]{40}$ && $base =~ ^[0-9a-f]{40}$ ]] || fail "$frozen names no frozen candidate and base yet; nothing to run"
# what the builds and the workload are made from: sources, Cargo's files and configuration,
# the harness, the corpus's builder and manifest, the judge and this recipe
measured=(src Cargo.toml Cargo.lock .cargo examples bench/policy_ab.py bench/wikidata.json bench/wikidata.py .just/bench.just)
for v in RUSTFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_RUSTFLAGS CARGO_TARGET_DIR CARGO_BUILD_TARGET RUSTC RUSTC_WRAPPER CARGO_PROFILE_RELEASE_OPT_LEVEL; do
  [ -z "${!v:-}" ] || fail "$v is set; unset it, so both builds are made the same, plain way"
done
[ -z "$(git status --porcelain --untracked-files=no)" ] || fail "the checkout has uncommitted changes"
[ -z "$(git status --porcelain --untracked-files=all -- "${measured[@]}")" ] || fail "untracked files among the build and workload inputs"
git cat-file -e "$cand^{commit}" && git cat-file -e "$base^{commit}" || fail "fetch the branch: the frozen commits are missing"
git diff --quiet "$cand" HEAD -- "${measured[@]}" || fail "HEAD's build or workload inputs differ from the frozen candidate $cand"
head=$(git rev-parse HEAD)
echo "head $head (build and workload inputs as candidate $cand)"; echo "base $base"
printf 'head %s\ncandidate %s\nbase %s\n' "$head" "$cand" "$base" > "$W/commits.txt"
[ -d "$B/build-$base" ] || git worktree add -q --detach "$B/build-$base" "$base"
[ "$(git -C "$B/build-$base" rev-parse HEAD)" = "$base" ] || fail "$B/build-$base is not at the base $base"
[ -z "$(git -C "$B/build-$base" status --porcelain --untracked-files=all)" ] || fail "$B/build-$base has changes or untracked files"
# (each executable as Cargo reports building it, from its own checkout, then copied into
# this attempt's directory: the copies are what run and what the digests describe, so
# neither build can overwrite the other's, wherever Cargo's target directory is)
built() {  # built DIR EXAMPLE...: build in DIR, print each example's executable, in order
  local dir=$1; shift
  local args=(); for e in "$@"; do args+=(--example "$e"); done
  (cd "$dir" && cargo build -q --release --message-format=json-render-diagnostics "${args[@]}") \
    | python3 -c 'import json,sys
want=sys.argv[1:]; got={}
for line in sys.stdin:
    m=json.loads(line)
    if m.get("reason")=="compiler-artifact" and m.get("executable") and m["target"]["name"] in want and "example" in m["target"]["kind"]:
        got[m["target"]["name"]]=m["executable"]
missing=[w for w in want if w not in got]
if missing: sys.exit("no executable reported for: "+" ".join(missing))
print("\n".join(got[w] for w in want))' "$@"
}
mkdir "$W/bin"
mapfile -t base_built < <(built "$B/build-$base" ab)
mapfile -t head_built < <(built "$PWD" ab windows)
[ "${#base_built[@]}" = 1 ] && [ "${#head_built[@]}" = 2 ] || fail "Cargo reported no executables for the builds"
cp "${base_built[0]}" "$W/bin/ab-base"
cp "${head_built[0]}" "$W/bin/ab-head"
cp "${head_built[1]}" "$W/bin/windows"
AB=$W/bin/ab-head
BASEAB=$W/bin/ab-base
WINDOWS=$W/bin/windows
dir=$(python3 -c 'import json; print(json.load(open("bench/wikidata.json"))["windows_dir"])')
if [ "$(ls "$dir"/*.bin 2>/dev/null | wc -l)" = 0 ]; then
  git fetch -q origin bench-data:refs/remotes/origin/bench-data 2>/dev/null \
    && git archive origin/bench-data "$dir" | tar -x || true
fi
python3 bench/wikidata.py fetch bench/wikidata.json "$dir" >/dev/null
python3 bench/wikidata.py build bench/wikidata.json "$dir" "$WINDOWS" "$B/wikidata.bz2"
ln -sf "$B/wikidata.bz2" "$W/wikidata.bz2"
# what ran: both executables, the corpus and the toolchain, by digest
{ sha256sum "$AB" "$BASEAB" "$WINDOWS" "$B/wikidata.bz2"; rustc -vV; cargo -V; } > "$W/inputs.txt"
cat "$W/inputs.txt"
"$AB" env > "$W/env.json"
lscpu > "$W/lscpu.txt"
"$AB" run "$W/runs.jsonl" --arms "mibz2,mibz2@$BASEAB" \
  --threads "8,9,10,11,12,16,20" --rounds 20 "$W/wikidata.bz2"
# the verdict: VALID PASS, VALID FAIL or INVALID, from the report's own fields; a report
# that couldn't be written is said to be so, and points to nothing
set +e
python3 bench/policy_ab.py report "$W/runs.jsonl" --out "$W/policy-ab"
rc=$?
set -e
if [ "$rc" -le 1 ] && [ -s "$W/policy-ab.json" ] && [ -s "$W/policy-ab.md" ]; then
  outcome=$(python3 -c 'import json,sys; v=json.load(open(sys.argv[1])); print(("VALID PASS" if v["pass"] else "VALID FAIL") if v["valid"] else "INVALID: " + "; ".join(v["problems"]))' "$W/policy-ab.json")
else
  outcome="REPORT ERROR: policy_ab.py exited $rc without writing its report"
fi
