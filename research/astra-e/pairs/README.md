# In-place two-literal residual contraction

This candidate keeps OpenGated's admission, endpoint construction, dictionary and dense fallback. A sequential pass over the residual table replaces two adjacent ASCII literals with one packed edge. The 14-bit literal-pair code occupies unused high bits above all existing word codes. The pass is skipped when the block cannot represent every such code in its existing 32-bit entries. The shipping kernel, memory allocator and scorer are untouched.

For a row with edge labelled `a` to `v`, if `a` and the current label at `v` are both ASCII literals and `v` is not the mark, store their two-byte concatenation and the current successor of `v`. Rows can be visited in any order. Previously rewritten edges carry a nonliteral label, so they fail the eligibility test. The scan reads and writes the same table, uses no visited flags and allocates no row storage.

## Exactness

Expand each edge into its path in the unmodified residual permutation. Initially these paths have one edge and have no interior marked row. A rewrite concatenates two incident paths and preserves their spelling, direction and endpoint. Its only new interior junction is the tested row `v`, so the invariant persists. Starting at the mark, the rewritten walk advances by existing path lengths without skipping the first marked return. It therefore emits exactly the original marked-cycle spelling and terminates. Repeating the spelling to exactly the block length, including a final partial repetition, gives the original output on periodic and multiple-cycle blocks. Unmarked components need no output order. Self-loops off the marked orbit do not affect it.

The packed format fits when the greatest code, `256 + dictionary_words + 128² - 1`, is at most `u32::MAX >> successor_bits`. The first new code follows the last original dictionary code. The separate wide-code table and out-of-room blocks retain the original interpretation.

## Reproduction

In the repository root, use the normal Rust toolchain and corpus setup documented by the scorer. The generator checks the original Open-Return source digest and exposes its exact constructor under this directory. It also references the unchanged endpoint source in the repository. No patch to the scorer is needed.

```sh
python3 research/astra-e/pairs/prepare.py
rustc -O --edition 2021 research/astra-e/pairs/property.rs -o /tmp/astra-pair-property
/tmp/astra-pair-property
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
python3 bench/moonshot/moonshot.py score research/astra-e/pairs/candidate.rs --with-opend
```

To collect just the exact structural counters, build the instrumented candidate, which embeds the same writer with compile-time counters and a walk over the original marked component. The probe's timed-run line has no performance interpretation.

```sh
MOONSHOT_CANDIDATE=research/astra-e/pairs/probe.rs cargo build --release --locked --manifest-path bench/moonshot/Cargo.toml
bench/moonshot/target/release/moonshot run --arm candidate --workers 1 --file corpora/wikidata.bz2 >/dev/null 2> /tmp/astra-pairs.csv
python3 research/astra-e/pairs/summarize.py /tmp/astra-pairs.csv
```

The raw CSV has no header. Fields in order are the literal prefix `pairs`, block length, primary row, mark in the residual, bits per successor, retained rows, word count, packing applicability flag, extra indexed successor reads, rewritten rows, original marked hops and final marked hops. Rows cover blocks inside opend's density gate only. In the one-worker corpus scan, the first row is the frozen block (ordinal 1). `property.txt` exhausts every permutation through five rows, every mark and all labelings by two ASCII literals, one non-ASCII literal and one dictionary word.

The contraction pass reads all eligible rows sequentially, performs the counted extra indexed reads and dirties lines on each rewrite. Every visited pair emits the same two bytes. A decrease in marked chain length does not price these operations or predict the result on the i9.

## Corpus structural result

On the canonical Wikidata corpus (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`), OpenGated selects 228 blocks. Of these, 187 can hold the extra pair codes. The full marked traversal falls from 25,484,201 to 19,355,474 hops (24.05%). The extra pass performs 15,160,694 indexed successor reads and writes 8,909,398 edge entries. The frozen block falls from 89,874 to 59,511 hops (33.78%) at a cost of 74,595 lookups and 44,257 rewrites. These are exact structural counts, before timing or cache modelling. Raw observations are in `wikidata.csv`, with `structure.txt` produced by the reduction above. The corpus probe was compiled with rustc 1.90.0 against scorer source `9c31461`; the canonical scoring campaign uses the pinned scorer build instructions.
