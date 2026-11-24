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

## 9. Louis: "didn't we already have a whole theory about, like, obstructions, remember?"

**Yes: `docs/team/research-charter/FRONTIER.md`.** The Coordinator had not connected it in §§3–8,
which is the lookup failure §3.2 diagnoses, made again. What it holds:

- **A format for obstructions:** "When a failure exposes a significant obstruction, record three
  things under its row: the concrete **witness**; its exact **scope**: which algorithms,
  representations and cost model it binds; the **escape condition**: which assumption a different
  formulation would have to drop. An obstruction with no stated scope is not established."
- **Foundational questions (track 1)**, each asking whether a core operation is necessary:
  1. *Must the decoder follow an orbit at all?* "Is there a formulation in which output positions
     are computed rather than reached?" Known obstruction: text-order run labels determine the
     text (record 84), so "any 'placement' must be cheaper than the labels it implies".
  2. *Is the first return the right object?* Repeated cuts shrink the retained set fast
     (110,273 → 17,657 → 3,916 rows on block 0). "What is the limit object of repeated cutting?"
  3. *Can geometry replace rows?* "an exact reconstruction whose work scales with the number of
     structural events … and whose only n-scale work is sequential byte writing?"
  4. *Is the inverse BWT the right problem?* "Not yet examined by anyone."
  5. *Copies from completed output.*
- **Known structural obstructions** ("Do not rediscover these; build on them"), among them:
  - the accelerated induction floor r (Cells at 1.013–1.019r), which rules out better monotone
    induction schedules (records 79–80);
  - run labels determine the text (record 84);
  - Φ-copy fragments are cyclic (85–94% on dependency cycles), which rules out in-place
    write-once Φ replay;
  - **Gamma's G1:** `Phi_BWT` is L-complete under uniform TC⁰ reductions, which rules out O(1)-depth
    global coordinate formulas (and NC¹ unless NC¹ = L). It is marked "under audit";
  - 3-byte global sensitivity, which rules out local-context or UUID-landmark placement;
  - the density-ratio obstruction against symbol-level recompression and tunnelling at post-Open
    mean width 1.6–2.2;
  - cross-block width-1 indistinguishability.
