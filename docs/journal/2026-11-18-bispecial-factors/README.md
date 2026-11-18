# Reproduce the factor-support inverse

The single-file Rust inverse takes the 67,024 bytes in this directory's evidence and writes the
frozen pre-RLE text. Its optional third argument writes the original byte BWT when the input is
one periodic orbit. It needs no dependencies or precomputed dictionaries. The journal describes
why this works and the separate counting problem at finite context length.
[Journal entry](../2026-11-18-bispecial-factors.md) · [Counting proof](counting.md)

From the repository root, with a current Rust compiler on PATH, run the following. The recorded
runs used rustc 1.90.0.

```sh
study="$PWD/docs/journal/2026-11-18-bispecial-factors"
out=$(mktemp -d)
rustc -O --edition 2021 "$study/support.rs" -o "$out/support"
"$out/support" "$study/evidence/frozen.support" "$out/raw" "$out/column"
sha256sum "$out/raw" "$out/column"
```

The program reports 899,931 terminal futures and a marked period of 899,931. Expected SHA-256
values, in order, are

```text
f87e5caa2d5f0df9d5958fe60901e2023ae04ac406603d4c1584c8bb4017fbd6
db8764224ac4980112ddefbe85b8593f5236643fed71bf8f0b739567d4f2ac67
```

The 52-byte SUP1 header contains the four-byte signature, 32-bit block length, 32-bit
distinct-future rank of the mark, a 256-bit alphabet-presence mask, and a 64-bit support bit count.
Integers are little-endian. The body is the row-major incidence of each successive bispecial
context, packed least-significant bit first. The two alphabets of an incidence matrix are
inherited by the parser. It discovers splits in FIFO order. The implementation is a research
reader for these bounded blocks. Its assertions check the invariants of the supplied binary data.

## Rebuild the support from the published byte column

The historical frozen data are on a pinned research commit. Extract them independently and check
the reconstructed files. The path named baseline-rle1 contains the bytes entering RLE1.

```sh
base=bf88365ba80388e17488a31c80d2f3f250b19fb2
records=docs/journal/2026-11-11-first-return-block/evidence
git fetch --depth=1 origin "$base"
git show "$base:$records/column.bin.gz" | gzip -dc > "$out/reference-column"
git show "$base:$records/baseline-rle1.bin.gz" | gzip -dc > "$out/reference-raw"
cmp "$out/raw" "$out/reference-raw"
cmp "$out/column" "$out/reference-column"
rustc -O --edition 2021 "$study/contexts.rs" -o "$out/contexts"
"$out/contexts" support "$out/reference-column" 264045 "$out/rebuilt.support"
cmp "$out/rebuilt.support" "$study/evidence/frozen.support"
"$out/contexts" pack "$out/reference-column" "$out/weighted" "$out/levels.csv"
"$out/contexts" unpack "$out/weighted" 264045 "$out/weighted-column" "$out/weighted-raw"
cmp "$out/reference-column" "$out/weighted-column"
cmp "$out/reference-raw" "$out/weighted-raw"
cmp "$out/levels.csv" "$study/evidence/frozen-levels.csv"
```

The research encoder builds a rank index of the input last column to discover the edges and
counts. EXT1 is a comparison format containing a full byte histogram, the same binary supports
and residual chord weights. Its frozen file is 92,119 bytes. SUP1 keeps the support stream from
that construction and carries the translated mark. The independent SUP1 executable above reads
only SUP1. Its SHA-256 on the frozen input is
`ea6b418be9192c2e3e4fdd8a2c45af696ae5060335d53b7937a9bedce85e10f8`.

## Exhaustive and identifiability checks

```sh
rustc -O --edition 2021 --test "$study/contexts.rs" -o "$out/tests"
"$out/tests" --nocapture
python3 "$study/count_classes.py"
python3 "$study/assembly.py"
python3 "$study/count_classes.py" "$out/reference-column" "$out/reference-raw" "$out" > "$out/trees.csv"
python3 "$study/assembly.py" "$out/reference-column" "$out/reference-raw" 264045 > "$out/assembly.csv"
python3 "$study/assembly.py" witness "$out/reference-column" "$out/reference-raw" 264045 4743 "$out/alternative.raw" > "$out/witnesses.csv"
rustc -O --edition 2021 "$study/compare_spectra.rs" -o "$out/compare-spectra"
"$out/compare-spectra" "$out/reference-raw" "$out/alternative.raw" 4744
```

The last two lines printed by the comparison are

```text
4744,899853,0,0
4745,899856,4,4
```

The fields are substring length, distinct strings in the union of the two cyclic spectra, number
of strings whose multiplicities differ, and the sum of the absolute multiplicity differences.
The witness text is reproducible from the three bytes in the candidate column. The original
interval starts at zero-based row 138,927. Its bytes are `6d 6e 65`. Replacing them in order by
`65 6d 6e` gives the alternative primitive text at the same numerical primary. The file
`frozen-witnesses.csv` gives both SHA-256 values and all candidate cycle decompositions.

## Full bzip2 stream check

Use the shipped scanner, inverse MTF, Plain inverse, RLE1 and CRC with the research example.
The Rust example can live in a temporary checkout. Set the toolchain to one installed locally.
The repository's own corpus builder needs Python, git and lbzip2.

```sh
git worktree add --detach "$out/mibz2" 84143ba0a74cf2e61ac633fdbb8e9173f82cf45b
cd "$out/mibz2"
mkdir -p examples/bispecial-evidence
cp "$study/contexts.rs" "$study/support.rs" examples/bispecial-evidence/
cp "$study/corpus.rs" examples/bispecial-evidence/main.rs
export RUSTUP_TOOLCHAIN=1.90.0
bash bench/corpus.sh
cargo build --release --example bispecial-evidence
target/release/examples/bispecial-evidence corpora/wikidata.bz2 "$out/decoded" > "$out/wikidata.csv"
bzip2 -dc corpora/wikidata.bz2 | cmp - "$out/decoded"
cmp "$out/wikidata.csv" "$study/evidence/wikidata.csv"
git fetch --depth=1 origin data/induction-records
git show FETCH_HEAD:corpora/prose.bz2 > "$out/prose.bz2"
target/release/examples/bispecial-evidence "$out/prose.bz2" "$out/prose.out" > "$out/prose.csv"
bzip2 -dc "$out/prose.bz2" | cmp - "$out/prose.out"
```

For each block the example constructs both experimental representations, recovers L from the
weighted stream, compares both pre-RLE outputs with Plain, and finishes the support-only output
using normal RLE1 and CRC. It also compares the support-only reconstructed L whenever the orbit is
single. All 431 Wikidata and six prose blocks in these runs had a single primitive orbit. The
small-column tests exercise other cases. Final Wikidata SHA-256 is
`3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf` and prose SHA-256 is
`0da092515b0911aad1f23304a7d5e9d058669191e5c1070afe0d43a99fb98959`.

In `frozen-levels.csv`, a row consumes the extension tables for contexts of length k. Its
`log2_class_drop` is the reduction from the class given the $(k+1)$-mer spectrum to the class
given the $(k+2)$-mer spectrum. In the corpus files `depths` counts waves, `right_special`
includes forced one-predecessor splits, `forests` counts bispecial contexts of cycle rank zero,
`weight_fields` counts residual integers in EXT1, and `positive_edges` counts present incidences
at all right-special contexts. SUP1 sizes include the per-block header and byte padding.
[Recorded evidence](evidence/)
