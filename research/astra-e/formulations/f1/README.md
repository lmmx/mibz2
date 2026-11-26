# F1. First return to dictionary anchors without a retained-row table

The input is the capped L run stream and primary. Width-16 Open-Return produces labeled translations P from retained F entrances to L exits and a dictionary for wide excursions. The independent narrow wiring S is stable sort of the retained L letters. In retained L coordinates the residual backward/text-order motion is T = S inverse after P inverse. At bottom rank j, the open band supplies the top argument k and the word to emit. Restrict k to an interval from one retained L run. S inverse then sends successive k to successive retained L ranks. Thus T is a labeled interval translation. The algorithm never allocates an m-entry residual successor table.

Each byte has a queue of *thin run intervals* giving its exclusive top end and first retained L rank. For each bottom-order open band, find its starting select interval by binary search and split only where its top arguments cross run boundaries. Join adjacent pieces with the same displacement and code. This constructs the residual translations, costing one 8-byte record per thin run, one binary search per band and a 12-byte tile per unsuppressed output piece. The preserved Open-Return endpoint module at `775e604` supplies the complementary elimination and dictionary spelling. They remain paid stages.

The anchors are the union of dictionary-labeled source intervals and the mark. For the leftmost uncovered anchor x, follow one representative through T until reaching the next anchor, collecting the literal tail after its first label. Take the minimum distance to the right edge of every tile encountered, including the final anchor tile and the starting anchor range. By induction on steps, displacing x by any smaller nonnegative offset displaces every subsequent point by exactly that offset, preserving both labels and its first return. This yields a translation on compact anchor ranks with a shared label and tail. Splitting at a literal primary makes its membership constant on all used tiles. Each finite permutation cycle with an anchor partitions into these paths. A cycle without an anchor cannot contain the mark. Starting at the compact mark and concatenating until its next return, then repeating and truncating to n, is consequently exact for disconnected and periodic columns. The implementation's cold verifier compares this finished pre-RLE1 sequence to the ordinary dense Plain inverse.

The previous two-sided switch tracer discovered both colors by tracing the original strip graph. Here endpoint elimination has already resolved the wide side and its dictionary; all later tracing is over retained coordinates and uses those dictionary labels as opaque first words. The 81% anchor reduction observed earlier built the full retained successor table first and staged separate literal bytes for individual anchored paths. This construction omits that table and can share tails across translated anchor ranges. It pays for interval composition, predecessor queries, a compact anchor map, output ordering and all output copies.

## Exact screen

```sh
d=research/astra-e/formulations/f1
cargo run --release --locked --manifest-path "$d/Cargo.toml" -- small
sha256sum corpora/wikidata.bz2
cargo run --release --locked --manifest-path "$d/Cargo.toml" -- corpora/wikidata.bz2
```

The corpus SHA-256 is `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. `small.txt` checks every binary column of lengths 1–10, every mark, and width cutoffs 1–3. `blocks.csv` checks every 24th of the 233 large canonical Wikidata blocks. Nine in this ten-block sample satisfy the production front and density gates, and all ten match Plain. The single excluded sample is tagged in the raw data. This structural screen has no wall times, resident memory measurements or libbzip2 stream verdict.

| Count | Frozen block | Nine admitted samples combined |
|---|---:|---:|
| Retained rows | 89,874 | 956,546 |
| Thin-run select intervals | 47,736 | 529,265 |
| Residual translation tiles | 47,547 | 527,921 |
| Anchor rows | 15,278 | 175,410 |
| Macro bands | 8,041 | 89,936 |
| Width-one macro bands | 5,975 | 67,533 |
| Tile queries, including stop tests | 69,140 | 760,725 |
| Extra endpoint comparisons after page lookup | 160,275 | 1,808,162 |
| Staged representative tail bytes | 53,058 | 580,853 |

The inexpensive falsifier asked whether the translated ranges suppress enough dependent lookups and metadata to buy the omitted row table. On the admitted sample, 75.1% of the macro bands have width one. The eight-row directory's query count is 79.5% of m before accounting for the additional endpoint comparisons. Its 12-byte piece table alone occupies 1.66 times the old four-byte row table. On frozen, select queues and tiles simultaneously contain 952,452 logical bytes while composing T. During later tracing, the tiles, directory, compact anchor ranges, output macro records and literal tape total 801,118 logical bytes. The frozen dense-anchor reference uses 617,476 bytes for its old edges, compact descriptors and staged tails, apart from its own open-construction queues. The two designs retain different temporaries. No complete-process RSS ranking follows from these byte counts. The compact macro stream alone drops to 181,714 bytes from 257,980 for individual descriptors and tails.

The measured fragmentation rejects the explicit 12-byte tile/8-byte select representation as a way to save working space at construction. A bit-packed interval-start index with per-start payloads and direct two-pass band/run composition would remove the select queues and shorten each predecessor query. Its extra scans and rank computations must be included in any subsequent comparison. The emitter here binary-searches the macro band for each marked anchor and copies its words. A dense table over only the anchors is a distinct storage/lookup tradeoff.
