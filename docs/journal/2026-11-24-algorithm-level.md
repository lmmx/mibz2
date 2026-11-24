# Working at the level of the algorithm's mathematics: a record of the conversation

*24 November 2026. Written by the Coordinator at Louis's request, to retain the conversation in the
Coordinator's session that began when he shared the SSL-Coq (Super Mario 64) work and ended with
his statement of what he wants the team to work on. It is a record, not a decision: the
formulation it points to is Louis's to write (§6).*

> Louis: "these past few messages have really gotten us close to what I've been trying to talk
> about for some time … I need this to be retained."

## 1. Louis's statement of intent, in his words

> "The payoff I'm talking about isn't in particular attempts. The payoff I'm talking about is in
> reframing what we're doing here as a specification which we try to supply new mathematical
> approaches to fit, such that we could establish a new equivalent to [Rauzy–Veech] induction, as
> that was, stepped to from the traversal walk that we had before."
>
> "there are these incremental parts, which are just changes to an existing algorithm, and what
> we're trying to actually do is step from one algorithm to another. … that is the level I want to
> be working at, but rather than working at optimizing a particular algorithm, we want to sort of
> change the algorithm structure based on the mathematics it implements. And we've been struggling
> to work at that level because it's not formalized at that level."
>
> "I think this is going to require a really well formulated problem, and that formulation is what
> I should work on, such that I can then start a new agentic effort at that level."
>
> "It's either a yes or no kind of thing, not a score."

Earlier in the same conversation:

> "the point is, we've talked about bouleversements, right? The point is, we've been trying to
> establish a different way to perform the permutation operation in the BWT inversion. That is our
> goal. And we have two ways of doing it already. The pot shots are just incremental … but the
> major goal here that we're going for is to reinvent how we do that. Reinvent the mathematics, to
> chart the mathematics and prove what is necessary, but also try and come up with new ways,
> because we already did that when we reframed it as induction … And by trying to achieve the
> bouleversement, we actually keep achieving sub goals, which give us these incremental gains, but
> they're not really the point. And I'm kind of asking, what could proof systems do that would
> uncover this alternative? Because I've tried all these kitchen sink approaches and wormhole
> approaches, and they still unfortunately don't seem to have gotten me what I want."
>
> "yes, a proof is an algorithm, and charting a proof means charting algorithms. Curry Howard. …
> the problem we keep finding, though, is that we're like changing the algorithm and retaining
> correctness without like a kind of cost element to it, which is actually necessary. And that's
> why it kept failing, and that's the part I'm trying to plug with this formalism. It feels like we
> have all the components here to sort of wire this together and finally close the circuit and have
> it run to the point that we actually establish some new goal."

### What this means, as the Coordinator understood it

- **Two levels of work.** *Incremental:* changes to an existing algorithm, judged by the bench
  (pot shots). *Structural:* a step from one algorithm to another, where the new algorithm
  implements different mathematics for the same specification. The precedent is the step from the
  LF traversal walk to Rauzy–Veech induction (interval exchanges, return words; the Cells and Open
  kernels).
- **The structural level is the goal.** Incremental gains are by-products of pursuing it, not the
  point.
- **The target is yes/no.** Either a new mathematical approach is found that fits the
  specification — correct, and meeting a cost requirement — or it isn't. It is not a quantity
  to minimise. (Louis calls this a "dense reward"; the operative meaning is a target that is
  met or not.)
- **What has been missing:** a formalisation at the level of "which mathematics does the algorithm
  implement", with a cost element alongside correctness. Without it, work drops to the
  implementation level (Kitchen Sink, Wormhole, T², the zero-fill), where correctness is preserved
  and cost is discovered only at the end.

## 2. The SSL-Coq material

