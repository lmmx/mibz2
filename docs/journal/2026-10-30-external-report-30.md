# External research report 30: the researcher's P74 status (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 29 (`2026-10-30-external-report-29.md`).

**What it contains:** P74's status after rounds 25–39.
- **Established:** correctness. The status takes in the portfolio's measured path shares,
  fallback traffic, shipped machine code and call densities.
- **Modelled, not measured:** the gain, 4.6–5.8% of a one-core decode.
- **What remains:** a compiled candidate and its disassembly, a complete-loop
  measurement, negative controls, and four-worker validation.

**The coordinator's notes:** `2026-10-30-research-portfolio.md` §3.2e, after (k).

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (card and figure labels flattened) and is kept as
received.

## The status, as received

````text
P74 progress status — 30 October 2026

P74 has passed its algorithmic correctness tests. Its remaining obstacle is proving a CPU-performance improvement in the complete decoder.

Current assessment

Correctness established

Register-held AVX2 move-to-front decoder

Algorithm and invariants

Synthetic and real-block verification

Instruction-level design

Compiled P74 code generation

Full-decoder speedup

What we’ve established

Area	Result
State representation	32-byte register head + memory tail
Common case	84.8% of explicit Wikidata tokens need only one AVX2 shuffle
Cross-lane case	13.6% use a multi-instruction register permutation
High-rank fallback	1.6% use the memory tail
Symbol extraction	The updated register’s byte zero is exactly the selected MTF symbol
Correctness	Synthetic adversarial streams and 20 real blocks passed the reported checker
Checkpoints	Compatible with existing 16 KiB count checkpoints
Short alphabets	Correct for all alphabet sizes, including 1–32 symbols

The remaining 1.6% fallback is also bounded by Wikidata’s observed alphabet sizes. Its tail moves at most 139 logical bytes per event, excluding register preservation and other instruction overhead.

Important update from Claude’s latest report

The portfolio now contains a more advanced reconciliation of rounds 25–37, including shipped-wheel disassembly and actual call-density measurements.

That changes several earlier estimates:

* Verified path shares (20 real blocks): 84.6% one-shuffle, 13.8% cross-lane, 1.59% fallback.
* Actual high-rank tail traffic: 0.040 logical bytes per output byte, substantially below our conservative 0.136 B/B bound.
* Shipped machine code: confirms the overlapping stack-resident MTF-list loads and stores that motivate P74.
* Library-call frequency: approximately 329 calls per 1,000 explicit tokens; roughly 310 are RLE2 memset calls.
* Short-run optimization: inlining runs of at most 31 bytes is modeled to reduce potential call boundaries to approximately 49 per 1,000 explicit tokens.

The literature instruction tables also support a one-cycle state-update dependency for the common one-shuffle path, versus a modeled 15–17-cycle dependency in the shipped memory-resident loop.

These are meaningful advances beyond correctness. They make P74’s mechanism much more credible.

Performance outlook

Modeled potential full-decoder gain · one core

4.6–5.8%

Based on an estimated 0.6–0.75 cycles saved per decoded byte, before proving the candidate’s compiled behavior and end-to-end performance.

This is a model, not a measured speedup. The actual gain might be smaller, absent, or negative if dispatch, register pressure or compiler scheduling erase the theoretical advantage.

What’s left before P74 can be called a win?

* Compiled candidate: verify that the persistent YMM head survives ordinary tokens without repeated spills, and that the one-shuffle path is actually emitted.
* Complete-loop CPU measurement: compare P74 against shipped 0.2.1, including RLE2 and checkpoints—not just synthetic shuffles.
* Negative controls: confirm no unacceptable regression on random and records inputs, particularly given #25’s previous 11% random-data regression.
* Four-worker validation: demonstrate improved full-decoder throughput without violating CPU-seconds/GB or peak-RSS constraints.

My assessment: P74 is now a technically credible optimization candidate with an established correctness proof, independent instruction evidence and a reasonably favorable cost model. It is not yet a performance win.

The next valuable step is a read-only compiled-code audit of a P74 candidate, once one exists. There is little remaining value in further algebra on the already-proved shuffle state machine.
````
