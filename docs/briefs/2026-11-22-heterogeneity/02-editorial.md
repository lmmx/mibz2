# Transmission 2: editorial analysis, after Louis's correction (A5)

Read `01-source-material.md` first. This file says what changes, and why. It adds to the material;
it doesn't replace it. Where this file and the material disagree, Louis's words (A1–A5) win.

## 1. What the first outline got wrong

**It made the post about formal verification, with mibz2 as an example.** The post is about real
data and the decoder it forces. Lean is a foil in one section, not the protagonist.

**It described the result (a family of conditional routes) but not why it has that form.** The why is
the thesis. Real data has no single structural property that lets a decoder in. A Wikidata dump is
repetitive in some places. In others, identifiers defeat every contraction. Some blocks are dense,
some thin, and some sit near a worst case. A decoder that is fast across all of it has to be built
from exact constructions that each exploit a structure present in some of the data. The
heterogeneity of the data is mirrored in the heterogeneity of the decoder's mathematics.

**It flattened the difference between three things.** It said "routes" and let the reader hear
"dispatch". The three things are:
- a universal method with implementation tweaks (SIMD against scalar);
- special cases bolted onto a uniform decoder;
- a construction whose mathematical objects change with the structure of the data. The Open kernel
  doesn't skip rows faster; it stops working in rows where the block admits strips.

The post is about the third, and it must make the reader feel the difference.

**It declared saturation** from an oracle that covers one parameter family, (L, μ). It must not. The
honest position, and the more interesting one, is that we don't know how complete the family of
constructions is. That uncertainty is the research frontier.

**Its titles were slop-shaped:** "X, many Y: Z in a W that V". Avoid the colon-formula and
the noun-pair template.

## 2. The thesis, sharpened

> For real data, correctness is uniform and cost is not.

- **One specification, R, says what any decoder must output, for every block.** One proof route
  suffices to show a decoder correct. This is why lean-zip, and the RH proof, can take a single
  clean route: their objects are uniform with respect to what the proof needs.
- **The cost of decoding has no uniform structure on real data.** What makes one block cheap (long
  runs, strips that are pure translations) is absent from the next (GUIDs).
- **So the decoder we need is not one route proved correct.** It is many exact constructions, each
  valid under a hypothesis that is a structural property of the data: Pᵢ(x) ⇒ Fᵢ(x) = R(x). We
  burrow in at many places at once, on purpose.
- **The program is the language the proof is written in.** So the proof has to vary the way the data
  does. Its case analysis isn't chosen for elegance; the corpus dictates it. This is Louis's
  sentence in A5: "our language being the program which we set out to prove in language and
  therefore the proof must stutter in the way it shows". That sentence is the claim to develop, not
  decoration.
- **This is a deliberate strategy, not a consolation.** Global changes of representation are rare.
  There has been one: the Open kernel's induced (first-return) maps replaced the LF walk's per-row
  steps, and it beat the old decoder at every thread count. Below that, we pursue local optima that
  are conditional on the data's structure. Louis's word is "unreasonable": deliberately extreme
  specialisation where a regime is common enough to pay for it.
- **And it stays open.** A family of local constructions may later expose a common invariant: local
  to global, the way first-return maps generalised the strip contractions. We don't claim that will
  happen. We claim the strategy keeps it possible, where waiting for a universal leap does not.

Keep the plain first-person plural ("we burrow in at many places at once"), and keep the
intensity. Precise is not the same as cautious.

## 3. Title

It must come from heterogeneity, and it must be a claim, not a template. My order of preference:
1. **"Correctness is uniform, cost is not"**: the sharpest true claim in the post. Don't add a
   colon subtitle unless the dek fails to carry the scope.
2. **"Real data has no single way in"**: the reason, in plain words.
3. **"A decoder that stutters with its data"**: only if the Deleuze section earns it in the body.
   On a title alone it reads as drama, so it's the riskiest.

Your call, within the write-blog-post title rules. I'll show Louis the chosen title once it's live.
The URL slug should follow the new title. Version 1's slug is
`2026-11-22-one-theorem-many-routes`; rename the file, and keep a redirect or alias if zensical
supports one, since v1 is live only briefly.

The dek should state the thesis and the scope (one decoder, one workstation, Wikidata as the main
corpus) and carry a little joy (check_prose).

## 4. Architecture

**Opening.** Lead with the finding, not the background. Real data offers no universal way in, so
mibz2 is fast through exact constructions conditional on the structure of each block. The decoder's
mathematics is as heterogeneous as the data. Put the takeaways in the opening or §1:
1. Correctness is uniform and cost is not. A correctness proof needs one route; speed on real data
   needs many.
2. Each fast route is exact under a hypothesis that is a structural property of the data. The
   hypothesis is what we search for.
3. When the data refuses a hypothesis, there are three responses: weaken it; certify it more cheaply;
   or change the representation so it's no longer needed. The refusals are where the next
   construction comes from.
4. Global changes of representation are rare (one so far). We pursue conditional local constructions
   deliberately, and they may reveal a general one. We don't know how complete the family is.
5. The scope, and what isn't claimed: no Lean proofs yet; no claim that Deleuze is evidence; i9
   measurements only.

**§1. Correctness is uniform, cost is not.**
- **lean-zip as the clean case.** One theorem, a ratchet for optimising agents, still about 2.5×
  behind libdeflate on decode (quote its README).
