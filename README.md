# Benchmark data

Data only, never merged: the input of mibz2's benchmark (`.github/workflows/bench.yml` on
master) that its sources no longer serve. The Benchmark workflow adds to it from master's
runs.

## windows/wikidata-20261019

12 byte-range windows of `wikidata-20261019-all.json.bz2` (103,488,697,311 bytes,
SHA-1 `a05b8b809ab80576da6d838c67138ddf401055e6`, from https://dumps.wikimedia.org/wikidatawiki/entities/20261019/wikidata-20261019-all.json.bz2): Byte-range windows of a dated Wikidata JSON dump (Wikimedia's own bzip2 blocks, CC0): 12 windows of 1 MiB at (2i + 1)/24 of the file. examples/windows.rs keeps each window's whole blocks, their bits untouched, as a stream of its own; the streams one after another are the corpus wikidata.bz2. `wikidata.json` beside
them (the repository's `bench/wikidata.json`) holds each window's offset, length and
SHA-256, and the corpus's SHA-256.

Wikidata's data is CC0.
