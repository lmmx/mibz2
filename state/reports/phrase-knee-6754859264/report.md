# Phrase-knee: REVISE before a workstation slot

Assignment: [6754859264](https://github.com/lmmx/mibz2/pull/162#issuecomment-6754859264), responding also to proposal 6754771062.
Reviewed head: **9323525026d9dcea55cc1e7956a604241c426f9a**, PR #313.
Authority: SPEC.md, SPEC.RESEARCHER.md, MODE.md, RULES.md and PROTOCOL.md at **a4f97714cc14366984015869d9d094d89488ac57**.

**REVISE.** The scaling question is useful, but the census does not certify exact copies, the traffic comparison mixes cache levels, and §6 infers causes its measurements cannot identify. Correct those before buying the workstation slot. The unsafe prototype also needs B5 evidence; calling it an example is not an exemption. This is a research assessment, not a production qualification or a stop on the whole line.

## 1. A three-byte counterexample to the census certificate

At the reviewed head, `examples/phrase_census.rs:135–144` defines `coherent(q,t)` by constant row displacement alone. It does not check that each source/target pair remains in one cell, or emits equal bytes.

Take cyclic text **aab**. Its sorted rotations give **L=baa**, origin 0, **F=aab**, Ψ=[1,2,0], trajectory [0,1,2], cells [0,2) and [2,3).

At t=0 policy 0 records the first visit to cell 0. At t=1 it selects q=0. The displacement is 1 at both compared positions, so `coherent` returns 2. But the source is **aa** and the target **ab**. A genuine same-cell check permits only one byte. All three census policies select that same source here. Conservation of real steps plus copied bytes cannot detect this error.

This is a valid BWT, not an arbitrary permutation. The attached Python replay derives it from sorted rotations and reproduces the predicate. It does **not** execute Rust. The example is small enough to be excluded by the corpus-size filter; it disproves the copy certificate, not the numerical size of its error on the 233 selected blocks.

The same displacement/unequal-label issue was already identified in the packet-census journal associated with #280. Repair by independently checking emitted bytes and the landing state, as well as the same-cell/room invariant. Include adversarial small BWTs, overlaps, periodic inputs and boundary crossings; then rerun the affected census and preserve the original beside the correction.

There is a second mismatch. Policy 1 chooses the **largest future coherent length** of first/latest visits, whereas its doc comment and the prototype choose the nearer row. Policy 2 likewise chooses between candidate sides using future length. These are oracle selections, not the stated online policy. Either label their corrected counts as oracle simulations or count the actual implementable selection. They are not a proven optimum over all source policies.

Consequently **0.0777–0.0868 events/row and the copied-byte fractions are not accepted as certified online-copy evidence**. I have not calculated their corrected Wikidata values.

## 2. What survives, and what is new

The same-cell induction in §2 is sound: within a common translation cell the labels agree and displacement persists. The cyclic phrase histogram is a separate calculation, unaffected directly by the defective `lz` predicate. Its interpretation needs the assumed trajectory coverage: `landings.len()==cells` is not a general theorem for periodic cyclic BWT inputs.

The histogram's sources may be **later** in the text, as §2 correctly says. The proposal's “already walked” paraphrase is not valid for that histogram. It cannot turn the 69.2% long-phrase share directly into an online-copy guarantee.

The PR gives rounded tables and abbreviated digests, not the complete 233 per-block records in the changed files inspected. I cannot independently authenticate the reported size-biased mean 484.4 or exact shares from those tables. Archive the records, full digest, input identity and aggregation command.

Prior-art comparison:
- **Precopy #36:** earlier-text copying to avoid random table accesses already exists. Its local forward-chain detector and its measured cost differ from this room-state kernel. The original [Cache Friendly Burrows-Wheeler Inversion paper](https://www.cs.helsinki.fi/u/tpkarkka/publications/ccp2011.pdf), Kärkkäinen and Puglisi, already describes the copying principle. This is not a priority claim for that principle.
- **Move structure #42:** an O(r) translation-cell representation and the single-chain versus laned comparison already exist locally. A smaller table did not by itself beat the production room walk.
- **Packet census #280:** tandem coverage was already an inadequate universal ceiling; more general occurrence reuse was explicitly open. The new weighted phrase distribution and concrete per-position room-state implementation could add useful evidence, once validated.

The prototype checks room rather than mere displacement. The census counterexample therefore does **not** establish a wrong-output bug in `phrase_walk`. Its reported 431-block byte comparison is useful but not independently replayed here.

Two claimed necessities should become bounded observations: a roughly 6% real-step floor is a property of the restricted source/cell schemes, not “nothing earlier resembles” those rows; five packet policies do not prove that every exact copy design needs a per-position state array. Likewise the first-split census caps words at 64, so “no word passes 64” is not an independently measured natural ceiling.

## 3. The traffic model and §6's decisions

The production figure 0.51 lines/byte is a **DRAM-level** estimate. The cited 1.37 is misses beyond L2, before the 36% L3-miss factor: 1.37×0.36≈0.493. Comparing the phrase estimate 0.45 to 1.37 yields “one third,” but comparing 0.45 to 0.51 yields about **0.88**. Neither comparison establishes a measured phrase DRAM saving.

Moreover, 0.115×(2–3)+10/64 gives about **0.386–0.501**, with assumptions about reuse, write allocation, memo traffic and cache-line utilisation. It is a model, not a worst-case bound. A traffic identity does not establish bandwidth saturation. The model needs a common cache level and scope before it predicts a 3× advantage.

§6 can measure per-thread kernel cost inflation and whole-pass throughput for **N unpinned replicas, each scanning all 431 predecoded columns**. That is useful. It cannot conclude from relative inflation alone that the state stream is as contended as the T-vector, or that a residual gap is the unoverlapped chain. Frequency, scheduling, allocation, cache residency and startup/tail effects remain possible causes.

Replace those causal conclusions with observations and conditional next hypotheses. Give the decision branches a priority: ratio≥1 and phrase inflation≥production inflation can both hold. Fixed ascending N also confounds N with within-round time/heat. Either prospectively balance that order or retain the limitation explicitly; no post-result exclusion or repeat-to-pass. There is no start barrier, so nominal N is not proof of sustained N-way occupancy.

Keep all five round records and their paired ratios visible. A new counter suite is not a prerequisite to the narrow descriptive question, but timing alone must not be sold as identifying the memory mechanism.

## 4. Readiness

The freeze file names **26416d4ac35271e93c7673dd7fca04ef4e4adb2b**. Comparison with the reviewed head found only the added freeze file; the measured content is consistent. The assignment's abbreviated freeze spelling is not the authoritative SHA.

The reporter's three existing tests pass by direct invocation. Independent fixtures show that it also exits zero for:
- NaN wall time, producing NaN throughput and ratio;
- negative build time, producing negative ns/B;
- incorrect positive byte counts, producing arbitrary normalisation.

The current producer uses unsigned integers, so these are admission failures, not a claim that the native producer normally emits NaN. They matter because reporter success becomes the recipe's VALID outcome. Require exact integer domains, positive bytes/wall time, nonnegative component times, and expected bytes from the checked corpus times N, alongside the complete grid. Preserve positive-control and missing/duplicate tests. Add bounded completion handling and exercise recipe refusal, child failure, reporter failure and bundle paths in an untimed dry run.

The prototype expressly lacks a written safety argument, Miri, fuzzing, mutation tests and independent safety review. The corpus check alone does not meet pinned RULES B5. Supply that evidence, or use a safe diagnostic implementation whose question and predictions are registered accordingly. I found no concrete UB here and do not claim a completed safety review.

CI observed at this exact head: 19 successful, two skipped, one bench check in progress. No green-CI or successful dry-run claim is made. No native build, corpus run or timing was performed in this review; Rust was unavailable in this environment.

## 5. Value and disposition

Expected: fewer dependent events might offset stream writes and the unoverlapped chain under pressure. Observed: only reported local slowdown (4.72/2.71≈1.74×), corpus agreement, and an unsound census certificate; no workstation result. **No production gap is demonstrated closed.**

Learned: the room invariant matters, oracle source selection is distinct from the prototype, and the proposed DRAM advantage is not derived consistently. The next highest-value work is the cheap census correction, evidence archive and safety/readiness closure, ahead of a workstation run or laned implementation.

After those repairs, one frozen descriptive sweep has reasonable information value: it can tell whether this concrete kernel's throughput disadvantage narrows or reverses on the target. It cannot select lanes versus compact state by causal proof, nor qualify adoption. Keep the revised question narrow, charge build and walk together, and compare its value with pending production work before queuing.

My authenticated pin still reserves workload/thread-grid and repeat-policy changes under C6. This scientific recommendation grants none of that authority; reconcile the applicable approval or task re-pin before dispatch. No new role, queue entry, ready tag, merge or implementation is authorised here.

Replay: place `probe.py` beside `phrase_knee.py` and `test_phrase_knee.py` from the reviewed head; run `python3 probe.py`. Results are in `probe-result.json`. Routing audit since the previous Researcher publication found no violations; REVIEW routes to #145. Migration head and manifest matched before the receipt. All work remains on the Researcher's state branch.
