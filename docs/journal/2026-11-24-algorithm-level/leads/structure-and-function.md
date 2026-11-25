# Structure and function: Alon's method, and what it should mean for us

*The Coordinator, 25 November 2026, on Louis's Alon question (`../external/chatgpt-alon-motifs.md`). This is a steer, not a result. Every claim about the literature is from memory unless marked as read; a full reading is commissioned below. Prior work stays prior work (`../essence.md`).*

## Louis's framing, kept

> "i would not frame as universal motifs but optimal for our algorithmic purpose potentially... we do not need to force the analogy in here, there may indeed be other thinkers we should reach for"

## What ChatGPT's answer gets right, and where it thins out

**Right:**
- Alon's method is a loop: structure → function → perturbation test → design principle. Jain's is a loop of the same shape.
- An activity (the Huffman scan, the inversion) is a component, not a motif. A motif is a *relation* between components that produces a characteristic behaviour.
- ChatGPT itself notes that Alon's motifs are defined statistically, as over-represented against a randomised network, and that we have no such null model for computations.

**Where it thins out:**
- **Its six "computational motifs" mix three kinds of thing.** Some are pipeline organisation (fan-out with ordered fan-in; negative-feedback admission). One is a dependency shape (the serial chain). Two are *transformations* between equivalent computations (contraction and enrichment along composition). A taxonomy needs those levels kept apart: a structure and a rewrite of one structure into another are different objects.
- **The Jain–Alon table** matches words, not results. Its value is only in the loop shape above.
- **It undersells Alon's Part III,** which is the part that fits Louis's framing best. On "optimal for our purpose":
  - Alon's optimality work asks why a circuit has *this* parameter value, given a cost and a benefit. Dekel and Alon (2005) found the *lac* system's expression level near the optimum of measured cost against benefit.
  - With several objectives, designs lie on a Pareto front whose corners are "archetypes", each best at one task (Shoval et al., 2012).
  - That is our situation exactly: wall time, CPU and memory are competing objectives, and the worker-count question (7 against 8) is a point on such a front. The motif *vocabulary* matters less to us than the *optimality* argument: given the cost model, which organisation is best, and why.

## Other thinkers to reach for

For a taxonomy of *transformations between equivalent computations*, the part Louis most wants, there is a mature tradition that already has theorems: **the algebra of programming.** The correspondences, as a steer to verify:
- **Contraction along composition is fusion.** A function composed with a fold is itself a fold when a homomorphism condition holds. Examples are deforestation (Wadler) and the Bird–Meertens laws (Bird and de Moor, *Algebra of Programming*). They say *when* two passes can become one, and what the fused step must be.
- **Enrichment along composition is tupling, the "banana split".** Two folds over the same structure become one fold that carries a pair (Meijer, Fokkinga and Paterson, 1991). Forward-mode automatic differentiation, carrying a derivative alongside a value, is a case of it. That is Louis's JAX intuition, in this vocabulary.
- **"Can a serial dependency become a collective operation?"** That is the homomorphism theorems' question. Gibbons's third homomorphism theorem (1996) says a function computable by both a leftward and a rightward fold is a list homomorphism, so it divides and parallelises. Associativity is the hinge: record 105's CRC is an associative scan for exactly this reason.

Those theorems give the "reverse direction" ChatGPT admired in Alon: from the behaviour wanted (one pass; parallel), to the structural condition that makes it possible.

## The concrete lead this points to: the scanner as a composition of transition functions

