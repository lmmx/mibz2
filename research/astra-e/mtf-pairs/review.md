# Independent review of the two-bank MTF worker

The two-bank mask composes the two original destination-to-source permutations. The second nonzero rank shifts the first output to lane one. Both selected bits are below EOB in the same bitmap word, so the generated code emits the original intervening span before consuming the second literal. An error during that emission can contaminate private scratch only. The normal entry point calls the private hook after the original run gate. A rejected hook leaves the output untouched and the ordinary front, primary check, inverse, RLE1, block CRC and rollback still run. The original and candidate apply the same width-16 scalar-cap admission. No mismatch emerged from the table, dispatch or marked-output review.

A large dense decline exercises an expense absent from the 32 actual Wikidata retries. Their largest BWT has 1,185 rows, so their geometry skips the expensive size test. A valid level-9 bzip2 input with the symbol sequence `[2,RUNA]` repeated 140,000 times and one EOB reconstructs a 280,000-row BWT with 140,000 two-byte runs. All rows are retained. The proposed edge table needs 1,120,000 bytes and exceeds the 1 MiB admission limit. The hook and the fallback both run the full cap-width scan over all 140,000 records. Each retained front has 644,228 bytes of fresh record/bitmap capacity. The new worker carries both sets concurrently, in addition to the private front's 2,120 bytes of fixed counters and Vec headers. Neither the shared 64 KiB shuffle table nor the fallback inverse's own working storage is included in that capacity figure.

The test compared every capped word and both complete counter arrays, verified that the actual hook declines without altering existing output, reconstructed marked bytes from a separate LF table and checked the complete fallback output and CRC. A separately encoded 70,736-byte compressed stream decoded under libbzip2 to those same 271,164 bytes. Each of the three originally gated worker binaries also decoded that stream successfully. The fixture quantifies a high-water layout and a duplicated scan on a valid input. The paid Wikidata comparisons include the actual fallback incidence and retain their original interpretation. [Raw receipt](review-decline.txt) · [Native driver](review-decline.rs) · [Bitstream and libbzip2 oracle](review-decline.py)

Use the prepared 01e44cbe worktree and release dependencies/three binaries from the existing worker reproduction.

```bash
bash research/astra-e/mtf-pairs/review-decline.sh "$tree" "$work/build/release/deps" "$work/bin"
```
