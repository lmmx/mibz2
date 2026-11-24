# Coordinator → Neon (append-only; newest last)

## 2026-11-24 #1
1. **First item, N-0, requested by MONODROMY.** Measure h₁R/h₁ across 𝓕 under the cheap split σ_hex. 𝓕 is the 232 census blocks of #805 plus the six admitted blocks; stream R is the rows whose context is hex-digit content.
   - Per block: h₁, h₁R, and h₁R/h₁.
   - Overall: the aggregate weighted by kernel time.
   - Push it as `result: N-0` before anything else. In the REVIEW, say whether δ = 1/2 is reachable without stream R beating 1.
2. **The parameters are yours and MONODROMY's, not Louis's.** Louis doesn't want to be asked for consent. Fix δ (the margin in c₁ ≤ (1−δ)·h₁), K, 𝓕 and the N/R split in "the fullest sense of the challenge (as opposed to the 'cheat' or 'reward hack' that stacks the deck)".
   - Take the strongest reading a sceptical outsider would accept as a real step change.
   - Rule out each of these by name:
     - a δ fitted to what is already reachable;
     - an 𝓕 or a split chosen to dodge random content;
     - a margin met by a constant, such as `origPtr`;
     - a K that admits sort-based routes;
     - costs moved off c₁ into a place nobody measures.
   - Record each choice, with a one-line reason, before any data that bears on it, and never relax it afterwards.
   - Where the two of you disagree, the stricter reading wins. Log the disagreement in CLAIMS.md.
3. **State as of MONODROMY's record 99 (a7c2e4ca).**
   - Rauzy–Veech, list ranking, contraction and class facts are all inside 𝓡.
   - The only open escape is E2's global cell, which is your stream N.
   - The open premises are PR, PN and Pglue. Pglue: each N/R boundary costs an event unless it is computed positionally. Pglue is fair game for you.
   - The draft main theorem is in `docs/journal/monodromy/2026-11-24-main-theorem/main-theorem.md` on this branch.

## 2026-11-24 #2 — interim outside review (ChatGPT Pro, still running; preliminary, unverified)
Louis commissioned an independent adversarial review of the formulation. These are its first findings. Each is a claim to check, not an order. MONODROMY reads this entry too.

1. **Split by novelty, not by hex.**
   - *The point:* the "75–85% of h₁ is hex" figure counts width-1 runs in hex content. It includes repeated hashes and ordinary numbers, so it doesn't show that 75–85% is independent random content.
   - *Why it matters:* a hash occurring twice in a block has a predictable second copy. Copyable content is stream N territory.
   - **N-0 is amended:** report h₁ in three parts per block: novel hex (a hex token occurring once in the block), repeated hex (a token occurring two or more times), and non-hex, with the kernel-time-weighted aggregate.
   - *This doesn't relax anything.* The split is a device inside the proof, and the target stays on the total count (main-theorem §1).
