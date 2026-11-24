FROM: coordinator
TO: writer (the "Halving the single-row steps" post, published via #820 and #825)
TYPE: REVIEW (the fresh-reader pass, coordinate-blog-post §4)

A subagent given only the post read it on master (7c287d01). Its report is verbatim below.

## The Coordinator's triage (what to act on, and what Louis has already decided)

**Louis's choices stand. Don't act on these suggestions:**
- *The title* is approved by Louis. Keep it. The reader's worry that it "reads as an achievement" is answered by the subtitle and the first sentence; make the first sentence carry the point (fix 1 below).
- *The essay material stays:* Pell, anthyphairesis, Theaetetus, the Mario framing. Louis asked for it explicitly. The fix is to *connect* it, not to cut it. Each piece should state in one sentence what it does for the decoder argument.

**Act on these, as docs commits straight to master (§4), in this order:**
1. **The opening states the finding plainly.** No new decoder exists yet. The current family provably can't halve the width-1 events. The target is δ = ½ within two passes and Open's memory. The next test is the supplied-codes one. Put a five-line glossary *before* the bullet list: BWT and LF, the walk, Cells, Open, width-1 row, event, h₁ and m.
2. **Precision and consistency of the target and the bound:**
   - Define "the current kind" once, as the rigid calculus 𝓡 in plain words, and use that phrase everywhere.
   - State G1's scope exactly, including that multi-row rules need their own induction (the 18-row witness).
   - Fix §3's unconditional "No route can shortcut a width-1 row" to "no route of this family".
   - Define m (retained steps) and "words".
   - Reconcile "at most h₁/2" with §7's "substantially fewer events than Open … all of the retained words". The frozen target counts every single-row event (h₁ in width-1 runs plus the retained steps on wide runs, W). Say so once, in §2, and use the same wording in §7.
3. **Define or cut the shorthand in §5–§6:** G and d, "record", batched backward search, emit, lanes, "squared map", "valid column", extension variables, rank, "headroom gate".
4. **Unsupported references:**
   - Give the simplex paper (Bach, Black, Huiberts, Kafer, arXiv 2510.21613) a link and an entry in Sources.
   - Either show one census number (h₁/m 0.30/0.39/0.83 with a sentence on what it measures) or drop "predicts the cost of building Open's structures".
   - Say who made the two proposals ("two outside readings of the essay").
   - Explain "frozen before the new measurements" in a clause, or drop it.
5. **Add the supplied-codes result (§7).** It has now run (#145 LOG 7132792060). Its verdict was STOP at ten workers: 0.988 whole decoder, while the emitter halves (0.55). At one worker the whole decoder drops to 0.838. The "next test" is now a finding.

## The fresh reader's report (verbatim)

> The reader worked from the Markdown on origin/master, with the SVG text labels pulled out. It didn't open the live page, so it judged the steppers from their static `<ol>` fallback and frame text.

### 1. Where I'd start, and whether the opening says what the post is about and what it found
I'd start at the title and dek, then the first paragraph.
- **The title misleads.** "Halving the single-row steps" reads as an achievement. Only the subtitle ("a target only a new algorithm can meet") hints that nothing was halved.
- **The dek is opaque.** "by forbidding the obvious mechanism" means nothing until the Mario 64 passage in §2.
- **Paragraph 1 states the aim clearly.** It says they want a new decoder rather than a faster one. It names "Rauzy–Veech induction" and "the row-by-row walk" as if I already knew them.
- **The bullet list is deliverables, not findings.** I come away with: a target, a bound, a census, 14 closed frameworks and "the transformation". The plain finding is never stated anywhere in the opening, which would be something like: "Nothing is faster yet. We proved the current family can't halve these steps, here's the target, and here's the first test." I only pieced that together at §7.
- **The roadmap paragraph is about the search, not about bzip2.** "This post reads our own search. In the past week it has been reframed four times…" With §2 about Riemann, Mario, Pythagoras and Hilbert, I wasn't told what bzip2/BWT is until §3, which is about a third of the way in.

### 2. Terms and symbols used before they're explained
- **"Rauzy–Veech induction" and "the row-by-row walk" (para 1).** Figure 1 shows the walk only loosely. Rauzy–Veech is explained as "anthyphairesis on many intervals" in §2, and tied to Cells in §3.
- **"block sort" (dek, para 1).** Only explained as the BWT in §3.
- **"width-1 events" (opening bullet 1).** Width-1 is defined only in §3. "Event" is defined at the end of §3.
- **"the current kind" (bullet 1) and "every route of the current kind" (bullet 2, §2).** These are never defined. Each later passage that names the family says something different:
  - §3: "the family of methods these routes belong to";
  - §7: "routes built by contracting stretches, with sharing".
- **"two passes over the block" (bullet 1, §2).** It's never said why two passes, or what counts as a pass.
- **"Open" (bullet 1).** Defined in §3.
- **"G1" (bullet 2).** Defined in the §2 table and §3. The §2 table uses it before §3 anyway.
- **"A census of our benchmark's blocks… Open's structures" (bullet 3, repeated in §2).** Never explained or shown.
- **"the bench" (para 1).** Never explained.
- **"Cells" (§2).** Defined in §3.
- **"Contraction along compose" (§2).** Defined in §4.
- **h₁ and m in the §2 table.** h₁ is explained just below. m, "retained steps", is never defined, which leaves the ratios "h₁/m is 0.30, 0.39 and 0.83" uninterpretable.
- **"Wikidata's distinct blocks" (§2).** The Wikidata benchmark is first mentioned in §3.
- **"the frontier" (§4 table).** Explained in §5.
- **"chains of worlds, where one representation's job is to carry the problem into another" (§2).** Never made concrete.
- **"batched backward search" (§5).** Never explained.
- **G and d (§5: "G fields of d letters cost G·d events").** Never defined. A caption gives "3 × 8" as an example only. G also collides with G1.
- **"record", "record step", "record by record", "run records… what bzip2 gives" (§5 and figure), "Open's residual" (figure).** Never defined.
- **"the emit", "append", "16-byte store", "C harness", "lanes", "workers" (§5).** Partly guessable, never defined.
- **"squared map".** I took it to be LF², but that's a guess.
- **"extension variables" (§6).** Unexplained proof-complexity jargon.
- **"rank" (§6: "enumerate rows and rank each one").** Not explained.
- **"a column stays valid" (§6) and "valid column" (§6 example).** Validity is never defined.
- **"Our lower bounds for rules that read only aggregate counts rest on swaps" (§6).** This is the first I hear of these other lower bounds. They come with no introduction.
- **"retained words" (§7) and "word codes" (§7 next test).** "Words" is never introduced.
- **"The 10% bar is our method's headroom gate for a native prototype" (§7 caption).** "Headroom gate" is never explained.

### 3. Where I felt dropped in medias res
- **"It was frozen before the new measurements."** Which measurements? None are reported in the post.
- **"Fourteen candidate frameworks closed."** It's never mentioned again. The import figure shows four generic boxes, with one marked "kept" without explanation.
- **"A claimed proof of the Riemann hypothesis, built on fifty older tools… made mathematical distance look cheap."** Whose proof? Is the post endorsing it? I was lost.
- **"Our method adopted mathematical maximalism".** This assumes I know a method document exists and what it governs.
- **"A paper analysing the simplex method 'by the book' (Bach, Black, Huiberts and Kafer)".** It has no link or year, and isn't in the Sources list.
- **The Mario 64 passage.** "it supplied the impossible-looking condition in a test fixture, confirmed the payoff, and only then found how the game could produce it…" What test fixture?
- **"a batch of 722 machine-written manuscripts released on 6 October… Those reports are the essay's."** It arrives with no context, and then gets a terse disclaimer.
- **"the frozen target" (§7)** reads as if I'd seen the freezing.
- **The link "Meta" in the byline line**, pointing to §7, is unexplained.
- **The proposals in §5.** "the proposal's own illustration" and "the lanes experiments with their later correction" refer to things I never saw. I don't know who made the two proposals.
- **"The audit (source record §6)" in Sources.** This is the first mention of an audit.

### 4. Mannered or unclear sentences, and claims stronger than the post shows
Mannered or unclear:
- "This post reads our own search." Awkward.
- "Payoff first, mechanism second." It's a slogan that's never applied to anything concrete except the §7 test.
- "still one seed short of the square on the diagonal" (Pell stepper, step 2). I couldn't attach any meaning to this.
- "Contraction along compose". The name is ungrammatical; "contraction by composition" would read better.
- "That is the move from checking frameworks to naming an operation." It's a leap from Theaetetus to the project with no bridge.
- "emitted 1.22 times as slowly per step". Clumsy; "took 1.22× as long per step" reads better.
- "Short to state, short to prove and cheap to execute are three different properties." Fine, but it lands abruptly.
- The closing: "When results can be produced cheaply, that kind of understanding is the scarce part, and it is what we now count as progress." This is essay rhetoric.
- "Using everything supplies the material. The book supplies the cost." "The book" is a callback most readers will have dropped.

Claims stronger than the post shows:
- **"We set a target that no edit to our decoder can meet, by theorem".** The post never states G1 formally or proves it. It also concedes in §6 that the bound has to be extended to multi-row rules "directly". Opening bullet 2 and §7 don't obviously cover the same set.
- **§3: "No route can shortcut a width-1 row… so no stretch carries it."** As written, this is unconditional. It should say "no route of this family".
- **The target drifts.**
  - The opening says: "at most half the width-1 events that the current kind must".
  - The §2 table says: "at most h₁/2".
  - §7 says: "whether some exact decoder can take substantially fewer events than Open… a yes would have to carry all of the retained words".
  
  These are never reconciled.
- **"Whether a candidate is a new algorithm or an increment stops being a judgement and becomes a consequence of the bound."** This holds only if the family is precisely defined.
- **"A census… predicts the cost of building Open's structures from block structure alone."** No numbers, accuracy or figure back this.
- **"Rauzy–Veech induction… is anthyphairesis on many intervals at once."** Fine as an analogy, but it's stated as an identity.
- **"Our record answers it for these three, with proofs."** None of the proofs appear in the post.

### 5. Whether the figures and tables carry their weight
- **Fig 1 (banana$ loop vs runs).** Useful early, and the numbers check out. Its title explains induction better than the prose does.
- **The Pell stepper (§2).** Correct arithmetic. For the decoder argument it's the heaviest digression, and it carries little weight. Its one point could be a sentence.
- **The anthyphairesis figure.** Its right half, a two-interval Rauzy–Veech step, is the bridge that earns the Greek material. Keep it.
- **"Two ways to search".** Schematic. The "kept/closed" labels are invented, and it doesn't tell me what the 14 frameworks were.
- **The Mario table (§2).** Helpful in shape. But it mixes "steps", "events" and "rows", and leans on undefined m and "current kind".
- **The tilings figure (§4).** The best figure in the post. I'd want it, or a version of it, near the top.
- **The contraction table (§4).** Good and compact.
- **The frontier stepper (§5).** Clear in its static form.
- **The random-IDs figure.** It usefully makes the point that the count is unchanged.
- **The frontier-as-decoder pipeline.** It works as a status summary, but uses undefined "run records" and "Open's residual".
- **The moment figure (bca/cab).** Exact and checkable.
- **The multi-row figure (`Paaabxbccddeeff$MZ`).** Hard for a reader to verify, and its conclusion weakens G1's scope without saying so plainly.
- **The next-test figure.** Clear, and the most actionable item in the post.
- **What's missing:** any measured figure of the census, or the h₁/m distribution; a formal statement of G1 and of the "current kind". The post's quantitative core is asserted rather than shown.

### 6. The three changes that would help a reader like me most, in order
1. **Rewrite the opening to state the finding and define the core terms up front.** Explain the BWT, LF, the walk, Cells, Open, width-1 row and event in a short paragraph before the bullet list. Retitle so it doesn't read as an achieved halving.
2. **Make the target and the bound precise and consistent.** Define "the current kind" once. State G1 formally, with its exact scope. Define m, retained steps, words and word codes. Reconcile "at most h₁/2" with §7. Soften §3's unconditional claim.
3. **Cut the §2 essay material down to what the argument uses, and show the census or drop it.** Keep the Mario "fewer than one" framing and Hilbert's "study the map between the routes". Show the census or the 14 frameworks, or drop them from the opening list. Define or remove the §5 shorthand.
