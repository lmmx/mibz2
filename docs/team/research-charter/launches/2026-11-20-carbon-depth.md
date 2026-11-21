# Launch record: Carbon Depth, 20 November 2026 (prepared)

- **Model:** Gemini Carbon, High, in a fresh Antigravity conversation and its own checkout.
- **Branch:** `research/carbon-depth`. WIP commits are labelled `wip(carbon-depth)`.
- **The flip:**
  - **The question it asks:** the other two lines ask "what faster decoder exists?". This one asks
    "what does exact reconstruction *cost*, in a model faithful to the machine?".
  - **How it works:** it works depth-first on one quantitative statement, either proving a lower
    bound or breaking it.
  - **Why this shape:** the claimed RH development reduced its target to one quantitative estimate
    (a growth bound on Galois cohomology) and then spent everything on that estimate.
- **Prepared by** the Coordinator at Louis's request. Paste everything inside the fence.

````text
# Carbon Depth — what does exact inverse-BWT reconstruction cost?

You are starting a third, independent research line for lmmx/mibz2. Two other Carbon sessions are
searching for faster decoders (research/carbon-word-transport: depth on mechanisms;
research/carbon-kitchen-sink: breadth across mathematics). Do not duplicate them, and do not touch
their branches.

Your line flips the question. Not "find a faster algorithm", but: **what is the true cost of exact
inverse-BWT reconstruction on a block, as a function of the block's own invariants, in a machine
model faithful enough that the answer predicts real decoders?** Then prove it, or break it.

Either outcome is a result. A lower bound that the shipped kernel (Open) meets tells us to stop
searching that direction and says precisely which assumption any faster method must violate. A
lower bound that Open misses by a wide margin tells us where the headroom is and roughly what shape
the faster method must have. A refutation of a plausible bound is itself the escape route.

## Why this shape

Levent Alpöge's claimed RH/GRH development with Claude (https://alpo.ge/remark.pdf; unverified,
treat as a provocation) is reported to reduce the whole problem to one quantitative estimate, a
growth bound on certain Galois cohomology groups, plus a covolume bound, and then to bring every
available theory to bear on that estimate alone, including a non-standard route through known
results. It began from a toy setting where the theorem was already known, re-proved it in a way
chosen to transfer, and replaced a missing ingredient (multiplicity) with a surrogate native to
the hard setting.

Do the same, in depth, on one statement:
1. **Fix the model before anything else.**
   - It must charge what real cores pay: dependent memory latency, with some memory-level
     parallelism (a few to a few dozen independent misses in flight); line transfers between a
     cache of size M and memory in lines of B bytes; and sequential bulk work at high throughput.
   - Pure work or depth models are known to be wrong for us. List ranking and integer linear
     systems are in NC, so depth alone allows polylog. The repository's measurements show that
     compact representations lose to Open because of dependent operations and cache misses,
     not operation counts.
   - Calibrate the model's constants against the native measurements on master: Open's phase
     times in `docs/journal/2026-11-20-open-u16q/` and `2026-11-20-open-stage-attribution.md`.
2. **Fix the invariants.** These are the block parameters the cost should depend on: n; r, the
   runs; the overlap structure (about 1.3 transitions per run, measured); the number and lengths
   of thin runs; and some measure of the block's entropy, for example the share of rows in
   one-row runs owned by unique random strings. Use measured values from the journals; don't
   re-measure what is already measured.
3. **State the target estimate** as one inequality relating cost to those invariants. Make it
   sharp enough to be false if Open is far from optimal.
4. **Find the toy where it is already true or false.**
   - *Candidates:* Sturmian and Christoffel words (every position computable, so cost about n
     sequential writes); de Bruijn words (arithmetic LF); random strings (where external-memory
     permutation bounds of the Aggarwal–Vitter kind apply); and the corrected damaged-toy family
     on master (`docs/journal/2026-11-20-toy-transfer.md`).
   - *What to establish:* where the estimate holds trivially, and what breaks it.
5. **Then bring everything to bear on the estimate alone:** cell-probe and communication
   complexity, external-memory lower bounds, information theory, ergodic theory of interval
   exchanges, algebraic complexity (the Kitchen Sink session has a unimodular integer system for
   orbit time: what does solving such systems cost in your model?), and anything else your
   judgement selects.

Prior art to check first, not to imitate:
- external-memory BWT inversion (Ferragina, Gagie and Manzini on lightweight indexing in external
  memory);
- list ranking and permuting lower bounds (Chiang et al.; Aggarwal–Vitter);
- run-length compressed indexes and move structures (Nishimoto–Tabei; Gagie–Navarro–Prezza);
- the repository's own obstructions: CONTEXT.md §4, FRONTIER.md's register, and the Carbon
  cycle 5 scoped GUID bound.

Several existing claims are scoped to representations, not to all methods. Part of your job is to
say which of them survive as genuine lower bounds in your model, and which do not.

## Standards

- **Tag every statement:** proved, verified (with case counts), measured (with program),
  conjecture.
- **A lower bound must state its model and its scope.** Say which algorithms it binds and which
  assumption an escape would have to drop. A bound with no stated scope is not established.
- **Verify small cases exhaustively** where the statement is finite. Formalise the central lemma in
  Lean if it helps you trust it, but don't make formalisation the work.
- **Experiments serve the theorem:** calibration of the model, and checking that the bound is not
  violated. No decoder engineering, no native benchmark campaigns, no production changes, PRs or
  merges.

## Environment

- **Setup:** start from the latest origin/master and report its SHA. Create research/carbon-depth
  in your own checkout.
- **Read:**
  - the charter (docs/team/research-charter/: METHOD.md with "Mathematical maximalism", CONTEXT.md,
    FRONTIER.md);
  - the native measurements above;
  - the published Carbon journals on master and on origin/research/carbon-word-transport, read-only.
- **The blocks:** authenticate the six blocks from data/admitted-bwt against manifest.json.
- **Checkpoints:** push wip(carbon-depth) at least every 30 minutes and before long computations,
  verifying with git ls-remote.
- **Handoffs:** each handoff is a journal docs/journal/YYYY-MM-DD-carbon-depth-<topic>.md, its
  programs, and a FRONTIER.md cycle-log row.

## The single question

Stay on it. When an attempt fails, the failure should sharpen the statement, the model or the
invariants, not move you to a different question. Continue autonomously until the statement is
proved, refuted, or reduced to a precisely stated open estimate with everything you tried recorded.
````
