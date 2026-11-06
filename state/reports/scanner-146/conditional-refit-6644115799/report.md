# Conditional joint refit: what must change, and what is unidentified

Trigger: [6644115799](https://github.com/lmmx/mibz2/pull/145#issuecomment-6644115799).
Researcher, 2026-11-04. No new timing, implementation, harness change or authority.

**A joint fit does not identify R, loaded latency or issue cost. At the displayed point estimates, the transferred B2 issue/replay assumptions cannot be repaired merely by changing a common R or common latency. Candidate-specific latency, effective window, issue cost or omitted overhead can each fit. The first correction remains the kernel mapping.**

## Evidence boundaries that still matter

The machine/run correction is accepted. However, #157 remains a different kernel from B2 B: it writes each ordinary byte immediately and handles marked lanes after each round; B2 B buffers eight successive steps per lane and has tiled commits/replay. [Source comparison, per-copy predictions and both bench snapshots](https://github.com/lmmx/mibz2/blob/fb4c4027d4746e810b645ebf96361bfcd44ea06d/docs/journal/compact30-adversarial/scanner-146/copy-predictions-6643992749/report.md) are already published. Nine instructions does not identify B2's macro-op cost.

The 9V45 stage figures are **not lost**: the archived pre-update comment gives walk .942 [.847,1.049], displayed 3.89→3.63 c/B, and stage total .973 [.927,1.022]. These remain earlier-run evidence, not the 7763's measurements. Both reports' walk rows include assembly; B2 excludes it.

Thus the new numbers reject a naive transfer of B2's projected gain to #157; they are not B2 §8's matched-kernel falsifier. A model-range miss can reflect a wrong intervention mapping or calibration as well as a wrong mechanism. Even on the same processor model, B2's older fitted latency/workload cannot simply be reused.

## Explicit conditional fit to A and B

For illustration only, take the **rounded 7763 medians** W_A=4.04, W_B=3.78 c/B, common assembly a=.13, and counterfactually impose B2's Zen-3 costs:

    U_A=14.8, U_B=8..9, R=256, m=24, K=32
    I_A=14.8/6=2.466667, I_B=1.8
    q_X=max(U_X/R, 1/m, 1/K)
    W_X=a + I_X + ell_X*q_X + h_X

Set h_A=0 and h_B=0.05–0.17 c/B, B2's estimated replay range; assume other overhead equal/absent. These are sensitivity assumptions, **not a claim that #157 replays tiles**. They intentionally expose the consequences of transferring the old ledger.

Then q_A=.0578125, q_B=1/24, and fitting A gives ell_A=24.9658. The following alternatives each fit B's displayed 3.78 exactly:

| Change allowed for B; others held as above | Required value |
|---|---|
| Loaded latency only | ell_B=40.32..43.20, or 1.615..1.730 times ell_A |
| Effective window only, ell_B=ell_A | R_B=110.96..133.75 effective slots, depending on U_B and replay |
| Issue cost only, common ell and R | I_B=2.43976..2.55976 c/B instead of 1.8 |
| Extra B overhead only, common ell/R and original issue | +0.63976–0.75976 c/B beyond the assumed replay |

This demonstrates non-identifiability constructively. The fitted ell values are in the report's clock-scaled units; they are not measured request latency. An arm-specific effective R is a parametrisation of exposed useful work, **not evidence that the physical ROB shrank**. An additive overhead can absorb tails, scheduling or model error without naming a microarchitectural cause.

Nor are these confidence intervals. They vary the assumed U/replay, not timing uncertainty. The .925 paired geometric-mean ratio must not be combined with one median to manufacture a paired B observation. Refit raw paired, boundary-matched observations for statistical inference; this calculation answers which assumptions could move.

## Why a common smaller R does not repair this point fit

With common R, m, K and ell, U_A>U_B implies q_A>=q_B. Under the transferred issue/replay assumptions:

    W_A − W_B
      = (I_A−I_B) + ell*(q_A−q_B) − h_B
      >= .666667 − h_B
      >= .496667 c/B.

The displayed difference is only .26. Therefore **no common R or common ell** fits that pair while all those assumptions stand. Reducing a common R makes the window penalty no smaller for A than B; it does not selectively remove B's benefit. A B-specific effective R or B-specific loaded latency can do so, as the table demonstrates.

This is a point-estimate algebraic inconsistency of the transferred assumptions, not a 95% rejection: the paired timing interval is broad, and the displayed medians are not paired observations. In particular, the upper improvement compatible with the reported ratio can exceed the .4967 bound. Unknown actual issue/port costs and whole-block overhead already invalidate treating this as a definitive hardware verdict.

Risk 5 is **loaded latency increasing under additional requests**, not a synonymous statement that R shrinks. Those are observationally interchangeable fitting parameters here, but different physical hypotheses.

## Consequences for the proposed closure and search

- A confidence interval reaching .95 does not satisfy “close if B/A>=.95.” It means the threshold is compatible with the data. The .925 [.856,.999] interval also permits a substantial benefit; no precise family closure follows, before even considering the kernel mismatch.
- Under a pure shared miss-service cap, fewer **demanding requests per useful byte** is one route. Shorter service time, improved cache residency/prefetch usefulness, reduced competing writes, or different concurrency can also relieve it. “Only fewer misses help” is too narrow. Fewer allocated table lines need not mean fewer misses.
- The replicate's pattern regression and the end-to-end gate are independent of this model fitting. A “Better” resource summary is not a universal no-regression certificate; no merge decision is granted or overturned here.

**Answer to “which parameter must move?”** At least one transferred assumption about the actual kernel's issue/replay/overhead, arm symmetry, or the additive model must change to match the displayed point pair. The current evidence does not select one. The most directly established error is the kernel/cost mapping; correct it before interpreting an effective R or ell as hardware. Keep the published per-copy discriminators rather than reporting a unique fit.

[Recomputed arithmetic](audit.json). Primary model: [B2 §§5–8 at 9723f068](https://github.com/lmmx/mibz2/blob/9723f06847b88a10ba93d07c3f35680e58766015/docs/journal/2026-10-30-b2-gathered-walk.md).
