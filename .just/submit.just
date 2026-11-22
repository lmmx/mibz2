# Workstation evidence, submitted (RULES C5): `just submit <recipe> [args]` runs the recipe
# exactly as `just <recipe>` would, then archives the bundle it made on the `evidence` branch
# and posts its identity and digest on #145. Opt-in: the bare `just <recipe>` submits
# nothing. A failed submission keeps the bundle: `just submit-bundle <tarball>` retries it
# without running anything again.
# One run at a time on the machine: `just submit` holds the flock on
# ${XDG_STATE_HOME:-~/.local/state}/workstation/queue.lock while the recipe runs (or keeps the
# one the workstation queue passed down), and refuses to start while anything else holds it.

# run a workstation recipe, then submit the bundle it made as evidence
submit recipe *args:
    #!/usr/bin/env bash
    set -uo pipefail
    gh auth status >/dev/null 2>&1 || { echo "submit: gh isn't signed in; run gh auth login first (nothing was run)"; exit 1; }
    if ! { [[ ${WORKSTATION_LOCK_FD:-} =~ ^[0-9]+$ ]] && flock -n "$WORKSTATION_LOCK_FD" 2>/dev/null; }; then
      lock=${XDG_STATE_HOME:-}; [[ $lock == /* ]] || lock=$HOME/.local/state
      lock=$lock/workstation/queue.lock
      mkdir -p "${lock%/*}" && exec 9>>"$lock" || { echo "submit: could not open the machine lock $lock (nothing was run)"; exit 1; }
      flock -n 9 || { echo "submit: another workstation run holds the machine lock $lock ($(cat "$lock" 2>/dev/null)); nothing was run"; exit 1; }
      echo "pid $$, just submit {{ recipe }}, since $(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$lock"
    fi
    before=$(ls target/*/*.tar.gz 2>/dev/null | sort)
    {{ just_executable() }} {{ recipe }} {{ args }}
    status=$?
    made=$(comm -13 <(echo "$before") <(ls target/*/*.tar.gz 2>/dev/null | sort) | grep . || true)
    [ "$(echo "$made" | grep -c .)" = 1 ] || { echo "submit: {{ recipe }} made $(echo "$made" | grep -c .) bundles, not one; submit one with: just submit-bundle <tarball>"; exit 1; }
    bash bench/submit.sh "$made" || { echo "submit: not submitted; the bundle is kept. Retry with: just submit-bundle $made"; exit 1; }
    exit $status

# submit a bundle already made (a retry: nothing is run again)
submit-bundle tarball:
    bash bench/submit.sh {{ tarball }}

# wait for the coordinator's ready tag on origin, then run `just submit <recipe>` once (under the machine lock) and suspend
submit-when-ready recipe branch *args:
    bash bench/await-submit.sh {{ recipe }} {{ branch }} {{ args }}

# run every ready queue entry once, then suspend: this checkout's, or every --repo PATH / ~/.config/workstation/repos queue in turns, under one machine lock
workstation-queue *args:
    bash bench/run-queue.sh {{ args }}

# list the checkouts the workstation queue serves (~/.config/workstation/repos), adding any given
workstation-repos *paths:
    #!/usr/bin/env bash
    set -euo pipefail
    c=${XDG_CONFIG_HOME:-}; [[ $c == /* ]] || c=$HOME/.config
    f=$c/workstation/repos
    for p in {{ paths }}; do
      d=$(git -C "$p" rev-parse --show-toplevel 2>/dev/null) || { echo "workstation-repos: $p is not a git checkout"; exit 1; }
      mkdir -p "${f%/*}"; touch "$f"
      if grep -qxF "$d" "$f"; then echo "already listed: $d"; else echo "$d" >> "$f"; echo "added: $d"; fi
    done
    if [[ -f $f ]]; then echo "$f:"; sed 's/^/  /' "$f"; else echo "no $f: just workstation-queue serves the checkout it is run from"; fi
