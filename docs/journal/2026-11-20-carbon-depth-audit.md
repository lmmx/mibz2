# Audit of Carbon Depth's handoff (first pass)

*20 November 2026. The Coordinator, on `research/carbon-depth` at e4cef682. This first pass covers
the cost model and the bounds' constants. MONODROMY is to check the combinatorial theorems.*

## The blind test cannot be run as posed

The plan was to test HBMC on a result it had not seen: #666, where natively the 16-wide queue
wins and the LRB front-end fusion loses by 20–36 ms over the corpus.

HBMC cannot price that. `model_open_qi16` (`calibrate_and_retrodict.py`) prices each of Open's
stages:
- **What a stage's price is built from:** a hand-chosen count of operations per element
  (42 per record, 28 per thin run, 135 per induction step, 16 per spelled byte, 5.5 per wired
  row, 12 per emitted step), times machine constants (`tau_op_ns`, `tau_br_ns`, latencies by
  working-set size).
- **What it lacks:** any term for the MTF/RLE2 front end, so the fusion's added cost is outside
  its scope. Pricing the fusion would mean choosing new per-element counts, which is fitting, not
  prediction.

**What this means for the 13/13 retrodiction:** it is agreement within a model whose stage
constants were chosen with those variants in view. It is not yet evidence of predictive power.
A fair blind test needs the model frozen, then a variant it has not seen, expressed only in
existing terms.

## The millisecond lower bounds depend on asserted constants

`lower_bounds_ms` evaluates the Tier B ("semigroup-faithful") floor. Its per-row and per-strip
costs are asserted minima, not derived ones:
- "at least 10 ALU/address ops per retained element";
- Chiang list ranking at 4 × (16 B + 12 ops) per element;
- "minimum 48 B per strip";
- 0.21 × h chord-copy reads;
- a front concurrency of at most 5.

The combinatorial statements may be proved (for example Theorem 1, |S| + H(S) ≥ r, and the
non-coalescence of one-row runs). The 0.29–0.59 ms floor, and so the "Open sits 4.5–6.1× above
it" headroom, are model estimates under those assumptions. They are not lower bounds in a
standard model.

**The unconditional floor** (0.083–0.118 ms) is essentially the time to write n output bytes
sequentially plus read the input. That is a valid floor, and an uninformative one.

## Status

| claim | status after this pass |
|---|---|
| Combinatorial theorems (1, 2, 4, 6, 7; Lemma 3) | not yet checked; for MONODROMY |
| HBMC 13/13 retrodiction | retrodiction with fitted stage constants; no blind test yet possible |
| Tier B floor 0.29–0.59 ms | a model estimate under asserted per-element minima |
| Unconditional floor | valid, and close to the cost of writing the output |
| Theorem 5 (coded rotations: about p runs, so widths grow with n) | plausible; the blog's damaged-toy separation holds only while p is comparable to n, where the cut points cost about as much as the run list. To be read before correcting the post |
