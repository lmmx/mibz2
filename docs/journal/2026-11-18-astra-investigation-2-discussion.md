# Astra's second investigation: the two inductions and the real blocks (18 November 2026–)

A working record of the second Codex (Astra 6.2) investigation in mibz2. It follows the first
(docs/journal/2026-11-18-astra-investigation-1.md). It was kept locally while the investigation
was in flight, so as not to steer it, and committed once Astra had finished. Text from Astra and
ChatGPT is as Louis pasted it.

**Reader's note.** This is a discussion record, not a research result. Astra's own journal and proof
are the primary record: docs/journal/2026-11-18-astra-investigation-2.md and
docs/journal/2026-11-18-open-return/ (draft PR #565). The interpretations here, by ChatGPT, the
coordinator and Louis, are commentary. Some are speculative, and none is a verified result. Where
they differ from Astra's journal, its proof or the independent reviews, those take precedence.
§§5 and 6 have corrections; in particular, the 11.0 B per output byte figure is 0.5.3's.
The coordinator's replies are given in substance.

**Context at the start:**
- MONODROMY's REVIEW 6976532126 and record 55 (4f4d8276), and the Researcher's REVIEW 6976571989,
  verify the adjacent-rank-gap induction.
- Record 55 relates its ceiling division to Zorich acceleration, its gap paths to saddle
  connections (record 30), and its certificate to the 15 November equal-words result. It finds
  record 54's conjecture supported: keeping the lengths removes the n/R factor.
- The frozen Wikidata block has 51,184 runs in 899,931 rows (r/n ≈ 0.057).
- Production is 0.5.5, with wide admission (#559). DRAM is about 8.05 B per output byte at 7
  workers.

---

## 1. The prompt Louis sent Astra

> Thank you for this investigation. The adjacent-rank construction is a substantial mathematical
> result, and I particularly appreciate the progression through phrase decompositions, extension
> graphs, context refinement and induced return systems.
>
> Both independent reviews have now verified your main mathematical claims, including the
> adjacent-rank gap argument, the interval bound and the equal-spelling certificate.
>
> MONODROMY has also examined how your construction relates to the Rauzy–Veech-style induction
> used by Cells. Please read its record 55 and the independent reviews on #145 alongside your own
> work.
>
> The comparison is interesting. Both constructions induce on the LF permutation and compose
> return words, but they choose different sections and expose different structure. MONODROMY
> relates your accelerated steps to Zorich induction, your gap paths to the saddle connections
> studied in its earlier work, and your equal-spelling certificate to its previous observations
> about neighbouring words. Your construction also establishes a halving property and a bound on
> the size of every induced representation.
>
> I'd like to understand what these two mathematical viewpoints reveal when considered together,
> and whether that understanding makes a better computation possible.
>
> There is, however, an important lesson from the experiments.
>
> Your construction is exceptionally fast on very-low-run blocks, but substantially slower than
> Cells on Wikidata. The mathematical advantage of accelerating long repeated passages appears to
> be largely absent from our principal workload.
>
> For scale, the frozen Wikidata block has 51,184 BWT runs in 899,931 rows, with (r/n≈0.057). The
> research record contains much richer measurements of its structure.
>
> I don't want us to assume that run density alone explains the problem. These real blocks have
> their own mathematical structure, and understanding that structure is part of the research.
>
> What mathematical features of these blocks actually govern the work required to reconstruct
> them? Which are intrinsic, and which arise from the representations and induction procedures
> we've chosen?
>
> Perhaps understanding the two inductions together will reveal a useful simplification. Perhaps
> it will expose a limitation they share, or point towards a different mathematical formulation
> entirely. Don't presume that a hybrid of the existing algorithms is the answer.
>
> I want the mathematics to lead the investigation, as it did when we first recognised inverse BWT
> as an interval-exchange problem. But this time, establish early whether the advantage you're
> pursuing exists in the regime of our real inputs. Counts, exact structural tests and inexpensive
> counterexamples are welcome ways to establish that, before investing in a large implementation.
>
> Don't abandon a mathematical possibility merely because its first implementation is slow.
> Equally, don't continue rescuing a construction once the underlying advantage has been shown to
> be absent from the relevant inputs.
>
> The objective remains an advance on the scale of induction: a mathematical connection that makes
> a fundamentally cheaper computation possible on the complex, real-world blocks mibz2 actually
> decodes.
>
> What does the combined mathematical picture now allow us to see that neither construction
> revealed on its own? Follow the strongest consequence far enough to establish whether it can
> become a better algorithm.
>
> The discussion of the first investigation, including the reviewers' findings and our evolving
> interpretations, is recorded in docs/journal/2026-11-18-astra-investigation-1.md.
>
> Read it for any mathematical insight that may be useful, but don't feel obliged to address every
> suggestion or reconcile every interpretation. Some were speculative or subsequently corrected.
> Your own mathematical judgement, grounded in the underlying problem and evidence, should
> determine which connections are worth pursuing.

---

## 2. Astra's first updates

> I'll read record 55 and the independent reviews, then use the frozen block and the other
> Wikidata blocks to test which structural differences actually predict work. I'll put the proofs,
> counterexamples and measurements in the repository journal.

> On the frozen block, the gap paths give 10,954 contiguous copy spans covering 848,747 bytes. But
> a span can depend on another span that partly depends on it. Even granting the output
> coordinates for free, whole-span copying settles only 5,063 of those bytes. I'm now testing a
> different consequence of adjacent-rank order: whether whole families of equal adjacent letters
> can be contracted directly in the compressed column, with their spellings known at construction
> time.

**Louis:** interesting; it may stop looking at copying altogether. It has read the journal but
isn't speaking in the notes' terminology (the named theorems). It seems to have kept its own
abstract, independent way of thinking.

**ChatGPT (summary):** two encouraging signs.
- **Better research judgement:** 94.3% of bytes are covered by copy spans, but only 0.56% are
  settled by whole-span copying, even with the output coordinates free. That is an obstruction
  found by counting on real data before any build. It is not an impossibility proof: partial
  copies or staging could do more.
- **A different move:** contract families of equal adjacent letters directly in the compressed
  column, with spellings known at construction. That answers investigation 1's deepest obstacle,
  that relationships were cheap to describe but costly to realise.
- **Independence kept:** it speaks in terms of its own objects, not the journal's names.
- Investigation 2 has started better than investigation 1. Don't intervene. Watch whether the new
  contraction produces spellings directly without another expensive intermediate.

**Coordinator:**
- Agreed on both signs. This is the count-before-building behaviour the prompt asked for.
- **Why whole-span copying stalls:** in LZ77-style copying, a source always precedes its target
  in text order, so dependencies can't be cyclic. Gap-path spans have no such order: a span's
  source can lie in a span that depends on it. 5,063 bytes is what has a resolved source with no
  cycle.
- **The open copying question:** whether iterating (resolve, then re-check) cascades further, or
  whether cycles dominate. Astra's "settles only" suggests it measured the closure, not one
  pass, but that is worth confirming in its journal.
- The new direction targets the right weakness: producing content and grouping together, from
  the column, rather than discovering equalities and then needing material to copy from.

---

## 3. Using a one-shot ChatGPT Pro session

**Louis (voice, to ChatGPT, summarised):**
- He has a weekly allowance of ChatGPT Pro messages and underuses it. Each one costs so much
  context that it suits a single-shot question, not a back-and-forth.
- He asks how it could help the leap without repeating Astra's work. It reads literature widely
  and writes dense formal arguments, more academic than the agents. It could open a way in, or
  push further on one already found.
- **Astra, about half an hour in:**
  - writing a journal, an `open_return` Rust example and an `open_induce` Python script
    (itertools, Counter);
  - reading a heap file and `port_cases`;
  - running the release build under `taskset -c 0` and `-c 1`, with `open-small.txt` as output.
  - Its last displayed message is still the contraction one.

**ChatGPT (summary):**
- **Its role:** an independent mathematician over the whole accumulated theory, aiming at
  synthesis, not a survey or another decoder.
- **Timing:** wait for Astra's next substantive result and its committed journal. Pro can then
  read the whole record.
- **Its draft prompt:** "What are we still failing to recognise about the mathematics of
  reconstructing these blocks?" It asks for real-block tests early, and labels what is
  established and what is conjecture.
- **A durable handoff:** definitions, hypotheses, derivations, counterexamples and consequences,
  so that Astra or MONODROMY can continue without asking Pro anything.
- **An alternative question, kept for later:** are we repeatedly solving a harder problem than
  decoding demands, by building intermediate objects richer than the output needs?

**Coordinator:**
- **Agreed:** wait for Astra's next result, and ask for a self-contained handoff. Nothing from
  Astra's investigation 2 is pushed yet: no branch on origin carries `open_return`. These local
  notes stay local, so Pro would see only committed material.
- **Three refinements:**
  1. **Put "the harder problem" first, framed as I/O complexity.**
     - **Why:** what limits the decoder is the memory controller, not instructions:
       - 75% of the IMC ceiling at 7 workers;
       - about 8.05 B of DRAM traffic per output byte;
       - against an unavoidable floor of about 1 B written plus the compressed input.
     - **The question:** where the other ~6 B go, and whether they are intrinsic.
     - **The literature:** inverse BWT is a permutation walk. In the external-memory model,
       permuting costs the sorting bound (Aggarwal–Vitter) unless the permutation has
       structure. Runs, return towers and gap paths are all ways of exhibiting that
       structure.
     - **What it gives the agents:** "find a cheaper computation" becomes a quantity with a
       lower bound to argue against. Both inductions are then measured by the traffic they
       imply, not by their elegance.
  2. **Give it data, not just links.** Pro can't run the decoder or see the frozen block, so
     "test on real inputs" is empty unless the numbers come with it. A small statistics packet
     would do:
     - the run-length histogram;
     - the context and phrase census figures from the records;
     - the per-block r/n distribution;
     - the DRAM figures.

     Louis decides whether any raw block leaves the machine.
  3. **A curated reading list,** not the whole repository. One shot can't afford to wander, so
     list the files: the dictionary, records 20, 30, 54 and 55, the investigation-1 journal,
     the IMC and knee records, and Astra's investigation-2 journal once committed.

---

## 4. Two constructors agree; requested heap halved on the frozen block

> The two constructors now agree on the frozen return map. One traces a representative of each
> bundle. The other removes the entire narrow-run wiring before induction and recovers the same
> correction in 11,215 endpoint steps. The width-16 tracing decoder also cut peak requested heap
> for the complete frozen decode from 10.35 MB with Cells to 5.44 MB. I'm checking whether that
> translates into throughput with concurrent workers and measuring construction separately from
> output.

**ChatGPT (summary):**
- **Most encouraging so far:** two different constructions give the same induced return map,
  which suggests something intrinsic.
- **Wiring removed before induction** could mean the induction runs on a smaller object from the
  start. But 11,215 endpoint steps aren't the total cost.
- **Requested heap 10.35 → 5.44 MB (−47%)** on the real block. That isn't RSS, live working set
  or DRAM traffic, though.
- **The right next checks:** throughput under concurrent workers, and construction separately
  from output. A win growing with the worker count would match the scaling aim.
- **Pro:** wait. This update strengthens the case.

**Coordinator:**
- **Agreed:** this is the first investigation-2 result in a resource on the real block.
- **The working set is what matters.** At 7 workers, about 7 × 10 MB of Cells working state sits
  in a 20 MiB L3, so the working set of the blocks in flight is what drives the knee. A 47% cut
  in that working set, if the requested-heap cut tracks it, would cut L3 misses and DRAM bytes
  per output byte. That is exactly the knee's mechanism. It could matter more at 7–10 workers
  than at one, so its single-thread speed against Cells shouldn't be the first verdict.
- **The question Astra is now asking settles it:** throughput with concurrent workers.
- **Three caveats:**
  - Peak requested heap may include allocations whose pages are never touched, so 10.35 MB may
    overstate Cells' live set.
  - What drives the knee is the bytes touched per output byte, not the bytes allocated.
  - The production comparison should be WideLazyCells (0.5.5), not Cells. Lazy letters already
    cut Cells' working state, so the baseline Astra needs to beat may be nearer 5 MB than
    10 MB.
- **What would make it adoptable:** a per-worker working set below WideLazyCells' at equal
  output, measured by the IMC counters (B per output byte) at 7–8 workers. That is the
  measurement the i9 can make once a prototype exists.
- **"Narrow-run wiring":** probably the LF links inside short runs, the part Wikidata
  (r/n ≈ 0.057) has most of and investigation 1 couldn't accelerate. If wiring can be removed
  before induction and corrected from endpoints, the cost moves from internal structure to
  boundary structure. That is the "harder problem" theme in practice.

---

## 5. Framing the Pro question: information against work, not DRAM accounting

**ChatGPT (summary):**
- **Accepts:**
  - the statistics packet, the most useful of the three suggestions, with variation across
    blocks and not just averages;
  - the curated reading list, with Pro free to go beyond it;
  - waiting for investigation 2 to be committed.
- **Declines the framing.** "Where do the other ~6 B go" would make Pro a sophisticated
  Optimiser.
  - Classical permutation lower bounds depend on the model, on what is known at the start, and
    on whether the permutation is arbitrary. They aren't automatically bounds on decoding a
    structured BWT from compressed input.
  - A new representation can change which bound is relevant.
- **Organising question instead:** the gap between mathematical information and computational
  work.
  - Smaller descriptions keep turning out expensive to recover. Are we building an unnecessarily
    hard intermediate object, or is that work intrinsic?
  - Memory traffic is the physical form the gap takes, and the test an advance must pass. It is
    not the question.
- **An instruction to add:** "Before committing to a mathematical direction, identify the
  property of our real inputs that makes it promising, and the cheapest way to discover whether
  that property is actually sufficient for a computational advantage."
- **Asks** for the 11.0 and ~8 B per output byte figures to be told apart.

**Coordinator:**
- **Accepted:** the framing, the added instruction, and the packet's precision. The DRAM figures
  become the evidence of the gap, not the question.
- **The three DRAM figures,** all at 7 lazy workers on the i9-10900K, Wikidata, unpinned:

| figure | source | B per output byte | reads + writes |
|---|---|--:|---|
| 11.0 | the knee run (knee/run-20261117T230729Z at 710c52a4, decoder identical to v0.5.3), before wide admission | 11.0 | 7.00 + 3.98 |
| 10.70 | master f71de528 (0.5.4's admission), the reference arm of #556's admit run | 10.70 | 6.76 + 3.93 |
| 8.05 | WideLazyCells (0.5.5), the candidate arm of the same admit run | 8.05 | 4.73 + 3.33 |

  - **Not the same decoder after all:** 11.0 is 0.5.3's and 10.70 is 0.5.4's, in different runs
    (corrected from the packet; the same 7-worker kernel also measured 10.97, 10.78 and 10.70 across
    runs).
  - **The release that changed things:** 8.05 is 0.5.5's.

---

## 6. The smaller and the bigger breakthrough

**ChatGPT (summary):**
- **The smaller breakthrough:** a smaller working set, so more workers share the cache.
- **The bigger breakthrough:** many short runs' internal wiring can be removed before induction,
  their combined effect fixed by far fewer endpoint relationships. It could lead past both Cells
  and the prototype.
- **Why it fits Wikidata:** it attacks the many-runs regime, not the few-runs one.
- **The baseline:** compare with WideLazyCells (0.5.5). A modest single-thread penalty is
  acceptable if throughput gains at the worker counts that matter, under the policy.
- **The question to keep:** what information about a run's interior is necessary to reconstruct
  the marked orbit, and when can boundary data replace it?
- **Next:** let Astra finish its throughput and construction measurements, verify the reduction
  independently, then decide on workstation benchmarking. Keep holding Pro.

**Coordinator:**
- **Agreed** on the order and the baseline.
- **A sharpening of the question.** Inside one run, LF is already a translation: LF(i+1) =
  LF(i) + 1. So a run's interior carries no information beyond its endpoints *for LF itself*.
  That is why move structures are O(r).
- **Where the information is:** in how the runs interleave. A run's image lands among other
  runs' images, so an orbit leaving it is split across their boundaries.
- **What "removing narrow-run wiring" plausibly removes:** the splits that short runs force on
  each other's images. Those splits are what multiply intervals in any induction.
- **So the sharper question:** which of those boundary splits does the marked orbit actually
  need resolved, and which can be settled from endpoints after induction? This is an
  interpretation to test against Astra's journal, not a claim about its construction.

---

## 7. The full-corpus comparison: beats LazyCells, loses to eager Cells

> The full-corpus comparison gives a clear boundary. The queued open decoder used about 39% less
> peak heap on the frozen decode and beat LazyCells in the paired Wikidata runs. Eager Cells was
> still faster, by the wall-time comparison at both one and four workers. Two real blocks with
> almost identical run densities also required 57,626 versus 34,775 representative letters, so
> run density misses a substantial part of the reconstruction work. I'm finishing the
> clean-checkout verification and journal record.

**Louis:** it has been benchmarking against LazyCells, not the WideLazyCells that shipped in
0.5.5. Is it still useful?

**ChatGPT (summary):**
- **Useful and encouraging:** the first investigation construction to beat a Cells variant on
  real Wikidata, with 39% less heap.
- **Not shown:** an advantage over the shipped decoder, or at 7–10 workers.
- **The density result:** blocks of equal density differing by 66% in representative letters
  point to a complexity parameter density misses. Keep it whatever the benchmark says.
- **WideLazyCells** changes admission, not necessarily the algorithm. So the comparison may
  still measure the kernel's intrinsic cost.
- **Next:** don't interrupt. After the commit, make one request: keep the results, tell the
  kernel comparison apart from the release-level one, test the 0.5.5 production path on the i9
  at the contended counts, and ask what property explains the difference in letters.

**Coordinator:**
- **Correct on WideLazyCells, and more exactly:** #559 changed only the admission (density
  < 0.15 against 0.10) and the budget (0.32n against 0.16n). The induction and writer are
  unchanged.
  - *The frozen block (r/n ≈ 0.057):* both admit it, so the kernel comparison there is nearly
    the same as against 0.5.5. Only the budget's scratch layout differs.
  - *The full corpus:* 32 of 383 sample blocks (28.8 M of 209.7 M rows) go lazy only under the
    wide admission. Which path Astra's comparison gave those blocks needs checking in its
    journal.
- **The bigger point:** at 1 and 4 workers production doesn't run LazyCells at all. Below
  `LAZY_FROM_THREADS` = 8 it runs eager Cells (the trims: eager 3/12 at 4 pinned threads). Lazy
  letters exist because they win under contention, and they lose to eager Cells at low counts
  by design.
  - *So at 1 and 4 workers,* the production baseline is eager Cells, and the open decoder loses
    to it there.
  - *At 7–8 workers* it is WideLazyCells, and that regime is untested. It needs the i9's 10
    cores and 20 MiB L3; a four-worker machine can't show contention at 7 workers.
- **What the result does show:** the open decoder beats the kernel built for the contended
  regime, measured in the uncontended one, with 39% less heap. That is a reason to test it on
  the i9, not yet a reason to expect a win.
- **The density finding agrees with production:** admission gates on density, and #559's
  census found density predicts admission poorly near the line. A better predictor of work
  would also improve admission, whatever happens to the open decoder.
- **The test, once committed:** an `ab` arm with the open kernel at 7/28 and 8/32 against
  0.5.5's own plan, unpinned on Wikidata ×4, with the IMC counters (B per output byte). It
  needs a C6 registration first. The Pathfinder or the coordinator builds it, so Astra isn't
  asked to.

---

## 8. The packet's run-length asymmetry; the journal corrected

**From the packet:**
- **The frozen block:** 68.6% of runs have length 1 but cover 3.9% of rows. The 497 runs of
  length ≥ 256 (1.0% of runs) cover 73.9% of rows.
- **The large blocks:** the median large block's r/n is 0.0623, and 82% of the 233 are below
  0.08.

**ChatGPT (summary):**
- **The asymmetry is the most consequential clue:** boundaries come from the short runs, rows
  from the long ones. It gives substance to removing narrow-run wiring.
- **Kept:** the split of work (Astra on representative-letter cost; the team on the 0.5.5
  comparison at 7–8 workers with IMC). Correct the journal. Put the distribution prominently in
  the Pro packet. The brief is unchanged.

**Coordinator:**
- **The investigation-1 journal is corrected on master** (d4642042). The 0.08–0.15 note is kept,
  with a dated correction beside it: the census quantiles and the run-length asymmetry.
- **The asymmetry is already in the packet,** in §2.1.

---

## 9. Astra's result: cutting LF open (draft PR #565 @ e6df6aee)

**Astra's summary (as pasted):**
- **The factorisation.** Retain rows S and delete their outgoing arrows. The remaining paths
  give a map P from exposed entrances to exits, with their words. The retained arrows' wiring is
  a stable sort of L[S]. So g = P ∘ LF_{L[S]}, the exact marked first return.
- **The band bound:** with H complementary strips, at most 3H + σ bands.
- **What matters on Wikidata:** how many occurrences travel side by side, and for how long.
  - *Frozen block, retaining runs of width ≤ 16:* 3,448 wide strips, 810,057 occurrences, and
    their spellings found in 50,709 letter visits.
  - *All 233 large blocks:* median wide-path area 88.7%, at a tracing cost of 5.5% of the length.
  - *Equal density, unequal work:* blocks at r/n 0.06348 and 0.06342 took 57,626 and 34,775
    visits.
- **The decoder:**
  - 6.29 MB of peak requested heap against eager Cells' 10.35 MB, with fewer instructions and
    fewer simulated cache misses on three blocks;
  - +17% wall against eager Cells at one thread and +9% at four; −7–8% against LazyCells;
  - a Xeon VM with no SMT;
  - libbz2 parity, and the CRC rejection checks pass.
- **Its journal** has the same name as the planned file for these notes:
  docs/journal/2026-11-18-astra-investigation-2.md, plus docs/journal/2026-11-18-open-return/.

**ChatGPT (summary):**
- **Stronger than investigation 1.** It is a genuine factorisation, and the stochastic
  complement (Meyer 1989) is the connection.
- **The advantage exists on real blocks:** 88.7% of the area for 5.5% of the length.
- **Not yet a production win.**
- **Next, kept separate:**
  1. independent verification;
  2. the i9 at 7–8 workers against 0.5.5;
  3. preparing Pro, which need not wait for every benchmark.
- **A candidate framing for Pro:** eliminating internal states, solving a boundary system, and
  restoring only what the marked orbit needs. Keep the brief, but include the result. Pro
  shouldn't mistake the 11,215 endpoint steps for the total work.

**Coordinator (actions):**
- **Verification:** CHECKPOINTs to MONODROMY (6978697609, its session woken) and the Researcher
  (6978698021). Seven claims each; MONODROMY also gets the theory question: relations to
  Rauzy–Veech, record 55 and record 54, and whether U is Wikidata's complexity parameter.
- **The production comparison:** QUEUED for the Pathfinder (6978711740).
  - *Arms:* the open kernel at 7/28 and 8/32 against 0.5.5's plan, unpinned, Wikidata ×4, IMC
    counters and RSS.
  - *Before data:* a C6 registration.
  - *Order:* queued after bench-recalibrate's bundle.

**Coordinator (reading):**
- **The four-worker figure:** +9% at four workers against +17% at one thread is the direction
  the contention hypothesis predicts. The relative cost falls as workers compete. A VM with 4
  vCPUs and no SMT can't show whether it crosses over at 7–8 on a 20 MiB L3.
- **The key quantity is A/U,** the height-weighted multiplicity: 810,057 / 50,709 ≈ 16 on the
  frozen block. That is "how many occurrences travel together". It is the parameter that varies
  at fixed density, a property of the return paths, not of the runs.
- **Where the overhead probably is:** the retained subcolumn's stable sort, plus the packed
  residual table of 4–8 B per retained row. The narrow rows still need an LF of their own.
  Whether the retained system can itself be cut open recursively, and what that does to U, is
  the obvious mathematical question. It is Astra's or Pro's to pursue, not the team's.
