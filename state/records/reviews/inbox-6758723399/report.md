# Compact30 plain workers: pricing checkpoint

Inbox [6758723399](https://github.com/lmmx/mibz2/pull/160#issuecomment-6758723399), dispatch QUEUED6758603004. Governing spec a2da49638ac54d7763de1c167bd054f67ffdc763. Proposals and historical-data arithmetic only.

## Findings committed before completion

The register at 3a63542b2446071726bba30870e6af7012662d5b predicts about 2350 MB/s for seven independent compact30 workers. Replaying the archived 420 rows reproduces that arithmetic. It does not establish the candidate's speed.

1. Section B infers compact30 g≈1.18 from whole-decoder CI losses of 11.4–13.7%, assuming inversion fraction 0.7. This is not a measured i9 construction+walk ratio for compact30. The assumption includes construction only through an unattributed whole-decoder residual.
2. The production grid measures build+walk. Its inferred X therefore includes construction congestion; multiplying all of X by dependent-load fraction a discounts construction without justification. Separate build and walk costs before extrapolating a different representation.
3. The old grid harness at b99b92bcd89f43c22003018231e866f631ae57c3 has no compact30 arm. The compact30 implementation at ba94ff5ceda1b68a66bb2978af9674604569847d has no matching coop example. A common harness adapter is needed for a controlled inversion diagnostic. Existing kernel does not mean zero instrumentation work.
4. Prefer a separate diagnostic to expanding #326's G2/G9 workload comparison. Its current 61×12=732 slots would grow by305 for one five-concurrency compact30 grid, or610 for both levels, before adding the missing seven-worker controls.
5. D4-delta's registered filters stay unchanged. Instruction loads, dependent cache-line requests, misses and exposed latency must be distinguished.

Next: finish arithmetic with the archive's upper-middle median convention; publish the full construction/memory budget, common-harness diagnostic scope and whole-decoder decision.