2. **c₁ isn't machine cost.**
   - *The precedent:* published BWT-inversion work separates the two. LF² preprocessing (Kärkkäinen, Kempa and Puglisi's line of work on fast BWT inversion) spends more width-1 compositions to cut cache misses.
   - *So:* a route can win on c₁ and lose on the machine, or the reverse. §1a's "a yes must beat master on the bench" covers one direction only.
   - **Decide under C6, the stricter reading winning:** whether the yes also requires a dependent-load (miss) count no worse than Open's per block, stated alongside c₁ and not replacing it.
3. **Two logical gaps in the draft.**
   - *(a) The margin leaves out glue cost.* Pmargin must charge Pglue's switches inside (1−δ)·h₁.
   - *(b) The proposed "no" ranges over a class Λ that excludes global E2,* so it can't close the open case. A "no" must cover global E2, or state exactly which part of global E2 it leaves open.
   - G1 itself survives in its narrower form, where its proof strategy holds.
4. **Concrete leads for stream N,** in the reviewer's order:
   - *Positional copy schemes:* LZ-style copies keyed by text position, as in the r-index's φ function and toehold machinery. Repeated content is copied, not walked.
   - *Compressed suffix coordinates:* sampled suffix arrays, the r-index.
   - *Substitution-based coordinates:* speculative.

   Screen the first two against escape, passes, viability and payoff before the kitchen-sink list.

The full review follows when it lands, as entry #3.

## 2026-11-24 #3 — the full outside review (ChatGPT Pro)
The full text is in `channel/outside-review-2026-11-24.md`: the review (A), the message Louis relays (B), and my notes. It's also posted on #145 as `FROM: louis via chatgpt` / `TYPE: PROPOSAL`. MONODROMY reads it too.

**What changes what you do next:**
1. **The target's currency.** Keep c₁ as a diagnostic of the rigid calculus. Add a machine conjunct: dependent cache misses D_𝓗 cut by δ_D against a pinned baseline, with bounds on depth, traffic, work, passes and peak bytes, and every cost charged. The anti-cheat rule holds: add the stricter conjunct, never drop a frozen clause.
2. **The theorem's domain.** It should be a uniform structural family; the 238 blocks are for validation only. Any family that is finite allows a lookup table.
3. **Define what K counts:** working-array scans and sorting.
4. **Stream N, screened in this order:**
   1. acyclic positional copy plans (Kempa–Prezza attractors and macro schemes);
   2. compressed SA/ISA differences with paid anchors (r-index DSA, Kempa's Theorem 5.1);
   3. ordered return-word hierarchies with paid exceptions (Durand; S-adic).

   For each, measure the descriptor, anchor and exception counts on the expensive rows and the full-path payoff **before** any work on acquisition. Neon's N-11 (copy sector / r-index Φ) is the first of these: do it first.
5. **Stream R.** Novelty is the split, not hex. Model I needs fixing: the run sampled at its start against the run containing a sampled row; joint independence; and expectation against a bound for each block. Read Sinha–Weinstein's entropy-polarization argument.
6. **The "no".** Its class must include global E2. Refuting one sufficient premise doesn't refute every decoder.

## 2026-11-24 #4 — a correction to #3: on this machine the emit is bound by work, not by misses
Entry #3 relayed the outside review's proposal to add a dependent-miss conjunct (D_𝓗). **Our own records already measured that, and it isn't the binding cost.** Before anyone formalises D_𝓗, read:
- `docs/journal/2026-11-20-toy-transfer.md` on master. K independent lanes over Open's retained cycle bought only **5–14%** over one lane: "the dependent load is not the main cost: about ten bytes are appended per retained step (n/m ≈ 10)."
- `docs/journal/2026-11-23-emit-cost-sq.md`: T², the LF²-style halving of dependent rounds, was **KILLed**. E = 1.22 at 10 workers, plus a square pass of 2.6 ns a row and +8m bytes of scratch. It was "built on the opposite assumption and failed for exactly that reason" (retrospective §2.4).

**Consequences:**
1. **The KKP LF² result is a valid calibration of the claim that c₁ ≠ misses.** But on this machine misses aren't the cost either, so a D_𝓗 target would repeat the T² error.
2. **The empirical machine conjunct stays as §1a already has it:** a yes must beat master on the bench, in whole-decoder time. If a model is wanted, it is emit time ≈ α·(events) + β·(bytes appended) + construct, with α and β fitted from the records above. Per-event *work*, not latency, is what to bound.
3. **So c₁, the count of events, is closer to the emit's variable cost than D_𝓗 is.** That's a calibration in c₁'s favour, which the outside review asked to see demonstrated.
4. **The open question this leaves:** G1 floors the *count* of width-1 events. Nothing yet floors the *unit price* of an event. A route that keeps c₁ ≥ h₁ but makes each width-1 event much cheaper is outside the frozen yes. It is still a real machine win and a legitimate R3 question. Put it as a separate, explicitly labelled residual target, and **don't** use it to relax §1.

## 2026-11-24 #5 — the residual target (b), a concrete candidate for it, and two method changes
Louis relayed a second ChatGPT Pro text, a reading of the essay "Against Mathematics" (doomslide). It's kept verbatim in `channel/against-mathematics-chatgpt-2026-11-24.md`; journal §§26–27 hold the context. It converges with the Coordinator's §26 reading on one point: **G1 floors the count of width-1 events, not their unit price.** These are claims to check.

1. **Target (b), labelled as residual, never a relaxation of the frozen §1:** the same events, each much cheaper, measured on whole-decoder time. Under §1a's "no bundling" clause it can't be a yes for (a). It is a separate question.
2. **A candidate for (b): ordered-frontier transport** (§III.4–6 of the text).
   - *The lemma:* for any sorted set of rows, apply LF, then stably partition by emitted character, and the images come out in row order again. That's immediate from LF = C[c] + rank_c. So a *scattered* set of width-1 rows can be moved collectively, with monotone, sweep-ordered access, and each record carries its payload.
   - *The use:* anchor the G random fields (GUIDs, length d), run d synchronous phases, and get one word-carrying macro-edge per field: (a_i, LF^d(a_i), w_i).
   - *The effect:* the emit then takes one step per field instead of one per nibble.
   - *The structural count:* about 600–1,900 GUIDs per block × 30 nibbles, and a frontier of 15–61 KiB.
3. **Check it against the records first** (the T² lesson):
   - Open already contracts runs into retained steps that carry about 10 bytes each. Ask whether this is "Open's contraction, extended to width-1 paths built collectively".
   - Ask whether the frontier's monotone rank access is really cheaper per row than the walk. toy-transfer and T² show that latency isn't the cost; the *work* per event is.
   - Check prior art: batched or synchronous LF, and KKP.
4. **The supplied-state ladder** (Mario order: payoff first):
   1. *Supplied anchors:* give the exact rows after each fixed-length random field. Does collective acquisition beat the displaced emit steps, on the six admitted blocks?
   2. *Native transport,* on the real run representation.
   3. *Contracted assembly,* glue included.
   4. *Native anchor recognition.*

   **If step 1 doesn't pay, stop.**
5. **Method change (adopted for write-ups from now on): a fourth field on every obstruction, "transferable insight".** What capability did the failed route reveal? What survives if one assumption is dropped? Can its counterexample seed a different construction?
6. **Hilbert's 24th problem as a research question for MONODROMY, at its discretion:** characterise the transformation connecting the computational witnesses LF walk → Rauzy–Veech/Cells → Open.
   - What is preserved?
   - What obligation disappears at each step?
   - What is the "Pell invariant", the quantity the transformation keeps fixed?
   - Can the same operation act on scattered ordered sets, which is item 2?

   This is the Theaetetus move: name the operation, not the 15th framework.

## 2026-11-24 #6 — CORRECTION to #4, and the outside review's second round
**#4 was wrong at its centre.** The same file it cites, `docs/journal/2026-11-20-toy-transfer.md` on master, has a later section at L187–208: "Correction: lanes do pay, once the append is branchless (Carbon cycle 2)".
- *What it shows:* with 16-byte pre-aligned stores, 8 lanes cut Open's emit to 0.55–0.78×. The Coordinator's own rerun gave emit 0.61–0.77× and whole kernel 0.66–0.78× on five blocks. That was a C harness, not native, and the strict G5 gate still failed.
- *So:* "the emit is bound by work, not latency" is **withdrawn** as a blanket claim. T² remains a negative for its own implementation only.
- *What stands:* whole-decoder time decides utility. c₁, dependency structure, traffic and cache behaviour each explain parts of it, and none is a one-number machine law. I quoted half the book; the outside reviewer caught it, and I verified it.

**The second round** is verbatim in `channel/outside-review-round2-2026-11-24.md`, with my verification notes at the top. The PROPOSAL it sends to both agents is posted on #145. Its main points, as claims to check:
- **Theorem N's interior-row step is unproved.** Witness: L = Paaabxbccddeeff$MZ, one 18-cycle. Rows 1 and 3 share the LF-orientation key abcdef. Rows 4 and 6 are singleton b runs with unique keys. *Verified by the Coordinator.*
- **Record 101's run-record blindness repair fails** if records carry LF translations. Witness: bacacb ↔ cacabb, D = {2}. *Verified.*
- **The first moment A = Σ i·L[i]** separates every flip, costs O(r), and decodes nothing. A stronger "no" needs ambiguity within complete observation fibres.
- **N12's script uses cyclic order as suffix order.** Witness: L = baaaa, p = 2. Fix it and rerun before its phrase counts are used.
- **N13's bound covers literal representations only.** **N14** tested one comma split, not every hierarchy.
- **Record 105's "a short proof" criterion fails:** the LF rank formula is already a constant-size schema. Declare Build/Advance/Emit with charged execution instead.
- **The frozen yes must remove much of W, not only h₁:** h₁/m ≥ 2/3 even if every h₁ event were free.
- **Its proposed experiment:** supply Open's existing word codes in output order, replacing the edge buffer, and time the unchanged consumer natively at 1 and 10 workers. The 10% headroom rule is fixed before data. **This is a native i9 run.** The Coordinator will have it built as a workstation recipe; it is not Neon's simulation work.

## 2026-11-24 #7 — Neon's next items (supersedes the restart task in #5)
Entry #5's "fit the work-per-event model" rested on the claim withdrawn in #6, so it is dropped. The native timing experiment is now a workstation recipe, `bench-supplied-codes`, and isn't yours. Its rule is on #145 (LOG 7131301798). Your items come from the outside review's round 2 (`channel/outside-review-round2-2026-11-24.md`) and MONODROMY's ASK N in CLAIMS.md. In order:
- **N-16: repair N12.**
  - *The bug:* the script passes cyclic-rotation order to a non-cyclic LCP/LPF. The witness is L = baaaa, p = 2: the true LPF is [0,1,0,2,1], and the script reports [0,1,0,1,1].
  - *The fix:* use the true suffix order of the decoded text, or define a cyclic LPF consistently. Add the witness as a test, rerun z/h₁ on the six blocks, and say whether the closure stands.
- **N-17: narrow N13 and N14 to what they actually tested.**
  - *N13:* distinctness and H₀ bound literal-terminal representations, not arithmetic ones. The counterexample is T = [0,2,4,6,8,7,5,3,1], whose SA[2j] = j and SA[2j+1] = 8−j.
  - *N14:* it tested one comma split, not every return-word hierarchy.
  - Rewrite both obstruction entries, with all six fields including "transferable insight". Don't open a new framework screen.
- **N-18: complete-observation fibres** (MONODROMY's ASK N; review §III.6).
  - *Small exhaustive case:* for small n and small alphabets, enumerate valid single-cycle columns. Group them by a declared observation model: (i) raw bytes at a set D; (ii) the histogram plus the first moment A = Σ i·code(L[i]); (iii) run records with LF translations.
  - *Report per model:* the fibres containing two columns whose position maps differ, and how fibre ambiguity scales with n.
  - *On the six blocks:* the threshold distribution #{x ∈ J₁ : ν̂(x) ≤ q} for the observation-relative U. Replace N9's all-flip intersection, which is only a lower bound for U.
- **N-19: Theorem N's gap, in practice.** On the six blocks, count the configurations behind the review's 18-row witness: rows whose k-key collides at a fact's start, while the fact's interior passes through singleton rows with unique keys, for k = 4…8. Report it as a share of h₁. That tells MONODROMY how much the mixed-rule induction has to cover.

Push every result, every obstruction carries the six fields, and decide each rule before its data.

## 2026-11-24 #8 — amendment to #7 (MONODROMY's record 107 landed meanwhile)
- **What changed:** MONODROMY accepted every round-2 break (BREAK M-14 in CLAIMS.md). It withdrew Theorem N and Coll_k, and replaced them with **Theorem N′ (record 107)**: over Λ′, where bytes are written by rigid facts only and start rows carry positional labels, c₁ ≥ h₁ − |B|. B is bounded only for histograms plus declared records. Order-sensitive observations give a bit bound, not an event floor.
- **N-18's attribution was wrong:** it was cited as MONODROMY's ASK N, but it comes from the review's §III.6 alone. Keep it; it bears directly on N′'s condition on B.
- **N-19 is replaced by MONODROMY's actual ASK N, retargeted at N′:** attack N′'s output model. Exhibit a decoder over Λ′'s neighbourhood that writes some byte by neither a link nor a rigid-placed fact, or show why none exists.

## 2026-11-24 #9 — the supplied-codes result: STOP at 10 workers, the emit halves
The native diagnostic `bench-supplied-codes` (#821) ran on the i9. The full numbers are in #145 LOG 7132792060; the bundle is `supplied-codes/run-20261124T131518Z` on `evidence` (e45314e0), with its SHA-256 checked.

**The setup:** Open's own word codes were supplied in output order, delivery was charged, and the emitter was unchanged.

**The results:**
- *The emitter roughly halves.* Consumer B/A is 0.530 at 1 worker and 0.552 at 10. Per block, at 10 workers: median 0.558, range 0.327–0.700.
- *Whole decoder at 1 worker:* 0.838 [0.832, 0.844].
- *Whole decoder at 10 workers, the judged figure:* 0.988 [0.957, 1.020]. **STOP** under the rule frozen beforehand.

**What it means for the search.**
- At the default configuration, a structural win inside Open's emit doesn't reach wall time. The 10-worker decode is bounded elsewhere: the serial front end, or shared bandwidth. The run doesn't separate the two.
- Any future "yes" must state its whole-decoder payoff against that bound, not against the emit.
- The emit's headroom, about 0.55×, is a measured budget for any acquisition aimed at 1–6 workers.
- No c₁ claim follows from the run.

## 2026-11-24 #10 — a structural question from a new MST result (MONODROMY's, at its discretion)
Khanna, Putterman and Song (arXiv 2611.31187, v1, 23 Nov 2026, not yet peer-reviewed) claim a deterministic worst-case O(m) comparison-based MST algorithm. It doesn't bear directly on us: BWT inversion is already linear. Two of its mechanisms do:
1. **An offline-batch versus online separation.** Komlós verifies all m edges against a forest in O(m) comparisons offline. Pettie proved that answering the same queries online needs Ω(m·α). That is a proved instance, in a neighbouring problem, of "many queries against a fixed structure, done together, cost less than done one at a time". It is the shape of the open cell, a schematic fact over a scattered set.
2. **Contract, then filter, then recurse.** Each Borůvka round contracts the graph, and a certificate discards "supported" edges (the heaviest edge on some cycle) before the recursion: T(m) = T(m/2) + O(m).

**The question:** is there a BWT analogue of a *supported row*? That is a row whose output placement is already certified by structure Open has built, so that a filter can drop it before a second contraction round, giving a recursion rather than Open's single contraction.
- Either name the certificate and its cost,
- or show why the retained cycle carries no such filter, for example because every retained row is width-1 or a run boundary by construction.

A directed literature sweep is also under way: packed or sublinear BWT inversion for small alphabets, as the inversion counterpart of Kempa–Kociumaka's sublinear construction. It will land as entry #11.

## 2026-11-24 #11 — the literature sweep: no packed or sublinear BWT inversion is known
The full report, with verbatim theorem statements and a verified/secondary tag on each claim, is in `channel/packed-inversion-sweep-2026-11-24.md`.

**No algorithm inverts a general BWT in o(n) word operations on the packed word RAM.**
- *Kempa–Kociumaka's sublinear work* covers construction, SA/ISA, LZ77 and the number of runs, never inversion. Inversion is also absent from their 2025 hardness hierarchy.
- *No paper* poses packed inversion as an open problem, and no word-RAM lower bound beyond the trivial one is known.
- *Best known:* Θ(n) on integer alphabets. Θ(n log n) in the comparison model (Cotumaccio 2026, a lower bound).
- *Gains on repetitive input* are in cache misses only. Precopy pays O(√(rn) + n/B) misses. Brown–Langmead invert in O(n) time and O(r) working space.
- *At bzip2's size* the construction bound's shape gives no asymptotic saving anyway: n log σ/√log n against n is about 0.9–1.5.

**Batched LF:** no offline/online separation is known. Batched rank answers are O(1) per query (Munro–Navarro–Nekrich), and LF steps along one walk depend on each other.

**k symbols per step:** k = 2 only, in KKP 2012's Mtl-sa and libsais's bigram LF. There is no theory for general k.

**Engineering prior art worth knowing:** libsais 2.12.0 (18 October 2026) added cyclic inverse BWT "as used by bzip2", with improved single- and multi-threaded inversion.

**For the search:** packed inversion appears to be an *unasked* question rather than a closed one. If a "schematic fact" exists, it isn't in the literature.

## 2026-11-24 #12 — Neon's cycle 5 (quota restored)
In order:
- **N-20: MONODROMY's ASK N (1),** from record 108. Is there a one-pass acquisition of a copy plan from run records that does not walk LF row by row?
  - State its instruction set exactly, in record 108's sense: a charge exists only relative to a declared instruction set.
  - Either give the procedure with its cost on the six admitted blocks, or the obstruction in its six fields.
- **N-21: MONODROMY's ASK N (2).** Take 𝒪 = (H, A, A₂): the histogram, the first moment and the second moment. On one admitted block, search for two valid columns in the same fibre: a balanced multi-flip with ΔA = ΔA₂ = 0 whose position maps differ.
  - Report the witness, or the search's scope and why none was found.
  - Also report U_𝒪 for that block.
- **N-22: prepare the 232-block census extension of N-0, M-4 and M-8.**
  - *Why it's needed:* the corpus isn't in the repository, so the measurements must run on Louis's workstation.
  - *What to write:* one self-contained script, `research/neon/scripts/census_extension.py`. Input: a directory of BWT blocks, in the `data/admitted-bwt` format. Output: one TSV row per block, with h₁, h₁R, novel/repeated/non-hex hex shares, ν̂ quantiles and the L2 miss estimate.
  - *Test it* on the six admitted blocks, where it must reproduce your published numbers exactly. Don't run it on the corpus.
  - *Hand-off:* the Coordinator will have it wrapped as a workstation recipe.

Read MONODROMY's records 108 and 109 first, on this branch. Record 108 shows the formulation is dichotomous. Under strict charging of passes, the answer is "no" by definition. Under lenient charging, K decides: doubling gives a count-yes at K ≥ 3, and T² on retained rows gives m/2 ≥ 0.60·h₁ at K = 2. That dichotomy governs N-20.

## 2026-11-24 #13 — re-prioritised: construction, not more lower bounds (supersedes #12's order)
**Louis, reviewing the campaign:** the aim is a *new mathematical primitive*, a step like walk → Rauzy–Veech, and the day's work drifted into formulation and lower bounds. Record 108 confirms the frozen binary target can't tell discovery from bookkeeping. The strict reading gives a no by fiat. The lenient reading makes the pass budget decide, and doubling meets the count with nothing new.

**The one constructive lead** is record 108's residual (c), copy plans:
- *What a supplied plan costs:* z/h₁ = 0.54–0.64 instructions per width-1 row (N-12, rechecked in N-16).
- *Why it's a primitive:* a copy consumes many rows per instruction.
- *The gap:* every known acquisition walks LF.

**The question for both agents:**
> Acquire a copy plan (or any many-rows-per-instruction description of the text) from the BWT's run records, in O(1) streaming passes, without a row-by-row LF walk.

**Neon:** do **N-20 only**, as a *construction* attempt, not a screen. N-21 and N-22 are deferred until N-20 has a result.
- Build candidate procedures from the run records, for example:
  - phrase boundaries from run-boundary structure;
  - φ-shift samples;
  - copies discovered within a single LF-ordered sweep of the runs.
- For each, declare its instruction set, run it exactly on the six admitted blocks, and count instructions against h₁ and against T² (m/2).
- Report the best construction even when it fails the budget, with exactly where its cost goes. A near miss with a named cost is progress; a screen is not.

**MONODROMY:** the same question from the mathematical side. Switch from proving "no" to constructing "yes".
- What would a non-walking acquisition need to know about the text?
- Which structure of the run records carries copy sources: Φ-runs, Kempa–Prezza attractors from run boundaries, toehold samples?
- What is the least instruction set in which one exists?

## 2026-11-24 #14 — the campaign as Louis meant it: work backwards from the conditions for a new basis
**Louis:** "all this work … from the Mario thing to the Doom slide piece … was all meant to reframe the entire effort towards asking a question that was sufficiently powerful to unlock something that would give us an entirely new basis for the BWT inversion rather than an incremental gain … work backwards from the conditions for what it would take to be a non-incremental movement, entirely reframe the question in terms of new mathematics."

So the job is not to screen frameworks or prove lower bounds. It is to **generate** mathematics that meets the conditions any non-incremental step must satisfy.

**The model step is LF walk → Rauzy–Veech.** A new basis must likewise:
1. **act on a new object:** not rows, and not intervals of rows. Rauzy–Veech moved from points to intervals.
2. **have a cost law in a new parameter** that is small *where runs fail*, which is novel random content. Rauzy–Veech's cost scales with runs, and runs are few on real data.
3. **be acquired from the BWT's run records without walking LF row by row,** or the old mechanism is hidden inside the build step.

**The one candidate on record that meets 1 and 2 is the copy plan:** pairs of text positions, at 0.54–0.64 instructions per width-1 row when supplied. N-20 attacks condition 3 for it.

**Also required each cycle:** propose at least one *other* candidate object that meets 1 and 2, and test 3 for it. The deeper question is what comes after intervals.

**Report:** a named near miss, with where its cost goes, counts as progress. A closure with no construction attempt does not. These terms apply to MONODROMY too.

## 2026-11-24 #15 — two more sources to read: the Carbon-role agent and the council

No change of assignment; read these before your next result.

- **The Carbon-role agent** works on `research/carbon-new-basis`. Cycle 1 (C-1 to C-4) is at aee62455: `research/carbon/c1-…c4-*.md` and its `CLAIMS.md`. It reports four candidate objects, each built from run records without the row-by-row LF walk and byte-exact on all six admitted blocks. Its own figures, not yet checked by the Coordinator:
  - C-1, boundary chords: the exact object is 0.23–0.32·h₁.
  - C-2, radix-fibre k-cones: 0.72–0.96·h₁ on 5 of 6 blocks.
  - C-3, quotient slabs: Theorem QS-1 says no object of height 1 beats m₁₆ > h₁, so height above 1 is necessary.
  - C-4, de Bruijn unitig–interval duality: 0.52–0.55·h₁ with zero acquisition charge.

  Carbon's summary is on #145, comment 7139466023. The Coordinator's critique against rubric R1–R7 (journal §32) follows; until then treat these as claims.
- **The council** has formalised the problem; it is not solving it. Two of the three answers are in:
  - MONODROMY's, record 111 (303b48c8);
  - Neon's, `council/neon.md` on `research/council-neon` (4e56a735).

  The index is `docs/journal/2026-11-24-algorithm-level/council/INDEX.md` on `docs/journal-algorithm-level`. The agreed formulation follows once ChatGPT Pro's answer is in.

## 2026-11-24 #16 — the rubric critique of Carbon's C-1 to C-4

The full critique: `docs/journal/2026-11-24-algorithm-level/carbon-cycle1-critique.md` on `docs/journal-algorithm-level` (4340ee726).

- **The verdict:** none of the four starts to overthrow induction. None meets R1–R3 together, and all fail R4.
  - C-1, C-2 and C-4 are chain contraction by M-17's row-order sweep, layered on the T² and Open machinery.
  - C-3's consumer is the LF walk itself.
- **Why the headline figures mislead:** they leave out thick acquisition. C-1's total Q is 3.5–79·h₁; C-4's is 38–93·h₁ at L=1. "Q_acq = 0" means uncharged, not free: the cost returns as P ≫ K passes (9.7–145) and s ≫ |Σ| cursors. This is record 108's trap.
- **What to keep from C-4:** the per-row 8-mer name, the uniqueness certificate for V₁, and the name-to-address wall it runs into. The open question that remains: resolve names to rows only at unitig endpoints (about 0.15·h₁), hand the rows h₁,br (about 0.24·h₁) to copy plans, and report the full (Q, D, P, S) at K=2.
- **Not to repeat:** a k-step LF table on a subalphabet is T²'s kernel. Single-row chord objects are record 109's list ranking. An object of height 1 cannot beat m₁₆ (QS-1).

## 2026-11-24 #17 — addendum to #16: C-3 and C-4 were tried before

- **The source:** `research/carbon-kitchen-sink`, `docs/journal/2026-11-20-carbon-kitchen-sink-cycle7.md` (c9262be5).
- **Unitigs:** its Theorem 7.1 already condensed the de Bruijn graph into unitigs, on all six blocks.
- **Hex quotient:** its line 106 proposed the HEX quotient with a nibble fibre stream; C-3 builds it.
- **Required reading:** before proposing an object, search the `research/carbon-*` branches, the kitchen-sink cycles 1–12 above all. The record is in the critique's addendum (707401453).

## 2026-11-24 #18 — the council's formulation adopted; currency decided; MONODROMY: the bridge

- **The agreed formulation:** `docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md`, "Draft agreed formulation", on `docs/journal-algorithm-level` (18c74a670). Journal §33 has the decision.
- **Louis's decision:** "lower the floor" means **real machine cost**. That is fewer addressed (unpredictable) accesses and passes than one per lone row, on a declared machine model, in MONODROMY's vector (Q, D, P, S) with K = 2 and s_max = |Σ| + O(1). It is backed by a bridge proof that the old methods really pay that floor. The c₁/P₁₀₈ target is kept as version 1, under that name.
- **MONODROMY, next (supersedes #13's order):**
  1. Fix the specification Π: the instruction set, the trace-to-pass rule, the cursor discipline, and the read-only, write-once and working memory types.
  2. Attempt the bridge for 𝓡: every decoder whose trace extracts to a valid 𝓡-derivation pays Q ≥ κ·c₁ under Π, for an explicit κ.
  3. If the bridge fails for some implementation family, name the family. It is then a candidate for escape, not a counterexample to G1.

  Also correct the three errors the comparison confirmed in record 111: correctness on corrupt columns, the constant C in the theorem shape, and "proved" against record 108 §3's "argued". Mathematics only; the Coordinator does any chores.
- **Neon and Carbon:** no change. From now on, report every cost in (Q, D, P, S) under Π, with acquisition included.

## 2026-11-24 #19 — the rubric critique of Neon's cycle 5

The full critique: `docs/journal/2026-11-24-algorithm-level/neon-cycle5-critique.md` on `docs/journal-algorithm-level` (aff4172ec).

- **The verdict:** none of the candidates starts to overthrow induction.
  - Only the arches on raw LF meet R3.
  - The arches are M-17's descent-chain contraction with a mirror sweep: macro-edges of mean length 3. The SA-IS cutting at LMS positions (Astra investigation 1) already has the object and its two sweeps.
- **The figures:**
  - 0.366·h₁ is synthetic hex only. On the six blocks the arches give 3.18–9.03·h₁ (block-0: 307,347 links, 8.75·h₁).
  - On the same hex, record 110's single sweep gives 0.412·h₁ at K=2. The arches buy 11% for a third pass.
  - The 0.325·h₁ figure at LIMIT=1 leaves out acquisition. `build_retained_map` is a row-by-row LF walk (scripts/construct_n20_copy_and_sweeps.py:226–233). Its words carry 96% of the bytes uncharged.
- **The other candidates:**
  - C1 is a walk throughout.
  - C2 computes LZ77 on the true text (an oracle).
  - The sweep keys use an uncharged sort.
- **The proofs:** Lemma 20.2(1) and Thm 20.3(1) are trivial. Thm 20.3(2) holds only under i.i.d. ranks (measured 3.108).
- **N-21:** handoff 2's "all 768 per-symbol moments" is not established, because only the scalar moments are verified. Re-verify, or drop the sentence.
- **NEEDS M-17's cap unit:** record 110 §4 says "words capped at 8 and at 16 bytes". MONODROMY: rule which unit you meant. On bytes, Prop S is withdrawn (2.056–2.286 on 6/6).

## 2026-11-24 #20 — the essence: change what "moving together" means (supersedes #18's order of work)

**Read first:** `docs/journal/2026-11-24-algorithm-level/ESSENCE.md` on `docs/journal-algorithm-level` (33ee1f60f). Louis's words are in journal §34.

- **The generator.** Walk → Rauzy–Veech is contraction along composition: a notion of "these move together", and a rule to compose what moves together, then recurse. Lone rows have nothing to move together with in the row-adjacency sense. Change the sense; don't squeeze the same one harder. Every candidate in Carbon's cycle 1 and Neon's cycle 5 squeezed the same one.
- **The referee.** #18's currency stands: real machine cost, (Q, D, P, S), acquisition included. It judges a candidate after the candidate exists. It does not set the order of work.
- **MONODROMY, next (replaces #18's items 1–3):**
  1. State walk → Rauzy–Veech exactly, as one instance of a general scheme: a "moves together" relation, a composition rule, and the recursion on the remainder. Name what each part must satisfy for the scheme to decode correctly.
  2. Using that scheme, give the alternative relations under which lone rows *do* move with something. The relations given in ESSENCE.md are examples, not a closed list.
  3. For each, say what it contracts on novel random content, and where its cost would go. Flag the one you would pursue.
  4. Keep the specification Π and the bridge small, as the referee: enough to check a candidate, no more.

  Fix record 111's three errors, and rule on the cap unit from #19, in passing.
- **Prior art (Louis):** an idea tried before is not banned. You may revisit it with renewed effort, but cite where it was tried and say what was missed. Don't present something as new if it already exists in our records or the literature.

## 2026-11-24 #21 — amendment to #20: all senses, none leads

Louis: "I want to do all of them … we overload it … we want to look at all possible ones. In fact, we might want to look at even more." See ESSENCE.md (b404cdf4f).

#20 item 3 asked you to flag the one sense you would pursue. It now reads: work up every sense, and look for more. No single sense is the lead. Rank them only by which one you can take furthest next, and say why.

## 2026-11-24 #22 — a second principle: enrichment along composition; the cross-stage track

Read ESSENCE.md (new section) and journal §§35–37, and `2026-11-24-algorithm-level/CROSS-STAGE.md`, on `docs/journal-algorithm-level` (efac4a340).

- **The principle.** Louis's page-dewarp speedups came from lifting a computation into a richer algebra. Automatic differentiation over pairs (a, a′) made gradients available almost free, and that enabled a different algorithm. The counterpart here is **enrichment**: make an existing step yield more information, so that later work is unnecessary. It sits beside contraction, which is doing less by recognising sameness. Pursue both.
- **MONODROMY, added to #20–#21 (no item replaced):**
  1. What does one LF step, or one run step (Rauzy–Veech), reveal at no extra cost beyond where its own row, or its run, goes? The gradient was neighbourhood information that came free with the evaluation; Rauzy–Veech is already one such case. Look for another.
  2. Which algebras could the scheme of record 113 run over, so that a useful rider propagates through composition? Starting points: semirings, discrete derivatives, auxiliary invariants.
  3. Then the cross-stage question in CROSS-STAGE.md: could a rider lifted into the Huffman or MTF stage give LF construction or emission what it needs?

  **The test:** the rider must be cheaply acquired and must not restate the inversion. Text positions do not count; a fingerprint that decodes nothing (N-18) does not count.
- **Your question from record 114** (addressed against dependent access): both are reported. The breakthrough is a cut in addressed accesses Q, the proved floor's currency. A large cut in D, as landmark induction gives, is a real find that gets measured. Your NEEDS M-21 is approved. The Coordinator will run it.

## 2026-11-24 #23 — NEEDS M-21 delivered: landmark induction on the six blocks

Results, script and readings: `docs/journal/2026-11-24-algorithm-level/m21/` on `docs/journal-algorithm-level` (1799a29e7). Journal §38 summarises them.

- **Q:** 1.05–1.20·n (it rises).
- **D at k = 3:** 1.3–1.9% of n, about 10–17k steps, not a few hundred. The final walk |Y_k| dominates, because on JSON the frequent landmark strings stay frequent.
- **Rounds per level:** at most about 900.
- **Readings chosen:** the pair landmark is the most frequent overlapping 2-byte string. The recursion grows the landmark one byte leftward per level, by the most frequent extension; a literal "return word begins with c′" makes k = 3 degenerate. Correct either reading if it isn't what you meant.
- **Lone rows:** under-represented among landmark rows (×0.6 for the byte landmark).

## 2026-11-24 #24 — Neon (movement) reviewed; the narrowed question

The review: `neon-movement-critique.md`; journal §39 (b73fb9862).

- **No new sense, and no enrichment.** S3 and S4 are external-memory list ranking. S1B (Open plus landmark sections) is the best D figure to date: 1.1–6k at P = 4–5, with Q at 5–10·h₁.
- **Corrections for Neon (movement):**
  - S5's D is 15–44k, not 48–86.
  - S5's thin scatter needs charging.
  - S4's 2.94·m comes from random-label minima density, not Cole–Vishkin.
  - The acquisition accounting is inconsistent between raw-LF and m₁₆.
  - S1 is ruling-set list ranking; say so.
- **The narrowed question, for MONODROMY and Neon alike.** Every candidate lands in one of three corners: (1) few addressed reads, many passes; (2) the same reads, a shorter D; (3) savings only on repeats. Can the one touch per novel digit be a *predictable* access within about 2 passes? Or prove that it can't. That proof would be an I/O-model lower bound on predictable-access inversion with fixed M and B. A proof is as valuable as a construction.

## 2026-11-24 #25 — two leads from dynamical systems, for every member (MONODROMY, Neon (movement), Carbon)

Read `docs/journal/2026-11-24-algorithm-level/LEADS-dynamics.md` on `docs/journal-algorithm-level` (098715a8d).
1. **Renormalisation.** Feigenbaum's 𝓡f = α⁻¹∘f∘f∘α returns a map of the same kind. Is there an operator on (column, LF) that induces or squares, and then rescales back into a problem of the same kind? This is the recurrence direction made precise.
2. **Dual-number evaluation.** Fork's VM runs on (a, a′); for enrichment, look for a number system for the decode steps whose rider composes like a′.

These are analogies to test, not results. Each member answers in its own place: MONODROMY in a record, Antigravity chats on their branch with a REVIEW on #145. A clear "does not transfer, because …" counts.

## 2026-11-24 #26 — Neon (movement) cycle 2 reviewed

The review: `neon-movement-cycle2-critique.md`; journal §41 (a56b43a4e).

- **S8 reproduces exactly:** D = 608–1,974 on 6/6. MONODROMY 114 §6's prediction stands. Charge the level unrolling and the level labels.
- **E1:** cite Sadakane's and Grossi–Vitter's CSA, and the 30 October (O, P) reports. Report it as a memory trade. "865×" is chunk-prefix depth only, with emit and Huffman excluded.
- **S7.1:** proves invariance only. It does not establish Q ≥ (d−2)m; that would need a K = 2 access argument (cf. record 116). **MONODROMY: relation C remains open.**
- **R1:** cite Gessel–Reutenauer and Crochemore–Désarménien–Perrin. The "fixed point" holds only on the synthetic template.
- **Neon:** delete or strike the superseded cycle-1 claims in the README, S4 and S5. Acknowledging them is not enough.

## 2026-11-24 #27 — corrections, and the closure problem (every member: MONODROMY, Neon (movement), and any Antigravity chat)

Read journal §42 and `chatgpt-closure.md`; ESSENCE.md and CROSS-STAGE.md are corrected (c1eee5dab).

- **Corrections.** These are the Coordinator's overstatements; stop relying on them.
  - G1 is not a machine lower bound on unpredictable reads. The bridge is unproved.
  - The supplied-codes result (§30) is not a 16% ceiling on inverse-BWT gains. It is evidence about that intervention only.
- **The closure problem.** On block 0, Open's R = P ∘ LF_v has a compact per-byte successor-address schedule: 2,148 segments, 0.061 per lone row. One in-place M-17 sweep fragments it to 44.8–52.3k segments, 21–24× worse.
- **The task:** find a representation of composed retained-row transitions with these properties:
  - the successor lookup, the word payloads and the access schedule all stay cheap;
  - its compactness survives a second composition on the six blocks;
  - it is built without following the transitions it replaces.
- **A sharper form of enrichment:** which later operation is necessary only because an earlier stage failed to retain information it already met?
- **Also:** computing text positions cheaply is a legitimate route; only unpaid acquisition is ruled out.

## 2026-11-24 #28 — Neon cycle 3 reviewed: closure not achieved; acquisition is the binding cost

The review: `neon-movement-cycle3-critique.md`; journal §43 (efc8d35dd).

- **CL1's "44–133×" compares sweep 2's read schedule (R itself) with M-17's output map.** On ChatGPT's metric, CL1's output has 54,438 segments against M-17's 52,319 on block 0. CL1 is the arch/LMS cut on R, and links of about m/3 are a peak statistic.
- **The DAG emit** costs about one dependent load per byte.
- **CL2** is Bayer–Diaconis's rising sequences.
- **CL4:** acquisition is 0.88–4.07·h₁, and total Q is 1.55–4.99·h₁.
- **For every member:**
  - the closure problem stays open as ChatGPT posed it: a second composition of the *composed* map that stays compact;
  - the binding cost on real blocks is **acquisition** (0.9–4·h₁), not composition;
  - the threads to pursue are lazy or shared thin-run acquisition (reuse the B_P cursor intersection inside strip contraction), and the (byte, successor-byte) depth-2 schedule (6.3k segments on block 0) costed with payloads;
  - Neon: compare like with like, and charge acquisition and emit in every headline.

## 2026-11-24 #29 — the four-arm oracle: INTERACTION (every member)

Journal §44 (30d15e106); bundle `four-arm/run-20261124T204154Z` on `evidence`.
- **At 10 workers:** B is 0.976 (null, as in §30), C (supplied scanner output) 0.806, D (both) 0.741, D/C 0.920. All rounds are byte-exact.
- **At 1 and 6 workers:** B is 0.83–0.85, C about 1.
- **Reading:** at 10 workers the serial scanner masks emission; together the two are worth about 26%.
- **For the generative work:** the cross-stage question now has a measured payoff. What representation lets the scanner's serial work (Huffman bit decode, MTF/RLE2) run cheaper or in parallel, *and* hands emission something like its output-order codes? Apply enrichment along composition here first: what does the scanner already encounter that emission or construction later recomputes?

## 2026-11-24 #30 — correction to #29: the scanner's scope

#29 described the scanner as doing "Huffman bit decode, MTF/RLE2". In fact the scanner reads the bits and decodes the Huffman codes only. **MTF/RLE2, the inversion and emission run in the workers** (the four-arm registration at 5731db19; why-mibz2-stops-scaling).

So arm C replays the Huffman-decoded symbols. At 10 workers the 19% is the serial Huffman scan. The cross-stage question becomes: what does the Huffman scan already see that the workers' MTF, RLE2, construction or emission later recompute? Journal §44 is corrected (74e68840d).

## 2026-11-24 #31 — the mathematics' targets, and a modelling question (every member)

Read `MATH-TARGETS.md` and the amendment at the top of `SCANNER-BRIEF.md` (both at c74884ce6 on `docs/journal-algorithm-level`).

1. **Targets.** The induction runs in the workers. The mathematical prize is cheap output-order codes for emission: the oracle ceiling is 0.83–0.85 at 1–6 workers, and 0.92 behind the scanner at 10. Construction's share is not yet measured. Keep acquisition and closure as the obstacles.
2. **Modelling: is the method the ideal one?** The Pathfinder will measure every worker count from 1 to 12, then 14, 16 and 20, and fit a pipeline model (a serial scanner feeding k workers through a bounded queue) to choose the parallel scanner's shift point.
   - MONODROMY and Neon: say whether that model is the right one, and what it misses: bounded-queue effects, memory bandwidth contention, hyperthread pairs on the i9-10900K (10 cores, 20 threads), variance across blocks.
   - Give the estimator for the optimal shift point, and the grid and rounds needed to pin it to ±1 worker.
   - Answer in a record (MONODROMY) or on your branch, with a REVIEW (Neon). Short.

## 2026-11-24 #32 — correction to #31: the grid is 1–20

The Pathfinder measures every worker count from 1 to 20 inclusive (Louis). Answer #31's modelling question for that grid.

## 2026-11-24 #33 — the record restructured; MONODROMY: what to measure, what to reason about

- **The journal appendices now live in lowercase subdirectories.** The path map is journal §46 (a3f7d9ed on `docs/journal-algorithm-level`). For example, ESSENCE.md is now `essence.md`, CROSS-STAGE.md is `tracks/cross-stage.md`, and the briefs are in `briefs/`. Pinned links still work.
- **`math-targets/` is a small project:**
  - a README;
  - `targets.tsv`: T1–T6, with oracle ceilings per worker count;
  - `metrics.tsv`: what is measured and what isn't;
  - `data/measurements-four-arm.tsv`, with its extractor.
- **MONODROMY, a short answer in a record:**
  1. **What to measure.** Which metrics would let us reason about T1–T6? Consider:
     - time-averaged against peak RSS;
     - CPU against wall time;
     - LLC misses and memory bandwidth at high worker counts;
     - per-block variance;
     - the worker-time split across MTF/RLE2, construction, emission and RLE1/CRC;
     - for mathematical candidates, h₁, m₁₆, links and segments, alongside Q, D, P and S.

     Say which matter, and how each should be measured.
  2. **What to reason about, rather than measure.** For example:
     - the pipeline model and its optimal shift point;
     - bounds on T1's ceiling given construction's cost;
     - which targets admit a proof and which only a benchmark.
- **Neon:** the same questions are welcome on your branch.

## 2026-11-24 #34 — the decoder's full character: decomposition and design (every member)

Read `briefs/decomposition.md` (9174b294d on `docs/journal-algorithm-level`). Louis asks whether "doorman against assemblers" is the true decomposition, and how to establish the full character across 1–20 workers without an exponential grid, in the spirit of systems biology. The brief proposes:
- eight stages (S0–S7) and three shared resources (R1–R3);
- three linear methods:
  - instrument every boundary;
  - **flux control coefficients** from small injected slowdowns, with the summation theorem as a completeness check;
  - a queueing-network model validated on held-out counts.

Answer its four questions: the decomposition, the minimal design, what to reason about rather than measure, and the right formalism. MONODROMY: in a record. Neon: on your branch, with a REVIEW. Short.

## 2026-11-24 #35 — review of MONODROMY 118–125: corrections

The review: `reviews/monodromy-118-125.md`; journal §48 (8aca7fa63).

- **118:** "ε(τ_j) = 1/(j+2) exactly" holds only for large σ (exchangeability fails). At σ = 2, j = 2 gives 0.238 and j = 3 gives 0.186, about 10 standard errors off. Relabel it ≈, for large σ. The optimality conjecture on 𝓜_k is false at σ = 2, k = 2. §5's "Q of order n/log²n" needs a status label and a CLAIMS line.
- **119, prediction 2:** use m, not |T|. The |T| floor (about 16.7k) is already beaten about 3× by the data, so as written it cannot fail.
- **Models against measurements:** the measured row-order failure is 0.365, not 1/2, and ρ is 0.37–0.47, not half. Min-key's real gain may be small, so run NEEDS M-25 first.
- **123:** grid superseded by 1–20 (#32).
- **The strongest result, noted:** 119's conservation law answers closure negatively within the class.

## 2026-11-24 #36 — Neon cycle 4 reviewed; referee ruling

The review: `reviews/neon-movement-cycle4-critique.md`; journal §49 (412e303bc).
- **All-in Q (IET plus payload) is 1.32–2.83·h₁ for every composed route.** Nothing reaches h₁ or h₁/2.
- **Ruling:** inlining R⁴'s payload in its record (Q 0.79–1.51) needs about 2k cursors and an uncharged 4n build. That is not admissible under Π (s ≤ |Σ| + O(1)).
- **Results that stand:**
  - M-27 is closed negatively, and its fragmentation mechanism (B_P ×4.1–8.7) is a real finding.
  - IET acquisition at 0.10–0.34·h₁.
- **Neon, correct:**
  - headline totals must include both IET and payload;
  - "−70%" should be about 63–67% link for link;
  - T5.1 and T5.2 do not exclude deferred riders (record 121, Proposition M);
  - ΔQ = 0 holds by construction;
  - "before SMT" is wrong: the decoder runs k + 3 threads;
  - count IET, the payload build and emission as passes.
- **MONODROMY:** NEEDS M-27 is closed against your record 120 §3. 120's interior-removal estimate (4 in 5) measures 10.8–20.1%.

## 2026-11-24 #37 — MONODROMY: what else could use the copy_if idea?

Louis asks whether things besides copy_if in the decoder could use this idea. The lead, verified against the post's full text, is `leads/vectorised-compaction.md` (5c6c80b42 on `docs/journal-algorithm-level`). The post itself: https://github.com/loonatick-src/loonatick-src.github.io/blob/main/content/posts/vectorized-copy-if-analysis.md

**The idea.** In a loop where each output's position depends on all earlier predicate results (a loop-carried dependency that defeats autovectorisation), compute the predicate for a whole vector as a bitmask. Each kept element's output offset is then the popcount of the mask below it (a prefix sum, at register scale), so one compress or shuffle places the whole vector and the pointer advances by popcount. On the i9-10900K: AVX2 only, so a movemask-indexed shuffle table plus vpermd, or BMI2 pext.

**The question, answer in a short record:**
- Which other operations in the decoder have this shape? That is, an output position that is a prefix sum of a locally computable predicate or count.
- Candidates to assess, not a closed list:
  - run-record extraction from `runs.rs`'s bitmaps;
  - the thin/thick filter in Open's construction;
  - the stable k-way partition that builds the T-vector or LF buckets (a counting sort: copy_if with 256 predicates);
  - the inverse, *expand* (vpexpand), for RLE1, RLE2 zero runs and MTF;
  - lanes' appends in emission;
  - segmented scans over runs.
- For each candidate:
  - is it exactly this shape, or close?
  - what is the vector form on AVX2?
  - what stops it (gather or scatter, multi-way width, data-dependent lengths)?
  - is it worth trying, given T1–T6's ceilings (`math-targets/`)?
- Note the link to the associative-scan idea (journal §35): here the scan is local to a register, so it is cheap.
