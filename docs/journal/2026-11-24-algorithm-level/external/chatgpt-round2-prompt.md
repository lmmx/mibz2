You are returning, as the independent adversarial mathematical reviewer for Louis, to the mibz2 inversion-mathematics programme (github.com/lmmx/mibz2, a public repo; a Rust bzip2 decoder). Twice today you contributed: (1) an adversarial review of the main-theorem formulation, and (2) a reading of doomslide's essay "Against Mathematics" that proposed Hilbert's twenty-fourth problem as method and the ordered-frontier lemma as a candidate. Both were relayed verbatim to the agents (MONODROMY, a Claude mathematical agent; Neon, a Gemini agent), and they have answered. This round you read what they did with your contributions and move the programme forward.

Work in five movements, in order, and do not skip ahead: each movement's output is the next one's input. Take the strictest reading wherever something is ambiguous, and say so. Do not ask Louis questions.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
READ (all public, pinned to exact commits)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Base for MONODROMY's branch: https://github.com/lmmx/mibz2/blob/aea014932d3e737cb725b344d258cfdb7c0a707f/

MONODROMY's records, in order (append each path to the base above):
- 96, the cost theorem (G1: rigid routes spend ≥ 1 event per width-1 run): docs/journal/monodromy/2026-11-24-guid-wall.md
- 98, the calculus 𝓡 of rigid facts: docs/journal/monodromy/2026-11-24-fewer-than-h1.md
- 99, cycle 1 of the main theorem: docs/journal/monodromy/2026-11-24-main-theorem.md
- the main theorem itself, with its frozen §1 and §1a (parameters and anti-cheat clauses): docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md
- 100, positional facts: docs/journal/monodromy/2026-11-24-positional-facts.md
- 101, swap invariance (count-preserving 3-byte flips blind every aggregate rule): docs/journal/monodromy/2026-11-24-swap-invariance.md
- 102, the repairs after Neon's breaks (c₁ convention, the three-way R/N/W split, repeats are wide): docs/journal/monodromy/2026-11-24-review-and-n1-repairs.md
- 103, G1 as a scoped provenance theorem (your review's demands): docs/journal/monodromy/2026-11-24-g1-scoped.md
- 104, the ordered-frontier lemma placed; the walk → Cells → Open transformation: docs/journal/monodromy/2026-11-24-ordered-frontier.md
- 105, G1 under abbreviation and batching; the CRC; c₁ against machine time: docs/journal/monodromy/2026-11-24-abbreviation-and-crc.md
- 106, Theorem N (the scoped no over a declared class Λ*), and the process changes: docs/journal/monodromy/2026-11-24-scoped-no.md
- CLAIMS.md (the live state, NEEDS and ASK lines) and the Coordinator's channel, channel/coordinator-to-neon.md (entries #1–#5), at the same base.

Neon's branch, at https://github.com/lmmx/mibz2/blob/2658c5b0157536779566240f9c06ba17f7309fd5/:
- research/neon/n1-monodromy-audit.md (the four breaks)
- research/neon/n8-amended-n0-copies-rindex.md (the novelty split: novel hex is ~65% of h₁)
- research/neon/n9-needs-m4-swap-invariance.md (the swap flips measured; Λ_agg closes stream N too)
- research/neon/n11-stream-n-premises.md (P_N-1, P_N-2)
- research/neon/n12-…, n13-…, n14-… (your three §5 leads, all closed even as free oracles)
- research/neon/n15-needs-m8-memory-model.md (Open's simulated dependent misses)
- CLAIMS.md

The machine-cost evidence, on master at https://github.com/lmmx/mibz2/blob/b643d28ed5f3e631260fbd4f086e64cbbc4df0bc/:
- docs/journal/2026-11-20-toy-transfer.md (independent lanes bought only 5–14%; "about ten bytes are appended per retained step")
- docs/journal/2026-11-23-emit-cost-sq.md (T², the LF²-style kernel: KILLed, E = 1.22 at 10 workers)
- docs/journal/2026-11-24-retrospective-proposal.md, §2.4 ("we already had the book")
- docs/journal/2026-11-24-against-mathematics-source.md (your essay reading and Claude's, verbatim; §5 corrects your frontier figure: for L = annb$na, LF = [1,4,5,3,0,6,2])
- docs/team/research-charter/FRONTIER.md ("The target", the obstruction table) and METHOD.md

The formulation record, with Louis's words verbatim (§§19–27):
https://github.com/lmmx/mibz2/blob/bfccc7ede3a9aa3a111f434377daf6d77df987f7/docs/journal/2026-11-24-algorithm-level.md

The agents' replies, as REVIEW comments on https://github.com/lmmx/mibz2/pull/145. MONODROMY's cycles 2–8: comments 7129340413, 7129400544, 7129589449, 7129786636, 7130039012, 7130202633, 7130227614. Neon's three cycles: 7129375601, 7129569150, 7129781415.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
THE STATE, IN BRIEF (verify it; don't trust it)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

- **The frozen target, set by the agents themselves before any data.** Every one of the 238 blocks must satisfy aN·h₁N + aR·h₁R + c₁W + c₁(Pglue) ≤ h₁/2, within K = 2 passes and state ≤ Open's peak scratch. The anti-cheat clauses include no bundling, no advice, a uniform family 𝓖(S, m, k) with 𝓕 used only for validation, and the requirement to beat master on whole-decoder time.
- **G1 is now a scoped provenance theorem:** proved in prose, row-preserving, not machine-checked.
- **Your review's points.** Accepted: the domain repairs, the "no" over a declared class, and E1's bulk uses left open. Refined: "σ_hex includes copyable repeats". Repeats are provably wide rows (the M-7 lemma), and 3,645 repeated SHA1s give 3 width-1 rows in total.
- **Your currency proposal was not adopted as a target.** On this machine the emit is bound by work per event, not by misses (toy-transfer, T²). Clauses 6/6b ("dependent loads and misses no worse than Open") are kept as guards only.
- **Your ordered-frontier lemma was placed by MONODROMY (record 104):**
  - it is inside 𝓡, with c₁ = G·d, the walk's count;
  - the anchors are cheap: one range scan;
  - as a unit-price witness it is "predicted no", since it is lanes with a different address order plus ≥ 16 bytes moved per event;
  - it is Open's contraction on one-row paths under a round schedule;
  - "LF on a scattered set is exactly a set of gap counts", so the frontier and the aggregate cell are the same object.
- **Hilbert's 24th, answered for three routes (record 104 M-11).** walk → Cells → Open is contraction along compose. It preserves the emitted word, orbit order and tiling. It removes one LF evaluation per interior row of an interval. Its invariant is the set of width-1 rows J₁. On scattered ordered sets it acts, but discharges nothing.
- **Record 105.** G1 survives abbreviation and batching, and breaks only for a *schematic* fact: one proof, parametric over a scattered set. That is "the open cell, stated as proof complexity". c₁ and machine time are incomparable, with witnesses in both directions. The block CRC is inadmissible as a verifier for an exact theorem.
- **Record 106, Theorem N:** c₁ ≥ h₁ − |J₁∩U| − |J₁∩Coll_k| over Λ* = 𝓡 ∪ single-row positional facts ∪ k-local multi-row facts ∪ block-histogram global rules. It is conditional on two counts not yet produced (NEEDS M-4, M-13). The residue it names is **schematic facts over scattered sets with fine input**: no construction and no obstruction.
- **Neon** closed 14 of 14 frameworks, including your three §5 leads. It has stopped, and will be restarted for the residual target (b): the unit price of an event.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
THE FIVE MOVEMENTS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

**I. Chart.** Reconstruct the arc from the records, not from the brief above:
- what was proposed;
- who answered;
- what each answer did to the formulation;
- where your own contributions were accepted, refined or refuted.

Draw it as a single map: the claims as nodes, and "refines / refutes / closes / opens" as edges. Then state, in one paragraph, where the programme actually stands. Flag every place where my brief above misstates the records.

**II. Audit.** Be adversarial towards the agents and towards yourself. Try to break each of these, with a concrete witness or a precise gap:
- (a) Record 104's placement of the frontier inside 𝓡: "the simulation is a bijection, so c₁ = G·d". Does a frontier step really lower to exactly one compose per record? Or is the stable partition a genuinely multi-row operation that the calculus is silently expanding into singletons, which is the Pythagorean danger you named?
- (b) "The frontier and the aggregate cell are the same object." Is that an identity, or an analogy?
- (c) Record 105's claim that G1 survives abbreviation and batching, and the definition of a schematic fact.
- (d) Theorem N: its class Λ*, its two counts U and Coll_k, and its "does not say" section. Does it now quantify over global E2 in the sense you asked for? What exactly does it leave uncovered?
- (e) The argument that the emit is work-bound (toy-transfer, T²). Is the inference from "lanes bought 5–14%" to "the cost is work per event, not latency" sound? What else could explain it?
- (f) Your own earlier claims that the agents rebutted (σ_hex and the repeats; the miss currency). Concede or defend, with evidence.

**III. The porous reading.** Connect freely and liberally. The live object is now named: a *schematic fact*, one proof parametric over a scattered set of rows with fine input, i.e. the place where G1's per-row charging fails. Read it against everything you know, for example:
- proof complexity: extension variables; cut and its elimination; Herbrand complexity; schematic and parametric proofs; uniform families of proofs; bounded arithmetic and witnessing;
- parametricity, and "free theorems";
- automata and transducers acting on whole sets at once;
- word-RAM bit-parallelism and SIMD as "one instruction, many rows";
- sorting networks and oblivious computation;
- the r-index's φ and its toehold lemma;
- generating functions over the BWT;
- Pell and anthyphairesis as the two-interval Rauzy–Veech case;
- anything else.

Two facts deserve particular attention:
1. **"LF on a scattered set is a set of gap counts."** The rank differences between consecutive frontier rows are all that LF needs. A scattered set is thus described by its gaps, and record 104's generative target is a row-set family where LF^h has an o(|S|) description.
2. **The unit-price residual (b),** outside the theorem: the same events, each far cheaper on the machine.

For every connection, say what would have to be true for it to matter, and how that could be checked cheaply.

**IV. Eureka.** Move from connections to one construction, or one obstruction. Either:
- (i) **Exhibit a schematic fact.** One derivation, parametric over a scattered set of width-1 rows with fine input, whose cost is not per row, on a valid BWT and inside the frozen budget. Then state the family of row sets on which its description is o(|S|), and why real Wikidata blocks would contain that family. Or:
- (ii) **Prove that no such fact exists** for a precisely declared class that genuinely includes the global E2 residue, with its "does not say".

If you can do neither, give the single sharpest intermediate result: a lemma, a counting bound, or a supplied-state experiment with a pre-declared stopping rule. Choose the one that would decide between (i) and (ii) fastest. Apply the Mario method, payoff first: what would the decoder gain if the schematic fact were supplied free? Neon's numbers bound it: h₁ emit work is about 19% of single-thread Open kernel time on the 232-block census. Answer the question the essay left us with: **what is the operation that produces the triangles, here?**

**V. Messages.** Two paste-ready blocks, with no praise and no recap:
- **A.** Under 450 words, in this envelope:
  FROM: louis via chatgpt
  TO: monodromy, neon
  TYPE: PROPOSAL

  It holds only the corrections, the breaks, the sharpest next step for each agent, and any decision rule that must be fixed before data.
- **B.** Under 200 words, for Louis alone: what this round changes about the likely outcome (yes, a scoped no, or a residual-only machine win), and what that would mean for the blog post he is writing on "what the current moment in mathematics represents".

**Labels.** Tag every claim proved / argued / conjectured / guessed, with references where they exist. Where you correct the agents or yourself, cite the file and the line.
