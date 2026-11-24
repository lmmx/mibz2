# The council brief: formalise the problem, don't solve it

*24 November 2026. Commissioned by Louis Maddox for lmmx/mibz2, a Rust bzip2 decoder. Three members answer this brief independently: Gemini 4 Neon in Antigravity, MONODROMY (Claude), and ChatGPT Pro. The Coordinator compares their answers afterwards.*

## Your task

Read the source material below. Then write **a mathematically formal statement of the problem the mibz2 research programme is trying to solve**, faithful to its sources.

**Do not attempt to solve it.** Don't propose a new algorithm, object or construction. Your job is to make the *question* exact, and to check that it is the question the sources actually pose.

**Work independently.** Don't read the other council members' answers before writing yours. They are in `docs/journal/2026-11-24-algorithm-level/council/` on branch `docs/journal-algorithm-level`; leave them unread until yours is done.

## The intent, in Louis's words

From the formulation record (journal §§1–32):
- "the Mario thing … the Doom slide piece … was all meant to reframe the entire effort towards asking a question that was sufficiently powerful to unlock something that would give us an entirely new basis for the BWT inversion rather than an incremental gain."
- "work backwards from the conditions for what it would take to be a non-incremental movement, entirely reframe the question in terms of new mathematics."
- "lower the floor, more accurately than raise the ceiling."
- "it's no longer a matter of asking the models to formulate the problem, but rather to begin taking a pop at solving it in this kind of proof formalized way." That is the *next* step, after this council has firmed up the formulation.

The model of the step sought is **LF walk → Rauzy–Veech induction**. That step changed the decoder's object from points (rows) to intervals (runs). It changed the cost law from per row to per run, and runs are few on real data. And it was built from the input without the old mechanism.

## Read (all public; pinned commits)

Base URLs:
- master: `https://github.com/lmmx/mibz2/blob/c282b422b4a1ae6ffde4c153ffe65a01a0088013/`
- formulation journal: `https://github.com/lmmx/mibz2/blob/2a99cd517ffdb370bedf8583b6130b3c3df2fe75/`
- MONODROMY's branch: `https://github.com/lmmx/mibz2/blob/f86a4d3b3b5d819c010c6eb27a4856acbd87a90a/`
- Neon's branch: `https://github.com/lmmx/mibz2/blob/cc3b76f32adb1b50d07a520c67b2760b4c73b884/`

1. **The two source pieces.**
   - *doomslide, "Against Mathematics"* (24 October 2026): https://doomslide.substack.com/p/against-mathematics. Read especially the Hearers and Learners, Archimedes' *Method*, Hilbert's 24th problem, and Boats I–III: the oracle, the Pell rule that generates the triangles, and anthyphairesis.
   - *The Mario / SSL-Coq proof:* https://github.com/tra38/sm64-wmotr-abc-proof. It proves, in Rocq, an attempt to show that a level of Super Mario 64 cannot be finished with zero A presses. What the programme took from it: a minimisation phrased as a yes/no target; a main theorem whose premises are discharged one by one; "forbid the obvious mechanism"; and payoff first from a supplied state, mechanism second.
2. **The formulation record, with Louis's words verbatim.** Journal base, `docs/journal/2026-11-24-algorithm-level.md`, §§1–32. Its appendices are in `docs/journal/2026-11-24-algorithm-level/`:
   - `outside-review.md` and `chatgpt-round2-response.md`: two adversarial reviews;
   - `against-mathematics-chatgpt.md`: a reading of the essay.

   On master, `docs/journal/2026-11-24-against-mathematics-source.md` has both readings of the essay.
3. **The target as frozen, and the charter.**
   - MONODROMY's branch: `docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md`, §1 and §1a, with their anti-cheat clauses.
   - master: `docs/team/research-charter/FRONTIER.md` ("The target", the obstruction table) and `docs/team/research-charter/METHOD.md`.
4. **The mathematics so far.** MONODROMY's records in `docs/journal/monodromy/2026-11-24-*.md`. The ones that carry weight:
   - 96 (the GUID wall);
   - 98 (the calculus 𝓡 and c₁);
   - 103 (G1 as a scoped provenance theorem);
   - 104 (the ordered frontier; contraction along compose);
   - 105 (abbreviation, batching, schematic facts);
   - 107 (Theorem N′);
   - **108, `2026-11-24-charge-convention.md`.** It shows that the frozen binary target's answer is set by conventions (whether a bulk pass is charged, and the pass budget K), not by the column.
   - 109 (no supported-row filter).
5. **The measurements.** Neon's branch, `research/neon/n*.md`, items N-0 to N-19. Key numbers:
   - novel hex is about 65% of h₁;
   - repeats are wide;
   - supplied copy plans need 0.54–0.64 instructions per width-1 row;
   - the fibre results (N-18).
6. **The published account:** https://lmmx.github.io/mibz2/blog/2026-11-24-halving-the-single-row-steps/.
7. **The critique rubric, R1–R7,** fixed before any construction is judged: journal §32.

## What to produce

One document. Mark every claim **proved / argued / conjectured / taken from source**, and cite the source for each element.

1. **Definitions.**
   - The input: a valid bzip2 BWT block (L, origPtr), single cycle, the actual size and alphabet ranges.
   - The output, and the correctness condition.
   - The **model of computation and the cost model**. Resolve record 108's dichotomy: what is charged, what a pass is, what memory is.
   - What an "instruction set" is, so that a charge is meaningful.
2. **What "a new basis" means, formally.** Turn the three conditions into precise mathematical conditions: a new object, a cost law in a new parameter, and acquisition without the old mechanism. Show that walk → Rauzy–Veech satisfies them, as a sanity check that the definition captures the model step.
3. **What "lowering the floor" means.** Say which floor (G1's, in which class) and in which currency. Say why it is not "relocating cost into an uncharged pass". Say how it relates to whole-decoder time (frozen clause 3).
4. **The main theorem's shape,** Mario-style: the statement a successful programme would prove, its premises, and what a "no" would have to quantify over to mean anything.
5. **A check against the sources.** Where does this formalisation match the doomslide/Mario/record material, and where does it diverge or add something? Flag every place where the records contradict each other.
6. **A critique of the current formulation.** Which parts of the frozen target, the conditions in channel entries #13–#14, and the rubric R1–R7 are ill-posed, degenerate (cf. record 108) or missing? For each, give the corrected statement.
7. **Open choices,** where the sources don't decide. For each, state the options and give your recommendation and reason.

Length: as long as rigour needs, with no padding and no recap.
