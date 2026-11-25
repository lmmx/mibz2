# Candidate uses, chapter by chapter

*Moved here from the précis on 25 November 2026, after ChatGPT's review, so that no application hypothesis sits inside a summary of the book. Each was written by the chapter's reader as speculation; the Coordinator's corrections are marked. The judged version of these ideas is `measurements.md` and `levers.md`.*

## Chapter 1: Introduction (pp. 1–23)

*From `../precis/ch01-introduction.md`.*

*Speculative; for the team lead to evaluate.*

- The scanner–workers–reader pipeline can be written in exactly this format: continuous state is elapsed time and per-worker progress timers (flow), and each traced event (issue, enqueue, start, ready, taken, consumed) is a jump with its own guard (a subset of $D$) and reset.
- The admission rule "issued − taken < W" is naturally a guard on the issue jump; the counts themselves are discrete modes that only change at jumps, like $q$ in Section 1.4.1.
- Several events stamped with the same time map to consecutive jumps, as in the Newton's cradle and quantiser examples; the model has room for them without inventing artificial time offsets.
- The hysteresis device of Example 1.7 (switching between two regimes with separated switching sets) is a template for describing which resource is the bottleneck either side of roughly 8 workers without chattering between regimes in the model.
- The impulsive-equation discussion (1.4.3) suggests modelling fixed-schedule events with count-down timers rather than exact time sets, if the model is later used for robustness arguments.

## Chapter 2: The solution concept (pp. 25–42)

*From `../precis/ch02-solution-concept.md`.*

*Speculative; for the team lead to evaluate.*

- A per-block trace with timestamps and causal sequence numbers is essentially a hybrid time domain: the timestamp is $t$ and the global sequence number is $j$; events sharing a timestamp are zero-length steps. The lexicographic order on $(t, j)$ matches the trace's causal order within one run.
- The average dwell-time generator (2.9) is a token bucket; checking each trace against $j - i \le (t-s)/\tau_D + N_\circ$ for each event type would give empirical burst and rate bounds (e.g. issues per unit time) in a form the stability results can use.
- The three ways a maximal solution can end (Proposition 2.10) give a checklist for a pipeline model: does it run forever, stall because no flow or event is allowed (deadlock, e.g. window full and no worker ready), or step outside its invariant set (a conservation law violated)?
- Proposition 2.11's three conditions are a way to state when the model is deterministic: no simultaneous choice between "wait" and "fire", and no choice of which event fires. Where the real scheduler has such choices, set-valued $G$ represents them honestly.
- Zeno-type behaviour (many jumps with little elapsed time) is the formal name for event bursts; whether the model admits it is decidable from the data.

## Chapter 3: Uniform asymptotic stability, an initial treatment (pp. 43–71)

*From `../precis/ch03-uniform-asymptotic-stability.md`.*

*Speculative; for the team lead to evaluate.*

- Steady-state operation of the decoder is naturally a closed set, not a point: for example the set where the window occupancy, queue lengths and worker phases lie within fixed bounds, with clocks free. UGpAS of such a set is the formal statement that the pipeline settles after warm-up, uniformly over starting backlogs.
- The $\mathcal{KL}$ bound of Theorem 3.40 in $t + j$ suggests measuring convergence from the traces against both wall time and event count; event count is the right clock in burst phases (many events at little time).
- Proposition 3.24 fits a potential function such as outstanding work (issued but not consumed, weighted by remaining work) that cannot increase between events but drops at each consumed event, provided consumed events occur persistently; the exact conservation laws could supply such a function and its bounds.
- Proposition 3.29 is a template for a potential that rises at issue events and falls at completion events, stable provided the trace domains satisfy a balance inequality, which could be checked directly on the recorded event counts.
- Examples 3.34–3.36 are a warning relevant to the throughput switch near 8 workers: two regimes that are each well behaved can combine badly, so analysing each bottleneck regime separately does not prove the switching pipeline is well behaved.

## Chapter 4: Perturbations and generalized solutions (pp. 73–95)

*From `../precis/ch04-perturbations-generalized-solutions.md`.*

*Speculative; for the team lead to assess.*

- The admission rule $\text{issued}-\text{taken}<W$ and the in-order reader are "thin" guards in the sense of Example 4.7: a timing perturbation at the instant a guard is reached can change whether an event fires then or later. Comparing the exact trace model with its regularised version (guards treated as closed, simultaneous events allowed in either order) could enumerate which orderings small timing jitter can produce.
- Hermes ≈ Krasovskii suggests a test: event orders observed under jitter should all be solutions of the regularised model; an observed order outside it would indicate a modelling error or a missed causal edge in the trace.
- Near the ~8-worker switch in the limiting resource, the effective throughput may behave like the convexified flow of Example 4.1: rapid alternation between two bottleneck regimes averaging to a rate in between. Long-run rates measured there might be predicted as convex combinations of the two regimes' rates, with the weight being the fraction of time each binds.
- Hysteresis (Example 4.19) and side-memory (Example 4.18) suggest a design check: any scheduling decision taken at an equality boundary (e.g. exactly $W$ outstanding) is a candidate for fragility; a decision with memory or slack is robust by construction.
- Proposition 4.10's overlap condition may be a useful lens on stalls: a state where neither "admit" nor "wait" is enabled under a small timing shift corresponds to nonexistence of a perturbed solution.

