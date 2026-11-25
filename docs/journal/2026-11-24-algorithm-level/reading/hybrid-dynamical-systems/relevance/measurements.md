# What to compute and measure

*The Coordinator's interpretation. Written 25 November 2026 and **revised the same day after ChatGPT's review** (`../../../external/chatgpt-hybrid-systems-review.md`, §5; journal §70). The revision changed the following:*
- *item 3 no longer claims a certificate;*
- *item 2's optimisation direction is fixed;*
- *item 4 no longer infers a remedy from divergence;*
- *item 5 is descriptive;*
- *item 6 is narrowed.*

*Each item uses stage 1's traces (`character/run-20261125T001135Z`) or stage 2's. Analyses are programs, written to files, not read by eye.*

## 1. Reconstruct the event and state semantics (keep, with the mapping corrected)

- **What:** from the per-block traces, rebuild:
  - the cumulative counts, by counting event types in causal order (the `q_*` columns are sequence positions, not counts);
  - ownership of each block by the scanner, a worker or the reader;
  - the ready set, with its identities and bytes.
- **Then:** specify the state between events, and the latent quantities (remaining work, contention). Only then check the transition model of `decoder-as-hybrid-system.md` against every decode: its guards, including admission checked before scanning, and its ownership.
- **Why:** every later item needs the semantics right.

## 2. Burst and rate frontiers per event type (keep as description; optimisation repaired)

- **What:** for each event type and a chosen rate $\rho$, the empirical burst requirement $b(\rho) = \sup_{s < t} [N(t) - N(s) - \rho (t - s)]_+$. Report the frontier $b(\rho)$ (or the least rate for a given burst budget).
- **Correction:** there is no "largest rate". A higher rate always satisfies the bound more easily.
- **Use with care:** an envelope fitted to finite traces is not a guarantee. Sizing a buffer also needs a downstream *service* curve: in network calculus, the backlog bound is the vertical gap between the arrival and service curves. Admission itself shapes the observed issue process, so a frontier fitted under the current window does not size a different window.

## 3. Lower bounds, predictors and certificates, kept apart (rewritten)

- **Lower bound:** $L = \max(S, W/k, R)$ per decode, from measured service. Report the wall's excess over $L$ per count. This is the existing efficiency figure, read the right way round.
- **Predictor:** a trace-driven simulation or a reduced model that predicts completion. Validate it on **held-out** rounds and on stage 2's **pulses**, without supplying their realised durations. A replay with each block's own durations only checks the bookkeeping.
- **Upper certificate:** a separate proof obligation (`decoder-as-hybrid-system.md`, "Three quantities"). The finite-state-graph construction is the concrete route, for a small family of policies.

## 4. Divergence between runs, as hypotheses to test (keep; no remedy inferred)

- **What:** align runs at the same count by their *causal* events.
- **Why not graphical closeness alone:** it matches states at the same jump index. If two independent workers finish in opposite orders, the intermediate ready states differ ((1,0) against (0,1)) even when the times are arbitrarily close, so commuting reorderings need their own alignment.
- **Output:** candidate locations where runs part, such as the scanner's fast or slow mode, as places to test with stage 2's pulse and placement interventions. Divergence does not identify a cause. The admission guard is an exact integer test under a lock, not a noisy threshold, so hysteresis is *not* indicated by it. Hysteresis is a candidate only for a future adaptive policy driven by noisy measurements.

## 5. Finite operating regimes and progress (replaces the invariance item)

- **What:** per count, describe:
  - conditional occupancy;
  - starvation episodes;
  - the head-of-line block's age;
  - the progress rate (blocks consumed per unit time);
  - recovery after stage 2's pulses.
- **Why:** this is useful engineering description in its own right. The steady middle of a decode keeps occupancy roughly constant *while work steadily falls*, so it is not "where a certificate stops falling". The book's invariance results need complete, precompact trajectories, which a finite run does not supply. Defer invariance until a bounded reduced model is justified.

## 6. Scaling, narrowly (narrowed)

- **What:** in a fixed event model, scale every duration by λ and check that completion scales by λ. Analyse the scanner/worker crossover directly as a max of two resource terms.
- **Corrections:**
  - "degree 0 in demand" was wrong: $T(\lambda w) = \lambda T(w)$ is degree 1 in workload, and the book's convention gives the residual-work dynamics degree −1;
  - the book's conical approximation concerns local stability near an equilibrium, not the curvature of a wall-time curve against worker count;
  - changing block sizes or the corpus changes routes, memory and caches as well, which is a much stronger experiment than duration scaling.

## Who takes what

- **MONODROMY (#40, question 6):** items 1, 3 (the predictor and the certificate route) and 5.
- **The Pathfinder (stage 2's analysis):** item 4, on stage 2's pulses.
- **The Coordinator:** item 2 and item 3's lower-bound table, as programs on stage 1's bundle.
