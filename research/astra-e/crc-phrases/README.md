# Selective CRC composition on the combined i9 writer

The measured source is 80d8409359214b4e5abfd4430213921e82e9cd30. The stock block finish first emits the complete decoded bytes and then computes CRC-32/BZIP2. At length 128 or more the detected x86 implementation folds four 128-bit lanes, eight carryless multiplications per 64 bytes. Short gaps use slicing by eight. On the complete local corpus the paid CRC-and-return stage of this writer took 16.29 ms against 856.51 ms total worker CPU, 1.90% at those medians. Removing this phase for free cannot supply the remaining 9.82–12.25% of target elapsed cost under that additive local accounting. Actual i9 parallel timing and overlap can differ. [Full local phase denominator](../budget/README.md) · [Target evidence](../workstation/combined-i9.md)

The earlier MONODROMY study already established word CRC as an affine action on a 32-bit register. Its complete grammar computed CRC on coded data and its separate proof composed the RLE1 state with checksums on decoded output. F5 and the combined word actions paid their parser preparation and retained the final contiguous CRC. The earlier fused-postwalk worker checked chunks as decoded output grew. This falsifier instead tests sparse reuse of checksums on actual, long, repeated decoded substrings separated by ordinary hardware-checksummed gaps. [Grammar proof at 7686bc3, §§3.4 and 8.6](../../../docs/journal/monodromy/2026-11-10-rauzy-veech.md) · [Word actions](../formulations/f5/README.md) · [Earlier chunk folding](../../../docs/journal/2026-11-05-fused-postwalk.md)

## Mark and count contract

The probe regenerates the pinned target-writer inputs and extracts its actual marked edge cycle and dictionary. It expands each cycle to the true row count, clips the final visit when needed, checks the entire coded inverse against Segmented, expands with stock RLE1, compares all emitted bytes to Python libbzip2, and verifies every stored block CRC. The 431 canonical blocks agree across 209,700,196 decoded bytes. The affordability study restricts itself to the 228 admissions actually priced by this constructor.

Streaming RLE1 emits each literal byte as it arrives; a count token expands to zero through 255 copies of the preceding byte. A word that includes no count token emits precisely its coded bytes regardless of the incoming count state. The experiment identifies count positions by a complete offline pass over the actual marked coded stream. A candidate interval containing a count token or a clipped last word is excluded. A change in final position from earlier edits is accounted for exactly. For every chosen interval the emitter's dictionary bytes are compared with the actual decoded output slice. The first safe visit stays in the ordinary gap. A digest shortcut begins only when that same dictionary ID recurs. There were 6,602 excluded full word visits containing count tokens, covering 1,545,152 coded bytes.

For u a prior CRC register and a safe decoded word W of length l, the new register is (u times x^(8l) mod P) XOR CRC_register(0,W). P = x^32 + 0x04c11db7. The hardware combiner multiplies and reduces with a degree-32 polynomial Barrett quotient. The initial register is all ones, the final CRC complements it. The concatenated hash covers final word and gap bytes in occurrence order. In the paid-descriptor arm the first recurring visit calculates and stores its pair. In the optimistic arm the trace, span starts/ends and every descriptor are available free. Both still read all ordinary gaps and perform every state composition. Output emission, selection, offset translation and storage of the span plan are excluded from the timed region in both arms. The benchmark is therefore favorable to this proposed change.

## Corpus and native screen

| Minimum repeated word length | Decoded bytes skipped in stock CRC | Reused word visits | Fresh digest input bytes | Ordinary gaps <128 / all gaps |
|---:|---:|---:|---:|---:|
| 32 | 71.68% | 1,140,703 | 8,117,772 | 792,648 / 870,942 |
| 128 | 48.83% | 370,215 | 5,412,215 | 123,309 / 340,159 |
| 512 | 7.62% | 22,806 | 832,552 | 549 / 23,001 |
| 2048 | 0.216% | 75 | 35,822 | 0 / 303 |

The initial 54-round nine-arm price uses the nine preexisting systematic admitted blocks and independently checks every output CRC. At the 128-byte threshold, the free-descriptor path took 2.800 [2.658,2.950] times the stock CRC, paid-descriptor cost 4.261 [4.032,4.502]. At 512, the corresponding ratios were 1.067 [1.006,1.131] and 1.270 [1.174,1.375]. At 2048 *none of the nine blocks had a selected span*. An apparent 0.926 ratio in the prepared arm there is entirely a difference between benchmark controls and cannot support phrase speedup. The source builds stock incremental advancement independently from the target revision. A same-code empty-plan control and enriched long-repeat population are required to adjudicate tiny threshold effects. [Coverage](coverage.txt) · [Initial native ratios](ratios.txt) · [Raw 4,374 timed samples](times.csv)

```sh
d=research/astra-e/crc-phrases
PAGE_DEPS=/path/to/release/deps bash "$d/run.sh" /tmp/astra-e-phrase-crc 4
python3 "$d/report.py" /tmp/astra-e-phrase-crc /tmp/astra-e-phrase-crc
```

The harness requires x86-64 with PCLMULQDQ, SSE4.1 and SSSE3 for this comparison; its input hashes and source revision are checked. The measured implementation is an experiment outside the shipping block decoder. It has no worker, CI or target-machine performance claim. The original trial uses a simple polynomial implementation to prepare length factors. Free descriptors bound the benefit of speeding up that preparation on this implementation.