## Chapter 5: Preliminaries from set-valued analysis (pp. 97–115)

*From `../precis/ch05-set-valued-analysis.md`.*

*Speculative; for the team lead to assess.*

- Graphical convergence and $(\tau,\varepsilon)$-closeness give a principled distance between two event traces whose events shift slightly in time while keeping the same order index, e.g. comparing a run with a perturbed rerun, or a simulator against a measured trace, without being thrown by a single reordered or shifted event.
- The inner/outer limit view of reachable sets (Examples 5.2, 5.30) suggests treating the set of pipeline states reachable by time $\tau$ (e.g. tuples of issued, taken, consumed counts) as a set-valued function of $\tau$, and checking empirically whether it varies continuously; a discontinuity would mark a switching condition.
- Outer semicontinuity is a natural correctness property for a model of "which events may fire next" as a function of state: at a tie (two events ready at the same instant), the model should allow every order that nearby states allow, as in the step-function example on p. 102–103.
- The tangent cone viability condition (Lemma 5.26) may be a compact way to state when the admission constraint $\text{issued}-\text{taken}\le W$ can be kept while the scanner continues; at the boundary, only progress directions that do not increase the gap are admissible.

## Chapter 6: Well-posed hybrid systems and their properties (pp. 117–137)

*From `../precis/ch06-well-posed-hybrid-systems.md`.*

*Speculative; for the team lead to assess.*

- Treat the pipeline model as a hybrid system and check the basic conditions on it: admission and ordering guards written with non-strict inequalities (closed sets), and event maps that allow every tie-break at equalities (osc). If they hold, Proposition 6.34 suggests that small timing perturbations produce traces $(\tau,\varepsilon)$-close to some unperturbed trace, which would bound how far jitter near the admission and ordering switches can move the observed behaviour.
- Proposition 6.35 maps onto a sanity property: if the model admits no infinite burst of events in zero time (e.g. no unbounded run of admissions without time passing), then there is a robust uniform bound on events per unit time. The exact traces could test this bound directly.
- $\omega$-limit sets (Propositions 6.21, 6.26) could describe the steady-state regime of the pipeline, e.g. the attracting set of (outstanding-window, worker-occupancy) states; weak invariance means the steady state can be sustained by some complete schedule, which is a natural consistency check on measured steady-state occupancy distributions.
- The $\rho$-perturbation $\mathcal H_\rho$ offers a model of measurement and timing uncertainty that is state-dependent (inflation radius $\rho(x)$); one could compare traces against a $\delta\rho$-inflated model rather than the exact one, with $\delta$ fitted to timer resolution.
- The counterexample on p. 133 (sign-type ODE) warns that a model whose deterministic rule switches strictly at a threshold can be "nominally" fine yet admit a sliding/chattering limit under perturbations; near the ~8-worker resource switch, the regularised (both-regimes-allowed) model is the safer one to reason with.

## Chapter 7: Asymptotic stability, an in-depth treatment (pp. 139–168)

*From `../precis/ch07-asymptotic-stability-in-depth.md`.*

*Speculative; for the team lead to evaluate.*

