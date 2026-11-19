# F5. Reconstruct decoded bytes from RLE1 word actions

The reconstructed object consists of dictionary-word transducers and the marked sequence of dictionary references and literal bytes. On the admitted run route, reusable word interiors can be copied into the caller's decoded output with RLE1 already undone. The scheme removes reconstruction of the complete RLE1-coded block and its subsequent scan/copy. It retains the endpoint constructor, residual ordering, final decoded writes, block CRC and combined stream CRC. The earlier streaming variant stages bounded pieces of the encoded sequence and still runs the ordinary parser on those pieces.

## Incoming-state proof and cuts

Emit every literal as soon as it arrives. The streaming state consists of the last literal byte v and its current consecutive count k in 1 through 4, or an empty state with k=0. At k=4, the next input byte b appends b further copies of v and resets to empty. At k<4, b is a literal and its new count is k+1 when the previous byte matches or one otherwise. This counts runs in the same groups of four literals and one count byte as the repository parser. Only k=4 at actual block end is an unfinished run. Count bytes that happen to equal the run byte follow the same rule.

A cut immediately before w[c] is safe when c≥4, w[c-1] differs from w[c], and the preceding four input symbols are not all equal. Pending count would require four immediately preceding equal literal inputs. Every other incoming state at this cut is empty or has w[c-1] as its last byte. Reading w[c] consequently emits the same literal and reaches the same state as reading it from empty. The exact incoming states at the cut itself can still differ. For two safe cuts, the middle can therefore be decoded once from empty. Each occurrence processes the prefix as delivered, copies this decoded middle and processes its suffix starting from empty at the second cut.

One cut also suffices. Predecode the word from empty and record its terminal state, including a pending count. To find the earliest reusable suffix, after the first input token represent possible k values by a five-bit mask. A repeat rotates the counts 0,1,2,3,4 to 1,2,3,4,0; a change maps k<4 to one and k=4 to zero. With a singleton mask every possible entry and the empty entry now have identical states. On a change with no possible k=4 the next symbol is independent of entry. Either permits reusing the remaining output from the empty-entry decode and adopting its terminal state. Until this happens the prefix runs under the actual state, including the incoming run byte if the first token is a count. Words such as one repeated symbol can have no reusable suffix. As a guard, a word beginning with a different byte from the previous nonpending byte, or with an empty incoming state, can reuse its entire empty-entry result. If the marked cycle repeats to reach exactly n, the transducer state continues across repetitions. A clipped final word is parsed as its actual prefix. CRC is over the resulting final bytes.

## Pinned verification

```sh
d=research/astra-e/formulations/f5
python3 "$d/extract.py" corpora/wikidata.bz2 /tmp/f5-tapes > /tmp/f5-inputs.csv
python3 "$d/screen.py" small
python3 "$d/screen.py" real --corpus corpora/wikidata.bz2 --inputs /tmp/f5-inputs.csv --tapes /tmp/f5-tapes
```

The extractor pins the SHA-256 of the reference queued Open-Return implementation at `9c31461` and authenticates the established corpus. It captures actual marked words on every 24th large block, including one forced outside route admission. The verifier rewraps each original bit interval with its own CRC as a one-block stream and compares every byte from all three word-action variants with Python's libbzip2. `inputs.csv`, `blocks.csv` and `small.txt` are raw receipts. Independent tests compare 13,516 full incoming-state / word cases (including foreign pending-count symbols) and 276 truncated-period cases against token parsing and its final error. `preregister.md` specifies the screen before its run.

| Cost / coverage | Frozen | Nine admitted samples combined |
|---|---:|---:|
| RLE1-coded bytes | 899,931 | 8,099,915 |
| Decoded bytes checked against libbzip2 | 900,000 | 8,099,814 |
| Two-cut reusable input bytes | 759,282 | 6,573,300 |
| One-cut reusable input bytes | 794,307 | 6,963,408 |
| Guard plus one-cut reusable input bytes | 825,219 | 7,316,963 |
| Dictionary visits | 15,278 | 175,408 |
| Whole accepted / partial suffix / fully scalar visits | 15,220 / 30 / 28 | 174,501 / 280 / 627 |
| Runtime phrase-prefix or fallback bytes | 116 | 1,814 |
| Residual literal bytes | 74,596 | 781,138 |
| Literal stretches (of them length ≤3) | 7,374 (3,293) | 86,971 (43,176) |
| Old AVX2 detection windows | 28,136 | 253,249 |
| Old RLE output spans, excluding scalar literal tail pushes | 28,182 | 253,673 |
| Old count tokens | 34 | 363 |
| Compiled distinct word bytes | 52,035 | 462,345 |
| Fresh-entry decoded words differing from original / their total decoded bytes | 3 / 25 | 13 / 2,107 |

The guarded object bypasses per-occurrence RLE classification for 90.33% of input bytes on the admitted sample. Its writer performs 174,781 full or suffix copies, plus scalar actions for residual literals and the remaining prefixes and words. Reference Open-Return already emits 175,408 dictionary copies and 781,138 literal pushes to its intermediate buffer. The reference then traverses the contiguous RLE input with its vectorized parser and writes the final output. The fused path adds state updates to every residual literal and pays dictionary inspection, descriptor accesses and predecoding before output. Literal stretches may be interrupted by nonempty word copies and cannot be treated as independent fresh parses. The spans counted above are source-level operations, not instructions. The structural win does not settle wall time.

Fresh-entry decoded words identical to their original spellings can borrow that storage. Huge expansions are possible on other inputs, and preparing all expanded words eagerly could grow memory much more than observed here. A benchmark candidate should bound its extra decoded dictionary and fall back to a streaming action when the bound is exceeded. No allocator policy change is required. The ordinary final-output buffer and CRC remain necessary.
