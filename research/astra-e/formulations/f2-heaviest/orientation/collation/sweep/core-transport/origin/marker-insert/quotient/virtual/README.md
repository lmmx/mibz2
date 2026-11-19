# Stream the only query to the delimiter bucket

For a token BWT with chosen delimiter d, let B=[b,b+s) be the source bucket of d. The contracted second trie queries B exactly once during discovery. After its initial query, every extension either stops on the symbol d, or takes a child interval contained in the source bucket of a different symbol. This invariant holds through unary contraction, coalescing equal children, and multiple LF cycles. If the marked shift starts outside B, every examined row is outside B until it maps to B and stops. If it starts inside, only its first query can require the contents of B. Capture that one row during the root scan. A cycle missing the delimiter still uses the ordinary fallback.

Index the complementary m−s rows after deleting B from the coordinates. A non-d LF target uses its original source ordinal below B and subtracts s above it. A d target needs only its ordinal within B. Every nonroot interval lies wholly on one side, so the traversal and its ordinary within-page lookup can remain in the compact coordinate space throughout. A root reducer streams selectors and child interval endpoints. At most one pending input piece is needed to join two adjacent equal labels before this reducer. A coincident pair across the cut can share one indexed piece only if both its word and LF translation agree. Retaining the cut as a boundary is always correct.

## Direct quotient stream from the old first-word root

Use the old incoming RLE L, selected IDs S, and coincident old ID Z. Write z for the old Z count and s for the new marker count. In a first scan record each non-Z selected run's old word, width, number of preceding Z rows, and same-ID occurrence count. In a second scan of L relabel selected words to Z to produce the ordinary background. For each background row y whose original label is selected, its LF is the new base of Z plus the number of earlier **selected** old rows at y. For an unselected label c it is the new base of c plus its old c occurrence count. An inserted event of label j maps to the new base of j plus its saved old j occurrence count. Insertions have positive width and appear in chronological order at gaps b+preceding-Z of the old background. The resulting pieces, with their LF translations, are already in incoming order.

Write only the background outside the old Z bucket to the search index. Send background inside and interleaved events to the root reducer instead. This uses two sequential scans of R old records, O(K) counts/bases for K words, O(k) event descriptors, and O(R_out+(m−z)/64) run/page records. The old first-word root still has to exist, its original counts must be available or collected, the k descriptors need construction, and the root selector and child tables still have to be written. There is no request for an index over the inserted events. A marker losing the heaviest-word tie can use the ordinary quotient path. The underlying single-root-query lemma also applies to a different winning delimiter, including the named Compact baseline.

The falsifier generates the virtual root and outside records directly from L. An independent explicit quotient supplies the full column and its LF table. The checker compares all row translations and maximal pieces on the root and each side, then replays each contracted look-up, branch and terminal interval of native Second.discover. For auditing it collects the streamed root in a list. The constructive bound above uses a streaming sink. All 10,170 small objects and 75,495 positions match. The 7,650 nontrivial chosen-marker tries needed 7,650 root queries over 20,954 nodes. On each of ten systematic Wikidata tapes the same check ran for the old heaviest word and three quotient depths, and all forty selected delimiters matched the intended class. [Implementation](view.py) · [Finite evidence](small.json) · [Per-tape measurements](wikidata.csv)

| Ten-block sum | Ordinary indexed records | Complement indexed records | Root pieces streamed | Ordinary / complement 64-row pages | Hazard records | Nonroot lookups |
|---|---:|---:|---:|---:|---:|---:|
| Named original | 84,317 | 78,647 | 5,670 | 21,209 / 17,372 | 0 | 69,989 |
| Quotient h4 | 84,427 | 45,501 | 38,918 | 23,140 / 18,732 | 4,104 | 125,926 |
| Quotient h8 | 84,995 | 44,251 | 40,736 | 23,241 / 18,732 | 5,562 | 124,496 |
| Quotient h12 | 84,641 | 39,458 | 45,175 | 23,414 / 18,732 | 9,990 | 124,494 |

At h4 eliminating root-run indexing removes 46.1% of run records and 19.0% of rank pages in Second's column. The stock element sizes are 12 bytes per run, 4 bytes per page index and 8 bytes per mask. These counts exclude reserved vector capacity and the final output. H4 still takes 125,926 nonroot rank lookups against 69,989 in named Compact. The net complete runtime needs a paid native comparison.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/virtual
python3 -B "$d/view.py" | diff - "$d/small.json"
python3 -B "$d/view.py" research/astra-e/dense/target/wikidata | diff - "$d/wikidata.csv"
```