**The source.** Tariq Ali's work with OpenAI Codex on the *Super Mario 64* A-button challenge in
Shifting Sand Land ("SSL-Coq", repo `tra38/sm64-wmotr-abc-proof`, the
`less-than-one-a-press` project); a transcript of its last days, "The Eureka Movement For
'Astral Rejection'", which Louis shared, with a Google Doc version
(`docs.google.com/document/d/1mk5xFh7Mjh__F_c5DTFNygnFCzjn-fZVIc-iPTgjsIw`), and ChatGPT's
explanation of it.

**Its goal is a minimisation phrased as a yes/no question.** The A-button challenge minimises A
presses. With one press the best known, the project asks whether *fewer than one* is possible:
yes (a witness route) or no (an impossibility proof). "The idea is simple — if we could prove that
there was no viable setup, then we get closer to proving spawning displacement is impossible."

**Their "move 37", in two halves.**
1. *The insight.* The goal looked contradictory: touch the warp below the pyramid while standing
   on its top. The game keeps not one Mario position but three — movement, collision and stored
   graphics — normally in step. Make them disagree, and one touches the warp while another selects
   the top ("Astral Rejection"). An impossible conjunction became possible once one concept turned
   out to be several representations that only usually agree.
2. *The method.* They did not first find how the game could produce the disagreement. They
   **supplied it** in a fixture, confirmed the payoff (warp and top together, spawning
   displacement), and only then worked backwards to make the game generate it — the stopped crawl
   (a 973-unit gap), shell mounting, object pushes, substitutions of actors — replacing one
   supplied condition at a time. **Payoff first, mechanism second.**