- **mibz2's two correct changes with opposite fates.** R5 against the block reader (the bundles in
  E): CPU saved without wall time, and wall time won on the serial stage.
- **The point.** Correctness certifies admissibility. Cost lives in the data and the machine.
- Keep this section short. It is the foil.

**§2. What heterogeneity looks like in a Wikidata dump.** This is new, and the post needs it. Make
the heterogeneity visible with measurements, not adjectives. Candidates (check each, and use what
the records support):
- block-level structure. The earlier post "Admitting dense blocks to the Open kernel: 29% less peak
  memory" and its journals have admission data on what share of blocks are dense, thin or
  run-heavy;
- the GUID share of emission (28.7%, from the audit);
- per-corpus differences on CI's paired bench: the same change gains differently on wikidata, text,
  records, random, runs and onebyte. For example, the block reader's records −6.6% at 4 threads and
  random −5.8% (#745's CI bench), and #739's runs and onebyte at +4%;
- the oracle's 3.17% on Wikidata against 12.8–13.1% on non-Wikidata blocks: different data, a
  different headroom over the same family.

**Figure:** a per-block map (blocks along the x-axis, coloured by which structure dominates or which
route admits), or a small-multiples panel per corpus. Use the dataviz skill.

**§3. Exact constructions, each conditional on a structure.** For the shipped routes only, verified
in src/ on master (the conditional-routes follow-up PR to #751 is verifying the list in parallel;
coordinate with its results if it lands):
- for each route, the structural hypothesis it needs, the construction, and why it's exact;
- the shape Pᵢ ⇒ Fᵢ = R, with R the reference (the LF walk on valid input) and not "the fallback"
  unless the code shows it;
- the Open kernel at length, as the case where the unit of computation changes: rows become strips
  and returns.

**Figure:** the specification fanning out to the routes, each labelled with its hypothesis. v1's
routes figure may serve; relabel it with hypotheses, not function names.

**§4. When the data refuses.**
- **The Sturmian solver** never fires on Wikidata (`gcd(w) = 1`; 0.00% of steps with q ≥ 2).
- **The GUID wall**, 28.7% of emission.
- **The copy sector:** cutting 16–58% of hops still loses 1.14–2.64× end to end.

Then the three responses, with an example of each where the records allow. Then the hypothesis map
as a proposed artefact: transformations between hypotheses, with obstructions preserved. One rule
belongs here: a failed implementation is not an impossible construction.

**§5. The stutter.** This is the conceptual centre, so give it room. It sits after the reader has
seen the data and the constructions. The analogy, made precise:
- **The essay's opening distinction:** bad novelists vary their dialogue tags ("he murmured", "he
  stammered"), reporting a stutter from outside. A great writer instead makes the *language itself*
  stutter: variation from within the language's own system, not a description laid over a fixed
  one. Check this against the essay. I believe it opens on exactly this point. His examples include
  Gherasim Luca's "Passionnément", Melville's Bartleby and its formula "I would prefer not to",
  Kafka and Beckett.
- **The mapping:**
  - special cases bolted onto a uniform decoder are the dialogue tag, "he said, stuttering": the
    variation reported from outside a fixed language;
  - a construction whose mathematical objects change with the data's structure is the language
    itself stuttering;
  - the conclusion (recover the text) is the fixed thing being said; the means vary from within.
- **What drives the variation:** for Deleuze it is the writer, working the language. Here it is the
  data's resistance, worked by us.
- **Its limits, stated plainly:** Deleuze wrote about literature and language. He made no claims about
  algorithms or proofs, and the post claims none for him. The analogy names the shape of the process;
  it is not evidence that the process works. The evidence is §§1–4.
- Cite the essay properly: « Bégaya-t-il… », *Critique et clinique* (1993); "He Stuttered", *Essays
  Critical and Clinical*, tr. Daniel W. Smith and Michael A. Greco (1997). Read it, or a reliable
  source on it, before paraphrasing. Your v1 flag said you hadn't, so this is required now.

**§6. Local and global, and what stays open.**
- **The global case:** Open is the one change of representation so far, a classical idea (Poincaré;
  Kakutani 1943) reached through Rauzy–Veech induction (1979–82).
- **Below it:** regime-bound constructions.
- **Local to global** is possible. The RH precedent is a different proof of a solved case: Weil
  (1948), then Stepanov (1969) and Bombieri (1973), whose route transplanted. Phrase it as cautiously
  as the existing post `2026-11-22-two-centuries-of-tools.md` does.
- **Where it becomes exact:** the proof programme's substitution points (L0–L6, #751). Mention them;
  don't make them the protagonist.
- **The oracle's scope:** it bounds the (L, μ) family only. Whether the family of constructions is
  near complete is unknown, and that's the frontier.

**Closing.**
- **Proved:** nothing in Lean yet. The plan is the specification, the LF walk, then the Open kernel as
  a conditional route.
- **Measured:** the i9 figures.
- **Open:** the frontier above.
- End on the open question, not a summary.

## 5. Process for this rewrite

- Write it on a new branch with its own PR, so version 1 stays live and unpatched in the meantime.
  It merges as the replacement: the same post, renamed.
- It is a restructure, not appended paragraphs.
- Run publish-page on the final head and report the flagged-sentence share. Report each source you
  read for Deleuze, and which figures you couldn't trace.
- No chat quotes in the post, Louis's included. His words in the brief are there to carry the
  argument's force, not for quoting.