The scanner is the bottleneck above about 8 workers (§57), and 81% of its work is Huffman decoding at about 1.5 ns a symbol. A Huffman decoder is a finite-state machine over bits. Its transition on a chunk of input is a *function on the state set*, and functions compose associatively. Hence:
- **The structure:** decode a chunk of input from *every* possible start state, then compose the chunks' functions with a parallel prefix. This is data-parallel finite-state machines (Mytkowicz, Musuvathi and Schulte, ASPLOS 2014), and, for Huffman codes specifically, parallel decoding by self-synchronisation (Klein and Wiseman, 2003).
- **Why it can be cheap:** codes self-synchronise. From different start states the decoder converges to the same state within a few codewords, so a chunk's function collapses to few distinct outcomes. That is "contraction along composition" with a measurable contraction rate.
- **The obstruction particular to bzip2:** the decoding table changes every 50 symbols (the selectors), so the state includes the symbol count modulo 50 and the group index. The count doesn't self-synchronise, since it depends on how many symbols came before. So the composed function has to carry the count offset as well: at most 50 cases, or it is resolved by a cheap first pass that counts codewords. Whether that leaves a net gain is the open question.
- **Its status:**
  - **conjectured** as a gain;
  - **prior art** as a technique (cite it, don't present it as new);
  - **relation to earlier work:** it differs from record 122's cross-block parallel parse, which finds block boundaries; this is *within-block* parallelism of the Huffman stage;
  - **what it would do:** raise the ceiling, not close a gap, which is what the queueing analysis says matters.

## How to progress it

1. **Record and steer (this file).** Done.
2. **MONODROMY (channel #40, queued after #39):** three questions, with its referee lens.
   - **(a)** Map the decoder's structure onto the algebra-of-programming laws: which of its passes fuse, which tuple, and which are homomorphisms. What does each law say the fused or parallel step costs, under Q, D, P and S?
   - **(b)** The scanner as a transition-function composition. Bound the contraction (self-synchronisation length for bzip2's tables, measurable from the per-block records and the tables). Bound the selector-count obstruction. Give a predicted scanner cost per symbol for a k-way within-block parallel decode, with the four-arm and profile ceilings as the referee.
   - **(c)** Alon's optimality argument (Part III) applied to our objectives: is the current configuration on the Pareto front of wall, CPU and memory? What are its archetypes?
3. **A full reading, as with the courses:** Alon's book, the motif papers (Milo et al. 2002; Shen-Orr et al. 2002; Mangan and Alon 2003), Alon 2003 "The tinkerer as an engineer", Dekel and Alon 2005, Shoval et al. 2012; then Bird and de Moor, Meijer–Fokkinga–Paterson, Gibbons 1996, and Mytkowicz et al. 2014. Saved where they can be, and read in full rather than through a subagent's summary.
4. **ChatGPT Pro at Pro effort:** the same three questions as (2), independently, then a comparison. The prompt is `../briefs/chatgpt-structure-prompt.md`.
5. **A measurement before any code:** the self-synchronisation length of bzip2's Huffman tables on the corpus. It is cheap, offline, and decides whether lead (b) is worth building.

## What not to do

- **Don't build a taxonomy for its own sake.** A class of structure earns its place only if it predicts a cost or a possibility on our referee.
- **Don't claim universality.** Our "motifs" are hypotheses about useful abstractions, judged by whether they are *optimal for the decoder's purpose* under its cost model.

## After ChatGPT's second answer (same day)

ChatGPT's second answer (`../external/chatgpt-alon-motifs.md`, second section) withdraws its own motif catalogue as trivialising. It converges with this file on three points, independently written:
- optimality, not recurrence;
- Bird's program calculation;
- for the scanner, asking what downstream work *needs* from it, rather than how to make it faster.

Four things it adds are adopted:

1. **The generative direction: Ma, Trusina, El-Samad, Lim and Tang (2009).** Start from the behaviour wanted, enumerate a space of structures, test each across parameters, and extract the principle separating the ones that work. Its catch is that this needs a *finite, meaningful* design space. We have one in hand:
   - **The obligations:** the scanner–worker boundary splits into a handful of obligations: boundary discovery, code interpretation, symbol determination, the selector and group bookkeeping, MTF, and the run records.
   - **The placements:** each can be placed serially in the scanner, in a worker, speculatively (computed from every start state and composed), or not at all, where an intermediate object makes it unnecessary.
   - **The test:** obligations × placements is a finite space. Each point can be scored first by the two-station model and the referee (Q, D, P, S), then by a simulation with the measured per-block demands.

   That is a Ma-style enumeration we can actually run, and it is the **generative test** ChatGPT asks for.
2. **An algebra of synchronisation: Baccelli, Cohen, Olsder and Quadrat (1992), max-plus.** This fits our pipeline more exactly than MVA. Event times in a pipeline with fixed service times obey max-plus linear recurrences, and the throughput is the maximum cycle mean. The profile's own fit, wall(k) = c + max(b, a/k), already has that form. Admission (issued − taken < W) and in-order resequencing are max-plus constraints too. Variable service times make it stochastic max-plus (a Lyapunov exponent), which is still algebraic rather than simulated. It is a candidate replacement for "MVA on a surrogate".
3. **What makes a decomposition good: Simon (1962), near-decomposability; and Ragan-Kelley et al. (2012), separating the algorithm from its schedule.** The decoder's decomposition is inherited from the format. Halide's separation is a way to state the space of valid organisations of a fixed computation. Simon's criterion asks which boundaries the mathematics needs and which history left.
4. **A retrospective reconstruction as the methodology's own test.** Take two breakthroughs that happened: the traversal → Rauzy–Veech transition (MONODROMY's records) and Louis's page-dewarp change of representation (`../external/jax-idea-verdict.md` and its sources). Ask which framework *accounts for how each became possible*, and which only describes it afterwards. A framework that can't reconstruct a known breakthrough is unlikely to generate a new one.

**The acceptance criterion (ChatGPT's, adopted):** a framework is judged by whether it generates a formulation we wouldn't otherwise have had, not by how comprehensive it looks. Categories that only re-file existing ideas count as a failure.

**The FSM-composition lead stands, and the two answers meet there.** The chunk's transition function *is* an intermediate object C, with a different cost structure from decoding (F = D ∘ C). Self-synchronisation is what keeps its composition cheap, the "fragmentation under repeated composition" worry. It is the first candidate the enumeration in point 1 should score.

**Revised progression:**
1. MONODROMY, channel #40, queued after #39:
   - the obligations × placements enumeration, scored by the referee;
   - the max-plus form of the pipeline against the profile's 20 walls;
   - the self-synchronisation bound and the selector-count obstruction;
   - the retrospective reconstruction of the Rauzy–Veech transition.
2. ChatGPT Pro at Pro effort, independently: the same, with the page-dewarp reconstruction, since Louis holds that history. Prompt: `../briefs/chatgpt-structure-prompt.md`, updated.
3. A full reading of the sources: Alon (book Part III; Shoval 2012; Dekel and Alon 2005), Ma et al. 2009, Bird and de Moor, Bird and Mu on the inverse BWT, Gibbons 1996, Mytkowicz et al. 2014, Baccelli et al. chapters 1–3 and 7, Simon 1962, Ragan-Kelley et al. 2012. Saved, and read in full.
4. One cheap measurement first: the self-synchronisation length of bzip2's Huffman tables on the corpus.

## After ChatGPT's third answer and the Khammash explanation (same day): necessity first

The records:
- ChatGPT's third answer (`../external/chatgpt-alon-motifs.md`, third section): the design-enumeration lineage, Ma et al. 2009 → Cotterell and Sharpe 2010 → Lim, Lee and Tang 2013 → Araujo and Liotta 2018; plus SPIRAL and optimal-depth sorting networks.
- Its explanation of Khammash 2021 (`../external/chatgpt-khammash.md`). It is ChatGPT's reading, to be checked against the paper.

**The programme Khammash states fits ours almost exactly:**
1. specify the function;
2. derive necessary conditions;
3. characterise the solution space;
4. check physical realisability;
5. establish performance and optimality.

**We have done more of this than it looks, under other names:**

| Khammash's step | the biology | ours, already | what's missing |
|---|---|---|---|
| the function | robust perfect adaptation | exact inversion (and decoding) within a cost bound: the referee (Q, D, P, S), `../essence.md` | — |
| necessary conditions | the determinant condition; the internal model principle (an integrator is necessary) | the lower-bound work: G1; record 118's Theorem I; record 119's closure result (composition keeps the schedule's form, not its compactness) | a necessity statement in the *positive* form: "any inversion within K passes and s cursors must contain a structure X". That is the internal-model analogue |
| the solution space | enumeration (Ma) and then a basis (Araujo and Liotta: opposer and balancer modules, and their composition) | nothing systematic | **the enumeration, and then the basis** |
| realisability | the stochastic setting kills the autocatalytic controller; the antithetic one survives event by event | the referee against the real machine: "G1 is not a machine lower bound"; the four-arm and profile ceilings | the same move: which abstract solutions survive the machine (cache, dependent reads, SMT) |
| optimality | speed, noise and resource trade-offs among the admissible | the performance work (Jain, queueing, stage 1 and 2) | joining the two |

**Two ideas from the Khammash explanation that apply directly:**
- **Same function, different structure (Figure 3).** Two systems with an identical transfer function: one adapts by its form, the other by numerical cancellation, and the second breaks under a small change. For us, two decoders with identical output can have different cost structures and different robustness across inputs. That is why the referee counts structure (dependent reads, passes, state) and not only output. It is also why "works on the six admitted blocks" is a fine-tuned property until shown structural.
- **The hidden coordinate (Figure 5).** Five physically different controllers integrate in the coordinates Z, −Z, log Z and Z₁ − Z₂. The function lives in a *function of the state*, not in any one component. That is our "representation change" (Rauzy–Veech in place of LF traversal) put as a method: search for the coordinate in which the work cancels or contracts.

**The concrete proposal: our three-node network.** ChatGPT's open question is what plays the part of Ma's three-node network for us. Our candidate is the referee's own class:
- **The space:** streaming machines with at most s ≤ |Σ| + O(1) cursors and K passes, with a bounded rule table, on a tiny alphabet (|Σ| = 2 or 3) and short strings.
- **The function:** exact BWT inversion on *every* string up to length n. That is exhaustive correctness, the analogue of Ma's robustness across parameter sets.
- **The method:** enumerate the rule tables, reduced by symmetry, as Bundala et al. did for sorting networks. Find which ones invert. Cluster the survivors into families (Cotterell and Sharpe). Then try to state the algebraic condition that separates them (Araujo and Liotta), and a positive necessity statement (the internal-model analogue). Where the enumeration finds no survivor within the bound, a SAT encoding can make that a proof within the class.
- **Its status:** conjectured as fruitful. Its feasibility depends on how large the rule-table space is for s and K that matter. That is MONODROMY's first question below. If the space is too large, the abstraction is wrong, and finding the right one is the work, as ChatGPT says.

**Added to MONODROMY's channel #40 (amended before dispatch):** question 5, the referee-class enumeration above: its feasibility, the right abstraction, and what a positive necessity statement would look like. Plus the Khammash table above as a frame.

## The dynamical layer (25 November, after `../external/chatgpt-dynamics.md` and stage 1's data)

ChatGPT proposes a third stage between the performance analysis and the design-space work: **dynamical characterisation**. It names hybrid dynamical systems (Goebel, Sanfelice and Teel) as the language: discrete events with continuous quantities, the "digital on analogue" Louis described. Alongside it: system identification (Ljung), max-plus algebra (Baccelli et al.), model reduction ("find z = h(x) with self-contained dynamics", the Khammash move), and Hellerstein et al., *Feedback Control of Computing Systems* (2004), as a textbook to mine like Jain's.

**What stage 1 already gives this layer, measured:**
- **The conservation laws hold exactly, event by event.** In all 840 grid decodes, each buffer's occupancy equals enqueues less dequeues, and its occupancy integral equals its blocks' summed residence (Little's law). ChatGPT's A − B, B − C and A − C identities are these, and they are now verified rather than assumed (§66).
- **The trajectories exist.** Every block's issued, enqueued, got, ready, taken and consumed times, with under-lock sequence numbers that give a valid linear order, at every count, for four rounds (`blocks-r00..r03.tsv` in the bundle). Queue-occupancy trajectories, starvation episodes and head-of-line episodes can all be rebuilt exactly. That is the "properly synchronised event traces" ChatGPT asks for.
- **A first model-reduction fact:** occupancy alone is *not* a sufficient coordinate. Above the knee, mean admitted blocks sit at about 20 whatever the window (32–80), while worker starvation goes from 4% to 56%. What changes is the scanner's per-block service and the workers' contention, so the state needs at least the scanner's rate and a contention measure.

**Adopted:**
1. **Time-localised (pulse) interventions in stage 2.** Apply the pacing dose only over a contiguous range of blocks (say blocks 600–800 of 1724) at k\* and at 10, for the scanner and for emission (A7), and measure from the traces: the queue's depletion, when starvation begins, the reader's rate, and recovery. That gives the time-dependent response matrix G_ij(t, τ) as finite responses. The pacing hook already runs per block, so this is a block-range condition, not new instrumentation. Proposed to the Pathfinder as stage 2's sixth choice.
2. **System identification on stage 1's traces before stage 2's data.** Fit the smallest state that predicts each block's completion time from the previous state, starting from the max-plus recurrence (issue, worker, reader) with the measured service times. Then ask which added variable (head-of-line age, contention, the scanner's instantaneous rate) removes the most error. This goes to MONODROMY as question 6 of #40 (amended before dispatch).
3. **Hellerstein et al. (2004)** goes on the reading list with the others, to be read in full.

**Not adopted now:** SINDy on the raw traces (the switching makes smooth regressions misleading); chaos, bifurcation and Koopman methods (no evidence yet that they're needed). Optimal control of the worker count is the conventional application, and it stays in run 4.
