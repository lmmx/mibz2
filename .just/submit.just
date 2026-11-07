# Workstation evidence, submitted (RULES C5): `just submit <recipe> [args]` runs the recipe
# exactly as `just <recipe>` would, then archives the bundle it made on the `evidence` branch
# and posts its identity and digest on #145. Opt-in: the bare `just <recipe>` submits
# nothing. A failed submission keeps the bundle: `just submit-bundle <tarball>` retries it
# without running anything again.

# run a workstation recipe, then submit the bundle it made as evidence
submit recipe *args:
    #!/usr/bin/env bash
    set -uo pipefail
    gh auth status >/dev/null 2>&1 || { echo "submit: gh isn't signed in; run gh auth login first (nothing was run)"; exit 1; }
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

# wait for the coordinator's ready tag on origin, then run `just submit <recipe>` once and suspend
submit-when-ready recipe branch *args:
    bash bench/await-submit.sh {{ recipe }} {{ branch }} {{ args }}
