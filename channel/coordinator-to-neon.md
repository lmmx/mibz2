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
