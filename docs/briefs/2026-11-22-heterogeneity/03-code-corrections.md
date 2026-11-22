# Transmission 2a: what the code says (corrections; these override 02 where they conflict)

The conditional-routes follow-up (PR #753, `docs/team/research-charter/PROOF-PROGRAMME.md` §5) checked
every shipped route in `src/` on master at 47b370190. Its findings change the thesis, and they
strengthen it.

## 1. Shipped admission conditions are cost gates, not correctness hypotheses

Every shipped route produces `walk`'s exact output on any column with a valid primary, and is tested
for it. The admission tests don't make a route correct. They decide when a route is worth running:
speed and memory. So the correctness claim for shipped routes is `∀x, F_i(x) = R(x)`, with no
hypothesis at all. P_i is the condition under which F_i is cheap.

This is the thesis made exact. **Correctness is uniform: every route is right on every block.
Cost is not: each route's gate is a hypothesis about cost, and a structural property of the data.**
Rewrite §3 on that basis:
- Don't present routes as "exact only under P_i". Present each as exact everywhere and fast under
  P_i.
- The hypotheses we hunt for are cost hypotheses. A structure in the data makes a construction
  cheap. Its absence makes the construction slow, never wrong.
- Research candidates can still carry correctness hypotheses. The Sturmian solver, for example,
  needs gcd conditions to apply at all. Keep that distinction where it arises.

The three responses in §4 still hold, restated for cost: weaken the cost hypothesis, certify it more
cheaply, or change the representation.

## 2. The shipped routes, with their gates (verified at file:line; see #753 for the full list)

- **Which kernel each decoder holds:**
  - `decode` and `decode_blocks` hold `Cells`.
  - `Plan::Workers(n)` holds `Cells` below 8 workers and `OpenCells` from 8.
  - Teams of 2–4 run `Cooperative<k>`.
- **Records against column:** chosen by `runs::takes_runs`.
- **`OpenCells` on records,** with admission by `cap`:
  - uncapped where `lazy_counts_admit` holds (20r < 3n and 64r ≥ n);
  - a dense band (64r ≥ n and 25r < 7n), capped at min(256,000, 0.3n) retained rows and 16,384
    thick strips;
  - refused otherwise, delegating to `Segmented` on the column route.
- **Inside Open:**
  - `try_run`, LIMIT = 16;
  - spiral contraction when the widest strip exceeds 32,768;
  - two-way emission when the codes fit and m ≥ 65,536;
  - `repeat` for a short marked cycle.
- **`Cells` and `LazyCells` density gates:** 10r ≥ n and 20r ≥ 3n respectively; the sparse gate is
  64r < n.
- **`Segmented`** is the total path: a segmented forward walk on the column route, and a segmented
  backward walk plus `repeat_cycle` on the records route. The plain `walk` runs only below 65,536
  rows. So R, the LF walk, is the mathematical reference, and `Segmented` equals it by the
  programme's lemmas T1c and T1d. It is not literally "the LF walk as fallback".
- **The gates are expressed in r/n,** runs per row, a structural statistic of the block. That is the
  data's heterogeneity in one number: the dense band, the lazy band and the sparse regime are
  regimes of r/n. Consider making §2's figure the distribution of r/n over Wikidata's blocks, with
  the routes' gates marked on it. That would be the heterogeneity and the decoder's response in one
  picture, if the records or a quick census support it.

## 3. The headroom oracle is broader than "(L, μ)", and its base isn't master

- **What it chooses between:** the fastest of 13 kernels per block. These are R5-Head's Open with
  μ = 2 at 8 limits, μ = 1 at L ∈ {8, 16, 32}, a plain forward walk and an 8-lane interleaved walk.
  On the 6 admitted blocks it chooses among 18.
- **Its base is R5-Head,** the construction stack that was not adopted on the i9, not master's
  `OpenCells`.
- **Headroom,** all on the research host:
  - 0.45% on the 6 admitted blocks (1.23% in the referee's rerun);
  - 3.17% on all 431 blocks.
- **The 198 small blocks** already reach `Segmented`, never Open, on master.
- **How to state it in the post:** "an oracle choosing the best of 13 kernel configurations per block,
  over a research build, finds 3.17%". It bounds that family only. Keep the post's conclusion: we
  don't know how complete the family of constructions is.

## 4. Smaller corrections

- **The 28.7% GUID figure** is a share of emission (S5) time on the six admitted blocks, not a share of
  bytes. Say "28.7% of emission time on the admitted blocks".
- **LazyCells' induction is unreachable inside `OpenCells`** on master: Open takes every block
  LazyCells would admit. Don't list LazyCells as a separate shipped route in the Open path.
- **Naming:** the research docs call the line "Neon's Kitchen Sink". The post doesn't need any agent
  or line names; say "our research search" or "the audit".
