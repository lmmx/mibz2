# The factors: the decoder's parts, and how each is made ideal

**What "ideal" means.** The part's output is supplied, already computed, recorded outside the clock from a real decode of the same input. Everything downstream consumes it exactly as it would consume the real part's output.
- **The check:** the decoder's output is byte-identical to the shipped decoder's, so the oracle supplies exactly what the part would have produced.
- **What the oracle costs:** its memory (the recorded output must be held) and the time to hand it over. Each run reports both, so an ideal arm is never mistaken for a free one.

| factor | part | where it runs | what "ideal" supplies | oracle | target (`../math-targets/targets.tsv`) |
|---|---|---|---|---|---|
| S | **the Huffman decode** (with the bitmap, selectors and tables: the whole scan) | the serial scanner | each block's symbol stream (MTF/RLE2 symbols) and its boundaries | **exists:** `scan-oracle` (four-arm arm C, `src/scan_oracle.rs` on #835) | T4 |
| E | **emission** (writing the text from Open's structures in output order) | workers | the output-order codes | **exists:** supplied codes (four-arm arm B, #821) | T1 |
| C | **construction** (Open's runs, retained map and induction from L) | workers | the built structures, per block | **to build** | T2 |
| M | **MTF/RLE2** (undoing move-to-front and the zero runs) | workers | each block's L (the BWT column) | **to build** | T3 |
| R | **RLE1, CRC and output** | workers | each block's final bytes | **to build** | T6 |
| D | **delivery** (the in-order reader handing bytes to the caller) | the reader | a reader that takes and discards, with no copy | **to build** (a reader mode, not an oracle) | — |

**Not factors: the environment,** held fixed in every arm, so that it doesn't mix with the parts:
- **the clock:** the 100 ms pre-spin on every CPU before each decode, with the clock recorded (§81);
- **placement:** the scanner pinned to a quiet core, sibling idle (stage 2b, §78). An unpinned link arm is kept at one count, to carry the result back to production's regime;
- **memory contention:** none beyond the decoder's own. A dose arm can be added later as a factor of its own.

**Ordering.** The supply chain runs scanner → MTF/RLE2 → construction → emission → RLE1/CRC → reader. An ideal part upstream doesn't make a downstream part ideal: each downstream part still does its own work on the supplied input. An ideal part downstream does make the upstream part's work unnecessary *for that block's output*. So each oracle is defined so that the parts upstream of it still run in full, and only its own work is removed. The design checks this with each part's busy time in the ledgers.
