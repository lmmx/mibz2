# The research method: divergent exploration, convergent verification

*20 November 2026. The Coordinator, at Louis's direction. This is the persistent methodology for
mathematical research on mibz2's inverse kernels. A session's opening prompt
([MASTER-PROMPT.md](MASTER-PROMPT.md)) sets the mission and points here, and this document governs
the method.*

## 1. The mandate: foundational change

The project's breakthroughs changed the **computational primitive**, not its implementation:
- **row-by-row LF traversal → Rauzy–Veech induction** on the interval exchange LF is (Cells);
- **whole-exchange induction → cutting the exchange open:** eliminate the thick strips and
  reconstruct first returns on a retained set (Open).

The programme's primary ambition is the next change of that scale: a mathematical formulation in
which a major source of paid work **becomes unnecessary**, not merely cheaper.

There are three levels of ambition:

| Level | What changes | Example |
|---|---|---|
| Foundational | the primitive or the reconstruction principle | LF traversal → Rauzy–Veech induction |
| Architectural | the representation or decomposition | Open's retained-row first returns |
| Incremental | how the same algorithm executes | packing queues, fusing loops, scheduling loads |

Foundational and architectural work should dominate the effort. Incremental work is legitimate when
it enables or tests a larger principle. It must not become the agenda: six incremental attempts on
Open all lost (CONTEXT.md §4).

**Radicality is not a score.** A formulation that sounds new but secretly needs an LF walk is not a
breakthrough. A modest-looking identity that removes a whole dependent computation may be one. The
measure is paid work removed.

