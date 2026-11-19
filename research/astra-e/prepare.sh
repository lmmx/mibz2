#!/usr/bin/env bash
# Prepare external constructor modules for the Rust scorer; the other entries are standalone.
set -euo pipefail
case "/$1" in
  */research/astra-e/edge-budget/*.rs|*/research/astra-e/streamed/*.rs|*/research/astra-e/narrow-short/*.rs|*/research/astra-e/narrow-endpoint/*.rs|*/research/astra-e/pairs/*.rs|*/research/astra-e/anchors/*.rs|*/research/astra-e/lookahead/*.rs|*/research/astra-e/dictionary/*.rs)
    python3 "$(dirname "$1")/prepare.py"
    ;;
esac