**How they used Coq.**
- One main theorem (`MainTheorem`) states the goal; its unproven premises are the open research
  questions. Each session proves a premise or a local obstruction, and an audit checks the wiring
  into the main theorem and that nothing is assumed silently ("no project axioms or admitted
  proofs").
- The proofs are about the real game code (the decompilation, with CompCert semantics): "Checked
  Coq execution returns 2 without changing memory".
- Proofs mostly close branches ("Ordinary dropping cannot substitute a high remembered hand height
  for Mario's movement height"), and explain failed searches ("Source inspection explains the
  obstacle").
- Scope is stated honestly ("bounded", "not a global impossibility theorem"); a proof that did not
  go through is kept as unchecked, not claimed.
- Every fixture is labelled supplied or controller-produced; results carry receipts.

**The search side:** reverse scattershot (propose predecessors of a successful state, simulate
forward, keep those that still succeed; 2,200 proposals in one run), beam and continuation
searches (13,889 and 18,684 branches), intermediate fitness measures (gap size, height) — with the
trap that improving an intermediate measure need not improve the goal ("a larger position gap can
be useless if Mario's subsequent floor query selects the wrong surface").

## 3. How the conversation mapped it onto mibz2

### 3.1 Earlier the same day: "by the book"

Bach, Black, Huiberts and Kafer, *Beyond Smoothed Analysis: Analyzing the Simplex Method by the
Book* (arXiv 2510.21613, STOC), via Jeremy Kun's review, led to: cost theorems beside correctness
theorems (PROOF-PROGRAMME §5.9, #803); contention as a model term (§5.10); records by the book
(METHOD.md §6a); the Wikidata block census (#805: construct cost predicted from block structure,
held-out R² 0.964; emit not, 0.656; contention a uniform ×1.12 at 10 workers); MONODROMY record 96
(every rigid jump-walk route pays one single-row operation per width-1 run, D ≥ h₁, and Open meets
it); and the retrospective of failed ideas (`2026-11-24-retrospective-proposal.md`, running).

### 3.2 The failures read against SSL-Coq

T² broke all three of SSL-Coq's disciplines: its pre-screen's ALIVE was a supplied-state success
(synthetic cycles, a C harness) quoted as evidence about the native target; it jumped to a full
build without the intermediate rungs (one real block, all blocks, the shipped path at one worker);
and the decisive fact was already in the record (`2026-11-20-toy-transfer.md`: "the dependent
load is not the main cost"). More generally, the team has worked **mechanism first, payoff last**,
the reverse of SSL-Coq's order. That is Louis's "missing cost element".

### 3.3 Proofs as a way to find new algorithms, not only to check them

- **Curry–Howard.** A constructive proof that the BWT can be inverted is an inversion algorithm.
  The LF walk is the proof by induction along the text (hence a serial chain); the induction
  reframing is a proof by induction over the interval exchange (Rauzy induction composing return
  words). A new proof of invertibility with a different structure is a new algorithm with a
  different shape. A proof assistant makes that search checkable and can extract a decoder that is
  correct by construction.
- **SSL-Coq's backward method on the mathematics.** `2026-11-20-toy-transfer.md` already found a
  "supplied state" for inversion: perfectly clustering words, above all the two-letter Sturmian
  case, where every output position is computed in closed form with no walk (letter k is
  ⌊(k+1)α + ρ⌋ − ⌊kα + ρ⌋). The team's Sturmian solver is exact on 5,627 Sturmian BWTs and fires on
  0% of Wikidata. The ladder: two letters, one run each → k letters, one run each (interval
  exchanges; Ferenczi and Zamboni) → few runs per letter → real blocks. At each rung, attempt to
  prove the walk-free inversion still holds; a proof is progress, and a failed proof names the
  ingredient real blocks lack (the toy-transfer note guessed run width, broken by random
  identifiers).
- **Program calculation.** Bird and Mu derived inverse-BWT algorithms by equational reasoning from
  the specification ("Inverting the Burrows–Wheeler transform", *Journal of Functional
  Programming*, 2004). With the definitions formalised, laws can be applied mechanically, and
  equality saturation can enumerate equivalent programs for a cost model to rank — the
  mathematical counterpart of reverse scattershot.
- **Necessity.** Theorems of what every inversion must compute (record 96 for walk-like methods)
  say where a structural step must break with the present one; theorems of what is *not* needed
  open routes.

### 3.4 Closing the circuit with cost: SSL-Coq's order

1. Name a capability: "if the decoder knew X for free".
2. **Supply X and measure the payoff:** decode normally, record X, rerun the kernel with X handed
   to it, on real blocks, through the shipped code, on the i9. This bounds what any reframing that
   produces X can buy. Cheap: about an agent-day and minutes of i9 time.
3. If the payoff is large, state in Lean that the kernel is correct given X.
4. Work backwards to produce X natively: a new characterisation, a new proof (a new algorithm),
   checked in Lean and priced by the cost theorems and the census.
5. Discharge what is supplied, one condition at a time.

A hypothesis for the "three positions" of mibz2 (not a finding): a retained row's place in the BWT
is known, but its place in the output is learned only by walking, which makes emit a chain. The
Sturmian toy is the case where the second is computed directly. Candidate capabilities to supply:
retained rows' output positions; the order of the strips (construction); the final positions of
wide runs.

### 3.5 Corrections Louis made along the way

- **Not a floor.** The Coordinator proposed a floor question ("the least work any inversion must
  do") as the main theorem. Louis: the floor is not the point; the goal is a new way to do the
  permutation.
- **Pot shots and the major shot are different tasks.** A threshold just below the current record
  invites incremental work; "fewer than one A press" was the major shot.
- **Not "decide freely" what to prove:** take how SSL-Coq used proofs, in service of its
  particular goal.
- **The level of work.** Not optimising an algorithm, but changing the algorithm's structure
  according to the mathematics it implements, judged yes/no.

## 4. What exists that the formulation can build on

- **The precedent step:** LF walk → Rauzy–Veech induction (Cells, Open). The team has made one
  structural step already.
- **The specification:** PROOF-PROGRAMME §1 (the contract in `src/`, pseudo-Lean), §2 (T1: the LF
  walk reconstructs the text; T2: the Open kernel equals the LF walk), §5 (conditional routes, the
  hypothesis map, the route census), §5.9 (cost statements beside correctness), §5.10 (contention).
- **Cost:** the census (#805) for block features and their per-block cost; the cost-weighted
  reading (the dense tail is 19.4% of blocks and 27.6% of kernel time; the costliest 10% of blocks
  16.2%); emit is about 47% of kernel time at one worker.
- **Necessity:** record 96 (D ≥ h₁ for rigid jump-walk routes; Open meets it), and the GUID wall
  (PROOF-PROGRAMME §5.4).
- **The toy:** `2026-11-20-toy-transfer.md` (perfectly clustering words, the Sturmian closed form,
  Ostrowski and Rauzy–Veech coordinates as a candidate route to random access, and "a native
  surrogate for width" as the open question).
- **No formal proofs yet:** nothing in the repository is checked by a proof assistant. "Proved"
  in the records means argued in prose and reviewed. PROOF-PROGRAMME §4 plans a Lean package in a
  sibling repository (new infrastructure, Louis's approval).

## 5. What the Coordinator suggested, not yet decided

- Supplied-state payoff experiments for two or three capabilities, before any formal work.
- A small Lean repository (`mibz2-lean`), with MONODROMY leading, starting from the Sturmian toy
  and its first generalisation step.
- The evidence-scope rule (#806, awaiting Louis) and the next-round candidates
  (`2026-11-24-next-round-candidates.md`, in #806).

## 6. What comes next: Louis's formulation

Louis will formulate the problem at the structural level, then start a new agentic effort on it.
Questions the formulation will have to settle, collected from this conversation as prompts, not
answers:

- **The specification** that a new mathematical approach must fit: the inverse BWT, stated how —
  as the standard permutation, as a walk, as an interval exchange, as a bijection to necklaces?
- **The yes/no criterion:** what makes an approach count as a new structural step rather than an
  increment — correctness (checked) plus which cost property, in which units, on which blocks?
- **Where cost enters:** the supplied-state payoff, the census, the cost theorems.
- **The toy and its ladder:** where the search starts, and what each rung must show.
- **What the agents do:** propose characterisations, attempt proofs, extract algorithms, measure
  payoffs; how a failed proof is recorded so that it names the missing ingredient.
- **How a yes is confirmed:** a proof checked by Lean, and a measured payoff on real blocks.

## 7. Addendum: ChatGPT on "the move 37", relayed by Louis

Louis asked ChatGPT "what would you say the move 37 of this work is?" Its answer, in summary,
with its key lines quoted:

- **The Mario project's move 37** is Astral Rejection itself: treating Mario not as one entity at
  one location but as "a collection of memory fields that can temporarily disagree". The question
  changes from "How can Mario physically reach the pyramid top without jumping?" to "How can the
  game simultaneously believe Mario is touching the warp and standing on the pyramid top?", which
  makes stored graphics, cached floors, object interactions and allocation order components of a
  solution.
- **Its proposed move 37 for us: impossibility results as constructive operators.** "Specify a
  desirable performance bound → derive necessary conditions for achieving it → determine which
  conditions are currently impossible → find a representation that removes those constraints →
  discover a different class of algorithm." "Instead of asking how to make the existing algorithm
  faster, we ask what would have to be true for an apparently impossible performance improvement to
  become possible? Then work backward."
- **Artificially construct the winning implementation** ("oracle-assisted algorithm discovery"):
  intermediate information already available, expensive lookups free, data in the ideal
  representation, perfect scheduling foreknowledge; then "progressively replace each artificial
  advantage with something the real algorithm produces, measuring the cost of discharging every
  assumption." It distinguishes this from the assumption-discharge ladder: "That ladder validates
  an existing candidate. This technique would generate candidates backward from artificially
  achieved performance."
- **Three methodologies:** forward optimisation ("What can we improve?"), formal investigation
  ("What is possible or impossible?"), and backward synthesis ("If we could already achieve the
  desired result, what would be the cheapest way to make its assumptions real?"). "It might be
  discovering that the expensive operation we have been trying to optimise does not need to exist
  in its present form at all." And: "I don't think we've demonstrated that this will produce
  MONODROMY's breakthrough yet. But of all the methodological ideas we've discussed, this is the
  one I would most like to see tested."

### The Coordinator's reading

- **It agrees with §3.4**, and sharpens it: the supplied-state step is a way to *generate*
  candidates, not only to check one. The capability supplied is the hypothesis about where a new
  algorithm's advantage would come from.
- **It connects to Louis's yes/no target.** "Specify a desirable performance bound" is the
  threshold; "what would have to be true" is the specification's necessary conditions; a new
  representation that makes them true is the structural step. A yes is a representation whose
  assumptions have all been discharged, proved correct and measured on real blocks.
- **One caution.** An oracle can trivialise the problem: supplying the output itself makes
  everything free. The supplied capability must be an intermediate mathematical object a new
  algorithm could plausibly compute — rows' output positions, the cycle's structure, a
  coordinate system on the interval exchange — not the answer. Choosing which objects to supply is
  where the mathematics enters, and where the formulation of §6 has to be precise.
- **Necessity results become inputs, not end points.** Record 96 says every rigid jump-walk route
  pays per width-1 run. Read constructively, a step to a different class must not be a rigid
  jump-walk over those runs: it must represent them differently. That is a constraint the next
  representation has to satisfy, and so it guides the search.

## 8. Louis: "keep adding these exchanges"; "what is the Sturmian toy?"

Louis: "yes keep adding these exchanges to the notes for the duration of the discussion of this
formulation effort." This record is kept up to date as the discussion continues.

**The question:** "what is 'the Sturmian toy'?"

**The Coordinator's answer.**

- **The toy is the easiest case of the problem, where inversion needs no walk.** Take a two-letter
  text whose BWT is as compressible as possible: all of one letter, then all of the other,
  b…b a…a, one run per letter. Such texts are exactly the (conjugates of powers of) Christoffel
  words, finite pieces of Sturmian words (Mantaci, Restivo and Sciortino, 2003). Sturmian words are
  the binary sequences with the fewest possible distinct factors for an aperiodic sequence, and
  the classical example of a "mechanical" sequence: they record when a line of irrational slope α
  crosses the grid lines.
- **Why inversion is trivial there.** If the BWT is b^p a^q (length n = p + q), its first column
  is a^q b^p, and the LF map sends the i-th b to row q + i and the i-th a to row i − p. Both cases
  are the same formula, **LF(i) = i + q (mod n)**: the permutation is a rotation of a circle by q.
  So the row reached after k steps is just start + k·q mod n, computed directly, with no step
  depending on the one before. Equivalently, the text is mechanical: letter k is
  ⌊(k+1)α + ρ⌋ − ⌊kα + ρ⌋. Every output position can be computed independently and in parallel.
- **How it sits on the ladder.** In general the LF permutation is an *interval exchange*: cut the
  rows into pieces at the run boundaries, and LF moves each piece rigidly. Two pieces make a
  rotation (the Sturmian toy). One run per letter with k letters gives a k-interval exchange
  (Ferenczi and Zamboni, 2013), which Rauzy–Veech induction handles. A real block has one piece
  per run, r pieces in all, and Wikidata's blocks have many short runs, most of them a single
  row (record 96: the width-1 runs are 30–83% of retained rows, a median of 39%). The step the
  team took from the walk to Rauzy–Veech induction is a step up this ladder. The closed form at
  the bottom is the "walk-free" extreme.
- **Why it matters for the formulation.** It is the team's "supplied state": a case where a radically
  different inversion (arithmetic instead of a walk) is known to work. The solver for it is exact
  on 5,627 Sturmian BWTs and fires on 0% of Wikidata (PROOF-PROGRAMME §5.4). The SSL-Coq method
  says not to discard it but to ask, rung by rung, which of its conditions real blocks break, and
  what representation would restore its effect. The toy-transfer note's candidate is Ostrowski
  numeration, generalised to Rauzy–Veech and Bratteli–Vershik coordinates, for random access to
  position k; its guess at what breaks is run width, lost to random identifiers.
