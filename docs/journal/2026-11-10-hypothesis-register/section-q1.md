### (g) Entered the record 31 October to 6 November

**What this section is.** Every idea, hypothesis, design or measurement finding that the journal
entries dated 2026-10-31 to 2026-11-06 propose, price, test or reject, and that has no ID in the
register of 30 October. Each has a new ID, Q01 to Q76, in the order the entries are met.

**How it was read.**
- The slice is the 30 top-level entries of `docs/journal/` dated 2026-11-02 to 2026-11-06 (master
  holds none dated 31 October to 1 November), the `.md` files of their evidence directories, and
  the five front files of `2026-11-05-binding-census/`.
- Also read, because the entries cite it for ideas they only name: the frozen channel log
  `docs/journal/compact30-channel/log.md` (its 4 and 5 November sections).
- File names below drop the leading `docs/journal/2026-`. "census/x.md" is
  `11-05-binding-census/x.md`. "log" is the channel log.
- An idea that is an old ID has no row. It is in "Bears on old IDs", after "Mined".
- The old register's section 6 holds seven entries of 4 November with no ID (#157, B2 kernel B,
  B2's interval model, behaviour at more copies, memory interference, #155, the construction-only
  gate). Where this slice bears on one it is listed under "§6 (no ID)". Where this slice built a
  distinct construction from one, the construction has a Q row.
- Boundaries and machines are the register's. "CI" is a 4-vCPU runner on 2 physical cores with
  SMT. "4t" is its four-thread row. Every figure is copied from the file named. "(inferred)" marks
  my own arithmetic or reading.
- Production's whole-loop price, for comparison: build 4.92 and room walk 10.41 instructions a
  row on Wikidata (`reg2/design-e-registration.md` §4).

