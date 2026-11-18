#!/usr/bin/env bash
# Invoked in a disposable checkout of the PR head by astra-e.yml.
set -euo pipefail
: "${BASE_SHA:?frozen PR base required}" "${HEAD_SHA:?frozen PR head required}" "${RUNNER_TEMP:?}"
root=$(git rev-parse --show-toplevel)
cd "$root"
out=$root/astra-e-result
mkdir "$out"
exec > >(tee "$out/console.log") 2>&1
test "$(git rev-parse HEAD)" = "$HEAD_SHA"
base=$RUNNER_TEMP/astra-e-base
git worktree add --detach "$base" "$BASE_SHA"
candidate=$(cat research/astra-e/ci/candidate)
git ls-files --error-unmatch -- "$candidate"
bash research/astra-e/prepare.sh "$candidate"
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
fresh=$(git rev-parse origin/master)
# HEAD retains the last scorer that built the candidate in the research tree.
last=$(git log -1 --format=%H "$HEAD_SHA" -- bench/moonshot)
export RUSTUP_TOOLCHAIN
RUSTUP_TOOLCHAIN=$(sed -nE 's/^channel = "([^"]+)".*/\1/p' "$base/rust-toolchain.toml")
rustup toolchain install "$RUSTUP_TOOLCHAIN" --profile minimal --no-self-update
rustc -vV
cargo -V
lscpu > "$out/lscpu.txt"

build_pair() {
  local rev=$1
  git checkout "$rev" -- bench/moonshot .just Justfile || return 1
  git -C "$base" checkout "$rev" -- bench/moonshot .just Justfile || return 1
  MOONSHOT_CANDIDATE="$base/bench/moonshot/candidates/lazy_cells.rs" \
    CARGO_TARGET_DIR="$RUNNER_TEMP/astra-e-lazy" \
    cargo build --locked --release --manifest-path "$base/bench/moonshot/Cargo.toml" || return 1
  MOONSHOT_CANDIDATE="$root/$candidate" CARGO_TARGET_DIR="$RUNNER_TEMP/astra-e-candidate" \
    cargo build --locked --release --manifest-path bench/moonshot/Cargo.toml || return 1
}

chosen=$fresh
if ! build_pair "$fresh"; then
  test "$(git rev-parse "$last:bench/moonshot")" != "$(git rev-parse "$fresh:bench/moonshot")"
  echo "Fresh scorer build failed. Retrying both builds at $last."
  chosen=$last
  build_pair "$last"
fi
base_bin=$RUNNER_TEMP/astra-e-lazy/release/moonshot
head_bin=$RUNNER_TEMP/astra-e-candidate/release/moonshot
{
  echo "base $BASE_SHA"
  echo "head $HEAD_SHA"
  echo "fetched master $fresh"
  echo "selected scorer source $chosen"
  git log -1 --format='scorer path revision %h %H' "$chosen" -- bench/moonshot
  sha256sum "$candidate" "$base_bin" "$head_bin"
  "$base_bin" info
  "$head_bin" info
} | tee "$out/manifest.txt"

bash bench/moonshot/corpus.sh
sha256sum corpora/wikidata.bz2 >> "$out/manifest.txt"
for arm in candidate lazy cells; do
  bin=$base_bin
  if [ "$arm" = candidate ]; then bin=$head_bin; fi
  "$bin" check --arm "$arm" --wikidata corpora/wikidata.bz2 \
    > "$out/$arm.check.jsonl" 2> "$out/$arm.check.stderr"
done
python3 research/astra-e/ci/paired.py "$base_bin" "$head_bin" corpora/wikidata.bz2 "$out"
cat "$out/comparison.md" >> "$GITHUB_STEP_SUMMARY"
# A fallback uses an older scorer than the fetched one. Let the next job retry the fresh scorer.
if [ "$chosen" = "$fresh" ]; then
  python3 research/astra-e/ci/inputs.py record "$RUNNER_TEMP/astra-e-inputs.json" "$BASE_SHA" \
    --depfile "$RUNNER_TEMP/astra-e-candidate/release/moonshot.d"
  echo "remember=true" >> "$GITHUB_OUTPUT"
fi
