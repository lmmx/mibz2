#!/usr/bin/env bash
# Prepare external constructor modules for the Rust scorer; the other entries are standalone.
set -euo pipefail
case "/$1" in
  */research/astra-e/front-fusion/edge/certificate/candidate.rs)
    python3 "$(dirname "$1")/../../../edge-budget/prepare.py"
    ;;
  */research/astra-e/worker-dose/*.rs)
    python3 "$(dirname "$1")/../edge-budget/prepare.py"
    ;;
  */research/astra-e/front-fusion/cap-batch/worker/candidate.rs|*/research/astra-e/front-fusion/precount/worker/candidate.rs|*/research/astra-e/front-fusion/edge/worker/candidate.rs)
    python3 "$(dirname "$1")/prepare.py"
    ;;
  */research/astra-e/formulations/f2-rle/*.rs|*/research/astra-e/formulations/f5/kernel/lean/*.rs|*/research/astra-e/formulations/f5/kernel/*.rs|*/research/astra-e/formulations/f2-endpoint/*.rs|*/research/astra-e/final-slot/*.rs|*/research/astra-e/edge-budget/*.rs|*/research/astra-e/streamed/*.rs|*/research/astra-e/narrow-short/*.rs|*/research/astra-e/narrow-endpoint/*.rs|*/research/astra-e/pairs/*.rs|*/research/astra-e/anchors/*.rs|*/research/astra-e/lookahead/*.rs|*/research/astra-e/dictionary/*.rs)
    python3 "$(dirname "$1")/prepare.py"
    ;;
esac
