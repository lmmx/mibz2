# 5. Prompt: a full account of *A Thousand Plateaus*, then a decisive cross-reading against mibz2

*26 November 2026, at Louis's request. The first cross-reading (`04`) worked mostly from the one passage Louis showed it, the rhizome's principles 3 and 4. This asks for the whole book's relevant machinery first, then a cross-reading that commits to consequences. Run it in a fresh Antigravity session. The swarms study's own finding is that a fresh start beats a long session.*

---

In lmmx/mibz2, fetch origin/master. Reading and writing only: no code. Commit as Louis Maddox <louismmx@gmail.com>, with no AI attribution anywhere, trailers included. Never put a PR or issue number straight after fix, close or resolve. Work on a branch `docs/deleuze-account` and open a PR against master. Don't push to master.

**Read first:**
- `docs/journal/2026-11-26-rhizome/` (all files);
- the first cross-reading, `docs/journal/2026-11-26-rhizomatic-swarms-cross-reading.md`, if it has merged; otherwise its PR;
- `docs/site/blog/conversations/2026-11-22-thomas-murphy.html`;
- `docs/team/MODE.md` and `MODE.RATIONALE.md`, which already lean on Deleuze;
- `docs/journal/2026-11-24-algorithm-level/plan.md`.

## Stage 1: an account of the book (`docs/journal/2026-11-26-rhizome/deleuze-guattari-account.md`)

Build a working account of Deleuze and Guattari, *A Thousand Plateaus* (1980; trans. Massumi 1987), from published secondary sources.
- **Sources:** the Stanford and Internet Encyclopedias of Philosophy, Massumi's *A User's Guide to Capitalism and Schizophrenia*, Holland's *Reader's Guide*, Adkins's *A Critical Introduction and Guide*, DeLanda (*Intensive Science and Virtual Philosophy*, *A New Philosophy of Society*, and *Assemblage Theory*, which is explicitly about science and engineering), and reputable chapter summaries and lecture notes online.
- **Copyright:** the book is in copyright. **Paraphrase. Quote at most a line or two at a time, with the plateau and page,** and never reproduce passages.
- **Cite every claim to a source.** Mark where the sources disagree.

Cover every plateau, at the depth it deserves for a team of agents building software and mathematics. At least:
- **"Introduction: Rhizome":** all six principles (connection, heterogeneity, multiplicity, asignifying rupture, cartography and decalcomania), the map against the tracing, and n − 1.
- **"1914: One or Several Wolves?":** the pack against the mass; molecular against molar multiplicities.
- **"10,000 B.C.: The Geology of Morals":** strata, double articulation, content and expression, the abstract machine.
- **"November 20, 1923: Postulates of Linguistics":** order-words and the incorporeal transformation. A prompt or assignment is an order-word: say what follows.
- **"587 B.C.–A.D. 70: On Several Regimes of Signs":** the signifying and post-signifying regimes, the despotic centre, overcoding.
- **"November 28, 1947: How Do You Make Yourself a Body without Organs?":** the warnings against too-sudden destratification, and caution as method.
- **"Year Zero: Faciality".**
- **"1933: Micropolitics and Segmentarity":** rigid, supple and flight lines; molar against molecular; the dangers of each.
- **"1730: Becoming-Intense…".**
- **"1837: Of the Refrain":** territory, the refrain as an organising routine, consistency.
- **"1227: Treatise on Nomadology—The War Machine":** essential here.
  - The state apparatus against the war machine.
  - **Royal (theorematic) science against nomad (problematic, ambulant) science:** following the material's singularities against reproducing laws. The journeyman, metallurgy and the phylum.
  - The tool against the weapon.
  - How the state captures the war machine.
- **"7000 B.C.: Apparatus of Capture":** stock, rent, overcoding by measure.
- **"1440: The Smooth and the Striated":** the technological, maritime and mathematical models (Riemannian against Euclidean space, metric against non-metric multiplicities).
- **"Conclusion: Concrete Rules and Abstract Machines".**

For each plateau, give:
- its central concepts and their mechanics, in plain language;
- the dangers the authors themselves name, where they name them;
- the operational content that could bear on how a team works or how a computation is organised. Name it as such.

## Stage 2: the cross-reading (`docs/journal/2026-11-26-rhizome/deleuze-guattari-cross-reading.md`)

Cross-read the account against mibz2: its decoder work, its mathematics programme, and its team machinery. Be **decisive**.

- **For each concept, one of three verdicts:**
  - **CONSEQUENCE:** a concrete change, with the file or process it touches, and how we would know it worked;
  - **DIAGNOSIS ONLY:** it explains something observed in the repository, with evidence, but suggests no change;
  - **NO BEARING:** say why in a line.

  Analogy without an operational claim doesn't count as a consequence.
- **Map the parts of mibz2 onto the book:**
  - *Team machinery:* coordinator, prompts, the #145 channel and its routing, registrations, CI and its checks, the evidence branch.
  - *Methods:* the ideal-parts characterisation and the hybrid-system model.
  - *Mathematics:* the Rauzy–Veech induction and Open's construction; contraction and enrichment (`essence.md`).
- **Test the first cross-reading's redesign against the whole book, not just principles 3 and 4.** Does the `frontier/` claims ledger risk re-capture by an apparatus? Is it a striation of a smooth space? Does the BwO plateau's caution apply to retiring the coordinator? What would "nomad science" ask of the mathematics programme that "royal science" (the theorem-first search) doesn't?
- **Argue against your own mappings where they're weak.** Note where the secondary literature warns against applying D&G instrumentally, and what survives that warning.

End with a ranked list of at most seven consequences, each with its cost and its test.

**Deliver:**
- the two files;
- one PR;
- a short REVIEW on #145 linking the PR at its head.