- **[Corrected after ChatGPT's review: Example 7.35 concerns a constrained inclusion on a compact flow set under the basic conditions, not any terminating system; and our bottleneck maximum is a lower bound, not such a function. See `decoder-as-hybrid-system.md`.]** **Example 7.35 is the most transferable result.** The decoder is a terminating computation: model it as a hybrid system whose flow and jump sets contain only "work remaining" states, so it has no complete solutions. The chapter then guarantees a continuous function of state that bounds remaining time (or, in the purely discrete version, remaining events). A natural candidate is a weighted sum of blocks not yet scanned, issued, taken and read; the conservation laws could fix the weights. The result is existence only, under Assumption 6.5 and compactness, so a candidate still has to be constructed and checked against the traces.
- **The Lyapunov decrease conditions (7.9)–(7.10) as trace checks.** "Strict decrease by a fixed factor at every jump, and at a fixed rate during flow" is a testable invariant on exact per-block traces for a candidate potential, for example blocks outstanding weighted by remaining work. A check that passes on all traces is evidence of a uniform completion bound, not a proof.
- **Proposition 7.13 corresponds to "no burst of unbounded events in bounded time".** If no instantaneous-Zeno behaviour exists at the terminal state, the number of events per unit time is uniformly bounded over a compact set of starting configurations. In pipeline terms, the window $W$ and the finite $k$ should give this directly; the proposition is the general form.
- **Corollary 7.28 (slow switching among stable modes)** may describe a decoder that changes regime, for example as the binding resource changes near 8 workers. If each regime on its own drains the queue robustly, switching that is slow enough on average preserves the practical property. For a terminating run, "practical" means completion up to a residual whose size depends on the switching rate; that residual has to be quantified separately.
- **Corollary 7.24 (reduction principle) for a cascade.** **[Corrected: the safety invariant is $\text{issued}-\text{taken} \le W$, since an issue from $W-1$ reaches $W$; and attractivity and compactness premises must be established separately.]** First show that the admission-window set $\{\text{issued}-\text{taken}\le W\}$ is invariant; then analyse the dynamics restricted to it, such as reader order and worker occupancy. Most asymptotic content does not transfer: the result concerns attractors of infinite runs, and the pipeline's terminal state is reached in finite time.

## Chapter 8: Invariance principles (pp. 169–184)

*From `../precis/ch08-invariance-principles.md`.*

*Speculative; for the team lead to evaluate.*

- **Conservation laws as the $V$ with $u_C=0$.** The exact conservation laws (for example, blocks scanned equal to blocks issued plus blocks pending admission) are functions that are constant during flow and change only at events, like the bouncing ball's energy (Example 8.5). Theorem 8.2 and Corollary 8.4 then say where a long run can accumulate. For a terminating run there are no complete solutions, so the theorem says nothing about the end state. Its natural use is a steady-state model, for example an idealised infinite stream of blocks, in which the "limit set" is the sustained regime: which resources are saturated, and how full the window is.
- **The hybrid "no dissipation" set as a bottleneck locator.** Among steady states with constant throughput, the set (8.9) consists of states where no event makes progress on a chosen potential. Taking $V$ as outstanding work, this candidate set should contain any stall or livelock region, for example the window full while the in-order reader waits on one slow block. Showing the set is empty for $r>0$ (Theorem 8.8(b)) is the shape of a no-livelock argument.
- **Meagre-limsup principles are well matched to exact traces.** Theorem 8.11 needs a condition only along the one observed solution. Given a recorded trace, an output such as idle-worker time that is integrable, or tends to zero on average over windows, places the trace's limiting behaviour in the zero set of that output. On a finite trace this is a heuristic about the tail of a long run, not a theorem.
- **Corollary 8.9 (ii)/(iii) for regime switching.** If a potential strictly decreases at every event (block completion) but not during flow (waiting), it is enough to rule out a run that only waits forever. In the pipeline that is a deadlock check, which bounded $W$ and $k$ workers might make easy to discharge.
- **Section 8.5's (S2) condition for the two throughput regimes.** With one Lyapunov-like function per regime (scanner-bound and worker-bound, switching near 8 workers), the "value at re-entry not above value at last exit" condition is a concrete property to test on traces where the regime changes during a run.

## Chapter 9: Conical approximation and asymptotic stability (pp. 185–197)

*From `../precis/ch09-conical-approximation.md`.*

*Speculative; for the team lead to evaluate.*

- **Homogeneity as a scaling law in block size.** If doubling every block's size doubles every work quantity, the per-block service times and the jump structure (block hand-offs) stay the same and only the speed of flow changes. That is the content of Lemma 9.3, and it predicts that whole traces at one block size are time-rescaled copies of traces at another. Testing this on the exact traces would show where it fails, for example through fixed per-block overheads. **[Corrected: in Example 9.16, $f_2$ (degree 5) is the *higher*-degree term, not the lower; and duration scaling in a fixed event model is the narrower, valid check. See `measurements.md` item 6.]**
- **Degree as a diagnostic of how completion time scales.** The negative-degree, finite-time case is the one that matches a terminating decoder. Proposition 9.4's "halve in bounded time at every scale" argument, translated, means a fixed fraction of remaining work is cleared in time proportional to the current scale. That gives completion time linear in total work. Treat this as an analogy unless a homogeneous model is actually written down. Note also that worker count is an integer, not a dilation variable.
- **Conical approximation at the regime switch (about 8 workers).** Near a point where the binding resource changes, the state space is locally two cones (scanner-limited and worker-limited) meeting at a boundary. Theorem 9.11's recipe, tangent cones plus constant or linear dynamics, is a principled local model of behaviour at the switch. The examples warn that a stable original can have an unstable or non-attractive cone model (Examples 9.14, 9.15), so a contradiction between such a model and the traces would not be proof of anything.
- **Theorem 9.8's homogeneous-perturbation robustness.** This is the formal version of "proportional jitter in service times does not change the qualitative behaviour", as opposed to absolute jitter. It is relevant if measured variability scales with block size. The theorem needs well-posedness and an infinite-horizon pAS model, so it would apply to a steady-state model of the pipeline rather than to a single finite run.

