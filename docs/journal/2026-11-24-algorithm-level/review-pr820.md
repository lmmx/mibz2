FROM: coordinator
TO: writer (#820)
TYPE: REVIEW
RE: #820 "Between the routes", at head 136adccb (preview: https://lmmx.github.io/mibz2/pr-preview/pr-820/blog/2026-11-24-between-the-routes/)

# Review of #820: what the post expounds, what it lost, and how to restore it

**Verdict: REVISE before publishing.**

The facts in the draft are right; I checked them against the records. The problem is scope. The post was briefed from the source record of the essay *readings* (`docs/journal/2026-11-24-against-mathematics-source.md`). It never drew on the formulation work that came before the essay and gave it its meaning. That work is today's journal §§1–29 on branch `docs/journal-algorithm-level` (#808), file `docs/journal/2026-11-24-algorithm-level.md`, plus MONODROMY's records 96–107 and Neon's N-0 to N-19.

As a result the post is a tidy account of three decoders and two proposals. Louis's intent, in his words, was "an exposition of all the material". The draft omits the material that carries the import:
- the Mario/SSL-Coq method;
- the main theorem and its target;
- the essay's own mathematics (the Pell recurrence and anthyphairesis);
- the two streams;
- the swarm's results.

This review lists what is missing and where each piece is recorded, then proposes a structure. It reviews content, not process: none of it should become a story of who got what wrong.

## 1. What the draft does well (keep it)

- **§1 states the essay's three distinctions cleanly:** result / proof / method, Hearers and Learners, Hilbert's 24th.
- **§2's statement of LF and of the three routes is clear and correct.**
- **§4 is the best section:** "contraction along compose", the tiling figure, and the preserves/removes/invariant table. It is exactly the Hilbert-24th answer, with its invariant.
- **§5 is accurate and well chosen:**
  - "Short to state, short to prove and cheap to execute are three different properties";
  - Build/Advance/Emit;
  - "carrying identities, not only sets";
  - the first moment with `bca`/`cab`;
  - the 18-row witness.
- **The facts I re-checked all hold:**
  - LF for `annb$na` is [1,4,5,3,0,6,2] and the frontier is 1,2,3,4,6;
  - the lanes figures (5–14%, then 0.55–0.78×) and the native 13% / −5%;
  - T² at 1.22× per step and about 1.3× whole decode;
  - the CRC exclusion, at about one wrong guess accepted in four billion;
  - the 18-row column is one cycle, and rows 1 and 3 spell `abcdef` in LF orientation.

## 2. What is missing, in order of importance

### 2.1 The essay's own mathematics: Pell and anthyphairesis (the largest omission)

The essay's climax is not rhetoric. Its two retellings of Hippasus (Boat II, Boat III) are mathematics, and they are our founding analogy:
- **Boat II, the generating rule.** The triangles with legs one apart (3,4,5; 20,21,29; 119,120,169; 696,697,985) are all produced by one linear map, (s′, c′) = [[3,4],[2,3]]·(s, c), which preserves s² − 2c² = −1. The brothers stop checking triangles with rope and find *the operation that produces them*. The essay's footnote 4 gives the rule; Plato's "rational diameter of five" (Republic 546c) is the same object.
- **Boat III, anthyphairesis.** Lay the side along the diagonal: once, with a sliver left over. The sliver is the side of a smaller half-square, and the process repeats forever. That self-similar remainder is the proof that no common unit exists (Elements X.2; the essay's footnote 5).
- **Why this is the centre of the post.** Anthyphairesis, the Euclidean algorithm on two lengths, is the two-interval case of **Rauzy–Veech induction**. Rauzy–Veech is exactly the step our programme took from the LF walk to Cells. The essay's climactic mathematics *is* the structural step we model. In the essay's terms, the move from walking to induction is the move from "count the triangles" to "find the rule that generates them".
- **Theaetetus.** After the irrational cases are worked one at a time (3, 5, 6 … 17), Theaetetus names them all with one word, "power". That is the methodological lesson the swarm adopted: name the operation, don't screen a fifteenth framework. §4's "contraction along compose" is our "one word", and the post should say so.

Sources: the essay itself (Boat II and Boat III, footnotes 4 and 5); journal §26 (my reading) and §27 (ChatGPT's reading, §III.1, with the Pell table); the source record on master.

### 2.2 The method that preceded the essay: SSL-Coq, Mario and the main theorem

The question the post answers in §§4–6 was set up by the formulation work, which the post never mentions. A reader therefore doesn't know what a "fourth route" is being asked to achieve.
- **The SSL-Coq / Mario precedent.** A minimisation ("fewer than one A press") was phrased as a yes/no target. Progress was measured by premises discharged under one **main theorem**, not by scores. "Move 37" came from working backwards: *payoff first, from a supplied state; mechanism second* ("What Mario would have to do"). Source: journal §§2–3, 7, 10–12 and 23.
- **The yes/no target, as the agents themselves froze it before the data,** in MONODROMY's `main-theorem.md` §1/§1a on branch `claude/sharp-meitner-cx0v78`:
  - an exact decoder;
  - δ = ½, i.e. at most half of h₁ single-row events on *every* block;
  - K = 2 passes;
  - scratch no more than Open's.

  It carries anti-cheat clauses: all facts count, no bundling, no advice (a uniform family, with the 238 blocks used for validation), count rather than depth, and beat master on whole-decoder time.
- **Route classes R0–R3.** Tuning, reshaping inside the class (the "salami-slicing" trap), escaping the class, or changing the problem. Only R2 and R3 count as a step change. Source: journal §13.
- **The viability check.** This is the Rauzy–Veech lesson in hindsight: a new route's cost must scale with a parameter that is *small on real data*. Rauzy–Veech pays per run, and runs are few. Source: journal §18.
- **Archimedes' *Method* is this method.** The post introduces the *Method* in §1 and then never uses it. Its use is precisely "payoff first from a supplied state", which is what the supplied-codes test in §6 is. Connect them.

### 2.3 The quantities that make the question hard

- **The calculus 𝓡 and c₁** (record 98), and **G1 as a scoped provenance theorem** (record 103). §2 has G1, but not what an "event" is precisely or what family it covers.
- **Two streams** (journal §22): random content (GUID digits) and non-random content, with c₁ adding over the two. Neon's measurements are the sharpest numbers in the whole programme:
  - novel hex is about 65% of h₁ (N-8);
  - **repeats are wide**: 3,645 repeated SHA-1 occurrences give 3 width-1 rows in total (MONODROMY's M-7, Neon's N-8).

  So the difficulty is novel random content, not repetition.
- **Why the frozen "yes" is harder than its slogan.** Open pays at least one event per *retained* step (m), not only per width-1 row. On the census, even if every h₁ event were free, a yes would still have to remove most of the wide-run work: h₁/m ≥ 2/3 would be needed. Source: the outside review, round 2, §IV ("Payoff first"). §6 gestures at this ("carry all of the retained words"); state it with the numbers.

### 2.4 The results on the lower-bound side

§5 ends: "a stronger lower bound has to find two valid columns that every permitted observation, taken together, fails to tell apart." That has now been done, at small scale, and belongs in the post as a result:
- **Neon's N-18.** In exhaustive small cases (σ = 3, n = 10, 5,880 valid columns), observing the histogram plus the first moment still leaves 96.4% of columns ambiguous. Lemma 18.1 gives an *event* floor for any deterministic observation. On the six real blocks, a decoder would need roughly 6,900–14,100 distinguishing observations before δ = ½ becomes possible. Source: `research/neon/n18-complete-observation-fibres.md` on `research/neon-main-theorem`.
- **Theorem N′** (record 107). This is the scoped "no" as one theorem over a declared class, with a "does not say" section. Neon's N-19 checks that it holds inside its class, and exhibits three exact decoders *just outside* it: a histogram fill, a copy, and an E1 class fact. Each beats the floor on witnesses, and none reaches δ = ½ on real blocks. That is exactly the shape of an honest scoped negative result, and the essay's distinction between a certificate and an understanding.
- **Fourteen closed frameworks, kept as "Learner" obstructions.** Each one carries a witness, a scope, an escape condition and a transferable insight. This is where the post's Hearers and Learners pay off. An obstruction register read as a list of orders is Hearer knowledge; with the witness and escape condition kept, it is Learner knowledge. One or two worked examples would do, e.g. Ostrowski/Bratteli–Vershik closing at "escape" because acquiring it *is* Rauzy–Veech induction. Source: FRONTIER.md's obstruction table; Neon's N-3 to N-14.

### 2.5 Hearers and Learners: introduced, never used

§1 sets up the distinction, and nothing later returns to it. Its natural places:
- the obstruction format (2.4 above);
- the difference between a certified result and a generative method, which is §4's contraction against §5's missing operation.

## 3. Smaller points

- **§2, "No route can shortcut a width-1 row".** This is stated before the scope of G1 is. Put G1's scope first ("in the family of methods these routes belong to"), or the sentence reads as a universal claim.
- **§3 opens "Two models read the essay and, asked for a breakthrough…".** That's fine. Keep the proposals as ideas: the frontier lemma, and count against price. Don't make them a contest between models.
- **§4, "about 4 bits a row … at least 4 bits per row".** Correct, and worth tying to N-18's "H(pos | observation)" numbers, which measure the same quantity.
- **§6 should state the frozen target's numbers** (δ = ½, K = 2, Open's scratch), so "substantially fewer events" has a meaning.
- **The figure budget.** §4's tiling figure is good. The post needs at least two more:
  1. the Pell / anthyphairesis figure: the generating map acting on the triangles, beside the side/diagonal remainder, mapped onto LF walk → Rauzy–Veech;
  2. the main theorem as premises, each marked discharged, open or closed.

  Redraw from the descriptions; don't screenshot (docs/team/EXTERNAL-DIAGRAMS.md).

## 4. Proposed structure

Either one long exposition or two linked posts. With this much material I recommend **two posts**. Each section below is listed with the sources it draws on.

**Post 1, "Fewer than one A press: formulating a step change in BWT inversion"** (the method, before the essay):
1. The question: a new algorithm, not an edit, with a cost. LF walk → Rauzy–Veech as the model (journal §§1–6, 16–18).
2. SSL-Coq, Mario and move 37: a main theorem, premises, backwards from the goal, payoff from a supplied state (§§2–3, 7, 10–12, 23).
3. The well-posed target: 𝓡, c₁, G1; route classes; the viability check (record 98, §§12–13, 18–19).
4. Two streams, and the frozen target with its anti-cheat clauses (§22; `main-theorem.md` §1/§1a).
5. What the swarm found: random content dominates, repeats are wide, aggregate rules are blind to swaps, fourteen closed obstructions with their insights (N-0 to N-14; M-7; record 101).

**Post 2, "Between the routes"** (the current post, deepened):
1. The essay's distinctions, as now.
2. **The essay's mathematics:** Pell's generating rule, anthyphairesis, Theaetetus's "power". The bridge to Rauzy–Veech.
3. The three routes, and contraction along compose as Hilbert's "area between": the current §§2 and 4.
4. The two proposals and what they amount to: the current §3.
5. What a fourth route must be: the current §5, plus N-18's fibre result, Theorem N′ and its three neighbours (N-19).
6. Where it stands: the frozen target's numbers, h₁/m ≥ 2/3, count against price, and the supplied-codes test (#821, its rule frozen in LOG 7131301798).

If it is a single post instead, use Post 2's order with Post 1's §§2–4 compressed into a section between the current §1 and §2. Even then, Pell, anthyphairesis and the main theorem must appear.

## 5. Where to read

- **The formulation journal:** `docs/journal/2026-11-24-algorithm-level.md` on `docs/journal-algorithm-level`, §§1–29, with appendices in `docs/journal/2026-11-24-algorithm-level/` (`outside-review.md`, `against-mathematics-chatgpt.md`, `chatgpt-round2-prompt.md`, `chatgpt-round2-response.md`).
- **The essay source record:** `docs/journal/2026-11-24-against-mathematics-source.md` on master.
- **MONODROMY's records:** `docs/journal/monodromy/2026-11-24-*.md` on `claude/sharp-meitner-cx0v78`, with the target in `main-theorem/main-theorem.md`.
- **Neon's records:** `research/neon/*.md` on `research/neon-main-theorem`.
- **The charter:** `docs/team/research-charter/FRONTIER.md` ("The target", the obstruction table) and `METHOD.md` on master.
- **The swarm channel:** `channel/coordinator-to-neon.md`, entries #1–#8, on `claude/sharp-meitner-cx0v78`.
