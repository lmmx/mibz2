#!/usr/bin/env bash
# Submit one workstation bundle as evidence (RULES C5): archive it on the orphan `evidence`
# branch and post its identity and digest on #145. Run by Louis, through `just submit
# <recipe>` (which runs the recipe and then this) or `just submit-bundle <tarball>` (a retry).
#   bash bench/submit.sh target/three-team-policy/20261106T100854Z.tar.gz
#
# Idempotent by run and digest: a bundle already on the branch isn't committed again, a comment
# already on #145 for it isn't posted again, and a different bundle under the same run is
# refused. A failure keeps the bundle and changes nothing published; run it again to retry.
#
# The run's identity is read from the bundle (its directory, outcome.txt, commits.txt,
# inputs.txt), never typed in.
set -euo pipefail
repo=${EVIDENCE_REPO:-lmmx/mibz2}      # (both overridable only so the tests can use a scratch
remote=${EVIDENCE_REMOTE:-origin}      #  repository; a real run uses neither)
gh=${GH:-gh}
channel=145
branch=evidence

die() { echo "submit: $*" >&2; exit 1; }
tarball=$(realpath "${1:?usage: bench/submit.sh BUNDLE.tar.gz}")
[ -f "$tarball" ] || die "no bundle at $tarball"
case "$tarball" in *.tar.gz) ;; *) die "$tarball is not a .tar.gz bundle" ;; esac

# identity: the recipe's directory and the attempt's timestamp, as bundle.sh named them
stamp=$(basename "$tarball" .tar.gz)
recipe=$(basename "$(dirname "$tarball")")
run="$recipe/$stamp"
[[ $run =~ ^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$ ]] || die "can't name a run from $tarball"
size=$(stat -c %s "$tarball")
[ "$size" -lt $((95 * 1024 * 1024)) ] || die "$tarball is over 95 MB; GitHub refuses files over 100 MB"
digest=$(sha256sum "$tarball" | cut -d' ' -f1)
member() { tar -xzOf "$tarball" "$stamp/$1" 2>/dev/null || true; }
tar -tzf "$tarball" >/dev/null || die "$tarball isn't a readable gzip tarball"
outcome=$(member outcome.txt); outcome=${outcome:-"(no outcome.txt in the bundle)"}
commits=$(member commits.txt)
inputs=$(member inputs.txt | head -4)

# where it is published
url=$(git remote get-url "$remote") || die "no git remote $remote"
case "$url" in *"$repo"|*"$repo.git"|*"$repo/") ;; *) die "remote $remote is $url, not $repo" ;; esac
"$gh" auth status >/dev/null 2>&1 || die "gh isn't signed in; run: gh auth login (the bundle is kept)"

# 1. the archive: one commit on the orphan branch adding $run.tar.gz and $run.tar.gz.sha256,
#    made with git's plumbing, so neither the checkout nor its index is touched
path="$run.tar.gz"
blob=$(git hash-object -w "$tarball")
for attempt in 1 2 3; do
  git fetch -q "$remote" "+refs/heads/$branch:refs/remotes/$remote/$branch" 2>/dev/null || true
  parent=$(git rev-parse -q --verify "refs/remotes/$remote/$branch^{commit}" || true)
  if [ -n "$parent" ] && have=$(git rev-parse -q --verify "$parent:$path" 2>/dev/null); then
    [ "$have" = "$blob" ] || die "$path is already published with different contents; a run is published once"
    commit=$(git log -1 --format=%H "$parent" -- "$path")
    echo "submit: $path is already archived at $commit"
    break
  fi
  index=$(mktemp -u)
  export GIT_INDEX_FILE=$index
  [ -n "$parent" ] && git read-tree "$parent"
  git update-index --add --cacheinfo "100644,$blob,$path"
  sum=$(printf '%s  %s\n' "$digest" "$stamp.tar.gz" | git hash-object -w --stdin)
  git update-index --add --cacheinfo "100644,$sum,$path.sha256"
  tree=$(git write-tree)
  unset GIT_INDEX_FILE; rm -f "$index"
  commit=$(git commit-tree "$tree" ${parent:+-p "$parent"} -m "evidence: $run" -m "sha256 $digest")
  if git push -q "$remote" "$commit:refs/heads/$branch"; then
    echo "submit: archived $path at $commit"
    break
  fi
  # (refused: someone else published first, so build again on theirs; never a forced push)
  commit=""
  sleep $((attempt * 2))
done
[ -n "${commit:-}" ] || die "couldn't push to $branch (the bundle is kept; run the same command again)"

# 2. the notice on #145, once: the marker names the run and its digest
marker="<!-- evidence: $run sha256:$digest -->"
posted=$("$gh" api --paginate "repos/$repo/issues/$channel/comments?per_page=100" \
  --jq ".[] | select(.body | contains(\"$marker\")) | .html_url" | head -1)
if [ -n "$posted" ]; then
  echo "submit: already posted: $posted"
  exit 0
fi
head=$(awk '$1 == "head" {print $2}' <<<"$commits")
body=$(mktemp)
{
  echo "FROM: louis via workstation"
  echo "TO: coordinator"
  echo "RE: $run${head:+ @ $head}"
  echo "TYPE: LOG"
  echo
  echo "$marker"
  echo "A workstation run submitted with \`just submit\` (RULES C5)."
  echo
  echo "- **Run:** \`$run\`"
  echo "- **Outcome:** $outcome"
  echo "- **Bundle:** [\`$path\`](https://github.com/$repo/blob/$commit/$path) at $commit, $size bytes"
  echo "- **SHA-256:** \`$digest\`"
  echo
  if [ -n "$commits$inputs" ]; then
    echo '```'
    [ -n "$commits" ] && echo "$commits"
    [ -n "$inputs" ] && echo "$inputs"
    echo '```'
    echo
  fi
  echo "Fetch and check: \`git fetch origin $branch && git show $commit:$path > b.tar.gz && sha256sum b.tar.gz\`"
} > "$body"
posted=$("$gh" api "repos/$repo/issues/$channel/comments" -F "body=@$body" --jq .html_url)
rm -f "$body"
echo "submit: posted $posted"
