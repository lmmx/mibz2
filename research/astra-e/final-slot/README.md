# Repair sparse RLE1 edits inside the final worker slot

The candidate delegates marked reconstruction to the combined narrow-endpoint, short-word, one-edge-lookahead kernel. On its gated AVX2 route, when the caller's output is empty, the RLE1-coded result goes directly into the final output slot. The candidate parses all edits before moving any bytes. A bounded per-worker directory records each encoded source, decoded destination, run byte and count. This ports the signed displacement schedule of Astra D, using an owned edit directory because the research hook receives the consumed run records by shared reference. [Astra D implementation](https://github.com/lmmx/mibz2/blob/4f7d0235025940bd3d744463152515d189eb13dd/docs/journal/2026-11-18-astra-d-memory/direct.rs) · [Combined constructor](../narrow-short/)

Each record occupies eight bytes. The 20-bit source and 28-bit destination fields are sufficient under the strict input bound of 2²⁰ coded bytes, even at 259 expanded bytes per five coded bytes. The directory grows only as needed with a 512-record limit. Its maximum observed capacity was 4,096 bytes. The initial output reservation is n+256, giving 192 additional bytes beyond the stock AVX2 RLE1 reservation. This avoids a fallback for small expansions on fresh slots such as the frozen block's 69 bytes. Actual capacity remains a property of the caller's recycled vector.

## Exact reconstruction and fallback

Input token parsing starts at zero. After locating four equal bytes it consumes the following count and restarts at the next token. A missing count returns `RunUnfinished`. The AVX2 detector tests 32 candidate starts with four unaligned vectors when at least 35 bytes remain, then checks its short tail with scalar comparisons. The public hook declines an empty or oversized *row population* through its mark-validation, size or density gate, and declines nonempty caller output, unselected blocks and scalar kernels without touching the caller vector. The crate's ordinary combined inverse and stock finish then apply. No changes are required in the shipping crate or its allocator.

For an accepted repair, every literal span has disjoint final destination in original order. Move left-shifted spans first in increasing source order. Such a move cannot affect a later source, and its final destination begins after the end of every earlier right-shifted span's destination (and therefore after that earlier span's source). Move right-shifted spans next in decreasing source order; these cannot affect earlier sources. All completed destinations are disjoint. Given the captured run bytes and counts, fill run destinations last and truncate to final length. Zero displacement needs no literal movement. The argument covers arbitrary changes of the cumulative displacement's sign. The core library, through the existing optional expanded-output hook, checks block CRC, rolls back the output length on an error and returns the checked block CRC to the unmodified stream layer.

Layout is abandoned before writes if the directory would overflow or the final length would exceed the already allocated slot. The candidate transfers ownership of the encoded vector to a reusable spare using `mem::swap`, clears the destination and runs ordinary RLE1. No byte copy and no second inverse are required for this handoff. Stock RLE1 still sees and rejects a late malformed token beyond the directory limit. A fallback may keep a block-sized extra spare in that worker after the block. Kernel workspace, caller capacities and process RSS require separate accounting.

## Structural evidence

On the canonical Wikidata corpus, 227 of 228 selected blocks repair in place; one hit the directory limit and needed an additional stock RLE1 pass over 900,000 coded bytes. The selected blocks have 205,197,794 encoded bytes and 205,191,352 output bytes. Counting all recorded edits and the partially inspected fallback, the planner records 10,993 entries and scans through 204,669,009 bytes. The repair moves 155,645,358 literal bytes left, 28,163,527 right, fills 47,146 run bytes, and initializes 2,206 newly exposed vector-tail bytes. Its maximum retained fallback capacity in this one-worker probe was 900,256 bytes. These counts exclude the baseline input scan, BWT construction, CRC and scanner costs, which still run in the timed worker.

Frozen uses 34 edits. It moves 730,501 bytes right and fills 239 run bytes to produce 900,000 bytes. Its observed directory capacity was 512 bytes and the spare remained unallocated. The inverse stage in both the base candidate and this candidate is the same compiled source. Its structural auditor verifies every eligible block's RLE1-coded bytes independently against LazyCells. Frozen has 89,874 retained rows, 7,786 initial nodes, 11,215 elimination steps, 4,750 allocated binary rules, and 1,326 dictionary words. Per-block counters include the full residual table and dictionary sizes. These quantities identify the construction being held constant and price only the added layout work. [Edit counts](structure.json) · [Raw layout observations](wikidata.csv) · [Construction counters](construction.csv)

The focused checker compares the sparse result or its fallback with stock monolithic RLE1 across 31,626 scalar and detected cases. Complete blocks exercise libbzip2 agreement, arbitrary nonempty caller prefixes, corrupted CRC and retry, a one-row marked cycle among many LF components, and an unfinished run beyond a full directory. The scalar and occupied-slot branches explicitly return `None` before writing. [Receipt](check.txt)

## Reproduction

Run from the repository root with the release toolchain and the verified canonical corpus. The regular candidate has no structural counters.

```sh
python3 research/astra-e/final-slot/prepare.py
git fetch origin master && git checkout origin/master -- bench/moonshot .just Justfile
git log -1 --format=%h origin/master -- bench/moonshot
python3 bench/moonshot/moonshot.py score research/astra-e/final-slot/candidate.rs --with-opend
python3 docs/journal/2026-11-18-open-return/frozen.py corpora/wikidata.bz2 /tmp/final-slot-frozen.bz2
bash research/astra-e/final-slot/check.sh /tmp/final-slot-frozen.bz2
bash research/astra-e/final-slot/construct.sh corpora/wikidata.bz2 > /tmp/final-slot-construction.csv
MOONSHOT_CANDIDATE=research/astra-e/final-slot/probe.rs cargo build --locked --release --manifest-path bench/moonshot/Cargo.toml
bench/moonshot/target/release/moonshot run --arm candidate --file corpora/wikidata.bz2 --workers 1 >/dev/null 2>/tmp/final-slot.csv
python3 research/astra-e/final-slot/summarize.py /tmp/final-slot.csv
```
