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

## 45. Louis's four points: a finer grid, notes kept, the mathematics' targets, durable records

Louis: "first off, 1 6 and 10 is too coarse. we should tighten that so we can model better, set shift points [or at least optimal ones] properly. also perhaps we need to ensure we have done it in the ideal way (maybe add to the antigravity work list and monodromy) … secondly i want you to be keeping notes of your last message in the journal please. thirdly i would like you to establish what exactly we can find new maths for - my sense is that the induction part is the ... factory er, not the doorman the other one... also please can you ensure everything is a durable record and just respond by summarising these rather than you know, writing off the cuff..."

1. **Grid.** `SCANNER-BRIEF.md` is amended. Every worker count runs from 1 to 20 (as corrected). The gate's threshold is derived from the measured crossover and a fitted pipeline model, before part 2's data. The method's soundness goes to MONODROMY and to Neon's work list (channel #31).
2. **Notes.** `COORDINATOR-NOTES.md` records each substantive reply to Louis, from now on.
3. **The mathematics' targets.** `MATH-TARGETS.md`. The induction runs in the workers (the assemblers), so the mathematical target is the inversion's emission and construction. Emission's oracle ceiling is 0.83–0.85 at 1–6 workers, and 0.92 at 10 behind the scanner (D/C). Construction's share is unmeasured, so a worker-time-split oracle is queued. The scanner (the doorman) is first an engineering target.
4. **Durability.** Every decision is in this journal, the briefs and the channel; replies to Louis are summaries of the record.

## 46. The record restructured: lowercase names, subdirectories, and `math-targets/` as a project

Louis: "i dont want this all caps md file approach it is not very good, it is not the proper way to do this is it? can we do it properly please - if you want a math_targets subproject with a readme that is better than MATH_TARGETS.md . it would be good to compile much more detail (maybe tsv more appropriate than md) on each of them as well as the high level as an overview of the data … ask monodromy what would be worth metricising etc here and also for what to reason about. maybe we also wanna measure things like avg rss peak rss etc etc etc"

Path map, old to new, all under `docs/journal/2026-11-24-algorithm-level/`. Earlier sections and pinned links keep the old names.

| old | new |
|---|---|
| ESSENCE.md | essence.md |
| CROSS-STAGE.md | tracks/cross-stage.md |
| LEADS-dynamics.md | leads/dynamics.md |
| FOUR-ARM-BRIEF.md | briefs/four-arm.md |
| SCANNER-BRIEF.md | briefs/scanner.md |
| COORDINATOR-NOTES.md | notes/coordinator.md |
| MATH-TARGETS.md | math-targets/overview-2026-11-24.md |
| council/BRIEF.md, INDEX.md, COMPARISON.md | council/brief.md, README.md, comparison.md |
| *-critique.md, review-pr820.md, fresh-reader-halving.md | reviews/ |
| chatgpt-*.md, outside-review.md, against-mathematics-chatgpt.md, packed-inversion-sweep.md, jax-idea-verdict.md | external/ |

`math-targets/` holds:
- `README.md`: the overview;
- `targets.tsv`: T1–T6, each with its stage, oracle, ceilings, obstacle, owner and status;
- `metrics.tsv`: what is measured and what isn't;
- `data/measurements-four-arm.tsv`, derived by `scripts/extract_four_arm.py` from the bundle's raw rows, which are kept in `data/`.

## 47. Beyond doorman and assemblers: the full character, measured linearly

Louis asked whether the two-stage picture (scanner and workers) is the true decomposition. He asked for a finer one that establishes "the full character" across 1–20 workers without an exponential run, recalling the systems-biology analogy from the conversation post of 22 November. He wanted it put to every member. The brief is `briefs/decomposition.md`. It covers:
- **Eight stages and three shared resources:**
  - S0 input; S1 headers; S2 Huffman; S3 hand-off;
  - S4 MTF/RLE2; S5 construction; S6 emission; S7 RLE1/CRC/output;
  - R1 memory bandwidth and LLC; R2 the allocator and faults; R3 SMT pairing.
- **Three methods, each linear in the number of stages:**
  1. instrument every boundary, once per count;
  2. metabolic control analysis: each stage's *flux control coefficient*, from small injected slowdowns, with the summation theorem as a check that the decomposition is complete;
  3. a queueing-network model fitted to the data and validated on held-out counts.
- **To MONODROMY (channel #34), ChatGPT Pro and Neon (by paste):** is the decomposition right? What is the minimal design? What should be reasoned about rather than measured? Are control coefficients the right formalism, or are elasticities, causal profiling (Coz) or roofline models better?

## 48. MONODROMY's records 118–125 reviewed; the decomposition answers converge

The review is `reviews/monodromy-118-125.md`.

**The strongest result: record 119's conservation law with Lemma W.** A sweep is an induction on its failing set, and the surviving classes are the return words. The law reads |T|·log C + S·log|T| + W ≥ n·log σ. A composition keeps the form of the access schedule but not its compactness on high-entropy rows. This explains ChatGPT's 21–24× fragmentation (§42) and answers the closure problem **negatively, within this class**.

**Record 118's Theorem I** bounds every interleaving of monotone cursors. It also breaks record 116's conjecture.

**Corrections, sent to MONODROMY:**
- Record 118's "exactly 1/(j+2)" holds only for large σ. At σ = 2 the simulation gives 0.238 and 0.186.
- Record 119's prediction 2 should use m, not |T|; as written it cannot fail.
- Records 118–120 assume row order fails on 1/2, but the measured value is 0.365. So NEEDS M-25 should run first.
- Record 123's grid predates the 1–20 correction.

**What moves a target:**
- Record 120: search-free strip contraction, projected at about 0.6·h₁ (conjectured).
- Record 121: an exact composable chunk summary for MTF, RLE2 and the run records, which bears on T3.
- Record 122: h₁, r and the width histogram are free counters of the Huffman scan. A parallel parse across blocks from the block magic is exact; this is lbzip2 prior art.

**The decomposition answers converge.** MONODROMY's record 125 and Neon's `DECOMPOSITION.md` (00c08750) agree on several points:
- slowdowns of 20–25%, not 5–10%, because the noise is too large;
- the summation theorem checks *time*, not resources;
- drop NUMA, and treat block heterogeneity as covariates;
- add a serial tail stage: retirement, reordering and output;
- pin threads, and separate SMT siblings;
- train on some counts and validate on the rest.

They differ in two places:
- **Formalism:** MONODROMY makes the roofline per stage primary, with Coz worth trying; Neon keeps control coefficients as validation.
- **What they add:** MONODROMY adds the leader/helper split inside teams; Neon adds k+3 threads, so SMT oversubscription starts at k = 8–9, turbo droop, and CFS placement.

ChatGPT Pro's answer is pending.

## 49. Neon (movement) cycle 4 reviewed: nothing gets below h₁ with every charge counted

The review is `reviews/neon-movement-cycle4-critique.md`. All-in Q at k = 0, with both acquisition (IET) and emission payload added (Neon's tables added one or the other):

| route | all-in Q / h₁ |
|---|---|
| Open walk | 4.31–6.88 |
| M-17 flat | 1.66–2.83 |
| CL1 flat | 1.48–2.59 |
| R⁴ pair flat | 1.32–2.33 |

**Referee's ruling on the edge case.** With R⁴'s four-word payload inlined in its record, Q would be 0.79–1.51·h₁, below h₁ on three blocks. That needs about 2k concurrent cursors and a 3.5 MB per-block record array, with its 4n build uncharged. Π allows s ≤ |Σ| + O(1) cursors (council comparison; channel #18), and under that limit R⁴ costs 2.67–4.18·h₁. **Not admissible.** It is recorded as a metric artefact.

**Results that stand:**
- **NEEDS M-27 closed negatively,** under record 120's pre-registered rule. Lazy thin contraction leaves m/m₁₆ at 0.89–0.93. The new mechanism: removing interior rows fragments the read schedule 4.1–8.7×, so a 9% cut in links becomes a 15–81% loss.
- **IET acquisition (0.10–0.34·h₁)** puts acquisition back at its true scale. It is not new: Open's constructor, and record 120 §1.

**Overclaims:**
- "−70% against Open" counts L1 dictionary reads; link for link the cut is 63–67%.
- T5.1 and T5.2 show that final LF values depend on the whole input. They do not exclude cheap deferred riders; record 121's Proposition M carries such riders.
- ΔQ = 0 holds by construction, since every rider is streamed.
- The four-arm attribution does not follow from the theorems.
- "Before SMT" contradicts the k + 3 thread count.
- Passes are understated.

**Recommendations taken into the recipe brief:**
- Estimate k* from the per-round starvation gap, with a bootstrap interval, rather than a global fit. Fit USL only on k = 1–9.
- Q-to-seconds exchange rates for emission and construction, from the 232 census blocks.
- Fuse the `runs.rs` front end into Open's Pass 1 inside the worker, saving a pass. Unmeasured.
- Pass the Huffman selectors on as a regime hint. Unmeasured.

## 50. Report: from a search to a formulated, measurable problem

Louis: "ok nice so basically we are now producing a proper formulation of the task and properly turning it into a toy problem, this is excellent. can you please write a report on what we found and put it in the journal". The report is `reports/2026-11-24-from-search-to-formulated-problem.md`. It covers:
- the formulation;
- the decoder as a system, with the four-arm ceilings and the decomposition;
- what has been ruled out, proved and left open in the mathematics;
- the toy problem as it now stands;
- the method, and an index of the records.

## 51. The decomposition: three answers compared, and the recipe brief

Three answers to `briefs/decomposition.md`:
- MONODROMY's record 125, with 123 and 124 (§48);
- Neon's `DECOMPOSITION.md` (00c08750);
- ChatGPT Pro's, recorded verbatim in `external/chatgpt-decomposition.md`.

They are merged into `briefs/character.md`.

**All three agree:**
- the doorman/assemblers split is a coarse topology, not a decomposition;
- the summation theorem is not a completeness test (ChatGPT Pro's counterexample: J = min(v₁, v₂) sums to 2 for slowdowns and 0 for speedups);
- drop NUMA, and treat block heterogeneity as covariates;
- add a serial tail;
- SMT sharing begins around k = 8–9, because the decoder runs k + 3 threads.

**What ChatGPT Pro adds, and the brief adopts:**
- **Requested threads are not actual workers.** The policy maps requests of 10 or more to 7 lazy/Open workers and changes the kernel with k. So the mechanism study uses `mibz2-workers-lazy`, with a separate public-API sweep.
- **The output-to-scanner feedback.** Admission is `issued − taken`, plus the 12 MiB ready threshold. So scanner waiting can come from the consumer or from a long-tail block.
- **MTF/RLE2 and run formation are one fused activity.** RLE1/CRC stays tiled, and the graph is routed (Open admitted, skipped or fallen back).
- **Completeness comes from accounting identities, not from the coefficient sum.**
- **A discrete-event simulation of the network,** driven by per-block service vectors in original order, with fit and held-out counts fixed in advance. Roofline acts as a constraint (MONODROMY), and the response matrix replaces scalar rankings.

**Where they disagreed, and the decisions:**
- **Dose:** 10% (ChatGPT Pro), 20% (MONODROMY), 25% with pinning (Neon). Decided: **+20% primary, +10% follow-ups.** The 95% half-width at 3% SD over 12 rounds is ±0.105 at 20% against ±0.20 at 10%; the 10% runs check linearity.
- **Placement:** unpinned primary (ChatGPT Pro, deployment-oriented), with pinned and SMT-sibling runs as targeted checks (Neon).
- **Anchors:** {1, 6, 8, 10, 12, 20}, with 14 added if the baseline shows the predicted L3 cliff.
- **The primary model:** the discrete-event network is primary. Roofline (MONODROMY) is a constraint, and control coefficients (Neon) are local rankings.

**Cost:** 240 baseline observations, plus 648 intervention observations (p = 9), plus 36 PMU, plus follow-ups. It is split into two recipes, after the scanner profile.

**Housekeeping:** at Louis's instruction ("do not title things with times of day … title them according to the content or relevance"), the report is renamed `reports/2026-11-24-from-search-to-formulated-problem.md`.

## 52. The programme chain, after Jain; MVA and a fractional factorial added

Louis restated the aim. The toy problem and simulation set-up are for a Jain-style performance analysis (https://www.cs.wustl.edu/~jain/iucee/index.html; *The Art of Computer Systems Performance Analysis*, 1991). That analysis is to yield a systems-biology model of the program, which in turn locates where new mathematical devices fit.

`programme.md` lays out the chain in eight steps, each with its status. It is now the entry page:
1. task and referee — done;
2. the toy problem — done;
3. structure — designed;
4. measurement — the scanner profile is running;
5. experimental design — being built;
6. the model;
7. where mathematics fits;
8. the mathematics.

**Amendment to `briefs/character.md` (§7a):**
- the operational laws (Little, utilisation, forced flow) are the completeness checks;
- mean-value analysis of the closed network (W = 4k) sits beside the discrete-event simulation;
- a 2^(9−4) resolution-IV fractional factorial at k = 8 and 10 replaces one-factor-at-a-time there, so that two-factor interactions are not confounded with main effects. The four-arm result was itself an interaction.

## 53. Three performance-analysis courses, read in full

Louis asked ChatGPT Pro what three courses contain: Jain's IUCEE course, Harchol-Balter's 15-857, and her theory course 15-849b. He then asked for it to be recorded "durably … properly, full text not glossed", and for the Coordinator to answer the same question by reading the full material, without summarising subagents.
- **The exchange and ChatGPT Pro's first answer,** verbatim: `external/chatgpt-performance-resources.md`. Its second answer was pending.
- **The Coordinator's answer:** `methodology/performance-courses.md`. It was read directly:
  - the three pages;
  - all 30 Jain module topic lists, and six slide decks in full (k_02, 16, 19, 25, 33, 34);
  - the textbook's contents;
  - the Fall 2025 schedule;
  - the theory course's topic list and book list.

  The stripped source texts are kept in `methodology/sources/`.

**What it adds to `briefs/character.md` (§7b):**
- device demands Dᵢ = Vᵢ·Sᵢ per activity, with asymptotic and balanced-job bounds for the closed network at each k;
- **modification analysis** (Harchol-Balter ch. 7) to price each mathematical candidate as a change to one demand, before it is built;
- multiplicative (log) models with allocation of variation (Jain ch. 18–19);
- a fluid approximation as a closed-form check on k\*;
- sample-path analysis, rather than stochastic assumptions, for the fixed block trace;
- job classes for the Open and fallback routes (classed networks).

## 54. ChatGPT Pro on the three courses, recorded; where it adds to the Coordinator's reading

ChatGPT Pro's answer to "tell me all about the content of these" is recorded verbatim in `external/chatgpt-performance-courses.md`. It agrees with the Coordinator's own reading (`methodology/performance-courses.md`, §53) on scope and structure. Both found that the 2003 notes link is broken. It adds four things the Coordinator did not have:
- **The 2025 handouts skipped textbook chapters 19 and 22,** the closed-network, product-form, MVA and BCMP treatment. Operational analysis of closed systems was still taught. The Coordinator's mapping of "MVA through the Arrival Theorem (ch. 19)" refers to the book, not to the 2025 course.
- **A 2025 recitation on modification analysis:** raising an LRU cache's hit ratio can *reduce* throughput, by intensifying contention on the hit path. Measurement, simulation and a bound together show a rise, a plateau and then a decline. This is the closest classroom analogue to the decoder's masked bottleneck (§44), and to ChatGPT Pro's warning that slowing one activity can raise throughput (§51).
- **The M/G/1 waiting-time formula** E[W_q] = λE[S²]/(2(1 − ρ)) puts the second moment of service time into the delay. In ChatGPT Pro's illustration, at 80% utilisation and a 10 ms mean, waiting is 20, 40 and 200 ms as C²_S goes 0, 1, 9. Per-block service-time variability (long-tail blocks) therefore matters on its own account. Stage 1's per-block service vectors should report C²_S per activity.
- **Fluid models,** with dq/dt = incoming rate − processing rate, which the Coordinator also recommended (§53).

**Addition to `briefs/character.md` §7b:** report the second moment, or C², of per-block service time per activity, beside the means and demands.

## 55. A design review before stage 2 is frozen

Louis asked whether to review the experimental plan with the agents before the model is built from its results. Decided: **yes, one round**. `briefs/character-design-review.md` goes to three reviewers, independently:
- MONODROMY (channel #38);
- Neon (by paste, committing to its branch);
- ChatGPT Pro (by paste, answering in chat; the Coordinator records it verbatim).

Its questions:
1. which competing mechanisms must be told apart (identifiability);
2. the most informative design for about 2–3 workstation hours (grid, anchors, dose, factorial or sequential);
3. measurement validity on the i9 (Mytkowicz-style biases);
4. whether every input each model needs is logged;
5. what to cut.

Stage 1, the 1–20 baseline, proceeds meanwhile and sizes stage 2. The Coordinator merges the three answers into `briefs/character.md` before the Pathfinder freezes stage 2.

## 56. Could a different queueing discipline do far better?

Louis asked whether the decoder's conventional queueing could be replaced by another form for far better performance. The answer is `leads/queueing-discipline.md`:
- **What the decoder is today:** a closed network (W = 4k in flight) with FCFS dispatch from one serial scanner and an in-order resequencing reader; admission is issued − taken, plus the 12 MiB ready cap.
- **What can't change (proved, operational laws):** the throughput ceiling, min(N/(D + Z), 1/D_max, k/b), is set by the demands. No discipline raises it; only lower demands or more capacity at the bottleneck do.
- **What can change (argued):** how close the decoder gets to that ceiling. The losses are head-of-line blocking at the resequencer, the straggler drain of a finite stream, and service-time variability.
- **Candidates:**
  - size-aware (LPT) dispatch, possible because record 122's scan counters give block sizes nearly free;
  - helpers for the head-of-line block, through the existing team machinery;
  - work-based admission;
  - relaxed resequencing;
  - a parallel scanner, the only one that raises the ceiling.
- **The deciding measurement:** efficiency against the bound at each k, from stage 1's per-block timestamps. If the gap is large, replay each discipline in the simulation with the measured demands before writing any code. That is modification analysis applied to the discipline.

The question is added to the design review as question 6.

**Correction:** the time-of-day point was one file title, not a standing rule for prose.

## 57. The scanner profile across 1–20 workers: the toy model predicts the knee, the discipline has little to give, and the default over-provisions

The bundle `scan-profile/run-20261124T223748Z` (`evidence` at 41389cbe, SHA-256 530f97e0…8260, checked) is from `bench-scan-profile` (#838) on the i9-10900K: 10 cores, 20 threads, one corpus (47.4 MB compressed, 839 MB out, 1724 blocks), 20 timed and 20 untimed decodes at each worker count from 1 to 20, unpinned. It is a diagnostic: it decides nothing. The per-count summary is `math-targets/data/measurements-scan-profile.tsv`, derived by `math-targets/scripts/extract_scan_profile.py` from the bundle's `runs.tsv` (copied beside it, with `report.txt`).

**What was found (measured):**
- **The two-station model predicts the knee.** From the counts 1–4 alone, the service times are 106.6 µs a block for the scanner and 798.3 µs a block for a worker, so the predicted crossover is k = 798.3 / 106.6 = 7.49. The measured crossover, where rho falls through 1, is 7.67. The fit wall(k) = c + max(b, a/k) over all 20 counts gives k\* = 7.17, a = 1.279 s, b = 0.178 s, c = 0.032 s, RMS residual 13.6 ms. The residuals are systematic in two places: about −13 ms at k = 8–14, and about +21 ms at k ≥ 17 (the step below).
- **Below the knee the workers bind; above it the scanner does.** Scanner slot waits fall from 0.86 of the wall at k = 1 to 0.003 at k = 8. Worker idle time stays under 4 ms until k = 8, then grows with every added worker.
- **The best wall is 0.191 s, at k = 8** (4390 MB/s of output). From 8 to 14 the wall is flat at 0.197 ± 0.005 s.
- **At k ≥ 16 the wall steps up 20%, to 0.230 s, because the scanner slows.** The scanner's own work rises from 0.186 s (k ≤ 8) to 0.229 s (k = 16), +23%, while its share of the wall stays at 0.97. That the scanner's thread now shares a physical core with a busy worker (SMT) is the hypothesis; the step lines up with 16 workers plus the scanner and reader exceeding the 10 cores' spare siblings, but no run here pinned threads to test it.
- **Efficiency against the busiest server's demand is 0.945–0.998 at every count.** That is max(scanner work, one worker's busy time) over the wall. It is 0.97 on the plateau and lowest (0.945) at k = 10.
- **The scanner's work is 81% Huffman decode,** at 1.41 ns a symbol at k = 1 and 1.51 ns at k ≥ 3 (7% slower: a lower all-core turbo is the likely cause, not measured). Tables are 7%, selectors 4%. A per-block fit finds no cost per table switch (−3 ns a switch, i.e. nothing measurable), with 438 switches in 1163 selectors a block.
- **Over-provisioning costs:** the Python binding's default is `available_parallelism()`, 20 workers on this machine. Against k = 8, k = 20 has wall 0.230 s against 0.191 s (+20%), user CPU 2.14 s against 1.61 s (+34%) and the decode's own peak RSS 74.8 MB against 54.1 MB (+38%). From k = 9, CPU rises with no gain in wall.

**What it means:**
- **Step 4 of the programme is done for this corpus,** and steps 2–3's toy model (a serial stage feeding k parallel servers) is validated: four counts predict the knee to within 0.2 workers. The model's two numbers, s and w, are the whole first-order story here.
- **Louis's queueing question (§56) has its answer on this corpus: no discipline is far better.** Efficiency against the busiest server is at least 0.945 everywhere, so reordering, size-aware dispatch, helpers or relaxed resequencing could recover at most about 5% (3% on the plateau). The big gains are elsewhere: lower demand at the bottleneck, or more servers for it.
- **Two cheap engineering leads follow, and both are production defaults, so both need an adoption registration:**
  1. **Don't spawn workers past the knee.** A default of k\* rather than every logical CPU saves 20% wall, a third of the CPU and a third of the memory on this machine. k\* = w/s depends on the data (compression ratio, block content), so a fixed number is a stopgap. Both s and w are measurable online during a decode (the scan's counters and the workers' busy times), so an adaptive cap is possible.
  2. **Keep the scanner's core to itself.** If the SMT hypothesis holds, reserving the scanner's sibling thread removes the 23% scanner slowdown at high k. A pinned run tests it first.
- **Where the mathematics fits, sharpened.** The response is piecewise. Below k\*, wall ≈ a/k, so only the workers' demand (targets T1–T3 and T6: emission, construction, MTF/RLE2, RLE1/CRC) moves it. Above k\*, wall ≈ b, so only the scanner (T4, Huffman decode at 1.5 ns a symbol) moves it. On a 10-core machine k\* ≈ 7.5 is already inside the machine, so beyond k\* the scanner is the binding target, which is what the four-arm test's arm C showed at 10 workers (0.806). Reducing worker demand lowers k\* and helps machines with fewer cores; reducing scanner demand raises the ceiling everywhere.

**Limits:** one machine, one corpus, unpinned. The crossover depends on the corpus. The SMT and turbo explanations are hypotheses. Stage 1 of the character recipe (#841, 12 randomised rounds, per-visit moments and per-block timestamps) is the next measurement and will test the efficiency figure against its gap decomposition.

## 58. The courses mined against the scanner profile

Louis: "that is awesome and also suggests we need to mine these courses for opportunities then." The result is `leads/course-opportunities.md`, read from the full course sources against the measurements of §57. The findings, largest first:
- **The machine bound (balanced job bounds):** against the busiest server the decoder is 0.97 efficient. Against the whole machine it is about 0.57 (estimate): total demand about 1.62 s over about 14.9 lone-thread equivalents gives about 0.109 s, against the best wall of 0.191 s. The 43% gap belongs to the topology (one serial scanner), and no discipline reaches it.
- **Right-size the worker count (capacity provisioning), measured:** the Python default of 20 workers costs 20% more wall, 34% more CPU and 38% more memory than 8. The cap should be adaptive, from an online renewal-reward estimate of w/s.
- **Put the bottleneck on the fastest core (hypothesis):** the scanner decodes 7% slower once other cores are busy and 23% slower at k ≥ 16. A pinned run on a favoured core with its sibling idle tests it.
- **MVA instead of the hard-max fit:** the fit's 32 ms constant is an artefact of the knee's curvature (only 2.5 ms of the wall at k = 8 is outside the scanner).
- **Little's law for the window,** and **workload classes:** 792 of the 1724 blocks are tiny (0.1% of the bits); the real blocks' sizes have C² ≈ 0.07.
- **Ruled out:** size-based scheduling and redundancy, by the low variability and the 0.945 efficiency floor. The decoder is a closed batch system, where scheduling matters little (Schroeder, Wierman and Harchol-Balter, NSDI 2006).

## 59. ChatGPT Pro's design review, recorded

Recorded verbatim as `external/chatgpt-design-review.md`. Its verdict: keep stage 1; amend stage 2 before freezing it. It is the first of the three design-review answers (MONODROMY and Neon are outstanding). The points that change what we do:
- **Stage 2 shrinks to a sequential design:** a screen of the nine +20% coordinates at two counts bracketing the transition (120 observations), a full 2³ over at most three coordinates (128), then mechanism contrasts (W × byte threshold; placement × verified frequency) and a reserved 20% for unseen validation. The two nine-factor fractional factorials go: 32 rows cannot resolve 46 coefficients, and the amended grid would have been about 1,440 observations, not 888.
- **A factorial main effect is not the baseline sensitivity:** for T = max(a, b) the baseline coefficient is 1, the balanced factorial's marginal effect 1/2. The two estimands must not be mixed across worker counts.
- **Exact MVA doesn't apply to the real pipeline;** it applies to a documented surrogate, with its error against the simulation reported. Bounds aggregate demands by resource: X ≤ min(1/D_scanner, k/D_worker, 1/D_consumer).
- **Tracing hazard, from the pinned code:** Ready is marked after the lock is released, Taken under it, and Enqueued after the send, so a timestamp sort can produce an impossible queue history. Transitions need under-lock sequence numbers.
- **The admission semantics enter the model verbatim:** full = ahead ≥ W ∥ (ready_bytes ≥ threshold ∧ ahead ≥ floor). The byte threshold gates new scanning only, and taken advances when the reader acquires a buffer, not when it finishes it.
- **Measurement validity:** one binary with runtime-selected treatments and fixed launch layout; the achieved frequency, temperature and throttling with every timing; counters gated to the stopwatch; a published PMU event manifest.
- **The busy-time crossing is renamed k_balance,** and doesn't alone show the scanner limits; concurrent states and interventions must.

Two corrections it forces on §58's opportunities are made in `leads/course-opportunities.md`: MVA only on a surrogate (opportunity 4), and the pinned-scanner run records placement and achieved frequency (opportunity 3). §58's claim that the scanner limits above the knee is supported, in aggregate, by the profile: at k = 10 the scanner works 0.945 of the wall with admission waits of 0.014, so a slow reader isn't the limit.

## 60. The three design reviews in; the Coordinator's decisions

Neon's review (`reviews/neon-design-review.md`, a verbatim copy) and MONODROMY's record 128 arrived alongside ChatGPT Pro's (§59). MONODROMY also answered channel #37 with record 127: which decoder operations have the compaction shape. Its strongest candidate is not a compaction at all: the CRC, folded with carry-less multiplication.

The decisions are `briefs/character-review-decisions.md`:
- **The three agree on eight things,** adopted as stated:
  - cut the 2^(9−4) factorials;
  - MVA only on a surrogate;
  - one binary with runtime switches;
  - log CPU placement and achieved frequency;
  - split the admission wait by cause, with the head-of-line block;
  - an unconditional window × byte-threshold test at the knee;
  - SMT crossed with turbo;
  - anchors chosen from stage 1.
- **Stage 1 (#841) takes twelve additions before its ready tag.** Among them:
  - under-lock sequence numbers;
  - lock waits bracketed;
  - the reader's split;
  - one fresh process per decode;
  - launch hygiene;
  - 16 rounds at k = 7–16;
  - a PMU event manifest;
  - the corrected corpus count.
- **Stage 2 becomes a sequence:**
  1. a screen at anchors chosen by rule;
  2. a 3×3 (scanner, workers) surface at the knee;
  3. mechanism contrasts: gates, placement × clock, order, memory dose, oracles;
  4. about 20% reserved for held-out validation.
- **One correction to the brief:** 1724 blocks are 431 distinct × 4; 932 take Open and 792 don't (Neon, from the four-arm rows). The scanner profile shows exactly 792 blocks under 10 kbit holding 0.1% of the bits, so the non-Open blocks are almost certainly the tiny ones.

## 61. ChatGPT Pro's course mining, and the corrections it forced

Recorded verbatim as `external/chatgpt-course-mining.md`. It answered independently, then compared with `leads/course-opportunities.md`. Its corrections were checked against the code (master 1c2ce5c0) and the arithmetic, and are accepted. They are listed at the top of the lead file, with the wrong lines struck or rewritten in place.

- **§57 and §58 overstated one claim: "the Python default of 20 workers costs 20%".** The public API never runs 20 workers on this i9. A request for 10 or more threads, unpinned, builds 7 lazy workers with a 28-slot window (`StreamDecoder::new` → `trimmed_here`), the cap set by the recalibrations. The live question is 7 against 8 workers: 0.207 s against 0.191 s in the profile, 7.6% less wall for under 1% more CPU. It has to be confirmed through the public API, which keeps its output where the profile reused a buffer.
- **The 0.109 s "machine bound" is a conditional scenario** resting on five unmeasured assumptions, and it is 0.568 of the best wall, not the other way round.
- **The reader is not excluded,** only disfavoured. A reader slightly slower than the scanner can hide in a finite buffered run.
- **Bottleneck migration:** a 5% cut in the scanner's Huffman demand at 8 workers already reaches the workers' limit. Scanner and worker improvements are priced together, with the count retuned: both cut 20% predicts 0.162 s at 8 workers.
- **Variability and Little's law:** the mixture's C² is about 0.98, not 0.07. Worker service per block is unmeasured, and the window sizing confused service with residence.

**Louis's caution, recorded:** the 7-worker cap was a temporary measure, because workers beyond about 7 bought nothing. It is a symptom of the scanner limit, not a reason to abandon ideas. Nothing that raises the ceiling is withdrawn: a cheaper or parallel scanner, worker mathematics, joint improvements. Raising the ceiling is what lets the cap lift. What falls is one claimed cheap win and the over-precise sizes.

## 62. Neon's course mining: two scanner plumbing costs, verified

Neon's answer is copied verbatim to `reviews/neon-course-mining.md` (from `research/neon-movement` at dd6cc26d; #145 comment 7147835505). It was written independently before comparing. Like ChatGPT Pro, it caught the default-count error (§61).

Its main find is concrete engineering that the Coordinator's lead missed, both verified in master's code:
1. **`ChannelSource::view`** (`src/parallel.rs`): the scanner wants a 2 MiB look-ahead and the feeder sends 4 MiB chunks. At each boundary, with part of the window still held, the scanner drains the held part and copies the new chunk in, on its own thread. Eleven blocks account for about 10 ms at 8 workers (5.4% of the scanner's demand, 96% of its unattributed "remainder"), and more at higher counts.
2. **The symbol buffer's re-zeroing** (`src/block.rs`): each block's `syms` is truncated to its n before going back to the pool. The next block to take it calls `syms.resize(max_out, 0)`, zero-filling it again. The 792 tiny blocks make this about 5 ms (2.7–3.0%). Output buffers already avoid this; symbol buffers don't.

Together that is about 15 ms (8% of the scanner's demand), with no algorithmic risk. **Bottleneck migration (§61):** at 8 workers, the workers rebind after about 4%, so the wall gain there is about 6%. Where the scanner binds (9–10 workers), it is 8–9%. A cheaper scanner also argues for 8 or 9 workers rather than 7.

**Decision:** routed for implementation as an ordinary performance change under PERFORMANCE-POLICY.md. It changes internal buffering, not a default. It is measured by CI's bench and, at 8–10 workers, on the workstation.

Neon's other points are added to the lead's corrections:
- the 10-core pooled figure is 0.163 s, or 0.125–0.142 s with realistic SMT gain, all conditional;
- the scanner's high-count slowdown is a bimodal mixture across runs from 10 workers on;
- favoured-core pinning likely gives 0–2% at 8 workers;
- the head-of-line waits sit exactly 4k blocks behind dense blocks, and block sizes are autocorrelated along the stream.

## 63. The experiment programme, the protocol, and the last reading commissioned

Louis asked for four things: durable records behind every reply; MONODROMY's pass at the course mining; the actual experiment; and the process recorded at the meta level, so it can be reused.
- **The report** pointing to the four readings: `reports/2026-11-25-course-mining.md`.
- **MONODROMY's final pass:** channel #39 (e87e43c9e), #346 comment 7148095764, session woken. It is the fourth independent reading, with its lens on where mathematics, not engineering, fits. It also drafts an analysis plan for stage 1's and stage 2's data, registered before the data. Its stale QUEUED 7128837429 (the main-theorem challenge, never dispatched) is withdrawn as superseded.
- **The experiment:** `briefs/experiment-programme.md`. It runs from the four-arm surprise ("0.9 and 0.9 make 0.8") to five runs in order:
  1. stage 1, the baseline;
  2. the scanner plumbing fix;
  3. stage 2, the mechanisms;
  4. the worker-count recalibration;
  5. the joint frontier.

  MONODROMY analyses runs 1, 3 and 5. Each controlling cost is classified as engineering or mathematics.
- **A rule on rejected ideas:** once the scanner's ceiling moves, re-qualify, in one batch, the rejected ideas whose evidence fits masking. That means a gain below the knee but none above it, or worker demand saved without a wall gain.
- **The protocol:** `process/protocol.md`, the twelve steps as run, the records each leaves, and rules for reuse.
- **The writer's brief:** `briefs/writer-deepening.md`, the deepening told from the beginning, by pointers to files. Sent once MONODROMY's record lands.

## 64. Louis's Alon question: structure and function, aimed at optimality

Louis saw a resemblance in method between Jain's courses and Uri Alon's network motifs. He framed it as "not universal motifs but optimal for our algorithmic purpose", and said other thinkers may fit better. Both of ChatGPT's answers are recorded verbatim (`external/chatgpt-alon-motifs.md`); the second withdrew the first's motif catalogue as trivialising. The Coordinator's steer is `leads/structure-and-function.md`:
- **Alon's value to us is his optimality work** (Part III; Dekel and Alon 2005; Shoval et al. 2012), not recurrence. The generative method of Ma et al. (2009) fits best: from the behaviour wanted, through a space of structures, to a principle.
- **The transformations already have a home:** the algebra of programming. Contraction along composition is close to fusion; enrichment is close to tupling ("banana split"); "serial into collective" is the homomorphism theorems (Gibbons 1996). Prior work, cited as such.
- **A concrete lead at the bottleneck:** the scanner as a finite-state machine whose chunk transition functions compose associatively. Data-parallel FSMs and self-synchronising Huffman decoding are prior art. bzip2's selector count is the obstruction, and the self-synchronisation length is the first, cheap measurement.
- **A finite design space for a generative test:** the scanner–worker boundary's obligations × placements, scored by the referee.
- **Max-plus algebra** (Baccelli et al. 1992) as the exact form of the pipeline's timing, against MVA on a surrogate.
- **Judging the methodology:** a retrospective reconstruction of two known breakthroughs (Rauzy–Veech; page-dewarp) and a generative test. A framework is judged by what it generates.

MONODROMY takes it as channel #40, queued after #39. ChatGPT Pro's prompt is `briefs/chatgpt-structure-prompt.md`.

## 65. The design-space lineage and Khammash: necessity first

Two more ChatGPT answers are recorded verbatim:
- the design-enumeration lineage (`external/chatgpt-alon-motifs.md`, third section): Ma et al. 2009, Cotterell and Sharpe 2010, Lim, Lee and Tang 2013, Araujo and Liotta 2018, SPIRAL, optimal-depth sorting networks;
- an explanation of Khammash 2021, "Perfect adaptation in biology" (`external/chatgpt-khammash.md`), which Louis is reading.

Khammash's programme is to specify the function, derive necessary conditions, characterise the solution space, check realisability and establish optimality. It maps onto ours (`leads/structure-and-function.md`, last section):
- **What we have, under other names:** the referee (the function); G1, record 118 and record 119 (necessary conditions); "G1 is not a machine bound" with the four-arm and profile ceilings (realisability); the performance work (optimality).
- **What we lack:** the solution space, enumerated and then given a basis, and a necessity statement in positive form (the internal-model analogue).
- **The proposal:** enumerate the referee's own class (small cursor machines, K passes, exact inversion on all short strings over tiny alphabets), reduced by symmetry. Cluster the survivors, look for the condition that separates them, and use SAT for "no survivor" proofs within the class. Feasibility is MONODROMY's first question.
- **Two ideas adopted as method:** "same function, different structure" (Khammash's Figure 3), which is why the referee counts structure; and "the hidden coordinate" (Figure 5: Z, log Z, Z₁ − Z₂), which states representation change as a search for the coordinate in which the work cancels.

Channel #40 was amended before dispatch with this as question 5.

## 66. Stage 1's data: the character across 1–20 workers

**The bundle.**
- **Run:** `character/run-20261125T001135Z`, on `evidence` at 870e61d7, SHA-256 bb72107b…2204, checked. #145 comment 7148523492.
- **What ran:** `bench-character` at #841's head 94cb742d on the i9: 840 grid decodes, 108 cost-check decodes, 30 counter decodes, 15 memory-controller decodes and 60 public-API decodes. It took about 15 minutes.
- **Stored:** the report and the summary TSVs are copied to `math-targets/data/character-stage1/`. The per-thread rows (`threads.tsv`, 3.8 MB) and the per-block event traces (`blocks-r00..r03.tsv`, about 7.9 MB each) stay in the bundle.
- **Status:** a diagnostic; it decides nothing.

**Validity.** Every one of the nine operational-law identities held in all 840 grid decodes: threads, partitioned timelines, flow, the under-lock linear order, forced flow, conservation, admission, Little's law (exact) and utilisation.

**Classification: PERTURBED.** The ledgers' on/off cost check is not clean at 10 and 20 workers:
- 1.0398 [0.9914, 1.0905] at 10;
- 1.0159 [0.9850, 1.0479] at 20;
- 1.0042 [0.9985, 1.0098] at 1.

The intervals include 1 but are wide (6 pairs), so the instrument's cost isn't shown to be small at high counts. Consequences:
- timings *from the ledgers* at 10 or more workers carry a possible inflation of up to about 9%;
- shares within a decode, the counts and the identities are unaffected;
- stage 2 should judge walls with the ledgers off.

**What it found (measured, one corpus):**
1. **The two-station model holds.** The limit moves from the workers to the scanner exactly at 8 (k_balance = 8, round-bootstrap interval [8, 8]). The near-optimal set is {8}, with a best wall of 0.1944 s. Efficiency against the per-resource bound is 0.976–0.996 up to 7 workers and 0.936–0.968 above.
2. **The scanner limits above the knee, now shown by concurrent states** as the design review required. The workers' idle time while the scanner is *active* rises from 0.035 of their time at 8 to 0.124 at 10 and 0.551 at 20. Their idle time while it waits for input or admission stays under 0.01.
3. **The reader is not a co-bottleneck, measured directly.** Its demand is 0.017–0.035 ms a block against the scanner's 0.099–0.130, at most about a third. Its copy-out time does double from 1 to 15 workers (0.031 → 0.057 s), which is memory contention. This settles the doubt raised in §61.
4. **Contention inflates service per visit.**
   - The scanner's Huffman decode (A3) costs 84 µs a block at 1 worker, 89 at 3–12, and 105–107 at 16 and above.
   - Worker emission (A7) costs 526 µs at 1, 592 at 8, 673 at 10, and peaks at 816 at 14.
   - Construction (A6a) rises from 215 to 353 µs.
   - Memory-controller reads per decode go 219 MiB → 807 (k = 8) → 1562 (10) → 1957 (16). That is about 8 GB/s at 10, far below DRAM's bandwidth, so this is an LLC-capacity effect rather than a bandwidth limit (argued).
   - IPC falls from 2.40 to 1.56 at 16.
5. **The scanner's slowdown at 16 and above is not frequency.** Its achieved MHz is 4900 at 16–18. It sat on one CPU 100% of the time at every count, while workers spread over all 20 logical CPUs from 17. So SMT sharing or cache interference is left (argued), which stage 2's placement contrast decides.
6. **Admission rarely binds above the knee.** Waits held by the window fall from 935 at 1 to about 10 at 9–14 and 0 at 16 and above. Waits held by the byte cap are 0–2. Ready bytes average at most 2.3 MB, peaking at 13.4 MB. Mean admitted blocks are about 20 at every count from 8 up, whatever the window (32–80).
7. **The routes confirm Neon's reading (§62):** 3648 + 80 Open rows (932 a copy) against 2400 "skipped" (n ≈ 30) and 768 "column" (n ≈ 8). So the 792 non-Open blocks *are* the tiny ones. Proved by route now, not argued.
8. **The public API, and a new finding.**
   - Asking for 10–20 threads builds 7 lazy workers with a 28-slot window: 0.209–0.223 s.
   - Asking for 8 or 9 builds 8 or 9 lazy workers: 0.1924 and 0.1902 s, 9–10% faster.
   - **Asking for 3, 5 or 7 builds *eager* workers**, because `Plan::lazy` starts at `LAZY_FROM_THREADS = 8` and the measured cells cover only 4, 6 and 10 or more. At 7 that is 0.337 s and 140 MB, against 0.209 s and 54 MB for 7 lazy workers: 61% slower and 2.6× the memory. At 5 it is 0.344 s and 130 MB, against 0.284 s for lazy (the stage-1 grid).
   - On other machines, where no measured cell applies, every count below 8 is eager.
   - The lazy kernel is better from about 4 workers up on this corpus; eager is still better at 1–2.

   This is a production default, so it needs an adoption registration with a recalibration across corpora. It goes into the programme's run 4 with the 7-against-8 question.

**Next:**
- stage 2's anchors, from its frozen rules (the Pathfinder);
- MONODROMY's analysis, once its plan (channel #39, part 2) is posted, written before it opens the bundle;
- the eager-below-8 finding, added to run 4.

## 67. The dynamical layer

Louis asked which dynamical-systems tools the programme should reach for, given its "digital on analogue" character. ChatGPT's answer is recorded verbatim (`external/chatgpt-dynamics.md`): hybrid dynamical systems, system identification, max-plus algebra, model reduction, and Hellerstein et al. on control of computing systems. The Coordinator's steer is the last section of `leads/structure-and-function.md`:
- stage 1 already verified the conservation laws exactly, and recorded synchronised per-block traces;
- occupancy alone is shown not to be a sufficient state above the knee;
- pulse interventions are added to stage 2's draft;
- system identification on stage 1's traces goes to MONODROMY (#40, question 6);
- Hellerstein et al. joins the reading list.

A third stage is added to the programme's method: performance analysis, then dynamical characterisation, then design-space exploration.

## 68. Hybrid Dynamical Systems: read, with levers

Louis supplied his private copy of Goebel, Sanfelice and Teel, *Hybrid Dynamical Systems* (2012). **It is not in the repository.**

**What was done.** The chapters were split and their text extracted on the command line. Each was read in full by one reader and given a précis in our own words. The notes are `reading/hybrid-dynamical-systems/`:
- `sections.tsv`;
- `precis/ch01…ch09`;
- `precis/extraction-notes.md`;
- the Coordinator's interpretation, separately, in `relevance/`.

ChatGPT's first reading is recorded verbatim (`external/chatgpt-hybrid-systems-book.md`).

**The interpretation** (`relevance/levers.md` first):
- **Five levers.** The decoder is $\mathcal H = (C, F, D, G)$. Every reorganisation changes one of five things:
  1. how fast work runs (the flow map);
  2. where work may proceed (the flow set);
  3. when events may happen (the jump set);
  4. what events do (the jump map);
  5. what is represented (the state space).

  Engineering pulls 1–4, mathematics mostly 5. Each current lead is placed on one.
- **The completion-time certificate.** The bottleneck bound, read as a Lyapunov-type function on the hybrid system, is a completion-time certificate. The book guarantees that such a function exists for systems that always terminate (Example 7.35). It is the cheap scorer a design-space enumeration needs.
- **Fragility at thin decision surfaces** (Chapter 4: Krasovskii = Hermes; hysteresis, Examples 4.18–4.19). This is the frame for the run-to-run divergence stage 1 shows: the bimodal scanner.
- **Six computations on the traces** (`relevance/measurements.md`), assigned to MONODROMY, the Pathfinder and the Coordinator:
  1. the trace as a hybrid arc;
  2. token-bucket event-rate bounds;
  3. the certificate;
  4. sensitivity at decision surfaces;
  5. the steady regime;
  6. homogeneity, and the knee's geometry.
- **Limits:** most of the book's stability theory assumes runs that never end. What transfers directly is the solution concept, perturbation and regularisation, well-posedness, and the finite-time certificate.

## 69. ChatGPT's hybrid-systems notes merged; the scanner fix re-priced

ChatGPT did the same reading task at Louis's request, from the public Chapter 1 sample only, and committed `docs/hybrid-systems-notes` (894b676e). It is merged into this branch at `methodology/hybrid-systems/`. Its report is recorded verbatim in `external/chatgpt-hybrid-systems-commit.md`. The two collections cross-link, and the comparison is at the end of `reading/hybrid-dynamical-systems/relevance/README.md`. Adopted from it:
- **an ownership lever:** compute credits apart from memory reservations, and taking a buffer apart from consuming and returning it;
- **replay is not prediction:** the certificate must predict held-out pulses and rounds, not replay them;
- **termination is not a time bound:** the assumptions are stated with the certificate;
- **a re-pricing of the scanner plumbing fix on stage 1's numbers.** At 8 workers the workers are within 0.3 ms of the scanner, so a 15 ms scanner cut is almost entirely masked there (0.1941 s against 0.1944 s). At 10 workers it gives about 7.5%. That revises §62's "about 6% at 8", which used the scan profile's lower worker time. Stage 1's figures are instrumented, so run 2's paired A/B at 8, 9 and 10 decides. A joint cut, the scanner plus 20% of emission, gives 0.180 s at 8 (conditional).

## 70. CORRECTION to §68: the bottleneck maximum is a lower bound, not a certificate

ChatGPT reviewed the hybrid-systems notes at 771e43086. The review is recorded verbatim in `external/chatgpt-hybrid-systems-review.md`.

**The main error, which was mine.** §68 and the relevance notes said that the bottleneck maximum $\max(S, W/k, R)$, read as a Lyapunov-type function, is a completion-time certificate, citing Example 7.35. **It is a lower bound.** One block with a 1 s scan, a 1 s decode and a 1 s delivery, done in series, has maximum 1 and takes 3 s. The maximum ignores dependencies, starvation, indivisible jobs and blocking. Example 7.35 also needs hypotheses (a compact flow set, the basic conditions, no complete solutions) that "always terminates" does not give: ChatGPT's two-line example terminates everywhere with no continuous bound.

**Corrected** in `reading/hybrid-dynamical-systems/relevance/`:
- **Three quantities, kept apart:** lower bounds (to prune), validated predictors (to rank; tested on held-out pulses, never by replaying a run with its own durations), and proved upper certificates.
- **A concrete route to the third:** backward induction on a finite, acyclic state graph with bounded edge durations.

**Other corrections applied:**
- **The state table:** the `q_*` trace columns are positions in the event order, not counts.
- **The event table** now matches the code: admission is checked before scanning, and `issued` advances after it; ownership guards are on finish and consumed; success, error, cancellation and deadlock are kept apart.
- **Discrete guards:** the admission guard is an exact integer test under a lock, not a noisy threshold. Hysteresis is a tool for future adaptive policies only, and `LAZY_FROM_THREADS` is construction-time configuration.
- **Levers:** the split "engineering pulls levers 1–4, mathematics 5" is dropped.
- **`measurements.md`:**
  - the token-bucket frontier is the right way round (burst as a function of rate);
  - divergence between runs is a hypothesis to test by intervention;
  - "steady regime" becomes a description of finite operating regimes, not invariance;
  - scaling is narrowed to duration scaling in a fixed model.
- **The précis:**
  - the authors' published errata are applied: the hybrid time domain definition; p. 34; Example 6.20's invariance; Example 5.13; Lemma 7.17; and others;
  - Example 6.4's flow is verified as $\dot x_1 = x_3 x_1^2$;
  - three points about the book as printed are flagged: Examples 3.1 and 3.23, and Theorem 5.29(b);
  - the speculative "candidate uses" sections are moved to `relevance/candidate-uses.md`, with corrections;
  - `sections.tsv` gains 1.4.1–1.4.4;
  - `extraction-notes.md` now marks each place as an erratum applied, verified, a point about the book, or unverified.

**Downstream:** MONODROMY's queued #40 is corrected before dispatch. Its items 1, 3 and 5 now carry no claim that a certificate already exists.

## 71. Stage 1 analysed by program: lower bound, bursts, starvation

`math-targets/scripts/stage1_analysis.py` (run on the bundle, outputs in `math-targets/data/character-stage1/analysis/`). It implements `measurements.md` items 2, 3 (the lower bound) and 5. A first version had the burst recursion the wrong way round (a minimum where a maximum is needed): every burst came out as 1. It was caught by a three-event check and fixed before any result was used.
- **The lower bound is tight below the knee, looser at it** (measured, instrumented decodes). The wall over $L = \max(S, \bar W, R)$ has a median of 1.002–1.014 up to 6 workers, 1.04–1.05 at 8–11, and 1.02 at 16–20. Dependencies, starvation and blocking cost most where the bottleneck switches.
- **Issuing is smooth; taking is bursty: the signature of in-order resequencing.**
  - At a rate allowance of 1.5× the mean, the scanner's issue process needs a burst of about 3 at every count.
  - Reader takes need about 13 at 8 workers and about 25 at 10 and 20. When a slow head-of-line block finishes, a run of ready blocks is taken at once.
  - The head-of-line wait's 99th percentile grows from about 0.05 ms (1–8 workers) to 0.12–0.23 ms (10–16).
- **Starvation episodes** (a worker idle more than 10 µs between its block's ready and its next start): 148 a decode at 8 workers, 551 at 10, about 1530 at 16–20, with the total idle worker-time rising from 46 ms to 2.7 s.
- **The progress rate** (blocks consumed a millisecond, in 20 windows) varies by 8–16% across the middle of a decode at every count.
- **The causal order** issued < enqueued < got < ready < taken < consumed holds per block in all 80 traced decodes.

These are descriptions of one corpus's traces, not guarantees. The burst frontier is not a window size without a service curve.

## 72. MONODROMY's records 129 and 130: the fourth reading, the analysis plan, and the design space

MONODROMY answered channel #39 (record 129, M-36) and #40 (record 130, M-37), on `claude/sharp-meitner-cx0v78`: `docs/journal/monodromy/2026-11-25-course-mining-and-analysis-plan.md` and `2026-11-25-design-space-and-identification.md`. Record 129's analysis plan was committed at 00:08 UTC, before stage 1 ran at 00:11. Record 130 opened stage 1's bundle only for its question 6.

**Record 129, the fourth course mining.**
- **The scan is a dependent chain at its latency floor:**
  - 1.41–1.51 ns a symbol is about 7 cycles, one first-level cache load's latency a symbol;
  - the lever is fewer *lookups* a symbol: compose the decode step, several symbols a lookup;
  - floor: lookups ≥ bits/b, which at b = 10 is about 2.6 symbols a lookup;
  - predicted 13–17% off the best wall (conjectured);
  - the cheapest test is cycles a symbol on the scanner at k = 1.
- **Under pooling,** where each worker scans its own block, every target's elasticity becomes its share of total demand. Worker mathematics then pays at every count.
- **The saturation law inside a worker** turns L\* = (ℓ + c)/c into the exchange rate between dependent depth and dependent reads.
- **A registered analysis plan for stages 1 and 2,** with rules for "points to mathematics" against "points to engineering", and a revisit list: ideas rejected while the scanner masked them, to re-qualify once the scanner's share of control falls below one half.

**Record 130, the design space** (scanner–worker obligations × placements, scored by the two-station law):
- an index of block starts, with each worker scanning its own block, gives −15%;
- composed symbol decoding gives −17%;
- both together give −19% (all conjectured).

The separating principle: put an obligation on a serial path only if its state has no small transformer, and cut the path wherever the format resets that state.
- **Within-block parallel Huffman decoding is ruled out.** The symbol count (it selects the table every 50 symbols) never re-synchronises, so a chunk carries at least 50 hypotheses, and a k-way decode loses for every k under 50 (argued; the obstruction is proved from the format).
- **The laws:** fusion removes intermediate objects; tupling *is* enrichment along composition; the third homomorphism theorem is the existence of chunk summaries (MTF/RLE2, CRC). The parse and the walk are dependent chains whose only contraction is composition, at an exponential table cost.
- **Max-plus:** the pipeline's recurrence, given each block's realised service times, reproduces stage 1's walls within 0.2% up to 7 workers and 1.3% above. That is a check of the bookkeeping, not a prediction (§70's distinction).
- **Identification:** two scalars per count, worker inflation and scanner inflation, explain the walls (0.3–2.5% above the knee). Inflation follows threads present, not threads busy. The conjectured reduced coordinate is the chance a thread shares a core; stage 2's placement contrast tests it.
- **The targets:** construction is 45% of a real block's worker time at one worker, emission 38%, MTF and runs 10%, RLE1 and CRC 6%. Construction is the largest mathematical target.
- **The referee's class:** exhaustive enumeration is infeasible (about 10⁵⁰ tables). A SAT encoding could settle a positive necessity statement in a bounded class: "any inversion within K sweeps and s cursors makes at least h₁ − c addressed accesses". Either answer is new.

**Decisions:**
- multi-symbol (composed) Huffman decoding becomes an engineering lead, to be prototyped as a PR after the current builds;
- the block-start index (pooling) is the larger structural lead, record 122 §3;
- the SAT necessity statement goes to MONODROMY's queue.

## 73. The scanner plumbing fix on CI: split, and held for the workstation

**#846's first head, 36084e6e (both fixes), failed PERFORMANCE-POLICY.md on CI's bench** (4 cores):
- wikidata at 4 threads: −3.0% (faster), the primary gain;
- onebyte at 4 threads: +8.0% wall and +8.8% CPU, a secondary loss over 5% that is not covered by twice the primary gain;
- peak RSS up 8–20% at 2–4 threads on several corpora (wikidata +10.6%, runs +19.9%).

The memory growth and the onebyte loss come from the input-window fix's 2 MiB headroom in the feeder's chunks.

**The split:**
- **#846 now carries only the symbol-buffer fix** (793ed64). Its CI bench shows no regression and no memory growth: random −3.1% and records −2.0% at 1 thread (secondary gains), and no primary change beyond noise.
- **The input-window fix is being redesigned without headroom,** through a reused seam buffer, as a new PR.

**Decision:** #846 cannot merge on CI, since it shows no primary gain there. That was expected: the fix saves scanner time, and the scanner limits only above about 8 workers, which CI's 4-core runner can't reach. The workstation A/B, `bench-scanner-plumbing` (in preparation), decides it at 6–12 threads under the house rule. **No release overnight** (Louis).

## 74. Stage 2a: which activity controls the wall, at each count

**The run:** `character-stage2/run-20261125T020224Z`, at 1901c3bb (#843). It is evidence 54896f60 on the `evidence` branch, SHA-256 64162665…0973, checked.
- **Every phase passed except the report.** The run took 19.5 min and the oracle 50 s; the oracle's check found 0 mismatches over 932 supplied blocks.
- **The report crashed** on a NaN coefficient: at k 16, A9's and A10's achieved doses were ≤ 0, so `interval()` was given NaNs and `statistics.stdev` raised.
- **I re-ran the report here** from the bundle's own `runs.jsonl` and `plan.json`, with one fix: `interval()` returns NaN for any non-finite input. The diff is in the LOG on #145 and goes to the Pathfinder for stage 2b. No data was re-collected or altered.
- **Where things are:** the report, `stage2.tsv` and `phases.tsv` are in `math-targets/data/character-stage2/`. The analysis program is `math-targets/scripts/stage2_analysis.py`, and its outputs are in `data/character-stage2/analysis/`.

**The screen** (+20% on one activity at a time; C is the elasticity of the wall, mean log(T/T0) / log(1 + achieved dose), over 6 blocks):

| k | controls (interval above 0.10) | negligible | the rest |
|---|---|---|---|
| 6 | construction A6a 0.14, A6b 0.15; emission A7 0.37 | the scanner (A2, A3), the reader (A10), A9 | A5, A6c unresolved |
| 8 | the scanner's main activity A3: 0.53 [0.14, 0.93] | none | all unresolved, with wide intervals |
| 10 | A3: 0.74 [0.37, 1.11] | none | A7 −0.30 (interval below 0: speeding the scanner's competitor? unexplained) |
| 16 | nothing resolved | none | four doses missed by over 25%; A9 and A10 got doses ≤ 0 |

- **Below the knee** (k 6), the workers' construction and emission set the wall, and the scanner doesn't matter.
- **At and above it** (8, 10), the scanner's main activity does, and nothing in the workers resolves.
- **This is stage 1's switch at 8,** now shown by intervention rather than by inference from busy times.

**The surface at k\* = 8** (both surfaces dosed together, 0/10/20%; `analysis/surface.tsv`):
- the scanner's elasticity, workers undosed: 0.87 at +10%, 0.93 at +20%;
- the workers' elasticity, scanner undosed: 0.50 and 0.64;
- the knee's roundness is 0.54 [0.46, 0.67], about the 0.5 a sharp maximum gives;
- with both dosed, the slowdown is no more than the scanner's alone (0.154 against 0.170 at 20/20). That is the max-plus shape: k 8 sits at the balance point, with the scanner just binding.

**The held-out test** (the combined arm, A3 and A7 both +10%, never used in a fit; `analysis/validation.tsv`). The screen's coefficients, composed log-additively, predicted:

| k | predicted | observed |
|---|---|---|
| 8 | 1.053 [0.994, 1.114] | 1.052 [0.963, 1.149] |
| 10 | 1.042 [1.003, 1.084] | 1.061 [1.041, 1.081] |

Both are inside their intervals. This is the first validated predictor of the programme, in §70's sense: a local one, for slowing, at 8 and 10.

**The pulses** (A3 or A7 +50% over places 600–800):
- **Slowing the scanner starves the workers.** At 8, the reader's rate falls to 0.69 of base, and the job queue empties: mean depth 4.4 before, 0.3 during.
- **Slowing emission fills the queue instead:** 3.9 to 8.9, with the rate at 0.82.
- **Recovery takes 2–4 ms.** The same holds at 10. This is the dynamical signature of a scanner-bound pipeline.

**The contrasts:**
- **Gates:** widening the window (4k to 8k) or the byte cap (12 to 64 MiB) at 8 changes nothing beyond noise (0.97–1.00, intervals across 1). This confirms §66: the gates rarely bind.
- **Memory interference is large:**
  - a co-running 2 MiB stream slows the wall 31% at 8 and 39% at 20;
  - a pointer chase slows it 110% at 8 and 34% at 20.
  
  The decoder is very sensitive to cache contention: a robustness finding, and a lever 1 target (`levers.md`).
- **Placement:** at 4 workers, putting them on SMT siblings rather than distinct cores costs 15% [11, 19]%.
- **Ledgers' cost:** within noise at 6, 8 and 16; PERTURBED at 10 (1.053), as in stage 1. The walls are from ledger-off decodes throughout.

**The oracle arms are VOID for comparison with this binary.**
- The four-arm binary's arm A runs 10.9% off this binary's baseline at k\*, past the 3% layout check.
- Within the four-arm binary, D/A is 0.747 [0.732, 0.762], replicating the four-arm test's 0.741, and C/A is 0.874.
- Those ratios stand only as internal to that binary.

**What it prices** (`analysis/pricing.tsv`). Cutting the scanner's main activity by 8% (about the plumbing fix's share, §57) implies a wall of:
- 0.957 at 8 (0.989 at the coefficient's lower bound);
- 0.940 at 10 (0.970 at the lower bound).

This is an *optimistic* reading: the screen measured slowing, and a speed-up of the binding activity buys at most its elasticity, less once the workers bind (bottleneck migration, §61). At 8 the workers are close behind (the surface), so the plumbing fix's gain there is likely under 4%. At 10 it is likely nearer 5%.
- **On the disagreement in the writer brief** (Neon's 6% at 8 against ChatGPT's nearly nothing), this sits in between, with 10 the better count to see it.
- **`bench-scanner-plumbing` (#850) decides it.**

**Decisions:**
- **The report fix goes to the Pathfinder,** for stage 2b's head.
- **Stage 2b,** as the Pathfinder sizes it from this run:
  - drops k 16's screen, which resolved nothing at 6 blocks;
  - re-runs the oracle with the four-arm binary rebuilt on this binary's layout, or drops the layout comparison.
- **Memory contention becomes a named lead:** the decoder's working set against a shared L3. It is placed under lever 1 and lever 4b (ownership of buffers).
- **Multi-symbol Huffman decoding is a worker lever.** This run says it lowers the knee (k 6's controllers are construction and emission) but does not raise the top speed. Above 8, only the scanner's work does. *(Wrong: corrected in §75. The Huffman decode is the scanner's A3, so multi-symbol decoding is the scanner lever above the knee.)*

## 75. CORRECTION to §74: multi-symbol Huffman decoding is the scanner's lever, not the workers'

**What §74 said:** multi-symbol Huffman decoding is a worker lever that lowers the knee but cannot raise the top speed.

**That is wrong.** The Huffman symbol decode is the scanner's activity A3 (`briefs/character.md`, the coordinate table). It is 81% of the scanner's work (§57), at about 1.5 ns a symbol.
- Stage 2a's screen names A3 as the *only* controller at 8 and 10 workers (0.53 and 0.74).
- So a faster Huffman decode is exactly the lever that raises the ceiling above the knee. It is the same lever as the scanner plumbing fix, and larger: record 129's −13–17% applies to A3 itself.
- Below the knee (k 6), A3 is negligible, so the decode's speed there doesn't matter.

**How it was caught:** the writer's draft of the post's stage 2 section (#855) labelled A3 "the scanner's Huffman decode". That sent me back to the coordinate table, and I found my error. §74's journal text called A3 only "the scanner's main activity" and then reasoned from the wrong owner.

**What changes:**
- **Multi-symbol (composed) Huffman decoding moves up.** It is now the first engineering lead for the top speed, ahead of the block-start index. The prototype follows the current builds, as decided in §72.
- **Its expected effect at 8–10 workers**, by the screen's coefficients (optimistic: slowing elasticities; stage 2b's speed-up arms measure the real share):
  - wall × (1 − c)^C, with c the cut in A3 and C = 0.53 at 8 and 0.74 at 10;
  - a 15% cut gives 0.917 at 8 and 0.887 at 10.
- **The LOG on #145 (7150763760) didn't make this claim.** The post's text (#855) is right.

**Also recorded:** Louis's rule of 25 November, in CLAUDE.md through #855: no redirects or moved-page stubs, ever, unless he asks. The stub on #848 was added at my request: I passed the old-URL concern to the writer as an instruction. That was the miscommunication his rule names.

## 76. The kernel count: lazy wins from 4 workers, the API builds eager at odd counts, and the cap of 7 costs 4–6%

**The run:** `kernel-count/run-20261125T022939Z`, at 97d5cfa6 (#849), master pinned at 16aaee3b. It is evidence 15646bfc, SHA-256 735398e1…a0a5, checked.
- 1,044 timed decodes, none failed, one output hash; status COMPLETE; DIAGNOSTIC.
- The report and its TSVs are in `math-targets/data/kernel-count/`.

**A labelling error in the report's table 1:** the column headed "RSS lazy/eager" holds the CPU ratio, and the one headed "CPU lazy/eager" holds RSS.
- `ratios.tsv` has them right: at k 1 the CPU ratio is 1.101 (1.591 s against 1.443 s), and RSS is 34.7 MB against 46.1 MB.
- The figures below are from `ratios.tsv` and the medians. The recipe's table is to be fixed if it is used again.

**1. Lazy against eager, explicit k workers** (wall, lazy/eager, paired over 12 rounds):

| k | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|
| wall | 1.123 | 1.046 | 1.000 | 0.922 | 0.834 | 0.737 | 0.639 | 0.546 | 0.488 |
| RSS (median MB) | 34.7/46.1 | 39.5/63.2 | 43.3/86.8 | 47.2/95.7 | 50.7/120.6 | 54.2/134.2 | 57.2/145.4 | 59.3/165.3 | 64.6/187.2 |

- **The crossover is at 3,** a tie. Eager is faster only at 1 and 2 workers (12% and 5%).
- **Lazy uses less memory at every k:** 25% less at 1, 65% less at 9.
- **Eager stops scaling at 6:** its wall rises from 0.334 s at 6 to 0.429 s at 9. Lazy keeps falling until 8.

**2. The public API (`BzReader::parallel(r, t)`, as shipped) builds the eager kernel at 3, 5 and 7 threads,** and lazy at 4, 6, 8 and 9. Its walls match the explicit builds (ratios within 1.000 ± 0.02).
- At 5 threads, the API is 0.349 s, against 0.289 s with lazy workers: **17% slower**.
- At 7 threads, it is 0.345 s against 0.215 s: **38% slower**, and slower than the API at 6.
- **At 3 threads there is no cost** (a tie).
- **The cause is to be read from `src/parallel.rs`:** the plan below `LAZY_FROM_THREADS` = 8, and why 4 and 6 come out lazy. It looks like a configuration accident, not a design.

**3. The cap:**
- Asked for 10 to 12 threads, the API builds 7 lazy workers (the i9 trim). That is 1.044–1.066 times lazy 8's wall.
- Explicit lazy 7 is 1.049 [1.002, 1.098] times lazy 8, and lazy 10 over lazy 7 is 0.954 [0.920, 0.989].
- **8, 9 and 10 workers tie on wall** (0.996, 1.001 against 8). Memory and CPU rise with k: at 10, RSS is +13% and CPU +15% over 8.
- **So 8 is the cheapest count at the top speed.** Trimming to 8 rather than 7 would gain about 4–6% at the default thread counts on the i9, for about +4% RSS and +5% CPU. This agrees with stage 1's k_balance = 8 and stage 2a's switch at 8.

**Decisions:**
- **A configuration PR follows:** lazy workers for every plan from 4 threads (the odd counts included), and the i9 trim at 8 workers, not 7.
  - These are production defaults, so they go to the workstation A/B under PERFORMANCE-POLICY.md, judged at 5, 7 and 10 threads.
  - CI's 4-core bench can't see either change: CI runs at most 4 threads, and 4 is already lazy. So it needs the workstation, like #846.
- **The trim interacts with the scanner work:** with the scanner faster (the plumbing fix, multi-symbol decoding), the balance moves above 8. Each change is priced at the count it will run at (§61, bottleneck migration).

## 77. The scanner's symbol-buffer fix (#846) on the workstation: NOT ADOPTED, on a noisy judged count

**The run:** `scanner-plumbing/run-20261125T025030Z`, recipe #850 at f5868d45, head #846 793ed644 against its merge-base. It is evidence 78a85fc4, SHA-256 f52bb59e…32a8, checked.
- Every phase passed, and every output was byte-identical.
- The verdict and summaries are in `math-targets/data/scanner-plumbing/`.

| threads | workers | wall head/base [95%] | step | peak RSS |
|---|---|---|---|---|
| 6 | 6 | 1.001 [0.995, 1.007] | INCONCLUSIVE | 0.995 |
| 8 | 8 | 0.974 [0.949, 1.000] | FASTER | 1.010 |
| **9 (judged)** | 9 | **0.987 [0.936, 1.041]** | INCONCLUSIVE | 1.008 |
| 10 | 7 | 0.989 [0.963, 1.015] | INCONCLUSIVE | 1.002 |
| 12 | 7 | 0.985 [0.964, 1.006] | INCONCLUSIVE | 1.003 |

**The lazy diagnostic** (explicit workers): 1.001 at 8, 0.986 at 9, 0.976 at 10. All are inconclusive, with intervals of ±4–7%.

**The verdict under the frozen rule: NOT ADOPTED,** because 9 threads was not FASTER. The rule stands (C2): #846 does not merge on this run.

**What the data says, descriptively:**
- **The direction matches the prediction.** Every count from 8 up has a point estimate of 0.974–0.989, against a predicted ≈0.974 at 9. The judged count's interval is ±5%, the widest in the run.
- **That width is the noise stage 2b was built to remove:** 2a found the scanner sharing its core about half the time above 6 workers (paired SD 8.1% at 8). At 6, below the knee, the interval is ±0.6% and centred on 1.000, as predicted.
- **No cost:** RSS within 1%, CPU within noise.

**The next test, registered now, before its data** (C1a, the house rule, same recipe):
- **What runs:** the *combined* scanner fix, #846's symbol buffers plus #857's seam window, as one head on a branch that merges both. It runs once #857's CI is green.
- **The rule:** identical to #850's, judged at 9 threads.
- **Why combined:** the two fixes act on the same activity, the scanner's copying. Their predicted sum (≈0.92–0.95 at 9) clears the unpinned noise that a 2.6% effect can't.
- **If it ADOPTS,** both merge together. If not, both are held, pending the multi-symbol Huffman decode, the larger lever on the same activity (§75).
- **#846 stays open and unmerged until then.**

## 78. Stage 2b: the scanner pinned. A3 controls at 8 and 10; the wall is a sharp maximum, not a sum; a faster scanner barely helps at 8; core sharing is the largest cost above the knee

**The run:** `character-stage2b/run-20261125T030124Z`, at #843's 8423a82b. It is evidence a4f0dcef, SHA-256 59bacb7b…d87c, checked.
- Every phase passed, and the report exited 0.
- The scanner ran alone on its pinned CPU (6, sibling 16 idle) in 86 of 86 pinned traces.
- 16 blocks.
- **Where things are:** the report, `stage2.tsv`, `cores.json` and the analysis (`stage2_analysis.py`, extended with `link.tsv` and a max-model column) are in `math-targets/data/character-stage2b/`.

**1. The screen, pinned.** The intervals are 2–4× narrower than 2a's.

| k | A3 (Huffman decode) | A7 (emission) | everything else |
|---|---|---|---|
| 8 | **0.795 [0.681, 0.908] CONTROLS** | 0.215 [0.066, 0.363] | within ±0.15, unresolved |
| 10 | **0.577 [0.420, 0.734] CONTROLS** | −0.224 [−0.414, −0.034] | unresolved; several negative |

- **The Huffman decode alone controls the wall at both counts.**
- **The negative worker coefficients at 10** (A7, A9 and A6b below 0) mean that slowing a worker activity *speeds* the wall. This is consistent with contention: slower workers press less on the shared cores and caches that the scanner also uses (see 4). It is unexplained in detail.

**2. The surface at 8:**
- **Roundness 0.500 [0.454, 0.548]:** the sharp maximum, now resolved.
- **The scanner's elasticity:** 1.05–1.08, workers undosed.
- **The workers':** 0.64–0.69, scanner undosed.
- **With both dosed 20%,** the slowdown (0.162) is *less* than the scanner's alone (0.198).

**3. The held-out arm (A3+A7 at +10%): log-additivity fails, a maximum fits** (`analysis/validation.tsv`):

| k | observed | log-additive prediction | max prediction |
|---|---|---|---|
| 8 | 1.069 [1.049, 1.090] | 1.101 [1.082, 1.121], **outside** | 1.079 [1.067, 1.090], inside |
| 10 | 1.063 [1.031, 1.097] | 1.034 [1.010, 1.059], **outside** | 1.057 [1.041, 1.072], inside |

- **What this corrects:** §74 called the log-additive model "the first validated predictor". 2b's precision refutes it. 2a's pass came from wide intervals.
- **The max model** (the larger coefficient alone) is inside in all four tests, 2a and 2b. But it was chosen *after* seeing this failure, so it is a hypothesis for the next held-out test, not a validated predictor.
- **What supports it beforehand:** the surfaces' roundness of 0.5, measured on other cells, before this comparison.

**4. Placement and memory: the largest effects in the run.**
- **Workers on SMT siblings rather than distinct cores:** +22% [17, 28]% at 4 workers, and **+48% [45, 51]% at 8**.
- **Memory pressure, as in 2a:**
  - a 2 MiB stream: +33% at 8, +38% at 20;
  - a pointer chase: +113% at 8, +31% at 20.
- **What follows:** above the knee, *where threads sit* costs as much as any activity's work. The i9 has 10 cores. 8 workers plus the scanner, reader and feeder is 11 threads, so some sharing is unavoidable at 8, and certain at 10.
- **This is the likeliest reading of H < 1** (5): pacing every activity doesn't scale contention.

**5. Speeding the scanner** (homogeneity; registered rule: e_up is interpreted only if H's interval is within [0.85, 1.15]):

| k | λ | H | e_down | e_up | label |
|---|---|---|---|---|---|
| 8 | 10% | 0.788 [0.594, 0.983] | 0.716 | 0.134 [−0.105, 0.373] | NOT-HOMOGENEOUS, ASYMMETRIC |
| 8 | 20% | 0.850 [0.723, 0.978] | 0.971 | 0.181 [0.117, 0.245] | NOT-HOMOGENEOUS, ASYMMETRIC |
| 10 | 10% | 0.445 | 0.555 | 0.509 | NOT-HOMOGENEOUS |
| 10 | 20% | 0.565 | 0.727 | 0.390 | NOT-HOMOGENEOUS, ASYMMETRIC |

- **Under the registered rule, e_up is not interpreted:** the wall is not homogeneous of degree 1 in the paced activities. Pacing everything by 10% slows it by about 7.8% at 8 workers and 4.3% at 10.
- **Descriptively, as a sharp maximum predicts:** at 8, where the workers are close behind, slowing the scanner costs about 0.7–1.0 per unit, but speeding it gains about 0.13–0.18 per unit. The bottleneck migrates at once. At 10 the asymmetry is smaller.

**6. Pinning the scanner is itself a lever** (`analysis/link.tsv`):
- the pinned base runs **0.960 [0.931, 0.990]** of the unpinned base at 8;
- A3's +20% costs 1.141 unpinned against 1.178 pinned. Production's unpinned scanner is a little less sensitive, as expected when part of its time is lost to sharing.

**7. MONODROMY's pre-registered prediction** (record 131, committed at 02:59:18Z, before this run started at 03:01:24Z), scored by its own rules:
- **NOT-HOMOGENEOUS:** it gave this about 1 in 3. With it, only e_oth and e_down are scored.
- **e_oth** = H − e_up = 0.654, against 0.65 [0.50, 0.78]: **hit**.
- **e_down** = 0.716, against 0.79 [0.66, 0.90]: **hit**.
- **Not scored, for the record:** e_up was 0.134, inside its 0.12–0.48 at the low edge; H was 0.788, just below its 0.80–1.02; d was 0.58, above its 0.25.
- **Its assumptions:** it took pinning as 3% faster, and the measurement is 4.0%. Its soft-max exponent (p ≈ 21, near a sharp maximum) agrees with the roundness of 0.500.

**What changes:**
- **Every scanner speed-up is priced at the count it will run at, with the workers retuned.** At 8 workers, a faster Huffman decode gains little (e_up ≈ 0.15). The gain appears only with more workers, and more workers mean more core sharing (+48% for siblings).
  - So the multi-symbol decode's value depends on cutting core sharing too: fewer, faster workers, or the scanner given a quiet core.
  - This makes the combined test (#861) at 9 threads harder to pass than §77 assumed. Its registered rule stands.
- **Placement becomes a named lever**, with its largest measured effect: the scanner on a quiet core (−4% at 8), and workers kept off siblings. Any policy must work unpinned, as a library can't own the machine. The next step is a design question for MONODROMY and the Pathfinder: what placement can a library request?
- **The predictor** for the next held-out test is the max model, registered here before that test.

## 79. The combined scanner fix (#861) on the workstation: NOT ADOPTED again at 9; faster at the default counts; held for the new production regime

**The run:** `scanner-plumbing/run-20261125T033008Z`, recipe #850 at b39c2cfc, head #861 5413191e against merge-base 16aaee3b. It is evidence 33568fad, SHA-256 2eacffeb…8ef3, checked. Every phase passed. The data is in `math-targets/data/scanner-plumbing-combined/`.

| threads | workers | wall head/base [95%] | step | peak RSS |
|---|---|---|---|---|
| 6 | 6 | 0.996 [0.988, 1.004] | INCONCLUSIVE | 0.965 |
| 8 | 8 | 0.975 [0.942, 1.009] | INCONCLUSIVE | 0.984 |
| **9 (judged)** | 9 | **0.973 [0.936, 1.011]** | INCONCLUSIVE | 0.977 |
| 10 | 7 | 0.982 [0.966, 0.998] | FASTER | 0.967 |
| 12 | 7 | 0.977 [0.961, 0.995] | FASTER | 0.974 |

**The lazy diagnostic, at explicit workers** (it decides nothing):
- 0.991 at 8;
- **0.921 [0.884, 0.959] at 9**;
- **0.936 [0.889, 0.984] at 10**.

**The verdict under the frozen rule: NOT ADOPTED** (9 threads not FASTER). The rule stands (C2): #861, #846 and #857 do not merge on this run.

**Descriptively:**
- **The direction is the same in both runs** (§77, and this one): every count from 8 up is below 1. On production's default thread counts, 10 and 12, the fix is FASTER by about 2%, and peak memory falls 2–3.5% at every count.
- **At explicit 9 and 10 workers the gain is 6–8%,** the size §77 predicted before stage 2b lowered it.
- **The judged count, 9 threads, is the noisiest count** in both runs (±4–5%). This is the core-sharing noise stage 2b measured (§78).

**Decision:** no third run of this registration. Repeating the same test until it passes is optional stopping. The question changes instead:
- **#860 (the kernel choice, queued)** moves production to 8 workers from 10 threads. If it ADOPTS, the scanner fix's value has to be judged in *that* regime, where the scanner binds (8 workers, sharp maximum, §78).
- **So the next test of the scanner fix** is registered after #860's verdict, and before its own data:
  - #861 rebased on the new production;
  - judged at production's default counts (10 and 12 threads), where the fix acts;
  - its prediction taken from §78's max model.
- **Until then, #846, #857 and #861 stay open.**

## 80. The kernel choice (#860) on the workstation: NOT ADOPTED on 10 threads; lazy at 5 and 7 is −16% and −37% with 60% less memory; split and re-registered

**The run:** `kernel-choice/run-20261125T034713Z`, recipe #862 at 139e90ec, head #860 7c343200 against merge-base f8edc1ac. It is evidence b6ab31a6, SHA-256 6f569ccc…8baf, checked.
- Every phase passed.
- Each build's probe showed exactly its registered plan, and the timed runs used the probed worker counts.
- The data is in `math-targets/data/kernel-choice/`.

| threads | base plan | head plan | wall head/base [95%] | step | CPU | peak RSS |
|---|---|---|---|---|---|---|
| 3 | 3 eager | 3 eager | 0.999 [0.984, 1.014] | INCONCLUSIVE | 1.000 | 0.982 |
| 4 | 4 lazy | 4 lazy | 1.001 [0.995, 1.007] | INCONCLUSIVE | 1.001 | 1.001 |
| **5 (judged)** | 5 eager | 5 lazy | **0.845 [0.838, 0.852]** | FASTER | 0.849 | **0.418** |
| **7 (judged)** | 7 eager | 7 lazy | **0.634 [0.630, 0.638]** | FASTER | 0.642 | **0.388** |
| 8 | 8 lazy | 8 lazy | 0.996 [0.968, 1.024] | INCONCLUSIVE | 1.000 | 0.968 |
| **10 (judged)** | 7 lazy | 8 lazy | **0.979 [0.944, 1.016]** | INCONCLUSIVE | 1.079 | 1.053 |
| 12 | 7 lazy | 8 lazy | 0.903 [0.875, 0.931] | FASTER | 1.017 | 1.046 |

**The verdict under the frozen rule: NOT ADOPTED,** since 10 threads was not FASTER. #860 does not merge.

**The two changes, as the data separates them:**
- **Lazy at 4–7 threads** acts only at 5 and 7 (4 and 6 were lazy already).
  - The gain is −15.5% and −36.6%, with 58–61% less peak memory and 15–36% less CPU, exactly as §76 predicted (0.834, 0.639).
  - The trim never acts at those counts, so these two rows measure this change alone.
  - The counts where neither change acts (3, 4 and 8) are 1.000 ± 0.03, as they should be.
- **The trim to 8 workers from 10 threads** acts at 10 and 12.
  - At 10 it is inconclusive, at a CPU cost of +8%. At 12 it is FASTER (−10%).
  - It is the change the rule could not confirm.

**Decision: split, and test the lazy part afresh.** Adopting it on these rows would select after the data, which C2 forbids even when the rows are unambiguous. So it gets its own registration, written here before its data:
- **The head:** #860 without the trim. At 4–7 threads unpinned it runs the plan's own workers on the lazy kernel; from 10 threads, 7 lazy as on master.
- **The recipe:** `bench-kernel-choice` (#862), with the expected head plan as a parameter. The plan check stays: the run is VOID unless each build probes its registered plan (head: lazy at 4–7, 7/28 at 10 and 12).
- **The rule (the house rule):** ADOPTS iff 5 and 7 threads are FASTER, none of 3, 4, 8, 10 and 12 is SLOWER, and median peak RSS head/base ≤ 1.10 at every count. HELD ON MEMORY, NOT ADOPTED and VOID as in #862's registration.
- **The prediction:** ≈0.845 at 5 and ≈0.634 at 7; ≈1.00 elsewhere, where the builds are identical.
- **The trim to 8 is held,** to be re-asked together with the scanner fix (§79). Both act above the knee, and both are priced by §78's max model.

## 81. The workstation's noise is the CPU clock after idle: a 100 ms spin removes it

**Where it began.** The Pathfinder re-read stage 2b and found a residual noise that pinning didn't explain (paired SD 4.4% at 8 workers, 6.4% at 10). Its frequency analysis (LOG 7152489390, `bench/character_freq.py` on #843) found the clock has two levels. The clock is PMU cycles over reference cycles, per decode.
- **The levels:** 4.91 GHz, and a low level of 4.09–4.56 GHz.
- **How often:** about a quarter of decodes run low, scattered at random. There is no throttling.
- **How much it explains:** R² 0.96 of 2b's paired noise at 10, and 0.37 at 8.
- **Prior art:** journal 2026-10-28 §4 (`bench-regimes`) had already seen slow runs follow a 0.15 s idle gap, with the mechanism left open.

**The test:** `clock-mechanism/run-20261125T042759Z`, #843 at 1dfb11c8. It is evidence a1f1ddf8, SHA-256 bca16eb6…a756, checked; every phase passed.
- **The setup:** the pinned base at 10 workers, 48 randomised blocks.
- **The C1a rule,** posted before the data (REVIEW 7152567425): an arm is a CONTROL iff its low-clock share minus the 1 s settle's has a Newcombe 95% interval wholly below 0. LOW is a clock under 4.50 GHz.

| arm | before each decode | low share [95%] | vs S1 | label | clock median | wall / S1 [95%] | SD of log wall |
|---|---|---|---|---|---|---|---|
| S0 | nothing (back to back) | 0.00 [0.00, 0.07] | −0.40 | CONTROL | 4.91 | **0.943 [0.931, 0.955]** | **0.9%** |
| S0.2 | 0.2 s idle | 0.90 [0.78, 0.95] | +0.50 | WORSE | **2.96** | **1.450 [1.383, 1.520]** | 15.6% |
| S1 | 1 s idle (every recipe's settle) | 0.40 [0.27, 0.54] | ref | | 4.56 | 1 | 4.4% |
| S3 | 3 s idle | 0.12 [0.06, 0.25] | −0.27 | CONTROL | 4.91 | 0.970 [0.956, 0.985] | 3.6% |
| SPIN | 1 s idle, then 100 ms of spin on every CPU | 0.00 [0.00, 0.07] | −0.40 | CONTROL | 4.91 | **0.949 [0.938, 0.961]** | **1.1%** |

**What it means for measurement:**
- **Most of tonight's noise above the knee came from the settle itself.** A short idle drops the cores to a low P-state (the powersave governor under HWP), and they climb back slowly. After 0.2 s idle, a decode runs at 2.96 GHz and takes 45% longer. After 1 s, 40% of decodes are still caught.
- **This is why the scanner fix's judged count kept coming out ±4–5% wide** (§§77, 79). Those recipes settled 2 s unpinned.
- **A 100 ms spin before each timed decode removes the low state entirely,** and cuts the SD of log wall from 4.4% to 1.1%. A test then needs about 16 times fewer rounds for the same interval.
- **Decision (standard from now on):**
  - Every workstation recipe at or above the knee warms each timed decode with a 100 ms spin on every CPU, or runs decodes back to back.
  - It reports the clock (cycles over ref-cycles) per decode.
  - It splits each effect into a cycles ratio and a clock ratio as a registered secondary (the Pathfinder's proposal). The wall stays primary.

**What it means for the product (a question, not a finding):**
- A real program that decodes after a short pause (0.1–0.5 s) may see the S0.2 regime. That is +45% on this machine, with this governor, at 10 workers.
- That is not the decoder's doing. But it may matter more to users than any change measured tonight, and it varies by machine and governor.
- Whether and how a decoder should respond (a ramp-up of its own, or simply documenting it) is a design question. It goes to MONODROMY and the Researcher. Nothing is to be built without measurement on more machines.

**The tests that now need re-asking with the spin** (registered when their recipes are updated, before their data):
- the combined scanner fix (§79);
- the trim to 8 (§80);
- the multi-symbol Huffman decode, when ready.

**The H < 1 puzzle (§78):** the Pathfinder showed the paced arms shift the clock (+3–6% at 10). That accounts for part of the shortfall (all20's H to ≈0.73 of the observed 0.57). MONODROMY's conjecture, record 132, is that a paced spinning thread yields its core's sibling. Both mechanisms may act. With the spin warm-up removing the low state, a re-run of the speed-up arms separates them (MONODROMY's M-39: H ≥ 0.9 at 6 workers predicted).

## 82. NEGATIVE: three symbols a lookup in the scanner's Huffman decode (#870); and a correction to §75's premise

**The premise was already in master.** §75 and MONODROMY's record 129 priced multi-symbol decoding at −13–17% of scanner time, assuming one symbol a lookup. But master's root table (`src/huff.rs`, since #745) already holds up to two symbols an entry.
- **On wikidata** it takes 1.85 symbols a lookup: 13.49 M lookups for 24.98 M symbols, of which 84% are pairs, 15% singles and 1% the slow path.
- **What that leaves:** only the step beyond two symbols was left to test. **CORRECTION to §75:** the price of −13–17% was for a change already made.

**What #870 tried:**
- 64-bit entries holding up to three codes' symbols in 16-bit lanes, so tables grow from 4 to 8 KiB;
- batches of five entries within the 50-symbol group, and an exact tail at the group's end;
- one subtraction for each batch's bits.

The output is byte-identical on seven corpora at 1, 4 and 8 threads, as are the scanner's symbol streams. The tests check every fast entry bit by bit, and CI is green.

**The result: no gain.**
- **Lookups:** on wikidata they fall 12%.
- **Scanner time a symbol, new over master** (an in-process A/B, local and indicative): wikidata 0.98–1.01, text 0.97, records 1.09, random 1.04. Table construction costs 30–50% more.
- **CI's bench** (EPYC, 10 rounds): wikidata at 1 thread +0.5% slower, and +0.2% and −0.8% at 2 and 4 threads (no change). The scanner stage is +2.4% slower; records +6.8% and random +2.4%, both slower.
- **The likely cause:** a lookup costs about 13–15 cycles, twice its dependency chain, and the third symbol's gain is eaten by per-group mispredicts and spills in the tail paths. A simulated ceiling at 12-bit tables gives −16% of lookups, at 2–4× the table size and build cost.

**Decision: REJECTED** (queue issue's "Rejected" list). #870 closes with this write-up.

**A by-product kept as a lead:** the batched bit count alone, on master's two-symbol table, measured about −2% scanner time on every corpus (wikidata 0.980, records 0.986, random 0.974; local and in-process).
- **Why it matters:** with §78's elasticity of about 0.8 at 8 workers, it is worth about −1.5% of wall there.
- **What it waits for:** it is resolvable only with the spin warm-up (§81) at about a 1% SD. It waits for the `ab` pre-spin option, then gets a registered workstation A/B.

**Where the scanner's lever now stands:** the Huffman decode is still the only controller above the knee (§78). But its arithmetic has little slack left: two symbols a lookup, at about 7 ns per pair. What remains:
- the copies (#861, the seam window and the symbol buffers);
- the micro-win above;
- placement: the scanner's core kept unshared, −4% (§§78, 81);
- the structural answer, a poolable or parallel scan (MONODROMY's T4), which removes the single serial core rather than speeding it.

## 83. Lazy workers at 5 and 7 threads (#868): ADOPTS, and merged

**The run:** `kernel-choice/run-20261125T044116Z`, recipe #862 at 377a3557, expectation `lazy-four-to-seven`, head #868 084275e6 against merge-base f8edc1ac. It is evidence 94728a2c, SHA-256 e0425cba…3c13, checked.
- Every phase passed.
- Each build probed its registered plan.
- The data is in `math-targets/data/kernel-choice-split/`.

| threads | base → head plan | wall head/base [95%] | step | CPU | peak RSS |
|---|---|---|---|---|---|
| 3 | 3 eager → 3 eager | 0.999 [0.981, 1.018] | INCONCLUSIVE | 1.000 | 1.031 |
| 4 | 4 lazy → 4 lazy | 0.999 [0.992, 1.006] | INCONCLUSIVE | 1.000 | 1.001 |
| **5** | 5 eager → 5 lazy | **0.840 [0.831, 0.850]** | FASTER | 0.847 | **0.421** |
| **7** | 7 eager → 7 lazy | **0.631 [0.624, 0.639]** | FASTER | 0.642 | **0.373** |
| 8 | 8 lazy → 8 lazy | 0.974 [0.944, 1.005] | INCONCLUSIVE | 0.982 | 0.997 |
| 10 | 7 lazy → 7 lazy | 0.995 [0.984, 1.005] | INCONCLUSIVE | 0.995 | 1.001 |
| 12 | 7 lazy → 7 lazy | 1.002 [0.988, 1.017] | INCONCLUSIVE | 0.999 | 0.999 |

**The verdict under §80's frozen rule: ADOPTS.**
- 5 and 7 threads are FASTER, none of the supporting counts is SLOWER, and peak RSS is ≤ 1.10 at every count (largest 1.031, at 3).
- The result replicates §80's first measurement (0.845, 0.634) and §76's prediction (0.834, 0.639).

**Merged:** #868 at 084275e6, the head the bench measured, as 56c56573.
- CI was green on all 30 runs, Miri included, and head-sha passed before and after.
- **What changes for users:** on the i9 unpinned, `BzReader::parallel(r, 5)` is 16% faster with 58% less peak memory, and `parallel(r, 7)` is 37% faster with 63% less.
- **No release overnight** (Louis's rule). This waits for the release process.
- **#860 is closed:** its lazy part is merged here, and its trim to 8 is re-registered separately (§80) once the `ab` pre-spin (#874) lands.

**What is decided and what is open, this night:**
- **Adopted:** lazy at 5 and 7.
- **Held, to be re-asked with the spin warm-up (§81):**
  - the combined scanner fix (#861);
  - the trim to 8;
  - the batched bit count (§82).
- **Rejected:** three Huffman symbols a lookup (§82).
- **Open design questions:**
  - placement: what a library can request (§78, record 132);
  - the clock after idle: whether a decoder should respond (§81).

## 84. Three tests re-registered with the spin warm-up, before their data

`ab run --prespin-ms` is on master (#874, aa041e25), and the recipes take a PRESPIN argument (#850, #862, the Pathfinder). Each test below runs with PRESPIN 100 and records the clock per decode, with the cycles × clock split as a registered secondary (§81).
- **Sizing:** the paired SD expected with the spin is about 1% (§81: 1.1% at 10 workers).
- **Interval width:** 16 rounds give an interval of about ±0.6%.
- **Heads:** each head has master merged in (so it contains #868 and #874), and each base is that head's merge-base with master.

**The order, and why.** On the i9 unpinned, production runs 7 workers from 10 threads, below the knee of about 7.5. There the scanner doesn't bind, so a scanner fix can't show at production's default counts until the worker trim is decided. So:

**Test 1: the trim to 8 workers from 10 threads.**
- **What it is:** #860's change (b) alone, on a fresh branch off master.
- **The recipe:** `bench-kernel-choice` with expectation `trim-eight`. The head plan is 8/32 lazy at 10 and 12, and master's elsewhere (lazy at 4–7, from #868).
- **ADOPTS iff** 10 and 12 threads are FASTER, none of 3, 4, 5, 7 and 8 is SLOWER, and median peak RSS head/base ≤ 1.10 at every count.
- **Reported, not part of the rule:** the CPU ratio. §80 saw +8% at 10. PERFORMANCE-POLICY.md weighs it at merge.
- **Prediction (§80, unspun):** 0.979 at 10 and 0.903 at 12. With the spin, both resolve.

**Test 2: the combined scanner fix** (#861, the seam window and the symbol buffers), with master merged in.
- **The recipe:** `bench-scanner-plumbing`, PRESPIN 100.
- **ADOPTS iff** 10 and 12 threads are FASTER, none of 6, 8 and 9 is SLOWER, and peak RSS ≤ 1.10.
- **These judged counts replace 9:** §79 decided the judged counts move to production's defaults, decided before this data.
- **Prediction:**
  - at 7 workers (if test 1 fails), about 0.98 at 10 and 12, as §79 measured unspun;
  - at 8 workers (if test 1 adopts), between that and the explicit-worker diagnostic's 0.92–0.94 (§79).
- **It runs after test 1's verdict,** on whichever master results.

**Test 3: the batched bit count** (§82's by-product), on a fresh branch off master.
- **What it is:** each batch's consumed bits summed and subtracted once, on master's two-symbol table, with nothing else changed.
- **The recipe and rule:** the same as test 2, with the same counts.
- **Prediction:** about −2% of scanner time, so ≤ 1.5% of wall at 8 workers and less at 7. INCONCLUSIVE is likely at 7 workers.
- **It runs after test 2.**

**Each test is run once.** A verdict is final for its registration. Any further test is a new registration, with its reason stated.

## 85. §84 test 1, eight workers from 10 threads (#878): ADOPTS with the spin warm-up, and merged

**The first attempt** (`kernel-choice/run-20261125T055310Z`, evidence fb6ab984) failed in resolve after 22 s. That was a recipe bug, class A.
- **The bug:** the PRESPIN guard piped `git show` into `grep -q` under `set -o pipefail`. grep quit at its first match, `git show` took SIGPIPE, and the guard refused a head that had the option.
- **The fix:** the Pathfinder fixed it (`git grep -q`), with a test that reproduces the failure. The recipe went to 55a4a81d (#862) and a86e5d86 (#850).
- **No data was taken,** so the registration stood.

**The run:** `kernel-choice/run-20261125T061204Z`, recipe #862 at 55a4a81d, expectation `trim-eight`, PRESPIN 100. The head is #878 at cf61f483, against merge-base aa041e25 (master with #868 and #874). It is evidence 5d259c88, SHA-256 dbb88594…5716, checked. Every phase passed, and each build probed its registered plan. The data is in `math-targets/data/trim-eight/`.

| threads | base → head plan | wall head/base [95%] | step | CPU | peak RSS | clock ratio |
|---|---|---|---|---|---|---|
| 3 | 3 eager → 3 eager | 0.986 [0.966, 1.006] | INCONCLUSIVE | 0.987 | 0.972 | 1.000 |
| 4 | 4 lazy → 4 lazy | 0.999 [0.991, 1.007] | INCONCLUSIVE | 0.999 | 0.991 | 1.000 |
| 5 | 5 lazy → 5 lazy | 0.998 [0.994, 1.003] | INCONCLUSIVE | 0.999 | 0.997 | 1.000 |
| 7 | 7 lazy → 7 lazy | 1.003 [0.988, 1.018] | INCONCLUSIVE | 1.002 | 1.003 | 1.000 |
| 8 | 8 lazy → 8 lazy | 1.004 [0.972, 1.037] | INCONCLUSIVE | 1.002 | 1.005 | 1.000 |
| **10** | 7/28 → 8/32 lazy | **0.929 [0.908, 0.951]** | FASTER | 1.022 | 1.037 | 1.000 |
| **12** | 7/28 → 8/32 lazy | **0.932 [0.920, 0.945]** | FASTER | 1.033 | 1.046 | 1.000 |

**The verdict under §84's frozen rule: ADOPTS.** 10 and 12 threads are FASTER, no supporting count is SLOWER, and peak RSS is ≤ 1.10 (1.046 at most).
- **Weighed under PERFORMANCE-POLICY.md:** CPU +2–3% and RSS +4–5%, both secondary and within its limits, for −7% wall at the default counts.

**Merged:** #878 at cf61f483, the head the bench measured, as 959496c7.
- CI was green on all 30 runs, and head-sha passed before and after.
- **What changes for users:** on the i9 unpinned, `parallel(r, t)` for t ≥ 10 runs 8 lazy workers, not 7. That is about −7% wall at 10 and 12 threads, which includes the Python default (20).
- **No release yet:** it waits for the release process.

**What the spin bought:**
- **The clock was neutralised:** the clock ratio is 1.0000 at every count, so the decomposition puts the whole effect on the work, as it should for a worker-count change.
- **The intervals narrowed:** at 10 threads the interval is ±2.2%, against ±3.6% unspun (§80), where the same change was INCONCLUSIVE at 0.979.
- **Why the two runs differ:** the effect itself looks larger with the spin (0.929 against 0.979). An unspun run mixes in decodes at the low clock, where an extra worker matters less, since everything is slower. That reading is consistent with §81, but not tested separately.

**What follows:**
- **The production regime at 10 threads and more is now 8 workers,** at the knee, where the scanner binds (§78).
- **§84 tests 2 and 3,** the scanner fixes, are now judged in that regime.
- **Test 3's head** (#879) has master 959496c7 merged in, at 48d9f7d9. It is queued once CI is green.
- **Test 2** waits for PR 861 to be reopened (coordinator notes, #881).

## 86. §84 test 3, the batched bit count (PR 879): NOT ADOPTED; the noise left at the knee is not the clock

**The run:** `scanner-plumbing/run-20261125T064054Z`, recipe #850 at a86e5d86, PRESPIN 100. The head is PR 879 at 48d9f7d9, with master 959496c7 merged in, so production runs 8 workers from 10 threads. It is evidence a93afab5, SHA-256 32065ffe…b3d2, checked; every phase passed. The data is in `math-targets/data/batched-bits/`.

| threads | workers | wall head/base [95%] | step | clock ratio |
|---|---|---|---|---|
| 6 | 6 | 0.998 [0.992, 1.003] | INCONCLUSIVE | 1.000 |
| 8 | 8 | 0.995 [0.972, 1.018] | INCONCLUSIVE | 1.000 |
| 9 | 9 | 0.978 [0.956, 1.002] | INCONCLUSIVE | 1.000 |
| **10 (judged)** | 8 | **1.001 [0.973, 1.030]** | INCONCLUSIVE | 1.000 |
| **12 (judged)** | 8 | **0.994 [0.976, 1.012]** | INCONCLUSIVE | 1.000 |

**The lazy diagnostic,** explicit workers (it decides nothing):
- 1.002 at 8;
- **0.972 [0.953, 0.991] at 9, FASTER**;
- 1.005 at 10.

RSS is within 1.2% everywhere.

**The verdict under §84's frozen rule: NOT ADOPTED.** PR 879 does not merge. The verdict is final for this registration.

**Two findings:**
1. **The effect is too small for this machine to resolve at the knee.** The prediction was ≤ 1.5% of wall at 8 workers. Every point estimate at 8 and above is between 0.978 and 1.001, but the intervals are ±1.8–2.9%. The direction is plausible; the gain is not shown.
2. **The spin fixed the clock, not all the noise.**
   - The clock ratio is 1.0000 at every count, and at 6 workers the interval is ±0.5%, as §81 predicted.
   - But at 8 workers and above, the paired intervals stay ±2–3%. That is a paired SD of about 4–5%, roughly where 2b's pinned scanner left it (§78; the Pathfinder: clock R² only 0.37 at 8).
   - So at the knee a second noise source dominates once the clock is controlled: most likely placement, the scanner sharing its core part of the time (§78, record 132).
   - **Consequence for design:** tests of small scanner effects at 8 or more workers need either many more rounds (about 4× for ±1%) or the scanner pinned to a quiet core (2b's pin halved the SD).

**Decision:**
- **PR 879 is held, not rejected.** The micro-win stays a lead. A further test would be a new registration with the scanner pinned, or with 64 rounds; it is not run tonight.
- **§84 test 2** (the combined scanner fix, a larger expected effect) still waits for PR 861 to be reopened. Its registered design has the same noise problem at 10 and 12. Before it runs, consider re-registering it with more rounds or a pinned scanner. That is a design decision, recorded here before any of its data.

## 87. The ideal-parts matrix: the task, set up as its own subproject

Louis, 25 November, on reading the overnight summary: the point of the experiments was to characterise the system's shape, with each part as a toy problem with ideal behaviour, and then hand it to Antigravity to look for new mathematics. It was not to build engineering features.

**The overnight work drifted.** I kept the workstation fed with what was ready, which was fixes with PRs. It found two real gains (#868, #878, now released), but it did not run the ideal-part arms. Those are programme run 5, the joint frontier, and the missing T2, T3 and T6 ceilings. Stage 2a's oracle comparison was VOID, and stage 2b dropped it.

**The task, now stated where it can't be missed:** `ideal-parts/README.md`.
- **What it is:** a factorial over the decoder's parts, each real or ideal (its output supplied), at 1, 6, 8 and 10 workers. Slice A first: the two oracles that exist, the scanner's output and emission's codes, in one binary. Then the full 2^(6−1) design.
- **Where it comes from:** Jain's experimental design applied to the four-arm oracle idea (§44). It fills the measurements behind the hybrid-systems frame (an ideal part is lever 1 at its limit).
- **The rule:** frozen before any data (`ideal-parts/registration.md`).
- **Antigravity's first pass** (`ideal-parts/antigravity/prompt-1.md`) asks for mathematics per part, and for registered predictions of each part's room.

## 88. The ideal-parts matrix, slice A: emission has room below the knee, the scanner above it, and they co-limit at 8

`ideal-parts/run-20261125T105502Z` (evidence 04c9ecd9, checked; valid in every check). The full write-up is `ideal-parts/results/slice-a/README.md`.
- **1 and 6 workers:** emission ideal gives 0.827 and 0.849. The scanner ideal gives nothing.
- **8 workers:** neither alone resolves (both about 0.96). Each gives about 0.86 with the other ideal, and the interaction q_SE = −0.028 [−0.038, −0.018]. That is masking at the balance point.
- **10 workers:** the scanner ideal gives 0.919.
- **The floor:** both ideal, the wall is still 0.82–0.84 of today's. Most of the room is elsewhere: construction, MTF, RLE1/CRC and delivery, which the full matrix measures with Antigravity's oracles (PR 895).
- **Antigravity's pre-registered predictions:** 8 of 9 values hit; its two MASKED labels missed.

## 89. The full ideal-parts matrix: construction has the room below the knee, the scanner above it, and they co-limit at 6–8

`ideal-matrix/run-20261125T125334Z` (evidence 7dd7b070, checked; valid in every check; 2,492 decodes). The full write-up is `ideal-parts/results/full/README.md`.
- **The room, each part alone ideal:**
  - construction 0.58 at 1 worker and 0.75 at 6;
  - emission 0.83–0.85 below the knee;
  - MTF/RLE2 0.92;
  - the scanner 0.92 at 10.
- **The dominant interaction is scanner × construction** (q_SC −0.110 at 6, −0.071 at 8). At 8 workers, construction ideal saves 6% alone but 34% with the scanner ideal too, and the reverse holds. The two have to be attacked together.
- **The floor (all ideal) is mostly the oracles' own handover cost.** At 1 worker the handovers sum to the all-ideal wall, and from 8 to 10 the all-ideal wall rises. The decoder's irreducible residual is therefore unmeasured; that needs zero-copy oracles.
- **Antigravity's predictions:** 14 of 28 values hit. They named S × C as the largest interaction, correctly. The floor predictions missed, mostly because of the handover artifact.
- **The targets for the mathematics, by measured room:** construction (T2), then the scan (T4), then emission (T1), with S and C together at the operating points that matter.

## 90. Antigravity's second pass, and the next measurement: the decoder's own residual

`research/neon-movement/IDEAL-PARTS-2.md` at 7fab624e (#145, REVIEW 7179225952), against the full matrix (§89).
- **The model, revised to three stations.** Memory is the third (the shared L3 and DRAM bandwidth), beside the scanner and the workers.
  - Active workers tax the pinned scanner by about 6% through contention.
  - Once the parts are made fast, memory binds, which accounts for the all-ideal wall rising from 8 to 10 workers.
  - 8 workers, or 8–9 pooled, is the operating target.
- **The floor, decomposed by arithmetic** (not measured):
  - emission's oracle leaves the word spelling (`consume`, about 259 ms at 1 worker) in the clock;
  - pure plumbing is about 14–19 ms;
  - simultaneous copies add 39–69 ms at 8–10 workers.
- **The targets:** C, then S, attacked together; then E; then D. Enrichment across the scanner's boundary is obstructed (its Theorems T5.1 and T5.2), so dissolving the boundary by a pooled scan is proposed instead. There are concrete designs with kill tests for C, S and E+A6c.
- **The next characterisation step:** zero-copy oracles (its §3) and a short floor run.
  - **The build goes to Antigravity** (`ideal-parts/antigravity/prompt-build-zero-copy.md`). It designed the change.
  - **The recipe goes to the Pathfinder,** on #909's faster harness.
  - Characterisation only. The designs' kill tests wait until the floor is measured.

## 91. ChatGPT's review of the ideal-parts work, and the corrections it made

Recorded verbatim at Louis's request: `2026-11-24-algorithm-level/ideal-parts/reviews/2026-11-26-chatgpt.md`.
- **Its verdict:** the matrix is the experiment needed to direct the algorithmic work, and it is enough to start constructive design.
  - The remaining measurements, the zero-copy floor (#917) and A6b's time split (the Rauzy census), each have one decision to make, and then they close.
  - The S + C result is the headline: 195 → 125 ms at 8 workers, 1.56× throughput, against 189 and 184 ms for either alone.
- **Corrections:** the full list is in that folder's `reviews/README.md`. The ones that change readings:
  - **E's oracle left the expansion of codes into bytes in place** (about 259 of its 285 ms "handover" at 1 worker). So E's measured room is not emission's ceiling, and the all-ideal floor still contains most of emission.
  - **Memory bandwidth as a third bottleneck (§90) is a hypothesis.** The matrix can't separate it from the oracles' replay traffic, and the zero-copy run tests it.
  - **MTF/RLE2 has real room:** 7.5–8.4% below the knee.
- **The plan changes in one way:** threads 3–5 (pooled scan, C's A5+A6a fold and A6c, forward emission) may start design work now, alongside the two closing measurements (`plan.md` §1).

## 92. The zero-copy floor: plumbing is negligible, the scan alone is 179 ms at 8 workers, and two hidden costs

`ideal-parts/results/floor/` (evidence cc8ba137, `zero-copy-floor/run-20261126T112751Z`, 648 decodes, all valid).
- **The plumbing between the parts is about 5 ms at every count** (Residual B), 2.7% of the shipped wall. The prediction of 14–19 ms missed: the plumbing is smaller than predicted.
- **With every worker-side part free, the scan alone takes 179 ms at 8 workers,** against the shipped 195. Worker-side work can buy at most 8% there until the scan leaves its serial station. The pooled scan is the first build.
- **Hidden costs the matrix couldn't see:**
  - emission's spelling of words into bytes, 298 ms at 1 worker (22% of the wall);
  - delivery's serial copy-out, about 48 ms at every count, which becomes the next serial floor.
- **Memory doesn't bind at 8–10 with the oracles' copies gone:** Residual A is flat at 40–43 ms. The third-station hypothesis (§90) isn't supported.
- **Characterisation is closed.** The pooled-scan prototype is unblocked, and its §5 readings are set: 8 and 9 judged, the capture not uncapped.

## 93. The pooled scan's kill test: KILL, narrowly; pooling moves the scan's work, it doesn't remove it

`pooled-scan/results/` (evidence c71c4a56, 480 decodes, valid, every pooled decode wholly pooled).
- **K1 fires at 8 and 9:** the capture of the scan's room behind construction is 0.52 and 0.48 (point estimates). The upper bounds miss the √(SC/C) bar by 0.009 and 0.042. K2, K3 and K4 pass; the finder takes 4.8 ms.
- **The mechanism:** the workers' summed scanning equals the serial scanner's. Pooling shifts about 175 ms of CPU onto workers that bind below the knee.
  - So P alone is 10% slower at 6–7 and 5% at 8, and 6.4% faster at 10.
  - With construction ideal it takes 158 ms at 8, against C's 183 and SC's 135.
- **Stopped as built** (C2). The data points to a hybrid (idle workers scan ahead from slack only), and to a registered re-run once threads 4–5 make the workers cheaper.
