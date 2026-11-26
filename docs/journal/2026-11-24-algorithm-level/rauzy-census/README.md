# The Rauzy census: where construction's time goes, and whether a cheaper induction order would cut it

Thread 2 of [`../plan.md`](../plan.md). **Characterisation only:** no `src/` change, no production default.

## The question

Construction (C) is the decoder's largest room below the knee (`../ideal-parts/results/full/README.md`). Making it free takes 42% off at 1 worker, and half the 36% at 8 workers when the scanner is free too.

One of its three sub-phases, **A6b**, eliminates thick strips by an interval-exchange induction. That is Rauzy–Veech induction, the subject of the 10 and 12 November posts.

At 1 worker A6b takes **216 ms of 1,306** (`../math-targets/data/character-stage1/report.txt`). The order of the induction steps matters:

| order | steps per run | source |
|---|---|---|
| right end (what ships) | 1.812 | the companion post, §2.4 |
| best order found | 1.279 | MONODROMY, `../../monodromy/2026-11-10-rauzy-veech.md` §5.6 |
| proved floor, any order of these steps | 1.220 (r₂ − 1) | the same, §5.6 |

**What isn't known:**
1. how much of A6b's time is the steps themselves, and how much is setup and the dictionary;
2. whether fewer steps means less time, since a cheaper order might do more work per step;
3. whether a good order can be chosen *online*, from what's already decoded, cheaply.

## Why it matters

Suppose the steps were all of A6b and time followed steps. The known order would then save about 60 ms at 1 worker (5% of the wall), and less at 8 workers. That's the order of magnitude, and it could go either way:
- If the steps are a small share of A6b, this thread stops.
- If Zorich-style acceleration (batching repeated subtraction) or a two-sided order cuts much deeper, it grows.

A cheap answer now saves a 4–6k-line port of `surface_dynamics` (`../leads/research-software.md`, rank 3) that might buy nothing.

## The stop test (frozen before data, RULES C1a)

Stop this thread, writing the result up, if either holds:
- **the step share is too small:** (the steps' share of A6b) × (the best online order's step cut) < 3% of the 1-worker wall;
- **no online order:** no rule computable from the decode so far, at a cost below its saving, gets within half the way from 1.812 to 1.279.

Otherwise, the census's output is a ranked list of orders with their time and step counts. That list is the input to an engineering proposal, which is a separate decision.

## Files

- `prompt.md`: the Antigravity task.
- `results/`: once it exists.