- **Foundational formulations**, kept for their mathematics even when their first implementation
  loses (the direct BWT-space contraction algebra, Gamma's G1 and G2), each with what is known
  against it.

### How it bears on the formulation

- **It is the "negative space" the formulation needs, already half-built.** Each obstruction says
  what a new structural step can't be; its **escape condition** says what it must drop. A new
  algorithm must sit inside every escape condition at once. That is ChatGPT's "impossibility
  results as constructive operators" (§7), and FRONTIER already records obstructions in the form
  that use needs.
- **It constrains the Sturmian ladder directly.** The toy's closed form is an O(1)-depth coordinate
  formula. G1 (if its audit holds) says no such formula exists for the general BWT. So the walk-free
  inversion cannot generalise to all inputs, and must exploit the structure of the blocks it serves.
  G1's escape condition is the place to look: structure a general reduction doesn't have. Record 96
  and the census say what Wikidata's blocks do and don't have (width-1 runs a median 39% of retained
  rows).
- **Foundational question 1 is the formulation's question** ("output positions computed rather
  than reached"), and its record-84 obstruction is the first constraint on any answer.
- **Status:** much of FRONTIER's obstruction table is marked "under audit" or "conjectural", some of
  it is measured on six blocks, and none of it is machine-checked. A formal proof system would
  give these entries the status SSL-Coq's checked obstructions have.

## 10. Louis: "what are we drawing from it exactly, and where are we storing that durably? Is it going in FRONTIER?"

> "we were using the term escape condition, witness, scope, and so on, but surely we need to change
> it further to … align with what we've come up with, this Mario Eureka movement idea, right?"

**What already exists, in pieces** (found before answering, after §9's lesson):
- METHOD.md §3a's two axes: a *different primitive with a cost gain* is "a candidate
  bouleversement"; a different primitive without one is "mathematics worth keeping".
- FRONTIER's toy-transfer lead, sharpened since the note: "The capability to transfer is cheap
  global coordinate evaluation (or cheap composition of long orbit segments), not run width".
- FRONTIER's "Working backwards" lead: "oracle headroom first" — the Mario move, in one lead.
- FRONTIER's "Shape of the RH proof" lead: "the target as an exclusion statement over a family",
  never adopted.
- Obstructions with witness, scope and escape condition (§9).

**What the Mario work adds:**
1. **One target at the root.** FRONTIER has no statement that its leads, obligations and
   obstructions hang from. SSL-Coq's `MainTheorem` makes its open premises the agenda. Ours would
   be Louis's yes/no formulation, with each obligation, foundational question and obstruction
   wired to it (an open premise, a closed branch or an escape condition).
2. **Payoff first, as the default for reorientation.** §3a's first evidence for a reorientation is
   "a correct construction, and a cheap falsifier aimed at hidden acquisition". Mario reverses the
   order: supply the capability (the intermediate object the new mathematics would compute) and
   measure the headroom on real blocks, then build the construction.
3. **A ladder per toy or lead**, with each rung's status: supplied, partly native, native,
   machine-checked. Today a lead has a single word ("pending").
4. **Strength labels on obstructions:** bounded or global; checked, argued, or measured on N
   blocks. SSL-Coq kept these strictly ("bounded", "not a global impossibility theorem", unchecked
   proofs kept as unchecked).

**Where:**
- the *state* (target, premises, ladders, obstruction labels) in FRONTIER.md, with a "Target"
  section at the top left for Louis's formulation;
- the *procedure* (payoff first, supplied versus native) in METHOD.md §3a;
- the reasoning and these exchanges in this record.

**To clear first:** FRONTIER's steering log has "Methodology frozen until evidence arrives"
(decision 6d9acc87). Changing §3a and FRONTIER's format unfreezes it, which is Louis's call. The
Coordinator offered to draft the changes as one PR for his approval.

## 11. Louis: "the main idea I see is that it's like working backwards"

> "The main idea I see is that it's like working backwards, and we've been trying to do working
> backwards as well. Like you say, we already have that. This is the Mario move stated in one lead.
> And you see this in the document with phrases like 'What Mario Would Have To Do'."

**The transcript's phrasing.** A section headed "What Mario would have to do"; "the missing step is
a no-A way to get an already-high crawler a fresh low floor reading and move him into position
before that reading is replaced"; "work backward from the 923-unit vertical setup"; "Keep working
backward in Coq"; "The useful backward constraint is now checked: at the crawl stop, the remembered
floor height equals the fresh primary query's return."

**The Coordinator's distinction: backward from the input, or backward from the goal.**
- FRONTIER's "Working backwards" lead starts from the *text*: which classes of content own the
  expensive rows (the JSON skeleton cheap; GUIDs own 54–67% of width-1 rows). It is backward
  *diagnosis*: where cost comes from.
- SSL-Coq starts from the *success state* and asks what must have been true one step earlier, and
  repeats. It is backward *construction*: goal regression (Dijkstra's weakest precondition). Each
  step ends in one of three places: something produced natively (the chain closes); a named
  **missing step** (the next research question); or a proved obstruction (the branch dies). Its Coq
  proofs certify each backward link.

**The same move for mibz2, as an illustration, not a finding.** Start from "every output byte written
without walking" and ask what the decoder would have to know:
1. to write byte k directly, which row's symbol lands at k: each run's starting text position;
2. record 84 constrains this: text-order run labels determine the text, so the positions must come
   more cheaply than the labels they imply;
3. so it would have to obtain positions another way, for instance by summing return times along
   the induction's tree (FRONTIER's sharpened toy lesson: "cheap composition of long orbit
   segments");
4. either that closes on quantities computable from the run records within a cost budget — a yes,
   a new algorithm — or it stops at a named missing step, the next precise question.

That chain is what "a specification which we try to supply new mathematical approaches to fit"
could look like in practice, and the yes/no target is whether it closes.

**A status vocabulary from SSL-Coq.** Each result is one of: "a useful new lead", "a conditional
effect with an unproduced prerequisite", "a tested failure within a defined scope", or "an
established route". It is a better set of statuses for FRONTIER's leads than "pending".

## 12. Louis: "what a new algorithm for BWT inversion would have to do", distinct from incremental edits

> "So likewise we have this idea of 'What a new algorithm for BWT inversion would have to do' and
> this unit algorithm needs to somehow be distinct from incremental edits, which is the failure mode
> we have encountered, taking baby steps."

**The Coordinator's answer: forbid the obvious mechanism, as the A-button challenge does.** What makes
"fewer than one A press" force leaps is that it bans the tool every ordinary route relies on. mibz2
has an exact counterpart, with its floor already argued:

| | SSL-Coq | mibz2 |
|---|---|---|
| The obvious mechanism | the A press | following the orbit, one row at a time |
| The best known | one A press | Open: exactly h₁ single-row steps |
| The class's floor | ordinary routes need at least one | record 96: every rigid jump-walk route needs at least h₁ single-row operations |
| The challenge | fewer than one | **fewer than h₁** |

h₁ is the number of width-1 runs; h₁/m is 0.30 / 0.39 / 0.83 (min / median / max) across Wikidata's
distinct blocks (#805).

**Why this separates a structural step from an increment.** By record 96, no rigid jump-walk route
goes below h₁, and Open meets it exactly. So any inversion below h₁ is, by theorem, outside that
class: it cannot be an edit to Open. The distinction stops being a judgement ("is the old primitive
hidden in a helper?", METHOD.md §3a's orientation check) and becomes a consequence of the bound.
Increments to Open cannot meet the target even in principle.

**A candidate first line for the formulation:** invert the block with fewer than h₁ dependent
single-row operations. The backward chain of §11 starts from it.

**Three conditions:**
1. **The class must be defined precisely**, or a candidate can hide the walk in a helper. Record 96
   is proved only in prose; a Lean definition of the class and a checked floor make the guarantee
   real. This is the use of a proof system that the target needs.
2. **Fewer steps is not automatically faster.** A yes needs both: below h₁, and a payoff on real
   blocks within a cost budget (the supplied-state payoff, §3.4, gives the headroom before
   anything is built).
3. **It has teeth on the data.** At a median 39% of retained rows, h₁ is a large share of the work.

## 13. Louis: classify the routes, to fork abruptly rather than salami-slice

> "so as well as classifying every result we might perhaps want to sort of classify the routes so
> as to avoid forking down plausible subgoals which salami slice the gains until all we are left
> with is incremental edit to algorithms (the evolutionary approach), we want to fork more abruptly
> and full bouleversement"

**The mechanism of the trap, as the Coordinator reads it.** When the selection signal on each
intermediate step is "is it faster yet?", only subgoals that improve the current algorithm survive.
A route that leaves the class usually starts out slower, being new and unoptimised, and is pruned.
That is how the evolutionary efforts (Kitchen Sink, Wormhole) work: mutate an implementation and keep
what wins, which can only produce slices. SSL-Coq avoided it because its intermediate milestones were
judged by whether they discharged a prerequisite of the target, never by a score.

**A route classification.** Each fork states the class of its *best possible outcome*:

| Class | What it changes | Best outcome | Track |
|---|---|---|---|
| **R0, tuning** | the implementation; same algorithm | smaller constants | pot shots (the bench) |
| **R1, reshaping inside the class** | the representation or layout of the same objects; still a rigid jump-walk route | still ≥ h₁ steps (record 96) | pot shots. The salami trap: it looks mathematical but cannot escape (T², the lanes experiment) |
| **R2, escaping the class** | a different proof of inversion, hence a different algorithm (Curry–Howard) | below h₁; a new cost signature | the major shot |
| **R3, changing the problem** | what is computed at all (FRONTIER foundational question 4: using what the compressed stream already contains) | escapes the frame | the major shot |

**Rules proposed for the major-shot track:**
1. **Only R2 and R3 forks are admitted.** R0 and R1 ideas go to the pot-shot queue, however
   promising. Each subgoal is classified on entry, so the effort cannot drift from R2 into R1 one
   plausible step at a time.
2. **Every subgoal sits on the backward chain from the target** (§11, §12): a necessary condition for
   getting below h₁, not an improvement of the current route.
3. **Intermediate milestones are judged yes or no** (a premise discharged or not), never "faster
   yet". Speed is measured only on the supplied-state payoff at the start (is there headroom?) and on
   the finished candidate at the end (does it pay?).
4. **The class is decided by theorem, not taste:** "is the best outcome still ≥ h₁?" is answerable
   from record 96 once the class is formally defined.

**A second test, from the cost theorems (PROOF-PROGRAMME §5.9).** An increment changes the constants
of a route's cost formula; an escape changes its form, meaning the block statistics it depends on.
A finished candidate can be checked on the census: if its per-block cost still tracks h₁ as Open's
does, it has not escaped.

This is offered as the "what the agents may do" part of the formulation: admit R2 and R3 only;
every subgoal on the backward chain; progress as premises discharged.

## 14. Louis: "yes put fewer than h₁ to monodromy … this is gold"

> "yes put fewer than h₁ to monodromy. also please ensure we are recording these messages in
> journal - this is gold"

**Done.** CHECKPOINT to MONODROMY on #346 (see #145 for the routing), asking for inputs to Louis's
formulation, not the formulation itself:
1. a precise definition of the class record 96 bounds, tight enough that the walk cannot hide in a
   helper, with its escape condition;
2. whether "fewer than h₁" is well-posed: any trivial way to meet it that is still a walk; per block
   or over the corpus;
3. the backward chain from the target (§11), each step marked native, missing or blocked, checked
   against FRONTIER's obstructions (record 84, Gamma's G1, 3-byte global sensitivity, the induction
   floor r);
4. whether §13's route classes are the right cut, and whether "best outcome still ≥ h₁?" is
   decidable from record 96 alone;
5. whether a Lean statement of the class and the floor is the right first proof target, and its
   rough size, with nothing built until Louis decides on the repository.

**On recording.** Every exchange of this discussion is kept here, in order, with Louis's words
quoted (§§1–14). The record stays open on its branch (#808) while the formulation discussion
continues, and merges to master when it ends.

## 15. Louis: a prompt for Antigravity on the same question

> "can you give me a prompt for antigravity to explore the same question we just put to monodromy"

The Coordinator wrote it in MASTER-PROMPT.md's form, recommending **Gemini Neon, High**, on branch
`research/neon-fewer-than-h1`, working **independently of MONODROMY** (told not to read MONODROMY's
answer until it has committed its own), so that the two reads can be compared. Its content:

- **The goal behind the question:** a structural step to a new inversion algorithm, judged yes/no;
  the A-button challenge's lesson (forbid the obvious mechanism); record 96's floor (every rigid
  jump-walk route spends at least h₁ single-row operations; Open meets it); the candidate target,
  fewer than h₁.
- **Reading:** this record (§§1, 9, 11–13), record 96 (`docs/journal/monodromy/2026-11-24-guid-wall.md`
  on `claude/sharp-meitner-cx0v78`), FRONTIER.md's foundational questions and obstruction table,
  PROOF-PROGRAMME §§1, 2, 5, METHOD.md §3a and §6a, the toy-transfer note, the block census, and
  the Open kernel.
- **Questions:** (1) define the class precisely, with its escape condition; (2) is "fewer than h₁"
  well-posed, and is there a walk in disguise that meets it; (3) work backwards from the goal, at
  least two chains, each step marked NATIVE, MISSING or BLOCKED (by a named FRONTIER obstruction),
  one chain starting from the Sturmian toy and climbing its ladder; (4) are the route classes R0–R3
  the right cut, and is "best outcome ≥ h₁?" decidable from record 96; (5) a Lean 4 sketch of the
  class and the floor, with a size estimate. Small-instance exhaustive search welcome.
- **Deliverable:** a journal entry in METHOD.md §6a's form, claims tagged proved, verified, measured
  or conjecture, programs alongside; push the branch, report the SHA, stop after one cycle. Never
  touch `src/`, open PRs, merge, run long benchmarks or touch releases; no AI attribution lines.

## 16. Louis: unfreeze the methodology; is this sure to give a bouleversement, and not just RV again?

> "yes unfreeze the methodology and draft the frontier PR. can you get that so that neon has the
> latest to work from. also are you sure that this will now push it to give a bouleversement of a
> kind that we have not achieved? like how we went from traversal to RV induction? and RV induction
> will not just be the answer"

**Done:** PR #816 adds FRONTIER.md's "The target (the major shot)" (the candidate target, open
premises, route classes R0–R3), lifts the methodology freeze in the steering log, adds strength
and reach labels to obstruction rows, and puts working backwards, payoff first and result statuses
into METHOD.md §3a. It merges when green so that Neon reads it from master.

**The Coordinator's answer to the doubt: not guaranteed; here is what it secures.**
- **It should exclude Rauzy–Veech too.** FRONTIER's induction floor (records 79–80: no monotone
  induction schedule beats about r steps; Cells at 1.01–1.02r) and h₁ ≤ r put induction routes
  above the target. So "RV induction will just be the answer" fails the target by construction —
  *if* induction steps and single-row operations are counted in one currency. That condition was
  added to MONODROMY's assignment (#346 CORRECTION 7128643478) and belongs in Neon's work too.
- **It cannot guarantee a yes.** The answer may be no: Gamma's G1 already rules out constant-depth
  global formulas for the general BWT. A proved no ends the major shot cleanly, as an impossibility
  proof would end "fewer than one A press".
- **What it does guarantee:** no more salami-slicing. Any yes is new by theorem, and any no is
  definitive.

## 17. Louis: "Can we guarantee that we won't get big ideas that don't have any utility in the real world?"

> "Can we guarantee that we won't get big ideas that don't have any utility in the real world?
> Because that is another condition."

**The answer: ideas without utility can still be proposed, but none can count as a yes.** Utility was
written into the target (PR #816) as the third condition, beside correct and escaped, checked at
three points:
1. **Before formal work: cost-weighted applicability** on the census (#805) — the share of
   Wikidata's kernel time in the blocks where the idea's conditions hold, not the share of blocks.
   The two-letter Sturmian solver (exact on 5,627 Sturmian BWTs, firing on 0% of Wikidata) is the
   cautionary case: correct, new, and useless here.
2. **Before construction: the supplied-capability payoff** on real blocks through the shipped path.
3. **At the end: beating master on the bench** under PERFORMANCE-POLICY.md (faster beyond noise on a
   primary corpus, no primary loss). Fewer operations that cost more time is not a yes.

The thresholds for the first two are set before the data and are Louis's to fix with the
formulation. The same condition was sent to MONODROMY (CORRECTION 7128643478), asking it to mark
which backward chains could clear utility on Wikidata's measured structure.

## 18. Louis: in hindsight, Rauzy–Veech was always going to be cost-viable

> "We'll see, that's exactly the thing I'm talking about. As well as that, there's the cost aspect.
> Like, I think in hindsight we could see that Rauzy–Veech induction would always be like cost viable
> as opposed to the traversal, right?"

**The Coordinator's answer: yes, and the signal generalises.** The traversal pays one dependent,
cache-missing load per output byte, about n (around 900,000) a block. Induction made the cost scale
with the block's *structure*, its runs and then a walk over only the retained rows, and that
structure is small on real blocks: the census (#805) gives a mean maximal run of about 16 rows (about
n/16 runs) and a median m/n of 0.114 (about n/9 dependent steps, each copying about ten bytes). That
was knowable before building, by counting runs on real blocks.

**The general test:** a structural step is cost-viable when its cost scales with a *different*
parameter, *measured small on real inputs*: the cost theorems' "the form changes" (PROOF-PROGRAMME
§5.9) joined to the census. For any candidate: what does its cost scale with, and is that much smaller
than h₁ on Wikidata's blocks? If not, it may be beautiful and will not pay — Louis's utility condition
(§17), made predictive.

**What it points at for the next step.** Below h₁, the remaining expensive work is concentrated in
the width-1 runs, mostly random identifiers (h₁/m a median 0.39). A new algorithm must scale with
something much smaller than their count: FRONTIER's toy-transfer note already pointed at acting on
those runs collectively, through what they share (format, alphabet), not one by one. Whether such a
quantity is small on real blocks is now a measurable question, to be asked of every candidate.

Added to FRONTIER.md's target as "the viability check" (PR #816).

## 19. MONODROMY's answer on "fewer than h₁" (record 98)

MONODROMY's REVIEW on #145 (comment 7128634595; record 98,
`docs/journal/monodromy/2026-11-24-fewer-than-h1.md` at eae9ca91), offered as inputs to Louis's
formulation:

- **The class made precise: a calculus 𝓡 of "rigid facts".** A fact says LF^h(x) = x + t on an
  interval, with one emitted word. Facts come one per run, from its record, and are restricted,
  composed and joined. The LF walk, Cells (Rauzy–Veech), Open, lanes and T² are all in 𝓡. Its cost
  c₁ counts width-1 events *anywhere*, construction and helpers included, which puts induction steps
  and walk steps in one currency (the condition of §16). G1 (argued): c₁ ≥ h₁ for every decoder in 𝓡.
  **So Rauzy–Veech is inside the class: it cannot be the answer.**
- **"Fewer than h₁" is not well-posed as worded.** Three repairs: a *count*, not a depth (two-ended
  Open already halves the depth); a *streaming budget* K (sort-based list ranking has c₁ = 0 in
  Θ(log n) passes, a known and slower algorithm); and *per block over a stated family 𝓕*. Proposed:
  exact inversion of every column in 𝓕 with c₁ < h₁(column), in at most K streaming passes (fixed in
  advance, e.g. 3) and O(r) words beyond the output. K and 𝓕 are Louis's to fix.
- **The escapes** (drop "every fact is a translation on an interval"):
  - E1, class facts: blocked on random content (keys shatter by k ≈ 5);
  - **E2, positional facts: one open cell.** Blocked when labels come first (record 84), at O(1)
    depth (Gamma's G1) and as rigid segment composition; **open** for a global, non-rigid,
    non-O(1)-depth procedure in K passes, with neither a construction nor an obstruction;
  - E3, sort-based: native, but fails K;
  - E4, another input or problem: narrow, since a standard bzip2 stream carries only L and `origPtr`.
- **Conjectured:** on random-digit content (the GUIDs), no O(1)-pass decoder meets the target. A yes,
  if one exists, lives in E2's open cell on non-random content; the copy sector is a candidate, where
  E1's classes do group rows. The missing step: getting that grouping without each row's first step.
- **R0–R3, amended:** R1 = in 𝓡; R2 = a fact type outside 𝓡, within K. Decidable syntactically for a
  candidate written in the calculus; for Rust code it needs an audit mapping operations to rules.
- **Lean:** about 600–1,000 lines (facts and derivations, Lemma 3, G1's injectivity), second after the
  shared specification. A checked G1 makes the class boundary exact; it does not make escapes
  impossible.

**The Coordinator's reading for the formulation.** The search space for the bouleversement has been
narrowed from "anything new" to one named open cell: **positional facts by a global, non-rigid
procedure, within a few streaming passes, on non-random content**. Utility then turns on
cost-weighted applicability: random identifiers own 54–67% of width-1 rows, so a yes confined to
non-random content pays only on the remainder, and the census can say how much kernel time that is
before anything is built. The random-content conjecture, if it holds, says the GUIDs need a real R3:
a format change, not a new inversion.

Folded into FRONTIER.md's target section in PR #816.

## 20. Louis: beyond what anyone has thought of; the kitchen sink, but not at random; well-posed

> "Right, because we've got dependent loads as like a major kind of unit of work here. This is part
> of why we were able to consider the Rauzy–Veech change as being substantially viable. And it did
> indeed transfer well and became the new best. But again, like I said, part of what we're trying to
> get here is like an improvement to the mathematics that goes beyond what anyone else has thought
> of before, and we want to do this by rifling through this kitchen sink, but in a way that is kind
> of not just delving at random. And we also need this to be substantially well posed, which is, I
> think, what MONODROMY has said was the issue with our presentation."

**The Coordinator's answer.**
- **Well-posed, in one line** (record 98): exact inversion of every block in a stated family 𝓕, with
  c₁ < h₁ width-1 events counted anywhere (construction included), in at most K streaming passes.
  The count, not the depth; RV and the walk in one currency, both inside 𝓡; K so that the known slow
  sort-based method doesn't qualify trivially. K and 𝓕, with the utility thresholds, are the numbers
  the formulation still needs from Louis.
- **The kitchen sink, not at random.** Record 98 partitions the escapes into E1–E4 and finds one open
  cell: E2, positional facts by a global, non-rigid procedure within K passes, on non-random content,
  with neither a construction nor an obstruction. That cell is the filter: each framework is asked
  whether it supplies a global, non-translation positional coordinate computable in a few passes.
  Frameworks that only reshape (R1) are out at once. FRONTIER's leads already hold candidates of
  that type: Ostrowski and Bratteli–Vershik numeration; Rauzy fractals and the Pisot setting; Veech
  groups on the origami; discrete Morse gradient-path labels; Agol–Hass–Thurston interval pairing.
- **Beyond prior work:** nobody has a construction or an obstruction in that cell, so a yes there is
  new by the record, not a known slow method.
- **The utility caveat:** the cell is open only on non-random content, and random identifiers own
  54–67% of width-1 rows. A yes pays only on the rest, which the census can price before anything is
  built. If the random-content conjecture holds, the GUIDs need a format change, not new mathematics.

**Proposed next step:** a focused kitchen-sink cycle (Carbon or Neon) screening each framework
against the E2 cell, then the survivors' cost-weighted applicability on the census, once Louis has
fixed K and 𝓕.

## 21. MONODROMY's own account, as Louis saw it in its session

Louis relayed MONODROMY's session summary of record 98. It adds, in plain terms:

- **A gap it caught in its own proof before committing:** "without counting the output chain's links,
  the LF walk would score c₁ = 0, so I'm adding width-1 chain links to c₁ and redoing the injectivity
  argument around join points." The cost now counts "every operation on a single row anywhere in the
  program, including construction, helpers and each step of the output chain."
- **"Two existing algorithms already meet it"** as originally worded: two-ended Open, if read as
  depth ("only about h₁/2 single-row steps on the critical path"); and sorting-based list ranking
  (Chiang et al.), with no single-row steps at all, "known and slower, needing a logarithmic number of
  passes over the data". Hence the repairs: a count, not a depth; a pass budget K ("I suggested 3");
  per block over a stated family ("a corpus total could be met on easy blocks alone").
- **Where a yes would have to live:** "on non-random content such as repeated hashes and numbers.
  There the missing step is learning the grouping without each row's own first step."
- **On Lean:** "A checked version would make the class boundary exact; it would not rule out the
  escapes."

**Still Louis's to fix:** the pass budget K, the family of blocks, and the utility thresholds.

## 22. Louis: two work streams, random and non-random, and the step change as the challenge

> "I think we don't want to limit ourselves as well with the random element, the non-random element.
> Well, I mean, we probably want to separate those two, don't we, into two separate work streams, and
> establish what can be shown for each of them, best case and worst case. But then together we want
> the whole thing to work well, don't we? So this is part of the task and part of the formulation. But
> if we need to, establish a direction by kind of not considering one part and then bringing it in,
> and that is of course an appropriate way to work. But at the end of the day, we do need to establish
> an application in the real world, of course. But like I say, we have this example of the traversal
> to Rauzy–Veech induction step change, which is what we want to formalize as something we want to
> do, right? And that's hopefully what we're moving towards, and hopefully what MONODROMY has helped
> us formalize as a task, a challenge."

**The Coordinator's answer.**
- **The split is by row, and it composes.** Blocks mix both kinds of content, and the width-1 count
  splits with them: h₁ = h₁(random) + h₁(non-random). The whole-block target (c₁ < h₁) is met if the
  non-random stream gets below its share while the random stream holds at its own, so the streams
  can be worked separately and their results add up.

| | Stream N: non-random (repeated hashes, numbers, copied text) | Stream R: random (the GUIDs) |
|---|---|---|
| Where it stands | record 98's one open cell (E2: positions by a global, non-rigid procedure within K passes) | conjectured closed (no O(1)-pass decoder) |
| Best case | a construction whose cost scales with something small on real data (the Rauzy–Veech pattern again) | the conjecture fails, or a cheap format-level change works |
| Worst case | an obstruction proved here too | the conjecture proved: a floor for random content; only a format change helps |
| Share of the prize | about a third to a half of width-1 work (GUIDs own 54–67% of width-1 rows) | the rest |

  Working one stream while setting the other aside is a legitimate way to find a direction, with
  the combination checked at the end on whole real blocks.
- **The step change, formalised as the challenge.** The traversal → Rauzy–Veech step was (1) a change
  of class, a different proof of inversion; (2) a cost that scales with a different parameter, small
  on real data (dependent steps from n to about n/9 on Wikidata); (3) a result that held on real
  blocks and became the new best. The challenge is to do it again: leave the class 𝓡 (which now
  contains the walk and Rauzy–Veech alike), with cost scaling with a parameter small on Wikidata's
  blocks (below h₁, within K passes), holding on real blocks against master, worked as two streams
  whose best and worst cases are partial or definitive answers. Those three conditions are the "yes"
  of FRONTIER's draft target (PR #816).
- **Remaining for the formulation:** K and the family 𝓕 (Louis's), and writing the two-stream split
  into the target section, offered as a draft for Louis to edit.

## 23. Louis: "Do you think that then we should have a main theorem like the Mario thing did?"

**The Coordinator's answer: yes, and now we can.** Record 98's calculus makes the challenge precise
enough to state as one.

- **What SSL-Coq's `MainTheorem` is:** not "a route exists" (unknown), but a conditional — if premises
  H₁…Hₙ hold, then no-A spawning displacement is reachable. The premises are the agenda; each session
  discharges one or proves an obstruction; an audit checks that results are wired in and nothing is
  assumed silently (no axioms, no `Admitted`).
- **Ours, in the same shape.** *Main theorem (the challenge):* for the block family 𝓕 and pass budget
  K, if premises P₁…Pₖ hold, then there is a decoder outside 𝓡 that inverts every block in 𝓕 exactly,
  within K passes, with fewer than h₁ width-1 events.
  - *Premises = the two streams (§22):* stream N, a global positional procedure for non-random rows
    whose cost scales with a parameter small on real data (the open cell); stream R, the random rows
    handled at no more than their share of h₁ (the existing walk does that); and a composition lemma,
    c₁ = c₁(N) + c₁(R).
  - *Closed branches = the obstructions:* G1 (anything in 𝓡 costs ≥ h₁: the walk and Rauzy–Veech);
    record 84; Gamma's G1; the random-content conjecture, once proved.
  - *A "no":* proving the open cell's premise false gives the main theorem's negation for that budget.
- **What stays outside it:** utility (wall time on the i9) cannot be proved, as SSL-Coq's original-ROM
  playback was a separate receipt, not a theorem. The full yes is: main theorem discharged +
  cost-weighted applicability on the census + beats master on the bench — the first formal, the other
  two measured, never confused.
- **How to start:** (1) now, on paper — MONODROMY writes the main theorem, its premises and the
  composition lemma as a new PROOF-PROGRAMME.md section in its pseudo-Lean, and FRONTIER's target
  points at it, so every agent attaches results to one statement; (2) later, in Lean — a `mibz2-lean`
  repository with the specification, the calculus 𝓡, G1 (about 600–1,000 lines), the main theorem with
  premises as explicit hypotheses, and a CI audit that none is assumed silently (new infrastructure,
  Louis's approval). Recommended: step 1 now, step 2 once the statement settles.

## 24. Both agents on one challenge, with crossed streams

**Louis:** "shall we now put the same challenge to both monodromy and neon and see what we get?"

**Louis:** "Should we perhaps encourage them to like cross their work streams in such a way that they don't overlap, but like are encouraged to like read what the other is saying at the same time by pulling in the branch from the other? This would kind of make them co-coordinators and mean that they don't have to communicate, but they could like swarm on the blocker themselves … mine all of the resources that have been established and really go hard on this problem and potentially make the breakthrough in collaboration."

**The design adopted.** Generation is collaborative and verification adversarial:
- *Split by stream.*
  - MONODROMY owns the main theorem's statement and stream R (random content).
  - Neon owns stream N (non-random content, the E2 open cell): it screens the kitchen-sink frameworks for escape, passes, viability and payoff, in that order.
- *Crossed reading, no messages.* Each fetches the other's branch at the start of every cycle and builds on it with citations.
- *Signals:*
  - `CLAIMS.md` lines `STARTED`, `PUSHED`, `DONE`, `DROPPED` (and `NEEDS` for a measurement MONODROMY wants run);
  - tagged commit subjects (`claim:`, `result:`, `premise:`, `break:`, `read:`);
  - a push after every result;
  - one REVIEW per cycle on #145, or on the branch under `channel/` if posting isn't possible.
- *Adversarial checks:* each tries to break the other's claimed proofs and constructions.
- *Provisional parameters:* K = 3, 𝓕 = 232 Wikidata blocks plus the six admitted blocks.

**Routing slip.** The Coordinator posted the challenge to MONODROMY's inbox, #346 (7128808679), while the router counted the role busy. It then withdrew that post and staged the challenge on #145 as QUEUED (7128837429). The role was idle; it counted as busy only because of unanswered Coordinator posts it had never been woken for. The fix was a direct message naming each item.

**Louis, on moving MONODROMY to a costlier model:** "please ensure you therefore encourage it to be economical and do not ever piddle around with things the coordinator or an implementation agent should do … focus only on mathematising." MONODROMY was told so: mathematics only; measurements requested via `NEEDS`; minimal signals.

## 25. No consent-seeking; the outside review; the first cycles

**Louis:** "ok i want chatgpt pro to engage with this exercise, can you give me a prompt that will help me give a solid response to the AIs? also, I want them to not lean on me for consent like this, I want them to determine the fullest sense of the challenge (as opposed to the 'cheat' or 'reward hack' that stacks the deck, as discussed)"

**Louis, on reaching Neon:** "i do not want to interrupt neon, you should have included a way for it to pick up messages via durable repo mechanisms, please use that". The mechanism adopted is `channel/coordinator-to-neon.md` on MONODROMY's branch, which Neon reads each cycle. It is append-only, and the Coordinator writes it.

**Louis, on cost:** "NO DO NOT ASK MONODROMY TO DO MUNDANE COORDINATOR TASKS YOU DO THEM IT IS V EXPENSIVE TO ASK IT TO DO THAT"

**Louis:** "can you create a sibling session and seed it as the new monodromy and decommission current monodromy as an opus variant of monodromy and we will use the new fable variant of monodromy?"
- The old session wrote a handoff, `docs/journal/monodromy/HANDOFF-2026-11-24.md` (dd1d6eae), and was archived.
- The new session is `session_0187sPR723oFTKXtty6qzCWK`.

**Louis:** "Please act on info that comes in asap, as cycles of monodromy / neon may rely on the files and chatgpt pro will take a while to return fully. We ought amend durable records so the swarm sees"

**The parameters the agents fixed themselves** (MONODROMY, 4d7d83b4; Neon adopted the same):
- 𝓕 = all 238 blocks, judged per block;
- δ = ½;
- K = 2;
- state at most Open's peak scratch;
- anti-cheat clauses: all facts count, no bundling, no hidden costs, count rather than depth, never relaxed.

**MONODROMY's cycle 1** (record 99):
- Rauzy–Veech, list ranking, contraction and class facts are all inside 𝓡.
- Only the global E2 cell is open.
- The target needs a margin δ.
- The main theorem is drafted.

**Neon's cycle 1** (#145, 7129375601):
- *The measurement:* h₁R/h₁ = 0.80–0.85 on six blocks, so δ = ½ is unreachable unless stream R beats 1. 85–89% of h₁ rows are unique at window 8.
- *Four breaks against MONODROMY:*
  - G1 is false on arbitrary columns;
  - Pmargin leaves out glue, and measured switches are 1.2–1.6 × h₁N;
  - Mirror attribution and c₁(Open) are inconsistent;
  - record 100's bound holds only in expectation.
- *Five frameworks closed:* Ostrowski/BV, Rauzy fractals, Veech, Morse, AHT.
- *The ceiling:* stream N can save at most about 3–9% of single-thread kernel time.

**The outside review** (ChatGPT Pro, commissioned by Louis). It's kept in full in [`2026-11-24-algorithm-level/outside-review.md`](2026-11-24-algorithm-level/outside-review.md). Its main points:
- c₁ is a statistic of the calculus, not machine cost. The LF² result of Kärkkäinen, Kempa and Puglisi shows that singleton compositions need not cost singleton misses. The proposed fix is a conjunct on dependent misses.
- A finite family admits a lookup table, so the domain should be a uniform family.
- σ_hex isn't randomness.
- Pmargin leaves out glue; it repeats Neon's break.
- G1 is a scoped provenance theorem for now.
- The proposed "no" leaves out global E2.
- Three leads for stream N:
  - acyclic positional copy plans;
  - compressed SA/ISA differences;
  - ordered return-word hierarchies.

It was relayed on #145 (7129531047) and written into the swarm's channel (e48f9044). The two independent reviews converged on two findings: glue is missing from the margin, and G1 fails on arbitrary columns.

## 26. "Against Mathematics": reading it porously, and the eureka it gave

**Louis:** "what can we draw from this piece? chart its narrative first, then read through with a porous mindset, connecting freely and liberally, then finally move into a eureka movement". The piece was doomslide, "Against Mathematics: a religion of the dead, by the dead, for the dead" (Substack, 24 October 2026), pasted in full.

**Its spine.** Mathematics has three heads, each running at its own speed:
- *the proud head:* discovery;
- *the blind head:* verification;
- *the slow head:* understanding.

When one outruns another, something dies: Jaffe–Quinn's "dead area", Thurston's "evacuation". The founding schism is the Pythagoreans':
- *the Hearers* kept the Sayings, "what is true", and threw away the "why";
- *the Learners* kept both.

From there the essay runs through Hilbert's paper god (Gödel the day before the radio broadcast), the proof assistants, and the oracles. Its three "boats" retell Hippasus:
- **I:** the triangles are taken on the god's word, and checked with rope;
- **II:** a brother sees a triangle that isn't on the wall, the generating rule that keeps the invariant;
- **III:** anthyphairesis, a remainder that never stops, so no common unit exists. Theaetetus then names the whole class with one word, "power".

The verdict: results with proofs and no reasons, "the gate stands between the results and their reasons", "the only way through is down".

**The eureka, as an action.** The essay's warning fitted the Coordinator's own last move. Entry #3 relayed the outside review's proposal to change the target's currency to dependent cache misses, without checking our own records. The records refute it on this machine:
- toy-transfer, 20 November: lanes bought only 5–14%, at about 10 bytes appended per retained step;
- T², 23 November: KILLed.

That is the retrospective's §2.4 failure ("we already had the book") repeated within hours, by taking a confident oracle's result without the why. The correction went to the swarm as entry #4 (0acb4187), with a direct note to MONODROMY.

**The eureka, as mathematics.** G1 floors the *count* of width-1 events. On this machine the emit's cost is count × *unit price* (work per event) + bytes + construct, and nothing floors the unit price. So the question splits cleanly in two:
- *(a) the frozen target:* fewer events. The evidence is converging on a scoped "no".
- *(b) a residual R3 target, labelled as such:* the same number of events, each much cheaper.

Following the essay's Theaetetus move, the swarm should stop closing frameworks case by case. It should name the invariant every closure shares (rigid, or blind to the flips), and ask what cheap thing *does* separate the flips. The flips are local: reading three bytes separates each one. That points (b) at fine-grained, local, batched work on width-1 rows. That is Louis's dependent-loads intuition in its honest form: not latency, but the price paid per serial unit of work.

## 27. ChatGPT Pro's reading of the same essay, and Neon's last cycle

Louis relayed ChatGPT Pro's reading of "Against Mathematics". It's kept verbatim in [`2026-11-24-algorithm-level/against-mathematics-chatgpt.md`](2026-11-24-algorithm-level/against-mathematics-chatgpt.md).

**Where it converges with §26:**
- the width-1 run "is a feature of one representation, not an irreducible atom of computation";
- Hilbert's 24th problem, read as "study the transformations between algorithms";
- Lean: keep cost on executable witnesses, apart from correctness proofs.

**What it adds:**
- **The ordered-frontier lemma:** LF on a sorted set, then a stable partition by character, gives a sorted set again. With it, *collective* transport of the random fields: one word-carrying macro-edge per GUID, built in d synchronous phases.
- **A supplied-state ladder** for testing it.
- **"Transferable insight"** as a fourth field on obstructions.
- **Terminal against generative mathematics.**

**Placed in the formulation.** It is a candidate for the residual target (b): the same events, a cheaper unit price. It is not a yes for the frozen target, under the no-bundling clause. It was sent to the swarm as channel entry #5, with the T² check to do first.

**Neon's cycle 3** (#145, 7129781415) was its last:
- 14 of 14 closed, including the three leads in the outside review's §5;
- G1's scoped proof (record 103) verified with no breaks;
- Open's L2 misses simulated at 0.42–1.20 × h₁, with no L3 capacity misses at 10 workers. That agrees with the work-bound correction.

It stopped, noting that only MONODROMY's work on stream R and the 232-block corpus run are left.

## 28. MONODROMY's answers, and the second outside round

**Louis:** "ok, send your current set of records to the MONODROMY (Fable) sibling session to take into account … provide an interpretation … encouraging it to improve the process as it sees fit". Then: "i heard that monodromy replied... can you amass a large prompt for chatgpt to return to process, directed similarly to how my last prompt stage managed its movements".

**MONODROMY on Fable, cycles 4–8** (#145, 7129589449 to 7130227614; records 103–106):
- **Record 103:** G1 is a scoped provenance theorem, proved in prose.
- **Clauses added (stricter only):**
  - *6b:* dependent misses no worse than Open's;
  - *7:* no advice, a uniform family 𝓖, with 𝓕 for validation.

  Kept as guards; the machine test stays "beat master on whole-decoder time".
- **Record 104, the ordered frontier:**
  - it is inside 𝓡, with c₁ = G·d;
  - the anchors are cheap;
  - as a unit-price witness it is "predicted no": lanes with a different address order;
  - it is Open's contraction under a round schedule;
  - LF on a scattered set is a set of gap counts, the same object as the aggregate cell.

  Hilbert's 24th for walk → Cells → Open: contraction along compose, with the width-1 rows J₁ as its invariant.
- **Record 105:**
  - G1 survives abbreviation and batching, breaking only for a *schematic* fact: one proof, parametric over a scattered set. That is the open cell stated as proof complexity.
  - c₁ and machine time are incomparable.
  - The CRC is inadmissible for an exact theorem.
- **Record 106, Theorem N:** a scoped no over a declared class Λ* that includes the block-histogram part of global E2. It is conditional on two counts not yet produced (NEEDS M-4, M-13). Its residue is schematic facts over scattered sets with fine input.
- **PROCESS M:**
  - name the operation before screening a framework;
  - a closure must exhibit a simulation that preserves cost, or be recorded "unplaced";
  - a no is one theorem over a declared class, with a "does not say" section;
  - obstructions carry "transferable insight".

**The second outside round:** a five-movement prompt for ChatGPT Pro (chart, audit, porous reading, eureka, messages), pinned to commits. It's kept in [`2026-11-24-algorithm-level/chatgpt-round2-prompt.md`](2026-11-24-algorithm-level/chatgpt-round2-prompt.md).

## 29. The second outside round, and the Coordinator caught reading half the book

Louis relayed ChatGPT Pro's second-round answer ("woo a lot going on in this chatgpt response..."). It's kept verbatim in [`2026-11-24-algorithm-level/chatgpt-round2-response.md`](2026-11-24-algorithm-level/chatgpt-round2-response.md), with verification notes.

**The finding that matters most is about the Coordinator.**
- *What happened:* §26's "eureka" corrected the swarm's currency by citing toy-transfer's early negative: lanes bought only 5–14%. It concluded that the emit is bound by work, not latency.
- *What the file actually says:* the same file, at L187–208, corrects that negative. Once appends are branchless, 8 lanes cut the emit to 0.55–0.78×.
- *Where the error went:* it travelled into channel entry #4, into MONODROMY's records 104–105, and into draft post #820.

It is the same failure as the retrospective's §2.4, and the essay's Hearer error. It was committed while accusing an oracle of the same thing, and caught by that oracle on its next turn.

**What was done:**
- #820 is held.
- Channel entry #6 withdraws the claim.
- The round's witnesses were verified before relay: the Theorem N counterexample, the flip repair failure, and the N12 bug.

**Other results of the round:**
- Theorem N is downgraded to a proposed scoped theorem, with a missing mixed-rule induction.
- The first moment Σ i·L[i] separates every flip but decodes nothing.
- A schematic fact needs an executable Build/Advance/Emit interface; a short proof is not enough.
- The frozen yes must remove most of W, not only h₁.
- The next test is a supplied-state experiment: Open's word codes in output order, timed natively.

## 30. The supplied-codes test: the emitter halves, the decoder doesn't move

Louis approved commissioning the outside review's proposed experiment. The recipe `bench-supplied-codes` (#821) was built, reviewed and queued (#822), with its rule frozen before data (LOG 7131301798). The workstation ran it on the i9 (`supplied-codes/run-20261124T131518Z`, evidence e45314e0).

**The results:**
- *Supplying Open's word codes in output order, free, roughly halves the emitter:* 0.53 at 1 worker, 0.55 at 10.
- *At 1 worker* that takes about 16% off the whole decode (0.838).
- *At 10 workers, the default,* the whole decoder barely moves: **0.988 [0.957, 1.020]**. The verdict is **STOP**.

**The reading.** At ten workers the decode is bounded outside Open's emit, by the serial front end or by shared bandwidth. So the search's "price" question has a second floor beneath it: a structural step inside the kernel pays in wall time only where the kernel is the bound. Verdict LOG 7132792060; channel entry #9.

## 31. A new MST result, and the literature on packed inversion

**Louis:** "Does this brand new discovery help us fundamentally in any way? And might we look for new significant proofs in fundamental algorithms in recent literature?" The discovery is arXiv 2611.31187, a deterministic O(m) MST. Then: "Do whatever you think."

**The MST result, read for structure:**
- It doesn't help directly: inversion is already linear.
- Two of its mechanisms are structurally close to ours:
  - an offline-batch versus online separation (Komlós against Pettie);
  - contract-then-filter recursion.
- They gave MONODROMY one question (channel entry #10): is there a "supported row", whose placement is certified by structure Open has already built?

**The directed sweep** is kept in [`2026-11-24-algorithm-level/packed-inversion-sweep.md`](2026-11-24-algorithm-level/packed-inversion-sweep.md):
- No packed or sublinear BWT inversion is known, and none is posed as open.
- The best known is Θ(n) on integer alphabets and Θ(n log n) in the comparison model.
- At bzip2's size, the asymptotics offer nothing.
- libsais 2.12 (October 2026) now does cyclic, bzip2-style inversion with bigram LF. It's engineering prior art to benchmark against.

**Also decided:**
- The retrospective's §2.4 gets a dated correction (#824).
- The 10-worker bound diagnostic is deferred.

## 32. Back to the original intent: construction, and a rubric fixed before the output

**Louis:** "all this work … from the Mario thing to the Doom slide piece … was all meant to reframe the entire effort towards asking a question that was sufficiently powerful to unlock something that would give us an entirely new basis for the BWT inversion rather than an incremental gain." Then: "it's no longer a matter of asking the models to formulate the problem, but rather to begin taking a pop at solving it in this kind of proof formalized way … lower the floor more accurately than raise the ceiling."

**The Coordinator's diagnosis.** The reframing was used to *police* candidates (fourteen screens, lower bounds, a rigorous target), never to *generate* them. Record 108 then showed that the frozen binary target can't tell discovery from bookkeeping. Channel entries #13 and #14 redirect both agents to construction, with three conditions for a new basis, modelled on walk → Rauzy–Veech:
1. a new object, not rows and not intervals of rows;
2. a cost law in a parameter that is small where runs fail;
3. acquisition from the run records without an LF walk.

**The plan, as Louis set it.**
- Two Antigravity chats run to completion: the Neon-role and Carbon-role prompts, both on the Gemini 4 Carbon model.
- The Coordinator critiques their output against the rubric below.
- A fresh pass on the Gemini 4 Neon model is then built from that critique. It avoids what Carbon reproduced, and attempts a proof-formalised construction from a different direction.

**The rubric (fixed before any output).** For each candidate, mark each criterion met, partial or not, with evidence:
- **R1 New object.** The decoder acts on something other than rows or intervals of rows, and the object is defined exactly.
- **R2 A new cost parameter.** Its cost law is stated in a parameter that is small exactly where runs fail (novel random content), and the parameter is measured on the six blocks.
- **R3 Acquisition without the old mechanism.** It is built from the BWT/run records in O(1) streaming passes, with no row-by-row LF walk hidden in the build. The instruction set is declared.
- **R4 Floor lowered, not relocated.** Below G1's floor in a declared execution model, with no cost moved into an uncharged build pass (record 108's convention).
- **R5 Proof form.** Stated as a theorem with premises, each marked proved, measured or open. A named near miss says exactly where its cost goes.
- **R6 Not incremental.** Route class R2 (escape) or R3 (reframe), not R1 (reshaping inside 𝓡). It is not a restatement of the walk, Cells, Open, T², lanes or the frontier.
- **R7 Utility.** A supplied-state payoff stated in whole-decoder terms, or an honest statement that none can yet be given.

**Verdict.** "Starts to overthrow induction" means R1–R3 are met and R4 is at least partial. Anything less is reported as such, with what it would take to close the gap.

## 33. The council, and the currency decided

Louis asked for "a sort of council approach" to firm up the formulation before any construction. Three members answered one brief independently (`council/BRIEF.md`): MONODROMY (record 111), Neon (`research/council-neon`) and ChatGPT Pro (`council/chatgpt.md`, verbatim). `council/COMPARISON.md` compares them.

- **Where they agree:**
  - A charge means something only against a declared instruction set.
  - A basis is a representation with soundness, a parameter defined from the input, and a cost law that includes acquisition.
  - The split into easy and hard rows leaves the theorem.
  - A "no" quantifies over every program within the budget.
  - Whole-decoder time is a separate empirical receipt.
- **The decisive point (ChatGPT's):** G1 is a floor on a proof calculus, not a machine floor. Claiming "below G1 on a machine" first needs a bridge theorem: every decoder of the old class costs at least κ·h₁ in the declared currency. None exists yet.
- **The currency:** Neon's all-instruction currency is degenerate, since r ≥ h₁ forces word-packing. The adopted structure is ChatGPT's, with MONODROMY's cost vector (Q, D, P, S).
- **Louis's decision.** Asked what "lower the floor" means, he chose real machine cost: fewer unpredictable memory reads and scans than one per lone row, on a declared machine model, backed by the bridge proof, with the old bookkeeping target kept as version 1. The Coordinator had asked him in plain terms, after he said the shorthand was unreadable to him.
- **Next:**
  - MONODROMY attempts the bridge for 𝓡, and the specification Π is fixed before any candidate (channel #18).
  - The Neon-model construction pass starts from the agreed formulation.

## 34. The essence, restated: change what "moving together" means

Louis questioned the order adopted in §33 (the rules, then the bridge, then construction): "I'm a little concerned that you're kind of over-indexing on machine instructions … we could have come up with Rauzy-Veech induction without overly concerning ourselves with machine cost … The whole Hilbert 24th problem thing was that we should be able to see how we went from the first algorithm version with the traversal to the induction version and kind of see what was invariant, and we said this ourselves, it was contraction along compose … this sort of direction of travel here, which we were going to try and generalize."

The Coordinator agreed. The cost rules are the referee, not the generator. The generative step is to change what "moving together" means. Both Antigravity chats' rounds had just failed in exactly this way (the two critiques in `2026-11-24-algorithm-level/`).

Louis then: "I just don't want to lose this essence, because it's really powerful and we need to retain it. Otherwise we're just going to keep going back down to performance tuning, incremental improvements, partial results. Enough of partial results, we need to do a breakthrough. And when I look at this, what I see is a kind of a looseness of language, a generalization, work smarter, not harder, rather. Find a different sense for something that can occur. It requires a kind of ability to think flexibly, laterally. And yeah, the fact that we've tried so many things already doesn't mean that that's, the set we're limited to … it's not necessarily the case that an idea we tried has to be disallowed because it's already been tried, but rather that we can perhaps have, think again and redouble our efforts if we think there might be something in it. But yeah, we shouldn't, feign surprise at novelty if the thing turns out to already exist."

**Recorded as `2026-11-24-algorithm-level/ESSENCE.md`.** It is the starting page for every brief, prompt and review. It supersedes the order of work in §33; the currency decision of §33 stands, as the referee. Channel #20 carries it to the agents.

**All directions, overloaded (Louis, the same evening).** Asked which direction should lead, Louis chose none: "No, I want to do all of them. I don't want to frame it as one way only … the term being like generalization, perhaps you might say, is the term, the Hilbert's 24th problem movement. And however you call it, we overload it, right, with the multiple ways in which it might occur. The recurrence, the moving together, the contracting along the text. Like, we want to look at all possible ones. In fact, we might want to look at even more." ESSENCE.md now says so; channel #21 carries it.

## 35. Louis's JAX idea, an outside verdict, and the cross-check

**The idea.** Louis had rewritten page-dewarp in JAX and got it many times faster. He asked whether that holds an actual, reusable change of approach for mibz2. He put the idea to another model, then gave it the Coordinator's vetting prompt. Both exchanges are recorded verbatim in `2026-11-24-algorithm-level/jax-idea-verdict.md`.

**The outside verdict.**
- Associative scan, list ranking and the permutation scan for MTF: solid but incremental, and already in the literature.
- Composing k-letter words through monotone occurrence cursors (a k-step LF built from the column): "needs one missing piece", namely cheap *selective* macro-symbolisation that leaves runs in their contracted form.
- What it would keep: "look for transformations that are secretly the same computation, expressed in different coordinates."

**The cross-check against the records** (the Coordinator):
- **The JAX instinct is the essence itself.** Composing transformations instead of stepping states is contraction along composition. The phrasing is added to ESSENCE.md.
- **Associative scan and list ranking for inversion** are Edwards–Vishkin, cited in the expert review of 29 October and in the Rauzy–Veech record of 10 November. Record 109 shows why they don't move the floor: list ranking lowers the depth, not the count of reads. Landmark induction (record 114) is the same trade.
- **MTF as composition of chunk permutations** was studied on 30 October, in external reports 5–7 ("the algebra of inverse MTF", the (O, P) chunk decomposition, the bzip2gpu source).
- **The strongest candidate, k-letter words through monotone cursors, has been tried three times:**
  - T², killed on 23 November;
  - MONODROMY's senses A and B (record 113): classes multiply as 16^k, and depth k costs 16^k cursors or k sweeps;
  - **Carbon's C-2 earlier today**, "subalphabet radix-fibre k-cones". It is exactly the selective, hex-only variant, built through cursors. Measured on the six blocks, the wide-run side plus the macro side cost at least 1.50·h₁ on every block, with 64–137 passes and 61 MB of state (`carbon-cycle1-critique.md`).

  The outside model's "missing piece" is therefore the experiment already run, and it came out negative.
- **The analytic 0.379 reads per lone row** at k = 3 assumes free acquisition and no wide-run side. The measured version is the 1.50 above.
- **What stays genuinely open from this exchange:** a single algebraic representation shared *across* stages (Huffman, MTF, BWT, emit). It is the only route around the whole-decoder cap: the inversion is about 16% of time at one worker and almost nothing at ten. Ordinary loop fusion of Huffman, MTF and construction was tried and failed; a common representation was not.

## 36. What page-dewarp actually did: expose hidden information, then switch algorithm

Louis asked the other model to read lmmx/page-dewarp's history and find where JAX came in and how it helped. The Coordinator checked the dates and PR numbers against the git history:
- #130, autodiff solver: 8dac8bd, 9 December 2025;
- #135, `maxcor` from 10 to 100: bc279a0, 10 December;
- #139, vmap batch: 7f52aa3, 11 December.

**What the history shows** (the other model's reading, condensed):
1. **#130** rewrote the projection in elementary JAX operations, replacing OpenCV's opaque `projectPoints` and `Rodrigues`. `value_and_grad` then supplied gradients, so SciPy could swap Powell (derivative-free) for L-BFGS-B. That gave 3.7–9.6× per image.
2. **#135** kept 100 past gradient pairs instead of 10: a richer summary of past work, so fewer future steps. Up to 16.7× cumulative, with 357–722× fewer objective evaluations.
3. **#139** vmapped the whole optimisation over a padded, masked batch: 4.1× more on 40 images. The GPU was slower until batching.
4. **The negative results:**
   - Levenberg–Marquardt needed far fewer evaluations but was slower overall.
   - The GPU was slower at first.

   Both are the same trap as here: a better abstract count that is not a better computation.

**Its closing line:** "You didn't accelerate the original algorithm. You exposed hidden mathematical information that made a better algorithm possible, and then accelerated that algorithm."

**The mapping onto this programme** (the Coordinator):

| page-dewarp | mibz2 |
|---|---|
| Powell observes one point per evaluation | the walk: one row per step |
| A derivative tells you about the neighbourhood of the point at the same cost | Rauzy–Veech: one step tells you where a whole interval goes. That is the "derivative" already taken |
| Richer history (`maxcor`) | reuse of past work: copy plans; Kärkkäinen–Puglisi cache-friendly inversion |
| vmap | the interleaved chains and lanes already in the decoder |
| LM: fewer evaluations, slower | T², and record 108's trap |

**The question it leaves, in the essence's terms.** What *other* information does one LF step carry, at no extra cost, beyond where its own row goes, and beyond where its run goes? It would have to be the discrete counterpart of a gradient taken along a different coordinate, one that enables a different algorithm.

**Caution from the records.** Order-free aggregates of the column, such as the first moment A = Σ i·code(L[i]), carry information that "separates every flip and decodes nothing" (N-18). The analogue sought is information about *the map's behaviour*, not a fingerprint of the input.

## 37. Enrichment along composition: the second principle

Louis, to the other model: "the one other thing that strikes me is that we had a gradient based problem… and we improved the quality of the gradients involved… hmmmmmm".

The other model's reply (condensed; Louis pasted it in the Coordinator's session):
- page-dewarp made three gradient improvements: autodiff (access to information the old solver didn't use), `maxcor` 10 → 100 (more information extracted from a succession of gradients; 871 → 147 evaluations on one image), and float64 (stability).
- AD is *lifting* a computation into a richer algebra. Forward mode works over pairs (a, a′) with (a, a′)(b, b′) = (ab, a′b + ab′), so the extra information propagates automatically.
- Hence the proposal: alongside *contraction along composition* (what else moves together? share existing work), pursue *enrichment along composition* (what else can be computed together? make existing work yield more, so that later work is unnecessary).
- It also proposed opening the shared-representation track under this framing. It asks whether enriching Huffman or MTF could yield constraints on text positions before LF is built. It warns that "some obvious enrichments, such as accumulating occurrence counts alongside MTF, are already represented in your research", and that information which merely restates the inversion problem doesn't count.

**Louis:** "Sure, do whatever you think best."

**The Coordinator's decisions:**
- ESSENCE.md now holds both principles.
- Channel #22 gives MONODROMY enrichment as a parallel line beside the senses of moving together. It starts with what one LF or run step reveals at no extra cost.
- The cross-stage track opens under the enrichment framing (`2026-11-24-algorithm-level/CROSS-STAGE.md`).

## 38. NEEDS M-21: landmark induction measured on the six blocks

Run by the Coordinator's agent at MONODROMY's request (record 114 §5). Script, raw numbers and readings: `2026-11-24-algorithm-level/m21/`. Checks: each L's SHA-256 and run count match the manifest, and LF is a single n-cycle.

- **Q rises, it does not fall:** 1.05–1.20 × n, every level adding to the first level's n steps.
- **D falls to 1.3–1.9% of n at k = 3, about 10–17 thousand dependent steps.** That is one to two orders of magnitude short of record 114's "a few hundred". On JSON the most frequent landmark strings stay frequent (`"` → `,"` → `","`; `":` → `e":` → `pe":`), so the final walk on the last induced map dominates. The rounds per level stay at about 900 or fewer.
- **Lone rows are under-represented among landmark rows:** byte landmarks 7.7–9.9% at k = 1, against Y's 14–15% of rows.

Verdict under the referee (#22): no cut in Q. The cut in D is real but modest, roughly 50–75×, and only a unit-price gain at that (residual b). It is reported to MONODROMY in channel #23.

## 39. The fresh Neon chat's six senses, reviewed; the three corners

Neon (movement) worked six senses (`research/neon-movement`, c71d0a22) and claimed no breakthrough. That is accurate. The review is in `neon-movement-critique.md`.
- **None is a new sense.** S3 and S4 are external-memory pointer jumping and list ranking (Chiang et al.). S1B is Open plus a ruling set. S2 is negligible, at most 2.8% of h₁. S6 is a sampled-rank residual.
- **No enrichment.** The only rider is the text position, which restates the inversion.
- **The best D result to date:** S1B gives D = 1,093–6,013 at P = 4–5, with Q still 5–10·h₁. Its M-21 figures agree exactly with ours (§38).
- **Discrepancies:**
  - S5's D headline counts the thin wave only; Neon's own JSON gives 15–44k.
  - S5's thin scatter is uncharged.
  - S4's "universal 2.94·m" is the density of local minima under random labels, not a Cole–Vishkin property.

**The three corners (the Coordinator, to Louis).** Every candidate so far lands in one of three corners:
1. Few or zero addressed reads, but many passes: doubling, list ranking, sort-based inversion.
2. The same reads, but a shorter dependent chain: lanes, landmarks, S1B.
3. Fewer reads only where text repeats: runs, copy plans.

On novel random digits each digit must be touched once, because it is the output (record 113). So the open question narrows to this: **can that one touch be a predictable, streamed access within about two passes?** Or is there a lower bound, from sorting and list ranking in the I/O model, saying that predictable-access inversion needs ω(1) passes? A construction would be the breakthrough. A proof of the bound would say that the gain must come from changing the problem, for example the cross-stage track. Louis has asked ChatGPT Pro to help; this framing was offered to pass on.

## 40. Leads from dynamical systems, sent to every member

Louis pointed at Fork (forkdynamics.com; hinsley/Fork), a bifurcation-continuation tool. Two of its ideas map onto ESSENCE's principles:
- **Feigenbaum renormalisation** (square, restrict, rescale, and get the same kind of map back), for contraction and the recurrence direction;
- **dual-number evaluation** (forward-mode AD), for enrichment.

Louis: "can you try not to just restrict it to being sent to one? Like we've got Monodromy, we've got Anti-Gravity, and we've got ChatGPT. So ideally, like maybe all of them." The brief is `LEADS-dynamics.md`. It goes to MONODROMY and the Antigravity chats through channel #25, and to ChatGPT Pro by paste.

## 41. Neon (movement) cycle 2 reviewed

The review: `neon-movement-cycle2-critique.md`. No breakthrough on Q is claimed, and none exists.
- **S8, count-targeted landmarks (MONODROMY 114 §6), reproduces exactly:** D = 608–1,974, that is 0.07–0.22% of n. It passes the pre-registered gate on 6/6. It is the best D result, but Q stays at about n. Uncharged: level unrolling and the precomputed level labels.
- **E1, the MTF monoid lift:** mostly known. The chunk scan is the 30 October (O, P) decomposition; gap-coded sampled Ψ is Sadakane's compressed suffix array. It is a memory trade, 1.47 against 4 bytes per row, and not a cheaper whole decoder. "865×" covers chunk prefix states only.
- **S7.1, the pairing-oracle obstruction:** it proves invariance of π under interior-digit swaps, not Q ≥ (d−2)m. Converting bits into accesses is the move record 116 rejects. **Relation C stays open.**
- **E3.2:** parts (1)–(2) are correct and near folklore. "Only cross-stage escapes" is asserted, not proved.
- **R1, renormalisation:** the descent invariant is Gessel–Reutenauer. The "fixed point" holds only on the synthetic template; on Wikidata, q_min rises 8–21×.
- **Cycle-1 corrections:** acknowledged in the README, but the old claims are still in the files.

## 42. ChatGPT Pro: two overstatements corrected, and the closure problem

ChatGPT Pro's answer is recorded verbatim in `chatgpt-closure.md`.

**Two corrections, both accepted. Both are the Coordinator's overstatements in summaries to Louis and in agent briefs:**
1. **G1 is not a machine lower bound.** It bounds charged operations in the rigid-interval calculus. The bridge to unpredictable machine reads is unproved, as the council said (§33, COMPARISON). Summaries that called it "the proved floor on reads" were wrong. The target stays "fewer than one unpredictable read per lone row, acquisition included". The claim that this is impossible is unproved.
2. **The 16% figure is not a ceiling.** It holds for the supplied-codes intervention only (§30): codes supplied in output order, with construction, word handling and later stages kept. It says nothing about an algorithm that also removes construction or changes emission or scheduling. CROSS-STAGE.md and ESSENCE.md are corrected.

**Prior art it surfaced for the cross-stage track** (added to CROSS-STAGE.md):
- `src/runs.rs` already builds the run records straight from inverse MTF/RLE2;
- Huffman with composed run and MTF actions lost on 22 November;
- RLE1 and CRC were composed through the word grammar on 10 November.

**Its structural result (block 0, its own reproducer, not in this repository).**
- It uses Open's factorisation R = P ∘ LF_v, where P is a piecewise translation.
- On the original retained map, each byte's successor targets advance in steps of 1. For that, 2,148 address segments are enough, 0.061 per lone row: an orderly access schedule, including for lone rows.
- After one in-place M-17 composition sweep, the same grouping needs 44,814–52,319 segments (1.28–1.49 per lone row). That is 21–24× more fragmented. The uncapped sweep's flat word storage grows from 0.90M to 2.51M bytes. All three variants were byte-exact.
- **The obvious recursion therefore fails on the first real block.**

**The closure problem it poses.** Starting from R = P ∘ LF_v, find a representation of composed retained-row transitions that does three things:
- it keeps successor acquisition, word payloads and the access schedule cheap;
- its compactness survives composition on the admitted blocks;
- it is built without following the transitions it replaces.

A second successful composition, with an explicitly and cheaply built representation, would be a meaningful new result. ChatGPT Pro's own verdict on the cursor idea: solid but incremental at one composition.

**A combined whole-decoder oracle (2×2), not yet run:**

| | Normal Open emission | Supplied codes |
|---|---|---|
| **Normal scanner** | A | B (already run, §30) |
| **Precomputed symbols** | C | D |

Delivery stays timed through the production bounded queue. It tests whether the two interfaces have *combined* potential that neither shows alone. It differs from the parscan attempts of 22 November (C-37 at 0.883 with regressions; the streaming version at 1.54×), because it measures potential before any implementation. Commissioning it needs a frozen recipe and a workstation run; this is put to Louis.

**Its qualification of the "answer in disguise" rule, accepted.** Free text positions are invalid. A cheap algorithm that *computes* text positions is a legitimate route. The issue is unpaid acquisition, not what the information is called.

## 43. Neon (movement) cycle 3 on the closure problem, reviewed: closure not achieved

The review: `neon-movement-cycle3-critique.md`. The recomputed figures on block 0:

| | M-17, 1 sweep | CL1, 2 sweeps |
|---|---:|---:|
| output map, exact +1 segments, all m rows (ChatGPT's metric) | 52,319 | **54,438** |
| output map, segments over survivors only | 19,467 | 17,562 |
| emit links | 32,833 | 29,295 |

- **"44–133× less fragmented" compared sweep 2's *read* schedule with M-17's *output* map.** Sweep 2's read schedule is R itself by construction. On the same metric CL1's output is more fragmented than M-17's. CL1 never composes the composed map, so closure is not achieved.
- **CL1 is the unimodal-arch / SA-IS cut (N-20 C4) applied to R.** Links of about m/3 are the peak statistic of a random permutation, not a floor.
- **The 8m binary DAG moves work into emit.** It costs about one dependent load per output byte.
- **CL2's bound** seg₊₁ ≥ 1 + des(T⁻¹) is the rising-sequence count (Bayer–Diaconis). It is correct, but it does not establish a third-composition wall.
- **CL4:** acquisition is still 0.88–4.07·h₁. **Total Q is 1.55–4.99·h₁ on the six blocks**, with the sweeps uncharged. Emit links alone are 0.67–1.08·h₁.
- **The binding cost is acquisition, not composition.** The most promising threads:
  - lazy or shared acquisition for thin runs, reusing the B_P cursor intersection inside strip contraction;
  - CL2's depth-2 schedule keyed by (byte, successor's byte), 6.3k segments on block 0, still to be costed with payloads.

## 44. The four-arm oracle: INTERACTION. At 10 workers the scanner is the wall, and behind it emission is worth having

`bench-four-arm`, run on the workstation (i9-10900K) at 5731db19. The bundle is `four-arm/run-20261124T204154Z` on `evidence` at c9302539. The Coordinator checked its SHA-256 (736e0624…976c). All 20 rounds' outputs match the reference: 838,800,784 bytes, CRC d7626c17, 0 mismatches.

Wall-time ratios against A, with 95% intervals:

| workers | B (supplied codes) | C (supplied scanner output) | D (both) | D / better single arm |
|---|---|---|---|---|
| 1 | 0.830 [0.825, 0.834] | 0.996 [0.991, 1.001] | 0.825 [0.820, 0.831] | 0.994 against B |
| 6 | 0.850 [0.844, 0.855] | 0.982 [0.979, 0.986] | 0.830 [0.827, 0.834] | 0.977 against B |
| **10 (judged)** | **0.976 [0.936, 1.017]** | **0.806 [0.782, 0.831]** | **0.741 [0.717, 0.766]** | **0.920 [0.901, 0.939] against C** |

- **Harness check:** B at 10 workers reproduces §30 (0.976 against 0.988; the intervals overlap).
- **Peak RSS:** C and D are 3–5% lower.
- **Verdict by the registered rule: INTERACTION.**
- **What it means:**
  - At 1 and 6 workers, emission is the lever and the scanner barely matters.
  - At 10 workers the serial scanner, which reads the bits and decodes the Huffman codes, is the binding constraint. *(Corrected by the writer: the workers do MTF/RLE2, the inversion and emission.)* Removing it alone saves 19%.
  - With the scanner removed, supplied codes save a further 8% (D/C 0.920). §30 saw nothing from them because the scanner hid it.
  - So the 10-worker null in §30 was masked, as ChatGPT Pro (§42) suggested. An algorithm that changes **both** the scanner's work and emission has up to about 26% of whole-decoder time on the table at 10 workers.
- **Caveats:**
  - These are oracles: the scanner output and the codes were supplied, with acquisition outside the clock. The open problem is acquiring them cheaply.
  - "Scanner removed" also removes serial bit-decode work that a real algorithm must still do somewhere, in parallel or cheaper.
- **Consequences:**
  - The cross-stage track (CROSS-STAGE.md) becomes a priority beside the inversion work.
  - At 10 workers the first target is the scanner's serial cost. Earlier parallel-scan attempts (C-37 at 0.883 with regressions; the streaming version at 1.54×) failed on scheduling and memory, not on potential.