| ID | idea, one line | the advantage it claims, as a quantity that could be false | status 2026-11-10 | closed by: construction / advantage / n.a., with the figure | whole-loop price: counted figure, or "derived: …", or "unknown" | cheapest falsifier now: kind and what | bearing on share-nothing | sources | premise the verdict rests on | premise still true? (yes / no / moved: how) |
|---|---|---|---|---|---|---|---|---|---|---|
| Q01 | The stage table moves between builds for code a change never calls | Claim under test: a `phases` row's change measures the change. False twice: build −1.4% [−1.7, −1.2] and RLE1 +3.2% [+2.3, +4.2] for P87, whose path `phases` never calls. | open | n.a. Unexplained. RLE1 read +2.6% (+2.0, +3.2) and +3.9% (+3.1, +4.8) again in #164's two runs (8573C, 6975P-C). | n.a. (an instrument) | read of an archived bundle: `11-05-p87-production-validation/bench-run*/stages.jsonl`, first pass against later passes around the parity call. The named audit of the phase binaries was never recorded as done. | none. It qualifies every verdict read from a stage row. | 11-02-p87-production-integration.md §4; 11-05-p87-production-validation.md "The stage table" | "the untimed run of the changed code could affect later passes … a hypothesis" | yes, still unexplained. Design E's rows are ratios inside one binary and one run, so this bias does not reach them (inferred). |
| Q02 | MODE.md: a telegraphic "high-pressure" register for every agent | More "new mathematical consequence per unit of effort". | open | n.a. "attribution to MODE is uncertain". An intensity ranking was tried and removed. | n.a. | none needed | none | 11-04-compact30-mode.md §§3, 6 | "The register does not add compute" | yes |
| Q03 | C7: a FULL bit in v3's tag replaces the active-tag check and the fullness test (charter prototype 4) | 6 × 1,647,375 − k × 1,795,482 − H7 instructions a corpus pass: 6,293,286 − H7 at k = 2. Minimum 4 M on Assembly 1. | superseded | n.a. Registered, result not in the record. The line was set aside with compact30 (11-06-team-neutral.md). | counted estimate: at most 6.29 M of a constructor pass over 207,006,515 rows, 0.03 an instruction a row (inferred). | none needed | none: a constructor saving on a table production does not use | 11-04-compact30-research-charter.md §§5.4, 6.1; 11-04-compact30-mode.md §6 | "the constructor is not the decoder": construction is "about a fifth of a block's decode" | moved: construction is 15% of a Wikidata decode, 12% on text, 9% on records (11-10-by-data-class.md §1) |
| Q04 | The Optimiser's ranked cuts C8, C9 and C10 | "exact frequency × Δ formulas". What each cut is: not in the record on master. | superseded | construction: the Researcher gave a counterexample to C8's premise (n = 60, A = 31, B = 29) and found C9 and C10 lacked their distributional counters. | unknown | none needed | none | 11-04-compact30-mode.md §6 | compact30 as the production table | no: compact30 lost on CI twice (Q12, Q52) |
| Q05 | A completion model for cooperative work: the per-step maximum over workers | A better predictor of a team's completion time than a sum. | open | n.a. Named once as the Researcher's replacement model. Never tested in the record. | n.a. | read of an archived bundle: the occupancy traces of `11-06-team-size/` give per-block decode against inversion spans | none: it prices the cooperation that share-nothing removes | 11-04-compact30-mode.md §6 | none stated | n.a. |
| Q06 | compact30's constructor, v1 to v2: the scalar append path cut | The common append path 49 → 12 instructions. Four-worker build CPU 1.316× → 1.086× production's, completion 1.364× → 1.114×, against limits 1.067× and 1.02×. | superseded | construction: both captures INVALID, v1 STOP. The ratios are "descriptive figures from captures that failed their validity gates". | counted: 12 instructions on the common append path. Whole constructor against production's build: −19.5% instructions a block as Assembly 1 (Q12). | none needed | none while the table loses in the walk | 11-04-compact30-research-charter.md §§2, 8.4 | construction-only gate (charter §5) | no: superseded by the whole-decoder ledger (charter §5.6) |
| Q07 | v3: an epoch-tagged finaliser for compact30's records | ≈ 60.2 M instructions a corpus pass (60.197 M isolated). Leaf 44 instructions with `ret`. | superseded | construction: STOP under its compiler gate. The saving is "preserved as a result". Safety closeout complete: 134 Miri cases. | counted: 60.197 M over 207,006,515 rows, 0.29 an instruction a row (inferred) | none needed | none | 11-04-compact30-research-charter.md §§5.2, 7 | as Q06 | no, as Q06 |
| Q08 | Prototype 1: scanner and tile-bounds cleanup of the v2 constructor, with the U2 unchecked access | Gross 88,622,080 + 3Q instructions, Q ∈ [0, 3,450]. Minimum 60 M. | superseded | n.a. It qualified at [84,043,218, 84,060,421] (log, D008; #146 is this prototype: inferred). Never used in production. | counted: 84.0 M over 207,006,515 rows, 0.41 an instruction a row of construction (inferred) | none needed | none | charter §§5.4, 6.1; log "The first real cycle" | as Q06 | no, as Q06 |
| Q09 | Prototype 2: hot descriptor pointer arrays | 71.59 M instructions. Scalar append ≤ 10 instructions. Minimum 40 M. | superseded | n.a. It qualified at [54,550,306, 62,277,795] (log, D008; #151: inferred). Overlap with prototype 1 at least 2,393,313. | counted: 54.6 to 62.3 M a corpus pass | none needed | none | charter §§5.4, 6.1; log | as Q06 | no, as Q06 |
| Q10 | Prototype 3: SSE2 first-chunk refinement of v3's finaliser | 4 instructions × 1,647,375 leaf finishes = 6,589,500. Minimum 4 M. | superseded | n.a. Optional. Result not in the record. | derived: 6.59 M a corpus pass at most | none needed | none | charter §§5.4, 6.1 | as Q06 | no, as Q06 |
| Q11 | Prototype 3b (C6): a successor shortcut at the refill boundary | (69 − C) × 1,084,913 − g × 710,569 − H_D: 27,909,632 − H_D at C = 40, g = 5. Minimum 10 M. | superseded | n.a. "C6 remains a conditional proposal" (log). No gate 3 result in the record. | derived: 27.9 M a corpus pass at most, less H_D (unknown) | none needed | none | charter §§5.4, 6.1; log | as Q06 | no, as Q06 |
| Q12 | compact30 (Assembly 1) as production's default kernel, #155. In the old register's §6 without an ID. | "A faster production decoder" from a 2.13 B/row table. | rejected | construction, at a boundary the advantage was never claimed for: build −1,059,858 instructions a block (−19.5%), record walk +4,780,808 (+31.0%), whole +3,778,665 (+14.7%). CI 7763: +18.7%, +11.0%, +7.3% wall at 1, 2, 4 threads. | counted: walk 20,207,700 instructions on 899,971 rows, 22.45 a row, against 15,426,892 then (17.1 a row, inferred). Parity needed ≤ 18.26. | superseded by Q52's run | shrinks the table to 1.95 MB a block. Its unstaged walk is what failed. | charter §5.6; log "Production port"; census/nogo.md | "a record walk at parity … ≤ 18.26 instructions per row" | moved: master's walk fell to 10.41 a row, so the bar moved with it. See Q52. |
| Q13 | Instructions executed as the gate: the constructor ledger, then the whole-`decode_block` ledger | That Δinstructions × frequency predicts time. | conditional | advantage, in part: "Instructions predicted time in two places only": MTF's serial chain, and where the change relieved the memory system. #168 (−2.6%), #169 (−2.4%), #173 (−2.0%) bought nothing measurable. #181 ran +4% and was faster. | n.a. (a method) | none needed. Design E supplies the opposite case: a counted 3.41× and a measured 3.67×. | none. It sets how far a counted price can be trusted. | charter §§5, 5.6; 11-05-walk-round.md "instructions against time"; 11-05-staging-only.md | "parity is a screen, neither necessary nor sufficient (D016)" | moved: for a large ratio the count predicted the time (design-e-result.md: 3.41× counted, 3.67× measured). For changes of 2 to 4% it did not. |
| Q14 | A run's own interval understates its uncertainty at 1 and 2 threads | τ between runs: 0.69% at 1 thread, 1.04% at 2, 0.00% at 4, against a within-run SE of 0.42%, 0.74%, 0.90%. 2 of 9 nulls flagged. | validated | n.a. Harness-only A/A, k = 9. True A/A: 4 of 63 intervals exclude zero. The ladder that follows (a second run on another CPU model, chosen blind) was used for #207. | n.a. | none needed. "a rounds input on the PR tier" and τ in the history band are "not yet done". | none. It widens every 1-thread CI interval in this table by about 0.7%. | 11-05-bench-precision.md "The noise" | "the runners' noise is not stable from run to run" | yes |
| Q15 | The chained per-PR history is not a measurement of the cumulative change | Direct v0.2.1 → master on a 7763: −15.4% (−17.5, −13.3) at 1 thread. The 14 links chained: −4.2%. At 4 threads −25.8% against −27.0%. | validated | n.a. A bisection to before #204 on a 9V74: −8.3% (−8.7, −7.9) direct against −4.2% chained. | n.a. | CI A/B: "a second direct run of v0.2.1 against master on a non-7763 model", not yet done. | none | 11-05-bench-precision.md "The chained history" | "each link measures its PR on whatever CPU its run drew" (a likely mechanism, "not shown") | yes |
| Q16 | One-thread gains do not carry across CPU models | "Removing a store-and-reload from a serial chain gains more on Zen 3 than on later cores": #207 −5% (7763) against −2% (9V45). | open | n.a. "plausibly … not shown". | n.a. | read of an archived bundle, done here: `bench/release-scaling/2026-11-06-intel-r-core-tm-i9-10900k-cpu-3-70ghz/runs.jsonl`. See "Mined" 2. | cuts CPU a byte: decides whether a CI-validated one-thread gain exists on the i9 at all | 11-05-bench-precision.md; 11-05-mtf-two-level.md; 11-05-huffman-register.md | "serial-chain latency" differs by core | yes, and stronger on the i9 than the entry knew (Mined 2) |
| Q17 | At 4 threads on CI the decoder is CPU-capacity-bound | Utilisation 0.91 to 0.97 on every runner. 4t wall = co-run CPU ÷ 4. An observed 4t CPU cut maps about 1:1 to wall. | validated | n.a. Holds for CI's 4 threads on 2 SMT cores only. Check: #181 4t CPU −3.5% with wall −3.2% (9V45). A 1-thread gain becomes a 4t CPU gain "at an unpinned ratio: 0 to over 1". | n.a. | none needed | none: the frame is CI's. On the i9 the default plan is 68% busy at ten threads (11-10-share-nothing.md §3). | 11-05-binding-census.md "The frame"; census/accounting.md; census/nogo.md | "4 vCPUs are only about 2.3–2.5 one-thread equivalents" | yes on CI. Not the i9's regime. |
| Q18 | Each stage's co-run inflation, four pinned copies over one | Walk ×2.12 (7763), ×2.22 (9V74), ×1.46 (8370C). Build ×1.43, ×1.76, ×2.25. MTF ×1.33, ×1.23, ×1.54. Assembly ×2.97, ×3.53, ×1.77. | validated | n.a. Observations, one run a CPU model. "proxies, not ceilings": a stage's factor moves when its own code changes (#171: amplification 2.23× → 1.84× with 1 thread flat). | n.a. | CI: a `sweep.yml` `tool=phases` run that lands on an 8573C or 9V45. Neither has factors. | none: it shows that a stage's cost under sharing is not its cost alone, on CI | 11-05-binding-census.md; census/serial-stages.md; census/accounting.md | "Four copies are not four threads"; fastest of 5 passes; uncontrolled co-runners | yes |
| Q19 | The walk's AMD co-run excess: core-side per-byte work, or lines in flight | Lo 0, hi about 16 to 18% of 4t wall on the 7763 and 9V74. | open | n.a. Contested. Core-side: kernels 3.3× apart in line rate all lose ×2.12 to 2.16; the fit's lane-independent term rises ×2.7 to 3.1. Memory-side: walk MLP falls from 9.3 to 9.8 alone to about 5.5. | n.a. | CI: the pair matrix (Q29), never dispatched. Q52's result supports core-side: the core term is "about 3 to 5 times the footprint term". | none: an SMT-pair question. Ten workers on ten cores need not pair (but see Mined 9). | 11-05-binding-census.md item 1; census/walk.md FALSIFIER; census/accounting.md MISSED 1 | no PMU on any runner | yes |
| Q20 | Placement C1: stage-specialised pairs, a walker and a compute thread on each core | Modelled 4t wall: under M, AMD −10% to −19%; under C, −4.5% to +7.8%. | open | n.a. Held for the pair matrix. Design only. Its single-CPU pinning is pruned. "rejected … the moment the matrix reads C". | unknown | CI: the pair matrix | none: "engages only where threads exceed physical cores" | 11-05-corun-placement.md; census/serial-stages.md | "the walk's own footprint is a constant" (the collision's shared assumption) | no: #171 changed the walk's co-run cost with 1 thread flat |
| Q21 | Placement C2: one walk token a core | Modelled: under P, AMD −15% to 0%; under C, +1% central, Intel up to +8.5%. | open | n.a. Held for the pair matrix. Its patch `c2-token/token-probe.patch` "was not compiled". | unknown | CI: the pair matrix | none, as Q20 | 11-05-corun-placement.md | as Q20 | no, as Q20 |
| Q22 | Placement C3: one thread a core, MTF and scanner work interleaved into the walk's rounds | Fill the walk's stall time in-thread. | rejected | construction: absorption e about 0.35 to 0.55 against a break-even of 0.63 to 0.69. Zen 3 leaves 48 ROB entries beside a 208-macro-op round, against about 140 of filler. Modelled, not timed. | unknown | counting probe: its absorption probe, "kept only as a contingent measurement" | cuts CPU a byte if it worked at one thread a core, which is share-nothing's shape. It was priced for Zen 3 only. | 11-05-corun-placement.md "What the collision found" | Zen 3's window; the idle sibling belongs to the caller | moved: at ten workers on ten cores there is one thread a core by design, and the i9's window was never priced |
| Q23 | Workers wait for jobs without holding the jobs' lock (#203) | 4t wall −0.2% to −0.8%. `w.lock` 3.0 to 3.5% → ≤ 0.3%. | rejected | advantage at 4t on CI: 7763 −0.9% (−3.3, +1.6); 9V45 +0.9% (−1.3, +3.2), CPU +3.1% (+1.5, +4.7). The mechanism worked: `w.lock` 0.0085%, `i:q` 1.6%. | counted: about 900 → 333 instructions a job, "negligible" | read of an archived bundle: `11-06-team-size/occupancy-t8-k2.json` for the i9's job waits. Then an i9 recipe. | none of the four: it removes idle. "no premise at one thread a physical core … idle removed could be wall gained" (Mined 7) | 11-05-job-handoff.md; census/serial-stages.md MISSED 3 | "a worker's idle gives its sibling the whole core" | no on the i9: no sibling takes up the idle there. "At higher core counts the case is unmeasured." |
| Q24 | The Huffman symbol loop keeps the bit reader's state in registers (#204) | Scanner stage 15 to 25% faster on Intel. Wikidata 1-thread wall 1.5 to 3.5% faster. | shipped | n.a. Merged as 264ae5bb, in 0.2.4. 8370C: scanner −12.9% (−13.9, −11.9), 1-thread wall −2.7% (−4.3, −1.1). 9V74: scanner −5.0%, wall +1.0% (−0.7, +2.6). | counted: scanner 3.56 → 3.47 instructions a decoded byte | i9 recipe or a read of the release-scaling bundle (Mined 2): never read on the i9 | cuts CPU a byte in every worker once workers read their own symbols | 11-05-huffman-register.md; census/serial-stages.md | store-forward latency F "where the access is not memory-renamed" | moved: the scanner is 11% of a Wikidata decode, 16% on text, 27% on records (by-data-class §1). The i9's F is unmeasured. |
| Q25 | The build's co-run inflation tracks its alone IPC, not its store streams | "Instruction cuts in the build and RLE1 should pay more at 4 threads than at 1 thread." | open | n.a. INFERRED. RLE1, one store stream, tracks the build on three CPUs: 2.08 against 2.25, 1.86 against 1.76, 1.53 against 1.43. | n.a. | read of #312's three runs (11-09-build-window-runs-v2.md): 7763 1 thread −8.1%, 4t CPU −3.0%; 8573C −1.4% and −1.6%. They do not show "more at 4 threads" (inferred). | none | 11-05-binding-census.md item 7; census/accounting.md MISSED 2 | no PMU; one run a CPU model | moved: a later CI run found "the build is not store-bound" (R_floor 0.321, R_warm 1.010, 11-09-run-fed-stage-result.md), which agrees with the IPC reading at one copy |
| Q26 | A write-combining (staged-bucket) T-vector build; also `prefetchw` and a two-pass scatter | Build's four-copy inflation below about 1.8×. Recoverable 0 to 6% of 4t wall on the 8370C. | open | n.a. Never timed. "disfavoured" by Q25's reading. "The journals record no staged-build experiment." The build front that held it was parked when the swarms stopped (log). | unknown | counting probe: the 11-10 ledger's cache model, with the build's stores staged. Then an i9 recipe. | none shown: it would change the order of the build's stores, not their bytes. The build is 35.5% of the default plan's cold written lines (11-10-bandwidth-titration.md §2). | census/accounting.md NEXT 3 and FALSIFIER; census/serial-stages.md MISSED 2; log "wf_91b6b8b3-196" | "the build's excess tracks alone IPC … not the number of store streams" (CI, four copies) | yes on CI. No regime past the i9's knee was ever run, and there the build is the largest cold writer in the cache model. |
| Q27 | A small first chunk and first lookahead, to cut the start-up wait | 0.5 to 1.1% of CI's 4t wall: 1.9 to 3.4 ms in which every worker idles. | open | n.a. Not built. "about 0 at release scale". | n.a. | CI A/B: an arm with a 128 to 256 KiB first chunk; the claim fails if `s.input` stays above 0.5 ms | none at release scale. It biases every 0.15 s i9 burst figure by a fixed few ms (inferred). | census/supply.md claim 3 and NEXT 3 | "a per-run fixed cost … real in the 0.18–0.39 s bench" | yes. The i9's runs are 0.15 s, shorter than CI's. |
| Q28 | Idle converts to wall at 0.59 to 0.72, not 1:1 | From #180's own A/B: idle 14.93% → 2.44%, wall −8.87%, CPU +5.70% (8370C). | validated | n.a. CI's SMT pairs only. "Filling an idle sibling slows the busy one." | n.a. | none needed | none on the i9: the rate is an SMT-pair property | census/supply.md MISSED 1; census/accounting.md MISSED 3 | two threads share each core | yes on CI |
| Q29 | A sibling-pair matrix: a measured kernel beside a chosen load on its SMT sibling | Decides core-side against memory-side (Q19) and whether pairing pays (Q20, Q21). | open | n.a. Never dispatched. Its cross-exam "found a blocker: its loads could not separate C, M and P". The fixes were parked when the swarms stopped. | n.a. | CI: the instrument itself, 9 rounds | none | 11-05-binding-census.md "The decisive experiment"; 11-05-corun-placement.md; log (last section) | "`sweep.yml` today runs one copy, then N identical copies" | yes |
| Q30 | A radix-partitioned walk: queue each lane's step by T-vector region, serve a region while it is L2-resident | Random L3 and DRAM misses become L2 hits plus sequential queue traffic. | rejected | advantage, in simulation at CI's cache sizes: a control it omitted (the room walk with the same dense sorted starts) captures 74% to over 100% of its saving. Best cell −10.6% traced misses, about +30% with the omitted passes. | counted: about 31 to 35 instructions a byte against 10.4 to 10.65; about 10 memory operations a step against 5 | counting probe: the same trace simulator with a 256 KiB L2 and a 20 MiB L3 shared by ten tables. Never simulated for the i9. | none as built: it triples the walk and leaves the table at 3.6 MB | 11-05-partitioned-walk.md | "the four-copy walk [is] 63–70% a lane-independent term"; L2 of 512 KB or 1 MB | moved: its regime is the miss-bound one, and CI's is not. Its own reopening test asks for "the miss term rising to about 70% or more". Past the i9's knee the read waits are 137 cycles at 6 tables and 361 at 8 (11-07-knee-mechanism.md E2). |
| Q31 | The bench's sampler is pinned on the decoder's CPUs at 4 threads | It costs +1.2% to +2.4% of 4t wall, with ±4% intervals. The 6% "unattributed" 4t CPU is a sampler artefact, mostly worker CPU. | validated | n.a. | n.a. | none needed | none | census/supply.md MISSED 2 and 3; 11-05-binding-census.md | `bench/CHANGELOG.md` figures | yes |
| Q32 | First-touch page faults are a bench-only cost | About 0.5 to 2% of 4t CPU, from RSS 25 → 74 to 78 MB with a fresh process a run. | open | n.a. INFERRED. "A one-line test: record ru_minflt in ab.rs's child." Not recorded as done. | n.a. | read of an archived bundle: any `runs.jsonl` once `ru_minflt` is recorded. The 11-10 two-modes design reads minor faults. | none at release scale. Bears on the i9's short runs. | census/accounting.md FALSIFIER and MISSED 4 | "every bench run is a fresh process" | yes. The two-modes entry lists pages as one candidate for the i9's slow mode (H4: "minor faults ~20,000 against ~100"). |
| Q33 | For the release ETA at 4 threads, `wall_n` is the right reading | `wall_n_fixed` (2313 s) under-reads by about 21%. Release 4t ≈ 2998 × 0.98 ≈ 2,940 s. | validated | n.a. A reading of one 8573C run. | n.a. | CI: a doubled-corpus bench: 4t wall should grow by about 2× less 5 ms | none | census/supply.md claim 8 and its verdict | "about 80% of the gap is real CPU and grows with bytes" | yes |
| Q34 | Time nothing accounts for; A2, the parallel structure's own overhead | 2 to 4% of 1-thread wall unaccounted. A2: 2-thread CPU 4% above the sequential path on Intel, 7 to 9% on AMD. | open | n.a. "A2 IS UNRANKED." | n.a. | CI: an occupancy run and a `phases` four-copy on one runner in one job | cuts CPU a byte: A2 is what a plain-worker plan still pays over the sequential path | census/accounting.md claim 5, MISSED 6 | cross-instance comparisons | yes |
| Q35 | A zero-copy reader: lend the decoded block instead of copying it into the caller's buffer | Read copy 0 to 1.5% of 4t wall on CI. | open | n.a. Not built. | derived: 1.3 of 26.7 instructions a byte at one thread (11-10-borrowed-tricks.md §3, model) | counting probe: `impl BufRead` and a recount | shrinks other live memory: the copy is 17.7% of all data writes and 52% of the cold reads in the ten-thread ledger run (borrowed-tricks §3) | census/accounting.md claim 8; census/supply.md claim 2 | "only a zero-copy API removes it" | moved: priced on 11-10 as "the next serial stage after the scanner once the decoder passes about 5 GB/s" |
| Q36 | Lower the workers' priority (nice +1 to +3) so the scanner is not preempted | Recover part of the 2 to 4% mid-stream idle. | open | n.a. Named once, as a falsifier arm. Never run. | n.a. | CI A/B | none: no oversubscription with a core a worker (inferred) | census/accounting.md claim 4 FALSIFIER | "With 6-7 threads on 4 vCPUs the scanner is preempted" | no on the i9 |
| Q37 | #169: a build run's 16 entries as four SSE2 stores | −2.4% of `decode_block` instructions. | rejected | construction and power: build stage −3.1% (−3.5, −2.7) on an 8573C; 4t wall −0.5% (−4.2, +3.4) and −3.4% (−8.2, +1.6). "unresolved, not 'no gain'" (census). | counted: build 5,429,062 → 4,960,733 instructions on block 1, 6.03 → 5.51 a row (inferred) | none needed: #312 (11-09) rebuilt the build's windows and merged | cuts CPU a byte in the build | 11-05-build-run-vectors.md; census/nogo.md; census/accounting.md | "the build's scattered writes bound it" ("An inference, not measured") | no: "the build is not store-bound" (11-09-run-fed-stage-result.md) |
| Q38 | #173: the room walk's tail in deferred-flag passes | A tail step costs about 18 instructions against the busy phase's 7. −2.13% of block 1's instructions. | rejected | construction and power: 7763 4t −0.4% (−1.9, +1.2); 9V45 +2.5% (−2.6, +7.8). `pattern` −4.0% at 1 thread, −7.9% at 2. | counted: 18,725,285 → 18,325,660 instructions on block 1. Master's tail runs at about 17 a byte against the busy phase's 10.7. | counting probe, then a 1-thread CI A/B: a tail in passes "using #181's per-lane start test has never been built" | cuts CPU a byte at one thread, which is where the tail is largest | 11-05-tail-passes.md; census/nogo.md; 11-05-segment-cost.md | it "used the deferred-flag replay … which #181 later showed costs time" | no: the replay it was built on was removed by #181 |
| Q39 | The staging tile at 512 or 1024 offsets (#177, #178); a tile chosen by L1D size | Local VM: −1.9% and −2.0% (512), −0.6% and −3.2% (1024) at 1 and 4 threads. | rejected | advantage at these sizes: 512: `pattern` at 1 thread slower in both runs. 1024: +23.5% at 1 thread and +8.2% (+6.0, +10.5) at 4 on a 9V74; +2.3% (+1.3, +3.4) at 4 on an 8573C. | counted: instructions flat (+0.05%, +0.04%) | none needed. A per-CPU tile "is worth at most 1%". | none | 11-05-tile-size.md; census/nogo.md | "A 32 KiB tile is the whole of Zen 4's 32 KiB L1D" | yes. The i9's L1D is not in the entry. |
| Q40 | Split condition variables, with targeted notifies | Fewer wrong-predicate wakes. | rejected | advantage, counted before building: about one wasted wake a block, 0.2 to 0.4% of CPU. "The waits are real, not wake-driven." | n.a. | none needed. Reopen "only with many more threads than blocks in flight". | none | 11-05-split-condvars.md; census/supply.md | local VM, 1 thread a core, no SMT; window 4 × threads | yes |
| Q41 | A byte-budgeted admission window | The window's gain at lower memory. | open | n.a. Never built. "No-go for time: ≤0.1–0.3% of 4t wall … A byte budget could save memory, but buys no time." | derived: each slot reserves 2.7 MB (11-05-window.md) | counting probe: the 11-10 ledger's simulator "can price it". See Mined 5. | shrinks other live memory: twelve 0.9 MB output buffers cycle in the default plan | 11-05-window.md "What a pass does not establish"; census/supply.md; 11-05-split-condvars.md | "buys no time" on a runner with no cache or bandwidth wall | no on the i9, where live bytes are what binds |
| Q42 | #168: the walk tests its 32 lanes' start flags once an offset (deferred flags), with a replay | −2.53% of block 1's instructions. | rejected | advantage, by the reverse A/B #181: removing it cost +3.96% instructions and was faster: 1 thread −2.0% (−2.7, −1.2) on a 9V74, 4t −3.2% (−5.6, −0.7) on a 9V45. | counted: walk 9,859,793 → 9,348,635 instructions on block 1, 10.96 → 10.39 a row (inferred) | none needed | none | 11-05-deferred-flag-walk.md; 11-05-staging-only.md | "the replay's second slot of rows and the out-of-line `spec_offset` are the likely cost" (inference) | yes |
| Q43 | Fewer, longer passes in the room walk | "Half of a block's 142 chunks are a pass's short last chunk." | open | n.a. "It hasn't been tried." "untried, not a no-go, and is unpriced". | unknown | counting probe: callgrind of a pass rule that waits for more room | cuts CPU a byte, size unknown | 11-05-walk-round.md "Also found"; census/nogo.md UNKNOWNS | "the room is the smallest spare among 32 lanes" | yes |
| Q44 | The AVX2 gather walk: four `vpgatherdd` an offset. The old register's §6 lists its ancestor (B2 kernel C) without an ID. | −31.3% of block 1's instructions. | open | construction only, and locally: reopened on 5 November as a window-light walk, then parked. Local VM, no SMT: 1.204 s at 1 thread and 0.469 s at 4, against the staged walk's 1.120 and 0.416. "never on CI and never at four copies". | counted: 13,859,535 instructions for block 1's whole decode against master's 20,184,311 | CI: Gate 0 of 11-05-corun-placement.md: variant over room ≤ 0.95 at four copies. Branch `research/window-gather` was parked. | cuts CPU a byte: a third fewer instructions. Its time on the i9's core is unmeasured. | 11-05-staged-walk.md "A negative result"; 11-05-corun-placement.md | "the gathers cost more than the scalar loads they replace, and they leave no room for the per-lane prefetch" (Cascade Lake VM) | yes for that VM. Never read on a Zen or on the i9. |
| Q45 | Break the build's same-byte cursor chain: take runs of 2 to 15 equal bytes with one cursor step | 1-thread build 10 to 25% faster. 60.1% of one-row steps reload a cursor stored a row earlier. | shipped | n.a. Shipped after the slice, as #312 (fce9149d): build stage −28.4%, −8.7%, −9.2%; Wikidata 4t CPU −3.0%, −1.6%, −3.1% (7763, 8573C, 8573C). The other half, "interleaving 2-4 row ranges with cursors from mtf_checkpoints", was not built. | counted now: build 4.92 instructions a row on Wikidata, 11.04 on text | none needed | cuts CPU a byte in the build | census/serial-stages.md build claim; 11-09-build-window-runs-v2.md | "a store-forward chain on ftab[c]" | yes, in effect: the chain-breaking build is faster on every run |
| Q46 | A wider or multi-symbol Huffman table, beyond pairs | "at most about 10% more to the scanner (under 1% of 4t wall)" after Q24. | open | n.a. "never tried". The build front's "multi-symbol Huffman decode in the scanner" was parked (log). | unknown | counting probe: lookups a symbol by class, then callgrind of `read_item` | cuts CPU a byte where the scanner is large: text and records | census/serial-stages.md NOGO; 11-05-isa-probe.md; log | "the whole scanner is 5.4%"; "83% of lookups already decode a pair" (Wikidata) | no: 11%, 16% and 27% of a decode by class (by-data-class §1). The pair rate off Wikidata is not in the record. |
| Q47 | A drain-without-hash bench arm, as a causal test of the CPU-capacity model | 4t wall falls by ΔCPU ÷ 3.85. | open | n.a. Never run. | n.a. | CI A/B | none | census/supply.md NEXT 2 | Q17 | yes |
| Q48 | An occupancy extension: idle split into start, steady and tail; sibling co-idleness; support threads' CPU | Turns the supply bounds into point estimates. | open | n.a. Not recorded as built. | n.a. | CI | none | census/supply.md NEXT 1 | "No instrument records per-sibling co-idleness" | yes |
| Q49 | `decode_parallel`'s own path is not benchmarked | It copies 4 MiB `to_vec` chunks and sizes its output at `data.len() × 4`, against Wikidata's 17.7:1. | open | n.a. A finding with no measurement. | unknown | counting probe | none: the par-scan harness's `lib` arm is this path (11-10-parallel-scan.md §3) | census/supply.md UNKNOWNS | source reading at d1fd4405 | not checked at today's master |
| Q50 | A longer, or latency-scaled, prefetch distance in the walk | Hide more of the chain latency. | rejected | advantage, at 4 threads on CI only; open at 1 thread. At 4t: every AMD curve is flat from 24 to 64 lanes at four copies. Its premise "latency already hidden" fails on Zen 5 at 1 thread: 24 lanes +19.0% (+14.5, +23.6). A "late" variant lost locally. | unknown | CI: the lane sweep on an Intel runner ("No Intel lane or prefetch sweep has run") | none | census/walk.md NOGO and FALSIFIER verdict 2; 11-05-walk-round.md | "In a pointer chain each lane's lead is one step" | yes |
| Q51 | The walk's 4t penalty on CI is the SMT sibling's, not the other core's or the L3's | 2-thread spread amplification 1.04 to 1.11. Cross-core and L3 terms at most about 2 to 6% of 4t wall. | validated | n.a. CI only. | n.a. | none needed | none on CI. It says CI cannot see the i9's mechanism, which is the L3. | census/walk.md claim 6 and its verdict | 2-thread runs pin one worker a core | yes |
| Q52 | compact30 under the staged walk, RAW arrivals by C-resolve (#195) | 4t wall faster from a table of about 2.13 bytes a row (1.95 MB a block). | rejected | construction, on a core-bound regime: 9V74 +11.4% (+10.1, +12.7) at 1 thread, +4.8% (+3.2, +6.4) at 4; 7763 +13.7% and +7.7% (+5.9, +9.6). Its advantage showed: amplification −3.9% (−5.3, −2.5) and −2.7% (−4.6, −0.7), RSS 77 → 71 and 70 MB. | counted in part: ordinary step 12 instructions and 3 loads a lane against production's 8 and 2. Whole walk a row: unknown. Derived: about 10.41 + 4 = 14.4 a row (inferred). | counting probe: callgrind of `perf/c30-staged` (ba94ff5c), build and walk, complete loop. Then plain workers at 6 to 10 on the i9. | shrinks the table to 1.95 MB. See Mined 3. | 11-05-compact30-staged.md; census/walk.md | "on CI the per-byte core term dominates"; "no further compact30 production build … unless the pair matrix reopens it" | moved: the objective's regime is the i9's L3, where an inversion up to about 2.5 times production's still ties the default (share-nothing §3b). Never run there. Flush-at-wait (scheme D), "the named follow-up", was never built. |
| Q53 | compact30's split layout L: fields at 60g + 2j, headers at 60N + 4g | A layout control at the same footprint. | rejected | advantage: `c30split` alone +14.1% (8573C) and +78.3% (9V45); four copies +36% to +46% and +30% to +77%. | counted: 20 instructions, 3 loads, 2 prefetches a step | none needed | none | 11-05-compact30-staged.md | "as its 20-instruction step predicts" | yes |
| Q54 | The compact table's gain is an L2-capacity cliff | Walk only, `c30` alone: −9.4% (−12.2, −6.5) on an 8573C (2 MB L2); +21.4% (+20.0, +22.8) on a 9V45 (1 MB L2). At four copies the gain is gone. | validated | n.a. Two walk-only sweeps, diagnostic. Inference drawn from it: residency at four copies needs "about 1 byte a row on the 8573C, and about 0.5 byte on Zen 4/5 … out of reach at any step cost the walk can bear". | n.a. | i9 recipe: plain workers on `lbzip2 -2` input (share-nothing §4.2) prices the L3 version of the same cliff | shrinks the table: it shows residency, not compactness, is what pays, at L2 scale | 11-05-compact30-staged.md "The walk-only sweeps" | per-core L2 shared by two SMT walks | moved: share-nothing asks for L3 residency of ten tables (14.4 MB live), not L2 residency of two |
| Q55 | Dispatch to compact30 where the table fits the per-core L2 at one thread a core | The 8573C's −9.4% walk gain, where L2 is 2 MB. | conditional | n.a. Parked. "No whole-decoder run landed on an 8573C." "The i9 (256 KB L2) and the AMD runners never qualify." | unknown: build cost on that model unmeasured | CI A/B that lands on an 8573C | none on the i9 | 11-05-compact30-staged.md "Parked, not pursued" | "only if wikidata-pq's release target is such a machine" | yes |
| Q56 | C21: bounded whole-block phase pairing among ready blocks | Siblings' costly phases coincide less, with four end-to-end workers kept. | open | n.a. Held, "not modelled here". It arrived after the collision. The ACK said it would join the collision "and it never did" (log). | unknown | CI: the pair matrix | none | 11-05-corun-placement.md "A fourth rival"; log | "a symmetric pair benefit, and enough predictable job diversity" | n.a. |
| Q57 | A one-store-a-step room walk: stage the next row as a u32 in the tile and read it back | Halves the step's store-queue entries. AMD 0 to −20% of 4t wall (central −3% to −8%). | open | n.a. Branch `research/window-onestore`, parked with the swarms. No Gate 0 result. | unknown | CI: Gate 0, a four-copy walk-only sweep row | none at ten workers on ten cores | 11-05-corun-placement.md "The third possibility" | "the ROB and the store queue are statically partitioned once both threads run" (Zen 3, 4) | yes on CI; an SMT lever only |
| Q58 | The CRC folded into the output as it grows (`crc::Running`), apart from the fusion | The CRC's re-read of m bytes, while they are in cache. | open | n.a. "Not built: Astra's CRC-on-output-alone arm and the assembly-to-RLE1-alone arm." | unknown | counting probe: the ledger's cache model | shrinks other live memory: one re-read of every decoded byte a worker | 11-05-fused-postwalk.md "The decision rule" | "neither separates without new code" | yes |
| Q59 | The `Step` refactor of the contiguous RLE1 kernel, alone | −168,884 instructions on Wikidata block 1; −507,439 on `runs`. | open | n.a. CI, 7763: −3.5%, −1.3%, +1.8% at 1, 2, 4 threads, "all within noise". It "may become its own candidate, under its own registration". None was made. | counted: the figures at left | CI A/B at 1 thread (resolves ±0.5 to 1%) | cuts CPU a byte in RLE1 (7% of a text decode with the CRC, by-data-class §1) | 11-05-fused-postwalk.md | none | n.a. |
| Q60 | ISA-level (x86-64-v3) clones for the scalar stages | 5% of a stage, or 2% of 1-thread CPU. | rejected | advantage, by its registered rule, on one 7763 run: build −4.5% (−5.1, −3.8), scanner −2.4% (−2.8, −2.1), whole 1 thread +1.0% (+0.0, +1.9). | n.a. | none needed. A local retest on 11-10 was null (borrowed-tricks §1). | none | 11-05-isa-probe.md | "No stage reaches 5%" | yes |
| Q61 | MTF's stage is fragile to codegen | +19.3% (+18.8, +19.8) under a global x86-64-v3 target, with its AVX2 kernel chosen at run time on both arms. | open | n.a. "a lead for the MTF work". See Mined 11. | n.a. | read: #207 rewrote the kernel afterwards (merged 2026-11-05 22:44 UTC, `git log`). A re-run of the probe on today's MTF would say whether the fragility survived. | none | 11-05-isa-probe.md | the pre-#207 list in memory | moved: #207 put the list's first 32 bytes in registers. Not re-tested on the new kernel as a stage row. |
| Q62 | #172: MTF's list holds the bytes, not their indices | One load off the serial chain. −131,727 instructions on block 1. | shipped | n.a. Merged as 0892322c, in 0.2.4. 7763: 4t CPU −2.2% (−3.5, −0.8). On top of #171, not shown beyond noise (−0.4%). | counted: about 0.6% of decode instructions | none needed | cuts CPU a byte in MTF | 11-05-mtf-byte-list.md | "at the edge of the bench's power" | yes |
| Q63 | #170: MTF's short runs as two 32-byte stores | −326,433 instructions on block 1 (−1.62%). | shipped | n.a. Merged as e7c30a86, in 0.2.4. 9V74: 1 thread −1.4% (−2.7, −0.1). 7763: −2.1% (−3.0, −1.1), 4t −1.9% (−3.6, −0.2). MTF stage −4.4%, −9.7%. | counted: the figure at left. Variants: one store for runs ≤ 32, −236,165; a store loop for runs ≤ 256, −29,173. | none needed | cuts CPU a byte in MTF. In the ledger `memset` (MTF's runs) is 12.8% of cold writes. | 11-05-mtf-short-runs.md | "MTF is one serial chain per block, so its instructions are on the critical path" | yes for MTF as built. The run emitter (11-10) would remove these stores. |
| Q64 | Cross-chain lockstep: many short segments from dense, row-sorted starts | T-vector misses a byte 0.859 → 0.584 alone at 512 KB; 0.926 → 0.756 for an SMT pair. | rejected | advantage at CI's miss cost: the lean path costs 33 to 64 cycles a segment against a gate of about 10, and reuse forces one segment per 8 bytes or fewer. At K = 8: 48 to 107% slower than the four-copy walk. | counted: 25.37 instructions a byte at K = 8 against master's 10.74; fit 10.9 + 108 a segment | none needed on CI. Unpriced where a miss costs DRAM latency. | none as built | 11-05-partitioned-walk.md "Parked"; 11-05-segment-cost.md | a miss in a dependent chain costs "about 24–26 cycles … half an L3 latency" | moved: past the i9's knee a read waits 137 to 361 cycles (knee-mechanism E2). The gate's arithmetic was done for CI only. |
| Q65 | What a walk segment costs | 262 to 338 cycles net; about 457 instructions steady. Search 138, sort 108, two `memcpy` about 82. | validated | n.a. Counted; the cycles are INFERRED and fitted to CI's net. Also: the sweep segments 198 blocks under 64 Ki rows that production walks plainly, "roughly a tenth" of its 512 → 2048 difference. | counted: 457 instructions a segment | none needed | none: but D's lanes (share-nothing §3a) would pay for their ends in this price class | 11-05-segment-cost.md "Why a segment costs" | "agreement with CI is by construction" (log) | yes |
| Q66 | A lean per-segment path: starts set by the build, one arena, an 8-byte record, the join as a second-level walk | At most about 10 cycles a segment. | rejected | construction and advantage: 108 instructions a segment and 1.9 join misses: 33 to 64 cycles. "An idealised floor of about 30–35 Ir a segment still gives 12–25 cycles." | counted: 108 instructions a segment (walk 37.3, join 70.6) | none needed | none | 11-05-segment-cost.md "The lean path, and the gate" | "the join is a random permutation of n/K pieces" | yes |
| Q67 | The tail's lost memory-level parallelism, and a CI tail clock to price it | E ≈ q·lat·(H_M − 1)/32: 0.25 to 0.27 c/B alone, 5.8 to 9.3% of the walk (9V74, 9V45); 2.6 to 3.5% at four copies. | open | n.a. INFERRED from a fit. "Zero stays the demonstrated lower bound." The tail clock was specified and never run. | counted: the tail is 5.7 to 6.0% of a block at about 17 instructions a byte | CI: the tail clock (busy and tail clocks a byte, tail bytes binned by live lanes) | cuts CPU a byte at one thread a core: "tail levers move the 1-thread and 2-thread rows" (log) | 11-05-segment-cost.md "What the anatomy opens"; log | exponential segment lengths (residuals "memoryless: 0.96–1.04× the mean") | yes |
| Q68 | Three tail constructions: soft starts used only in the tail, cheap segments in master's walk, cross-block continuity | Keep the lanes full in the tail "for less than a segment costs". | open | n.a. A workflow of three rivals and a collision; parked when the swarms stopped. No result in the record. | unknown | counting probe, then the tail clock | cuts CPU a byte, at most Q67's pool | log "New front … the walk's tail" | Q67 | yes |
| Q69 | C22: launch segments longest first | An oracle order cuts the tail from q 6 to 7.6% to 0.04 to 0.16%, E to about 0.001 c/B. | rejected | advantage for any predictor: "No start-local feature predicts length (\|ρ\| ≤ 0.10)." The oracle figure stands as "the ceiling a splitting construction aims at". | n.a. | none needed | none | log "C22, length-aware launch order" | six held-out full blocks | yes |
| Q70 | #171: the walk's bytes staged in one 8 KiB tile, copied out in bursts | −5.61% of block 1's instructions with #168. | shipped | n.a. Merged as 6b337819, in 0.2.4. 4t wall −14.4% and −13.9% (9V45), −10.4% and −13.0% (7763), −5.2% (8573C). 1 thread: −0.1% and +0.4% on the 9V45, −7.3% and −12.6% on the 7763. | counted: 7 instructions a step; now 10.41 a row for the whole room walk (after #181) | i9: never read there by itself. See Mined 2. | cuts CPU a byte: 32 store streams become one L1 tile. Flush variants (fixed-size, SIMD, non-temporal) were "neutral". | 11-05-staged-walk.md; 11-05-walk-round.md | "the walk is bound by the number of misses a core can keep in flight" ("an inference") | moved: the census's falsifier found the 4-copy walk "tracking per-byte work" on AMD. The mechanism is contested (Q19). |
| Q71 | Shared `CARGO_TARGET_DIR` across worktrees reuses stale binaries | Every tile-size run "reported 256 until `src/bwt.rs` was touched". | validated | n.a. A hazard, seen once. | n.a. | none needed | none | 11-05-walk-round.md "Also found"; log "A measurement hazard" | cargo's fingerprinting by package and version | yes |
| Q72 | More helpers on the same four tables: 4 × 3 and 4 × 4 | 1.00 to 1.12 and 0.95 to 1.08 against 4 × 2 at 8 threads. | rejected | advantage, registered, i9, 20 rounds: 4 × 3 1.017 (1.008–1.026) for 1.26× the CPU; 4 × 4 1.003 (0.993–1.014) for 1.52×. A block decodes in 1.39, 1.38, 1.40 ms at k = 2, 3, 4. | n.a. (a plan) | none needed | none: it shows that helpers add CPU, not speed, at a fixed table count | 11-06-team-size.md "The registered rule" | 0.2.4's teams, unpinned, one process a run | moved: the default is now three pipelined teams (0.2.6). The i9's two modes (11-10-two-modes.md) are not in this bundle: its 320 walls run from 0.1434 to 0.2473 s (read here from `runs.jsonl`). |
| Q73 | Throughput follows the number of planned teams, not the thread count | Per table: 558 to 583 MB/s at 2, 461 to 466 at 3, about 320 at 4, 230 at 5, 175 at 6, 110 at 8. | validated | n.a. "These are exploratory." Each team also brings 4 window slots and about 15 MB of peak RSS, so the axis is not table bytes alone (Mined 1). | n.a. | read of an archived bundle: the older `mibz2-coopK` arms keep 2 slots a team; a grid with both separates slots from tables | shrinks the table: the whole-decoder evidence that fewer live tables pay | 11-06-team-size.md "The whole grid"; 11-07-knee-mechanism.md E1, E5 | "the capacity reading … not shown by counters: perf did not run" | moved: E1 was reworded to "planned teams … not a measured count of tables live at once" |
| Q74 | The decoder's other per-block buffers contend for the cache | "14–16 MB in the occupancy ledger at the default window … a plausible reason the knee has moved since 29 October. That is not shown." | open | n.a. Ranked last on 7 November ("evidence: none"). | n.a. | counting probe: the 11-10 ledger by buffer, for `mibz2-workers` at 10 | shrinks other live memory. See Mined 1. | 11-06-team-size.md "What this means"; 11-07-knee-mechanism.md H4 | capacities, "not bytes being accessed at the same time" | moved: the simulated ledger finds "the traffic is the decoder's sequential buffers, not the T-vector's random reads" (11-10-bandwidth-titration.md) |
| Q75 | Profile-guided optimisation of the decoder | A build-level gain with no source change. | rejected | advantage, weakly and locally: the probe of 5 November was parked unreported. A local test on 11-10: Wikidata 0.974 at 1 thread and 0.981 at 4, "within 3% either way". | n.a. | none needed. It reads the MTF row first (Q61). | none | log "wf_91b6b8b3-196"; 11-10-borrowed-tricks.md §1 | "the hot loops are written against their own listings" | yes |
| Q76 | The pooled re-registration of #183 on a fresh-run 4t criterion | Pooled 4t wall about −1.4% over three CI rows. | open | n.a. Proposed twice in the census. "#183 is closed and is not reopened without the maintainer." | n.a. | see Mined 4: the question has moved from CI's 4t wall to the i9's cold writes | shrinks other live memory: as P85 in "Bears on old IDs" | 11-05-binding-census.md item 6; census/nogo.md; census/serial-stages.md | "a statistical-power case, not a failed mechanism" | moved: design and acceptance are delegated as a class since 9 November (CLAUDE.md), so the maintainer is no longer the gate (inferred) |

#### Mined

Twelve findings, ranked by what they could be worth to the share-nothing objective.

1. **Q73 and Q74, "throughput follows live tables" against "the other buffers also contend".**
   - **The verdict and its premise.** `11-06-team-size.md`: "Throughput follows the number of
     live tables, not the thread count." `11-10-share-nothing.md` builds on it: "a table under
     about 1.2 MB a block … lets ten workers each run the one-thread decoder and share nothing",
     with the caveat "This assumes the table is small enough to behave as level 1's does".
   - **What the record says against it.** The same team-size entry: "The decoder's other
     per-block buffers, 14–16 MB in the occupancy ledger at the default window, also contend for
     the cache." Every axis the record has varied moves those buffers with the table:
     - the team-size grid gives each team 4 window slots ("keep `SLOTS_PER_THREAD` a team"), and
       "Peak RSS rises about 15 MB per team" for a 3.6 MB table (`11-07-knee-mechanism.md` E5);
     - the level sweep of share-nothing §2 shrinks every per-block buffer with the block
       (inferred: a level-1 block has a ninth of the rows);
     - `11-07-knee-mechanism.md` E1 was reworded for this: "planned teams … not a measured count
       of tables live at once".
   - **Read here from an archived file.** `11-10-bandwidth-titration/sim-workers10.txt`, ten plain
     workers, 16 MiB cache model: of 11,804,206 cold written lines, `bwt::build` writes 4,444,008
     (37.65%), `memcpy` 2,729,745 (23.13%), `rle::unrle_avx2` 2,700,261 (22.88%), `memset`
     744,584 (6.31%). So 62% of a plain-worker decode's cold writes are not the table's. The
     model "cannot rank plans whose difference is live tables", because it under-counts
     concurrent random reads, so this is the write side only.
   - **What that makes possible.** The target may have to be live bytes a worker, not table
     bytes. Three cuts to the other buffers are already built or priced: P87 (shipped), #183
     (built, Mined 4) and the window (Mined 5).
   - **The cheapest falsifier.** An i9 recipe on an existing branch: plain workers at 4 to 10 on
     level-9 input with `perf/c30-staged` against master. It is the one whole-decoder build that
     changes table bytes and nothing else (Mined 3). The registered `lbzip2 -2` run changes
     everything at once and cannot separate them.
   - **Worth, derived.** If the build's cold writes fall with the table from 4 to 1.33 bytes a
     row, they fall by two thirds of 37.65%: 25% of the plain-worker decode's cold written lines
     (inferred). The other 75% stay. Whether they cost what random table misses cost cannot be
     bounded from the record. This decides whether share-nothing's 2.6 GB/s at g = 1.2 is
     reachable by a table alone.

2. **Q16, one-thread gains validated on CI are almost absent on the i9.**
   - **The verdict and its premise.** `11-05-bench-precision.md`: a direct run of v0.2.1 against
     master gives "−15.4% (−17.5, −13.3)" at 1 thread on a 7763, and "The 1-thread gains depend
     strongly on the CPU … A likely mechanism, not shown". The decoder merges of this slice
     (#157, #164, #170, #171, #172, #180, #181, #204, #207) were each admitted on CI's rows. All
     nine are in v0.2.4 (`git log v0.2.1..v0.2.4`).
   - **Read here from an archived bundle.**
     `bench/release-scaling/2026-11-06-intel-r-core-tm-i9-10900k-cpu-3-70ghz/runs.jsonl`, the
     i9, 0.2.4 against 0.2.1, 10 rounds a cell. The two middle walls of each sorted cell:

     | threads | 0.2.4, s | 0.2.1, s | ratio of the medians (inferred) |
     |--:|---|---|--:|
     | 1 | 0.5205, 0.5212 | 0.5264, 0.5305 | 0.986 |
     | 2 | 0.2954, 0.2963 | 0.3270, 0.3273 | 0.904 |
     | 4 | 0.2035, 0.2036 | 0.2390, 0.2402 | 0.850 |
     | 8 | 0.1606, 0.1607 | 0.1662, 0.1673 | 0.963 |
     | 10 | 0.1643, 0.1648 | 0.1649, 0.1654 | 0.996 |

     These are unpaired medians, one machine. At 1 thread the cells overlap: 0.5138 to 0.5406 s
     against 0.5222 to 0.5471 s.
   - **What it changes.** On the i9 one thread decodes at about 400 MB/s in both releases
     (209.7 MB in 0.52 s, inferred), which is share-nothing §3's base rate. The walk round's
     kernels did not move it by more than about 1.4%. The precedent is in the old register's §6:
     the walk unroll read 1.017 (1.006–1.028) at 1 worker on the i9 against −4.2% and −2.4% on
     CI. No journal entry that cites `release-scaling` reads this bundle at 1 thread.
   - **What that makes possible.** Every row above marked "cuts CPU a byte" on a CI one-thread
     figure (Q24, Q45, Q62, Q63, Q70, and P74 through #207) is unverified on the machine the
     objective is stated for. Either the i9's one-thread decode is bound by something those
     changes do not touch, or a gain is there and hidden by this bundle's noise.
   - **The cheapest falsifier.** A read of the two later bundles in `bench/release-scaling/`
     (0.2.5 and 0.2.6) at 1 thread, then the release-scaling recipe again (a
     `ready/bench-release-scaling` tag sits on v0.2.7).
   - **Worth.** If the cause is a fixable difference, up to CI's 15% of every worker's CPU:
     3.0 ÷ 0.85 = 3.5 GB/s in §3b's model at g = 1 (inferred). If the i9's one thread is
     latency-bound in the walk, the worth is the reverse: instruction cuts in the walk are worth
     nothing there, and only fewer misses pay. Cannot be bounded further from the record.

3. **Q52, compact30 under the staged walk: closed on CI's core term, never read on the i9's
   cache term.**
   - **The verdict and its premise.** `11-05-compact30-staged.md`: "on CI the per-byte core term
     dominates", "residency at four copies is out of reach at any step cost the walk can bear",
     and "no further compact30 production build is made unless the pair matrix reopens it". The
     premise is two SMT walks sharing one core's L2.
   - **What later evidence changes.** `11-10-share-nothing.md` §3b: "Break-even with the
     cooperative default … is near g = 2.5", for a table that keeps ten workers under the L3.
     Design E, at 2.04 bytes a row, was closed at g = 3.67 with a step of 22 to 26 instructions
     (`reg2/design-e-result.md`), with the condition "a step and skeleton counted, before any
     timing, at no more than 1.44× the room walk's instructions a row (about 15)". compact30 is
     2.13 bytes a row with a counted step of 12 instructions against production's 8, in
     production's own staged skeleton. Its whole decode cost +11.4% and +13.7% at one thread on
     CI, and its smaller table already cut amplification by 3.9% and 2.7% there.
   - **What that makes possible.** A built, byte-exact, Miri-clean table of E's footprint at
     about a third of E's price (inferred, below). Its walk a row is about 10.41 + 4 = 14.4 instructions
     (inferred, not counted), under E's reopening bar.
   - **The cheapest falsifier.** A counting probe: callgrind of `perf/c30-staged` at ba94ff5c,
     build and walk, the complete loop with the RAW resolve (ρ = 0.00708 on Wikidata). Then the
     i9 run of Mined 1.
   - **Worth, derived.** Charging the whole one-thread loss to the inversion, 0.3 + 0.7 g =
     1.114 to 1.137, so g ≈ 1.16 to 1.20 (inferred). §3b's model then gives 3.0 ÷ 1.13 =
     2.6 GB/s only if a 1.95 MB table behaves as level 1's 0.4 MB does. §2 says it will not
     fully: CPU growth from 4 to 10 workers is ×2.65 at 2.4 MB and ×1.44 at 1.2 MB, against
     ×1.10 at 0.4 MB. Scaling by those, 1.1 to 2.0 GB/s (inferred), against the default's 1.5.
     The bound straddles the default, which is why the count and the run are worth making.

4. **P85 and Q76, the fused post-walk pass (#183): rejected for CI's four-thread wall, where its
   mechanism cannot show.**
   - **The verdict and its premise.** `11-05-fused-postwalk.md`: "the post-walk traffic is not
     shown to bind the 4-thread wall on two SMT cores." The old register closed P85 unless "a
     runner or a change puts the assembly over 5% of a decode".
   - **What later evidence changes.** `11-10-bandwidth-titration.md` §1: "CI's runners have no
     bandwidth wall at 4 threads, so CI could not have shown what it is worth on the i9." Its
     ledger puts `memcpy` at 20.3% of the default plan's cold written lines.
     `11-10-share-nothing.md` §3 lists segment assembly at 15.2% of cold writes, removed by
     "RLE1 read from the segments in chain order (#183)". The fusion already measured: simulated
     L2 misses of the post-walk phase −57%, and 1-thread wall −1.1% (−1.6, −0.7), −2.5%
     (−4.1, −0.9) and −2.3% in three CI runs, "measured but not registered".
   - **What that makes possible.** The code exists (beef4e8f, +988 −43, byte-exact, Miri
     clean). It removes one pass over every row in each worker, which is other live memory in
     Mined 1's sense.
   - **The cheapest falsifier.** A counting probe: the fused head in the ledger's cache model
     at ten plain workers. Then a registration on the 1-thread wall, which CI resolves to
     ±0.5 to 1%.
   - **Worth, derived.** At most 15.2% of the default plan's cold written lines. If the i9 is
     bandwidth-bound, T = W ÷ β gives at most 1 ÷ (1 − 0.152) = 1.18× (inferred). At one
     thread, 1.1 to 2.5% of wall, measured on CI.

5. **P107 and Q41, the window: validated on CI's runners, never varied alone on the i9.**
   - **The verdict and its premise.** `11-05-window.md`: 4 blocks a thread, merged on −8.1%,
     −10.3% and −8.9% of CI's 4t wall. "Weighed for a laptop of about 16 GB, 20–55 MB is small,
     if it buys time." `census/supply.md`: "A byte budget could save memory, but buys no time."
     The premise is seven threads on 4 vCPUs, where "the reader is often not running".
   - **What later evidence changes.** `11-10-bandwidth-titration.md` §5 lists "Fewer output
     slots in flight (the window)" as "the cycling of twelve 0.9 MB buffers … unpriced: the
     simulator can price it … #180 widened it for CI". On the i9 the window does not starve
     anyone: "The scanner waits for a free slot for 38–46% of the run" (`11-06-team-size.md`).
     In Mined 2's bundle the release that carries the wider window is 0.4% faster at 10 threads
     and 3.7% at 8 (inferred), with eight other merges inside the same difference. The
     three-team policy then cut the window from 16 to 12 together with the plan: "The windows
     differ by design: they are part of the change being tested."
   - **What that makes possible.** The window was never varied alone on the i9. Part of the
     three-team gain (1.088 and 1.089) may be the four fewer slots.
   - **The cheapest falsifier.** A counting probe in the ledger's simulator at 2, 3 and 4 slots
     a team. Then one i9 recipe with `parallel_planned` at a fixed plan.
   - **Worth.** Each slot reserves 2.7 MB and a Wikidata block writes about 1.1 MB of it
     (0.9 MB of output and "about 0.2 MB of its symbols", `11-05-window.md`; inferred). Ten
     workers at the default's 4 slots each hold 40 slots, about 44 MB written against a 20 MiB
     L3 (inferred). In time, it cannot be bounded from the record.

6. **Q38 and Q67, the tail: rejected as one construction, on a replay that was later removed.**
   - **The verdict and its premise.** `11-05-tail-passes.md`: "closed as not shown (#173)", on
     7763 4t −0.4% (−1.9, +1.2) and 9V45 +2.5% (−2.6, +7.8).
   - **What later evidence changes.** `census/nogo.md`: "The tail used the deferred-flag replay
     … which #181 later showed costs time … A tail in passes using #181's per-lane start test
     has never been built." `11-05-segment-cost.md`: the tail "walks 5.7–6.0% of a block at
     about 17 Ir a byte", and its lost parallelism is 5.8 to 9.3% of the walk alone. The log:
     "tail levers move the 1-thread and 2-thread rows", and an oracle launch order cuts the tail
     to 0.04 to 0.16% of a block (Q69).
   - **What that makes possible.** A lever at one thread a core, which is each share-nothing
     worker. Three constructions for it were parked unbuilt (Q68).
   - **The cheapest falsifier.** A counting probe of a tail in passes on the per-lane start
     test, then the CI tail clock that `11-05-segment-cost.md` specifies.
   - **Worth, derived.** Instructions: (17 − 10.7) × 0.057 to 0.060 = 0.36 to 0.38 a byte, 3.5
     to 3.6% of the walk's 10.41 (inferred). Time: the fitted pool is "about 3.3–3.5%" of decoder wall
     alone on a 9V45, with zero "the demonstrated lower bound". On the i9: not in the record.

7. **Q23, the job handoff: closed where a sibling takes the idle, open where none does.**
   - **The verdict and its premise.** `11-05-job-handoff.md`: "Removing handoff idle is closed
     where threads exceed physical cores", because "a worker's idle gives its sibling the whole
     core".
   - **What the same entry and a later one say.** "It changes no premise at one thread a
     physical core … At higher core counts the case is unmeasured." `11-10-share-nothing.md` §3
     puts the i9's default at "ten threads 68% busy in the fast mode (1.08 CPU-seconds in
     0.159 s)".
   - **What that makes possible.** The branch `perf/job-handoff` is kept, Miri-clean, with
     `w.lock` at 0.0085% of worker time.
   - **The cheapest falsifier.** A read of an archived bundle: the i9 occupancy traces
     (`11-06-team-size/occupancy-t8-k2.json`, and the later pipelined bundles on `evidence`) for
     the lock and queued-idle shares.
   - **Worth.** Bounded above by the idle: "Team leaders … are idle under 3%" at four teams
     (`11-06-team-size.md`). For a plain-worker plan at ten: cannot be bounded from the record.

8. **Q46 and P71, Huffman work: the premise "the whole scanner is 5.4%" has moved.**
   - **The verdict and its premise.** The old register closed P71 on "The whole scanner is 5.4%
     of a decode", with the condition "the scanner becoming a larger share".
     `census/serial-stages.md` caps a wider table at "about 10% more to the scanner", on "83% of
     lookups already decode a pair".
   - **What later evidence changes.** `11-10-by-data-class.md` §1: the scanner is 11% of a
     Wikidata decode, 16% on text and 27% on records. §4b: `read_item` runs "at least 2.7
     instructions a cycle", so it is instruction-bound on text. The 83% is Wikidata's. The pair
     rate on text and records is not in the record. P71's condition holds.
   - **What that makes possible.** Under the parallel scan this CPU is in every worker.
   - **The cheapest falsifier.** A census: lookups a symbol, by corpus.
   - **Worth, derived.** If the census's 10% carries over: 10% × 11 to 27% = 1.1 to 2.7% of a
     decode (inferred). More if the pair rate off Wikidata is low.

9. **P94, placement on the i9: a lead of about 7% (inferred: 1,369 ÷ 1,276), deferred twice.**
   - **The verdict and its premise.** The old register: "No two team threads on SMT siblings
     … shipped (0.2.0, as a design rule). Not separately A/B'd for teams."
   - **What this slice adds.** `11-06-team-size.md`: at 4 × 2, "packed is 23% slower than
     spread", and "Pinned spread, 4 × 2 reached 1,369 MB/s, above the unpinned timed 1,276 …
     a lead, not a measurement". Its proposal 2, a pinned timed arm, was not run:
     `11-07-team-pipeline-result.md` has "H2 (placement) stays deferred until per-thread
     affinity records exist."
   - **What that makes possible.** The decoder runs unpinned. A plain-worker plan at ten runs
     twelve threads on ten cores (ten workers, the finder and the joiner), so two workers share
     a core with something. The par-scan bundle fits that: level 1 with the finder gives 2,429
     MB/s at 8 workers and 2,563 at 10 (`11-10-share-nothing.md` §3b), 1.055× for 1.25× the
     workers (inferred).
   - **The cheapest falsifier.** An i9 recipe: the par-scan harness at 8, 9 and 10 workers,
     pinned one a core with the finder and the joiner on siblings, against unpinned.
   - **Worth.** At most 1.25 ÷ 1.055 = 1.18× at ten workers if the whole shortfall were
     placement (inferred). The all-core clock and the CPU growth share it, so it cannot be
     separated from the record.

10. **P146, the i9's counters: the register says "i9 yes"; the record now says no.**
    - **The verdict and its premise.** The old register: "PMU availability (CI none; VM none;
      i9 yes)".
    - **What later evidence changes.** `11-06-team-size.md`: "perf could not count on the
      machine (`perf_event_paranoid`), so there is no counter pass."
      `11-10-bandwidth-titration.md`: "The counters that could say are gone:
      `perf_event_paranoid` is 4 on the workstation".
    - **What that makes possible.** Nothing by itself. It removes the cheapest falsifier of
      every old ID whose next step is "counters on the i9". The setting is a fact only Louis
      has, and a change he alone can make.
    - **The cheapest falsifier.** None needed: ask.
    - **Worth.** Cannot be bounded. It prices every mechanism question on the i9 at a
      titration or a simulation instead of a counter read.

11. **Q61 and Q60, MTF's codegen fragility: a lead that a later table answers.**
    - **The verdict and its premise.** `11-05-isa-probe.md`: under x86-64-v3 MTF + RLE2 was
      "+19.3% (+18.8, +19.8) slower", "a lead for the MTF work: something in the stage's
      codegen moves by a fifth with the compilation context alone". The probe's whole-decoder
      figure, +1.0% (+0.0, +1.9) at 1 thread, carries that loss.
    - **What later evidence changes.** #207 replaced the kernel afterwards, the same day.
      `11-10-borrowed-tricks.md` §1 rebuilt the whole crate at x86-64-v3: Wikidata 1.006 at 1
      thread, text 1.012, records 0.974 (local, indicative). A stage still losing 19% would
      cost 15% × 19.3% = 2.9% of a Wikidata decode and 50% × 19.3% = 9.7% of a records decode
      (inferred from `11-10-by-data-class.md`'s shares). Neither shows.
    - **What that makes possible.** The fragility looks gone with the in-memory list. The
      ISA probe's bounds for the other stages stand: build −4.5%, scanner −2.4%.
    - **The cheapest falsifier.** One CI run of the probe's PR shape on today's master, read
      on the MTF row.
    - **Worth.** About nothing remains. It removes a caution, "read the MTF row before
      anything else", that the probe attached to every global codegen change.

12. **P33, huge pages: closed on a counter that is gone, with the setting never read.**
    - **The verdict and its premise.** The old register rejected P33 on the i9's page walks at
      "1.1%, 1.1%, 1.0% and 2.2% of cycles", with the condition "a different machine class or
      block size shows page-walk activity over about 10%".
    - **What this slice and later entries say.** `census/nogo.md` MISSED 8: the runners' THP
      setting "can be read without a PMU, from /sys/kernel/mm/transparent_hugepage/enabled in
      any CI job, before anyone asks for authority". It was put in the pair matrix's
      environment records, which were parked. `11-09-run-fed-fallback-codegen.md`: "The
      runner's THP mode was not recorded." `11-10-two-modes.md` lists huge pages as mechanism
      H4 for the i9's slow mode: "a run gets 2 MiB pages or 4 KiB ones, as the last run's
      freed blocks allow".
    - **What that makes possible.** The question is no longer page-walk cycles. It is whether
      page backing varies run to run.
    - **The cheapest falsifier.** A read in any CI job, and the same file on the workstation.
    - **Worth.** If H4 were the slow mode: the two modes are 0.159 s and 0.306 s. Otherwise
      the old bound stands. Cannot be bounded further from the record.

#### Bears on old IDs

Ideas of this slice that are already IDs, with what the slice adds. No row above repeats them.

- **P01 (pair walk, Ψ²):** `census/nogo.md` and `census/walk.md`. Its closing premise, "the walk
  is bound by instructions per step", was reversed by the walk round, and that "does not rescue"
  it: the bigram build is 7.1 to 7.6 against 2.5 c/B, and CI's 4t was +24.3%. A staged Ψ² room
  walk as a walk-only row of the four-copy sweep was proposed and never run. On the i9 it "wins
  only past the knee: 1.107-1.117 at 8-10 workers, 0.816 at 4 … a regime the team policy already
  avoids".
- **P04 (Precopy):** `census/nogo.md`: closed. "copies still replace steps that don't miss".
- **P11, P36 to P40 (move structure, compact Ψ tables):** `census/nogo.md`: "Closed for CI; an
  i9 question only." The caveat: "'Footprint is the minor term' rests on 8573C's 1.04x 2t-spread
  figure; 7763 shows 1.11x." Q52 then measured the footprint term on CI: amplification −3.9%
  and −2.7%. `11-07-knee-mechanism.md` H3: a smaller table was "never measured under cooperative
  teams".
- **P23 (lane and segment counts):** re-measured on the staged kernel. Two sweeps (9V45, 9V74),
  59 settings: "no setting … is faster than the default with four copies, beyond noise".
  48 lanes −3.3% alone on the 9V45 (not beyond noise); 24 lanes +19.0% (+14.5, +23.6) alone
  there; 2048 segments +13.5% and +15.5% alone. "No Intel lane or prefetch sweep has run."
- **P25 (prefetch hint; the conditional 16-lane L2 cell):** `11-05-coresident-walk.md`: not
  reproduced. Room/16/L2 is +62.6% and +28.0% alone and slower beyond noise on 7 of 8 CPUs at
  four copies. "The old four-copy interaction was a property of kernel A." Locally `prefetchnta`
  is +33% at 4 threads. Under SMT the prefetch is worth more: none against L1 goes +48% → +66%
  (9V45) and +23% → +40% (9V74).
- **P27, P29 (instruction window; lanes against misses in flight):** `census/accounting.md`:
  walk MLP by Little's law 9.8 → 5.5 (7763), 9.3 → 5.5 (9V74), 6.4 → 6.9 (8370C), all INFERRED.
  `census/walk.md` MISSED 4 gives the vendors' miss capacities: "No model is at its documented
  miss capacity at 1t." `11-05-corun-placement.md`: "If the walk's SMT loss is a halved window
  … no placement recovers it."
- **P33 (huge pages):** Mined 12.
- **P34 (per-block kernel choice):** the `pattern` corpus moved with three walk changes:
  staging alone −25% wall locally; tail passes −4.0% and −7.9%; tile 512 +1.3% and +1.6%
  (a confirmed harm). `census/nogo.md` records #27 as "noise on wikidata".
- **P70, P71 (Huffman):** Mined 8, Q24, Q46.
- **P72 (parallel scanner):** `census/nogo.md`: "0 at 4t on CI; possibly positive on the i9 at 8
  or more threads." `census/supply.md`: the scanner would bind only if worker CPU fell 2.0 to
  3.8×; reopen "if scanner slot-wait falls below 20% of the span". On the i9 at four teams the
  scanner waits for a slot 38 to 46% of the run (`11-06-team-size.md`). The log names a parked
  construction: "speculative block discovery by bit-level magic search, with a fused
  Huffman+MTF on the workers", for a CPU gain "from overlapping two dependency chains and
  dropping the 1.8 MB symbol array". That is the first statement of the 11-10 parallel scan.
- **P73, P74, P75 (MTF head in registers; the store-forward chain; Seward's list):** shipped as
  #207 (bc899400, in 0.2.4): libbzip2's two-level list, bytes 0 to 31 in two xmm registers.
  - Chain 11.0 → 3.0 cycles (znver4, llvm-mca).
  - 7763, three runs: Wikidata 1 thread −5.2%, −5.2%, −4.8%; MTF + RLE2 −23.5%.
  - 9V45: 1 thread −2.1% (−3.8, −0.4); stage −18.5%. No Intel run.
  - It answers P73's regression: `records` −25% to −28% and `random` −13% at 1 thread (7763),
    where #25 was +11.1% on `random` at 1 thread and +9.7% on `records` at 4.
  - P75's condition ("in case a records-heavy workload appears") is overtaken: the two-level
    list is in production (inferred: `mtfa` is the structure P75 names).
  - Before it, `census/nogo.md` priced a rebased P74-C at 0 to −2.7% of CI's 4t wall, and found
    that #25's 4t result was null and the i9's net w4 gain about +1%.
- **P78 (fused front end):** only the log's line above. Not built.
- **P79, P80, P81 (the build):** Q37 and Q45. `census/nogo.md` reads #114's CI 4t CPU of +1.1%
  as "the sign expected if these reads cost under SMT" (INFERRED), which bears on P81's open
  question of why CI and the i9 disagree. The build's four-copy inflation is ×2.25 (8370C),
  ×1.76 (9V74), ×1.43 (7763).
- **P83 (first-column packing, a write-only build):** `census/nogo.md` reopens it as rank 3:
  "#114 removed the init pass. What remains … is the read of ll at each destination slot."
  Priced at 0 to −1.5% of 4t wall. Test named and not run: "time the build alone in the …
  `kernels` table, a write-only build against today's".
- **P85 (remove the segment-assembly copy):** rebuilt as #183 with the CRC folded in, and closed
  as "not shown at 4 threads". Mined 4. Assembly is the most co-run-inflated pass on AMD
  (×2.97, ×3.53) at 2.7 to 3.2% of four-copy CPU, so the old 5% condition is not met in time.
- **P86 (non-temporal stores):** `11-05-walk-round.md`: non-temporal copies in the staging flush:
  "No time resolved", locally. `11-07-knee-mechanism.md` H4: "non-temporal stores are not
  automatically better for data that is soon read back."
- **P87 (output in L's buffer):** shipped as #164 (e26e3d9c, in 0.2.4).
  - 4t wall −2.2% (−4.0, −0.5) on an 8573C and −1.1% (−2.3, +0.2) on a 6975P-C.
  - Peak RSS 71 → 66 MB in both. Instructions −400 to −542 a full block.
  - Its rule was amended after the literal cutoff; the entry says so and reads both rules.
  - The register's pass bar (occupancy at 6 to 10 workers ≥ 1.03× on the i9) was never run.
    `11-06-team-neutral.md` notes the register still had it "open".
- **P90, P101, P103 (teams below six cores; the worker cap):** forced teams on CI decode at
  0.71× the workers (491.2 against 694.9 MB/s, unpaired, 7763). `census/accounting.md` UNKNOWNS
  7: on the 9V45, 2 workers with the scanner and reader on the siblings "might come close to 4
  workers' wall at about 60% of the CPU (INFERRED)". Unmeasured.
- **P91, P100 (team shape; the 0.7 × L3 cap):** reversed on the i9.
  - `11-06-team-size.md` (exploratory): 3 × 3 at 10 threads 1.075 (1.058–1.093) and 3 × 4 at 12
    1.097 (1.083–1.111) against 4 × 2 at 8.
  - `11-06-three-team-policy.md` (registered, two runs): 1.089 (1.075–1.103) and 1.088
    (1.076–1.101) at 10 threads, CPU 0.88 and 0.89, RSS 0.82 and 0.81. Merged as #234 (0.2.5).
  - "The policy's cap is one table too many on this machine now": 3 tables are 10.8 MB, "about
    0.5 of the L3".
  - Not shown: thread counts 13 to 15 and 17 to 19, any other 10-core 20 MiB machine, any
    corpus but Wikidata at level 9. The window changed with the plan (Mined 5).
- **P94 (SMT siblings in a team):** Mined 9. Packed against spread at four teams: −23% at
  k = 2, −8% at k = 3, −1% at k = 4 (`11-07-knee-mechanism.md` E6).
- **P99 (pool wake-up):** `11-05-job-handoff.md`: the old `recv` "spun and yielded before
  parking, which absorbed gaps of a few microseconds"; at 2 threads involuntary switches rose
  0.136 → 0.255 a block without it. `11-07-knee-mechanism.md` E3 keeps "pool wake and
  completion skew" open.
- **P107 (the admission window):** shipped as #180 (eb5363c2, in 0.2.4), against the register's
  "rejected (demoted)". 4t wall −8.1% (−10.5, −5.5), −10.3% (−11.7, −8.9), −8.9% (−10.5, −7.3);
  2 threads −12%; 4t CPU +4.3% to +6.1%; RSS +8 MB. The old 7 to 8% bound held: "only the
  ranking changed" (`census/nogo.md`). At window 16 a wider one is worth ≤ 0.4% of 4t wall.
  Mined 5.
- **P108 (run-queue delay):** the register has "run-queue wait ~6% of a block's time".
  `census/supply.md`: the run queue inside `decode_block` is 11.7 to 12.6% of worker time, and
  `census/serial-stages.md`: "Each block at 4t is preempted about once." It is the support
  threads' CPU, so the old verdict ("the cost is on a CPU") stands in kind, not in size.
- **P109, P110, P111, P125 (CI's four-thread ceiling):** Q17, Q18, Q51. SMT buys 1.13 to 1.24×
  from 2 to 4 threads. P110's 9V45 remainder is still open: its 4t idle is 6.8 to 8.4% against
  2.7 to 4.5% elsewhere, and no occupancy run has landed on a 9V45.
- **P112, P113, P115 (the knee):** Q73. `11-06-team-size.md`'s reading: "The 8-thread ceiling
  is the four live tables."
- **P120 (AMAC):** `census/nogo.md`: "Closed." Its condition is not met.
- **P141 (order bias):** `census/nogo.md` applies it: bench 1.0.0 "favoured head by about 2%",
  so #25's 1-thread −1.6% is within it.
- **P145 (occupancy trace):** used throughout the census and for team sizes 2 to 4.
- **P146 (PMU):** Mined 10.
- **P153 (code-layout control):** five new cases: Q01; `runs` at 1 thread under #171 ("a
  layout-only rebuild moves it by up to 2.7%"); tile 512 shifting "every hot function by
  0x20–0x30 bytes"; #207's local 2 to 6% in untouched stages; the three-team policy's 8-thread
  control at 0.986 (0.977–0.995), whose "cause, a build or layout effect, is untested".
- **P158 (version ladder):** Q15 is its first run: v0.2.1 against master, direct, on a 7763.
- **P172, P173 (wikidata-pq's threads):** `11-05-window.md`: where the decoder shares cores,
  #180's "extra CPU is a cost. It should be measured there before a release." Not recorded as
  measured. `11-05-corun-placement.md`: wikidata-pq "decodes with one thread a physical core",
  so no SMT construction of this slice moves its ETA.
- **K37 (elasticity of time to walk instructions):** Q13 extends it to the whole decoder.
- **§6 (no ID), "compact30 in production (#155)":** Q12, Q52.
- **§6 (no ID), "B2 kernel B as specified":** `11-05-deferred-flag-walk.md` prices its eight
  speculative rounds: "one instruction a step, the instruction this saves, plus a replay
  probability of about 1 − e^(−256/1758) ≈ 13.6% a tile instead of about 1.8% an offset". The
  one-offset form (#168) then lost to no speculation at all (#181). Kernel B is still unbuilt.
- **§6 (no ID), "B2 kernel C" (AVX2 gathers):** Q44.
- **§6 (no ID), "B2 interval model (iv)":** `11-05-coresident-walk.md`: its prediction "that 16
  lanes would be slower than 32 even on an SMT pair, holds in sign. Its size is not tested."
- **§6 (no ID), "Behaviour at more copies" and "Memory interference":** Q17 to Q19 and Q51 are
  the answer for CI: the loss at four threads is the SMT sibling's.
- **§6 (no ID), "#157":** `census/nogo.md`: it "shows that 4-worker nulls carry over from the
  i9 to CI". Mined 2 shows the reverse direction does not hold at one thread.
- **§6 (no ID), "the construction-only gate":** Q13.

#### Changed since 30 October

Old IDs whose status or evidence this slice changed:
- **P87:** open → shipped. `11-05-p87-production-validation.md` (#164).
- **P107:** rejected → shipped. `11-05-window.md` (#180).
- **P74:** open → shipped; **P73:** its regression answered; **P75:** overtaken.
  `11-05-mtf-two-level.md` (#207).
- **P91, P100:** teams of 2 under 0.7 × L3 → three teams of three at 10 or more threads on the
  ten-core topology. `11-06-team-size.md`, `11-06-three-team-policy.md` (#234).
- **P25:** conditional → rejected. `11-05-coresident-walk.md`.
- **P85:** rejected on a bound → built, and closed as not shown at 4 threads.
  `11-05-fused-postwalk.md` (#183).
- **P23:** re-measured on the staged kernel. `11-05-coresident-walk.md`, `census/walk.md`.
- **P72:** priced at 0 on CI at 4 threads, with a reopening test. `census/supply.md`.
- **P83:** superseded → priced again as a screen. `census/nogo.md`.
- **P94:** first placement figures for teams. `11-06-team-size.md`.
- **P108:** the run-queue share doubled in the record. `census/supply.md`.
- **P146:** "i9 yes" no longer holds. `11-06-team-size.md`.
- **P153:** five more unexplained layout-sized shifts. See "Bears on old IDs".
- **P158:** first direct ladder run. `11-05-bench-precision.md`.
- **P01, P04, P11, P36 to P40, P90, P103, P120:** closures confirmed for CI, each with its
  premise re-read. `census/nogo.md`.

New IDs whose status or premise changed after they entered the record:
- **Q45:** open → shipped. `11-09-build-window-runs-v2.md` (#312).
- **Q37:** its inference contradicted. `11-09-run-fed-stage-result.md`.
- **Q13:** a counted ratio confirmed by timing. `reg2/design-e-result.md`.
- **Q12, Q52, Q54:** the regime they were judged in is no longer the objective's.
  `11-10-share-nothing.md`.
- **Q24, Q46:** the scanner's share by data class. `11-10-by-data-class.md`.
- **Q35, Q41, Q74, Q76:** priced by the simulated ledger. `11-10-bandwidth-titration.md`,
  `11-10-borrowed-tricks.md`.
- **Q60, Q61, Q75:** re-tested locally, null. `11-10-borrowed-tricks.md`.
- **Q72, Q73:** the default plan changed under them (0.2.6), and E1 was reworded.
  `11-07-knee-mechanism.md`.
- **Q16:** the i9 reading in Mined 2 is new here.

#### Reopen candidates

Rejected or superseded IDs whose rejection was of a construction only, with what a new
construction would have to cost.

- **Q52 (and Q12), compact30 under the staged walk.** Closed on CI's core term. It reopens for
  the i9 if its build and walk, counted as a complete loop, come to no more than about 1.2 times
  production's 15.3 instructions a row (18), and plain workers at 6 to 10 hold their CPU a block
  better than the 3.6 MB table does. Break-even is g near 2.5 only if the table also keeps ten
  workers under the L3, which 1.95 MB does not promise.
- **P85 (#183), the fused post-walk pass.** Built and correct. It needs no new construction. It
  needs a criterion it can meet: the 1-thread wall on CI, or cold written lines and an i9 run.
- **Q38, tail passes.** A tail that steps in passes with #181's per-lane start test and no
  second row slot. It must bring the tail's 17 instructions a byte towards the busy phase's 10.7
  without the replay that cost #168 its time.
- **Q23, the job handoff.** The same branch, read at one thread a physical core. It must convert
  queued idle into wall, which on CI it could not.
- **Q44, the gather walk.** Gate 0 as registered: variant over room ≤ 0.95 at four copies on a
  Zen 3 or Zen 4 runner. For share-nothing it must instead beat the staged walk at one thread on
  the i9, where it has never run.
- **Q22, software interleaving (C3).** Absorption of at least 0.63 to 0.69 of the filler, counted
  on the target core. Zen 3 gave 0.35 to 0.55 in the model.
- **Q30 and Q64, the partitioned walk and cross-chain lockstep.** Both were priced at CI's miss
  cost. A per-segment path of at most about 10 cycles is the recorded bar. Neither shrinks the
  table, so neither is a share-nothing design.
- **Q37, the build's run stores.** Overtaken by #312. Nothing to reopen.
- **Not candidates,** because the advantage itself was disproved at the boundary that matters:
  Q39 (tile sizes), Q40 (split condition variables), Q42 (deferred flags), Q53 (split layout),
  Q60 (ISA clones), Q66 (lean segments), Q69 (launch order), Q72 (more helpers a table).
