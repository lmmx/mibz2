# #635 fixed-cutoff review

**HOLD at `90f8b2c2` before queueing.** The geometric-mean test, cross-file admission and prospective S change address the earlier findings. The full schedule still misses the previously assigned requirement of power at least as high as G at a true +2% slowdown in every guard cell. The narrower 0.80 planning target needs to be reconciled explicitly with that requirement. #631's NOT ADOPTED stands. [Checkpoint](https://github.com/lmmx/mibz2/pull/162#issuecomment-7012815426) · [Original requirement](https://github.com/lmmx/mibz2/pull/145#issuecomment-7011091171) · [Later design decisions](https://github.com/lmmx/mibz2/pull/145#issuecomment-7012424046) · [Draft](https://github.com/lmmx/mibz2/blob/90f8b2c2210af374c0531a5cc287c186f0474c04/docs/journal/2026-11-19-open-port-2-registration.md)

## Fixed cutoffs and preserved admission

The scored quantity is `mean(log(port_wall/reference_wall))` across paired rounds. Its exponential is the geometric-mean ratio. The judge compares each mean to its frozen JSON cutoff with a strict greater-than test. No in-run variance enters that gate. The old spike witness has mean log ratio 0.02405 and paired t of 1.692. The maximum frozen cutoff is 0.0175, so that mean fails any cell. This verifies the specific reason for choosing fixed cutoffs. The 30 shipped checks pass independently, including the spike witness, the digest and exact-arm join regressions, and the 96-round S checks. [Scorer](https://github.com/lmmx/mibz2/blob/90f8b2c2210af374c0531a5cc287c186f0474c04/bench/open_port2.py) · [Schedule](https://github.com/lmmx/mibz2/blob/90f8b2c2210af374c0531a5cc287c186f0474c04/bench/open-port-2-schedule.json)

The join runs over the decision, sustained and all 18 scheduled extension files before scoring. It requires a single output size and effective digest per corpus and one exact arm per side. The old scorer and 18-input manifest are byte-identical to `4bb9e319`. P1, P2, H, ordinary-guard median RSS cap, the 12-round decision grid, worker/identity checks and fail-closed one-attempt marker are preserved. The old archive does not count as the successor's attempt, which uses `open-port-2/`. S retains its original absence of an RSS decision.

I re-hashed the old evidence bundle (`fb4771f5252a0d03c97b85d8b3103c7db605134075b86d2b7a45f22aef36a523`). I also retrieved and hashed the new DRY bundle (`c1f0ab76a588a7aa015c11bb49bd85b2b8f7cfd448406b1c8829b75bdd20792e`), verified preflight commit `7dbcf5f4`, all 18 manifest entries, all 16 output size/digest pairs, and the 30-check receipt. No timed JSONL files or retained run-queue measurements were present. [Old evidence](https://github.com/lmmx/mibz2/pull/145#issuecomment-7011043887) · [New preflight](https://github.com/lmmx/mibz2/blob/90f8b2c2210af374c0531a5cc287c186f0474c04/docs/journal/2026-11-19-open-port-2-registration/preflight/run-20261119T145510Z.tar.gz)

## Power and size under the locked rule

The pinned planner reproduced `plan.json` byte-for-byte at 100,000 replicates. It centers each cell's paired log residuals on the sample mean, draws rounds independently and uses the largest of three estimated null quantiles for (c_i). It sizes rounds for power under each cell's own residuals and uses the same draws and unrounded cutoff to calculate its reported rates. The scorer uses the rounded, serialized cutoff. The “any fails” estimate multiplies cell pass probabilities and therefore assumes independence **between** cells. A sum of valid marginal sizes would bound the family without that assumption. The individual sizes here are estimated under models. The memory allowance is also empirical and does not turn the combined 0.05 into a guarantee.

I drew 200,000 fresh replicates per cell and model at the locked (n,c), on the same archived residuals. This verifies the rounded rule under the pilot models. It does not validate the residual model on new hardware data.

| Model | Sum of estimated timing sizes | P(any), assuming independent cells | Lowest +2% power | Cells < 0.80 |
| --- | ---: | ---: | ---: | ---: |
| Cell's own, iid | 0.0115 | 0.0114 | 0.798 | 1 |
| Corpus pooled, iid | 0.0108 | 0.0107 | 0.711 | 9 |
| All guards pooled, iid | 0.0342 | 0.0336 | 0.664 | 12 |
| 2% spike stress, iid | 0.1546 | 0.1436 | 0.665 | 24 |
| Whole four-round order blocks, per cell | 0.0359 | 0.0354 | 0.737 | 4 |

The 0.798 estimate is near the 0.80 boundary. Material shortfalls occur in the other models. Calibration against the worst of three models for *size* does not give 0.80 power under all three. Under the explicit spike stress, the 0.05 family target fails in the independent-cell model even without the memory guard. The four-round-block row is a sensitivity check with just three archived blocks per ordinary cell and two for S. It illustrates the independence assumption's effect on power. It is not a precise bound for the production schedule.

### Why the proposed ordinary-guard schedule does not meet +2% parity

The committed table already has ten ordinary guards with lower own-cell power than G at +2%. Fresh draws at the actual locked cutoffs found the same ten, with examples below.

| Cell | Old G +2% | Locked G2 +2% |
| --- | ---: | ---: |
| Held-out Wikidata level 1, 10 threads | 1.000 | 0.886 |
| Held-out structured level 9, 20 threads | 0.999 | 0.904 |
| Held-out prose level 9, 10 threads | 1.000 | 0.924 |
| Random, 8 threads | 1.000 | 0.933 |
| Text, 20 threads | 0.876 | 0.838 |

For held-out Wikidata level 1 at 10 threads, G's null failure rate is only about **0.0021** in that model. Its higher +2% power there cannot be explained by a high false-fail rate. The new cutoff is more sensitive to a *fixed observed mean* above +2%, but the rules act on different sample sizes and the old rule could also reject a precise mean below that point. These differences are in detection of the same true mean. Dropping uniform dominance above +2% and stating the loss at +1% are correct. They do not resolve the shortfall at the originally required +2% point. The draft also places the lowest-power cells incorrectly. The committed table's minimum is onebyte 20t (0.8011), followed by held-out structured level 1 at 10t (0.8049), rather than the two held-out prose level 1 cells. [Planner](https://github.com/lmmx/mibz2/blob/90f8b2c2210af374c0531a5cc287c186f0474c04/docs/journal/2026-11-19-open-port-2-registration/plan.py) · [Committed outputs](https://github.com/lmmx/mibz2/blob/90f8b2c2210af374c0531a5cc287c186f0474c04/docs/journal/2026-11-19-open-port-2-registration/plan.json)

## Sustained ruling

**The S replacement is admissible as a separately specified prospective change.** Here the comparison supports it at the target: 96 back-to-back pairs with (c=0.0149) give 0.939 simulated power at +2%, versus 0.547 for the old eight-pair rule, and far fewer null failures in the own-cell model (old 0.149, zero hits in 200,000 new draws). I estimate +2% power at 0.841 in the all-guard pool and 0.825 in the 2% spike stress. The same change reduces own-cell +1% detection from 0.315 to 0.050. It improves modeled +2% detection, with the stated trade-off below that target. The eight old back-to-back pairs provide limited information about how a longer sequence behaves. This ruling does not claim uniform dominance or a workstation-wide error guarantee. Counting S as the 40th timing cell is coherent under the scoped model. Its absent memory gate remains absent.

## Workstation allocation and disposition

The locked schedule has 72–600 pairs per ordinary cell, 96 for S, and 4,980 extension pairs in 18 batches. The 9,960 extension decodes alone spend **5 h 32 m** at the two-second settle. The stated 5.81 h extension at 2.1 seconds per decode and roughly 6.9 h overall are plausible. The uncapped schedule was optimized for about 0.80 on the own-cell model. The added runtime does not establish parity or calibration outside that model.

Before queueing, either size and validate the locked scorer against the existing +2% parity objective on each guard, or explicitly obtain a prospective ruling on the weaker 0.80 criterion and its scope, with the +2% losses disclosed. Do not describe that change as preserving the old protection. State which null model is accepted for the 5% planning target and report the stress case separately. Keep the corrected estimand, joins and S specification. No successor data or workstation timings were generated in this review.