**Mathematical maximalism** (Louis, 20 November 2026, after the claimed RH/GRH proof: "RH is proved
by using *everything*!"). Importing distant theory is now cheap, so the search uses everything.
- **No limit on mathematical distance or complexity.** A theory far from compression is not
  deprioritised for that reason.
- **Chains of worlds.** A representation's job may be to enable a move into another world, where a
  different theorem applies. A lift is not exhausted when its first operation fails. An
  obstruction may say which further world the construction needs, not that the lift is dead.
- **Auxiliary theorems count.** They need not improve a benchmark; each must contribute a precise
  capability to some route to exact reconstruction.
- **The asymmetry:** unlimited ambition in the mathematics that *discovers* a decoder, strict
  economy in the decoder that *executes*. A hundred-page argument for a handful of machine
  operations is a good trade. The cost of recognising when a specialised construction applies is
  part of the execution.
- **What does not relax:** the obligation register and native timing remain the final economic
  test, and every claim keeps its tag (proved, verified, measured, conjecture).

## 2. The cycle

Research runs in repeated cycles, not one shot. This keeps what made the Astra investigations
productive, moving freely between mathematical formulations and their engineering consequences, and
adds the cost discipline learned since.

| Phase | What happens | What it needs |
|---|---|---|
| **Explore** | Generate several **genuinely distinct** approaches: not parameter variations of one architecture | For each: the mechanism, the information it needs, and its most likely cost trap. No proofs or manifests yet. |
| **Triage** | Kill repetitions of rejected ideas, contradictions and obvious acquisition traps | Cheap reasoning, a small counterexample, a quick count on one block |
| **Deepen** | Take the strongest survivor (or two, one per track, §3) through the gates (§4) | Proof, manifest, cost model |
| **Confront** | Run the falsifier and the small experiments | Exact results against a fixed baseline, rule fixed first |
| **Learn** | Record what won or lost, and why, in [FRONTIER.md](FRONTIER.md) | The specific structural reason |

**Two rules for every Explore phase:**
1. Generate from mathematical structure, through any of these (tools, not quotas; unexpected
   ideas are welcome):
   - **Representation import:** what richer structure is this an instance of, and what operations,
     invariants or theorems does that make available? Cells (LF as an interval exchange) and Open
     (elimination as a stochastic complement) both came from here.
   - **Obstruction inversion:** take a failure's narrowest witness, and ask which assumption a
     different primitive could escape.
   - **Primitive ablation:** forbid a dominant operation in a bounded experiment, then check that
     the alternative doesn't pay the equivalent cost somewhere else.
   - **Small-instance exact discovery:** find optimal exact constructions on tiny cases, then look
     for the law behind them. Optimality holds only for the primitives and costs searched.

   Computational obligations (§3a) judge a candidate's economics. They do not limit what may be
   imagined.
2. Check FRONTIER.md and CONTEXT.md §4 first. A previously refuted approach comes back only with a
   new mechanism that addresses its recorded failure.

**After a failure,** the default is not a repair. Name the structural obstruction it exposed and ask
whether it points to a different primitive. A repair is acceptable when the failure was incidental
(a bug, a bad layout), not structural.

**The two failure modes to avoid:** endless brainstorming without testing, and spending a whole
session elaborating one idea that has lost its credible potential.

## 3. Two tracks

- **Foundational exploration:** high uncertainty, potentially transformative. Its evidence is
  theorems, counterexamples, small exhaustive checks and counts on real blocks. It does **not** need
  10% end-to-end headroom demonstrated to be worth pursuing: the mathematics that would make it
  affordable may not exist yet. It does need a stated route by which its information could become
  affordable, and a stated way it could be refuted.
- **Feasibility development:** a mathematically settled idea is taken through full acquisition and
  producer–consumer accounting, executable verification and, if it passes, a bounded native
  experiment run by the Coordinator.

The tracks feed each other. A feasibility failure is a foundational question ("what made this
unaffordable?"), and a foundational result becomes a feasibility candidate.

**Portfolio and scoring are different questions:**
- The gates (§4) answer: *is this proposal sound, affordable and ready for further investment?*
- The portfolio answers: *are we exploring diverse enough principles to have a real chance of
  something transformative?*

Review the portfolio in FRONTIER.md at every cycle. If every live candidate is incremental, or all are
variations of one primitive, the next Explore phase must widen.

## 3a. Two modes: reorientation and traction

A **computational obligation** is an expensive class of work the decoder pays. Obligations are
how candidates are *judged*: what work disappears, and what replaces it. Ideas are generated from
mathematical structure (§2). Each cycle declares one of two modes.

| | Reorientation (the default) | Traction |
|---|---|---|
| **What varies** | the factorisation: intermediate objects, primitives, even where inversion starts in the bzip2 pipeline | the implementation or architecture of a known mechanism |
| **Aim** | make an obligation unnecessary | make a known mechanism cheaper end to end |
| **First evidence** | the payoff of the capability, supplied: on real blocks through the shipped path, with the intermediate object the new mathematics would compute handed over (never the output itself); then a correct construction, and a cheap falsifier aimed at hidden acquisition | a measured bottleneck, a causal hypothesis, a discriminating experiment |
| **Failure signal** | hidden acquisition cost, incorrectness, or the displaced work reappearing | regression, or no end-to-end gain |
| **What a negative result is worth** | an obstruction or a new opportunity, recorded on the obligation | the next dominant cost, which feeds reorientation |

Move into traction when measured evidence or a surviving idea justifies it. A successful traction
step should report the newly dominant residual cost.

**Every serious candidate states four things in its write-up.** This is not a gate: it is there to
sharpen the reasoning.
- **Mode:** reorientation or traction.
- **Displaced obligation:** what existing work disappears. For traction, an implementation cost
  (queue fill, scattered writes) is enough.
- **Replacement principle:** what is done instead, and the mathematical fact that permits it.
- **New paid obligations:** what must now be acquired, built and consumed.
- **Nearest prior art:** what is genuinely different. Ancestry is context, not disqualification.

**Working backwards from the goal** (24 November 2026; FRONTIER.md, "The target"; the reasoning in
`docs/journal/2026-11-24-algorithm-level.md`). Reorientation works backwards from the target, as
the SSL-Coq project worked backwards from a successful game state:
1. **Supply the capability and measure the payoff first.** Name the intermediate object a new
   algorithm would compute (rows' output positions, the cycle's structure, coordinates on the
   interval exchange), hand it to the shipped kernel on real blocks, and measure what it buys. This
   bounds what any reframing that produces it can win, before any construction.
2. **Regress from the goal.** Ask what the decoder would have to know or compute one step earlier,
   and repeat. Each step ends **native** (computable from the run records within a stated budget),
   **missing** (a named research question) or **blocked** (by a named obstruction in FRONTIER.md).
3. **Discharge supplied conditions one at a time,** cheapest first, re-measuring after each. Stop at
   the first rung where the effect disappears, and record what distinguishes it from the rung below.
4. **Classify every fork** R0–R3 (FRONTIER.md); only R2 and R3 belong to the major shot, and their
   milestones are judged by premises discharged, not by speed.

**Result statuses** (adopted from SSL-Coq): every result is one of **a useful new lead**; **a
conditional effect with an unproduced prerequisite**; **a tested failure within a defined scope**;
or **an established route**. A supplied-state success is never quoted as a native one.

**At every cycle boundary,** first read what is new since the last one: published audits, and FRONTIER.md's pending mathematical leads. Audits correct the current work; leads open new directions. Then choose the next cycle. Then answer one question in FRONTIER.md's obligation register: *what did
this cycle teach about the underlying obligation?* Killing a candidate does not close its
obligation. Record any positive finding inside a negative result.

**The orientation check, at every cycle boundary.** Keep the steering apart from its effect.
1. The researcher records what it investigated, and the four fields above, for each serious
   candidate. It does not classify its own result.
2. **The Coordinator or MONODROMY** classifies each serious outcome, using the anonymised mechanism
   (is the old primitive hidden in a helper?), whether the old obstruction dissolves, and the
   measured cost proxies. It is classified on two independent axes:
   - **conceptual turn:** a different primitive or formulation, or the same one;
   - **economic turn:** an obligation removed without a comparable replacement cost, or not.

| | No cost gain | Cost gain |
|---|---|---|
| **Different primitive** | mathematics worth keeping (e.g. Half-Bipolar) | a candidate bouleversement |
| **Same primitive** | a negative engineering result | successful traction (e.g. Backward Open, if it wins natively) |

A claim of reorientation is not evidence of one. A new object built with the old traversal has not
removed the obligation. One cycle does not establish that a steering change worked. The outcome
goes in FRONTIER.md's steering log.

## 4. The gates (for promotion, not for imagining)

The gates regulate investment. They don't regulate creativity. An idea can be explored, discussed and
recorded without passing any of them.

| Gate | What a candidate provides | Needed before |
|---|---|---|
| G1 Possibility | The mechanism, a correctness proof (primary mark, periodic and multi-cycle cases), an exhaustive small-case verifier with its output | being called a result |
| G2 Availability | An acquisition manifest from the entry state I₀ that passes `docs/team/acquisition/check_manifest.py` | the feasibility track |
| G3 Producer–consumer cost | For the whole decoder: W, D, B, Q, M; what work disappears, what moves and where, what branches, boundaries or dependent chains the consumer gains; peak live state | any native code |
| G4 Headroom | The measured share of Open's time it attacks; the ceiling if that share went to zero; whether a perfect version beats Open by at least 10% end to end | a native prototype |
| G5 Paid-cost falsifier | One small executable experiment on the six blocks, aimed at the likeliest hidden cost, with its pass/fail rule fixed before it runs | a native prototype |

**The native prototype** is run by the Coordinator, under
[PERFORMANCE-POLICY.md](../PERFORMANCE-POLICY.md). A researcher never declares a candidate
production-ready.

## 5. The scorer

Claims meet executable checks, not reviews of prose:
1. **The researcher's own verifiers:**
   - exhaustive small cases: all binary columns to n ≈ 8 and ternary to n ≈ 5, at every primary
     mark, with periodic and multi-cycle cases;
   - byte-exact output on the six authenticated blocks against a dense LF walk;
   - report the case counts and the outputs.
2. **The manifest checker:** `check_manifest.py`. It fails:
   - unpaid edges;
   - `assumed` or `open` edges on the path to the output;
   - paths that hold a text-determining object (such as run labels) before the final edge.
3. **MONODROMY** independently audits theorems. **The Coordinator** reruns programs, measures
   native cost and decides what reaches engineering.

**A cost model is diagnostic until it has been shown to predict native results.** It can explain
traffic, but it doesn't predict competitiveness. The auditors check models against known native
variants (CONTEXT.md §5a lists the record so far).

The registry, `docs/team/acquisition/primitives.toml`, is a record of costed primitives, not a
whitelist. A new construction that exposes a cheaper primitive is welcome. Add it with its
derivation and cost.

## 6. Reporting

- **Tag every claim:**
  - **proved;**
  - **verified** (bounded domain, with the case count);
  - **measured** (with the program);
  - **conjecture.**
- **Keep three levels apart:**
  - **possible:** described and proved;
  - **feasible:** its information is obtainable and consumable at a checked cost;
  - **competitive:** a native implementation beats Open end to end.

  Almost every idea that disappointed passed the first and failed the second or third.
- **Don't:**
  - convert operation counts into speedup percentages;
  - read a structural lower bound as a time bound;
  - call anything optimal without naming the class it is optimal in.

## 6a. Records by the book

*24 November 2026, with PROOF-PROGRAMME.md §5.9. The form follows Bach, Black, Huiberts and
Kafer (arXiv 2510.21613), who fix a model's assumptions from the real implementation and real
inputs before any bound, and close with a postmortem. MONODROMY record 95 (#145, REVIEW
7099387596) is why: a synthetic pre-screen's latency was read onto a kernel bound by work.*

A record that models or predicts cost opens with **the book** and closes with **the postmortem**.
Keep both short.

**The book** (before any model or prediction):
1. **Input features relied on.** Each one, and whether it is measured: a link to the census
   output or the evidence it comes from (bundle and digest, file and line). No link means
   assumed, and says so.
2. **The real code path.** The shipped route the claim is about, at a commit, with its stage
   costs as measured (emit, construct, and so on, in ns a row at the worker counts judged).
3. **Assumptions knowingly unrealistic**, each with why it is tolerable here.

**The pre-screen checks** (record 95), before a synthetic pre-screen's ratio is read onto a real
kernel:
- **A. The m-slope test.** If the real loop's elasticity in m is a fraction φ of the synthetic's,
  at most φ of its time is the synthetic's bottleneck.
- **B. The latency-free ablation.** The real loop with its dependent loads made independent gives
  its work floor; a latency trick's ceiling is (real − floor) / real.
- **C. The footprint check.** Extra bytes a worker × W must fit in what L3 leaves at the judged
  worker count.

A synthetic pre-screen is valid only for the bottleneck it reproduces.

**The postmortem** (once the outcome is in):
1. **Input assumptions:** which features held on the real blocks, and which did not.
2. **Model assumptions:** which held, and which the outcome refuted.
3. **Method:** what the check that should have caught the error was, and whether it ran.
4. **Outcome against observation:** predicted against measured, figure by figure.

**Refining a parameter** (their §6.3). A parameter may change mid-argument only when it refines
the model of an assumption already in the book, and it is then checked on data. A parameter
that adds an assumption goes into the book first.

## 7. The lessons that cost the most

These are in [CONTEXT.md](CONTEXT.md) §4–5, with evidence. In brief:
- **elegant but unaffordable:** an output representation whose construction costs the inversion
  it replaces;
- **steps are not time:** dependent steps cut 3–7× while total time doubled;
- **cheap to make, dear to use:** a compressed intermediate that the consumer must unpack
  somewhere worse;
- **fusion can lose:** two simple loops fused into one complex loop ran slower;
- **the baseline already did it:** read the code before claiming it doesn't;
- **phantom stalls:** a stall assumed where the CPU already overlaps the loads;
- **noise:** 2-run medians on a shared machine.

## 8. Handoff: publishing a research branch

Committing and publishing are different acts.
- **While a cycle is active:** commit as often as is useful. Also push a work-in-progress commit,
  labelled `wip(<model>): …`, at least every 30 minutes of work and before any run likely to take more
  than a few minutes. A session can stop without warning (a quota or a crash), and unpushed work is
  then at risk. A WIP push is not a handoff and needs no write-up.
- **At a handoff, push the `research/<model>-<topic>` (for example `research/neon-…` or `research/carbon-…`) branch to origin.** A handoff is any of:
  - a research cycle finished;
  - an independent audit requested;
  - work handed back to the Coordinator.

  Pushing is authorised at these points without asking.
- **The handoff contains:** the cycle's write-up, negative results and failed proof attempts,
  reproducible verifiers and their outputs, cost manifests, and the updated FRONTIER.md.
- **Verify, then report:** run `git ls-remote origin research/<model>-<topic>` and report the branch
  and the exact pushed SHA. That is what lets another agent retrieve it. A handoff is not "remotely
  available" until that check shows the commit.
- **If a push fails** (permissions, connectivity), say so plainly, with the error. Don't describe
  the work as published.
- **Never:**
  - push to master;
  - force-push;
  - merge;
  - open pull requests;
  - change production code;
  - start long benchmarks.

  Each of these needs separate authorisation.
- **Publication is not approval.** A pushed result may be labelled promising, unverified or
  failed. MONODROMY and the Coordinator audit the exact pushed evidence. The push implies no
  endorsement.
