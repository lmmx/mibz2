# Moonshot scorer: handoff

The state of `bench/moonshot-scorer-v2`. The design and the results are in
[README.md](README.md). #606 merged v0 (9c31461). v1 is PR #608, from this branch.
Only the Coordinator merges.

## Done

- **The candidate interface, the correctness gate and container timing,** as before: byte-exact
  against libbzip2, with block and stream CRCs; paired timing with a 95% interval and peak RSS.
- **The proxy, rebuilt for v1** (README §3):
  - *Each block in a worker's steady state:* the block decoded once unseen (no buffer growth in
    the region), then the warm block, which fills the caches, then the block measured.
  - *The sample:* stratified (233 full, 198 tiny blocks), with a paired bootstrap interval.
  - *Stores:* fast-string and non-temporal stores take no read for ownership.
  - *DRAM latency:* from each arm's own bandwidth (bench-imc-ten's queue, solved as a fixed
    point).
  - *The report:* drivers for each prediction, and range flags against the reference arms
    stored in `calibration.json`.
- **The diagnosis of v0's misses** (README, "What was wrong in v0"):
  - buffer growth measured inside the region;
  - a 10-full-block sample;
  - a cold cache;
  - one DRAM latency for every arm.
- **Calibration:** κ = 0 (at its bound), on opend 7/28. Held out: every cell on the right side of
  1, every cell inside its 95% interval, 9 of 9 pairs in the i9's order.
- **The i9 recipe** `bench-moonshot-calibrate`:
  - *Fixes:* DRY is the 5th positional argument and a misplaced `DRY=1` is refused; the counters
    are checked first; a short sweep still bundles.
  - *Counters added:* the L3-miss latency.
  - *The dry run* passed: the gate for all four arms, 2 rounds at 6 W, the bundle.

## Next

1. **The i9 run.** The Coordinator queues it and sets the tag. Every cell is held out against
   `calibration.json`'s `reference_predictions`, under the rule in the #145 LOG.
2. **The opend sweep (#604).** When its bundle lands, read it against the same predictions.
3. **The DRAM overprediction for the open kernels** (1.10–1.14 against 0.98–1.04). The suspect
   to test next is LRU against the i9's adaptive, scan-resistant L3 replacement.

## Commands

```sh
bash bench/moonshot/corpus.sh                   # once (needs lbzip2)
just moonshot-score path/to/candidate.rs --rounds 5
python3 bench/moonshot/moonshot.py calibrate --write          # about 100 min on 4 CPUs
python3 bench/moonshot/moonshot.py calibrate --again DIR      # re-read saved runs, no Valgrind
python3 bench/moonshot/test_moonshot.py
just bench-moonshot-calibrate 10 2 4 1,2,4,7,8,10 1           # dry run (DRY is 5th)
```
