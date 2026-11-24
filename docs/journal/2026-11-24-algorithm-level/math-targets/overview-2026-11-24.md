# What the mathematics is for: the targets, with their measured ceilings

*Established 24 November 2026 at Louis's request: "establish what exactly we can find new maths for — my sense is that the induction part is the … factory er, not the doorman the other one". It rests on the four-arm oracle (journal §44; LOG 7144221012), the supplied-codes oracle (§30), and the four-arm registration at 5731db19.*

## Who does what in the decoder

| Stage | Where it runs | What it does |
|---|---|---|
| **The scanner ("the doorman")** | one thread, serial | reads the compressed bits and decodes the Huffman codes |
| **The workers ("the assemblers")** | k threads in parallel | MTF/RLE2 → building the inversion structures (Open: runs, the retained map, the Rauzy–Veech induction) → emission of the text → RLE1 and CRC |

**Louis is right.** The induction, the inversion this programme is about, runs in the **workers**. The "lone rows", "moving together", contraction, enrichment and closure are all mathematics of the workers' inversion: its construction and its emission.

## What each target is worth, measured by oracle

The table gives whole-decoder wall time against the shipped decoder; lower is better. These are ceilings: the oracle supplies the thing for free.

| Target | Oracle | 1 worker | 6 workers | 10 workers |
|---|---|---|---|---|
| **Inversion emission** (output-order codes supplied) | B | **0.830** | **0.850** | 0.976 alone; **0.920** with the scanner removed (D/C) |
| **The scanner's Huffman decode** | C | 0.996 | 0.982 | **0.806** |
| both | D | 0.825 | 0.830 | **0.741** |

**What this means for new mathematics:**
1. **The inversion's emission is worth 15–17% at 1–6 workers,** and about 8% at 10 once the scanner stops masking it. Mathematics that delivers output-order codes cheaply targets exactly this. The obstacle is **acquisition**: building the structures costs 1–4 reads per lone row on the real blocks (§43). It is also **closure**, keeping the composed representation cheap (§42).
2. **Construction is not separately measured.** The workers' MTF/RLE2 and the building of Open's structures stayed inside the clock in every arm, so their share is unknown. A further oracle should split worker time into MTF/RLE2, construction and emission, giving each mathematical target its own ceiling. Queued as the next measurement.
3. **The scanner is mostly an engineering target:** parallelising, gating and queueing (`SCANNER-BRIEF.md`). Some mathematics applies there too: Huffman self-synchronisation, speculative decoding at unknown bit offsets, and table-switch structure. But its 19% at 10 workers is the first thing to bank by engineering.
4. **Cross-stage enrichment** is the one mathematical question that spans both: what does the Huffman scan already see that the workers' MTF, construction or emission recompute?

## The standing order of mathematical work

1. Acquisition and closure for the inversion (MONODROMY, Neon), aimed at the emission ceiling above.
2. Cross-stage enrichment, scanner to workers.
3. The scanner's own mathematics, only after engineering has banked what it can.
