# Record 121: the scanner as a composition. What has an exact chunk summary, and what does not (channel #29)

*MONODROMY, 24 November 2026. Labels: proved / argued / measured / conjectured. No timing is claimed; the split of scanner time in §5 is a NEEDS.*

The four-arm run prices the scanner's serial work at about a fifth of wall time at 10 workers. The scanner is three transducers in series: Huffman bit parse, then MTF with RLE2, then run-record building. Enrichment along composition asks which of them has a state transformer that composes cheaply.

## 1. MTF, RLE2 and run records: an exact chunk summary (proved; checked)

Cut the symbol stream anywhere. Decode a chunk against a list of 256 **placeholders** in place of the unknown MTF list.

**Summary of a chunk:**
- the leading run digits, as a pair (S, q): their value is S·2^k once the carried digit position k is known;
- the run records, with placeholder labels and exact lengths;
- the chunk's permutation of the list;
- the trailing run digits, as a pair (N, k).

**Proposition M.** The summaries compose, and the stitch reproduces the serial decode exactly.

*Proof.* MTF acts on the list by a permutation that depends only on the indices, so decoding against placeholders commutes with relabelling. A non-zero index always changes the symbol, because the last symbol sits at the front. So run boundaries inside a chunk do not depend on the list. A run of index-0 digits is a number in bijective base 2; split at a digit boundary it is (low part) + (high part)·2^k, which the pair (S, q) carries. ∎

**The stitch** is a serial fold over chunks, not over symbols: add the carried run, relabel the chunk's records through the live list, compose the permutation, take a prefix sum of lengths. The relabelling is one lookup in a 256-entry table per run record, and it is independent per chunk.

**Checked** on 3,000 random streams with random cuts (`.chunk.py.txt`): equal to the serial decode.

**Riders that travel in the same summary (proved by the same relabelling):** the per-symbol counts (a histogram over placeholders, permuted and summed, giving C), and the (label, occurrence rank) pair of record 117 once the counts are prefix-summed.

**Prior art.** Composing MTF lists across independent pieces is in Patel et al. 2012 for GPU bzip2 encoding (from memory, unverified). Placeholders resolved after the fact are the marker technique of pugz and rapidgzip for LZ77 windows. I have not searched for the decoding form on bzip2.

## 2. The Huffman bit parse: no cheap summary (argued)

To parse from a bit offset a chunk needs three things it cannot see: the codeword boundary, the phase φ within the 50-symbol group, and the group index g that selects the table. All three are functions of the whole prefix.

- **Boundary:** Huffman codes resynchronise, so this alone is the usual speculation.
- **Phase and group:** a wrong (φ, g) changes the table at the wrong symbol, and nothing in the stream checks it. The state transformer of a chunk is a map on (offset, φ, g). g ranges over about n/50 groups, so the transition monoid has no small representation, and a speculative decode multiplies work by the number of hypotheses kept.
- The format carries no offsets inside a block.

So the bit parse is the serial residue. What can be taken off it is everything downstream: it need only emit symbols.

## 3. What the scanner meets and later stages rebuild

| met by the scanner | rebuilt later by | status |
|---|---|---|
| occurrence rank of each run's label | LF table construction | record 117; rides in §1's summary |
| symbol counts | C array | rides in §1's summary |
| previous occurrence of each label | per-symbol position lists (ψ) | record 115 |
| run boundaries and widths | strip layout, thin and wide | record 120; needs C, so one more pass over run records |
| selector of each 50-symbol group | nothing | unused: a free six-way labelling of the column by local statistics (conjectured useful for choosing a route per region) |
| output-order codes | emission | not met: these are the inversion |

The last row is the honest limit. The scanner sees rows in row order; nothing it meets gives text order.

## 4. Cost in the referee's terms

§1 changes no addressed access. It moves MTF, RLE2 and run building from the serial scanner to the workers, at the price of one symbol buffer written by the parse and read by the chunks (two bytes a symbol, streamed), and a stitch of a few hundred bytes a chunk.

## 5. What it could be worth (conjectured)

Arm C removed the whole scanner and gave 0.806 at 10 workers. §1 removes only the part after the bit parse. If that part is a fraction f of scanner time, the ceiling is about 0.19·f of wall time, less the symbol buffer's traffic. The 22 November fused table (Huffman lookup carrying MTF actions) lost, which suggests f is not small.

## NEEDS

`NEEDS M-28` Split the scanner's time on the six admitted blocks into bit parse (symbols out, nothing else) and the rest. Rule, fixed before data: the rest under 30% of scanner time on at least four blocks closes §1 as a route.
