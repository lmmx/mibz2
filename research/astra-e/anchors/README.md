# Dictionary-word anchors in the open residual permutation

After the width-16 queued Open-Return construction, retain every row whose outgoing edge emits a dictionary word, and retain the primary unconditionally. The remaining row labels are single bytes. Scan all residual entries once and replace each anchor entry by a compact anchor index with the sign bit set. Save its normal label and outgoing row in a separate 12-byte descriptor. On a wide-code block, move every literal into the unused high bits of its existing successor entry. A literal occupies at most 28 bits because block lengths are at most 900,000 and a code is at most 255.

Follow each anchor's outgoing path until the next tagged row. Stage its intervening literals in one of four streams, replacing the descriptor's original successor with that next anchor's compact index. Four independent lanes advance simultaneously, taking the next anchor in their own index residue class whenever a path finishes. Each descriptor stores its literal stream's starting offset. The descriptor four positions later supplies the ending offset, with four closing entries for the last anchors. Starting from the compact primary, emit each descriptor's original word (or literal primary) followed by its saved literal tail and advance to the next compact anchor.

## Correctness and accounting

In a finite permutation, each component is a cycle. Every cycle containing at least one anchor is partitioned into disjoint paths beginning at anchors and ending immediately before the next anchor. An empty tail represents adjacent anchors. A sole anchor in a cycle returns to itself. Each path's staged bytes are exactly the original sequence of literal labels following its anchor. Thus the new marked walk emits exactly one original first-return word before reaching the compact primary again. Repeating to block length preserves short marked cycles and a possible final partial repetition. Components consisting entirely of unmarked literals have no walk and cannot contribute to the marked output.

For `m` retained rows, `a` anchors and `t` literal rows belonging to anchored cycles, construction scans `m` entries, and independent discovery walks make `a+t` indexed reads, stage `t` bytes and change the anchor successors. The emitter makes one dependent descriptor lookup per marked anchor and performs a word copy and a literal-tail copy as appropriate. Additional logical storage is `12(a+4)+t` bytes, with actual retained vector capacity reported by the probe. The number of dependent descriptor lookups does not include time for construction and literal staging. The user's bound is whole-process peak RSS relative to LazyCells.

This differs from the historical width-16 switch constructor, which traces narrow paths directly through the piecewise LF map, and the historical ranked writer, which stages individual word codes for every retained rank. Here the existing constructed Open-Return permutation supplies path discovery, and only literals are staged.

## Reproduction

Prepare and test from the repository root.

```sh
python3 research/astra-e/anchors/prepare.py
rustc -O --edition 2021 research/astra-e/anchors/property.rs -o /tmp/astra-anchor-property
/tmp/astra-anchor-property
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
python3 bench/moonshot/moonshot.py score research/astra-e/anchors/candidate.rs --with-opend
```

The independent property test enumerates every permutation through five rows, all marks and all combinations of literal labels 0, 127 and 255 or dictionary label 300, in both the packed and wide layouts. Its expected result is the original marked walk.

```sh
MOONSHOT_CANDIDATE=research/astra-e/anchors/probe.rs cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml
bench/moonshot/target/release/moonshot run --arm candidate --workers 1 --file corpora/wikidata.bz2 >/dev/null 2> /tmp/astra-anchors.csv
python3 research/astra-e/anchors/summarize.py /tmp/astra-anchors.csv
```

The CSV columns are the prefix `anchors`, block length, primary, residual mark, retained rows, anchors, staged literal bytes, marked descriptor count, and capacity in bytes of the additional reusable scratch. It covers the 228 blocks entering the same OpenGated density route. In this corpus each has a single marked cycle spanning the retained rows; this was measured independently in the adjacent pairs experiment.

On the canonical Wikidata corpus (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`) the frozen block uses 15,278 compact marked lookups, down from 89,874. This requires 89,874 additional discovery reads and 74,596 literal tape bytes. The one-worker scratch capacity increase is 327,680 bytes on frozen. Across the 228 OpenGated blocks the marked lookups fall from 25,484,201 to 4,726,246 (81.45%), with 25,484,201 additional discovery reads, 20,757,955 staged literal bytes and a maximum 1,048,576-byte increase in reusable capacity. The probe used Rust 1.99.1 and scorer source `9c31461`. The corpus decode checked its normal CRCs. The 1,280,392 independent marked-permutation/layout checks are recorded in `property.txt`; the authoritative libbzip2 scorer gate is separately required.
