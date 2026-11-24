# Retrospective cases: the extracted facts

Phase 2 of the retrospective (`docs/journal/2026-11-24-retrospective-proposal.md` §4a, as corrected by MONODROMY 97). One section per idea, in the order of `cases.tsv`. It states no verdicts: those are in `outcomes.sealed.tsv`, which coders do not open. Method and limits are in `README.md`.

## walk-flagged-starts: Segment starts flagged in their own T-vector entries; raw per-lane write pointers

- **PRs:** [#12](https://github.com/lmmx/mibz2/pull/12)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; PR #12 body names no other proposer
- **commit_to_build:** 2026-10-27T11:47Z, PR #12 first commit (author date) (https://github.com/lmmx/mibz2/pull/12/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 1 matching lines in 1 documents dated before commit_to_build, searched on `segment start|idle.lane|flag` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T09:58Z] docs/journal/2026-10-27-bz2-decoder.md (doc): "# 2026-10-27: A native bz2 decoder for the split, behind a flag: byte for byte lbzip2's, its decode 1.4–2.3x faster, the split's CPU −20% on CI" — https://github.com/lmmx/mibz2/blob/c1aed8c6c476/docs/journal/2026-10-27-bz2-decoder.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #12) / Optimiser: unrecorded (no comment from the role naming #12)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## walk-lane-prefetch: Each lane's next T-vector entry prefetched as soon as its row is known (32 lanes, 512 segments)

- **PRs:** [#21](https://github.com/lmmx/mibz2/pull/21)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #5 (lane count and prefetch)
- **commit_to_build:** 2026-10-27T13:55Z, PR #21 first commit (author date) (https://github.com/lmmx/mibz2/pull/21/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "`phases` on the Wikidata blocks, at one thread: \| \| master \| this PR \| \|---\|--:\|--:\| \| walk, flagged \| 15.26 c/B \| 10.20 to 11.22 c/B \| \| whole decode \| 27.0 c/B \| 23.0 to 23.1 c/B \| A local `ab` run, 5 paired rounds against master, gave the same bytes on every corpus." (https://github.com/lmmx/mibz2/pull/21 (PR body, current text))
- "Wall ratios with 95% intervals: \| corpus \| 1 thread \| 4 threads \| \|---\|--:\|--:\| \| wikidata \| 0.809 (0.696, 0.942) \| 0.742 (0.664, 0.829) \| \| text \| 0.722 (0.415, 1.256) \| 0.835 (0.539, 1.294) \| \| records \| 0.893 (0.792, 1.006) \| 0.906 (0.688, 1.193) \| ## CI's measurement CI's be…" (https://github.com/lmmx/mibz2/pull/21 (PR body, current text))

**must_hold** (quoted):

- "`phases` on the Wikidata blocks, at one thread: \| \| master \| this PR \| \|---\|--:\|--:\| \| walk, flagged \| 15.26 c/B \| 10.20 to 11.22 c/B \| \| whole decode \| 27.0 c/B \| 23.0 to 23.1 c/B \| A local `ab` run, 5 paired rounds against master, gave the same bytes on every corpus." (https://github.com/lmmx/mibz2/pull/21 (PR body, current text))
- "It runs 10 paired rounds against the merge base at 1 and 4 threads, on the gen corpora and the Wikidata dump windows." (https://github.com/lmmx/mibz2/pull/21 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T13:56Z] "One segment means fewer segments than lanes, so only the drain loop runs." (https://github.com/lmmx/mibz2/pull/21 (PR body, current text))
- [2026-10-27T13:56Z] "If 32 lanes loses there, a revision with fewer lanes (16 or 24, still with the prefetch) follows." (https://github.com/lmmx/mibz2/pull/21 (PR body, current text))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `prefetch|lanes? count|32 lanes|512 segments` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #21) / Optimiser: unrecorded (no comment from the role naming #21)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## huffman-two-symbols: Up to two Huffman symbols a table lookup, four lookups a refill

- **PRs:** [#24](https://github.com/lmmx/mibz2/pull/24)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #8
- **commit_to_build:** 2026-10-27T14:34Z, PR #24 first commit (author date) (https://github.com/lmmx/mibz2/pull/24/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "It changes the scanner's Huffman decode, about 1.9 c/B of a decode on one thread." (https://github.com/lmmx/mibz2/pull/24 (PR body, current text))

**must_hold** (quoted):

- "The scanner is the serial part: at 4 threads it binds from about 9 worker threads (#21's model)." (https://github.com/lmmx/mibz2/pull/24 (PR body, current text))
- "CI's paired bench decides." (https://github.com/lmmx/mibz2/pull/24 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T14:34Z] "The scanner is the serial part: at 4 threads it binds from about 9 worker threads (#21's model)." (https://github.com/lmmx/mibz2/pull/24 (PR body, current text))

**same_quantity_facts:** 3 matching lines in 2 documents dated before commit_to_build, searched on `huffman|two.symbol|lookup table|refill` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "- multi-symbol Huffman tables (≤ 1 c/B of 1.9);" — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md
- [2026-10-27T09:44Z] docs/journal/2026-10-27-bz2-bakeoff.md (doc): "decoding at ≤ 1 of the Huffman phase's 1.9 c/B." — https://github.com/lmmx/mibz2/blob/734f9da49a33/docs/journal/2026-10-27-bz2-bakeoff.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #24) / Optimiser: unrecorded (no comment from the role naming #24)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## build-four-quarters: T-vector build in four interleaved quarters

- **PRs:** [#23](https://github.com/lmmx/mibz2/pull/23)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #7
- **commit_to_build:** 2026-10-27T14:30Z, PR #23 first commit (author date) (https://github.com/lmmx/mibz2/pull/23/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "It changes the T-vector build (`bwt::build`), about 21% of a decode at one thread." (https://github.com/lmmx/mibz2/pull/23 (PR body, current text))

**must_hold** (quoted):

- "CI's paired bench decides." (https://github.com/lmmx/mibz2/pull/23 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T14:30Z] "- The last column is now cut into four quarters." (https://github.com/lmmx/mibz2/pull/23 (PR body, current text))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `counting sort|build stage|bwt::build|store.to.load|dependent store` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #23) / Optimiser: unrecorded (no comment from the role naming #23)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## mtf-avx2-registers: MTF list's first 32 entries in two AVX2 registers

- **PRs:** [#25](https://github.com/lmmx/mibz2/pull/25)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #8
- **commit_to_build:** 2026-10-27T14:34Z, PR #25 first commit (author date) (https://github.com/lmmx/mibz2/pull/25/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "It changes the AVX2 move-to-front, about 2.1 c/B of a decode on one thread." (https://github.com/lmmx/mibz2/pull/25 (PR body, current text))

**must_hold** (quoted):

- "CI's paired bench decides." (https://github.com/lmmx/mibz2/pull/25 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 2 matching lines in 1 documents dated before commit_to_build, searched on `move.to.front|MTF` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "\| 1 \| **bz2 decoder built for memory-level parallelism** \| the walk in 12 joined chains; PCLMULQDQ CRC; AVX2 RLE1 and MTF \| **1.76× measured** (42.7 → 24.3 c/B, byte-identical); ceiling ~2.8× \| 53.5% of the split \| **−46 min** measured (6h51m → 6h05m, −11%); −69 min ceiling; −59/−88 by the re…" — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #25) / Optimiser: unrecorded (no comment from the role naming #25)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## walk-few-runs-plain: Blocks with very few runs walked plainly (per-block kernel choice)

- **PRs:** [#27](https://github.com/lmmx/mibz2/pull/27)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #9
- **commit_to_build:** 2026-10-27T14:42Z, PR #27 first commit (author date) (https://github.com/lmmx/mibz2/pull/27/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "perf(walk): a block with fewer than one run per 1024 bytes walked plainly (#9) A short repeated pattern's list is a few long cycles through the T-vector, which the hardware prefetcher follows, so the segmented walk's lanes only cost: on the pattern corpus the plain walk took 0.34-0.62 of its time, …" (https://github.com/lmmx/mibz2/pull/27 (first commit message))

**must_hold** (quoted):

- "CI's paired bench decides." (https://github.com/lmmx/mibz2/pull/27 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T14:42Z] "perf(walk): a block with fewer than one run per 1024 bytes walked plainly (#9) A short repeated pattern's list is a few long cycles through the T-vector, which the hardware prefetcher follows, so the segmented walk's lanes only cost: on the pattern corpus the plain walk took 0.34-0.62 of its time, at ~10^5 bytes a run; every real-data block measured had at most ~19 bytes a run." (https://github.com/lmmx/mibz2/pull/27 (first commit message))
- [2026-10-27T14:42Z] "**What changed (`bwt::Segmented::invert`):** after the build, a block whose last column has fewer than one run of equal bytes per 1,024 bytes is walked plainly." (https://github.com/lmmx/mibz2/pull/27 (PR body, current text))

**same_quantity_facts:** 6 matching lines in 4 documents dated before commit_to_build, searched on `pattern|few runs|plain walk|repetitive` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "- the walk 31 → 10.0–14 c/B (2.2–3.1×) on the same blocks, equal to the plain walk;" — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md
- [2026-10-27T09:10Z] docs/journal/2026-10-27-bwt-inversion-literature.md (doc): "3. **Wikidata JSON is highly repetitive** (17.5× in bzip2, against ~3.5–4× for English), which puts it nearer the root's "highly repetitive" regime, where Precopy won. Whether that holds per 900 kB block is an empirical question; the queued follow-up's Precopy screen decides it." — https://github.com/lmmx/mibz2/blob/e8d57ae7ff5a/docs/journal/2026-10-27-bwt-inversion-literature.md
- [2026-10-27T09:40Z] docs/journal/2026-10-27-mibz2-v2.md (doc): "- a short repeated pattern decodes at 0.67× libbzip2, because the segmented walk loses" — https://github.com/lmmx/mibz2/blob/09984cf9c1ba/docs/journal/2026-10-27-mibz2-v2.md
- [2026-10-27T09:58Z] docs/journal/2026-10-27-bz2-decoder.md (doc): "- **A short pattern repeated** is 0.67× libbzip2 on one thread. Its walk is" — https://github.com/lmmx/mibz2/blob/c1aed8c6c476/docs/journal/2026-10-27-bz2-decoder.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #27) / Optimiser: unrecorded (no comment from the role naming #27)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## build-run16-store: 16 equal last-column bytes linked with one 16-wide store

- **PRs:** [#32](https://github.com/lmmx/mibz2/pull/32)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #7
- **commit_to_build:** 2026-10-27T16:56Z, PR #32 first commit (author date) (https://github.com/lmmx/mibz2/pull/32/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.00; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "This PR works on #7's goal, the T-vector build: 6.3 c/B, 28% of a Wikidata decode at one thread (`examples/phases`)." (https://github.com/lmmx/mibz2/pull/32 (PR body, current text))
- "**Why it should matter for Wikidata:** in the same-binary reference check, libsais's cyclic inverse ran 12.1 c/B against our 18.2 on 431 Wikidata blocks, with identical output. 78.5% of libsais's rows go through this shortcut." (https://github.com/lmmx/mibz2/pull/32 (PR body, current text))
- "CI's paired bench decides): \| Corpus \| Rows via run stores \| Build before → after \| \|---\|---\|---\| \| wikidata \| 81.9% \| 5.70 → 2.55 c/B \| \| text \| 48.7% \| 5.63 → 4.40 c/B \| \| runs \| 70.5% \| 5.62 → 3.29 c/B \| \| records \| 0% \| no consistent change \| \| random \| 0% \| no consis…" (https://github.com/lmmx/mibz2/pull/32 (PR body, current text))

**must_hold** (quoted):

- "CI's paired bench decides): \| Corpus \| Rows via run stores \| Build before → after \| \|---\|---\|---\| \| wikidata \| 81.9% \| 5.70 → 2.55 c/B \| \| text \| 48.7% \| 5.63 → 4.40 c/B \| \| runs \| 70.5% \| 5.62 → 3.29 c/B \| \| records \| 0% \| no consistent change \| \| random \| 0% \| no consis…" (https://github.com/lmmx/mibz2/pull/32 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T16:57Z] "**Why it should matter for Wikidata:** in the same-binary reference check, libsais's cyclic inverse ran 12.1 c/B against our 18.2 on 431 Wikidata blocks, with identical output. 78.5% of libsais's rows go through this shortcut." (https://github.com/lmmx/mibz2/pull/32 (PR body, current text))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `run of (16|equal)|16.wide|equal last.column|build stage` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #32) / Optimiser: unrecorded (no comment from the role naming #32)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## lf2-pair-walk: LF² / ψ² pair walk (super-alphabet, two bytes a step; libsais port)

- **PRs:** [#29](https://github.com/lmmx/mibz2/pull/29), [#34](https://github.com/lmmx/mibz2/pull/34)
- **Journal sources:** docs/journal/2026-10-27-pair-walk.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #6, from the literature (Kärkkäinen, Kempa and Puglisi, DCC 2012) and libsais 2.12
- **commit_to_build:** 2026-10-27T14:58Z, PR #29 first commit (author date) (https://github.com/lmmx/mibz2/pull/29/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.10; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- "Made the default here only so the paired bench measures the whole decoder against the flagged walk; not for merging unless it wins." (https://github.com/lmmx/mibz2/pull/29 (first commit message))
- "**An experiment for #6.** On this branch, `bwt::Pairs` replaces the decoder's default only so that the paired CI bench can time the whole decoder with it against master's flagged walk." (https://github.com/lmmx/mibz2/pull/29 (PR body, current text))
- "Units are cycles per output byte (at 2.8 GHz) on one pinned core, best of 5." (https://github.com/lmmx/mibz2/pull/29 (PR body, current text))
- "**An experiment for #6, not expected to merge.** It makes the new kernel the default only so the paired CI bench can compare it with `Segmented` at 0.1.4, which includes #32's run stores." (https://github.com/lmmx/mibz2/pull/34 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T14:58Z] "**An experiment for #6.** On this branch, `bwt::Pairs` replaces the decoder's default only so that the paired CI bench can time the whole decoder with it against master's flagged walk." (https://github.com/lmmx/mibz2/pull/29 (PR body, current text))
- [2026-10-27T14:58Z] "- the pair table alone costs about 10 c/B;" (https://github.com/lmmx/mibz2/pull/29 (PR body, current text))
- [2026-10-27T17:18Z] "**An experiment for #6, not expected to merge.** It makes the new kernel the default only so the paired CI bench can compare it with `Segmented` at 0.1.4, which includes #32's run stores." (https://github.com/lmmx/mibz2/pull/34 (PR body, current text))
- [2026-10-27T17:18Z] "The walk takes half the steps, but each step costs about twice as much." (https://github.com/lmmx/mibz2/pull/34 (PR body, current text))
- [2026-10-27T17:18Z] "With prefetching, the walk is bound by instructions per step rather than by memory latency, so the extra work per step (the first-byte lookup and the second byte) cancels the halving." (https://github.com/lmmx/mibz2/pull/34 (PR body, current text))

**same_quantity_facts:** 1 matching lines in 1 documents dated before commit_to_build, searched on `LF²|ψ²|psi2|pair (walk|table)|super.alphabet|libsais|two bytes a step` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T09:10Z] docs/journal/2026-10-27-bwt-inversion-literature.md (doc): "- **The super-alphabet:** precompute LF² = LF[LF[i]] with the two characters, so each dependent lookup yields two characters. This is the "nearly 2× fewer misses"." — https://github.com/lmmx/mibz2/blob/e8d57ae7ff5a/docs/journal/2026-10-27-bwt-inversion-literature.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6729693845 (2026-11-09T00:56Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #29, #34)

**monodromy_involved:** yes: records naming its PRs or recipes: 2026-11-23-emit-cost-sq-review.md, 2026-11-23-refinement-and-composition.md, 2026-11-23-squared-trace-arm.md

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## precopy: Precopy (copying repeats, Kärkkäinen-Puglisi 2011) in the segmented walk

- **PRs:** [#30](https://github.com/lmmx/mibz2/pull/30), [#36](https://github.com/lmmx/mibz2/pull/36)
- **Journal sources:** docs/journal/2026-10-27-precopy-forward-chains.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #9, from the literature (KP 2011)
- **commit_to_build:** 2026-10-27T15:26Z, PR #30 first commit (author date) (https://github.com/lmmx/mibz2/pull/30/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.09; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- On Wikidata, the 31.5% of steps that copies remove saved almost nothing." (https://github.com/lmmx/mibz2/pull/36 (PR body, current text))

**must_hold** (quoted):

- "Made the default here only so the paired bench measures it; not for merging unless it wins." (https://github.com/lmmx/mibz2/pull/30 (first commit message))
- "**#9 step 3: a bounded experiment.** On this branch `bwt::Precopy` replaces the decoder's default, only so that the paired CI bench can time it against master's flagged walk." (https://github.com/lmmx/mibz2/pull/30 (PR body, current text))
- "**An experiment for #9, not expected to merge:** the gated configuration through the paired bench." (https://github.com/lmmx/mibz2/pull/36 (PR body, current text))
- "- **Gate.** `Segmented` uses this walk for blocks where at least a quarter of the rows went in by run stores (`build_runs` returns the count)." (https://github.com/lmmx/mibz2/pull/36 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T15:26Z] "**#9 step 3: a bounded experiment.** On this branch `bwt::Precopy` replaces the decoder's default, only so that the paired CI bench can time it against master's flagged walk." (https://github.com/lmmx/mibz2/pull/30 (PR body, current text))
- [2026-10-27T15:26Z] "**Every byte's row is recorded,** so that later copies can use it as a source." (https://github.com/lmmx/mibz2/pull/30 (PR body, current text))
- [2026-10-27T15:26Z] "- the row recording alone costs about 13 c/B, against about 1 c/B of bandwidth for a sequential 4-bytes-per-byte stream." (https://github.com/lmmx/mibz2/pull/30 (PR body, current text))
- [2026-10-27T15:26Z] "- the restructured loop and its setup cost about 8 c/B more on top, with nothing recorded at all." (https://github.com/lmmx/mibz2/pull/30 (PR body, current text))
- [2026-10-27T15:26Z] "Even with copying free, the 44% of bytes the lanes copy saves at most about 5 c/B of walk." (https://github.com/lmmx/mibz2/pull/30 (PR body, current text))
- [2026-10-27T15:26Z] "Inside the segmented walk only 44% is copyable, against the 65% the offline analysis found, because the neighbouring row often hasn't been walked yet." (https://github.com/lmmx/mibz2/pull/30 (PR body, current text))

**same_quantity_facts:** 7 matching lines in 4 documents dated before commit_to_build, searched on `[Pp]recopy|repeats?|forward chain|copied` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "compares, a movemask), with the stretches between runs copied whole. **3.58 → 0.56" — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md
- [2026-10-27T09:10Z] docs/journal/2026-10-27-bwt-inversion-literature.md (doc): "3. **Wikidata JSON is highly repetitive** (17.5× in bzip2, against ~3.5–4× for English), which puts it nearer the root's "highly repetitive" regime, where Precopy won. Whether that holds per 900 kB block is an empirical question; the queued follow-up's Precopy screen decides it." — https://github.com/lmmx/mibz2/blob/e8d57ae7ff5a/docs/journal/2026-10-27-bwt-inversion-literature.md
- [2026-10-27T09:40Z] docs/journal/2026-10-27-mibz2-v2.md (doc): "- a 64-byte line repeated (200 MiB);" — https://github.com/lmmx/mibz2/blob/09984cf9c1ba/docs/journal/2026-10-27-mibz2-v2.md
- [2026-10-27T09:58Z] docs/journal/2026-10-27-bz2-decoder.md (doc): "2–5). lbzip2 itself ran 1.3–1.5× slower here than in #194. The ratios repeat across" — https://github.com/lmmx/mibz2/blob/c1aed8c6c476/docs/journal/2026-10-27-bz2-decoder.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6729693845 (2026-11-09T00:56Z, after commit_to_build) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6733103415 (2026-11-09T05:36Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## room-walk-passes: Flagged walk in passes with a shared offset (libsais's chain decode)

- **PRs:** [#37](https://github.com/lmmx/mibz2/pull/37)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; from libsais's chain decode
- **commit_to_build:** 2026-10-27T18:04Z, PR #37 first commit (author date) (https://github.com/lmmx/mibz2/pull/37/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "This PR works on the walk, now ~60% of a Wikidata decode." (https://github.com/lmmx/mibz2/pull/37 (PR body, current text))
- "Those are 10–14 ms decodes of synthetic, single-symbol or run-only blocks." (https://github.com/lmmx/mibz2/pull/37 (PR body, current text))
- "Weighed against −6 to −10% on Wikidata, it is kept." (https://github.com/lmmx/mibz2/pull/37 (PR body, current text))

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-27T18:04Z] "- **The gain depends on the CPU:** −10% on the EPYC 9V74 and −6% on the Xeon at one thread." (https://github.com/lmmx/mibz2/pull/37 (PR body, current text))
- [2026-10-27T18:04Z] "With the tail's grow removed, Miri reports the out-of-bounds write." (https://github.com/lmmx/mibz2/pull/37 (PR body, current text))

**same_quantity_facts:** 2 matching lines in 2 documents dated before commit_to_build, searched on `room|shared offset|passes|chain decode` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T09:58Z] docs/journal/2026-10-27-bz2-decoder.md (doc): "- **A block whose compressed form passes 64 MiB** is an error. A real block is at most" — https://github.com/lmmx/mibz2/blob/c1aed8c6c476/docs/journal/2026-10-27-bz2-decoder.md
- [2026-10-27T12:20Z] docs/journal/2026-10-27-site-and-bench-tracker.md (doc): "- examples/ab.rs `run` takes `--settle S`, the length of its load sample before each run (default 2 s, as before); the workflow passes 0.25" — https://github.com/lmmx/mibz2/blob/5121954e8efb/docs/journal/2026-10-27-site-and-bench-tracker.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #37) / Optimiser: unrecorded (no comment from the role naming #37)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## move-structure-walk: Length-capped move structure for ψ under the room walk

- **PRs:** none
- **Journal sources:** docs/journal/2026-10-27-move-structure-walk.md
- **i9 recipes:** none
- **Deciding machine:** local VM, one thread (the criterion's machine)
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #42
- **commit_to_build:** 2026-10-27T20:01Z, docs/journal/2026-10-27-move-structure-walk.md first commit 50d404c8 (no PR; earliest recorded) (https://github.com/lmmx/mibz2/blob/50d404c8bdd2/docs/journal/2026-10-27-move-structure-walk.md)
- **Cost:** i9 0 min [no i9 bundle named]; CI: unrecorded; calendar days, commit to verdict: 0.00; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 1 matching lines in 1 documents dated before commit_to_build, searched on `move structure|length.capped|run.length ψ|r.index` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T19:19Z] docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md (doc): "\| #8 memory \| §3: a run-length ψ would cut ≈3 MB a slot \| noted in #42; no change to #8 \|" — https://github.com/lmmx/mibz2/blob/3e5de265a6ff/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming the case) / Optimiser: unrecorded (no comment from the role naming the case)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## lane-prefetch-tuning-i9: Lane-count and prefetch tuning as the remedy for the i9's scaling knee

- **PRs:** none
- **Journal sources:** docs/journal/2026-10-28-workstation-scaling.md
- **i9 recipes:** none
- **Deciding machine:** i9-10900K (bench-variants)
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issues #39, #40
- **commit_to_build:** 2026-10-28T00:37Z, docs/journal/2026-10-28-workstation-scaling.md first commit c0336254 (no PR; earliest recorded) (https://github.com/lmmx/mibz2/blob/c0336254517f/docs/journal/2026-10-28-workstation-scaling.md)
- **Cost:** i9 0 min [none on the evidence branch]; CI: unrecorded; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 24 matching lines in 10 documents dated before commit_to_build, searched on `knee|lanes?|prefetch|scal(ing|es) to` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "- **CRC by PCLMULQDQ:** 4 × 128-bit lanes folded with x^576 and x^512 mod P, the last 64" — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md
- [2026-10-27T09:44Z] docs/journal/2026-10-27-bz2-bakeoff.md (doc): "\| **pbz2** \| **The same idea, found independently** (`pbz2-core/src/block.rs`, `walk_in_pieces`). 16 lanes over 128 evenly spaced starts plus the origin. A start is marked by bit 31 of the T-vector entry itself, so there is no separate bitmap. Pieces go into a paged arena of 1 KiB pages (linked b…" — https://github.com/lmmx/mibz2/blob/734f9da49a33/docs/journal/2026-10-27-bz2-bakeoff.md
- [2026-10-27T15:46Z] docs/journal/2026-10-27-literature-reading.md (doc): "- **indexed_bzip2** (Knespel): block-parallel decoding with prefetching and caching, sharing rapidgzip's architecture. Its README reports 6.1× over Python's bz2 at 12 threads, but gives no block-finder figures. Its block finder bears on #10, which stays deferred." — https://github.com/lmmx/mibz2/blob/24169e657c1f/docs/journal/2026-10-27-literature-reading.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-bloom-inverse-bwt.md (doc): "inverse BWT, build and 32-lane walk, is ≈13 c/B on Wikidata at 0.1.4 (local, one" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-bloom-inverse-bwt.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-precopy-forward-chains.md (doc): "per-chain records) in the lanes' order, copies 31.5% of Wikidata's bytes and is still" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-precopy-forward-chains.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "\| #6 LF² / ψ² \| 1: libsais's run lanes + 2-byte walk beat our build + walk by 34% on Wikidata, byte-identical \| reopened, ported (#34), slower: **closed**. The walk-loop question is queued \|" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-27T19:19Z] docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md (doc): "count falls to 88% at 9 lanes, 64% at 11 and 40% at 12, with a 10.1× speedup at 12." — https://github.com/lmmx/mibz2/blob/3e5de265a6ff/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md
- [2026-10-27T20:01Z] docs/journal/2026-10-27-move-structure-walk.md (doc): "32-lane loop rows against the 32-lane select rows). It is taken on 27–29% of steps." — https://github.com/lmmx/mibz2/blob/50d404c8bdd2/docs/journal/2026-10-27-move-structure-walk.md
- [2026-10-27T20:38Z] docs/journal/2026-10-27-walk-sweep.md (doc): "- **Prefetch survives decode order but saves less:** 13% at 16 or more lanes alone and" — https://github.com/lmmx/mibz2/blob/10e9761b63cb/docs/journal/2026-10-27-walk-sweep.md
- [2026-10-27T21:03Z] docs/journal/2026-10-27-output-side.md (doc): "- under the 32-lane room walk, it is 90–108% slower, build included;" — https://github.com/lmmx/mibz2/blob/2180ae5ee6a2/docs/journal/2026-10-27-output-side.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming the case) / Optimiser: unrecorded (no comment from the role naming the case)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## cooperative-decoder: Cooperative inverse BWT: k cores on one block's T-vector, as the default policy

- **PRs:** [#73](https://github.com/lmmx/mibz2/pull/73), [#104](https://github.com/lmmx/mibz2/pull/104), [#105](https://github.com/lmmx/mibz2/pull/105), [#106](https://github.com/lmmx/mibz2/pull/106)
- **Journal sources:** docs/journal/2026-10-29-cooperative-decoder.md
- **i9 recipes:** none
- **Deciding machine:** i9-10900K (registered full-decoder A/B against the release gates)
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issues #39, #65 (the knee); pilot #73
- **commit_to_build:** 2026-10-28T12:12Z, PR #73 first commit (author date) (https://github.com/lmmx/mibz2/pull/73/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 13 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 1.25; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "On the i9 it sits at 4 × 3.6 MB ≈ 0.7 × 20 MB, and the decoder keeps one T-vector per worker." (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))
- "- the discovery run's analysis (§7) and what follows it (§8). ## What the VM showed, and why it couldn't decide The VM has 4 vCPUs and a 260 MB L3, so there's no capacity pressure: only the costs of sharing show." (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))
- "- **Walk:** sharing scales it 2.4× on 4 cores (3.99 → 1.69 ns/B)." (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))

**must_hold** (quoted):

- "- bench/coop.sh, `just bench-coop`: builds libsais from a pinned, checksummed commit with OpenMP when gcc has it, runs both modes on one CPU of each physical core, and writes coop-<UTC>.md." (https://github.com/lmmx/mibz2/pull/73 (first commit message))
- "On the i9 it sits at 4 × 3.6 MB ≈ 0.7 × 20 MB, and the decoder keeps one T-vector per worker." (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))
- "This PR asks a structural question: can N blocks in flight, each built and walked by k cores sharing its one table, use N × k cores with only N tables live? ## The workstation's discovery run (measured, c2620b8) On the i9-10900K: 19 arms × 38 paired rounds, every thread pinned to its own physical c…" (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))
- "- **Controls:** - **Placement:** with `COOP_CPUS`, every thread is pinned to its own CPU and where it ran is checked." (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-29T17:14Z] "Each of k ranges, cut at multiples of the checkpoint stride, scatters its rows into its own window of every bucket, each entry written whole (ll[s] \| i << 8) with bwt::build's 16-row run stores, with no count, copy or barrier first." (https://github.com/lmmx/mibz2/pull/104 (first commit message))
- [2026-10-29T17:14Z] "Measured on the shared VM (indicative only): at k = 1 the window checks cost about 7-8% of the build (1.115 against 1.027 ns/B best of 14; paired ratios noisy)." (https://github.com/lmmx/mibz2/pull/104 (first commit message))
- [2026-10-29T17:41Z] "The policy, Plan::for_threads, keeps today's workers where Linux's topology can't be read or fewer than 6 physical cores serve the threads (counted from the process's allowed CPUs), else teams of 2 on the physical cores, capped to keep the live tables near 0.7 of the L3 (4 teams on the i9)." (https://github.com/lmmx/mibz2/pull/105 (first commit message))
- [2026-10-28T12:12Z] "- **Equal-segment controls (`4x1@1024`, …):** one core walking k cores' segments, so what the cores buy is told from what the finer cut does." (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))
- [2026-10-28T12:12Z] "- **Segment control:** the equal-segment control shows the finer cut costs 2–6%, so the cores account for the gain." (https://github.com/lmmx/mibz2/pull/73 (PR body, current text))
- [2026-10-29T17:29Z] "On the i9 that would be about 2% of a team's round." (https://github.com/lmmx/mibz2/pull/104 (PR body, current text))

**same_quantity_facts:** 5 matching lines in 2 documents dated before commit_to_build, searched on `cooperative|k cores|one block's T.vector|tables? live|knee` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- **Calibrate instead of trusting sysfs.** Before each bench, run a sub-second single-thread dependent pointer-chase over working sets of 256 KB→64 MB (×√2 steps). The latency knees give an *effective* L2 and L3 size as seen at that moment, neighbours included. This is the Pirate's self-verificatio…" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "would have room for about 1.6× at the knee, an upper bound, since the other block" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #73, #104, #105, #106) / Optimiser: unrecorded (no comment from the role naming #73, #104, #105, #106)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## one-pass-build: T-vector built in one pass (the scatter takes L's byte from L)

- **PRs:** [#93](https://github.com/lmmx/mibz2/pull/93), [#114](https://github.com/lmmx/mibz2/pull/114)
- **Journal sources:** docs/journal/2026-10-29-one-pass-build.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; issue #81
- **commit_to_build:** 2026-10-29T12:51Z, PR #93 first commit (author date) (https://github.com/lmmx/mibz2/pull/93/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.45; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **predicted:** build 1.03–1.12×." (https://github.com/lmmx/mibz2/pull/93 (PR body, current text))
- "- **VM, indicative only:** the build stage at 1.117 (1.051–1.186), the whole one-core decode at 1.029 (0.993–1.066)." (https://github.com/lmmx/mibz2/pull/114 (PR body, current text))

**must_hold** (quoted):

- "So the build now asserts that every slot was written: each byte's cursor must end at the next bucket's start, and the last at n." (https://github.com/lmmx/mibz2/pull/93 (PR body, current text))
- "The whole decode moves by the build's share of it, so the end-to-end figure may stay inside noise." (https://github.com/lmmx/mibz2/pull/93 (PR body, current text))
- "- **End check:** after the scatter, every bucket's cursor must equal its end." (https://github.com/lmmx/mibz2/pull/114 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-29T12:51Z] "**The table is only sized, not filled.** A table kept from a block as long or longer is cut to n, and every one of its stale words is overwritten." (https://github.com/lmmx/mibz2/pull/93 (PR body, current text))
- [2026-10-29T12:51Z] "- **predicted:** build 1.03–1.12×." (https://github.com/lmmx/mibz2/pull/93 (PR body, current text))

**same_quantity_facts:** 8 matching lines in 6 documents dated before commit_to_build, searched on `one pass|init pass|scatter|build stage` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "- the T-vector build (5.8 c/B: a scatter with 256 write streams);" — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md
- [2026-10-27T15:46Z] docs/journal/2026-10-27-literature-reading.md (doc): "- **Expected:** half the dependent loads at the same 4 B per row. The walk could go from ~12 toward ~6–7 c/B. The build gains a histogram pass (~1–2 c/B), and its scatter spreads over the bigrams in use rather than 256 counters." — https://github.com/lmmx/mibz2/blob/24169e657c1f/docs/journal/2026-10-27-literature-reading.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-pair-walk.md (doc): "- **The build** costs 3× ours. It's the same scatter, but keyed by bigram: 65,536 bucket" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-pair-walk.md
- [2026-10-29T07:13Z] docs/journal/2026-10-29-bz2ngrs.md (doc): "- Net of its count, bz2ngrs's scatter is about 0.96 ns/B against our whole build's" — https://github.com/lmmx/mibz2/blob/e0e6e4a47d90/docs/journal/2026-10-29-bz2ngrs.md
- [2026-10-29T07:34Z] docs/journal/2026-10-29-expert-review/libsais-unbwt-path.md (doc): "\| H2 \| **The instruction budget pays for itself only at high clock.** About 10.5 instructions per byte in the decode, plus about 2 extra T passes, a 65,536-stream scatter and 2 n-byte memcpys. If libsais is throughput-bound and mibz2 is latency-bound, the i9's higher clock favours libsais. A lowe…" — https://github.com/lmmx/mibz2/blob/311c76cf82a7/docs/journal/2026-10-29-expert-review/libsais-unbwt-path.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "\| E11 \| **The build in one pass**: the scatter writes `ll[s] \\| i << 8` and the init pass goes; the table is identical \| production's build writes L into the table, then reads each entry back to keep it. In a team (`coop_build`), a slot's line was usually written in phase 1 by another thread, a…" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #93, #114) / Optimiser: unrecorded (no comment from the role naming #93, #114)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## compact-psi-stage-a: A smaller ψ table alone to move the knee (compact-psi stage A)

- **PRs:** [#112](https://github.com/lmmx/mibz2/pull/112)
- **Journal sources:** docs/journal/2026-10-30-compact-psi-feasibility.md
- **i9 recipes:** none
- **Deciding machine:** i9-10900K (coop grid)
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; with the research chatbot (2026-10-29-frontier.md)
- **commit_to_build:** 2026-10-29T21:28Z, PR #112 first commit (author date) (https://github.com/lmmx/mibz2/pull/112/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.16; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The new kinds of `coop grid` arm: \| arm \| entry \| bytes a row \| a full block's table \| the byte written \| \|---\|---\|--:\|--:\|---\| \| `o` \| production's (the copy's control) \| 4 \| 3.6 MB \| from the entry \| \| `v` \| a 32-bit link, alone \| 4 \| 3.6 MB \| from F \| \| `u` \| a 24-bit l…" (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- "**Does the knee move?** u6 − p6 ≥ 0.05 and u8 − p8 ≥ 0.077 in L3 hit share, and `u` peaks at more blocks than `p`. 2." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- "**The VM's first look (indicative only):** its 4 MB L2 nearly holds a 3.6 MB table, so this is the compact table's worst case." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))

**must_hold** (quoted):

- "- **New recipe:** `just bench-compact-psi`, the i9 grid plus counters, about 10 minutes. ## Correctness - **Every arm matches production on every block**, checked by hash in the grid's round −1." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- "**What does it cost?** u1/o1, v1/o1 and u1/v1, paired at one block." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- "Reported, not gated. 3." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- "**Does it beat production's peak?** `u`'s best configuration against `p4`: a paired geometric mean ≥ 1.05, with its lower bound > 1.00. 4." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-29T21:44Z] "The footprint run there showed that smaller *blocks* extend the knee." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- [2026-10-29T21:44Z] "This tests whether a smaller *table* alone does, on the same blocks. ## What it adds `examples/coop/compact.rs`: production's build and room walk, copied into the harness and made generic over a row's entry." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- [2026-10-29T21:44Z] "The copy is needed because the library keeps its walk's scratch and table private." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- [2026-10-29T21:44Z] "- **A deliberately broken `u` build was caught:** it used wide stores at bucket ends, and the corrupted table made the walk run away until its memory cap stopped it. ## Pre-registered (docs/journal/2026-10-29-compact-psi.md) The predictions come from the footprint run's hit-share curve in live table bytes: 1." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))
- [2026-10-29T21:44Z] "- `v`'s build is 1.55× production's, while recovering the byte costs about 14% of the walk." (https://github.com/lmmx/mibz2/pull/112 (PR body, current text))

**same_quantity_facts:** 15 matching lines in 7 documents dated before commit_to_build, searched on `smaller table|compact|knee|24.bit|table bytes` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- **Calibrate instead of trusting sysfs.** Before each bench, run a sub-second single-thread dependent pointer-chase over working sets of 256 KB→64 MB (×√2 steps). The latency knees give an *effective* L2 and L3 size as seen at that moment, neighbours included. This is the Pirate's self-verificatio…" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "would have room for about 1.6× at the knee, an upper bound, since the other block" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "\| E9 \| The walk's output in L's freed buffer \| memory reuse; its effect on the knee is unknown (`2026-10-29-bz2ngrs.md` §4) \| the room walk (research) \| ~10 min \| as the room walk's oracle and Miri \| DRAM write bytes per output byte \| **pass:** occupancy curve at 6–10 workers ≥ 1.03 × today…" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md
- [2026-10-29T15:55Z] docs/journal/2026-10-29-literature-study/00-overview-research.md (doc): "Construction #81 improved cooperative build by 31.3% and build+walk by 10.2% in the reported 3×3 experiment Whether directly constructing compact Ψ can beat that stronger baseline" — https://github.com/lmmx/mibz2/blob/1e98588115c7/docs/journal/2026-10-29-literature-study/00-overview-research.md
- [2026-10-29T16:05Z] docs/journal/2026-10-29-literature-study/09-cacheline-ef-research.md (doc): "The paper reports a 26.5% reduction in rank-metadata overhead relative to the selected compact baselines, while attempting to limit the resulting performance cost. DROPS" — https://github.com/lmmx/mibz2/blob/cf530594dd27/docs/journal/2026-10-29-literature-study/09-cacheline-ef-research.md
- [2026-10-29T16:45Z] docs/journal/2026-10-29-frontier.md (doc): "**What series 2 already shows, with its limits stated.** In throughput against live table bytes, y3x3 (1657 MB/s from 10.8 MB of live tables) beats p4 (1206 MB/s from 14.4 MB). p4 is production's best worker count on the i9 in every scaling run. So at the inversion's level, sharing a table among a …" — https://github.com/lmmx/mibz2/blob/ae7bc3099933/docs/journal/2026-10-29-frontier.md
- [2026-10-29T16:45Z] docs/journal/2026-10-29-frontier/seward-frontier-research.md (doc): "A compact representation could bring six simultaneously active full-block tables down from 21.6 MB to approximately 10.9 MB." — https://github.com/lmmx/mibz2/blob/ae7bc3099933/docs/journal/2026-10-29-frontier/seward-frontier-research.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #112) / Optimiser: unrecorded (no comment from the role naming #112)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## level-aware-plan: Plan by the first stream's level (rule R1): teams of 2 on every core for levels 1-3

- **PRs:** [#113](https://github.com/lmmx/mibz2/pull/113), [#115](https://github.com/lmmx/mibz2/pull/115)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; level-policy measurement #113
- **commit_to_build:** 2026-10-29T21:47Z, PR #113 first commit (author date) (https://github.com/lmmx/mibz2/pull/113/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 10 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.13; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "`Plan::for_threads` caps teams of 2 at round(0.7 × L3 / 3.6 MB), which is 4 teams on the i9. 3.6 MB is a full 900 K-row block's table." (https://github.com/lmmx/mibz2/pull/113 (PR body, current text))
- "A file at level n has tables of about n × 0.4 MB, and the knee journal's footprint run (#107) showed that one core a block keeps scaling to 8 blocks at bzip2 -3." (https://github.com/lmmx/mibz2/pull/113 (PR body, current text))
- "**The candidate rule is fixed now, so the data can't choose it.** The cap becomes round(0.7 × L3 / (level × 0.4 MB)), still at most half the physical cores, with the level taken from the first stream's header." (https://github.com/lmmx/mibz2/pull/113 (PR body, current text))

**must_hold** (quoted):

- "The release-gate A/B on bzip2 -3 and -6 recompressions, the native corpus as control; just bench-level-policy" (https://github.com/lmmx/mibz2/pull/113 (first commit message))
- "`Plan::for_threads` caps teams of 2 at round(0.7 × L3 / 3.6 MB), which is 4 teams on the i9. 3.6 MB is a full 900 K-row block's table." (https://github.com/lmmx/mibz2/pull/113 (PR body, current text))
- "- **`just bench-level-policy`:** - the release-gate A/B (`bench/coop_decoder.py`): the whole decoder, 20 rounds, unpinned, the same output hash on every run;" (https://github.com/lmmx/mibz2/pull/113 (PR body, current text))
- "- with `mibz2-workers`, `mibz2-coop2` and `mibz2-auto` at 4, 6, 8, 10 and all CPUs. ## The decision rule (fixed before the run) **A level-aware cap is proposed** if, at -3 or -6, five teams at 10 threads, or the best arm at the default thread count, beats today's policy there:" (https://github.com/lmmx/mibz2/pull/113 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 21 matching lines in 5 documents dated before commit_to_build, searched on `level [1-9]|small.block|team cap|levels? 1.3|-[1-9] ` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "schema and no Arrow output. A SIMD stage 1 costs 0.6 and UTF-8 validation 0.22." — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md
- [2026-10-27T09:44Z] docs/journal/2026-10-27-bz2-bakeoff.md (doc): "4.6 MB at level 9." — https://github.com/lmmx/mibz2/blob/734f9da49a33/docs/journal/2026-10-27-bz2-bakeoff.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- The MRC is **linear, not a cliff**, so misses begin before N·W reaches C. That matches your 1.48× CPU amplification at 2→4 workers at -9 (aggregate 7–14 MB < 20 MB): inclusive L3, other buffers and imperfect LRU already push C_eff below C." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "A scanner cap sits above it for small blocks. On this machine mibz2 at 4 workers is 2.1× its" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-10-29T07:34Z] docs/journal/2026-10-29-expert-review/libsais-unbwt-path.md (doc): "\| **P = A** \| (n+1) × 4 B = 3.6 MB \| Entry `P[q]` for 1-based row q is `(i << 8) \\| c`, written at L11393-11474 as `P[bucket2[w]++] = (i<<8) + c`. Here `i = PSI²(q)` (the row two steps forward) and `c = L[i] = F[PSI(q)]` (the middle symbol). **Inference from the construction:** the position `bu…" — https://github.com/lmmx/mibz2/blob/311c76cf82a7/docs/journal/2026-10-29-expert-review/libsais-unbwt-path.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #113, #115) / Optimiser: unrecorded (no comment from the role naming #113, #115)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## p74-register-mtf: P74 Gate 1: register-held MTF head against the shipped kernels

- **PRs:** [#118](https://github.com/lmmx/mibz2/pull/118)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI (EPYC 9V74 and others)
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; P74 from the external reports (Astra)
- **commit_to_build:** 2026-10-30T15:03Z, PR #118 first commit (author date) (https://github.com/lmmx/mibz2/pull/118/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "LLVM had turned the checkpoint kernels' lower tail chunks into a `memmove` call. ## The decision rules (registered) **First measurement:** - **Positive:** the better arm improves Wikidata's checkpoint kernel by ≥ 5%, with an interval excluding zero, by more than twice the layout control's noise; an…" (https://github.com/lmmx/mibz2/pull/118 (PR body, current text))

**must_hold** (quoted):

- "- bench/stages.py records them (`arms`, `--mtf-arms`); bench/p74.py computes the paired per-round ratios, 95% t-intervals, the layout control and the registered rules." (https://github.com/lmmx/mibz2/pull/118 (first commit message))
- "**A research measurement, not for merge.** P74 Gate 1 is authorised and frozen in `docs/journal/2026-10-30-p74-gate1-preregistration.md` (#117, commit 0e3ca63)." (https://github.com/lmmx/mibz2/pull/118 (PR body, current text))
- "It is authorised, and its rule was frozen before its code (#117 at 50ff284):" (https://github.com/lmmx/mibz2/pull/118 (PR body, current text))
- "`bench/p74.py` applies the registered rules and appends a table to the bench comment. ## Checks before measuring - **Tests** (`tests/mtf_p74.rs`): both arms against the scalar kernels on the edge cases at every limit, the samples, the corpora, runs on checkpoints, every alphabet size 1–256, each ra…" (https://github.com/lmmx/mibz2/pull/118 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-10-30T15:04Z] "It is saved only around `memset` for runs over 32 bytes and around checkpoint snapshots." (https://github.com/lmmx/mibz2/pull/118 (PR body, current text))

**same_quantity_facts:** 18 matching lines in 10 documents dated before commit_to_build, searched on `move.to.front|MTF|register.held` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T01:46Z] docs/journal/2026-10-27-compute-frontier.md (doc): "\| 1 \| **bz2 decoder built for memory-level parallelism** \| the walk in 12 joined chains; PCLMULQDQ CRC; AVX2 RLE1 and MTF \| **1.76× measured** (42.7 → 24.3 c/B, byte-identical); ceiling ~2.8× \| 53.5% of the split \| **−46 min** measured (6h51m → 6h05m, −11%); −69 min ceiling; −59/−88 by the re…" — https://github.com/lmmx/mibz2/blob/b221b6a7eafc/docs/journal/2026-10-27-compute-frontier.md
- [2026-10-27T15:46Z] docs/journal/2026-10-27-literature-reading.md (doc): "Decode Huffman → RLE2 → MTF straight into the T-vector's (or P's) words. This removes the symbols buffer (1.8 MB a slot) and the last-column copy (0.9 MB a worker)." — https://github.com/lmmx/mibz2/blob/24169e657c1f/docs/journal/2026-10-27-literature-reading.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "On Wikidata, 98.5% of moves already take the AVX2 path. The whole MTF + RLE2 stage is" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-27T20:38Z] docs/journal/2026-10-27-walk-sweep.md (doc): "segment buffers and 0.9 MB each of MTF output and walk output) approach a 32 MB L3," — https://github.com/lmmx/mibz2/blob/10e9761b63cb/docs/journal/2026-10-27-walk-sweep.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "walk is ×9.9 and the build ×9.0, while MTF + RLE2 is ×1.8 and the scanner ×1.45." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "\| E5 \| **The parallel build without its count pass**: MTF already counts the block's bytes, so let it also snapshot the counts at the k cut points as it writes L. A team's threads then scatter at once. \| the build is 42% of 3x3's time; libsais's own parallel build also recounts (L11572) \| MTF a…" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md
- [2026-10-29T16:31Z] docs/journal/2026-10-29-marker-audit.md (doc): "- They are cumulative byte counts every 16 KiB of L, made by `mtf_cp` and `mtf_cp2`" — https://github.com/lmmx/mibz2/blob/4de4712f1abf/docs/journal/2026-10-29-marker-audit.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder.md (doc): "- PR 0 measured 4 × 2 at 1.439× p4 for MTF, build and walk, pinned (coop journal §17.4)." — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder.md
- [2026-10-29T23:34Z] docs/journal/2026-10-29-one-pass-build.md (doc): "- **The VM's noise:** MTF, whose code is unchanged, read 1.052, so differences of about 5%" — https://github.com/lmmx/mibz2/blob/1291566fdb5a/docs/journal/2026-10-29-one-pass-build.md
- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "- **The kernel-with-MTF at 1,707 MB/s (pinned) against the decoder's 1,256" — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #118) / Optimiser: unrecorded (no comment from the role naming #118)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## for14-records: FOR-14/raw fixed records (P40/P38/P84), stage 1

- **PRs:** [#130](https://github.com/lmmx/mibz2/pull/130)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** i9-10900K (frozen pin 7d7962f)
- **Origin (as recorded):** Astra's report 131 (external reports), authorised in report 133
- **commit_to_build:** 2026-11-01T22:52Z, PR #130 opened (https://github.com/lmmx/mibz2/pull/130)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 5 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.00; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The pin's own analysis, replayed on the uploaded `.jsonl`, reproduces `for14-analysis.json` exactly. \| figure \| measured \| registered \| \|---\|--:\|---\| \| G1 calibration: o1/p1, o4/p4 (walk) \| 1.005, 0.991 \| holds \| \| **W1 = h1/o1 (walk)** \| **0.475 [0.473, 0.477]** \| STOP if the upper …" (https://github.com/lmmx/mibz2/pull/130 (PR body, current text))
- "- o's walk loses 56% and h's only 8%, so the half-sized table removes 69% of the four-block contention penalty;" (https://github.com/lmmx/mibz2/pull/130 (PR body, current text))
- "- but h costs 1.40 ns per row more at one block, three times that saving. ## Packed-token check (reports 138–139; compile and correctness only, §8a) Two new arms with their own identities test the packed lane token (`row \| carry << 20`, next token = header low word + offset) and masked addressing:" (https://github.com/lmmx/mibz2/pull/130 (PR body, current text))

**must_hold** (quoted):

- "New lNxK arms call bwt::Cooperative<K> through InverseBwt::invert_checkpointed, the worker pinned to its k CPUs so the library's helpers inherit them; the checkpoints given are checked to be the library's own MTF's on every block." (https://github.com/lmmx/mibz2/pull/130 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-01T22:52Z] "- o's walk loses 56% and h's only 8%, so the half-sized table removes 69% of the four-block contention penalty;" (https://github.com/lmmx/mibz2/pull/130 (PR body, current text))

**same_quantity_facts:** 3 matching lines in 3 documents dated before commit_to_build, searched on `FOR.14|fixed records|raw records` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-29T16:03Z] docs/journal/2026-10-29-literature-study/08-patched-coding-plateaus-research.md (doc): "It may be more informative than simply classifying the 22–23% nontranslation population as FOR-14 or EF-12." — https://github.com/lmmx/mibz2/blob/62549a3c4857/docs/journal/2026-10-29-literature-study/08-patched-coding-plateaus-research.md
- [2026-10-29T16:32Z] docs/journal/2026-10-29-literature-study.md (doc): "- **Measured here:** translation coverage (about 77%); FOR-14 and EF-12 cover 99.5% of" — https://github.com/lmmx/mibz2/blob/d4d4b782569b/docs/journal/2026-10-29-literature-study.md
- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "\| 64-byte line records (FOR-14 / EF-12 / exceptions), 16.12 bits a row \| 1.81 MB \| 1 line fetch, a decode on the chain (about 15 cycles, the guardian's estimate); byte from the header \| census: 98.1% of rows FOR-14, 0.49% exceptions; **untimed (#130 `h`)** \|" — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #130) / Optimiser: unrecorded (no comment from the role naming #130)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work

**Notes:** PR #130's earliest listed commit is 2026-10-29T18:30Z, 76 h before it opened; it may belong to a base branch, so it is not used.

## p87-in-place: P87: Segmented inverts in place over ll; rle is never allocated

- **PRs:** [#122](https://github.com/lmmx/mibz2/pull/122), [#128](https://github.com/lmmx/mibz2/pull/128), [#164](https://github.com/lmmx/mibz2/pull/164)
- **Journal sources:** docs/journal/2026-11-02-p87-production-integration.md, docs/journal/2026-11-05-p87-production-validation.md
- **i9 recipes:** none
- **Deciding machine:** i9-10900K (four-core diagnostic) then CI
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; P87 from the external reports
- **commit_to_build:** 2026-11-01T17:50Z, PR #122 first commit (author date) (https://github.com/lmmx/mibz2/pull/122/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 10 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 3.28; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "RSS unchanged \| \| Wikidata, 1 worker, B/A \| no measurable change \| \| Steps at 4 workers (traced, perturbation ≤ 2.4%) \| assembly saving resolved (0.147 → 0.120 CPU-s/GB); traversal +0.036 [−0.004, +0.076], not resolved;" (https://github.com/lmmx/mibz2/pull/122 (PR body, current text))
- "So a `Segmented` worker never allocates `rle`: about one block less per worker (0.9 MB at level 9)." (https://github.com/lmmx/mibz2/pull/128 (PR body, current text))
- "**Measured so far: instructions.** On the seven evidence blocks, Callgrind on `decode_block` puts this branch below master on every block, by 270 to 542 instructions (the removed allocation's first growth), with identical output." (https://github.com/lmmx/mibz2/pull/164 (PR body, current text))

**must_hold** (quoted):

- "**A research diagnostic, not for merge.** It ran once on the i9-10900K at the frozen pin `6463e44`." (https://github.com/lmmx/mibz2/pull/122 (PR body, current text))
- "**Stages.** - **S1:** the paired trial; 3 corpora × w = 1 and 4 × A, B and A′; 20 rounds." (https://github.com/lmmx/mibz2/pull/122 (PR body, current text))
- "- **S2:** the phases, traced and paired against the trace off." (https://github.com/lmmx/mibz2/pull/122 (PR body, current text))
- "That trial's protocol, harness and freeze are not written yet, and no i9 run is authorised. ## The change - **New trait method:** `InverseBwt::invert_in_place(&mut self, ll: &mut Vec<u8>, counts, orig_ptr) -> bool`." (https://github.com/lmmx/mibz2/pull/128 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-01T17:57Z] "Any follow-up would be a separately registered trial." (https://github.com/lmmx/mibz2/pull/122 (PR body, current text))
- [2026-11-01T17:57Z] "- The branch's runner now refuses to start, because two sources differ from the frozen hashes." (https://github.com/lmmx/mibz2/pull/122 (PR body, current text))
- [2026-11-05T00:15Z] "- **Power, stated in advance:** control intervals are ±1–5%, and smaller harm can't be excluded." (https://github.com/lmmx/mibz2/pull/164 (PR body, current text))
- [2026-11-05T00:15Z] "**Measured so far: instructions.** On the seven evidence blocks, Callgrind on `decode_block` puts this branch below master on every block, by 270 to 542 instructions (the removed allocation's first growth), with identical output." (https://github.com/lmmx/mibz2/pull/164 (PR body, current text))

**same_quantity_facts:** 36 matching lines in 17 documents dated before commit_to_build, searched on `in place|dead ll|buffer reuse|rle is never|peak RSS` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T17:50Z] docs/journal/2026-10-27-pair-walk.md (doc): "Peak RSS is up 1–8%. The branch `perf/pair-walk` is kept as the record." — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-pair-walk.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "\| L407: markers replaced in place, Silesia +11%, FASTQ +36% \| Deflate's 16-bit marker buffer reinterpreted as 8-bit \| the T-vector built over the MTF output (#8) \| see 3 \|" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-27T21:03Z] docs/journal/2026-10-27-output-side.md (doc): "- **Believed:** a 0.6 MB run-based move structure in place of the 3.6 MB T-vector would" — https://github.com/lmmx/mibz2/blob/2180ae5ee6a2/docs/journal/2026-10-27-output-side.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning.md (doc): "most, against about 111 MB reserved and 74 MB peak RSS." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning.md
- [2026-10-29T07:13Z] docs/journal/2026-10-29-bz2ngrs.md (doc): "- **Memory:** bz2ngrs's peak RSS is 0.45 × mibz2's at 1 thread on Wikidata (12 MB" — https://github.com/lmmx/mibz2/blob/e0e6e4a47d90/docs/journal/2026-10-29-bz2ngrs.md
- [2026-10-29T16:13Z] docs/journal/2026-10-29-literature-study/10-synthesis-coordinator.md (doc): "I approve proceeding with the cooperative-decoder integration programme. Use your proposed ≥1.15× full-decoder speedup with 95% lower bound above 1.05× as the headline success target, plus identical output, peak RSS ≤1.15× and CPU ≤1.5×. Please also report smaller reproducible end-to-end improvemen…" — https://github.com/lmmx/mibz2/blob/918f78892dcf/docs/journal/2026-10-29-literature-study/10-synthesis-coordinator.md
- [2026-10-29T16:13Z] docs/journal/2026-10-29-literature-study/11-closing.md (doc): "Please proceed with the cooperative-decoder production integration as the main priority. I approve using the proposed ≥1.15× full-decoder gain as the headline success target, with identical output, ≤1.15× peak RSS and ≤1.5× CPU. Report smaller reproducible gains too, rather than rejecting them auto…" — https://github.com/lmmx/mibz2/blob/918f78892dcf/docs/journal/2026-10-29-literature-study/11-closing.md
- [2026-10-29T18:03Z] docs/journal/2026-10-29-cooperative-decoder/coop-decoder-20261029T174657Z.md (doc): "\| arm \| T \| MB/s \| paired ratio (95%) \| won \| peak RSS \| RSS ratio \| CPU-s \| CPU ratio \|" — https://github.com/lmmx/mibz2/blob/d6d4cac5ad49/docs/journal/2026-10-29-cooperative-decoder/coop-decoder-20261029T174657Z.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder.md (doc): "- **At 6 threads** the default gives 1,071 MB/s, 1.217× (1.178–1.258), at 60 MB peak RSS (0.83×)" — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder/coop-decoder-20261029T181511Z.md (doc): "\| arm \| T \| MB/s \| paired ratio (95%) \| won \| peak RSS \| RSS ratio \| CPU-s \| CPU ratio \|" — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder/coop-decoder-20261029T181511Z.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6645002054 (2026-11-04T23:26Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #122, #128, #164)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

**Notes:** PR #128's earliest listed commit is 2026-11-01T21:32Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #164's earliest listed commit is 2026-11-01T21:32Z, 75 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## compact30-default: compact30 record kernel replaces the T-vector kernel as the default

- **PRs:** [#146](https://github.com/lmmx/mibz2/pull/146), [#151](https://github.com/lmmx/mibz2/pull/151), [#155](https://github.com/lmmx/mibz2/pull/155)
- **Journal sources:** docs/journal/2026-11-04-compact30-research-charter.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** compact30 research campaign (charter revision 4; reports 217-259)
- **commit_to_build:** 2026-11-04T11:04Z, PR #146 first commit (author date) (https://github.com/lmmx/mibz2/pull/146/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.40; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Reads −3.39 M, writes +1.99 M. \| **How gate 3 was counted.** The net figure is callgrind's instruction count of each `build::<false>` over the primary corpus, used as an exact whole-function recount. callgrind simulates execution: nothing was timed, and no hardware performance counters were used." (https://github.com/lmmx/mibz2/pull/146 (PR body, current text))
- "research(compact30): prototype 2 (hot descriptor pointer arrays), pre-registered under the charter The compile targets: the scalar append at 10 instructions or fewer in all four copies, with no added per-row load, store or reload, and at most 5 M of added non-row work." (https://github.com/lmmx/mibz2/pull/151 (first commit message))
- "All 9 registered Miri classes completed, with no undefined behaviour. \| \| 2, compiled counts (diagnostic) \| **Partial miss.** Every scalar copy is 10 instructions, with no spill or reload." (https://github.com/lmmx/mibz2/pull/151 (PR body, current text))

**must_hold** (quoted):

- "Reads −3.39 M, writes +1.99 M. \| **How gate 3 was counted.** The net figure is callgrind's instruction count of each `build::<false>` over the primary corpus, used as an exact whole-function recount. callgrind simulates execution: nothing was timed, and no hardware performance counters were used." (https://github.com/lmmx/mibz2/pull/146 (PR body, current text))
- "The nine Miri classes that must be reached, the two mutation checks, and the rejection conditions." (https://github.com/lmmx/mibz2/pull/151 (first commit message))
- "It is not to merge until CI's paired A/B bench rules.** This ports the compact30 record kernel (research Assembly 1:" (https://github.com/lmmx/mibz2/pull/155 (PR body, current text))
- "The construction ledger never priced this term. ## Local timing (shared VM, indicative only) On 473 Wikidata blocks, paired and interleaved, 12 rounds, the branch/master ratios are:" (https://github.com/lmmx/mibz2/pull/155 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-04T11:51Z] "research(compact30): prototype 2 (hot descriptor pointer arrays), pre-registered under the charter The compile targets: the scalar append at 10 instructions or fewer in all four copies, with no added per-row load, store or reload, and at most 5 M of added non-row work." (https://github.com/lmmx/mibz2/pull/151 (first commit message))
- [2026-11-04T20:36Z] "The record walk costs about 5.3 more instructions per row, which is more than the constructor saves." (https://github.com/lmmx/mibz2/pull/155 (PR body, current text))
- [2026-11-04T20:36Z] "The construction ledger never priced this term. ## Local timing (shared VM, indicative only) On 473 Wikidata blocks, paired and interleaved, 12 rounds, the branch/master ratios are:" (https://github.com/lmmx/mibz2/pull/155 (PR body, current text))
- [2026-11-04T20:36Z] "The hypothesis is cache residency: the records take 2.1 MB per block, against 3.6 MB for the T-vector." (https://github.com/lmmx/mibz2/pull/155 (PR body, current text))
- [2026-11-04T20:36Z] "- The footprint ledger (`src/trace.rs`, `docs/ledger.md`) still models the T-vector." (https://github.com/lmmx/mibz2/pull/155 (PR body, current text))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `compact30|30.entry|record kernel|Ctor2` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 4 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6630137149 (2026-11-04T12:05Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #146, #151, #155)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6629821793

## compact30-staged: compact30 under the staged walk (C-resolve)

- **PRs:** [#195](https://github.com/lmmx/mibz2/pull/195)
- **Journal sources:** docs/journal/2026-11-05-compact30-staged.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** the Optimiser's proposal, REVIEW 6653724458 (per PR #195)
- **commit_to_build:** 2026-11-05T10:58Z, PR #195 first commit (author date) (https://github.com/lmmx/mibz2/pull/195/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.12; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "On CI the four-copy walk is mostly a per-byte core term, and C adds 4 instructions and a load to every step." (https://github.com/lmmx/mibz2/pull/195 (PR body, current text))

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T10:58Z] "- a RAW arrival (field 0xFFFF) saves its entry's index in pend[k], prefetches it and emits nothing in its turn, as the port's did; after every lane has stepped offset j, and before any handover, a resolve class reads each such entry and stages its byte at column j (the column the arrival left empty) and its successor as the lane's row." (https://github.com/lmmx/mibz2/pull/195 (first commit message))
- [2026-11-05T10:58Z] "- the take is the port's (each start's saved successor and byte), with the record prefetch." (https://github.com/lmmx/mibz2/pull/195 (first commit message))
- [2026-11-05T13:29Z] "This tests compact30's smaller table under today's staged walk, as the Optimiser proposed in REVIEW 6653724458." (https://github.com/lmmx/mibz2/pull/195 (PR body, current text))

**same_quantity_facts:** 9 matching lines in 8 documents dated before commit_to_build, searched on `compact30|smaller table|staged walk` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "- A fifth team is the most a smaller table can add: at most 5/4 = 1.25× on the" — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md
- [2026-11-04T12:12Z] comment 6630279852 (#145 researcher): "**PROPOSAL, outside the registered sequence:** [direct interior refill](https://github.com/lmmx/mibz2/blob/0ac2e67e6826b9f2b1940cb876c3003e7a022144/docs/journal/compact30-research/proposals/refill-boundary/proposal.md). A completed crossing/wide predecessor can also advance directly to a full inter…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6630279852
- [2026-11-04T15:16Z] comment 6634356048 (#145 s62-reviewer): "- **S62-F04 — Low; material: no at the observed bound.** Each harness's caller adds 868 instructions. Document the caller boundary and archive its call-site listing; charging the complete observed delta leaves both numerical intervals above their minima. [Finding](https://github.com/lmmx/mibz2/blob…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6634356048
- [2026-11-04T19:41Z] comment 6640324039 (#145 coordinator): "Decoder CPU ratio R = (1.847·w + 0.592·b) / 2.439, from the post's §9 per-GB constants (build 0.592, walk + assembly 1.847 CPU-s/GB). w = compact30 walk ratio from the validated traversal captures (one core 1.148; four workers 0.840). b = combined build ratio, unmeasured." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6640324039
- [2026-11-04T20:52Z] comment 6641832358 (#145 researcher): "**Additional finding:** at b4786f66, `examples/phases.rs` still directly calls legacy `bwt::build` and `walk_segments_room_parts` (unsplit check: `walk_segments_room`), while production Scratch defaults to Compact30. The phase table's walk −0.1% and build +4.7% therefore do **not** measure compact3…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6641832358
- [2026-11-04T22:37Z] comment 6643931565 (#145 researcher): "A smaller table can reduce resident lines, but must repay added dependent accesses/extraction, construction and tails. The published small-table #42 single-chain win/32-lane loss is the warning. Use the existing controlled-neighbour design to distinguish footprint, request and write sensitivity if …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643931565
- [2026-11-04T23:01Z] comment 6644364456 (#145 coordinator): "1. **compact30 in production failed, #155 closed.** Build −1.06 M instructions per block, record walk +4.78 M, whole +14.7%. CI on the 7763: +18.7%, +11.0% and +7.3% wall at 1, 2 and 4 threads. Evidence: research/compact30-channel c493a45d `evidence/port-b4786f66/`." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6644364456
- [2026-11-05T06:03Z] comment 6651023083 (#160 coordinator): "- **Re-price compact-table parity.** The 46.25% walk-Ir break-even is the wrong metric. Restate it in lines fetched at random per row: 4 B per entry against compact30's 64 B per 30 rows. Include the builder's own traffic." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #195) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6660203300 (2026-11-05T14:14Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work

## room-walk-eight-lanes: Room walk's pass steps eight lanes a turn, handovers after

- **PRs:** [#157](https://github.com/lmmx/mibz2/pull/157)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-04T22:13Z, PR #157 first commit (author date) (https://github.com/lmmx/mibz2/pull/157/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "With the handover out of it, the step is unrolled by hand, eight lanes a turn: 10 instructions per step plus the turn's control." (https://github.com/lmmx/mibz2/pull/157 (first commit message))
- "Callgrind, decode_block on full Wikidata blocks: 18.4% to 19.2% fewer instructions (block 1: 25,751,810 -> 21,026,251)." (https://github.com/lmmx/mibz2/pull/157 (first commit message))
- "The multi-thread regime is the subject of the [memory interference](https://github.com/lmmx/mibz2/blob/master/docs/site/blog/2026-11-01-inverse-bwt-memory-interference.md) post. ## What changed Production's room walk (`room_segments`, the default `Segmented` kernel) stepped its 32 lanes in a loop t…" (https://github.com/lmmx/mibz2/pull/157 (PR body, current text))

**must_hold** (quoted):

- "CI's paired A/B bench decides." (https://github.com/lmmx/mibz2/pull/157 (PR body, current text))
- "Instruction parity is a screen, not the merge criterion." (https://github.com/lmmx/mibz2/pull/157 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-04T22:13Z] "Callgrind, decode_block on full Wikidata blocks: 18.4% to 19.2% fewer instructions (block 1: 25,751,810 -> 21,026,251)." (https://github.com/lmmx/mibz2/pull/157 (first commit message))
- [2026-11-04T22:13Z] "So it removes per-step bookkeeping while keeping all 32 chains in one round." (https://github.com/lmmx/mibz2/pull/157 (PR body, current text))
- [2026-11-04T22:13Z] "The post estimated about 11% of decoder time." (https://github.com/lmmx/mibz2/pull/157 (PR body, current text))

**same_quantity_facts:** 18 matching lines in 4 documents dated before commit_to_build, searched on `eight lanes|handover|room walk` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T20:01Z] docs/journal/2026-10-27-move-structure-walk.md (doc): "shared machine, `walk_segments_room` measured 19.4 c/B, while the move room walk's" — https://github.com/lmmx/mibz2/blob/50d404c8bdd2/docs/journal/2026-10-27-move-structure-walk.md
- [2026-10-27T20:38Z] docs/journal/2026-10-27-walk-sweep.md (doc): "Locally, the room walk's row went from 2.85 to 4.06–4.36 ns/B, and its share of a decode" — https://github.com/lmmx/mibz2/blob/10e9761b63cb/docs/journal/2026-10-27-walk-sweep.md
- [2026-10-27T21:03Z] docs/journal/2026-10-27-output-side.md (doc): "2. #42's pre-registered contention test (≥10% under build + room walk per copy, with four" — https://github.com/lmmx/mibz2/blob/2180ae5ee6a2/docs/journal/2026-10-27-output-side.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "- **v0.1.5's room walk holds against v0.1.4** (1.05 at 4 threads, 1.001–1.101). master is" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 4 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6643931565 (2026-11-04T22:37Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #157)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6643481606, https://github.com/lmmx/mibz2/pull/145#issuecomment-6643744616, https://github.com/lmmx/mibz2/pull/145#issuecomment-6643809123

## mtf-short-runs: MTF run of at most 64 bytes as two vector stores, not memset

- **PRs:** [#170](https://github.com/lmmx/mibz2/pull/170)
- **Journal sources:** docs/journal/2026-11-05-mtf-short-runs.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T01:58Z, PR #170 first commit (author date) (https://github.com/lmmx/mibz2/pull/170/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "-270k to -330k Ir on every full block (-1.4 to -1.65%), output identical." (https://github.com/lmmx/mibz2/pull/170 (first commit message))
- "**`decode_block` Callgrind,** seven evidence blocks, the same driver for both arms: \| block \| master \| this PR \| change \| \|---\|--:\|--:\|--:\| \| 1 \| 20,184,311 \| 19,857,878 \| −326,433 (−1.62%) \| \| 41 \| 19,480,138 \| 19,207,725 \| −272,413 (−1.40%) \| \| 233 \| 19,989,050 \| 19,659,043…" (https://github.com/lmmx/mibz2/pull/170 (PR body, current text))

**must_hold** (quoted):

- "The decision rule is registered in `docs/journal/2026-11-05-mtf-short-runs.md` and is the same as theirs." (https://github.com/lmmx/mibz2/pull/170 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 7 matching lines in 6 documents dated before commit_to_build, searched on `memset|short runs?|RLE2|run of` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T15:46Z] docs/journal/2026-10-27-literature-reading.md (doc): "Decode Huffman → RLE2 → MTF straight into the T-vector's (or P's) words. This removes the symbols buffer (1.8 MB a slot) and the last-column copy (0.9 MB a worker)." — https://github.com/lmmx/mibz2/blob/24169e657c1f/docs/journal/2026-10-27-literature-reading.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "On Wikidata, 98.5% of moves already take the AVX2 path. The whole MTF + RLE2 stage is" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- [I] **Does an LL model capture working-set growth?** Yes, for capacity. A single-thread run with `--LL` set to C/N (power-of-2 sets, e.g. 16-way 2 MiB, 4 MiB or 8 MiB) gives a deterministic **miss-ratio curve**. Footprint theory (§3) says N fairly interleaved copies each see about C/N. A run of N…" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "walk is ×9.9 and the build ×9.0, while MTF + RLE2 is ×1.8 and the scanner ×1.45." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md (doc): "\| MTF + RLE2 \| 2.24 → 2.24 \| 19.4% \| +0.4% (−0.5%, +1.2%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md (doc): "\| MTF + RLE2 \| 2.19 → 2.19 \| 18.7% \| −0.1% (−0.8%, +0.6%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #170) / Optimiser: unrecorded (no comment from the role naming #170)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## staged-walk-tile: Room walk tests flags once an offset and stages each chunk's bytes in one tile

- **PRs:** [#168](https://github.com/lmmx/mibz2/pull/168), [#171](https://github.com/lmmx/mibz2/pull/171)
- **Journal sources:** docs/journal/2026-11-05-deferred-flag-walk.md, docs/journal/2026-11-05-staged-walk.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T01:39Z, PR #168 opened (https://github.com/lmmx/mibz2/pull/168)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.07; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The step is 8 instructions with the 32 lanes unrolled, against 9 plus a turn every 8 lanes. decode_block on the seven evidence blocks:" (https://github.com/lmmx/mibz2/pull/168 (first commit message))
- "-508k to -512k Ir on every full block (-2.5 to -2.6%), short blocks unchanged, output identical." (https://github.com/lmmx/mibz2/pull/168 (first commit message))
- "The step is 8 instructions with the 32 lanes unrolled, against 9 plus a turn every 8 lanes. decode_block on the seven evidence blocks:" (https://github.com/lmmx/mibz2/pull/171 (first commit message))

**must_hold** (quoted):

- "**Decision rule,** registered before timing in `docs/journal/2026-11-05-deferred-flag-walk.md`:" (https://github.com/lmmx/mibz2/pull/168 (PR body, current text))
- "- **Benefit:** met in every run." (https://github.com/lmmx/mibz2/pull/171 (PR body, current text))
- "At 4 threads, the CPU spent for the same bytes fell the most on Zen 5 (amplification −17.5% and −16.6%), while 1 thread on that runner was unchanged." (https://github.com/lmmx/mibz2/pull/171 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T01:39Z] "Every later speculative round would then need a mask, which costs the instruction saved, and replays would rise from about 1.8% to about 13.6%." (https://github.com/lmmx/mibz2/pull/168 (PR body, current text))
- [2026-11-05T02:02Z] "That removes a pointer load from every step." (https://github.com/lmmx/mibz2/pull/171 (PR body, current text))
- [2026-11-05T02:02Z] "- Instructions don't support a staging cost on `runs`: on its single block, this PR runs 0.8% fewer instructions than master (17,756,248 against 17,904,270)." (https://github.com/lmmx/mibz2/pull/171 (PR body, current text))

**same_quantity_facts:** 29 matching lines in 15 documents dated before commit_to_build, searched on `tile|staged?|flags? once|deferred.flag` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-29T16:45Z] docs/journal/2026-10-29-frontier.md (doc): "**What series 2 already shows, with its limits stated.** In throughput against live table bytes, y3x3 (1657 MB/s from 10.8 MB of live tables) beats p4 (1206 MB/s from 14.4 MB). p4 is production's best worker count on the i9 in every scaling run. So at the inversion's level, sharing a table among a …" — https://github.com/lmmx/mibz2/blob/ae7bc3099933/docs/journal/2026-10-29-frontier.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder.md (doc): "- The full decoder unpinned gives 1.248 at 8 threads, so about 0.19 of it goes to the stages outside the inversion and to unpinned placement." — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder.md
- [2026-10-29T23:34Z] docs/journal/2026-10-29-one-pass-build.md (doc): "- **The build stage is faster:** −6.0% (−8.5%, −3.5%)." — https://github.com/lmmx/mibz2/blob/1291566fdb5a/docs/journal/2026-10-29-one-pass-build.md
- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "\| shipped kernel with its MTF stage timed, 4 × 2 \| 1,707 MB/s \| rematch pass 2 \|" — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md
- [2026-11-04T22:37Z] comment 6643931565 (#145 researcher): "**Tiling/gathers are not established traffic reductions.** B2 counts one random T load per byte for A, B and C. On its consistent 7763 baseline, scalar B predicts 9.9–11.0% decoder saving; C 12.2–18.2%, conditional on latency, buffer and tail assumptions. #157 is not that complete tiled-output kern…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643931565
- [2026-11-04T22:40Z] comment 6643992749 (#145 coordinator): "\| decoder, one thread \| walk 45% of time → about −3% to −6% \| −3.9% stage total, −4.2% wall \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643992749
- [2026-11-04T22:43Z] comment 6644051935 (#145 guardian): "- **Runs are mixed.** 4.04→3.78 c/B, −7.5% walk and −3.9% stage total are from the **7763 replicate**, run 41925226768, head 3fc3f8c. Its wall changes are −2.4%/−4.9%/+0.2% at 1/2/4 threads. The −4.2%/+2.2%/+0.4% figures belong to the earlier **9V45**, run 41923819666; its walk change was −5.8% [−1…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6644051935
- [2026-11-04T22:45Z] comment 6644080568 (#145 researcher): "**Two corrections precede fitting.** The .925 walk ratio and −3.9% stage total are the **7763 replicate 41925226768**, not 9V45. Its wall changes are −2.4%/−4.9%/+0.2%. The older 9V45 run has walk .942 [.847,1.049] and wall −4.2%/+2.2%/+0.4%. The phase row includes assembly; B2 excludes it. Paired …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6644080568
- [2026-11-04T22:47Z] comment 6644115799 (#145 coordinator): "\| decoder, one thread \| ≈ 0 \| **−10% to −11%** \| larger \| **−2.4%** (stage total −3.9%) \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6644115799
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation.md (doc): "- Run 1's stage total is also slower, +1.2% (+0.1, +2.4); run 2's isn't, −0.1% (−0.3, +0.2)." — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #168, #171) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6758406452 (2026-11-10T03:47Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

**Notes:** PR #168's earliest listed commit is 2026-11-05T01:39Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #171's earliest listed commit is 2026-11-05T01:39Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## build-run16-sse2: Build writes a run's 16 entries as four SSE2 vectors

- **PRs:** [#169](https://github.com/lmmx/mibz2/pull/169)
- **Journal sources:** docs/journal/2026-11-05-build-run-vectors.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T01:50Z, PR #169 first commit (author date) (https://github.com/lmmx/mibz2/pull/169/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "-468k to -480k Ir on every full block (-2.3 to -2.5%), build 5.43M -> 4.96M on block 1, output identical." (https://github.com/lmmx/mibz2/pull/169 (first commit message))
- "`t` might alias `ll`, so LLVM kept them scalar, at about 31 instructions a run." (https://github.com/lmmx/mibz2/pull/169 (PR body, current text))
- "**`decode_block` Callgrind,** seven evidence blocks, the same driver for both arms: \| block \| master \| this PR \| change \| \|---\|--:\|--:\|--:\| \| 1 \| 20,184,311 \| 19,716,020 \| −468,291 (−2.32%) \| \| 41 \| 19,480,138 \| 19,003,853 \| −476,285 (−2.44%) \| \| 233 \| 19,989,050 \| 19,509,457…" (https://github.com/lmmx/mibz2/pull/169 (PR body, current text))

**must_hold** (quoted):

- "**Decision rule,** registered in `docs/journal/2026-11-05-build-run-vectors.md`: the same as #168's." (https://github.com/lmmx/mibz2/pull/169 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 2 matching lines in 1 documents dated before commit_to_build, searched on `run16|16 entries|SSE2|build stage` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-29T23:34Z] docs/journal/2026-10-29-one-pass-build.md (doc): "- **The build stage is faster:** −6.0% (−8.5%, −3.5%)." — https://github.com/lmmx/mibz2/blob/1291566fdb5a/docs/journal/2026-10-29-one-pass-build.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #169) / Optimiser: unrecorded (no comment from the role naming #169)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## mtf-byte-list: MTF list holds the bytes, not their indices

- **PRs:** [#172](https://github.com/lmmx/mibz2/pull/172)
- **Journal sources:** docs/journal/2026-11-05-mtf-byte-list.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T02:18Z, PR #172 first commit (author date) (https://github.com/lmmx/mibz2/pull/172/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.08; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 21 matching lines in 13 documents dated before commit_to_build, searched on `move.to.front|MTF|indices` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T20:38Z] docs/journal/2026-10-27-walk-sweep.md (doc): "segment buffers and 0.9 MB each of MTF output and walk output) approach a 32 MB L3," — https://github.com/lmmx/mibz2/blob/10e9761b63cb/docs/journal/2026-10-27-walk-sweep.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "walk is ×9.9 and the build ×9.0, while MTF + RLE2 is ×1.8 and the scanner ×1.45." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "\| E5 \| **The parallel build without its count pass**: MTF already counts the block's bytes, so let it also snapshot the counts at the k cut points as it writes L. A team's threads then scatter at once. \| the build is 42% of 3x3's time; libsais's own parallel build also recounts (L11572) \| MTF a…" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md
- [2026-10-29T16:31Z] docs/journal/2026-10-29-marker-audit.md (doc): "- They are cumulative byte counts every 16 KiB of L, made by `mtf_cp` and `mtf_cp2`" — https://github.com/lmmx/mibz2/blob/4de4712f1abf/docs/journal/2026-10-29-marker-audit.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder.md (doc): "- PR 0 measured 4 × 2 at 1.439× p4 for MTF, build and walk, pinned (coop journal §17.4)." — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder.md
- [2026-10-29T23:34Z] docs/journal/2026-10-29-one-pass-build.md (doc): "- **The VM's noise:** MTF, whose code is unchanged, read 1.052, so differences of about 5%" — https://github.com/lmmx/mibz2/blob/1291566fdb5a/docs/journal/2026-10-29-one-pass-build.md
- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "- **The kernel-with-MTF at 1,707 MB/s (pinned) against the decoder's 1,256" — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md
- [2026-11-04T22:27Z] comment 6643744616 (#145 coordinator): "**The finding that redirects the work:** a 36% cut in walk instructions bought 4% of wall time at 1 thread and nothing at 2 or 4. At the thread counts the deployment uses, walk instructions do not bind. Something else does: memory latency or bandwidth on the chase, the serial scanner (0.11 s of 1.2…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643744616
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md (doc): "\| MTF + RLE2 \| 2.24 → 2.24 \| 19.4% \| +0.4% (−0.5%, +1.2%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md (doc): "\| MTF + RLE2 \| 2.19 → 2.19 \| 18.7% \| −0.1% (−0.8%, +0.6%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #172) / Optimiser: unrecorded (no comment from the role naming #172)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## tail-passes: Room walk's tail in deferred-flag passes

- **PRs:** [#173](https://github.com/lmmx/mibz2/pull/173)
- **Journal sources:** docs/journal/2026-11-05-tail-passes.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T02:09Z, PR #173 first commit (author date) (https://github.com/lmmx/mibz2/pull/173/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.09; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "No segment is taken in the tail, so the finishing order changes no byte. decode_block on block 1: 18,652,124 Ir (-399,625 over the staged walk, -7.59% against master), output identical on every block." (https://github.com/lmmx/mibz2/pull/173 (first commit message))
- "Each step went through a `Vec` of lanes, testing the entry's flag and the buffer's end, at about 18 instructions a step, against the busy phase's 7." (https://github.com/lmmx/mibz2/pull/173 (PR body, current text))
- "**`decode_block` instructions (Callgrind)**, against master 6b337819: \| block \| master \| this PR \| change \| \|---\|--:\|--:\|--:\| \| 1 \| 18,725,285 \| 18,325,660 \| −2.13% \| \| 41 \| 18,066,394 \| 17,770,269 \| −1.64% \| \| 233 \| 18,524,537 \| 18,209,467 \| −1.70% \| \| 400 \| 18,081,311 \…" (https://github.com/lmmx/mibz2/pull/173 (PR body, current text))

**must_hold** (quoted):

- "It is #171's rule, plus a power clause: the tail is about 2% of instructions and runs while fewer than 32 lanes are live, so a gain that size may be below the bench's noise." (https://github.com/lmmx/mibz2/pull/173 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T03:34Z] "It is #171's rule, plus a power clause: the tail is about 2% of instructions and runs while fewer than 32 lanes are live, so a gain that size may be below the bench's noise." (https://github.com/lmmx/mibz2/pull/173 (PR body, current text))

**same_quantity_facts:** 6 matching lines in 5 documents dated before commit_to_build, searched on `tail|deferred.flag|18 instructions` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T17:50Z] docs/journal/2026-10-27-external-report-1.md (doc): "A remarkably detailed record of actual optimisations. For example, removing unnecessary buffers improved bzip2 decompression from 344 to 388 MB/s in one reported test. Other changes reduced memory use, avoided extra allocations and improved Huffman decoding." — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-external-report-1.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- [S] Every 500 ms it computes per-app tail-latency slack. It upsizes an app if slack < 0.05 and downsizes the one with the most slack if slack > 0.2. Sensitivity was characterised with microbenchmarks that thrash memory bandwidth on idle cores." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-11-04T22:33Z] comment 6643850845 (#145 guardian): "All 14 raw profiles reconcile. D017's missing raw-profile/boundary concerns are closed for 75b13f00: full-block decode instructions fall 21.6–22.6%; build/rest match; PROGRAM TOTAL includes 16 driver instructions. Whole-walk charges include handovers/setup/tails. This is the newer pin, not a replac…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643850845
- [2026-11-04T22:37Z] comment 6643931565 (#145 researcher): "A smaller table can reduce resident lines, but must repay added dependent accesses/extraction, construction and tails. The published small-table #42 single-chain win/32-lane loss is the warning. Use the existing controlled-neighbour design to distinguish footprint, request and write sensitivity if …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643931565
- [2026-11-04T22:40Z] comment 6643992749 (#145 coordinator): "**#157 is B2's kernel B.** B2 is `docs/research-portfolio:docs/journal/2026-10-30-b2-gathered-walk.md`, read-only, 30 October. Kernel B is "the same 32 scalar chains with deferred flags and tiled commits", predicted at 8.0–9.0 µops/B. #157 compiles to 9 instructions per step. The instruction side h…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643992749

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #173) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6660203300 (2026-11-05T14:14Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## tile-size: Tile of 512 or 1024 offsets per lane (16 or 32 KiB)

- **PRs:** [#177](https://github.com/lmmx/mibz2/pull/177), [#178](https://github.com/lmmx/mibz2/pull/178)
- **Journal sources:** docs/journal/2026-11-05-tile-size.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T05:31Z, PR #177 opened (https://github.com/lmmx/mibz2/pull/177)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.05; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "**Instructions are flat:** `decode_block` is +0.05% on blocks 1, 41, 233 and 400." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))
- "`spec_offset` is the same 237 instructions." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))
- "Over 62 paired rounds against 256, wikidata ran −1.9% (−3.1, −0.6) at 1 thread and −2.0% (−3.6, −0.4) at 4." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))

**must_hold** (quoted):

- "Over 62 paired rounds against 256, wikidata ran −1.9% (−3.1, −0.6) at 1 thread and −2.0% (−3.6, −0.4) at 4." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))
- "If this arm harms, the prediction is that it harms at 4 threads on the 7763 first." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))
- "**Decision rule:** registered in `docs/journal/2026-11-05-tile-size.md` before this run." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))
- "It is #171's rule with two changes:" (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T05:31Z] "There are fewer chunk ends, but glibc's copy is cheapest per byte at exactly 256 bytes, which cancels the saving." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))
- [2026-11-05T05:31Z] "If this arm harms, the prediction is that it harms at 4 threads on the 7763 first." (https://github.com/lmmx/mibz2/pull/177 (PR body, current text))
- [2026-11-05T05:31Z] "On the EPYC 7763, two SMT siblings' tiles take twice that, so 4 threads on the 7763 is where this arm is predicted to lose if it loses." (https://github.com/lmmx/mibz2/pull/178 (PR body, current text))

**same_quantity_facts:** 30 matching lines in 10 documents dated before commit_to_build, searched on `tile|L1D?|TILE` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "\| L615: "Only create Huffman LUT up to max code length in block: 140 → 195 MB/s" \| Deflate \| mibz2's LUT is `LUT_BITS = 10`, two symbols per u32 (`src/block.rs:59`): 4 KB a table, ≤ 24 KB for 6 tables, L1-resident \| no experiment: the whole scanner is 5.4% \|" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-27T20:38Z] docs/journal/2026-10-27-walk-sweep.md (doc): "(−1.0%, −6.0% to +4.3%) and 24 lanes with L1 (+0.1%)." — https://github.com/lmmx/mibz2/blob/10e9761b63cb/docs/journal/2026-10-27-walk-sweep.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- [S] the CPU-simulation instrument estimates cycles ≈ instruction cycles + L1-miss × L2/L3 cost (10–40 cycles) + LL-miss × RAM cost (100+). Each benchmark runs once; syscall time is excluded; cache sizes are not published;" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-10-29T07:34Z] docs/journal/2026-10-29-expert-review/libsais-unbwt-path.md (doc): "\| H7 \| **TLB.** Random P accesses span about 880 × 4 KB pages, and every step likely misses the L1 DTLB; nested paging in a VM makes STLB misses and walks costlier (inference). \| Random `P[p]` at L12493 \| `dTLB-load-misses` / `ls_l1_d_tlb_miss.*`; rerun with THP-backed A (`madvise(MADV_HUGEPAGE…" — https://github.com/lmmx/mibz2/blob/311c76cf82a7/docs/journal/2026-10-29-expert-review/libsais-unbwt-path.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "\| E7 \| The compact 20-bit ψ (ll16 + ll4 or packed) under pressure \| 37.5% less table (2.5 B a row against 4, plus an n/32-byte first-column table) \| the parked pilot \| ~10 min \| its self-test \| L3 hit share \| **pass:** its best absolute at 6–10 workers > p4's best. **fail:** below \|" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md
- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "\| `u`: 24-bit link + F window \| 2.70 + 0.03 MB \| 1 load (unaligned, masked) + 2 L1 loads off the chain \| **measured: fails stage B** (u1/o1 0.681) \|" — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md
- [2026-11-04T22:37Z] comment 6643931565 (#145 researcher): "**Tiling/gathers are not established traffic reductions.** B2 counts one random T load per byte for A, B and C. On its consistent 7763 baseline, scalar B predicts 9.9–11.0% decoder saving; C 12.2–18.2%, conditional on latency, buffer and tail assumptions. #157 is not that complete tiled-output kern…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643931565
- [2026-11-04T22:40Z] comment 6643992749 (#145 coordinator): "**#157 is B2's kernel B.** B2 is `docs/research-portfolio:docs/journal/2026-10-30-b2-gathered-walk.md`, read-only, 30 October. Kernel B is "the same 32 scalar chains with deferred flags and tiled commits", predicted at 8.0–9.0 µops/B. #157 compiles to 9 instructions per step. The instruction side h…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643992749
- [2026-11-05T03:20Z] docs/journal/2026-11-05-deferred-flag-walk.md (doc): "1 − e^(−256/1758) ≈ 13.6% a tile instead of about 1.8% an offset. T = 1 needs neither the mask" — https://github.com/lmmx/mibz2/blob/6b337819344c/docs/journal/2026-11-05-deferred-flag-walk.md
- [2026-11-05T03:59Z] docs/journal/2026-11-05-staged-walk.md (doc): "bytes, 8 KiB in L1, and every lane addresses it from one base. That removes a pointer load" — https://github.com/lmmx/mibz2/blob/dfba89795e6e/docs/journal/2026-11-05-staged-walk.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #177, #178) / Optimiser: unrecorded (no comment from the role naming #177, #178)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083

**Notes:** PR #177's earliest listed commit is 2026-11-05T02:51Z, 3 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #178's earliest listed commit is 2026-11-05T02:51Z, 3 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## window-four: Default window of 4 blocks a thread, not 2

- **PRs:** [#180](https://github.com/lmmx/mibz2/pull/180)
- **Journal sources:** docs/journal/2026-11-05-window.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T05:46Z, PR #180 first commit (author date) (https://github.com/lmmx/mibz2/pull/180/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "perf(parallel): the default window is 4 blocks a thread, not 2 BzReader::parallel and decode_parallel gave the decoder 2 x threads blocks in flight." (https://github.com/lmmx/mibz2/pull/180 (first commit message))
- "On CI's 4-vCPU runners the occupancy instrument shows the 4-thread workers idle for 24% of their time, mostly between blocks: waiting while the scanner reads the block they need (13%), on a slot (9%) and in the handoff (8%), with the scanner itself in a slot wait for 65% of the span." (https://github.com/lmmx/mibz2/pull/180 (first commit message))
- "A window of 4 x threads lets the scanner run ahead. parallel::SLOTS_PER_THREAD = 4 is the one place the default is set; the occupancy instrument follows it. docs/ledger.md's table is regenerated at the new default (+21.6 MB reserved at 4 threads, most of it symbols a real block never touches). docs…" (https://github.com/lmmx/mibz2/pull/180 (first commit message))

**must_hold** (quoted):

- "A window of 4 x threads lets the scanner run ahead. parallel::SLOTS_PER_THREAD = 4 is the one place the default is set; the occupancy instrument follows it. docs/ledger.md's table is regenerated at the new default (+21.6 MB reserved at 4 threads, most of it symbols a real block never touches). docs…" (https://github.com/lmmx/mibz2/pull/180 (first commit message))
- "- **Peak RSS:** up at 2 and 4 threads." (https://github.com/lmmx/mibz2/pull/180 (PR body, current text))
- "**Decision rule:** - **Correctness:** CI green, Miri included, and the same bytes." (https://github.com/lmmx/mibz2/pull/180 (PR body, current text))
- "Peak RSS at 4 threads must rise by no more than the ledger's 21.6 MB." (https://github.com/lmmx/mibz2/pull/180 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T05:46Z] "**Why.** Instruction cuts in the kernels have stopped moving 4-thread wall time (#179)." (https://github.com/lmmx/mibz2/pull/180 (PR body, current text))
- [2026-11-05T05:46Z] "So RSS should rise by less than the reservation." (https://github.com/lmmx/mibz2/pull/180 (PR body, current text))
- [2026-11-05T05:46Z] "**Predictions** (registered in `docs/journal/2026-11-05-window.md`):" (https://github.com/lmmx/mibz2/pull/180 (PR body, current text))
- [2026-11-05T05:46Z] "- **Occupancy:** this PR's occupancy run should show the slot-wait and scanner-reading shares falling." (https://github.com/lmmx/mibz2/pull/180 (PR body, current text))

**same_quantity_facts:** 33 matching lines in 16 documents dated before commit_to_build, searched on `window|blocks a thread|in flight` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "\| 0.14.0, L148–149: "Avoid unnecessary windows … bzip2 … 344 MB/s to 388 MB/s" \| 0.13.0 put bzip2 on the gzip `ParallelGzipReader` chunk architecture (L211), which made 32 KiB seek windows. bzip2 blocks need none, so making them was overhead of that architecture \| mibz2 has no windows or seek po…" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-27T20:01Z] docs/journal/2026-10-27-move-structure-walk.md (doc): "cost by 59–63%. A schedule with fewer independent chains in flight would gain where the" — https://github.com/lmmx/mibz2/blob/50d404c8bdd2/docs/journal/2026-10-27-move-structure-walk.md
- [2026-10-27T22:37Z] docs/journal/2026-10-27-four-thread-ceiling.md (doc): "- **Believed:** the admission window (`2 × threads` slots, released in order) starved the" — https://github.com/lmmx/mibz2/blob/ffc4629a05e5/docs/journal/2026-10-27-four-thread-ceiling.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- [S] McCalpin, Intel forum (https://community.intel.com/t5/Software-Tuning-Performance/Too-low-memory-bandwidth-utilization/td-p/1134260): DDR4-2666 x8 parts have tFAW = max(20 clk, 21 ns), which allows 4 activates per 21 ns per rank. "It was common that the four-activate-window limited throughput…" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder.md (doc): "- the build's window checks (about 2% of a team's round, if the VM's 7–8% of the build holds)." — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder.md
- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "\| `u`: 24-bit link + F window \| 2.70 + 0.03 MB \| 1 load (unaligned, masked) + 2 L1 loads off the chain \| **measured: fails stage B** (u1/o1 0.681) \|" — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md
- [2026-11-04T22:31Z] comment 6643809123 (#145 coordinator): "- At 4.49 cycles per step and about 38.55 cycles of lookup latency, about 8.6 lookups are in flight. Overlap is already sufficient; the instruction stream sets the pace." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643809123
- [2026-11-04T22:37Z] comment 6643931565 (#145 researcher): "**The published model does not imply “ell,R rather than u at >=2 workers.”** In B2, R is retire-window capacity and ell latency. In w=U/D+ell*U/R, fewer macro-ops reduce both terms. Its SMT model predicts similar relative gains under concurrency, unless a new shared bottleneck appears. The additive…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643931565
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md (doc): "- **The sample stands for the dump:** 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) …" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md (doc): "- **The sample stands for the dump:** 12 windows of 1 MiB at (2i + 1)/24 of the dump, 431 of its own blocks; their mix of entities and their ratio, 17.69× (the windows' 95% interval ±14.0%), stand for the whole dump's. Another sample of the same dump (wikidata-pq's bench/full, 24 windows of 25 MB) …" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #180) / Optimiser: unrecorded (no comment from the role naming #180)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083

## staging-only: Staged walk tests each lane's start flag, without the deferred flags

- **PRs:** [#181](https://github.com/lmmx/mibz2/pull/181)
- **Journal sources:** docs/journal/2026-11-05-staging-only.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T05:59Z, PR #181 first commit (author date) (https://github.com/lmmx/mibz2/pull/181/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "**Why:** - **Locally, staging alone carries all of #171's gain.** Every point estimate favours it: at 1 thread, #171 over staging alone is 1.0075 (0.973, 1.023), and 1.0295 (0.981, 1.093) pinned." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))
- "At 4 threads it is 1.0026 (0.962, 1.050)." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))
- "**`decode_block` instructions (Callgrind)**, against master c9433318: \| block \| master \| this PR \| change \| \|---\|--:\|--:\|--:\| \| 1 \| 18,539,867 \| 19,273,484 \| +3.96% \| \| 41 \| 17,896,118 \| 18,647,570 \| +4.20% \| \| 233 \| 18,332,436 \| 19,071,853 \| +4.03% \| \| 400 \| 17,907,569 \…" (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))

**must_hold** (quoted):

- "**Why:** - **Locally, staging alone carries all of #171's gain.** Every point estimate favours it: at 1 thread, #171 over staging alone is 1.0075 (0.973, 1.023), and 1.0295 (0.981, 1.093) pinned." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))
- "At 4 threads it is 1.0026 (0.962, 1.050)." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))
- "**Decision rule:** registered in `docs/journal/2026-11-05-staging-only.md` before this run." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T06:00Z] "This PR keeps the staging and removes the deferred flags." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))
- [2026-11-05T06:00Z] "**Why:** - **Locally, staging alone carries all of #171's gain.** Every point estimate favours it: at 1 thread, #171 over staging alone is 1.0075 (0.973, 1.023), and 1.0295 (0.981, 1.093) pinned." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))
- [2026-11-05T06:00Z] "- **The walk is bound by misses in flight, not instructions** (#179), so the flags' instruction saving may buy nothing." (https://github.com/lmmx/mibz2/pull/181 (PR body, current text))

**same_quantity_facts:** 14 matching lines in 9 documents dated before commit_to_build, searched on `staging|start flag|deferred` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T15:46Z] docs/journal/2026-10-27-literature-reading.md (doc): "- **indexed_bzip2** (Knespel): block-parallel decoding with prefetching and caching, sharing rapidgzip's architecture. Its README reports 6.1× over Python's bz2 at 12 threads, but gives no block-finder figures. Its block finder bears on #10, which stays deferred." — https://github.com/lmmx/mibz2/blob/24169e657c1f/docs/journal/2026-10-27-literature-reading.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "\| L5–6: block finder simplified, 64 → 92 MB/s \| Deflate's block finder \| #10's scanner \| #10 stays deferred: the scanner binds only from about 10 workers \|" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-11-04T22:40Z] comment 6643992749 (#145 coordinator): "**#157 is B2's kernel B.** B2 is `docs/research-portfolio:docs/journal/2026-10-30-b2-gathered-walk.md`, read-only, 30 October. Kernel B is "the same 32 scalar chains with deferred flags and tiled commits", predicted at 8.0–9.0 µops/B. #157 compiles to 9 instructions per step. The instruction side h…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643992749
- [2026-11-04T23:01Z] comment 6644364456 (#145 coordinator): "2. **#157 merged** (master 5f537871): production `room_segments` with deferred handovers, eight lanes unrolled, store before shift. Per step 16 → 9 instructions; `decode_block` −21.6% to −22.6%; walk −36%. Evidence: b0f6e89d `evidence/walk-75b13f00/`, with profiles, boundaries and listings." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6644364456
- [2026-11-05T03:59Z] docs/journal/2026-11-05-staged-walk.md (doc): "- Instructions do not back a staging cost on `runs`. Its single block runs 0.8% fewer instructions (17,756,248 against 17,904,270)." — https://github.com/lmmx/mibz2/blob/dfba89795e6e/docs/journal/2026-11-05-staged-walk.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-build-run-vectors.md (doc): "arms built from the driver of `2026-11-05-deferred-flag-walk/` (rustc 1.97.0, fat LTO," — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-build-run-vectors.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-mtf-short-runs.md (doc): "arms built from the driver of `2026-11-05-deferred-flag-walk/` (rustc 1.97.0, fat LTO). The" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-mtf-short-runs.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-tail-passes.md (doc): "driver of `2026-11-05-deferred-flag-walk/` (rustc 1.97.0, fat LTO, `codegen-units = 1`). The" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-tail-passes.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round.md (doc): "\| `runs` at 1 thread on #171: a staging cost? \| none found. Aligned layouts give +0.0% (−1.3, +1.4) over 500 rounds; a layout-only rebuild moves it by up to 2.7% \| noise at the bench's resolution \|" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #181) / Optimiser: unrecorded (no comment from the role naming #181)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083

## fused-postwalk: RLE1 over the walk's segments in place, CRC folded in

- **PRs:** [#183](https://github.com/lmmx/mibz2/pull/183)
- **Journal sources:** docs/journal/2026-11-05-fused-postwalk.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** Astra's REVIEW 6651150258 ranked the fusion first (per PR #183)
- **commit_to_build:** 2026-11-05T06:29Z, PR #183 first commit (author date) (https://github.com/lmmx/mibz2/pull/183/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.05; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "It was committed before any CI timing. ## Measured locally (Callgrind, one `decode_block`) \| block \| instructions \| of which refactor \| of which fusion \| post-walk lines missing a 1 MiB L2 \| \|---\|--:\|--:\|--:\|--:\| \| wd 1 \| −1.48% \| −168,884 \| −116,646 \| −57% (4.31 → 1.84 MB) \| \| w…" (https://github.com/lmmx/mibz2/pull/183 (PR body, current text))

**must_hold** (quoted):

- "It was committed before any CI timing. ## Measured locally (Callgrind, one `decode_block`) \| block \| instructions \| of which refactor \| of which fusion \| post-walk lines missing a 1 MiB L2 \| \|---\|--:\|--:\|--:\|--:\| \| wd 1 \| −1.48% \| −168,884 \| −116,646 \| −57% (4.31 → 1.84 MB) \| \| w…" (https://github.com/lmmx/mibz2/pull/183 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T06:29Z] "- RLE1's parse is positional, so a token can straddle segments, a segment can be a byte: at each boundary the bytes left (fewer than a step's window) and the next window's worth go into a 72-byte stitch, the straddling tokens are taken whole there, and the decode goes on that far into the pieces after." (https://github.com/lmmx/mibz2/pull/183 (first commit message))
- [2026-11-05T06:29Z] "Short blocks keep the plain walk over `ll`; a block whose chain would join a short cycle round again (a string repeated) is assembled over `ll` as before." (https://github.com/lmmx/mibz2/pull/183 (first commit message))
- [2026-11-05T06:29Z] "RLE1 over arbitrary cuts against the whole (runs at every offset to a boundary, 1-byte and empty pieces, a count byte first in a piece, an unfinished run in the last piece), the chain against the segment oracle, the chain through the crafted-permutation, short-cycle and periodic walk tests, the CRC in pieces, the split against `invert`, and the decoder's errors on the chain's path (one scratch th…" (https://github.com/lmmx/mibz2/pull/183 (first commit message))

**same_quantity_facts:** 24 matching lines in 13 documents dated before commit_to_build, searched on `RLE1|unrle|CRC|assemble|3n \+ 2m|passes over` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "\| RLE1 + CRC \| 0.50 \| 2.3% \|" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "- Segment assembly (×13.6) and RLE1 (×10.2) are the most sensitive relatively, but 3%" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder.md (doc): "- **The model** (the integration plan §5): 1.26–1.38× p4 for the decoder at 3x3 or 4x2. It adds the scanner, RLE1, the CRC and the joining to PR 0's inversion figures." — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder.md
- [2026-11-05T00:34Z] docs/journal/2026-11-02-p87-production-integration.md (doc): "−1.4% [−1.7, −1.2] and "RLE1" +3.2% [+2.3, +4.2] between the two builds." — https://github.com/lmmx/mibz2/blob/e26e3d9cbac6/docs/journal/2026-11-02-p87-production-integration.md
- [2026-11-05T00:39Z] comment 6646303348 (#145 researcher): "**2. RLE1 interpretation needs one source correction, including to my earlier shorthand.** Its timed phase path uses a separate walked buffer, so the +2.6%/+3.9% rows do not measure production in-place RLE1. But main calls production mibz2::decode for parity after the first timed pass, before subse…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6646303348
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation.md (doc): "- #128's integration record saw the same RLE1 shift (+3.2%). It is recorded the same way here:" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md (doc): "\| CRC \| 0.14 → 0.14 \| 1.2% \| −0.1% (−1.7%, +1.5%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md (doc): "\| CRC \| 0.14 → 0.14 \| 1.2% \| −1.2% (−3.1%, +0.8%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr171-run1.md (doc): "stages (1t c/B): walk 3.06->3.08 +0.3% (-1.3,+1.9); RLE1 -2.1% faster; total +0.0%" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr171-run1.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr171-run3.md (doc): "stages (1t c/B): walk 6.47->5.08 -20.2% (-27.6,-11.9) FASTER; scanner +0.7% and CRC +1.9% flagged (stage diagnostic, unchanged code); total 12.74->11.35 -10.0% (-14.6,-5.2) faster" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr171-run3.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6947023492 (2026-11-17T11:41Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #183)

**monodromy_involved:** yes: records naming its PRs or recipes: 2026-11-17-miss-cost-scoring.md

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work

## coresident-walk: Narrower walk with L2 prefetch for SMT co-residents (16 lanes, prefetcht1)

- **PRs:** none
- **Journal sources:** docs/journal/2026-11-05-coresident-walk.md
- **i9 recipes:** none
- **Deciding machine:** CI walk sweeps (EPYC 9V45, 9V74)
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; one cell of CI run 40573190665
- **commit_to_build:** 2026-11-05T08:06Z, docs/journal/2026-11-05-coresident-walk.md first commit 2b109039 (no PR; earliest recorded) (https://github.com/lmmx/mibz2/blob/2b10903998c9/docs/journal/2026-11-05-coresident-walk.md)
- **Cost:** i9 0 min [no i9 bundle named]; CI: unrecorded; calendar days, commit to verdict: 0.00; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 24 matching lines in 19 documents dated before commit_to_build, searched on `SMT|sibling|16 lanes|prefetcht1|co.resident` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-04T22:27Z] comment 6643744616 (#145 coordinator): "**The finding that redirects the work:** a 36% cut in walk instructions bought 4% of wall time at 1 thread and nothing at 2 or 4. At the thread counts the deployment uses, walk instructions do not bind. Something else does: memory latency or bandwidth on the chase, the serial scanner (0.11 s of 1.2…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643744616
- [2026-11-04T22:37Z] comment 6643931565 (#145 researcher): "**The published model does not imply “ell,R rather than u at >=2 workers.”** In B2, R is retire-window capacity and ell latency. In w=U/D+ell*U/R, fewer macro-ops reduce both terms. Its SMT model predicts similar relative gains under concurrency, unless a new shared bottleneck appears. The additive…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6643931565
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md (doc): "\| 4: all 4 CPUs, sharing SMT siblings \| 522 \| 6.60 \| 1.61× (1.58–1.64) \| 53% \| 532 \| 6.49 \| 1.59× (1.56–1.61) \| 54% \| −1.4% (−3.0%, +0.3%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md (doc): "\| 4: all 4 CPUs, sharing SMT siblings \| 690 \| 4.74 \| 1.51× (1.50–1.53) \| 53% \| 691 \| 4.70 \| 1.50× (1.49–1.52) \| 54% \| −0.8% (−2.0%, +0.5%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md
- [2026-11-05T06:03Z] comment 6651023083 (#160 coordinator): "**Asked:** re-rank on the binding resource, priced against c9433318 with #180's window assumed. Price each candidate in misses per output byte and their latency at the runner's L2: 512 KiB on Zen 3, 1 MiB on Zen 4/5, 2 MiB on Emerald Rapids. Price under 4 workers on 2 SMT cores, not in Ir." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083
- [2026-11-05T06:12Z] comment 6651150258 (#145 astra): "**Compact:** 1/16 versus 1/30 are compulsory distinct main lines per row, not random steady-state miss rates. Rec fields and header share one aligned line; RAW adds a dependent access. Two sibling compact tables total 3.84 MB, so neither pair fits the stated 2 MiB L2. The report models shared-L2 re…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6651150258
- [2026-11-05T06:12Z] comment 6651156608 (#145 coordinator): "**Compact parity:** your memory model (compulsory lines, a shared L2 of 3.84 MB for two sibling tables, builder traffic) is noted. There is no compact build until the fusion and #180 are decided." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6651156608
- [2026-11-05T06:40Z] docs/journal/2026-11-05-tile-size.md (doc): "Cascade Lake Xeon with a 32 KiB 8-way L1D, a 1 MiB L2, no SMT and no PMU." — https://github.com/lmmx/mibz2/blob/0769b52fea9b/docs/journal/2026-11-05-tile-size.md
- [2026-11-05T06:40Z] docs/journal/2026-11-05-walk-round/bench-runs/pr178-run1.md (doc): "rule: benefit not shown in run 1; wikidata 4t wall harm signal -> run 2 on the same head (registered prediction: larger tile harms at 4t under SMT; 8573C L1D 48 KiB < 2 x 32 KiB)" — https://github.com/lmmx/mibz2/blob/0769b52fea9b/docs/journal/2026-11-05-walk-round/bench-runs/pr178-run1.md
- [2026-11-05T06:40Z] docs/journal/2026-11-05-window.md (doc): "likely 2–5%, since an idle SMT sibling also speeds its pair." — https://github.com/lmmx/mibz2/blob/0769b52fea9b/docs/journal/2026-11-05-window.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming the case) / Optimiser: unrecorded (no comment from the role naming the case)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## isa-v3: ISA-level dispatch (x86-64-v3) for the scalar stages

- **PRs:** [#201](https://github.com/lmmx/mibz2/pull/201)
- **Journal sources:** docs/journal/2026-11-05-isa-probe.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T14:48Z, PR #201 first commit (author date) (https://github.com/lmmx/mibz2/pull/201/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Nothing measures what BMI2's `shrx`/`bzhi` or LZCNT would buy them. ## Registered reading (fixed before the bench runs) - **A stage gains ≥ 5% beyond noise** in the per-stage table, or `wikidata` 1-thread CPU falls beyond noise by ≥ 2%: that stage gets a multiversioned clone." (https://github.com/lmmx/mibz2/pull/201 (PR body, current text))

**must_hold** (quoted):

- "It is registered and benched as its own PR, against its own rule." (https://github.com/lmmx/mibz2/pull/201 (PR body, current text))
- "The 1-thread rows are pinned to one CPU." (https://github.com/lmmx/mibz2/pull/201 (PR body, current text))
- "The `same bytes` column must read yes everywhere." (https://github.com/lmmx/mibz2/pull/201 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T14:48Z] "Nothing measures what BMI2's `shrx`/`bzhi` or LZCNT would buy them. ## Registered reading (fixed before the bench runs) - **A stage gains ≥ 5% beyond noise** in the per-stage table, or `wikidata` 1-thread CPU falls beyond noise by ≥ 2%: that stage gets a multiversioned clone." (https://github.com/lmmx/mibz2/pull/201 (PR body, current text))

**same_quantity_facts:** 33 matching lines in 19 documents dated before commit_to_build, searched on `x86.64.v3|AVX2|ISA|target.cpu` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-05T06:12Z] comment 6651150258 (#145 astra): "**Compact:** 1/16 versus 1/30 are compulsory distinct main lines per row, not random steady-state miss rates. Rec fields and header share one aligned line; RAW adds a dependent access. Two sibling compact tables total 3.84 MB, so neither pair fits the stated 2 MiB L2. The report models shared-L2 re…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6651150258
- [2026-11-05T07:38Z] comment 6652503445 (#145 coordinator): "**The build:** `rle::unrle_chain` reads the walk's segments in order, where they lie, with a 72-byte stitch and an AVX2 step with a scalar twin. `crc::Running` folds in at 32 KiB. Logical traffic falls from 3n+2m to n+m. Simulated post-walk L2 misses: −57% on wikidata, −50% on `runs`. `decode_block…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6652503445
- [2026-11-05T12:23Z] comment 6657883434 (#160 coordinator): "**Binding.** At 4 threads the box is CPU-saturated (utilisation 0.91–0.97), so wall ≈ co-run CPU / 4. What counts is stage time × co-run inflation, not stage time alone. #171 cut 4-thread CPU 16–18% with 1-thread time flat: the levers live in co-run cost." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6657883434
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census.md (doc): "At 4 threads the decoder is **CPU-capacity-bound** (OBSERVED: utilisation 0.91–0.97 on every" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/accounting.md (doc): "7. THE MASTER BENCH CPU. The 8573C has no co-run factors. The three measured CPUs disagree in opposite directions (build 2.25 / 1.76 / 1.43; walk 1.46 / 2.22 / 2.12). Borrowing either set puts the 8573C's 4t walk at 40-53% of wall and its build at 12-20%, so its split is unpinned." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/accounting.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/serial-stages.md (doc): "WHY: The aggregate part holds (OBSERVED). Utilisation is 0.919-0.970 (8573C 41980640481 0.959; 8370C 41986908473 0.970; 9V45 41986885355 attempt 2 0.919). Where both resolved, 4t wall moved with 4t CPU: #183 fusion alone on the 8370C, wall -1.5% against CPU -1.9%; #181 run 3 (41979947814) on the 9V…" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/serial-stages.md
- [2026-11-05T12:36Z] comment 6658149381 (#145 optimizer): "**My ranking:** AMD walk interference is the largest conditional pool: 7763 ~16–18% at fixed utilisation/rest. The 9V74 calculation is a ~20–22% scenario, explicitly assuming a transferred worker share; it is not a measured decoder bound. On 8370C, build interference (~5–6%) ranks ahead of further …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6658149381
- [2026-11-05T14:01Z] docs/journal/2026-11-05-compact30-staged/bench-run1-9v74.md (doc): "**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU c…" — https://github.com/lmmx/mibz2/blob/90ecc7d3f446/docs/journal/2026-11-05-compact30-staged/bench-run1-9v74.md
- [2026-11-05T14:01Z] docs/journal/2026-11-05-compact30-staged/bench-run2-7763.md (doc): "**CPU per byte** is the process's CPU (every thread, user + sys) over the output. **Amplification** is that at N threads over the same build's at 1, paired by round: 1× is no extra CPU for the same bytes, and above it the work costs more as threads are added (stalls, sharing, synchronisation; CPU c…" — https://github.com/lmmx/mibz2/blob/90ecc7d3f446/docs/journal/2026-11-05-compact30-staged/bench-run2-7763.md
- [2026-11-05T14:04Z] docs/journal/2026-11-05-corun-placement.md (doc): "- **(a) The AVX2 gather walk** (`research/gather-walk`, aed6a213). It runs 31.3% fewer" — https://github.com/lmmx/mibz2/blob/1132857e1881/docs/journal/2026-11-05-corun-placement.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #201) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6758172458 (2026-11-10T03:31Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## job-handoff: Idle workers wait for jobs without holding the jobs' lock

- **PRs:** [#203](https://github.com/lmmx/mibz2/pull/203)
- **Journal sources:** docs/journal/2026-11-05-job-handoff.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** the census (2026-11-05-binding-census.md), per PR #203
- **commit_to_build:** 2026-11-05T14:34Z, PR #203 first commit (author date) (https://github.com/lmmx/mibz2/pull/203/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.06; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The census found an idle worker holding the jobs' `Mutex<Receiver>` across `recv`: at 4 threads, 3.0–3.5% of worker time in the lock, and 1.7–2.4% idle with a job queued." (https://github.com/lmmx/mibz2/pull/203 (PR body, current text))
- "**Prediction:** - **4-thread wall:** −0.2% to −0.8%, with a ceiling of about 1.0–1.7%." (https://github.com/lmmx/mibz2/pull/203 (PR body, current text))

**must_hold** (quoted):

- "The census found an idle worker holding the jobs' `Mutex<Receiver>` across `recv`: at 4 threads, 3.0–3.5% of worker time in the lock, and 1.7–2.4% idle with a job queued." (https://github.com/lmmx/mibz2/pull/203 (PR body, current text))
- "It holds the change, the untimed evidence, the prediction and the decision rule." (https://github.com/lmmx/mibz2/pull/203 (PR body, current text))
- "**Rule:** correctness required." (https://github.com/lmmx/mibz2/pull/203 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T15:43Z] "It holds the change, the untimed evidence, the prediction and the decision rule." (https://github.com/lmmx/mibz2/pull/203 (PR body, current text))
- [2026-11-05T15:43Z] "**Prediction:** - **4-thread wall:** −0.2% to −0.8%, with a ceiling of about 1.0–1.7%." (https://github.com/lmmx/mibz2/pull/203 (PR body, current text))

**same_quantity_facts:** 18 matching lines in 14 documents dated before commit_to_build, searched on `Mutex<Receiver>|jobs'? lock|idle with a job|handoff` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-04T23:40Z] comment 6645304470 (#160 coordinator): "5. **The steady handoff:** 34.7% of the attribution's Q2. The erratum says it fits a receiver-ownership convoy, but doesn't prove it; this is idle capacity, separate from cycle inflation." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6645304470
- [2026-11-05T00:47Z] comment 6646432357 (#145 astra): "Compact-table parity is now **10,861,581 walk Ir / 12.0688122 per row**, holding the old compact build/rest fixed: a **46.25% cut** from its existing walk merely to break even. Handoff has zero direct decode_block saving; C must justify its increment over B; Precopy's negative prior/reopening condi…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6646432357
- [2026-11-05T06:03Z] comment 6651023083 (#160 coordinator): "**The 4-thread bound was between blocks, not in them.** Occupancy before #171 had the workers idle 24%: 13% waiting on the scanner's read, 9% in slot waits and 8% in the handoff. #180 widens the default window from 2t to 4t. Its run 1 on a Xeon 8370C:" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083
- [2026-11-05T06:40Z] docs/journal/2026-11-05-window.md (doc): "75.1% on another runner. The waits for the scanner's read, for a slot and in the handoff are" — https://github.com/lmmx/mibz2/blob/0769b52fea9b/docs/journal/2026-11-05-window.md
- [2026-11-05T12:23Z] comment 6657883434 (#160 coordinator): "- **The job handoff.** A worker holds the `Mutex<Receiver>` across `recv`. At 4 threads that is 3.0–3.5% of worker time in the lock, with a ceiling of about 2–3% of wall." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6657883434
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census.md (doc): "- At 4 threads, 3.0–3.5% of worker time goes to the lock and 1.7–2.4% is idle with a job" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/accounting.md (doc): "CLAIM: There is time nothing accounts for. At 1t, 2-4% of wall is covered neither by the stage table nor by the harness's copy and hash. At 4t, the decoder's CPU on the 8370C exceeds four co-resident sequential decodes by about 8%, of which 3-6% is plausibly phases' fastest-of-5 optimism under co-r…" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/accounting.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/serial-stages.md (doc): "- At 4t: w.lock 3.0-3.5% and 'idle with a job queued' 1.7-2.4% of worker time (occupancy 42009406264 and 41976148952, timeline mode)." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/serial-stages.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/supply.md (doc): "UNKNOWNS: - **The EPYC 9V45 has twice the idle of the other runners at 4t:** 6.8-8.4% of vCPU time in the benches (r41979947814, r41986885355), against 2.7-4.5% elsewhere. No occupancy run has landed on a 9V45; the one dispatched here landed on a 9V74. Whether that idle is start-up and tail, steady…" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/supply.md
- [2026-11-05T12:36Z] comment 6658149381 (#145 optimizer): "For the smaller existing workflows: Huffman has a 0–4.3% conditional direct pool; handoff a 0–2% working envelope; MTF a 0–3% working envelope with larger but unverified ideal scenarios. None has a demonstrated positive lower bound. Handoff, scanner starvation and steady supply overlap. Placement i…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6658149381

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #203) / Optimiser: unrecorded (no comment from the role naming #203)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work

## huffman-register: Symbol loop keeps the bit reader's state in registers

- **PRs:** [#204](https://github.com/lmmx/mibz2/pull/204)
- **Journal sources:** docs/journal/2026-11-05-huffman-register.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it
- **commit_to_build:** 2026-11-05T14:44Z, PR #204 first commit (author date) (https://github.com/lmmx/mibz2/pull/204/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.12; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The reader escaped, so LLVM kept it in memory: in the bench's binary (rustc 1.99.0, ab) each lookup stored `buf` and did a read-modify-write on `live` through the caller's pointer, and the next lookup reloaded `buf`, a store-to-load forward on the lookup-to-lookup chain: four forwards of `buf` and …" (https://github.com/lmmx/mibz2/pull/204 (first commit message))
- "**Untimed evidence:** - **Instructions:** −5.9% per lookup iteration; −2.5% for the scanner as a whole." (https://github.com/lmmx/mibz2/pull/204 (PR body, current text))

**must_hold** (quoted):

- "The reader's address escapes, and LLVM must assume the `u16` symbol stores may alias it." (https://github.com/lmmx/mibz2/pull/204 (PR body, current text))
- "- **No other stage moves.** **Rule:** correctness required." (https://github.com/lmmx/mibz2/pull/204 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T14:44Z] "New tests: the state stepped in a local against the slice's bits, to its end and past it; the symbol loop against a reader in memory on crafted blocks (corrupt and long codes, flips, cuts), exact to the end bit; flipped symbol bits against libbzip2." (https://github.com/lmmx/mibz2/pull/204 (first commit message))
- [2026-11-05T15:43Z] "The reader's address escapes, and LLVM must assume the `u16` symbol stores may alias it." (https://github.com/lmmx/mibz2/pull/204 (PR body, current text))
- [2026-11-05T15:43Z] "A scanner cut under 10% on Intel refutes the mechanism; another stage moving would be an artefact." (https://github.com/lmmx/mibz2/pull/204 (PR body, current text))

**same_quantity_facts:** 160 matching lines in 41 documents dated before commit_to_build, searched on `bit reader|symbol loop|huffman|scanner` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-05T12:23Z] comment 6657883434 (#160 coordinator): "- **The Huffman bit buffer** is kept in memory, with a store-forward on every lookup: 0.5–4.1%." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6657883434
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census.md (doc): "- 0.5–4.1% of 4-thread wall, plus up to about 0.9% of scanner-starvation idle (INFERRED)." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/accounting.md (doc): "CORR: MTF 0-3%. Scanner 0-3%, plus up to about 2% from i:rd. Post-walk 0-2% on both vendors, mechanism unknown." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/accounting.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/nogo.md (doc): "UNKNOWNS: - **Four copies are not four threads.** The phases shares come from four processes, each with its own scanner and no window. Using them as the 4-thread shares is inferred; the support is that the four-copy total inflation (x1.62) matches the decoder's 4t amplification on 8573C (1.62x)." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/nogo.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/serial-stages.md (doc): "- scanner 1.32x, MTF 1.23x, build 1.76x;" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/serial-stages.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/supply.md (doc): "- The 9V45's 7-8% idle exists only in sampler-on bench runs; no occupancy run has landed on a 9V45. Part of it could be the sampler preempting the scanner or reader, the supply threads. This is unmeasured." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/supply.md
- [2026-11-05T12:36Z] comment 6658149381 (#145 optimizer): "For the smaller existing workflows: Huffman has a 0–4.3% conditional direct pool; handoff a 0–2% working envelope; MTF a 0–3% working envelope with larger but unverified ideal scenarios. None has a demonstrated positive lower bound. Handoff, scanner starvation and steady supply overlap. Placement i…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6658149381
- [2026-11-05T14:01Z] docs/journal/2026-11-05-compact30-staged/bench-run1-9v74.md (doc): "At 4 threads the workers bound it. Per byte: the scanner 7.6 ns a byte of bz2, the workers 3.84 ns a byte of output at 4 threads (CPU, summed over them), 2.54 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (head)." — https://github.com/lmmx/mibz2/blob/90ecc7d3f446/docs/journal/2026-11-05-compact30-staged/bench-run1-9v74.md
- [2026-11-05T14:01Z] docs/journal/2026-11-05-compact30-staged/bench-run2-7763.md (doc): "At 4 threads the workers bound it. Per byte: the scanner 10.2 ns a byte of bz2, the workers 5.14 ns a byte of output at 4 threads (CPU, summed over them), 3.21 ns at 1 thread (wall, less the scanner's part); the scanner would bind from about 9 worker threads (head)." — https://github.com/lmmx/mibz2/blob/90ecc7d3f446/docs/journal/2026-11-05-compact30-staged/bench-run2-7763.md
- [2026-11-05T14:04Z] docs/journal/2026-11-05-corun-placement.md (doc): "are the scanner (9.7–10.7% of 4-thread CPU), the reader and the bench sampler. The cost is" — https://github.com/lmmx/mibz2/blob/1132857e1881/docs/journal/2026-11-05-corun-placement.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #204) / Optimiser: unrecorded (no comment from the role naming #204)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## mtf-two-level: libbzip2's two-level MTF list, its first 32 bytes in two registers

- **PRs:** [#207](https://github.com/lmmx/mibz2/pull/207)
- **Journal sources:** docs/journal/2026-11-05-mtf-two-level.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** pre-team era: one Claude session (later the Coordinator) built, benched and closed it; from libbzip2
- **commit_to_build:** 2026-11-05T09:47Z, PR #207 first commit (author date) (https://github.com/lmmx/mibz2/pull/207/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.54; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The first, sub-lists 0 and 1, is two xmm registers: an index below 32 (98% of Wikidata's byte symbols) is two pshufb, an or, an alignr and a blend from a 32-row table, with no branch on the index and no memory on the chain: 2 cycles a byte symbol by Zen 4's latencies, 3.0 in llvm-mca's znver4 model…" (https://github.com/lmmx/mibz2/pull/207 (first commit message))
- "A move of index < 32 (98.5% of Wikidata's moves) is two `pshufb`, a `por`, a `palignr` and a `pblendvb`, from a 2 KB static table." (https://github.com/lmmx/mibz2/pull/207 (PR body, current text))
- "- **The chain:** llvm-mca gives 3.0 cycles (znver4) and 4.0 (skylake), against master's 11.0." (https://github.com/lmmx/mibz2/pull/207 (PR body, current text))

**must_hold** (quoted):

- "**Rule:** - **Correctness** is required." (https://github.com/lmmx/mibz2/pull/207 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-05T09:47Z] "The first, sub-lists 0 and 1, is two xmm registers: an index below 32 (98% of Wikidata's byte symbols) is two pshufb, an or, an alignr and a blend from a 32-row table, with no branch on the index and no memory on the chain: 2 cycles a byte symbol by Zen 4's latencies, 3.0 in llvm-mca's znver4 model (which takes alignr at 2)." (https://github.com/lmmx/mibz2/pull/207 (first commit message))
- [2026-11-05T18:16Z] "- **Replication:** at least two runs on two different CPU models." (https://github.com/lmmx/mibz2/pull/207 (PR body, current text))
- [2026-11-05T18:16Z] "- **Refuters:** - an MTF cut under 5% in every run;" (https://github.com/lmmx/mibz2/pull/207 (PR body, current text))

**same_quantity_facts:** 34 matching lines in 23 documents dated before commit_to_build, searched on `two.level|move.to.front|MTF` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-05T05:50Z] docs/journal/2026-11-05-mtf-byte-list.md (doc): "- **What it says:** about 0.6% of instructions, on MTF's serial chain, is at the edge of the" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-mtf-byte-list.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round.md (doc): "\| #172 \| MTF's list holds bytes \| −0.6% \| 4-thread CPU −2.2% in one run \| merged \|" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr168-run1.md (doc): "stages (1t c/B): scanner +0.4%; MTF+RLE2 +1.6% (+1.4,+1.8) slower; build +0.2%; walk 3.62->3.78 +4.0% (-0.7,+9.0); total +2.2% (+0.0,+4.3) slower" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr168-run1.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr169-run1.md (doc): "stages (1t c/B): scanner -1.1% faster; MTF+RLE2 +1.1% slower; build 1.73->1.68 -3.1% (-3.5,-2.7) FASTER; walk +0.1%; total -0.3% (-0.5,-0.2) faster" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr169-run1.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr170-run1.md (doc): "stages (1t c/B): MTF+RLE2 2.09->2.00 -4.4% (-4.8,-4.0) FASTER; build -0.2%; walk -2.2% (-8.7,+4.8); total -2.0% (-5.2,+1.3)" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr170-run1.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr170-run2.md (doc): "stages (1t c/B): MTF+RLE2 2.01->1.82 -9.7% (-10.0,-9.3) FASTER; total -3.0% (-4.3,-1.6) faster" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr170-run2.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr172-run1.md (doc): "stages (1t c/B): MTF+RLE2 2.03->2.00 -1.3% (-1.8,-0.8) FASTER; walk +2.4% (-3.8,+9.0); total +1.1% (-2.4,+4.7)" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr172-run1.md
- [2026-11-05T05:50Z] docs/journal/2026-11-05-walk-round/bench-runs/pr172-run2.md (doc): "stages (1t c/B): MTF+RLE2 2.00->1.99 +0.2% (-0.6,+1.0); scanner +0.7% (0.0,+1.3) slower (untouched by the diff); walk +0.0%; total +0.1%" — https://github.com/lmmx/mibz2/blob/c94333183e01/docs/journal/2026-11-05-walk-round/bench-runs/pr172-run2.md
- [2026-11-05T06:03Z] comment 6651023083 (#160 coordinator): "\| MTF short runs, byte list (#170, #172) \| −1.5%, −0.6% \| ~−2% wall; 4t CPU −2.2% \| merged \|" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6651023083
- [2026-11-05T06:40Z] docs/journal/2026-11-05-walk-round/bench-runs/pr181-run2.md (doc): "stages (1t c/B): walk -0.9% (-2.3,+0.5); MTF +0.7% slower (untouched by the diff); total -0.3%" — https://github.com/lmmx/mibz2/blob/0769b52fea9b/docs/journal/2026-11-05-walk-round/bench-runs/pr181-run2.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #207) / Optimiser: unrecorded (no comment from the role naming #207)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator

## three-team-policy: Three cooperative teams of three at 10 threads on the ten-core, 20 MiB-L3 machine

- **PRs:** [#219](https://github.com/lmmx/mibz2/pull/219), [#220](https://github.com/lmmx/mibz2/pull/220), [#234](https://github.com/lmmx/mibz2/pull/234)
- **Journal sources:** docs/journal/2026-11-06-team-size.md, docs/journal/2026-11-06-three-team-policy.md
- **i9 recipes:** three-team-policy/
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** team-size screen (#219) by the Coordinator; policy proposal per #234 (Optimiser, Researcher, Guardian named)
- **commit_to_build:** 2026-11-06T09:38Z, PR #219 opened (https://github.com/lmmx/mibz2/pull/219)
- **Cost:** i9 40.5 over 2 bundle(s) min [three-team-policy/20261106T220546Z: 29.9 min (bundle start (name)→evidence commit); three-team-policy/20261106T225836Z: 10.6 min (bundle start (name)→evidence commit)]; CI: 29 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.56; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- 3 × 4 at 12 threads gave 1.097 (1.083–1.111), and 3 × 3 at 10 gave 1.075 (1.058–1.093), each with 18% less peak RSS." (https://github.com/lmmx/mibz2/pull/219 (PR body, current text))
- "- In the occupancy traces, the teams decode 96% of the run, and a block takes 1.38–1.40 ms at k = 2, 3 and 4." (https://github.com/lmmx/mibz2/pull/219 (PR body, current text))
- "- 3 × 4 at 12 threads: 1.097 (1.083–1.111) against 4 × 2 at 8;" (https://github.com/lmmx/mibz2/pull/220 (PR body, current text))

**must_hold** (quoted):

- "bench: team size against live tables on the i9 (just bench-team-size), registered before any run The decoder runs 4 teams of 2 from 8 threads up on the i9, so its curve is flat there by policy." (https://github.com/lmmx/mibz2/pull/219 (first commit message))
- "The rule, prediction and outcomes are in docs/journal/2026-11-06-team-size.md." (https://github.com/lmmx/mibz2/pull/219 (first commit message))
- "bench: team size against live tables on the i9 (just bench-team-size), registered before any run The decoder runs 4 teams of 2 from 8 threads up on the i9, so its curve is flat there by policy." (https://github.com/lmmx/mibz2/pull/220 (first commit message))
- "The rule, prediction and outcomes are in docs/journal/2026-11-06-team-size.md." (https://github.com/lmmx/mibz2/pull/220 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-06T09:38Z] "The rule, prediction and outcomes are in docs/journal/2026-11-06-team-size.md." (https://github.com/lmmx/mibz2/pull/219 (first commit message))
- [2026-11-06T09:38Z] "The rule, prediction and outcomes are in docs/journal/2026-11-06-team-size.md." (https://github.com/lmmx/mibz2/pull/220 (first commit message))
- [2026-11-06T09:38Z] "The i9's release-scaling run is flat from 8 to 20 threads, because the decoder runs the same 4 teams of 2 at every thread count from 8 up." (https://github.com/lmmx/mibz2/pull/219 (PR body, current text))
- [2026-11-06T10:40Z] "- **Prediction:** 1.04–1.09 and 1.03–1.07, regressing from the screen." (https://github.com/lmmx/mibz2/pull/220 (PR body, current text))
- [2026-11-06T19:56Z] "- **Machines:** any CPU with exactly that topology gets the change; the planner can't check the CPU model." (https://github.com/lmmx/mibz2/pull/234 (PR body, current text))
- [2026-11-06T19:56Z] "- the recipe binds every build and workload input, the toolchain and the executables, and each attempt ends with one stated outcome." (https://github.com/lmmx/mibz2/pull/234 (PR body, current text))

**same_quantity_facts:** 7 matching lines in 6 documents dated before commit_to_build, searched on `3 ?× ?3|three teams|team size|live tables?|tables? (per|/) L3` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-29T15:55Z] docs/journal/2026-10-29-literature-study/00-overview-research.md (doc): "Construction #81 improved cooperative build by 31.3% and build+walk by 10.2% in the reported 3×3 experiment Whether directly constructing compact Ψ can beat that stronger baseline" — https://github.com/lmmx/mibz2/blob/1e98588115c7/docs/journal/2026-10-29-literature-study/00-overview-research.md
- [2026-10-29T16:03Z] docs/journal/2026-10-29-literature-study/08-patched-coding-plateaus-research.md (doc): "Our recently fused T-vector construction became 31.3% faster in the reported 3×3 configuration." — https://github.com/lmmx/mibz2/blob/62549a3c4857/docs/journal/2026-10-29-literature-study/08-patched-coding-plateaus-research.md
- [2026-10-29T16:45Z] docs/journal/2026-10-29-frontier.md (doc): "**What series 2 already shows, with its limits stated.** In throughput against live table bytes, y3x3 (1657 MB/s from 10.8 MB of live tables) beats p4 (1206 MB/s from 14.4 MB). p4 is production's best worker count on the i9 in every scaling run. So at the inversion's level, sharing a table among a …" — https://github.com/lmmx/mibz2/blob/ae7bc3099933/docs/journal/2026-10-29-frontier.md
- [2026-10-29T18:20Z] docs/journal/2026-10-29-cooperative-decoder.md (doc): "- at 10, 1.008 against three teams of 3 (0.994–1.023), the fastest fixed arm there." — https://github.com/lmmx/mibz2/blob/23794f04201c/docs/journal/2026-10-29-cooperative-decoder.md
- [2026-10-29T21:15Z] docs/journal/2026-10-29-literature-study/02-optimal-time-construction-research.md (doc): "mibz2 #81, 3×3 Reported speedup" — https://github.com/lmmx/mibz2/blob/8ed29b069639/docs/journal/2026-10-29-literature-study/02-optimal-time-construction-research.md
- [2026-10-30T01:21Z] docs/journal/2026-10-30-compact-psi-feasibility.md (doc): "- The L3 hit share followed live table bytes within 0.031 at every N." — https://github.com/lmmx/mibz2/blob/7e7cce49a4f2/docs/journal/2026-10-30-compact-psi-feasibility.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 6 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6683945162 (2026-11-06T11:38Z, after commit_to_build, before the first i9 bundle) / Optimiser: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6683232182 (2026-11-06T10:57Z, after commit_to_build, before the first i9 bundle)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: pre-team era (before RULES.md, 6 Nov): the one Claude session, later the Coordinator, did the work; origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6682167812, https://github.com/lmmx/mibz2/pull/161#issuecomment-6681847165, https://github.com/lmmx/mibz2/pull/161#issuecomment-6682032075

**Notes:** 2 i9 bundles over 1 recipe(s); coders check whether master moved between them. PR #219's earliest listed commit is 2026-11-06T09:38Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #220's earliest listed commit is 2026-11-06T09:38Z, 1 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## pipelined-teams: Pipelined cooperative team with park and small-block levers as the 3x3 default

- **PRs:** [#247](https://github.com/lmmx/mibz2/pull/247), [#249](https://github.com/lmmx/mibz2/pull/249), [#250](https://github.com/lmmx/mibz2/pull/250)
- **Journal sources:** docs/journal/2026-11-07-knee-mechanism.md, docs/journal/2026-11-07-team-pipeline-registration.md, docs/journal/2026-11-07-pipeline-levers-registration.md, docs/journal/2026-11-07-pipelined-default-registration.md
- **i9 recipes:** team-pipeline, pipeline-levers, pipelined-default
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** H1 of 2026-11-07-knee-mechanism.md (Coordinator)
- **commit_to_build:** 2026-11-07T01:06Z, PR #247 opened (https://github.com/lmmx/mibz2/pull/247)
- **Cost:** i9 29.5 over 4 bundle(s) min [team-pipeline/20261107T042539Z: 0.0 min (bundle start (name)→evidence commit); team-pipeline/20261107T042622Z: 8.1 min (bundle start (name)→evidence commit); pipeline-levers/20261107T091926Z: 10.5 min (bundle start (name)→evidence commit); pipelined-default/20261107T105448Z: 10.9 min (bundle start (name)→evidence commit)]; CI: 8 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.42; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The i9 run decides. ## Trade-offs weighed - **Memory:** about 2 MB more per team: a second `ll` and `checkpoints` set, a second assembled buffer, and the small table (at most 1 MB)." (https://github.com/lmmx/mibz2/pull/247 (PR body, current text))
- "- **Causal comparisons,** each against the pipe at 3 × 3 on 10 threads and 3 × 4 on 12:" (https://github.com/lmmx/mibz2/pull/249 (PR body, current text))
- "- **Lever B's evidence, all of it required:** - the corrected same-team cross-thread overlap, which must exceed the pipe's by at least 10% of the blocks;" (https://github.com/lmmx/mibz2/pull/249 (PR body, current text))

**must_hold** (quoted):

- "**the default decoder is unchanged.** This PR adds a pipelined cooperative team (H1 of docs/journal/2026-11-07-knee-mechanism.md), its tests and trace marks, and the registration and recipe for its i9 run." (https://github.com/lmmx/mibz2/pull/247 (PR body, current text))
- "The i9 run decides. ## Trade-offs weighed - **Memory:** about 2 MB more per team: a second `ll` and `checkpoints` set, a second assembled buffer, and the small table (at most 1 MB)." (https://github.com/lmmx/mibz2/pull/247 (PR body, current text))
- "A job is waited for only with the pipeline empty. ## Deferred H2's placement arm (pinned against unpinned, with the actual thread-to-CPU mapping recorded) needs per-thread affinity records, or an interleavable pinned arm in `ab`." (https://github.com/lmmx/mibz2/pull/247 (PR body, current text))
- "Every arm here runs unpinned." (https://github.com/lmmx/mibz2/pull/247 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-07T11:27Z] "- **Helpers 1..k−1:** block i's fused build, cut into k − 1 shares (`coop::FusedBuild`); the last helper to finish its share makes the end check and sets up the walk (`coop::RoomWalk::new`); all walk it, segments taken from one counter (`512 × (k − 1)` segments); the last thread to leave the walk assembles block i." (https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-team-pipeline-registration.md (at its first commit))
- [2026-11-07T11:27Z] "Otherwise a team of 2 would leave one thread inverting alone and its leader idle for most of each block." (https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-team-pipeline-registration.md (at its first commit))
- [2026-11-07T11:27Z] "At α ≥ 1.6 it predicts 1.01× or less." (https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-team-pipeline-registration.md (at its first commit))
- [2026-11-07T11:27Z] "**What it should do:** cut the CPU that the build waits spend yielding once their spins run out." (https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-pipeline-levers-registration.md (at its first commit))
- [2026-11-07T11:27Z] "- Each small block costs about one block of overlap." (https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-pipeline-levers-registration.md (at its first commit))
- [2026-11-07T11:27Z] "- **The alternative needs more machinery.** Letting the helpers take small blocks would need a second job channel into the pool, a buffer set per helper, and a rule for when helpers stop." (https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-pipeline-levers-registration.md (at its first commit))

**same_quantity_facts:** 10 matching lines in 9 documents dated before commit_to_build, searched on `pipelin|cross.block overlap|\bpark\b|small blocks?` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T19:19Z] docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md (doc): "- **Results:** up to 2.3× over group and software-pipelined prefetching on irregular lookups." — https://github.com/lmmx/mibz2/blob/3e5de265a6ff/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "A scanner cap sits above it for small blocks. On this machine mibz2 at 4 workers is 2.1× its" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/serial-stages.md (doc): "- FALLS: Pipelining stages on SMT siblings: AMD about 0 (-2 to +2%), because the walk is slowed about 2x beside any sibling; Intel -3% to +6.7%; the walk is about 50% of worker co-run time, so the halves balance." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/serial-stages.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/supply.md (doc): "FALSIFIER: A supply-only A/B (a prefetched job per worker, or spin-then-park in recv) that beats 2.8% of 4t wall on a Xeon, 7763 or 9V74 runner. Or an occupancy run where the scanner keeps its own vCPU at 4t, as when pinned apart, and worker idle while it reads stays above 2%." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/supply.md
- [2026-11-07T00:20Z] comment 6699480548 (#162 coordinator): "3. **H1's prediction of 1.10–1.30×** for pipelined 3 × 4 at 12 threads against 3 × 3 at 10. Is the arithmetic sound? Is the range wide enough, given that pipelining raises the live-table count toward 3?" — https://github.com/lmmx/mibz2/pull/162#issuecomment-6699480548
- [2026-11-07T00:21Z] comment 6699486962 (#145 coordinator): "- **H1, pipeline each team:** the leader overlaps the serial stages with the helpers' build and walk, still one table per team. Predicted 1.10–1.30× for pipelined 3 × 4 at 12 threads against 3 × 3 at 10, with falsifiers in the entry. Being prototyped on `research/team-pipeline` (opt-in, default unc…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6699486962
- [2026-11-07T00:29Z] comment 6699591835 (#145 researcher): "3. **H1's 1.10–1.30× is not a supported bound.** The 3×3 grid implies a 1.056 ms whole-run cycle proxy. Taking the entry's own “slows toward four-team inversion” premise (roughly 0.94–0.99 ms) gives only **1.07–1.12× before extra overhead**. A 1.30× gain requires a pipeline cycle ≤0.812 ms. Null an…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6699591835
- [2026-11-07T00:33Z] comment 6699649478 (#145 coordinator): "- The matched causal comparisons (pipelined against not, at 3×4 and 3×3) are kept apart from the product comparison against 3×3@10." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6699649478
- [2026-11-07T00:37Z] comment 6699704291 (#145 coordinator): "- the matched causal comparisons (pipelined or not, at 3×4 and 3×3) are kept separate from the product comparison against 3×3@10;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6699704291

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #247, #249, #250, bench-team-pipeline, bench-pipeline-levers, bench-pipelined-default) / Optimiser: unrecorded (no comment from the role naming #247, #249, #250, bench-team-pipeline, bench-pipeline-levers, bench-pipelined-default)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 4 i9 bundles over 3 recipe(s); coders check whether master moved between them. PR #247's earliest listed commit is 2026-11-07T00:41Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #250's earliest listed commit is 2026-11-07T00:41Z, 10 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## general-plan-rule-a: General plan policy, rule A (topology reader, unpipelined width-2 fallback)

- **PRs:** [#254](https://github.com/lmmx/mibz2/pull/254)
- **Journal sources:** docs/journal/2026-11-07-general-plan-policy.md
- **i9 recipes:** plan-boundaries, plan-bytes, plan-levels
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Louis's PROPOSAL 6712460830 (generalise the plan); design by the Coordinator (LOG 6712504596)
- **commit_to_build:** 2026-11-07T21:22Z, PR #254 first commit (author date) (https://github.com/lmmx/mibz2/pull/254/commits)
- **Cost:** i9 34.1 over 3 bundle(s) min [plan-boundaries/20261108T092231Z: 12.1 min (bundle start (name)→evidence commit); plan-bytes/20261108T093442Z: 7.7 min (bundle start (name)→evidence commit); plan-levels/20261108T094232Z: 14.3 min (bundle start (name)→evidence commit)]; CI: 5 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.51; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Topology::read reads a sysfs root for an affinity: physical cores by (package, core id), the L3 of a CPU in the affinity and whether it holds the whole affinity, and the core-kind signals (Intel's cpu_core/cpu_atom, ARM's cpu_capacity, a max-frequency spread over 20%)." (https://github.com/lmmx/mibz2/pull/254 (first commit message))
- "- `ENVELOPE` is data with one entry: 10 cores, cpu0's L3 = 20 MiB, N ≥ 10 → 3 × 3 pipelined." (https://github.com/lmmx/mibz2/pull/254 (PR body, current text))
- "Intel `cpu_core`/`cpu_atom` (both intersecting the affinity), ARM `cpu_capacity` that differs, and a `cpuinfo_max_freq` spread above 20%." (https://github.com/lmmx/mibz2/pull/254 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-plan-boundaries: "plan policy: rule A's boundaries at 8-9 threads, masks (#254, diagnostic)"
- queue entry bench-plan-bytes: "plan policy: bytes per L3, levels 6/7/9 (#254, diagnostic)"
- queue entry bench-plan-levels: "levels 1-3: 0.2.6 against 0.2.4, and forced 4x2 (#254, diagnostic)"
- "General plan policy, phase 2: rule A, its envelope and a topology reader, opt-in Rule A (the two-budget rule of docs/journal/2026-11-07-general-plan-policy.md) as pure, doc-hidden functions in parallel::general: rule_a, decide_general and plan_general (the envelope first, read as 0.2.6 reads the to…" (https://github.com/lmmx/mibz2/pull/254 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-07T21:31Z] "- An untimed dry run of each recipe, here on a 4-core box: each ended INVALID as it should, on DRY, topology and rounds." (https://github.com/lmmx/mibz2/pull/254 (PR body, current text))
- [2026-11-07T21:31Z] "- CI's paired bench should show **no change on the default path**, because the default's code is unchanged and its plans are proven equal on the grid. ## Deferred (per the journal) - `Policy::General` on the constructors." (https://github.com/lmmx/mibz2/pull/254 (PR body, current text))
- [2026-11-07T21:31Z] "The fixture models 5.3 against 5.1 GHz (a 3.9% spread), but this is unverified on the real machine." (https://github.com/lmmx/mibz2/pull/254 (PR body, current text))

**same_quantity_facts:** 188 matching lines in 65 documents dated before commit_to_build, searched on `rule A|4 ?× ?2|plan|8 threads|width.2` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-07T11:27Z] docs/journal/2026-11-07-pipeline-levers-registration.md (doc): "- Small is above 1 at both plans (+13–15%)." — https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-pipeline-levers-registration.md
- [2026-11-07T11:27Z] docs/journal/2026-11-07-pipelined-default-registration.md (doc): "team3@10, and its medians against the shipped default (mibz2@10, v0.2.5's plan) were 0.142 s" — https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-pipelined-default-registration.md
- [2026-11-07T11:27Z] docs/journal/2026-11-07-team-pipeline-registration.md (doc): "\| `mibz2@10` \| 3 × 3 \| today's default on the i9 (the policy, `Plan::for_threads(10)`) \|" — https://github.com/lmmx/mibz2/blob/0015de8c9612/docs/journal/2026-11-07-team-pipeline-registration.md
- [2026-11-07T12:08Z] comment 6706739698 (#145 coordinator): "- **What changed:** #250 (0015de8c) makes the pipelined team, with park and small, the default at the 3×3 plan alone. It was admitted on the 2026-11-07 terms in decisions.md, after the confirmation `pipelined-default/20261107T105448Z` came back VALID PASS: 1.072 (1.054–1.089) at 10 threads, and abo…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6706739698
- [2026-11-07T13:40Z] comment 6707672921 (#145 louis via chatgpt): "- `docs/journal/2026-11-07-pipelined-default-registration.md` and the confirmed workstation submission `pipelined-default/20261107T105448Z` linked in [#145 comment 6705994828](https://github.com/lmmx/mibz2/pull/145#issuecomment-6705994828), Coordinator's [verified result 6706005681](https://github.…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6707672921
- [2026-11-07T13:43Z] comment 6707704294 (#145 coordinator): "- one machine, one corpus, and the 3×3 plan only;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6707704294
- [2026-11-07T14:20Z] comment 6708094894 (#145 coordinator): "- **Scope:** one machine, one corpus, and the 3×3 plan only." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6708094894
- [2026-11-07T20:13Z] comment 6712460830 (#145 louis via chatgpt): "**Desired research and engineering outcome:** derive and validate a portable, inspectable decision rule that selects among ordinary workers, plain cooperative teams and pipelined teams (park/small as applicable), choosing team count, team width and scheduling mode from the actual budget and relevan…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6712460830
- [2026-11-07T20:17Z] comment 6712504596 (#145 coordinator): "- **Today's state:** `plan_with` gives 3×3 only at exactly 10 physical cores, a 20 MiB L3 and 10 or more threads, and `defaulted` pipelines only that plan. So two decisions (allocation: workers, teams, T and k; and the scheduling mode) are coupled to one machine." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6712504596
- [2026-11-07T21:13Z] comment 6713091254 (#145 coordinator): "- **The live release:** hybrid 10-core / 20 MB Intel parts, some two-socket machines and some VMs get 0.2.6's 3×3 pipelined plan unmeasured, as they have since 0.2.5. Output is unaffected." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6713091254

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6729233463 (2026-11-09T00:18Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #254, bench-plan-boundaries, bench-plan-bytes, bench-plan-levels)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 3 i9 bundles over 3 recipe(s); coders check whether master moved between them.

## service-core: Service core: dedicated service workers for the serial stages (svc2 at 10 threads, target-gated)

- **PRs:** [#251](https://github.com/lmmx/mibz2/pull/251), [#258](https://github.com/lmmx/mibz2/pull/258), [#299](https://github.com/lmmx/mibz2/pull/299), [#302](https://github.com/lmmx/mibz2/pull/302), [#316](https://github.com/lmmx/mibz2/pull/316)
- **Journal sources:** docs/journal/2026-11-09-service-2x3-result.md, docs/journal/2026-11-09-service-guarded-registration.md, docs/journal/2026-11-10-service-guarded-target-gated-registration.md
- **i9 recipes:** service-core, service-2x3, svc-target, service-guarded
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Coordinator's prototype #251 (cut from research/team-pipeline)
- **commit_to_build:** 2026-11-07T12:12Z, PR #251 opened (https://github.com/lmmx/mibz2/pull/251)
- **Cost:** i9 39.5 over 5 bundle(s) min [service-core/20261108T091321Z: 9.0 min (bundle start (name)→evidence commit); service-core/20261108T103216Z: 9.0 min (bundle start (name)→evidence commit); service-2x3/20261109T010708Z: 5.1 min (bundle start (name)→evidence commit); svc-target/20261109T162114Z: 0.2 min (bundle start (name)→evidence commit); service-guarded/20261109T195345Z: 16.2 min (bundle start (name)→evidence commit)]; CI: 34 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 2.93; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The service is the scarce resource, and a block's inversion costs about 2.8× its serial stages." (https://github.com/lmmx/mibz2/pull/251 (PR body, current text))
- "- At 1.39 GB/s of decode, demand is 0.79 of a core." (https://github.com/lmmx/mibz2/pull/251 (PR body, current text))
- "With the service 1.2× dearer on the i9 that falls to 0.98× the default, and at 1.4× to 0.84×." (https://github.com/lmmx/mibz2/pull/251 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-service-core: "service-core decoder (K service threads) against 0.2.6's default (#251)"
- queue entry bench-service-2x3: "the service core at two teams: svc2@10 vs plain/pipelined 2x3 and the 0.2.6 default (#258)"
- queue entry bench-svc-target: "the guarded service default's target preflight on the i9, untimed: predicate, dispatch at 9/10/11 (#302, frozen 278f09ce)"
- queue entry bench-service-guarded: "the guarded service default, timed A/B on the i9 (#302, frozen at d136df7e, bench 3.5.0)"

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-10T10:37Z] "This asks whether the proposed default improves the default that would otherwise ship; a release comparison would answer a different, user-upgrade question and include unrelated changes." (https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-09-service-guarded-registration.md (at its first commit))
- [2026-11-10T10:37Z] "`NOT-FROZEN.txt` then marks the attempt); a `CARGO_*`/`RUST*`/`AB_CENSUS_MS`/ `AB_SAMPLE_MS` override; differing toolchains; a Cargo configuration only the base build would read; the base worktree not clean at the base; a CORPUS without DRY." (https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-09-service-guarded-registration.md (at its first commit))
- [2026-11-10T10:37Z] "The twins at the other six budgets are reported, not judged (answer 4: extending validity to them would change the selection policy, not only the reporting)." (https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-09-service-guarded-registration.md (at its first commit))
- [2026-11-10T10:37Z] "The head runs fewer decoding threads than the base at 10 (8 against 9): no extra-thread label applies." (https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-09-service-guarded-registration.md (at its first commit))
- [2026-11-10T10:37Z] "Outcome INVALID (identity): a DRY run, the corpus not the registered one, the preflight NOT TARGET, the outputs not the registered length, as it should be." (https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-09-service-guarded-registration.md (at its first commit))
- [2026-11-10T10:37Z] "**Not done in this revision.** Moving the corpus build before the preflight, which would shorten the gap between the target observation and the measurements, was suggested in passing (report §2)." (https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-09-service-guarded-registration.md (at its first commit))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `service (core|worker|thread)|svc[12]|finish backlog|prep starvation` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 19 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6729358962 (2026-11-09T00:28Z, after commit_to_build) / Optimiser: 5 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6729279647 (2026-11-09T00:22Z, after commit_to_build)

**monodromy_involved:** yes: cited in the case's own documents (records 4, 33)

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 5 i9 bundles over 4 recipe(s); coders check whether master moved between them. PR #251's earliest listed commit is 2026-11-07T00:41Z, 12 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #302's earliest listed commit is 2026-11-09T12:15Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #316's earliest listed commit is 2026-11-09T12:15Z, 13 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## service-lookahead: Prepared-queue admission threshold for svc2 (lookahead)

- **PRs:** [#271](https://github.com/lmmx/mibz2/pull/271)
- **Journal sources:** docs/journal/2026-11-09-service-lookahead-result.md
- **i9 recipes:** service-lookahead
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Coordinator, after #258
- **commit_to_build:** 2026-11-09T01:25Z, PR #271 first commit (author date) (https://github.com/lmmx/mibz2/pull/271/commits)
- **Cost:** i9 6.7 over 1 bundle(s) min [service-lookahead/20261109T075939Z: 6.7 min (bundle start (name)→evidence commit)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.28; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "So with K service threads the prepared count P can reach L + K − 1; at 2 × 3 + 2, L = 2 already admits three." (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-service-lookahead: "the prepared-queue admission threshold at svc2 (#271)"
- "Max 99, median > max and blocks 0 were VALID \| Every complete timeline round's P, read from the raw runs, must satisfy 1 ≤ median ≤ max ≤ L + K − 1, with the registered 431 blocks, before any aggregate is read." (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))
- "The summary must meet the same invariant and be the median of its rounds \| `test_prepared_max_99_summary`, `test_prepared_max_99_every_round`, `test_prepared_median_above_max`, `test_prepared_above_its_own_ceiling`, `test_one_round_breaches`, `test_no_blocks`, `test_no_blocks_summary`, `test_summa…" (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))
- "This is not a six-complete-round gate \| `test_three_children_instead_of_eighteen`." (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-09T01:27Z] "A service worker starts a preparation while fewer than L blocks are prepared, and preparations under way aren't counted." (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))
- [2026-11-09T01:27Z] "**Fronts aren't a second bottleneck** (the Researcher's bound)." (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))
- [2026-11-09T01:27Z] "So a gain is read as **the effect of the implemented higher-L policy**, not as proof of a queue-supply bottleneck. ## The `src/` change (minimal, opt-in, +75 −1, unchanged by either revision) - **`src/parallel.rs`:** - adds `StreamDecoder::service_admission(teams, k, services, slots, admission, kernels)`, `#[doc(hidden)]`;" (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))
- [2026-11-09T01:27Z] "- About 8 minutes once built: an estimate, not a bound." (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))
- [2026-11-09T01:27Z] "THE ADMISSION THRESHOLD BINDS SVC2@10 AT 2 (an effect of the higher-L policy);" (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))
- [2026-11-09T01:27Z] "- the gaining arms and the relieved arms, named separately." (https://github.com/lmmx/mibz2/pull/271 (PR body, current text))

**same_quantity_facts:** 11 matching lines in 5 documents dated before commit_to_build, searched on `prep(ared)? (queue|starvation)|lookahead|admission threshold` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T09:58Z] docs/journal/2026-10-27-bz2-decoder.md (doc): "scanner's window from the block it is reading. Its lookahead starts at 2 MB and" — https://github.com/lmmx/mibz2/blob/c1aed8c6c476/docs/journal/2026-10-27-bz2-decoder.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/supply.md (doc): "- SURVIVES: 3. Start-up: a fixed 1.8-3.4 ms in which every worker idles (4 MiB first chunk, 2 MB lookahead); 0.5-1.1% of CI 4t wall." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/supply.md
- [2026-11-09T01:13Z] comment 6729896575 (#145 coordinator): "- **The trace:** NOT SATURATED (max utilisation 0.70), but PREP STARVATION (idle_prep 0.165). That fits the prepared queue's lookahead binding, though the run doesn't show it is." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6729896575
- [2026-11-09T01:22Z] comment 6730002640 (#161 coordinator): "**Why it matters now:** bench-service-2x3 ran VALID (`service-2x3/20261109T010708Z`), and the verdict was FOR SVC2@10. Against the 0.2.6 default at 10 threads it gave 1.045 [1.017, 1.073] speed and 14.6% less CPU. A production-qualification plan and a follow-up on the lookahead window are being pre…" — https://github.com/lmmx/mibz2/pull/161#issuecomment-6730002640
- [2026-11-09T01:24Z] docs/journal/2026-11-09-service-2x3-result.md (doc): "- In `src/parallel.rs` the prepared queue's lookahead is `teams`, two blocks at 2 × 3," — https://github.com/lmmx/mibz2/blob/6923ce438195/docs/journal/2026-11-09-service-2x3-result.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 5 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6730484783 (2026-11-09T02:03Z, after commit_to_build, before the first i9 bundle) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6739420751 (2026-11-09T11:54Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6730279906, https://github.com/lmmx/mibz2/pull/145#issuecomment-6730425051, https://github.com/lmmx/mibz2/pull/162#issuecomment-6730378782

## run-fed-stage: Run-fed T-vector stage (line A): T-vector filled from MTF run records

- **PRs:** [#266](https://github.com/lmmx/mibz2/pull/266), [#269](https://github.com/lmmx/mibz2/pull/269)
- **Journal sources:** docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md, docs/journal/2026-11-09-run-fed-stage-result.md
- **i9 recipes:** none
- **Deciding machine:** CI (sweep.yml registered diagnostic)
- **Origin (as recorded):** Louis via ChatGPT, #145 comment 6728860546 (per PR #269)
- **commit_to_build:** 2026-11-08T20:31Z, PR #266 first commit (author date) (https://github.com/lmmx/mibz2/pull/266/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.25; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **Records** cost 0.58–0.87 B a row pooled, and up to 1.6–2.4 B a row in the worst block." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- "- 82% of Wikidata's rows already go through `build_one_pass`'s 16-row run stores." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- "- Both builds come to about 2.5–2.6 instructions a row." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))

**must_hold** (quoted):

- "The paired bench should show no change. ## Line A: the existing T-vector filled from run records **Proved:** - Ψ's F intervals and L's runs tile [0, n) with r intervals each, so their common refinement has at most 2r − 1 cells." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- "**Against the gate** (a walk under about 4 c/B), by the instruction model:" (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- "- At the T walk's measured rate, that is about 13 c/B, three times the gate, even with a free build." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- "- run_stage::floor: the table's stores alone, a probe for the build's floor. examples/run_fed_stage.rs times the whole stage phase by phase in decode order, paired by round: the column (A), the records (B), the column with checkpoints (C, the pipelined leader's MTF) and the floor (D), with counters…" (https://github.com/lmmx/mibz2/pull/269 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-08T20:32Z] "The paired bench should show no change. ## Line A: the existing T-vector filled from run records **Proved:** - Ψ's F intervals and L's runs tile [0, n) with r intervals each, so their common refinement has at most 2r − 1 cells." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- [2026-11-08T20:32Z] "Each case is built five ways: plain and 8-wide fills, maximal and checkpoint-cut runs, and in 1–5 shares." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- [2026-11-08T20:32Z] "**What production would need:** - **No change** for `orig_ptr` or segment starts, since the table is identical." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- [2026-11-08T20:32Z] "An O(r) tiling check would replace `FusedBuild`'s soundness argument." (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- [2026-11-08T20:32Z] "**Cost model** (instructions from the code's shape, unmeasured):" (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))
- [2026-11-08T20:32Z] "**Against the gate** (a walk under about 4 c/B), by the instruction model:" (https://github.com/lmmx/mibz2/pull/266 (PR body, current text))

**same_quantity_facts:** 3 matching lines in 2 documents dated before commit_to_build, searched on `run.fed|run records|MTF runs?|16.row run path` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/serial-stages.md (doc): "CLAIM: T-vector build: not a random scatter. It writes 256 sequential bucket streams, and 81.9% of rows go through the 16-row run path. At 1t its cost is a store-forward chain on ftab[c]: the 60.1% of one-row steps that repeat the previous byte reload a cursor stored one row earlier, about 5-6 cycl…" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/serial-stages.md
- [2026-11-08T18:20Z] comment 6725272955 (#145 louis via chatgpt): "The scientific result of [§5 lines 336–360](https://github.com/lmmx/mibz2/blob/02a98d58850baf675717482d20ec32edcec2699f/docs/site/blog/2026-11-08-bwt-runs-from-mtf-codes.md#L336-L360) is strong and should stay: T-vector build+walk about 12.0 cycles/byte, whereas the tested move walk **alone** took …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6725272955

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 6 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6729473179 (2026-11-09T00:37Z, after commit_to_build) / Optimiser: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6730842547 (2026-11-09T02:32Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

**Notes:** PR #269's earliest listed commit is 2026-11-09T00:19Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## run-fed-leader: Run-fed leader for the pipelined teams

- **PRs:** [#270](https://github.com/lmmx/mibz2/pull/270), [#305](https://github.com/lmmx/mibz2/pull/305)
- **Journal sources:** docs/journal/2026-11-09-run-fed-leader-result.md, docs/journal/2026-11-09-run-fed-leader-workstation-result.md, docs/journal/2026-11-09-run-fed-leader-second-attempt-result.md
- **i9 recipes:** run-fed-leader
- **Deciding machine:** CI diagnostic, then i9-10900K (two attempts)
- **Origin (as recorded):** Coordinator (per PR #270 body; Researcher and Guardian reviews named)
- **commit_to_build:** 2026-11-09T01:14Z, PR #270 opened (https://github.com/lmmx/mibz2/pull/270)
- **Cost:** i9 4.3 over 2 bundle(s) min [run-fed-leader/20261109T080633Z: 2.0 min (bundle start (name)→evidence commit); run-fed-leader/20261109T201013Z: 2.3 min (bundle start (name)→evidence commit)]; CI: 18 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.81; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "`§8` says "six observations per mode". ## Revision 5 (`28298a4a`): the plan read from the decoders' own record (Guardian D059) - **The defect (D059's reproduction of §R4's counterexample).** ab's census polls `/proc` every 5 ms, and a new thread carries its spawner's name until it names itself." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- "- **Repeat run:** 350 runs (50 rounds × 7 arms, 10 threads) on the 20 ms corpus." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- "- **Disclosed:** on a 20 ms decode, ab's thread census once caught a pool helper before it named itself, so the plan read (4, 0, 5)." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-run-fed-leader: "the run-fed leader against default33, with the self-recorded thread census (#270)"
- "- run_stage::floor: the table's stores alone, a probe for the build's floor. examples/run_fed_stage.rs times the whole stage phase by phase in decode order, paired by round: the column (A), the records (B), the column with checkpoints (C, the pipelined leader's MTF) and the floor (D), with counters…" (https://github.com/lmmx/mibz2/pull/270 (first commit message))
- "The same snapshot rule in every arm is not claimed to leave out an equal cost across arms." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- "- **Repeat run:** 350 runs (50 rounds × 7 arms, 10 threads) on the 20 ms corpus." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-09T01:14Z] "- **premise:** it never unwinds;" (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- [2026-11-09T01:14Z] "The "uncontended", "small" and equal-cost claims are removed;" (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- [2026-11-09T01:14Z] "The leader's sort (`sort_one_share`, the helpers' prefix `merge_shares` in one share) refused any tiling of fewer than two records, while `run_stage::sort` takes `[0]`." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- [2026-11-09T01:14Z] "- **The fuzz loop now replays one case** (`MIBZ2_FUZZ_CASE`, `MIBZ2_FUZZ_REPEAT`) and saves any rebuild's input, so CI uploads it." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- [2026-11-09T01:14Z] "With the debug assertion also removed, it aborts with heap corruption natively and is UB under Miri at the witness (a dangling `&mut [[u32; 256]; 3]`)." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))
- [2026-11-09T01:14Z] "Every check takes a tiling exactly when an independent model says it tiles, and then makes its table." (https://github.com/lmmx/mibz2/pull/270 (PR body, current text))

**same_quantity_facts:** 39 matching lines in 28 documents dated before commit_to_build, searched on `run.fed|leader|pipelined team` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-07T21:13Z] comment 6713091254 (#145 coordinator): "- **The design:** rule A. It uses width-3 pipelined teams where the L3 holds them (about 6 MB a team) and they fill the whole budget, and otherwise 0.2.6's width-2 plan, or workers below 6 cores." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6713091254
- [2026-11-08T01:01Z] comment 6715369675 (#162 coordinator): "- **Its shape:** rule A ("two budgets", width-3 pipelined teams where the L3 holds about 6 MB a team and the plan fills the budget), plus an evidence envelope that keeps 0.2.6's choice by default." — https://github.com/lmmx/mibz2/pull/162#issuecomment-6715369675
- [2026-11-08T01:01Z] comment 6715370785 (#160 coordinator): "- **α (the parallel-stage slowdown when the leader leaves) is not identified:** fitted values of 1.39–1.55 depend on the assumed handoff cost." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6715370785
- [2026-11-08T20:38Z] docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md (doc): "- **The pipelined teams** (3 × 3, the release plan on 6 or more physical cores):" — https://github.com/lmmx/mibz2/blob/a4f97714cc14/docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md
- [2026-11-08T23:46Z] comment 6728860546 (#145 louis via chatgpt): "1. **Explicitly reconcile #266's objections before spending effort.** The production `build_one_pass` already uses 16-row run stores for 82% of Wikidata, and its model is ~2.6 instructions/row against run-fed ~2.5; both use the same 4-byte-per-row T-vector, no meaningful peak-RSS saving follows, an…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6728860546
- [2026-11-09T00:22Z] comment 6729279647 (#145 optimizer): "**Job cap: census first.** The default park+small arm reports 229.5 overlaps and 22.4 ms summed join delay; 236.5/26 ms belong to small-only. The later census implies the 198 small blocks contain just **8,895 BWT rows**, about 45 each. Count is not work. The leader's extra admissions search past ti…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6729279647
- [2026-11-09T00:34Z] comment 6729426658 (#160 coordinator): "**The question to you:** rank constructions that turn this into a decode-wall win in the shipped pipelined 3×3 with park and small, or in another plan. Give each one its mechanism, its expected saving on the leader's critical path or in total CPU, its risks, and the cheapest CI falsifier. Candidate…" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6729426658
- [2026-11-09T00:44Z] comment 6729548266 (#145 optimizer): "2. **C30, nearest run-fed integration:** leader emits; two helpers count, prefix, scatter and fill bounded row shares. Parallel stable sort needs both record-count and row-length histograms. Gross leader release is M−E−new leader work; charge helper sort, barriers, fill, and changed walk joining. T…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6729548266
- [2026-11-09T00:56Z] comment 6729693845 (#145 researcher): "**Continue through distinct constructions. Strongest expected winner: cheaper leader, present dense walk.** C29's matched MTF-list control comes first: the indicative 25% also changes the list kernel. Compare helper-side sorted fill with the Optimiser's C31 direct placement. I independently checked…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6729693845
- [2026-11-09T01:13Z] comment 6729896575 (#145 coordinator): "- svc2@10 replicates the earlier exploratory 1.045 and does it with about 13% less CPU. Two plain or pipelined teams alone don't do it." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6729896575

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 11 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6730026367 (2026-11-09T01:24Z, after commit_to_build, before the first i9 bundle) / Optimiser: 3 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6730842547 (2026-11-09T02:32Z, after commit_to_build, before the first i9 bundle)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6729932826, https://github.com/lmmx/mibz2/pull/145#issuecomment-6730425051, https://github.com/lmmx/mibz2/pull/160#issuecomment-6729932062

**Notes:** 2 i9 bundles over 1 recipe(s); coders check whether master moved between them. PR #270's earliest listed commit is 2026-11-09T00:19Z, 1 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## pair-from-runs: Pair table built from run intersections

- **PRs:** [#272](https://github.com/lmmx/mibz2/pull/272)
- **Journal sources:** docs/journal/2026-11-09-pair-from-runs-result.md
- **i9 recipes:** none
- **Deciding machine:** CI (registered diagnostic)
- **Origin (as recorded):** per PR #272 body (Researcher named)
- **commit_to_build:** 2026-11-09T01:42Z, PR #272 first commit (author date) (https://github.com/lmmx/mibz2/pull/272/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.08; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "`miri.yml` adds the AVX2 step, the control and the doc tests. ## Census (counts) - **Wikidata:** r/n 0.0722, cells/n 0.0881 (#266's figure), cells/r 1.220." (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))
- "- 0.342% of rows are flagged statically and 0.343% dynamically, with at most 13 flagged blocks per block;" (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))
- "- the old lookup's correction loop runs 0.002 times a step, so the prize is small. ## Registration (draft) The judged run is one copy pinned to one CPU, on Wikidata." (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))

**must_hold** (quoted):

- "- examples/pair_runs.rs: the diagnostic, (a) build cost row vs run vs one-hop and (b) the whole in-place entry point, production's Segmented vs the run-built pair path vs #34; every block validated first; raw paired timings, head and corpus identity; the spare-bit census. sweep.yml takes `tool: pai…" (https://github.com/lmmx/mibz2/pull/272 (first commit message))
- "- an 8-wide fill in rising q (AVX2, or SSE2), serial only: its stores spill into later cells, so it must never run in parallel across adjacent cells." (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))
- "- (b) the `invert_in_place` inverse-BWT entry point (not the whole decoder): production's `Segmented` (P), the run-built pair path with its admission rule (R), and #34 as it was (O)." (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))
- "- the old lookup's correction loop runs 0.002 times a step, so the prize is small. ## Registration (draft) The judged run is one copy pinned to one CPU, on Wikidata." (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-09T01:42Z] "Research: the pair table built from run intersections (opt-in), registration drafted - src/pair_runs.rs, behind the non-default `pair-runs` feature (and the tests): #34's pair table, per-row builder, pair_start and flagged 32-lane pair walk, recovered from perf/pair-walk (a3687d8f), the walk cut into setup and walk so it runs in place; and build_pairs_runs, the same words from the last column's r…" (https://github.com/lmmx/mibz2/pull/272 (first commit message))
- [2026-11-09T01:42Z] "It is cut into `setup` and `walk` so that it runs in place, as production's `Segmented` does." (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))
- [2026-11-09T01:42Z] "- **`examples/pair_runs.rs`, the diagnostic, run as `sweep.yml` `tool: pair_runs`:** - (a) build cost only: row, run, and the one-hop build as a reference." (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))
- [2026-11-09T01:42Z] "- POSITIVE: ≤ 0.95, hi < 1, and the worst synthetic point estimate ≤ 1.05 (a screen);" (https://github.com/lmmx/mibz2/pull/272 (PR body, current text))

**same_quantity_facts:** 18 matching lines in 8 documents dated before commit_to_build, searched on `pair table|run intersections?|ψ²|Ψ²` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T15:46Z] docs/journal/2026-10-27-literature-reading.md (doc): "\| #29 \| LF² as `LF[LF[i]]`: a gather pass and an 8-byte pair table (#6) \| +9.3% / +20.7% \| rejected \|" — https://github.com/lmmx/mibz2/blob/24169e657c1f/docs/journal/2026-10-27-literature-reading.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-pair-walk.md (doc): "libsais walks a ψ² table in ≈6.7 c/B, against our port's ≈10, so its decode loop does" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-pair-walk.md
- [2026-10-27T17:50Z] docs/journal/2026-10-27-research-synthesis.md (doc): "\| #6 LF² / ψ² \| 1: libsais's run lanes + 2-byte walk beat our build + walk by 34% on Wikidata, byte-identical \| reopened, ported (#34), slower: **closed**. The walk-loop question is queued \|" — https://github.com/lmmx/mibz2/blob/f398966ed883/docs/journal/2026-10-27-research-synthesis.md
- [2026-10-27T19:19Z] docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md (doc): "- #29's 8-byte pair table (7.2 MB a block) exceeded Skylake's reach and not Zen 4's. That fits" — https://github.com/lmmx/mibz2/blob/3e5de265a6ff/docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "\| E6 \| libsais's walk parameters in our ψ² port (24 lanes, a round of prefetch lead, 256-row chains) \| as libsais \| the pair-walk branch \| VM, then ~15 min \| as #6's (byte checks) \| — \| **only if E1 passes:** ≥ libsais's s4x1 / p4 on the i9, and no worse than 1.0 × production on CI's 2-core…" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md
- [2026-11-07T09:26Z] docs/journal/2026-11-07-left-field-archive-check.md (doc): "- a ψ² build shown untimed (callgrind) at no more than today's one-pass build plus 1 c/B." — https://github.com/lmmx/mibz2/blob/8b7c1d175e94/docs/journal/2026-11-07-left-field-archive-check.md
- [2026-11-08T20:09Z] comment 6726489772 (#145 louis via chatgpt): "**Why it might win, and why it probably won't without aggressive streamlining:** the single-hop move has ~41 instructions per *byte*. A 2-byte record might amortise record decode, fast-forwards, prefetch and state updates over *two* output bytes, with fewer dependent loads and a table still materia…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6726489772
- [2026-11-08T23:46Z] comment 6728860546 (#145 louis via chatgpt): "1. **Explicitly reconcile #266's objections before spending effort.** The production `build_one_pass` already uses 16-row run stores for 82% of Wikidata, and its model is ~2.6 instructions/row against run-fed ~2.5; both use the same 4-byte-per-row T-vector, no meaningful peak-RSS saving follows, an…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6728860546

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 5 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6730363373 (2026-11-09T01:53Z, after commit_to_build) / Optimiser: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6730372937 (2026-11-09T01:54Z, after commit_to_build)

**monodromy_involved:** yes: cited in the case's own documents (records 3)

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6730279906, https://github.com/lmmx/mibz2/pull/145#issuecomment-6730300928, https://github.com/lmmx/mibz2/pull/145#issuecomment-6730312307

## run-fed-direct: C31's direct placement (arm D) behind run-fed

- **PRs:** [#277](https://github.com/lmmx/mibz2/pull/277)
- **Journal sources:** docs/journal/2026-11-09-run-fed-direct-result.md
- **i9 recipes:** none
- **Deciding machine:** CI (registered diagnostic)
- **Origin (as recorded):** C31 (Optimiser/Researcher ledger, per PR #277)
- **commit_to_build:** 2026-11-09T04:06Z, PR #277 opened (https://github.com/lmmx/mibz2/pull/277)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.06; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- "- run_stage::floor: the table's stores alone, a probe for the build's floor. examples/run_fed_stage.rs times the whole stage phase by phase in decode order, paired by round: the column (A), the records (B), the column with checkpoints (C, the pipelined leader's MTF) and the floor (D), with counters…" (https://github.com/lmmx/mibz2/pull/277 (first commit message))
- "- `miri_direct_unchecked_states_refused`: every state D must never accept is refused." (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))
- "**36** mutations that must each read INVALID." (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))
- "They are paired by round, at the entry point." (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-09T04:06Z] "- **The cells** are cut where an L run ends." (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))
- [2026-11-09T04:06Z] "Production's room walk consumes the table unchanged. ## What it removes, against B (#269) It removes:" (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))
- [2026-11-09T04:06Z] "- its predicted table stores, counted from the source's store loops, not machine code;" (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))
- [2026-11-09T04:06Z] "Each is saved and minimised." (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))
- [2026-11-09T04:06Z] "- M3, the final tiling check removed: survives, as an equivalent mutant (D055 §1)." (https://github.com/lmmx/mibz2/pull/277 (PR body, current text))

**same_quantity_facts:** 11 matching lines in 9 documents dated before commit_to_build, searched on `C31|direct placement|run.fed` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-08T23:46Z] comment 6728860546 (#145 louis via chatgpt): "1. **Explicitly reconcile #266's objections before spending effort.** The production `build_one_pass` already uses 16-row run stores for 82% of Wikidata, and its model is ~2.6 instructions/row against run-fed ~2.5; both use the same 4-byte-per-row T-vector, no meaningful peak-RSS saving follows, an…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6728860546
- [2026-11-09T00:44Z] comment 6729548266 (#145 optimizer): "2. **C30, nearest run-fed integration:** leader emits; two helpers count, prefix, scatter and fill bounded row shares. Parallel stable sort needs both record-count and row-length histograms. Gross leader release is M−E−new leader work; charge helper sort, barriers, fill, and changed walk joining. T…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6729548266
- [2026-11-09T00:56Z] comment 6729693845 (#145 researcher): "**Continue through distinct constructions. Strongest expected winner: cheaper leader, present dense walk.** C29's matched MTF-list control comes first: the indicative 25% also changes the list kernel. Compare helper-side sorted fill with the Optimiser's C31 direct placement. I independently checked…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6729693845
- [2026-11-09T01:16Z] comment 6729932062 (#160 coordinator): "- **The C31 census:** about 163 predecessor lookups a block, against about 45k scatters removed. The C32 count is W/n ≈ 1.59." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6729932062
- [2026-11-09T01:38Z] comment 6730190055 (#222 coordinator): "- **#270**, the run-fed leader, found two things. The leader's 25% is mostly the list kernel (C29). The helpers' build is the critical path. The Researcher and the Optimiser both now favour C31, direct placement by the helpers, as the primary. It is being revised." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6730190055
- [2026-11-09T02:25Z] docs/journal/2026-11-09-run-fed-stage-result.md (doc): "1. **Direct placement with no sort (C31):** it removes the 0.317 of sort and its F array." — https://github.com/lmmx/mibz2/blob/66edecbe2b19/docs/journal/2026-11-09-run-fed-stage-result.md
- [2026-11-09T02:26Z] comment 6730769410 (#160 coordinator): "1. **The sort, 0.317:** C31's direct placement removes it. With the no-sort fill at 1.332 against 1.432, is a whole-stage win on one worker plausible? What does C31's lookup cost on this corpus?" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6730769410
- [2026-11-09T02:36Z] comment 6730895951 (#222 coordinator): "1. **The run-fed line's first measured answer.** #269's registered CI diagnostic, run 42549324025 at `fd60fffa`, was **VALID, Q1 STOP**. Wikidata's R_stage was 1.159 [1.139, 1.180] on one pinned worker (journal `66edecbe`, LOG 6730764444)." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6730895951
- [2026-11-09T03:45Z] comment 6731728422 (#145 optimizer): "1. **C31 one-hop direct-F first.** From emitted records, remove pair L reconstruction, first-symbol directory, fi/js composition metadata and pair-start/LF work; retain checked counts, prefixes, resumed cursors, F/L intersections, stores and ownership tails. Emit production T[q]=L[q]\|Psi(q)<<8 and…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6731728422

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 4 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6732362367 (2026-11-09T04:37Z, after commit_to_build) / Optimiser: 4 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6732797905 (2026-11-09T05:13Z, after commit_to_build)

**monodromy_involved:** yes: cited in the case's own documents (records 1)

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6731991703, https://github.com/lmmx/mibz2/pull/145#issuecomment-6732365462, https://github.com/lmmx/mibz2/pull/161#issuecomment-6732361106

**Notes:** PR #277's earliest listed commit is 2026-11-09T00:19Z, 4 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## c29-register-checkpoints: C29 register-list checkpoint kernel in the shipped pipelined front

- **PRs:** [#281](https://github.com/lmmx/mibz2/pull/281), [#288](https://github.com/lmmx/mibz2/pull/288)
- **Journal sources:** docs/journal/2026-11-09-c29-r5-result.md
- **i9 recipes:** none
- **Deciding machine:** CI
- **Origin (as recorded):** C29 (ledger item, per PR #281)
- **commit_to_build:** 2026-11-09T05:46Z, PR #281 opened (https://github.com/lmmx/mibz2/pull/281)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 7 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.32; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "It runs 20 rounds at 9 threads (3×3, judged) and at 3 threads (1×3, secondary, never judged)." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- "- Wikidata at 1 thread: wall and CPU **+1.1%** (+0.7, +1.4)." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- "- Its favourable 9-thread wall ratio, **0.9700 [0.9647, 0.9753]**, is disclosed." (https://github.com/lmmx/mibz2/pull/288 (PR body, current text))

**must_hold** (quoted):

- "This is #270's kernel from `03e52206`, unchanged except for comments, and no longer feature-gated." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- "Default off keeps both kernels in one binary, so the diagnostic is a clean paired A/B." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- "A default-on change is left to its own production gate." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- "It compares `mibz2-pipe3-both` (the control) with `mibz2-pipe3-both-c29`, from one build, paired by round." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-09T05:46Z] "- **Why default off:** #270 measured the gain in a copied research worker, on CI's 9-on-4 vCPUs. bench.yml's cells never run the pipelined path." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- [2026-11-09T05:46Z] "Every class of an independent model is reached, with the same counts as D054:" (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- [2026-11-09T05:46Z] "- **No minimal source fix restores base's offsets.** `#[cold]` or a separate path would not shrink the sections ahead of `.text`, and padding would be a hack." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- [2026-11-09T05:46Z] "Benching it against base would show whether the 1-thread cell is layout." (https://github.com/lmmx/mibz2/pull/281 (PR body, current text))
- [2026-11-09T08:56Z] "It exits 0 if the run would be judged, 3 if not." (https://github.com/lmmx/mibz2/pull/288 (PR body, current text))
- [2026-11-09T08:56Z] "`bench/test_c29_checkpoints.py` runs each fixture through `c29_identity.build` and the reporter, against a hand-written table of expected outcomes:" (https://github.com/lmmx/mibz2/pull/288 (PR body, current text))

**same_quantity_facts:** 46 matching lines in 37 documents dated before commit_to_build, searched on `C29|register.list|checkpoint` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-09T05:02Z] comment 6732659575 (#160 coordinator): "3. **ls against dir.** `ls` does its sort in the leader (0.215 ms) with no helper sort, while dir pays 0.754 ms of helper sort and 163 lookups a block. Should ls be the next registered primary? If so, what threshold, and what would distinguish it from C29 alone?" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6732659575
- [2026-11-09T05:02Z] comment 6732661221 (#222 coordinator): "- **The caveat for the account:** most of the gain reproduces with C29's register-list MTF kernel alone (reg 0.9745; dir/reg 0.992). The records add about 0.8% (dir) and about 2.5% (ls, never judged)." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6732661221
- [2026-11-09T05:06Z] comment 6732711502 (#145 observer): "- **4/1 — attribution is not a unique mechanism:** C29 alone supplies most of #270's measured gain; direct placement adds about 0.8% on this CI worker, with target-workstation effect unestablished ([result](https://github.com/lmmx/mibz2/pull/145#issuecomment-6732659255))." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6732711502
- [2026-11-09T05:10Z] comment 6732761085 (#145 researcher): "- The added dir/reg .992134 [.986550,.997749] and ls/reg .974647 [.969726,.979592] are sound paired descriptive contrasts, explicitly post-run. They neither replace the reference nor add a retrospective gate. “Direct fill adds .8%” is too causal: reg uses C29 on all blocks, while record arms retain…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6732761085
- [2026-11-09T05:11Z] docs/journal/2026-11-09-run-fed-leader-result.md (doc): "2. **"The direct fill adds 0.8%" would be too causal.** `reg` uses C29 on every block. The record" — https://github.com/lmmx/mibz2/blob/1c09aecb7b3b/docs/journal/2026-11-09-run-fed-leader-result.md
- [2026-11-09T05:13Z] comment 6732797905 (#145 optimizer): "2. **Fresh LS/matched-C29 registration**, with common rejected/small-block policy. Proposed diagnostic hurdle: at least2% incremental wall, upper95%<1, CPU upper≤1.02; prospective, not a production-gate change. Preserve LS as secondary in #270's transfer. LS/HS is the cleaner placement contrast; LS…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6732797905
- [2026-11-09T05:14Z] comment 6732803728 (#145 coordinator): "2. **A fresh registration of ls against matched C29,** with a common policy for rejected and small blocks. The hurdle would be at least 2% incremental wall, upper 95% < 1, and CPU upper ≤ 1.02. It is queued after C29. ls stays a secondary in #270's workstation transfer." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6732803728
- [2026-11-09T05:29Z] comment 6733004424 (#160 coordinator): "1. **C37 and C38 as bounded consumer diagnostics on a pre-built table:** what each must save per step to clear your 34.4% residual-proxy reduction. Should either be built before or after C29's production prototype, which is now under construction?" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6733004424
- [2026-11-09T05:36Z] comment 6733103415 (#145 optimizer): "**Rank: C29 production evidence first; existing matched LS/direct-F work next; C37, then conditional C38.** Each standalone consumer must save over **4.17669 nominal residual-equivalent cycles per ordinary two-byte pair step** to beat room parity (12.14426 → 7.96757;34.3923%). These are planning pr…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6733103415
- [2026-11-09T05:39Z] comment 6733146738 (#145 researcher): "**Pause the measured self-repeating packet arm. Nothing here displaces C31 or C29. Revise the universal exclusion of BWT-domain certification and the alleged 10% Precopy ceiling.** In reply to inbox **6733003420**. [Report, reproduced aggregates and independent finite checks](https://github.com/lmm…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6733146738

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 3 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6734359855 (2026-11-09T07:08Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #281, #288)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6733270759, https://github.com/lmmx/mibz2/pull/145#issuecomment-6733880555, https://github.com/lmmx/mibz2/pull/161#issuecomment-6733269661

**Notes:** PR #281's earliest listed commit is 2026-11-09T05:25Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #288's earliest listed commit is 2026-11-09T05:25Z, 4 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## build-window-runs: Build cuts each window into its runs, one cursor step a run (v1, then v2 with a classifier)

- **PRs:** [#310](https://github.com/lmmx/mibz2/pull/310), [#312](https://github.com/lmmx/mibz2/pull/312)
- **Journal sources:** docs/journal/2026-11-09-build-window-runs.md, docs/journal/2026-11-09-build-window-runs-v2.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** per PR #310 body (no proposer named)
- **commit_to_build:** 2026-11-09T21:41Z, PR #310 opened (https://github.com/lmmx/mibz2/pull/310)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.10; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "On wikidata-independent.bz2, 9.6% of all rows are such links inside mixed windows." (https://github.com/lmmx/mibz2/pull/310 (first commit message))
- "-2.6% to -3.6% instructions on full blocks, all in the build (-10% to -13%), walk identical, output identical." (https://github.com/lmmx/mibz2/pull/310 (first commit message))
- "On wikidata-independent.bz2, 9.6% of all rows are such links inside mixed windows." (https://github.com/lmmx/mibz2/pull/312 (first commit message))

**must_hold** (quoted):

- "- **The instruction saving is almost all #169's.** At `RUNS` = 1 the build is 4.63M on block 1, against 4.65M at 6. #169's instruction cut alone did not show end to end." (https://github.com/lmmx/mibz2/pull/310 (PR body, current text))
- "- **`wikidata`:** −1% to −3% at 1 thread, and −1% to −4% at 4 threads." (https://github.com/lmmx/mibz2/pull/310 (PR body, current text))
- "**Rule:** - **Correctness:** required." (https://github.com/lmmx/mibz2/pull/310 (PR body, current text))
- "The cause is not established; the journal records it as a risk for CI to settle. ## Decision rule The rule is #310's:" (https://github.com/lmmx/mibz2/pull/312 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-09T21:41Z] "perf(bwt): the build cuts each window into its runs, one cursor step a run build_one_pass took any window that was not 16 equal bytes row by row." (https://github.com/lmmx/mibz2/pull/310 (first commit message))
- [2026-11-09T21:41Z] "perf(bwt): the build cuts each window into its runs, one cursor step a run build_one_pass took any window that was not 16 equal bytes row by row." (https://github.com/lmmx/mibz2/pull/312 (first commit message))
- [2026-11-09T21:41Z] "- **A run with fewer than 16 slots left in its bucket:** one-row steps." (https://github.com/lmmx/mibz2/pull/310 (PR body, current text))
- [2026-11-09T21:41Z] "- **The instruction saving is almost all #169's.** At `RUNS` = 1 the build is 4.63M on block 1, against 4.65M at 6. #169's instruction cut alone did not show end to end." (https://github.com/lmmx/mibz2/pull/310 (PR body, current text))
- [2026-11-09T21:41Z] "This PR's claim is the chain it removes, not its count." (https://github.com/lmmx/mibz2/pull/310 (PR body, current text))
- [2026-11-09T21:41Z] "- Callgrind's simulated predictor gives +5k mispredictions on block 1, against about 86k chain links removed." (https://github.com/lmmx/mibz2/pull/310 (PR body, current text))

**same_quantity_facts:** 3 matching lines in 2 documents dated before commit_to_build, searched on `windows? (cut )?into (its )?runs|cursor step|16.row run path|run path|classifier|high.entropy window` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/serial-stages.md (doc): "RECOV: Build's 4t share: 69 ms (20.1%) on 8573C, 71 ms (15.6%) on 7763, 26 ms (14.3%) on 9V45, 75 ms (19.0%) on 8370C. Recoverable from breaking the same-byte chain (taking runs of 2-15 equal bytes with one cursor step, or interleaving 2-4 row ranges with cursors from mtf_checkpoints) plus run16 ve…" — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/serial-stages.md
- [2026-11-08T20:38Z] docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md (doc): "stores 82% of Wikidata's rows through its 16-row run path, and the model puts both builds" — https://github.com/lmmx/mibz2/blob/a4f97714cc14/docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #310, #312) / Optimiser: unrecorded (no comment from the role naming #310, #312)

**monodromy_involved:** yes: cited in the case's own documents (records 0, 5, 10)

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/161#issuecomment-6752605812

**Notes:** PR #310's earliest listed commit is 2026-11-09T21:41Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #312's earliest listed commit is 2026-11-09T21:41Z, 1 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## pool-wait-budgets: Pool-wait spin budgets (0, 64) and split helper-idle / caller-join waits

- **PRs:** [#321](https://github.com/lmmx/mibz2/pull/321), [#325](https://github.com/lmmx/mibz2/pull/325)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI screen (EPYC, 4 vCPUs on 2 cores)
- **Origin (as recorded):** the Optimiser's buildable brief A, #145 6756418444 (per PR #321)
- **commit_to_build:** 2026-11-10T02:05Z, PR #321 opened (https://github.com/lmmx/mibz2/pull/321)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.06; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Each is `mibz2-pipe3-both` with the pool budget changed; at 9 threads that is the default's own 3×3 decoder." (https://github.com/lmmx/mibz2/pull/321 (PR body, current text))
- "It also fixes the grid (7 corpora × threads 1, 2, 4, 9; 18 rounds; wall and CPU of arm/A, paired geometric mean with a 95% t interval)." (https://github.com/lmmx/mibz2/pull/321 (PR body, current text))
- "- Requested threads of 1, 2 and 4 all force the 1×3 plan." (https://github.com/lmmx/mibz2/pull/325 (PR body, current text))

**must_hold** (quoted):

- "- docs/journal/2026-11-10-pool-wait-budgets.md registers the hypothesis, arms, grid, decision rule and rejection condition before any timing; .github/workflows/pool-wait.yml is the one CI screen, run on the label bench:pool-wait." (https://github.com/lmmx/mibz2/pull/321 (first commit message))
- "- docs/journal/2026-11-10-pool-wait-budgets.md registers the hypothesis, arms, grid, decision rule and rejection condition before any timing; .github/workflows/pool-wait.yml is the one CI screen, run on the label bench:pool-wait." (https://github.com/lmmx/mibz2/pull/325 (first commit message))
- "Each is `mibz2-pipe3-both` with the pool budget changed; at 9 threads that is the default's own 3×3 decoder." (https://github.com/lmmx/mibz2/pull/321 (PR body, current text))
- "The stress must hang under every budget, and locally it does." (https://github.com/lmmx/mibz2/pull/321 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-10T02:04Z] "- docs/journal/2026-11-10-pool-wait-budgets.md registers the hypothesis, arms, grid, decision rule and rejection condition before any timing; .github/workflows/pool-wait.yml is the one CI screen, run on the label bench:pool-wait." (https://github.com/lmmx/mibz2/pull/321 (first commit message))
- [2026-11-10T02:04Z] "- docs/journal/2026-11-10-pool-wait-budgets.md registers the hypothesis, arms, grid, decision rule and rejection condition before any timing; .github/workflows/pool-wait.yml is the one CI screen, run on the label bench:pool-wait." (https://github.com/lmmx/mibz2/pull/325 (first commit message))
- [2026-11-10T02:05Z] "It sets out the hypothesis, the arms and the bound priced from the spin audit's counts." (https://github.com/lmmx/mibz2/pull/321 (PR body, current text))
- [2026-11-10T02:05Z] "On CI's runners the default is production's workers, so `bench.yml`'s head-over-base should show no change." (https://github.com/lmmx/mibz2/pull/321 (PR body, current text))

**same_quantity_facts:** 71 matching lines in 34 documents dated before commit_to_build, searched on `spin|pool wait|park|helper.idle|caller.join` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-09T16:47Z] comment 6745817777 (#145 coordinator): "2. **The preflight isn't passive.** `ab env` spins 2×ncpu threads and calls `Perf::probe`, and its first call has no time limit." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6745817777
- [2026-11-09T21:57Z] docs/journal/2026-11-09-spin-audit.md (doc): "about 140 cycles, at 3.5–4.7 GHz (an assumption), the bound of 3.3 M spins is about" — https://github.com/lmmx/mibz2/blob/64c1eb411752/docs/journal/2026-11-09-spin-audit.md
- [2026-11-09T21:57Z] comment 6752540754 (#145 coordinator): "**The lead it produced.** At every wait site, 94 to 100% of spins are wasted: they come before a park anyway. The earlier `spin2k` lever only tested the build-end wait. A per-site budget for the pool's waits, chosen from the counts, could cut CPU on CI as well." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6752540754
- [2026-11-09T21:59Z] comment 6752576037 (#145 coordinator): "2. **A frozen i9 registration at 3×3@10.** Given spin2k's NULL and a spin CPU estimate of only about 0.1 CPU-s per decode, its information value is low." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6752576037
- [2026-11-10T00:13Z] docs/journal/decisions.md (doc): "runners can't reach the 3 × 3 plan, so making the pipelined team with park and small the" — https://github.com/lmmx/mibz2/blob/02c2bbfb98c4/docs/journal/decisions.md
- [2026-11-10T00:14Z] comment 6755063677 (#222 coordinator): "- In the slow mode every thread's CPU grew about 2.2×, including threads that never spin. That fits frequency, SMT or memory contention, and the cause stays unresolved." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6755063677
- [2026-11-10T00:56Z] comment 6755766766 (#145 researcher): "**The titration cannot identify the proposed BOUND/NOT BOUND meanings.** Counterexample: original throughput 1.5 GB/s saturates W=28.5 at β=19. Reserving a core makes the spin-hog baseline CPU-limited at 1.0. An 8 GB/s hog leaves 1.079 GB/s memory capacity, so r=1: NOT BOUND despite the original ba…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6755766766
- [2026-11-10T00:57Z] comment 6755781476 (#314 coordinator): "- **The counterexample:** take original throughput 1.5 GB/s saturating W=28.5 at β=19. Reserving a core makes the spin-hog baseline CPU-limited at 1.0. An 8 GB/s hog then leaves 1.079 GB/s, so r=1 and the reading is NOT BOUND, despite a real bandwidth ceiling. "The same core cost cancels" fails whe…" — https://github.com/lmmx/mibz2/pull/314#issuecomment-6755781476
- [2026-11-10T01:18Z] comment 6756108082 (#145 pathfinder): "**The probe, unprivileged, no counters:** a pinned thread spins register arithmetic and stores `rdtsc` every 2²⁰ iterations. Iterations per invariant-TSC tick are effective clock × duty. One on a core fenced from the decoder. Beside it: thread placement every 2 ms (`/proc/<pid>/task/<tid>/stat` fie…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6756108082
- [2026-11-10T01:37Z] comment 6756418444 (#145 optimizer): "**Narrow the two-mode claims.** Endpoint equality does not exclude in-run events; every arm having both modes does not exclude arm×state effects. No universal SMT2× slowdown bound. The fenced spin probe changes capacity and power and measures progress, not frequency alone. Kernel documentation supp…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6756418444

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #321, #325) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6757343704 (2026-11-10T02:34Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/160#issuecomment-6757179363, https://github.com/lmmx/mibz2/pull/161#issuecomment-6757176848

**Notes:** PR #321's earliest listed commit is 2026-11-10T02:04Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #325's earliest listed commit is 2026-11-10T02:04Z, 1 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## design-e-line-table: Design E: the backward line table (Pathfinder)

- **PRs:** [#323](https://github.com/lmmx/mibz2/pull/323)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench (EPYC 9V74)
- **Origin (as recorded):** the Pathfinder (per PR #323)
- **commit_to_build:** 2026-11-10T02:51Z, PR #323 first commit (author date) (https://github.com/lmmx/mibz2/pull/323/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **CI's stage harness** also checks every Wikidata block's CRC from E's output on each run. ## Census (counted, untimed) \| corpus \| E: one line access \| E: line + overflow \| E bytes a row \| T-vector bytes a row \| \|---\|--:\|--:\|--:\|--:\| \| wikidata \| 98.18% \| 1.82% \| 2.039 \| 4 \| \| …" (https://github.com/lmmx/mibz2/pull/323 (PR body, current text))
- "**Instructions a row** (callgrind, first 3 blocks): \| \| wikidata \| text \| \|---\|--:\|--:\| \| E's build / the T-vector build \| 0.82× \| 0.56× \| \| E's 32-lane walk / the room walk \| 48.2 vs 10.4 (4.6×) \| 48.1 vs 10.7 (4.5×) \| \| inversion, build + walk \| 3.41× \| 2.50× \| ## The register…" (https://github.com/lmmx/mibz2/pull/323 (PR body, current text))
- "As compiled, E's step alone is 22–26 instructions, and the model compared it with the room walk's whole loop." (https://github.com/lmmx/mibz2/pull/323 (PR body, current text))

**must_hold** (quoted):

- "- bench 3.5.0: stages.py keeps the rows not in the total; report.py shows them beside their stage, paired by run." (https://github.com/lmmx/mibz2/pull/323 (first commit message))
- "- docs/journal/2026-11-10-design-e-registration.md: hypothesis, predictions, the deciding row, the falsifier, STEWARDSHIP's rule quoted, protocol." (https://github.com/lmmx/mibz2/pull/323 (first commit message))
- "- **The Miri workflow** now also runs when `src/line_table.rs` changes. ## Correctness - **The bench corpora** (`ab gen corpora 4` and the pinned Wikidata corpus): 488 blocks, 258.7 M rows." (https://github.com/lmmx/mibz2/pull/323 (PR body, current text))
- "- **The standing rule:** STEWARDSHIP's acceptance rule is quoted unchanged in the registration." (https://github.com/lmmx/mibz2/pull/323 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-10T02:51Z] "- docs/journal/2026-11-10-design-e-registration.md: hypothesis, predictions, the deciding row, the falsifier, STEWARDSHIP's rule quoted, protocol." (https://github.com/lmmx/mibz2/pull/323 (first commit message))
- [2026-11-10T02:52Z] "As compiled, E's step alone is 22–26 instructions, and the model compared it with the room walk's whole loop." (https://github.com/lmmx/mibz2/pull/323 (PR body, current text))

**same_quantity_facts:** 17 matching lines in 9 documents dated before commit_to_build, searched on `design E|line table|1\.44|walk gate|share.nothing` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning.md (doc): "Exact totals, 4 → 10 workers: ×3.50 at -9, ×2.65 at -6, ×1.44 at -3, ×1.10 at -1." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/results-table.md (doc): "\| 40ba07a \| AMD EPYC 7763 64-Core Processor \| 1.66 (1.64-1.69) \| 1.13 (1.12-1.15) \| 1.71 (1.70-1.71) \| 1.47 (1.44-1.49) \|" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/results-table.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-workstation-scaling.md (doc): "\| -3 \| 0.704 \| 1.012 \| ×1.44 \|" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-workstation-scaling.md
- [2026-10-29T07:13Z] docs/journal/2026-10-29-bz2ngrs.md (doc): "- On CI's EPYC it is 1.44 × slower." — https://github.com/lmmx/mibz2/blob/e0e6e4a47d90/docs/journal/2026-10-29-bz2ngrs.md
- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "its single-core kernel is 1.44 × slower than ours." — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation.md (doc): "rustc 1.97.0, fat LTO, `codegen-units = 1`, line tables. Input `wikidata-independent.bz2`, sha256" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation.md
- [2026-11-07T09:26Z] docs/journal/2026-11-07-left-field-archive-check.md (doc): "- **On CI's 4 one-core workers:** libsais is the slower kernel (1.05–1.44 × our time), and our" — https://github.com/lmmx/mibz2/blob/8b7c1d175e94/docs/journal/2026-11-07-left-field-archive-check.md
- [2026-11-08T20:38Z] docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md (doc): "\| **table, B/row at 8 B** (pooled; median, p90, max block) \| **0.96** (0.83, 1.44, 2.44) \| 2.20 \| 8.0 \| 8.0 \|" — https://github.com/lmmx/mibz2/blob/a4f97714cc14/docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md
- [2026-11-10T01:43Z] comment 6756520552 (#145 pathfinder): "**Take 2: design E, the line table.** One 64-byte line a 32 rows: the 32 bytes of L, a first-occurrence mask, up to 9 bases. LF is one compare and two popcounts, one access. 2 bytes a row. Counted (`examples/line_census.rs`): lines with at most 9 distinct bytes are 94% on Wikidata and 83% on text; …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6756520552

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #323) / Optimiser: unrecorded (no comment from the role naming #323)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6757811329, https://github.com/lmmx/mibz2/pull/145#issuecomment-6758128730, https://github.com/lmmx/mibz2/pull/160#issuecomment-6757808815

## run-front-records: Run-record front and backward LF table on blocks of few symbols

- **PRs:** [#324](https://github.com/lmmx/mibz2/pull/324), [#348](https://github.com/lmmx/mibz2/pull/348)
- **Journal sources:** docs/journal/2026-11-10-run-front-production.md
- **i9 recipes:** release-scaling/run-20261110T101103Z
- **Deciding machine:** CI paired bench (frozen rule), i9 check bench-324-i9
- **Origin (as recorded):** the Pathfinder (per PR #324)
- **commit_to_build:** 2026-11-10T03:09Z, PR #324 first commit (author date) (https://github.com/lmmx/mibz2/pull/324/commits)
- **Cost:** i9 10.7 over 1 bundle(s) min [release-scaling/run-20261110T101103Z: 10.7 min (bundle start (name)→evidence commit)]; CI: 6 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.07; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **A disclosed finding outside the bench:** short high-entropy blocks of 200–440 kB take about +10–11% whole-child instructions (Pathfinder 6763522358)." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))
- "A is −29% to −48% at the stage on accepted blocks, S −14% to −16%." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))
- "- **Prediction:** `wikidata` wall at 1 thread −4% to −12%, CPU at 4 threads −3% to −10%." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-release-scaling: "release scaling, 0.2.6 against 0.2.5 (RELEASING.md §5)"
- "- **Prediction:** `wikidata` wall at 1 thread −4% to −12%, CPU at 4 threads −3% to −10%." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))
- "- **§5, the rule:** #312's." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-10T03:10Z] "**`runs::front`** turns the MTF symbols into run records, `byte << 8 \| len` cut at 127 rows." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))
- [2026-11-10T03:10Z] "- **Prediction:** `wikidata` wall at 1 thread −4% to −12%, CPU at 4 threads −3% to −10%." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))
- [2026-11-10T03:10Z] "- **Falsifiers:** a `wikidata` 1-thread point estimate at or above −2% in two runs, or `text` at or above −3%." (https://github.com/lmmx/mibz2/pull/324 (PR body, current text))
- [2026-11-10T10:41Z] "The Coordinator opened the PR from its byte-exact patch, because its session can't yet push branches: patch file `docs/journal/2026-11-10-backward-walk/backward-walk.diff.txt` at a28813a9 on #313's branch, SHA-256 414137b6…2b3e, verified." (https://github.com/lmmx/mibz2/pull/348 (PR body, current text))
- [2026-11-10T10:41Z] "**Counts** (callgrind, whole child, against master; untimed apart from one noisy local run): \| block \| #324 \| this change \| \|---\|--:\|--:\| \| random \| \| −1.0% \| \| records \| \| −1.1% \| \| random cut to 200 kB \| +10.3% \| −0.8% \| \| random cut to 440 kB \| +11.4% \| −1.0% \| \| random cut to 460 kB \| +0.0% \| −1.0% \| Output bytes equal bzip2's on every local corpus." (https://github.com/lmmx/mibz2/pull/348 (PR body, current text))

**same_quantity_facts:** 8 matching lines in 7 documents dated before commit_to_build, searched on `run.record|few symbols|backward|LF table|records path` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-27T15:46Z] docs/journal/2026-10-27-literature-reading.md (doc): "- **The contrast with #30.** #30 searched **backward** for an already-walked neighbour, which forced a per-byte record of rows (4 B a byte, about 13 c/B locally). Forward detection needs no such record." — https://github.com/lmmx/mibz2/blob/24169e657c1f/docs/journal/2026-10-27-literature-reading.md
- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- [S] Two steps. (1) Sensitivity: run the target with a "bubble" on the remaining cores of the socket and dial its pressure up, giving a QoS-vs-pressure **sensitivity curve**. (2) Contentiousness: run the candidate co-runner against a single-threaded **reporter** whose own sensitivity curve was tra…" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-11-05T12:27Z] docs/journal/2026-11-05-binding-census/nogo.md (doc): "- NOGO Precopy: backward neighbour search (#30), forward chains (#36): of=The mechanism: the copied steps were the cache-resident ones (precopy-forward-chains.md:47-52). +118% and +6% at 4t." — https://github.com/lmmx/mibz2/blob/38af9b6fb8b4/docs/journal/2026-11-05-binding-census/nogo.md
- [2026-11-08T18:20Z] comment 6725272955 (#145 louis via chatgpt): "The actual payoff of §4 [lines 217–317](https://github.com/lmmx/mibz2/blob/02a98d58850baf675717482d20ec32edcec2699f/docs/site/blog/2026-11-08-bwt-runs-from-mtf-codes.md#L217-L317) is simple: **we need to process m compressed symbols and keep r run records, rather than materialise n BWT bytes**; eac…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6725272955
- [2026-11-08T20:38Z] docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md (doc): "at about 2.5–2.6 instructions a row. Replacing L with run records saves no peak memory in" — https://github.com/lmmx/mibz2/blob/a4f97714cc14/docs/journal/2026-11-08-run-fed-tvector-and-psi2-census.md
- [2026-11-10T00:40Z] comment 6755498073 (#145 pathfinder): "**For the Optimiser, a priced lead, not mine to carry:** `examples/run_front.rs` (an agent of mine; checked on 431 blocks and 117,929 synthetic streams against `mtf_scalar`, errors included). MTF symbols to run records with no column: 0.173 ns a row against `mtf::mtf`'s 0.423 (local, one host, load…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6755498073
- [2026-11-10T01:06Z] comment 6755915645 (#145 pathfinder): "- **A. The sequential table.** Records in arrival order, one counter a byte: no F-order sort, no sweep, no direct placement, no column, no bucket checks. It is the run-fed line (#269, #277, 11-08 line A) with nothing left to lose on. Fill 0.29 ns a row against `build`'s 0.46 locally, both near the …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6755915645

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 5 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6763089509 (2026-11-10T08:50Z, after commit_to_build, before the first i9 bundle) / Optimiser: 3 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6758406452 (2026-11-10T03:47Z, after commit_to_build, before the first i9 bundle)

**monodromy_involved:** yes: records naming its PRs or recipes: 2026-11-10-rauzy-veech.md, DESIGN.md, HANDOFF.md

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6750340953, https://github.com/lmmx/mibz2/pull/145#issuecomment-6758603004, https://github.com/lmmx/mibz2/pull/160#issuecomment-6758452562

## coop-ports: Cooperative ports stack: run16 and one-pass loop in FusedBuild::share, C29 MTF on the team path

- **PRs:** [#335](https://github.com/lmmx/mibz2/pull/335), [#337](https://github.com/lmmx/mibz2/pull/337), [#339](https://github.com/lmmx/mibz2/pull/339)
- **Journal sources:** none listed
- **i9 recipes:** release-scaling/run-20261110T095909Z
- **Deciding machine:** i9-10900K (bench-team-ports)
- **Origin (as recorded):** counted ports (per PR titles: Ir/D counts)
- **commit_to_build:** 2026-11-10T05:09Z, PR #335 first commit (author date) (https://github.com/lmmx/mibz2/pull/335/commits)
- **Cost:** i9 6.6 over 1 bundle(s) min [release-scaling/run-20261110T095909Z: 6.6 min (bundle start (name)→evidence commit)]; CI: 10 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.21; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Every figure is recomputed from the archived profiles, with their inventory and SHA-256s in `count.txt`. \| Ir/D \| port 1 (control, this session) \| port 2 \| Δ \| \|---\|--:\|--:\|--:\| \| `FusedBuild::share`, inclusive \| 6.786 \| **5.403** \| −1.383 \| \| build stage \| 6.955 \| 5.571 \| −1.383…" (https://github.com/lmmx/mibz2/pull/337 (PR body, current text))
- "- **Predicted band** (5.16 to 5.25): missed by 0.15." (https://github.com/lmmx/mibz2/pull/337 (PR body, current text))
- "It is off by 0.205: on the loaded machine, `ab`'s sampler thread executed about 32 M more instructions." (https://github.com/lmmx/mibz2/pull/337 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-release-scaling: "release scaling, 0.2.6 against 0.2.5 (RELEASING.md §5)"
- "It follows D088's rules: a frozen decoder/harness partition, instruction-identity conditions, the unattributed residual kept visible, and resource conditions recorded but not gating." (https://github.com/lmmx/mibz2/pull/339 (PR body, current text))
- "They break the registered rule that every harness function's name starts with `ab::`." (https://github.com/lmmx/mibz2/pull/339 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-10T05:22Z] "Port of one-worker `bwt::run16` into the cooperative build's equal-byte window, as #313's residual (`2026-11-10-team-residual.md`) priced it." (https://github.com/lmmx/mibz2/pull/335 (PR body, current text))
- [2026-11-10T05:22Z] "Ir a decoded byte: \| \| old (master) \| new \| Δ \| \|---\|--:\|--:\|--:\| \| `FusedBuild::share`, inclusive \| 8.937 \| **6.786** \| −2.151 \| \| whole process \| 37.916 \| **35.800** \| −2.116 \| - **Prediction:** `share` about 6.6." (https://github.com/lmmx/mibz2/pull/335 (PR body, current text))
- [2026-11-10T05:22Z] "- **One deviation is disclosed:** condition 3's registered debug-line test cannot tell the arms apart, because the inlined intrinsics are attributed to `core::arch` files." (https://github.com/lmmx/mibz2/pull/335 (PR body, current text))
- [2026-11-10T05:22Z] "**CI's A/B never runs a team (#330 §7), so its reading on this PR is expected to be neutral." (https://github.com/lmmx/mibz2/pull/335 (PR body, current text))
- [2026-11-10T06:25Z] "- **Predicted band** (5.16 to 5.25): missed by 0.15." (https://github.com/lmmx/mibz2/pull/337 (PR body, current text))
- [2026-11-10T06:25Z] "The sixth, a stricter guard, survives because it writes the same table and costs only instructions. ## Dispatch paths (G3) - **Changed:** - the pipelined team helpers;" (https://github.com/lmmx/mibz2/pull/337 (PR body, current text))

**same_quantity_facts:** 220 matching lines in 103 documents dated before commit_to_build, searched on `FusedBuild|share|run16|team path|Ir/D` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T04:21Z] comment 6758892969 (#145 coordinator): "**2350 MB/s at 7 workers is arithmetic, not a measurement.** It rests on g = 1.18, which was taken from EPYC losses and divided by an assumed 70% inversion share. Two problems:" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6758892969
- [2026-11-10T04:45Z] docs/journal/2026-11-10-run-front-production.md (doc): "\| accepted blocks' share of the stage \| 0.74 \| #269: (1.290 + 1.432) / 3.676 \|" — https://github.com/lmmx/mibz2/blob/27ed84df487c/docs/journal/2026-11-10-run-front-production.md
- [2026-11-10T04:52Z] comment 6759334709 (#145 pathfinder): "RE:   tall-tunnel copies, counted in decode order and certified; D4-delta's F2 is the wrong bar for share-nothing; a 1.2 MB table with one rule @ 7bf5769be304d8908910de76a4fe5050ac8cf8c4" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759334709
- [2026-11-10T04:57Z] comment 6759394097 (#161 coordinator): "**What the record already shows independently of (b):** the single-caller control saves 7.8 Ir/D with no team present. The saving belongs to the loop's construction, not to any scheduling artefact. The team adds only 0.08 over one caller." — https://github.com/lmmx/mibz2/pull/161#issuecomment-6759394097
- [2026-11-10T05:02Z] comment 6759461255 (#145 researcher): "**1. Gate stall: acceptable operational risk.** Keep the fixed 120-second slot bound, 5400-second drive bound, whole-attempt INVALID and no retry to pass. A successful attempt is conditional on this harness completing; it is not an unconditional workstation scaling claim. With 732 slots, even rare …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759461255
- [2026-11-10T05:04Z] comment 6759486415 (#145 observer): "3. Found: walk 17.77→9.97 Ir/byte, whole 38.06→30.21; helper share 69.5% > ⅔, so FAIL-MEASUREMENT. D4 F2 14.45 > 7.35; #324 boxing failed." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759486415
- [2026-11-10T05:04Z] comment 6759487306 (#145 guardian): "**Two evidence-scope corrections:** the one-caller saving reproduces as **7.805422 Ir/D**, supporting a construction signal without competing walkers. The team/control difference is **0.081459 Ir/M** on their common walked-byte denominator, not a bound on all team overhead or scheduling effects. Al…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759487306
- [2026-11-10T05:05Z] comment 6759502195 (#145 coordinator): "- **The scopes:** 7.805 Ir/D is a construction signal; 0.0815 bounds nothing about team overhead." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759502195
- [2026-11-10T05:06Z] comment 6759517902 (#145 pathfinder): "- **Then** `build_one_pass`'s loop whole (`run_joins`, `few_runs`): → about 26.5. Its provisional writes stay inside the share's own window if `lim` is the window's end; a run within 16 slots of it takes the row steps, under 0.07." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759517902
- [2026-11-10T05:06Z] comment 6759527030 (#145 coordinator): "1. **`run16` into `share`'s window:** 30.21 → about 27.9. Falsifier: `share` inclusive ≥ 6.9 Ir/D." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759527030

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-7084586885 (2026-11-22T19:40Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #335, #337, #339, bench-release-scaling)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6750340953, https://github.com/lmmx/mibz2/pull/161#issuecomment-6759768735, https://github.com/lmmx/mibz2/pull/222#issuecomment-6745073218

## team-runs-port: Team paths read the run-record front (#324 ported to teams)

- **PRs:** [#359](https://github.com/lmmx/mibz2/pull/359)
- **Journal sources:** none listed
- **i9 recipes:** release-scaling/run-20261110T160612Z
- **Deciding machine:** i9-10900K (bench-team-runs) and CI
- **Origin (as recorded):** the Pathfinder's PROPOSAL 6769926198
- **commit_to_build:** 2026-11-10T15:31Z, PR #359 first commit (author date) (https://github.com/lmmx/mibz2/pull/359/commits)
- **Cost:** i9 27.1 over 1 bundle(s) min [release-scaling/run-20261110T160612Z: 27.1 min (bundle start (name)→evidence commit)]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Locally (indicative only), the wall time was 0.82–0.93× on wikidata and 0.69–0.84× on text, across the three team paths, with the same bytes. ## Before merge - [x] The Guardian clears D097-F1 (D098)." (https://github.com/lmmx/mibz2/pull/359 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-release-scaling: "release scaling, 0.2.6 against 0.2.5 (RELEASING.md §5)"
- "The output bytes are the same on every path, and #353's admission rule applies unchanged on both entry routes." (https://github.com/lmmx/mibz2/pull/359 (PR body, current text))
- "On the i9 that covers plain teams at 6 and 8, the service core at 10 and pipelined teams at 12 and above, with 1, 2 and 4 as a worker-path control." (https://github.com/lmmx/mibz2/pull/359 (PR body, current text))
- "- **Rule:** section 8's, in the note." (https://github.com/lmmx/mibz2/pull/359 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 265 matching lines in 148 documents dated before commit_to_build, searched on `team path|run.record|records` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T10:03Z] comment 6764488488 (#145 pathfinder): "**W4. The parallel scan against the production default.** `par-scan/20261110T030608Z`: find over scan 1.615 on `wikidata-l1`, 1.780 on `records`, 10 workers. A measured win sitting in an example. Both arms were the share-nothing harness; the default was not an arm." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6764488488
- [2026-11-10T10:32Z] comment 6765072254 (#145 pathfinder): "- **Second: the universal backward walk.** Built locally on #324's head: `build_back`, about 60 lines of safe code, and admission by a sample of the block's rows. Bytes equal to bzip2 on every local corpus. Whole-child instructions against master: `random` −1.0%, `records` −1.1%; the short blocks 2…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6765072254
- [2026-11-10T10:37Z] docs/journal/2026-11-10-service-guarded-target-gated-registration.md (doc): "history has rare large contrasts (runs, onebyte 2, wikidata 4, records 4), a nominal 5%" — https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-10-service-guarded-target-gated-registration.md
- [2026-11-10T12:04Z] docs/journal/2026-11-10-run-front-production.md (doc): "- **`records` at 4 threads:** flagged in run 2 (+4.0%), not in run 3. Unresolved." — https://github.com/lmmx/mibz2/blob/c92cf3bfb470/docs/journal/2026-11-10-run-front-production.md
- [2026-11-10T12:22Z] comment 6767179948 (#145 observer): "> Version 0.2.8 has shipped two measured decoder improvements: a faster run-record path for Wikidata and text, and guarded service on Louis's ten-thread workstation target. On the four-core CI comparison with 0.2.7, Wikidata decoding was 12–19% faster and text 10–26% faster across one, two and four…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6767179948
- [2026-11-10T13:24Z] comment 6768489881 (#145 monodromy): "- It takes (byte, length) records; the front must cut orig_ptr's row out. #324's front saving (0.25 ns a row) is in no figure above, and #324's records path is not an arm." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6768489881
- [2026-11-10T14:27Z] comment 6769926198 (#145 pathfinder): "- On a worker thread the depth is fixed by the build, so any change to the binary can move a build into the band. The port alone did: `records` on plain teams at 1.40× [1.36, 1.44], with equal instruction counts. Master happens to sit outside it." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6769926198
- [2026-11-10T14:40Z] comment 6770249707 (#145 monodromy): "**Baseline, same commit:** master is merged. Production's inversion since 0.2.8 is the records' path, which the Pathfinder measures at 0.95 of the column's on these blocks, so 0.756 is about 0.80 of production. The example now times `runs::fill` + `walk_back_room` and feeds the cells kernel from re…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6770249707
- [2026-11-10T14:58Z] comment 6770687575 (#222 coordinator): "- **Merged:** #358 (→ 910472e), which aligns the MTF list so it sits in one page. It removes a stack-depth hazard that made one build's `records` 1.40× slower on teams. The bench was neutral; the Guardian's D097 said PROCEED." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6770687575
- [2026-11-10T15:03Z] comment 6770809231 (#145 observer): "3. Found: #358's neutral MTF alignment fix merged. Team-runs local gains remain indicative; D097-F1 found no forced plain-team fuzz path. #351's latest PR reports local cells time about 0.82× current records inversion on Wikidata after writer repair, with no production result." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6770809231

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-7084586885 (2026-11-22T19:40Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #359, bench-release-scaling)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6750340953, https://github.com/lmmx/mibz2/pull/161#issuecomment-6771542584, https://github.com/lmmx/mibz2/pull/222#issuecomment-6745073218

## onecore-inline: One-core walk's joins kept inline (0.2.8's machine code)

- **PRs:** [#361](https://github.com/lmmx/mibz2/pull/361), [#366](https://github.com/lmmx/mibz2/pull/366)
- **Journal sources:** docs/journal/2026-11-10-onecore-inline-i9.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench; i9 diagnostic
- **Origin (as recorded):** the Architect's 12-line patch (2026-11-10-coordinator-handoff.md)
- **commit_to_build:** 2026-11-10T17:45Z, PR #361 first commit (author date) (https://github.com/lmmx/mibz2/pull/361/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The compiler then stopped inlining them into `walk_back_room` and inlined `walk_back_room` into `decode_block` instead, whose frame grew from 0x48 to 0x58 bytes (563 instructions to 574)." (https://github.com/lmmx/mibz2/pull/361 (PR body, current text))
- "**Checked** (rustc 1.97; the bench harness's binary; tables in #313's `codegen/`): \| \| `decode_block::<Segmented>` \| `walk_back_room` \| \|---\|---\|---\| \| before the port (`910472e8`) \| 563 instructions \| 342 \| \| master `6782c0fbf` \| 574 \| inlined away \| \| master with this patch \| 56…" (https://github.com/lmmx/mibz2/pull/361 (PR body, current text))

**must_hold** (quoted):

- "bench: prepare pinned one-core i9 diagnostic wrapper" (https://github.com/lmmx/mibz2/pull/366 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-10T17:45Z] "At one thread the decode runs on the main thread, whose stack offset is drawn per launch, so a 16-byte shift should not move a mean there; the inlining of `walk_back_room` into `decode_block` is the other half of the change and is also undone." (https://github.com/lmmx/mibz2/pull/361 (PR body, current text))
- [2026-11-10T23:30Z] "No accounting-model change or new independent safety certification." (https://github.com/lmmx/mibz2/pull/366 (PR body, current text))

**same_quantity_facts:** 33 matching lines in 25 documents dated before commit_to_build, searched on `inline|machine code|one.core|0\.2\.8` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T03:45Z] comment 6758376066 (#161 coordinator): "**The implementer's leading hypothesis for the harm (untested):** a data-layout effect, not work. Each worker's `Scratch` grew by about 2.1 KiB of inline `runs::Runs`, which shifts `counts` against `mtf_avx2`'s frame modulo 4 KiB. That is the pattern #277's codegen entry documented. High-entropy bl…" — https://github.com/lmmx/mibz2/pull/161#issuecomment-6758376066
- [2026-11-10T03:45Z] comment 6758381275 (#145 coordinator): "2. **A static check, with no timing:** `counts`' 4 KiB alignment class relative to `mtf_avx2`'s frame, in master, in #324 and in a boxed-`Runs` variant. The implementer's hypothesis is that `Scratch` grew by about 2.1 KiB inline, shifting the layout #277 documented." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6758381275
- [2026-11-10T03:50Z] comment 6758452562 (#160 coordinator): "- **The implementer's hypothesis:** each worker's `Scratch` grew by about 2.1 KiB of inline `runs::Runs`, shifting `counts` against `mtf_avx2`'s frame modulo 4 KiB (the pattern #277 found). A static check of the three offsets (master, #324 and boxed `Runs`) is running." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6758452562
- [2026-11-10T04:06Z] docs/journal/2026-11-10-team-kernel-gap-count.md (doc): "\| build \| `bwt::build` (`build_one_pass` inlined) 5.160 \| `FusedBuild::share` 8.937, `FusedBuild::new` 0.162, `bwt::build` 0.002 (the short blocks) \|" — https://github.com/lmmx/mibz2/blob/c47fbe50cf99/docs/journal/2026-11-10-team-kernel-gap-count.md
- [2026-11-10T12:04Z] docs/journal/2026-11-10-run-front-production.md (doc): "grew by `runs::Runs`, about 2.1 KiB inline: two `[u32; 256]` and four `Vec`s. That moves" — https://github.com/lmmx/mibz2/blob/c92cf3bfb470/docs/journal/2026-11-10-run-front-production.md
- [2026-11-10T12:17Z] comment 6767076231 (#222 coordinator): "- 0.2.8 released: Wikidata −12% to −19% and text −10% to −26% against 0.2.7 on CI." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6767076231
- [2026-11-10T12:22Z] comment 6767179948 (#145 observer): "3. Found: 0.2.8 CI tag contrast Wikidata −12–19%, text −10–26% wall at 1–4 threads; #316's i9 target +6.8% throughput/−14.7% CPU. Ports missed their pre-run wall rule; #348 no primary gain. #351 first Wikidata kernel 1.276× incumbent time." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6767179948
- [2026-11-10T14:40Z] comment 6770249707 (#145 monodromy): "**Baseline, same commit:** master is merged. Production's inversion since 0.2.8 is the records' path, which the Pathfinder measures at 0.95 of the column's on these blocks, so 0.756 is about 0.80 of production. The example now times `runs::fill` + `walk_back_room` and feeds the cells kernel from re…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6770249707
- [2026-11-10T17:21Z] comment 6774193623 (#145 coordinator): "- **0.2.8 ÷ 0.2.9 wall, wikidata:** 1 thread **0.991 (0.983 to 0.998)**; 2 and 4 threads level; 8 threads 1.147; 10 threads 1.145; 12 threads 1.111; 16 threads 1.109; 20 threads 1.102." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6774193623
- [2026-11-10T17:31Z] comment 6774426674 (#145 coordinator): "0.2.9's release-scaling run (LOG on this channel) measured wikidata at 1 thread 0.9% slower than 0.2.8, beyond noise (0.991, 0.983 to 0.998). That is the path your codegen comparison found changed. Open `team-runs-3-over-2.diff.txt` as a maintenance PR against master today, per my earlier CHECKPOIN…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6774426674

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6793646018 (2026-11-11T10:41Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #361, #366)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## team-threshold: Move the team threshold below 6 threads

- **PRs:** none
- **Journal sources:** none listed
- **i9 recipes:** team-size/20261110T174300Z
- **Deciding machine:** i9-10900K (bench-team-threshold)
- **Origin (as recorded):** Coordinator (rule LOG on #145)
- **commit_to_build:** 2026-11-10T17:36Z, commission 6774528179 (https://github.com/lmmx/mibz2/pull/145#issuecomment-6774528179)
- **Cost:** i9 13.0 over 1 bundle(s) min [team-size/20261110T174300Z: 13.0 min (bundle start (name)→evidence commit)]; CI: unrecorded; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- "C1a rule for bench-team-threshold (master dfe04059): does 0.2.9 move the team threshold below 6 threads?" (https://github.com/lmmx/mibz2/pull/145#issuecomment-6774528179)
- "- **The run:** `just bench-team-size 20 "4,5,6,8"` at dfe04059, on the i9, with wikidata, 20 paired rounds." (https://github.com/lmmx/mibz2/pull/145#issuecomment-6774528179)

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 185 matching lines in 80 documents dated before commit_to_build, searched on `team threshold|forced team|4 threads|5 threads` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T10:22Z] comment 6764875247 (#145 coordinator): "- **Up to 4 threads,** #324 is 8–10% faster on the i9, in line with CI's −10 to −15%." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6764875247
- [2026-11-10T10:37Z] docs/journal/2026-11-10-service-guarded-target-gated-registration.md (doc): "2 and 4 threads +4.0% (+2.4, +5.6) and +3.8% (+1.7, +6.0) slower, `onebyte` at 4 threads" — https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-10-service-guarded-target-gated-registration.md
- [2026-11-10T11:18Z] comment 6765942197 (#145 coordinator): "- No primary corpus is faster beyond noise, so rule 1 fails. Secondary: random −4.8% at 4 threads." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6765942197
- [2026-11-10T12:04Z] docs/journal/2026-11-10-run-front-production.md (doc): "3. **The extra peak RSS** (+4–10% on Wikidata and text at 2–4 threads, about 3 MB) is the" — https://github.com/lmmx/mibz2/blob/c92cf3bfb470/docs/journal/2026-11-10-run-front-production.md
- [2026-11-10T12:17Z] comment 6767075813 (#145 coordinator): "- `wikidata` −19.0%, −16.6%, −12.1% wall at 1, 2 and 4 threads;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6767075813
- [2026-11-10T12:22Z] comment 6767179948 (#145 observer): "3. Found: 0.2.8 CI tag contrast Wikidata −12–19%, text −10–26% wall at 1–4 threads; #316's i9 target +6.8% throughput/−14.7% CPU. Ports missed their pre-run wall rule; #348 no primary gain. #351 first Wikidata kernel 1.276× incumbent time." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6767179948
- [2026-11-10T15:57Z] comment 6772170543 (#145 coordinator): "- **CI's bench at 58e0a4b, run 1:** it flagged `wikidata` at 1 thread, +0.9% (+0.2%, +1.6%), and `records`/`random` at 1-4 threads. That is a path the port says it leaves unchanged, so I used the one re-run PERFORMANCE-POLICY allows for a primary regression." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6772170543
- [2026-11-10T17:07Z] comment 6773861678 (#145 coordinator): "- text at 4 threads uses +3.6% more peak memory." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6773861678
- [2026-11-10T17:07Z] comment 6773865125 (#222 coordinator): "1. **0.2.9 is released** with #359, #353, #358 and #356. The registered gain is Wikidata 11–15% faster at 6–20 threads on the i9-10900K (1.138 at 10). CI shows no change at 1–4 threads." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6773865125
- [2026-11-10T17:21Z] comment 6774193623 (#145 coordinator): "- **0.2.8 ÷ 0.2.9 wall, wikidata:** 1 thread **0.991 (0.983 to 0.998)**; 2 and 4 threads level; 8 threads 1.147; 10 threads 1.145; 12 threads 1.111; 16 threads 1.109; 20 threads 1.102." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6774193623

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming bench-team-size) / Optimiser: unrecorded (no comment from the role naming bench-team-size)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/161#issuecomment-6682936439

## cells-default: Cells (Rauzy-Veech induction) as the Workers plan's inverse, with a sparse admission gate

- **PRs:** [#365](https://github.com/lmmx/mibz2/pull/365), [#378](https://github.com/lmmx/mibz2/pull/378), [#379](https://github.com/lmmx/mibz2/pull/379), [#380](https://github.com/lmmx/mibz2/pull/380), [#382](https://github.com/lmmx/mibz2/pull/382), [#383](https://github.com/lmmx/mibz2/pull/383)
- **Journal sources:** docs/journal/monodromy/2026-11-10-rauzy-veech.md, docs/journal/2026-11-14-cells-default/INTEGRATION.md
- **i9 recipes:** cells-vs-prod/, cells-live-path/, cells-selective-i9/, cells-scaling/
- **Deciding machine:** i9-10900K (bench-cells-scaling)
- **Origin (as recorded):** MONODROMY (record 1, Rauzy-Veech induction; #351); slice one #365; Codex writer screens #378-#380
- **commit_to_build:** 2026-11-10T19:13Z, PR #365 opened (https://github.com/lmmx/mibz2/pull/365)
- **Cost:** i9 65.6 over 4 bundle(s) min [cells-vs-prod/20261110T175613Z: 1.7 min (bundle start (name)→evidence commit); cells-live-path/20261113T101816Z: 2.3 min (bundle start (name)→evidence commit); cells-selective-i9/20261113T233358053141311Z: None min (bundle start (name)→evidence commit); cells-scaling/run-20261114T131004Z: 31.2 min (bundle start (name)→evidence commit); cells-scaling/run-20261114T134631Z: 30.4 min (bundle start (name)→evidence commit)]; CI: 13 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 3.79; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Wikidata 4t −0.3% (95% CI −1.6%, +0.9%); text 1t −1.2% (−2.6%, +0.3%)." (https://github.com/lmmx/mibz2/pull/365 (PR body, current text))
- "The original Wikidata 4t +1.9% flag did not reproduce on its permitted rerun." (https://github.com/lmmx/mibz2/pull/365 (PR body, current text))
- "Wikidata ratio **1.000 [0.990, 1.011]**, RSS **26 → 34 MB**; text **1.012 [0.980, 1.046]**; onebyte **1.109 [1.095, 1.123]**; pattern **2.232 [2.217, 2.247]**." (https://github.com/lmmx/mibz2/pull/365 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-cells-vs-prod: "MONODROMY cells kernel vs production records path, 1 thread pinned, 3 repeats: is it already better than prod on the i9? (rule: LOG on #145)"
- queue entry bench-cells-selective-i9: "Matched Segmented/original/selective Cells; 12x7x3"
- queue entry bench-cells-scaling: "#383 vs master on the i9, 1-20 threads, plan and workers (registration 6877415527)"
- "CI preserves the ordinary head/base arms, schedule and gates, and separately measures cells against the ordinary executable." (https://github.com/lmmx/mibz2/pull/365 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-13T11:50Z] "Researcher [6852460604](https://github.com/lmmx/mibz2/pull/145#issuecomment-6852460604) closes the local Miri/mutation objection under the stated constructor assumptions, covering writer835984e and fixture4f6e6ff." (https://github.com/lmmx/mibz2/pull/379 (PR body, current text))
- [2026-11-14T12:40Z] "A block with fewer than one run record per 64 rows keeps Segmented." (https://github.com/lmmx/mibz2/pull/383 (PR body, current text))
- [2026-11-14T12:40Z] "- On the saved i9 trace (Optimiser 98b553c3, §83) it refuses 22/22 pattern blocks and the onebyte block, and changes none of Wikidata's 431 decisions." (https://github.com/lmmx/mibz2/pull/383 (PR body, current text))
- [2026-11-14T12:40Z] "**`Plan::Workers` (fewer than six threads) inverts with Cells,** as the calling thread does." (https://github.com/lmmx/mibz2/pull/383 (PR body, current text))

**same_quantity_facts:** 148 matching lines in 95 documents dated before commit_to_build, searched on `Cells|Rauzy|induction|64r < n|r/n` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T15:32Z] comment 6771530689 (#145 monodromy): "3. What it settles: the wording of that conjecture in the entry and nothing else. It is not the i9, whose shared level is 20 MB against this container's reported 260 MiB; the workstation diagnostic and its rule (cells ÷ production at 10 copies no more than at 1; dead above 0.9 at 10) stand as frame…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6771530689
- [2026-11-10T15:38Z] comment 6771693515 (#145 pathfinder): "- **F6, §5:** "can go below 1.220": on eight blocks the best found was 1.223 pooled, 0.2% above the floor; one block ended 0.16% below. And "about 1.99 times" is the rounds' ordering and induction **without** writing the bytes." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6771693515
- [2026-11-10T15:40Z] comment 6771749702 (#145 coordinator): "3. **Where it wins and loses, against runs per byte.** Use #351's figures after the fix: Wikidata 0.818, prose 4.03, uniform bytes 8.94, `(ab)^k a` 11.2. Add the per-r/n bands from the journal's tables if they help. The analogy should make clear why short runs mean more steps per byte." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6771749702
- [2026-11-10T16:08Z] comment 6772437510 (#145 pathfinder): "- That this change caused the flags. This machine cannot resolve 1 to 3% at one thread, and at one thread the decode runs on the main thread, whose stack offset is drawn per launch; a 16-byte frame shift there should not move a mean. It fits the 2- and 4-thread cells better than the 1-thread ones." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6772437510
- [2026-11-10T16:11Z] comment 6772521000 (#145 monodromy): "1. **The parallel count in 6768489881 was a ceiling, not a schedule.** "P workers need S/P turns to within 1% up to P = 16" (2.00, 3.99, 7.97, 15.87 in the entry) splits one merge's steps among workers, and a merge's steps are a chain as the induction takes them. Whole merges in one round share no …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6772521000
- [2026-11-10T16:32Z] comment 6773007069 (#222 coordinator): "1. **#351 (MONODROMY's Rauzy–Veech kernel) merged as research evidence**, cabe3832 → d022139b. It holds an example and a journal only: no decoder change and no production claim. Its final figure is 0.82 to 0.86 of the shipped inversion stage on one thread, over twelve pinned runs. A disturbed thirt…" — https://github.com/lmmx/mibz2/pull/222#issuecomment-6773007069
- [2026-11-10T17:57Z] comment 6775018669 (#145 monodromy): "3. In the induction: cells above 0.16 n. Costs about nine tenths of production's time on the Wikidata blocks it refuses." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6775018669
- [2026-11-10T17:58Z] comment 6775038009 (#145 coordinator): "**Verdict under LOG 6774716029: PASS.** The ratio is below 1 in all three repeats. The cells kernel does the inversion stage in about 82% of production's time on the i9 (one thread, pinned), which agrees with the container's indicative 0.82–0.86." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6775038009
- [2026-11-10T17:59Z] comment 6775049636 (#145 coordinator): "The assignment's "wikidata and text faster" gate is replaced by this. `text` is refused by design (r/n 0.37), so it can only be level." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6775049636
- [2026-11-10T17:59Z] docs/journal/2026-11-10-coordinator-handoff.md (doc): "`text` is refused by admission (r/n 0.37, the kernel is 4× slower on prose), so it can only be level." — https://github.com/lmmx/mibz2/blob/edaa984c4f57/docs/journal/2026-11-10-coordinator-handoff.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** cells-vs-prod/20261110T175613Z (1.7 min; MONODROMY cells kernel vs production records path, 1 thread pinned, 3 repeats: is it already better than prod on the i9…)

**Review routing:** Researcher: 3 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6782367271 (2026-11-10T23:39Z, after commit_to_build) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6782313693 (2026-11-10T23:36Z, after commit_to_build)

**monodromy_involved:** yes: cited in the case's own documents (records 1, 2); records naming its PRs or recipes: HANDOFF.md, HANDOFF.md; origin field names MONODROMY

**coordinator_involved:** no citation found

**Notes:** 5 i9 bundles over 4 recipe(s); coders check whether master moved between them. PR #365's earliest listed commit is 2026-11-10T19:12Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #379's earliest listed commit is 2026-11-10T19:12Z, 65 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #380's earliest listed commit is 2026-11-10T19:12Z, 68 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #382's earliest listed commit is 2026-11-14T00:28Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #383's earliest listed commit is 2026-11-14T00:28Z, 12 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## cells-slot20: Cells 20-byte slot (buffer diet step 1)

- **PRs:** [#391](https://github.com/lmmx/mibz2/pull/391)
- **Journal sources:** none listed
- **i9 recipes:** cells-diet
- **Deciding machine:** i9-10900K (bench-cells-diet) and CI
- **Origin (as recorded):** MONODROMY §8 item 10 (per LOG 6883380160)
- **commit_to_build:** 2026-11-14T17:36Z, PR #391 first commit (author date) (https://github.com/lmmx/mibz2/pull/391/commits)
- **Cost:** i9 4.2 over 1 bundle(s) min [cells-diet/run-20261114T195323Z: 4.2 min (bundle start (name)→evidence commit)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.05; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "M4 and M6 also die with SIGSEGV in the lib binary. \| \| **Pre-commit status on the head** \| `pre-commit.ci - pr` = success. \| Not addressed here, because it is the coordinator's: the Guardian's separate flag on bench 6881593849 (text, 4 threads, +2.2% [+0.1%, +4.5%]) and its registered rerun rul…" (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))
- "This is how Cells' engagement is shown without changing the decoder: the dose feature at 0 MB streams nothing, and only counts `dose_blocks`, the blocks Cells admitted." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-cells-diet: "Cells 20 B vs 24 B slot: CPU amplification Q = A20(10)/A24(10), Workers 1/4/6/10t (LOG 6881022819, amended 6881074255; #391)"
- "M4 and M6 also die with SIGSEGV in the lib binary. \| \| **Pre-commit status on the head** \| `pre-commit.ci - pr` = success. \| Not addressed here, because it is the coordinator's: the Guardian's separate flag on bench 6881593849 (text, 4 threads, +2.2% [+0.1%, +4.5%]) and its registered rerun rul…" (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))
- "- **Merge rule:** PERFORMANCE-POLICY.md, judged on this PR's CI paired bench." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))
- "- **The scaling question** is a separate C1a run, `bench-cells-diet`, for the i9." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-14T17:39Z] "M4 and M6 also die with SIGSEGV in the lib binary. \| \| **Pre-commit status on the head** \| `pre-commit.ci - pr` = success. \| Not addressed here, because it is the coordinator's: the Guardian's separate flag on bench 6881593849 (text, 4 threads, +2.2% [+0.1%, +4.5%]) and its registered rerun rule." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))
- [2026-11-14T17:39Z] "Its 8-byte write at `+16` would otherwise overrun a 20-byte slot." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))
- [2026-11-14T17:39Z] "Its 4..20 are written (`wr32 +4`, `wr64 +8`, `wr_other`) when its cell is cut, before any read of it as a letter." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))
- [2026-11-14T17:39Z] "`cells == budget` is refused before a cell is cut, so `ft ≤ tops + (letters + budget) × SLOT ≤ ends[0]`, and the same for `fb`." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))
- [2026-11-14T17:39Z] "**Equivalence of value:** under 24 B, bytes 20..24 of the field were always 0, because every stored offset was below `u32::MAX`." (https://github.com/lmmx/mibz2/pull/391 (PR body, current text))

**same_quantity_facts:** 266 matching lines in 149 documents dated before commit_to_build, searched on `slot|20.byte|24.byte|footprint|amplification` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-14T15:32Z] comment 6879785194 (#145 coordinator): "- slot 24 → 20 B saves about 11% of lines (bytes 16..24 hold an offset that fits in 32 bits); a 16-byte slot saves about 22%;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6879785194
- [2026-11-14T15:59Z] comment 6880229350 (#145 coordinator): "**Noted for later, not started:** slot 24 → 20 bytes (the census estimates about 11% fewer touched lines). It is an isolated change to unsafe code, so it needs Miri and the Guardian's unsafe review. Its gate is decided after the dose result." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6880229350
- [2026-11-14T16:33Z] comment 6880728980 (#145 coordinator): "2. **The induction is 44% of Cells' instructions; slots are 30% and the writer 26%.** The writer split is parked (8/196 blocks). If the induction alone ran 2.5× faster on 3 threads, Cells per block would be at most about 1.36× faster (Amdahl, treating Ir as time; illustrative only)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6880728980
- [2026-11-14T16:35Z] comment 6880764544 (#145 coordinator): "1. **The buffer diet proceeds as the scaling fix.** The census (6879785194) names the cuts: slot 24 → 20 B (about 11% fewer lines), then 16 B (about 22%), and the output zero-fill (about 6% of writes). The diet's gate: CPU amplification of Workers + Cells at 10 threads relative to 1 falls, with no …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6880764544
- [2026-11-14T16:51Z] comment 6880984644 (#145 coordinator): "- The census (6879785194) puts 65% of Cells' lines in the slot orders, and the rest mostly in the writer and the induction's state. Shen touches neither the slot arrays nor the writer." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6880984644
- [2026-11-14T16:58Z] comment 6881070503 (#145 optimizer): "The memory census estimates ~11% fewer distinct lines, with the slot orders accounting for 65% of Cells' lines and 70% of modelled 2 MiB misses. The dose establishes greater streaming sensitivity at ten threads than one; it does not measure slot compaction. I expect a modest plateau relief, a small…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6881070503
- [2026-11-14T17:18Z] comment 6881329630 (#145 coordinator): "- The census puts the slot orders at 65% of Cells' lines. Each elided cell is a slot pair not materialised. If the 0.842 r elided cells are proportionate in lines, that is up to about 46% of slot-order traffic. That is an upper bound by counting, not a measurement." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6881329630
- [2026-11-14T17:21Z] comment 6881363652 (#145 coordinator): "- **What it would remove:** 0.842 r of Cells' 1.812 r cells are equal one-row cells that wouldn't need writing. The slot orders are 65% of Cells' cache lines. If the skipped cells account for their share of those lines, that's up to about 46% of the slot-order traffic. That figure comes from counts…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6881363652
- [2026-11-14T17:24Z] comment 6881405112 (#145 optimizer): "Historical one-lane deferred visits alone cost 5.2 ns/point against ~7.7 ns/average cell, leaving <2.5 ns for every extra; four *evenly spaced* lanes cost 1.4 ns, leaving <6.3 ns. Four Workers do not imply four lanes per block. At four threads the dose's wall stream-penalty ratio is ~1.32 versus ~0…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6881405112
- [2026-11-14T17:24Z] comment 6881408836 (#145 coordinator): "- **My "up to about 46% of slot-order traffic":** this overstated it. The proportional slot-line opportunity is about 0.031 of 0.1025 distinct lines per output byte, a gross figure before any debits, and not DRAM traffic." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6881408836

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #391, bench-cells-diet) / Optimiser: unrecorded (no comment from the role naming #391, bench-cells-diet)

**monodromy_involved:** yes: origin field names MONODROMY

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/160#issuecomment-6881024820, https://github.com/lmmx/mibz2/pull/161#issuecomment-6881612485, https://github.com/lmmx/mibz2/pull/161#issuecomment-6882196843

## cells-elision: Cells induction: recycle the dead front slot (elide fresh slots)

- **PRs:** [#408](https://github.com/lmmx/mibz2/pull/408)
- **Journal sources:** none listed
- **i9 recipes:** elision-scaling
- **Deciding machine:** CI and i9 (bench-elision-scaling)
- **Origin (as recorded):** per PR #408 (MONODROMY named)
- **commit_to_build:** 2026-11-15T11:59Z, PR #408 first commit (author date) (https://github.com/lmmx/mibz2/pull/408/commits)
- **Cost:** i9 0 min [none on the evidence branch]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "It used a temporary counter that is not committed. \| \| count \| \|---\|--:\| \| letters r₀ (slots per order) \| 11,090,867 \| \| cells \| 20,456,299 \| \| unequal cells = fresh slots elided, U \| **9,365,628** (45.8% of cells) \| \| slots touched before, 2r₀ + U \| 31,547,362 \| \| slots touched …" (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- "- **Per block:** 955,676 B on average, about 0.91 MiB, against a block footprint of 3.24 / 3.79 / 5.29 MiB (min / median / max)." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- "Expected effect: lower induction memory traffic, mostly at 4 threads, where co-running interference inflates CPU per byte about 1.55×." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-elision-scaling: "Cells induction elision (#408) vs its merge base, 1-5 threads (+6,8 no-change check), release-scaling recipe"
- "Output and every word, rule and offset outside the slots' regions are unchanged: the comparison against the original induction now leaves out only the slots' own offsets." (https://github.com/lmmx/mibz2/pull/408 (first commit message))
- "`write_cells`, the arena, the rules and the slot layout (still 20 bytes) are unchanged." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- "Expected effect: lower induction memory traffic, mostly at 4 threads, where co-running interference inflates CPU per byte about 1.55×." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-15T12:07Z] "It used a temporary counter that is not committed. \| \| count \| \|---\|--:\| \| letters r₀ (slots per order) \| 11,090,867 \| \| cells \| 20,456,299 \| \| unequal cells = fresh slots elided, U \| **9,365,628** (45.8% of cells) \| \| slots touched before, 2r₀ + U \| 31,547,362 \| \| slots touched after, 2r₀ \| 22,181,734 (**−29.7%**) \| These match the published census (r₀ and U in `docs/journal…" (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- [2026-11-15T12:07Z] "The 20-byte slot (step 1, released in 0.3.1) saved 0.60 B per output byte." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- [2026-11-15T12:07Z] "That part was mostly never touched, so it is not resident. ## Hypothesis The locality-bound note (MONODROMY, 2026-11-14) shows that at a worker's L3 share the induction misses only on first touch." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- [2026-11-15T12:07Z] "Expected effect: lower induction memory traffic, mostly at 4 threads, where co-running interference inflates CPU per byte about 1.55×." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- [2026-11-15T12:07Z] "Smaller or no effect at 1 thread." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))
- [2026-11-15T12:07Z] "Miri was **not** run locally, because the shared machine's disk was nearly full." (https://github.com/lmmx/mibz2/pull/408 (PR body, current text))

**same_quantity_facts:** 9 matching lines in 9 documents dated before commit_to_build, searched on `fresh slot|front slot|promised slot|elision` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-13T10:38Z] comment 6850491677 (#145 coordinator): "Within this already-owned assessment, reconcile the report's rank1 **in-sweep equal-unit-cell elision** with the current-source operation ledger before any implementation proposal. It is a reprioritisation of this queue item's existing representation-selective alternative, not a parallel constructo…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6850491677
- [2026-11-13T12:56Z] comment 6853599744 (#145 researcher): "**Ranked survivors:** (1) in-sweep equal-unit-cell elision, provided the deferred walk, lane-start discovery, linking and memory actually cost less than removed cells; (2) long-factor output scheduling/reuse, charged against the current guarded writer with every DAG occurrence, lookup, copy and all…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6853599744
- [2026-11-13T12:59Z] comment 6853665214 (#145 coordinator): "Two surviving hypotheses, still unpriced: in-sweep equal-unit elision with full deferred-walk/lane-start/link/memory costs; then long-factor scheduling with every DAG occurrence, lookup, copy and lifetime charged.79% unit-width cells is not0.842r equal-unit eligibility. Old prepass/sharing failures…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6853665214
- [2026-11-14T18:20Z] comment 6882129734 (#160 coordinator): "- Also say whether the result changes with the 20 B slot (about 11% fewer lines, if it merges) and with the elision screen (preconditions in 6881408836)." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6882129734
- [2026-11-14T19:52Z] comment 6883302716 (#145 coordinator): "- **CLOSED** if G ≥ 0.85: less than a 15% cut, even with an oracle. Locality scheduling under (a) goes on the Rejected list, and effort returns to shrinking the representation, elision, and shared working sets." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6883302716
- [2026-11-14T20:04Z] comment 6883444614 (#145 coordinator): "**A note for diet step 2.** The occupied slot extent is 2r₀ + U (equal steps allocate no fresh slot), not 2(r + S). So the bytes a narrower slot saves are 4 × (2r₀ + U). The step-2 builder is told to count it that way." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6883444614
- [2026-11-14T21:48Z] comment 6884716103 (#145 coordinator): "4. **INDUCTION** if s_ind ≥ 0.50 and U_ind ≥ U_wr. The diet and elision stay first." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884716103
- [2026-11-14T22:05Z] comment 6884914379 (#145 coordinator): "**The diet ends at step 1,** the 20 B slot, merged in #391 (MOVES THE PLATEAU, Q = 0.904). **Release 0.3.1 now proceeds with step 1 alone,** as Louis authorised, under `docs/RELEASING.md`. Bandwidth work continues through the attribution study (6884716103, amended 6884895257) and the elision count." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884914379
- [2026-11-15T11:55Z] comment 6893931538 (#145 coordinator): "- If the structural bound is small (say < 10% of bytes), say so plainly. That tells us to stop the elision route." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6893931538

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #408, bench-elision-scaling) / Optimiser: unrecorded (no comment from the role naming #408, bench-elision-scaling)

**monodromy_involved:** yes: cited in the case's own documents; origin field names MONODROMY

**coordinator_involved:** no citation found

## cells-writer-once: Cells writer: write the block once, front to back

- **PRs:** [#407](https://github.com/lmmx/mibz2/pull/407)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** per PR #407 (no proposer named)
- **commit_to_build:** 2026-11-15T12:03Z, PR #407 first commit (author date) (https://github.com/lmmx/mibz2/pull/407/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The LRU replay put the old store order at the compulsory minimum once the cache is 1 MiB or more, and this runner's Xeon (Emerald Rapids) has 2 MiB of L2 per core." (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))
- "The right factor's rule or bytes are prefetched when it is pushed, since it is read only after the left factor's whole expansion. ## What the bench should show If the hypothesis holds, `wikidata` (and `text`) CPU and wall at 4 threads should drop slightly (the writer is ~15% of worker CPU; a third …" (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))

**must_hold** (quoted):

- "Cells writer: write the block once, front to back Expand the cells grammar depth first in output order: a short word is copied from the arena, a rule met for the first time is expanded and its place noted, a rule met again is copied from that earlier place." (https://github.com/lmmx/mibz2/pull/407 (first commit message))
- "The writes land in the 900 kB block in the order rules were created, not by address, so each 64 B output line is written in several separate pieces at different times." (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))
- "This PR writes the block once, front to back, as a depth-first expansion of the grammar (LZ77-style): a short word is copied from the arena; a rule met for the first time has its place noted in `pos` and its two factors pushed; a rule met again is copied from its noted place, which is earlier and a…" (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))
- "- **Rules:** 6,339,139 rules in total, 0.036 per output byte; 99.9% of them (6,332,293) are expanded at a first place." (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-15T12:03Z] "The traffic it removes doesn't matter here." (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))
- [2026-11-15T12:03Z] "The order puts the factors right of the row first (the block starts `SHIFT = 1` byte after the row), then those left of it, then the row's byte; a `const` assert pins `SHIFT == 1`. ## Bytes removed per output byte (counted, not timed) Counted on the bench's `wikidata` corpus (`bench/wikidata.json`, 196 Cells blocks, 176,398,965 output bytes) by a temporary, uncommitted instrumentation of this wri…" (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))
- [2026-11-15T12:03Z] "The right factor's rule or bytes are prefetched when it is pushed, since it is read only after the left factor's whole expansion. ## What the bench should show If the hypothesis holds, `wikidata` (and `text`) CPU and wall at 4 threads should drop slightly (the writer is ~15% of worker CPU; a third to a half of its memory traffic is removed), with less effect at 1 thread where the block fits bette…" (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))
- [2026-11-15T12:03Z] "If the pointer-chasing cost dominates, 1-thread CPU could rise instead; that is the risk to watch. ## Safety The `unsafe` block's argument is in the code: a rule's two factors are words made before it, so the DFS ends; a short word has 16 readable bytes in the arena; the spine's factors plus the row's byte tile the period and a rule's factors tile it, so every write starts at `d` with `d + len <=…" (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))
- [2026-11-15T12:03Z] "`copy_small` and `copy_long` are removed (no remaining users)." (https://github.com/lmmx/mibz2/pull/407 (PR body, current text))

**same_quantity_facts:** 64 matching lines in 48 documents dated before commit_to_build, searched on `writer|rule pass|spine` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-14T18:17Z] comment 6882081160 (#145 louis via chatgpt): "Even a substantial reduction of half a cell per run would buy only about 1.94 instructions per output byte before charging the different constructor, memory accesses, grammar representation, and writer." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6882081160
- [2026-11-14T18:24Z] comment 6882177776 (#145 optimizer): "Measured independent Workers+Cells b(10)/b(3)=2.1018 supports the working-set premise; it does not measure shared-block execution. Treating the census s=.742 **optimistically as whole-decoder time share**, perfect k-way writer balance gives g₂=1.148, g₃=1.208. At **four cores**, k=2 has an ideal co…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6882177776
- [2026-11-14T19:52Z] comment 6883302716 (#145 coordinator): "- **The writer.** Measure before redesigning. The memory census already attributes 65% of Cells' lines and about 70% of modelled 2 MiB misses to the slot orders (6879785194; `research/cells-memory` 95b98eda), which ranks it below induction. The fused post-walk precedent (#2026-11-05) is noted. Not …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6883302716
- [2026-11-14T21:48Z] comment 6884716103 (#145 coordinator): "2. **WRITER** if s_wr ≥ 0.40 and U_wr ≥ 1.5 × U_ind, with both intervals excluding equality. The writer becomes the next target, ahead of further slot work." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884716103
- [2026-11-15T00:11Z] comment 6886389292 (#145 coordinator): "- **IMC traffic per output byte, from 1 to 4 workers:** reads 0.56 → 6.68 B, writes 0.47 → 6.05 B. Writes are nearly half the physical traffic at 4 workers. A2's RFO responses (flagged CML058, may overcount): rest 1.65, induction 0.50, writer 0.45." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6886389292
- [2026-11-15T00:55Z] comment 6886939287 (#145 researcher): "**R1:** 0.70 is a coverage heuristic, not permission to trust a phase split. Counterexample: observed induction=36, writer=34, unexplained=30 out of 100. Coverage passes, but assigning the residual to writer reverses the ranking. Scope mismatch, overlaps and phase-boundary carryover are not cured b…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6886939287
- [2026-11-15T10:29Z] comment 6892999618 (#145 coordinator): "\| writer \| 1.583 [1.537, 1.632] \| 0.155 \| 0.147 \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6892999618
- [2026-11-15T10:35Z] comment 6893060396 (#145 optimizer): "The held phase data imply mean four-worker occupancy only ~1.28 in induction and ~0.60 in writer. Under an explicitly *heuristic* independent-phase model, a cap of two clips ≥3 simultaneous writers ~1% of the time. A near-zero C treatment cannot identify a writer cause; require observed exposure be…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6893060396
- [2026-11-15T10:36Z] comment 6893071988 (#145 coordinator): "- **Caps of 2 barely bind.** The held phase data put mean occupancy at about 1.28 workers in induction and 0.60 in the writer. So the caps are set to **1**." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6893071988
- [2026-11-15T11:16Z] comment 6893500708 (#145 coordinator): "- D is read separately for each gate, by that gate's own exposure. On the VM's functional check, D's writer cap delayed only 4.1% of entries. So D may be NOT EXPOSED for the writer readout, which is a property of the cap." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6893500708

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #407) / Optimiser: unrecorded (no comment from the role naming #407)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## cells-bottom8: Cells 8-byte bottom slots

- **PRs:** [#414](https://github.com/lmmx/mibz2/pull/414)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI and i9 (release scaling)
- **Origin (as recorded):** per PR #414 (MONODROMY named)
- **commit_to_build:** 2026-11-15T13:27Z, PR #414 first commit (author date) (https://github.com/lmmx/mibz2/pull/414/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "If the footprint cut pays, it shows most at **4 threads** (co-running interference: #408 gained 0% at 1–2 threads, 4% less CPU at 4)." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))

**must_hold** (quoted):

- "The side reached second is overwritten unread by the letter's own first cell (rule R4, proved there, with 0 disagreements in 31.5 M reads)." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- "Both λ and (\\|w\\|, w) are used in the same step: λ for the ra/rb branch, and the word's length and offset for `len`, the small/rule choice and the copy." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- "`slots`, `write_cells`, the arena, the rules and the top slot are otherwise unchanged." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- "With `fresh_room` both kernels reserve wide room, so the arena and the rules lie at the same offsets." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-15T13:29Z] "Keeping either one leaves the other's load dependent, so only the full 20-byte slot removes the dependency." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- [2026-11-15T13:29Z] "A 12- or 16-byte slot would pay most of the line cost and keep the dependent load." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- [2026-11-15T13:29Z] "Dropping the word prefetch would have been the alternative." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- [2026-11-15T13:29Z] "I kept it, because the arena words read by `copy16` were hinted before." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- [2026-11-15T13:29Z] "Top-front reads: 14,815,055. ## Hypothesis At a worker's L3 share the induction misses only on first touch (#397, the locality-bound note), so the lever is distinct lines." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))
- [2026-11-15T13:29Z] "If the footprint cut pays, it shows most at **4 threads** (co-running interference: #408 gained 0% at 1–2 threads, 4% less CPU at 4)." (https://github.com/lmmx/mibz2/pull/414 (PR body, current text))

**same_quantity_facts:** 91 matching lines in 61 documents dated before commit_to_build, searched on `bottom slot|8.byte|footprint` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-14T21:48Z] docs/journal/monodromy/2026-11-14-locality-bound/method.md (doc): "Footprint of the induction (distinct lines × 64 B): 3.24 / 3.79 / 5.29 MiB (min, median," — https://github.com/lmmx/mibz2/blob/5c70b75c2671/docs/journal/monodromy/2026-11-14-locality-bound/method.md
- [2026-11-14T21:48Z] comment 6884716103 (#145 coordinator): "- The induction is at its compulsory floor in a single-block model, with a footprint of 3.2–5.3 MiB (#397, 6884461248). The rest of the traffic is unattributed." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884716103
- [2026-11-15T00:50Z] comment 6886866992 (#145 coordinator): "- **R3. PREFETCH-USEFUL** if prefetch off raises wall time at 4 workers by at least 3%, with the lower bound above 0. The traffic is real demand, fetched early. The lever is footprint and how many working sets are resident at once, and an induction-cap dose study comes next (k concurrent inductions…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6886866992
- [2026-11-15T10:29Z] comment 6893001194 (#160 coordinator): "For context: the bundle we already hold shows every phase slowing by 1.51–1.58× from 1 to 4 workers, in proportion to its base share. That is exploratory and decides nothing. So no phase is intrinsically the one that suffers; the open question is whose footprint causes the slowdown." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6893001194
- [2026-11-15T11:55Z] comment 6893931538 (#145 coordinator): "- At 4 workers, every Cells phase's CPU per byte inflates about 1.55× from co-running interference. Induction is about 33% of worker CPU, with a footprint of 3.2–5.3 MiB per block." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6893931538
- [2026-11-15T12:24Z] comment 6894274731 (#145 monodromy): "- the median footprint goes from 3.79 to 2.83 MiB;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894274731
- [2026-11-15T12:32Z] comment 6894367370 (#145 louis via chatgpt): "<caption>Median modelled induction footprint: 3.79 MiB → 2.83 MiB. Not a prediction of total decoder memory usage or speed.</caption>" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894367370
- [2026-11-15T12:33Z] comment 6894374065 (#145 coordinator): "- **The structural line route** is a layout change: 8-byte bottom slots, about 75% of the induction's lines, at the cost of an extra dependent load. Unmeasured, it is the next candidate in the induction direction." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894374065
- [2026-11-15T12:45Z] comment 6894519106 (#145 coordinator): "- **The gain grows with co-running Cells workers.** There's none at 1–2 threads, 4% less CPU at 4, and 5% faster at 5. That is consistent with a smaller per-worker footprint easing co-running interference." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894519106
- [2026-11-15T13:00Z] comment 6894696008 (#145 louis via chatgpt): "MONODROMY has identified a possible asymmetric layout with 20-byte top slots and 8-byte bottom slots. That could reduce the induction's modelled cache-line footprint by another 25%, although it introduces an additional dependent load on many accesses. It remains unmeasured, and the earlier failed 1…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894696008

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #414) / Optimiser: unrecorded (no comment from the role naming #414)

**monodromy_involved:** yes: cited in the case's own documents (records 0); origin field names MONODROMY

**coordinator_involved:** no citation found

## cells-equal-elision: Equal one-row cell's word linked in the slot it lets go

- **PRs:** [#418](https://github.com/lmmx/mibz2/pull/418)
- **Journal sources:** docs/journal/2026-11-15-equal-cell-elision.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** per PR #418 (MONODROMY named)
- **commit_to_build:** 2026-11-15T14:05Z, PR #418 first commit (author date) (https://github.com/lmmx/mibz2/pull/418/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **Less arena traffic:** about 0.08 to 0.16 B per output byte less sequential arena write, in streaming lines that, at 4 threads, compete for bandwidth (b(4) = 15.5 B/B, 6882123681)." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))

**must_hold** (quoted):

- "The cell is one point, so the word is a factor of exactly one later word: that reader (a small cell in the induction, or the writer's copy of a rule's small factor) copies its two factors' bytes." (https://github.com/lmmx/mibz2/pull/418 (first commit message))
- "MONODROMY's equal one-row cell elision (journal `2026-11-10-rauzy-veech.md` §6.4 and §8.8; z re-counted in #410; steelman 6881405112), built into `induce_cells` for CI's paired bench to judge." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))
- "**Long z words** (22% of z) keep their rule." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))
- "- **Less arena traffic:** about 0.08 to 0.16 B per output byte less sequential arena write, in streaming lines that, at 4 threads, compete for bandwidth (b(4) = 15.5 B/B, 6882123681)." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-15T14:05Z] "The cost is one extra `copy16` per reader, against two saved per link: about −2 × 4.29 M + 3.50 M + (writer readers) copies." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))
- [2026-11-15T14:05Z] "A chain of z cells links every other cell, because a z cell whose factor is linked materialises as before." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))
- [2026-11-15T14:05Z] "- **Where it should show:** a small fall in Cells' CPU and wall, mostly at 4 threads." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))
- [2026-11-15T14:05Z] "- **Expectation:** the expected gain is small, a few percent of the induction at most. #410's line census says the slot regions can't shrink this way." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))
- [2026-11-15T14:05Z] "- Words are compared by their lengths (marks included) and their bytes, because offsets shift once a word is linked." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))
- [2026-11-15T14:05Z] "- It still compares `rule_end` and `pos`, which are unchanged because only small words are linked." (https://github.com/lmmx/mibz2/pull/418 (PR body, current text))

**same_quantity_facts:** 32 matching lines in 17 documents dated before commit_to_build, searched on `equal (step|cell)|one.row cell|elision` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-14T17:18Z] comment 6881329630 (#145 coordinator): "- Only an equal one-row cell can be left without a write. Linking those in the sweep and walking the 0.842 r points at the end makes 0.970 r cells against 1.812 r." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6881329630
- [2026-11-14T17:21Z] comment 6881363652 (#145 coordinator): "- **What it would remove:** 0.842 r of Cells' 1.812 r cells are equal one-row cells that wouldn't need writing. The slot orders are 65% of Cells' cache lines. If the skipped cells account for their share of those lines, that's up to about 46% of the slot-order traffic. That figure comes from counts…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6881363652
- [2026-11-14T18:20Z] comment 6882129734 (#160 coordinator): "- Also say whether the result changes with the 20 B slot (about 11% fewer lines, if it merges) and with the elision screen (preconditions in 6881408836)." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6882129734
- [2026-11-14T19:52Z] comment 6883302716 (#145 coordinator): "- **CLOSED** if G ≥ 0.85: less than a 15% cut, even with an oracle. Locality scheduling under (a) goes on the Rejected list, and effort returns to shrinking the representation, elision, and shared working sets." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6883302716
- [2026-11-14T20:04Z] comment 6883444614 (#145 coordinator): "**A note for diet step 2.** The occupied slot extent is 2r₀ + U (equal steps allocate no fresh slot), not 2(r + S). So the bytes a narrower slot saves are 4 × (2r₀ + U). The step-2 builder is told to count it that way." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6883444614
- [2026-11-14T21:48Z] comment 6884716103 (#145 coordinator): "4. **INDUCTION** if s_ind ≥ 0.50 and U_ind ≥ U_wr. The diet and elision stay first." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884716103
- [2026-11-14T22:05Z] comment 6884914379 (#145 coordinator): "**The diet ends at step 1,** the 20 B slot, merged in #391 (MOVES THE PLATEAU, Q = 0.904). **Release 0.3.1 now proceeds with step 1 alone,** as Louis authorised, under `docs/RELEASING.md`. Bandwidth work continues through the attribution study (6884716103, amended 6884895257) and the elision count." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884914379
- [2026-11-15T11:55Z] comment 6893931538 (#145 coordinator): "- If the structural bound is small (say < 10% of bytes), say so plainly. That tells us to stop the elision route." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6893931538
- [2026-11-15T12:24Z] comment 6894274731 (#145 monodromy): "**z**, the equal one-row cells, on this cohort: **0.762 r** (0.691 / 0.752 / 0.861 r a block), 0.451 of cells. The 0.842 r on record was an older cohort." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894274731
- [2026-11-15T12:33Z] comment 6894374065 (#145 coordinator): "- **The equal-one-row-cell elision is MONODROMY's** (10 November, §6.4). It skips induction steps, it is distinct from #408, and it is not yet exploited. Its current ceiling is 0.762 r on this cohort (6894274731)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894374065

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #418) / Optimiser: unrecorded (no comment from the role naming #418)

**monodromy_involved:** yes: cited in the case's own documents (records 0); origin field names MONODROMY

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/222#issuecomment-6896264451

## crossover-10: Workers below 10 threads on their own cores; teams from 10

- **PRs:** [#425](https://github.com/lmmx/mibz2/pull/425)
- **Journal sources:** none listed
- **i9 recipes:** crossover/run-20261115T15
- **Deciding machine:** i9-10900K (bench-crossover)
- **Origin (as recorded):** per PR #425 (Louis named)
- **commit_to_build:** 2026-11-15T15:27Z, PR #425 first commit (author date) (https://github.com/lmmx/mibz2/pull/425/commits)
- **Cost:** i9 15.3 over 2 bundle(s) min [crossover/run-20261115T150151Z: 7.7 min (bundle start (name)→evidence commit); crossover/run-20261115T151452Z: 7.6 min (bundle start (name)→evidence commit)]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Peak RSS roughly doubles at 6–9 threads, under 200 MB, weighed for a 16 GB laptop as acceptable. ## The change - `WORKERS_BELOW_THREADS = 10` in `src/parallel.rs`:" (https://github.com/lmmx/mibz2/pull/425 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-crossover: "The crossover: shipped plan vs one block a thread forced, 4-20 threads (#145 LOG)"
- "parallel: workers below 10 threads on their own cores; teams from 10 (the crossover's run 2)" (https://github.com/lmmx/mibz2/pull/425 (first commit message))
- "Moves the default decoder's switch from workers (one block a thread, Cells inverting) to teams: workers now run whenever fewer than 10 threads each have a physical core of their own." (https://github.com/lmmx/mibz2/pull/425 (PR body, current text))
- "- Unchanged: below 6 cores; 10 threads or more (the i9's three teams of three; teams of 2 elsewhere)." (https://github.com/lmmx/mibz2/pull/425 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-15T15:27Z] "Moves the default decoder's switch from workers (one block a thread, Cells inverting) to teams: workers now run whenever fewer than 10 threads each have a physical core of their own." (https://github.com/lmmx/mibz2/pull/425 (PR body, current text))
- [2026-11-15T15:27Z] "`the_policy` now asserts workers at 6–9 threads on the i9, teams for 8 and 16 threads on 6 and 8 cores, workers for 8 threads on 16 cores. ## Measurement CI's runners have 2 physical cores, so its bench runs the same plan in both arms and should show no change; the i9 run above is the evidence." (https://github.com/lmmx/mibz2/pull/425 (PR body, current text))

**same_quantity_facts:** 6 matching lines in 6 documents dated before commit_to_build, searched on `crossover|workers? below 10|one block a thread` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-08T09:58Z] comment 6720088760 (#145 coordinator): "**Level 3 (information only):** 0.2.6's default is 1.047 / 1.067 against 0.2.4, so the crossover between the plans lies between levels 2 and 3." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6720088760
- [2026-11-10T19:24Z] comment 6777025573 (#145 chatgpt): "- [Cells stage verdict](https://github.com/lmmx/mibz2/pull/145#issuecomment-6775038009): 0.815/0.827/0.818 over 233 Wikidata blocks. Inversion stage only. [Crossover verdict](https://github.com/lmmx/mibz2/pull/145#issuecomment-6775000939): keep threshold 6. These are the recorded Coordinator result…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6777025573
- [2026-11-14T22:02Z] comment 6884880570 (#145 louis via chatgpt): "**Which beats `rep stosb` at 10–20 MiB?** I cannot responsibly give a universal winner or crossover without measurements on the actual reuse pattern. My expectation is retained mappings plus selective clearing for dense repeated use; lazy anonymous allocation can be valuable for sparse first use. G…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884880570
- [2026-11-15T14:56Z] comment 6896455773 (#145 coordinator): "The run: i9, master, `mibz2` (shipped plan) against `mibz2-workers` (one block a thread, Cells), Wikidata, 10 rounds, threads 4–10, 12, 16, 20. Ratio = workers wall / shipped wall, paired by round, 95% interval." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6896455773
- [2026-11-15T15:11Z] comment 6896638632 (#145 coordinator): "`bench-crossover` run 1 (crossover/run-20261115T150151Z, sha256 d4b3a00d…0355 checked; i9, master 7f56ce08, 10 rounds): **INVALID under its own rule (LOG 6896455773).** At 4 and 5 threads, where both arms should run the same plan, `mibz2-workers` was 10.6% [8.4, 12.9] and 11.6% [8.9, 14.3] slower. …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6896638632
- [2026-11-15T15:23Z] comment 6896798131 (#145 coordinator): "**Under the rule:** one block a thread is faster at every count from 6 to 9 and slower at 10, so N = 9 and the raise is `COOP_MIN_CORES = 10`. But at 9 threads it costs 12.4% more CPU, over the rule's 10%, so that raise is **flagged to Louis, not taken**. Counts 6–8 win on wall and on CPU alike." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6896798131

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** crossover/run-20261115T150151Z (7.7 min; The crossover: shipped plan vs one block a thread forced, 4-20 threads (#145 LOG)); crossover/run-20261115T151452Z (7.6 min; The crossover: shipped plan vs one block a thread forced, 4-20 threads (#145 LOG))

**Review routing:** Researcher: unrecorded (no comment from the role naming #425, bench-crossover) / Optimiser: unrecorded (no comment from the role naming #425, bench-crossover)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

**Notes:** 2 i9 bundles over 1 recipe(s); coders check whether master moved between them.

## cells-bottom-dead-buffer: Cells bottom slots in the dead output buffer, rule positions in dead top slots

- **PRs:** [#433](https://github.com/lmmx/mibz2/pull/433)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI and i9 (release scaling)
- **Origin (as recorded):** per PR #433 (MONODROMY named)
- **commit_to_build:** 2026-11-15T18:39Z, PR #433 opened (https://github.com/lmmx/mibz2/pull/433)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Only the forced route reaches that; on Wikidata the maximum is 0.67." (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- "This is counted exactly on the 196 Wikidata blocks of `corpora/wikidata.bz2`. \| per block, MiB (min / median / max) \| before \| after \| \|---\|--:\|--:\| \| tops + bottoms + arena + rules + positions \| 2.07 / 2.39 / 3.37 \| **1.60 / 1.86 / 2.61** \| \| removed: bottoms 8L \| \| 0.37 / 0.41 / 0.…" (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- "`wikidata` peak RSS 60 → 56 MB at 2t (−5.1%, **less**) and 93 → 85 MB at 4t (−7.5%). \| All 21 rows give the same bytes. ## Local timing (indicative only) Best of 5, three interleaved rounds, on a 4-core Xeon with 2 MiB L2 a core: \| \| master \| this PR \| \|---\|--:\|--:\| \| `decode_with::<Cells…" (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))

**must_hold** (quoted):

- "- **The writer's first-place table (4 B a rule) goes into the top slots**, which are dead once the induction returns." (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- "It falls back to its own vector when there are more than five rules a letter." (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- "It loses no fields and gains no loads or stores. ## Estimate Per block, the distinct memory Cells touches besides the output falls by 8L + 4R (L letters, R rules)." (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- "This is counted exactly on the 196 Wikidata blocks of `corpora/wikidata.bz2`. \| per block, MiB (min / median / max) \| before \| after \| \|---\|--:\|--:\| \| tops + bottoms + arena + rules + positions \| 2.07 / 2.39 / 3.37 \| **1.60 / 1.86 / 2.61** \| \| removed: bottoms 8L \| \| 0.37 / 0.41 / 0.…" (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-10T17:52Z] "No work read inverts in fewer than n steps or builds the tree from a column." (https://github.com/lmmx/mibz2/pull/433 (first commit message))
- [2026-11-15T18:39Z] "MONODROMY's first candidate for QUEUED 6898971658 (state whose lifetime can be cut short). ## What changes Two pieces of Cells' short-lived state move into memory that is dead while they live:" (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- [2026-11-15T18:39Z] "It loses no fields and gains no loads or stores. ## Estimate Per block, the distinct memory Cells touches besides the output falls by 8L + 4R (L letters, R rules)." (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- [2026-11-15T18:39Z] "This is counted exactly on the 196 Wikidata blocks of `corpora/wikidata.bz2`. \| per block, MiB (min / median / max) \| before \| after \| \|---\|--:\|--:\| \| tops + bottoms + arena + rules + positions \| 2.07 / 2.39 / 3.37 \| **1.60 / 1.86 / 2.61** \| \| removed: bottoms 8L \| \| 0.37 / 0.41 / 0.61 \| \| removed: positions 4R \| \| 0.10 / 0.12 / 0.15 \| The ratio is 0.776× on every block (0.773…" (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- [2026-11-15T18:39Z] "For comparison, #414 removed 12L. ## Contract change A refusal on the budget now leaves `out` holding the bottom slots." (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))
- [2026-11-15T18:39Z] "This is safe because:" (https://github.com/lmmx/mibz2/pull/433 (PR body, current text))

**same_quantity_facts:** 74 matching lines in 55 documents dated before commit_to_build, searched on `dead (output )?buffer|bottom slot|footprint` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-15T12:24Z] comment 6894274731 (#145 monodromy): "- the median footprint goes from 3.79 to 2.83 MiB;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894274731
- [2026-11-15T12:32Z] comment 6894367370 (#145 louis via chatgpt): "<caption>Median modelled induction footprint: 3.79 MiB → 2.83 MiB. Not a prediction of total decoder memory usage or speed.</caption>" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894367370
- [2026-11-15T12:33Z] comment 6894374065 (#145 coordinator): "- **The structural line route** is a layout change: 8-byte bottom slots, about 75% of the induction's lines, at the cost of an extra dependent load. Unmeasured, it is the next candidate in the induction direction." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894374065
- [2026-11-15T12:45Z] comment 6894519106 (#145 coordinator): "- **The gain grows with co-running Cells workers.** There's none at 1–2 threads, 4% less CPU at 4, and 5% faster at 5. That is consistent with a smaller per-worker footprint easing co-running interference." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894519106
- [2026-11-15T13:00Z] comment 6894696008 (#145 louis via chatgpt): "MONODROMY has identified a possible asymmetric layout with 20-byte top slots and 8-byte bottom slots. That could reduce the induction's modelled cache-line footprint by another 25%, although it introduces an additional dependent load on many accesses. It remains unmeasured, and the earlier failed 1…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894696008
- [2026-11-15T14:41Z] comment 6896264451 (#222 coordinator): "2. #414 (8-byte bottom slots) merged before its Miri job finished, on Louis's instruction; Miri then passed on head and merge commit. Shipped as 0.3.3, released ahead of its tag-to-tag bench, also on his instruction; the bench then showed 4 threads −4.2% faster, CPU −4.0%." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6896264451
- [2026-11-15T14:46Z] comment 6896328249 (#145 observer): "1. 0.3.3 ships #414 8 B bottom slots atop #408 recycling. Tag CI Wiki4t wall−4.2%, CPU−4.0%, RSS102→98 MB; Louis directs next effort at the <6 Cells/≥6 Teams knee." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6896328249
- [2026-11-15T15:43Z] comment 6897050326 (#145 louis via chatgpt): "Across the Wikidata benchmark's admitted Cells blocks, the slots shrink by 30%, and the complete induction buffer's occupied cache-line footprint falls by approximately 21.6%." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6897050326
- [2026-11-15T16:47Z] comment 6897885393 (#145 monodromy): "\| bottom slots \| 8 B × letters \| 0.37 / 0.41 / 0.61 \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6897885393
- [2026-11-15T18:11Z] comment 6898971658 (#145 coordinator): "**Why this is back on (Louis, this session).** The coordinator framed the plateau above 10 threads as something to diagnose before building. Louis disagrees, and he is right. Diagnosis failed once already: the hardware counters could not separate the causes. What moved the team threshold from 6 to …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6898971658

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #433) / Optimiser: unrecorded (no comment from the role naming #433)

**monodromy_involved:** yes: cited in the case's own documents; records naming its PRs or recipes: 2026-11-15-equal-words.md, 2026-11-15-replay-and-team.md; origin field names MONODROMY

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6899524781

**Notes:** PR #433's earliest listed commit is 2026-11-10T17:52Z, 121 h before it opened; it may belong to a base branch, so it is not used.

## rules-in-output-spare: Cells bottom slots and rules in the output's spare capacity (Pathfinder)

- **PRs:** [#449](https://github.com/lmmx/mibz2/pull/449)
- **Journal sources:** docs/journal/2026-11-15-rules-in-output-spare.md
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** the Pathfinder
- **commit_to_build:** 2026-11-15T23:29Z, PR #449 first commit (author date) (https://github.com/lmmx/mibz2/pull/449/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.09; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- "Cells: bottom slots and rules in the final output's spare capacity The induction writes its rules once and only the writer reads them: 9.1 bytes a letter of the kernel's buffer on Wikidata." (https://github.com/lmmx/mibz2/pull/449 (first commit message))
- "Prices and decision rule:" (https://github.com/lmmx/mibz2/pull/449 (PR body, current text))
- "The full record is `docs/journal/2026-11-15-rules-in-output-spare.md`. ## What changes Cells' induction writes one 16-byte rule for each cell whose word is longer than 16 bytes, and only the writer reads them." (https://github.com/lmmx/mibz2/pull/449 (PR body, current text))
- "On Wikidata the rules are 9.1 B a letter, a quarter of the kernel's buffer." (https://github.com/lmmx/mibz2/pull/449 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-15T23:56Z] "QUEUED 6902723569 on #145 (memory bandwidth is the lever; build cuts now)." (https://github.com/lmmx/mibz2/pull/449 (PR body, current text))

**same_quantity_facts:** 104 matching lines in 62 documents dated before commit_to_build, searched on `spare capacity|dead|rules|output buffer` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-15T21:11Z] comment 6901428255 (#145 monodromy): "- **Correctness (C):** all 233 Wikidata blocks are byte-equal to `bwt::walk`, in 10 to 16 rounds (median 12), with 1.94 r rules." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901428255
- [2026-11-15T21:22Z] docs/journal/monodromy/2026-11-15-compaction-rounds.md (doc): "max); rules 1.51 / 1.94 / 2.24 r. Per block in `rounds_wikidata.jsonl`." — https://github.com/lmmx/mibz2/blob/37f50a437922/docs/journal/monodromy/2026-11-15-compaction-rounds.md
- [2026-11-15T21:46Z] comment 6901872654 (#145 coordinator): "- **C** (your 10 November record, §5.6): with equal rules shared, the grammar is the Lyndon SLP, **0.470 S = 0.852 r** rules against 1.812 r. That is a 53% cut of the largest remaining item." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901872654
- [2026-11-15T21:48Z] comment 6901897952 (#145 pathfinder): "- The other 11 points keep the fetch-ahead off the rules. It was hinting whatever word a letter a few slots on had; at one site 1.37 M of 11 M hints named a rule, 97% of them already sent. Left alone, the hints would read the rules straight back into the cache, and the change would not test its own…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901897952
- [2026-11-15T21:54Z] comment 6901978751 (#145 monodromy): "- **Where the savings fall:** most duplicates are short words, so the arena gains more than the rules (30% of long rules)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901978751
- [2026-11-15T21:55Z] comment 6901989466 (#145 coordinator): "- #444: rules written past the cache, at +3.4% whole-decoder instructions." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901989466
- [2026-11-15T22:42Z] comment 6902593379 (#145 pathfinder): "- The kernel can only compare *pointers*: 13.2% of cells have the step before's two factors (11.9% short words, 1.3% rules)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6902593379
- [2026-11-15T22:52Z] comment 6902723569 (#145 coordinator): "- Instead of checking in the register-starved induction loop, which cost +41%, merge equal rules in a cheap pass between the induction and the writer, or as the writer meets them. The writer then reads and expands about half as many distinct rules." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6902723569
- [2026-11-15T22:58Z] comment 6902797865 (#145 pathfinder): "- **The pass costs more than that.** Finding twins after the fact needs a table over all rules with short factors compared by content: a random-access table of several hundred kilobytes, and by my estimate 25 instructions a letter or more, against a writer of 54. It adds working set in order to shr…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6902797865
- [2026-11-15T23:23Z] docs/journal/2026-11-15-memory-lever.md (doc): "\| word store (arena + rules) \| 15.64 B \| 14.28 B (−8.7%) \|" — https://github.com/lmmx/mibz2/blob/d7c26afb6ec7/docs/journal/2026-11-15-memory-lever.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #449) / Optimiser: unrecorded (no comment from the role naming #449)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/161#issuecomment-6903560983

## lazy-letters: Lazy letters for Cells (memory lever): from 8 threads

- **PRs:** [#441](https://github.com/lmmx/mibz2/pull/441), [#445](https://github.com/lmmx/mibz2/pull/445), [#447](https://github.com/lmmx/mibz2/pull/447), [#452](https://github.com/lmmx/mibz2/pull/452)
- **Journal sources:** docs/journal/2026-11-15-lazy-letters.md, docs/journal/2026-11-15-memory-lever.md
- **i9 recipes:** memory-lever, lazy-crossover
- **Deciding machine:** i9-10900K (bench-memory-lever, bench-lazy-crossover)
- **Origin (as recorded):** the Pathfinder priced it (#441, #445); the Coordinator's memory-lever arms (#447)
- **commit_to_build:** 2026-11-15T21:07Z, PR #441 first commit (author date) (https://github.com/lmmx/mibz2/pull/441/commits)
- **Cost:** i9 69.4 over 4 bundle(s) min [memory-lever/run-20261115T233002Z: 9.1 min (bundle start (name)→evidence commit); lazy-crossover/run-20261116T014443Z: 20.1 min (bundle start (name)→evidence commit); lazy-crossover/run-20261116T021858Z: 20.1 min (bundle start (name)→evidence commit); lazy-crossover/run-20261116T093528Z: 20.1 min (bundle start (name)→evidence commit)]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.17; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "On Wikidata's 196 admitted blocks the pool peaks at 0.318 of the letters (0.337 at most), so memory besides the output falls from 35.6 to 22.0 bytes a letter." (https://github.com/lmmx/mibz2/pull/441 (first commit message))
- "- **The footprint claim holds.** On Wikidata's 196 admitted blocks the pool peaks at 0.318 of the letters (0.295 to 0.337), which is MONODROMY's width for the right end." (https://github.com/lmmx/mibz2/pull/441 (PR body, current text))
- "research(pathfinder): lazy letters, the lean build: +28% instructions, over its +10% line The two levers named in LOG 6901919943 (slots taken where a letter is read from its entry, an original letter's word built with two additions) bring first stage plus induction from 266.6 to 252.2 instructions …" (https://github.com/lmmx/mibz2/pull/445 (first commit message))

**must_hold** (quoted):

- queue entry bench-memory-lever: "Is memory per core the lever at 10 threads? Lazy letters and rules past the cache as opt-in arms, 4-12 threads (#447; rule LOG 6902022809)"
- queue entry bench-lazy-crossover: "Confirms LAZY_FROM_THREADS = 8 and reads the memory dose (10t/1t), 4-20 threads (#452; rule 6903395401 as amended by LOG 6904285481)"
- "research(pathfinder): lazy letters for Cells priced and built; stopped at its instruction rule Cells with a slot only for a cell: original letters as two streams in the output buffer (a 4-byte entry in bottom order, a byte in top order), slots from a paired pool." (https://github.com/lmmx/mibz2/pull/441 (first commit message))
- "Answers QUEUED 6900823509 on #145 (lazy letters for Cells); the prices and decision rule were posted first, as LOG 6901130348. ## What it records `docs/journal/2026-11-15-lazy-letters.md`, with the candidate kernel as a patch beside it." (https://github.com/lmmx/mibz2/pull/441 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-15T21:07Z] "research(pathfinder): lazy letters for Cells priced and built; stopped at its instruction rule Cells with a slot only for a cell: original letters as two streams in the output buffer (a 4-byte entry in bottom order, a byte in top order), slots from a paired pool." (https://github.com/lmmx/mibz2/pull/441 (first commit message))
- [2026-11-15T22:07Z] "Both are compared state by state against the original induction; the staged kernel's store fence has a model that covers every exit (Guardian D108)." (https://github.com/lmmx/mibz2/pull/447 (first commit message))
- [2026-11-15T22:37Z] "- **A model checks every exit.** `send_rules` marks each send of whole lines; the fence clears the mark where it runs `sfence`; the writer asserts the mark is clear." (https://github.com/lmmx/mibz2/pull/447 (PR body, current text))
- [2026-11-16T00:59Z] "A plan of that many workers or more inverts with `bwt::LazyCells` (`Plan::lazy`); fewer keep `bwt::Cells`." (https://github.com/lmmx/mibz2/pull/452 (PR body, current text))
- [2026-11-16T00:59Z] "CI's bench runs at 1, 2 and 4 threads, so it should be level." (https://github.com/lmmx/mibz2/pull/452 (PR body, current text))
- [2026-11-16T00:59Z] "- **A machine of fewer than 6 cores asked for 8 or more threads** now gets lazy workers too." (https://github.com/lmmx/mibz2/pull/452 (PR body, current text))

**same_quantity_facts:** 74 matching lines in 54 documents dated before commit_to_build, searched on `lazy letters?|memory per core|footprint|rules past the cache` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-15T11:55Z] comment 6893931538 (#145 coordinator): "- At 4 workers, every Cells phase's CPU per byte inflates about 1.55× from co-running interference. Induction is about 33% of worker CPU, with a footprint of 3.2–5.3 MiB per block." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6893931538
- [2026-11-15T12:24Z] comment 6894274731 (#145 monodromy): "- the median footprint goes from 3.79 to 2.83 MiB;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894274731
- [2026-11-15T12:32Z] comment 6894367370 (#145 louis via chatgpt): "<caption>Median modelled induction footprint: 3.79 MiB → 2.83 MiB. Not a prediction of total decoder memory usage or speed.</caption>" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894367370
- [2026-11-15T12:45Z] comment 6894519106 (#145 coordinator): "- **The gain grows with co-running Cells workers.** There's none at 1–2 threads, 4% less CPU at 4, and 5% faster at 5. That is consistent with a smaller per-worker footprint easing co-running interference." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894519106
- [2026-11-15T13:00Z] comment 6894696008 (#145 louis via chatgpt): "MONODROMY has identified a possible asymmetric layout with 20-byte top slots and 8-byte bottom slots. That could reduce the induction's modelled cache-line footprint by another 25%, although it introduces an additional dependent load on many accesses. It remains unmeasured, and the earlier failed 1…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894696008
- [2026-11-15T15:43Z] comment 6897050326 (#145 louis via chatgpt): "Across the Wikidata benchmark's admitted Cells blocks, the slots shrink by 30%, and the complete induction buffer's occupied cache-line footprint falls by approximately 21.6%." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6897050326
- [2026-11-15T18:54Z] comment 6899524781 (#145 coordinator): "- **Why it matters for space:** under the knee law, live bytes per core is what matters. Two serial chains share one footprint, with no shared 3.6 MB table and no rounds. The feathered screen died of rescanning in rounds, and this has none." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6899524781
- [2026-11-15T19:33Z] comment 6900062843 (#145 louis via monodromy): "> 2. Lazy letters: originals implicit, a materialised pool ≤ W, the pool in the dead output buffer. Falsifier: base + pool ≥ 0.8 × 20 r." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6900062843
- [2026-11-15T20:26Z] comment 6900823509 (#145 coordinator): "- **B: lazy letters with eager free steps.** Pool about 0.11 r, 20% fewer steps, plus the worklist." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6900823509
- [2026-11-15T20:48Z] comment 6901130348 (#145 pathfinder): "2. **Counts, before timing.** The kernel's own pool peak is at most 0.40 of the letters on every admitted Wikidata block, and `slots` + induction are at most 187 instructions a letter (170 + 10%). If either fails, I report it and stop: the footprint claim or the price is wrong." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901130348

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #441, #445, #447, #452, bench-memory-lever, bench-lazy-crossover) / Optimiser: unrecorded (no comment from the role naming #441, #445, #447, #452, bench-memory-lever, bench-lazy-crossover)

**monodromy_involved:** yes: cited in the case's own documents

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6901445234, https://github.com/lmmx/mibz2/pull/145#issuecomment-6901989466

**Notes:** 4 i9 bundles over 2 recipe(s); coders check whether master moved between them.

## lazy-fetch-ahead: Fetch-ahead hints in the lazy induction

- **PRs:** [#459](https://github.com/lmmx/mibz2/pull/459)
- **Journal sources:** docs/journal/2026-11-16-lazy-fetch-ahead.md
- **i9 recipes:** lazy-hint
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** per PR #459 (Coordinator, Louis named)
- **commit_to_build:** 2026-11-16T02:00Z, PR #459 opened (https://github.com/lmmx/mibz2/pull/459)
- **Cost:** i9 3.4 over 1 bundle(s) min [lazy-hint/run-20261116T021013Z: 3.4 min (bundle start (name)→evidence commit)]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "It merges or closes on the i9's reading under that rule. ## Why The eager induction spends 16.7 instructions a letter on hints because they hide memory latency." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))
- "Its read misses are the letters the *other* front reached first, whose cells lie in slots anywhere in the pools: 1.45 of its 2.44 first-level read misses a letter in callgrind's model (`2026-11-16-lazy-overhead.md`, #457). ## What is added - **`mibz2-workers-lazy-hint`** (`bwt::HintLazyCells`): the…" (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))
- "- **The gain cannot be measured here.** Callgrind does not model a hint, and this container's caches are too large for the misses to cost (an indicative reading: 1.05, 1.01 and 1.03 of plain at 1, 2 and 4 threads, inside ±8% of noise)." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-lazy-hint: "Fetch-ahead in the lazy kernel: lazy-hint vs lazy at 4/8/9/10 threads (#459; rule LOG 6905226406)"
- "**The decision rule is LOG 6905226406 on #145, posted before any i9 data.** The full record, with the safety argument, is `docs/journal/2026-11-16-lazy-fetch-ahead.md`." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))
- "It merges or closes on the i9's reading under that rule. ## Why The eager induction spends 16.7 instructions a letter on hints because they hide memory latency." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))
- "- **`just bench-lazy-hint`:** `-lazy` against `-lazy-hint` at 4, 8, 9 and 10 threads, ten paired rounds, Wikidata." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-16T02:00Z] "It merges or closes on the i9's reading under that rule. ## Why The eager induction spends 16.7 instructions a letter on hints because they hide memory latency." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))
- [2026-11-16T02:00Z] "Its read misses are the letters the *other* front reached first, whose cells lie in slots anywhere in the pools: 1.45 of its 2.44 first-level read misses a letter in callgrind's model (`2026-11-16-lazy-overhead.md`, #457). ## What is added - **`mibz2-workers-lazy-hint`** (`bwt::HintLazyCells`): the lazy kernel with its induction fetching ahead." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))
- [2026-11-16T02:00Z] "CI's bench cannot see it either way, because it never runs the lazy kernel." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))
- [2026-11-16T02:00Z] "- **The gain cannot be measured here.** Callgrind does not model a hint, and this container's caches are too large for the misses to cost (an indicative reading: 1.05, 1.01 and 1.03 of plain at 1, 2 and 4 threads, inside ±8% of noise)." (https://github.com/lmmx/mibz2/pull/459 (PR body, current text))

**same_quantity_facts:** 92 matching lines in 54 documents dated before commit_to_build, searched on `fetch.ahead|prefetch|hint|dependent load` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-15T13:00Z] comment 6894696008 (#145 louis via chatgpt): "MONODROMY has identified a possible asymmetric layout with 20-byte top slots and 8-byte bottom slots. That could reduce the induction's modelled cache-line footprint by another 25%, although it introduces an additional dependent load on many accesses. It remains unmeasured, and the earlier failed 1…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894696008
- [2026-11-15T13:38Z] comment 6895145558 (#145 coordinator): "- extra dependent loads on 53.7% of slot reads, as MONODROMY predicted." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6895145558
- [2026-11-15T14:17Z] comment 6895882037 (#145 coordinator): "Equal one-row cell elision (#418, head 93e6f056): CLOSED unmerged. CI bench on Wikidata, 10 paired rounds: wall +11.9% (+9.3, +14.6) at 1 thread, +8.7% at 2, +8.2% at 4; CPU +9–12%; RSS −5.4% at 4 threads only; same bytes everywhere. Census: equal one-row cells are 0.450 of cells, but chains cap on…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6895882037
- [2026-11-15T16:48Z] comment 6897897097 (#145 monodromy): "- **A length byte before each short word in the arena.** That costs about 1 B a short word (≈ 68 k a median block), against the 4 B a rule saved (≈ 32 k rules). Net ≈ 0.06 MiB, about half what I claimed, plus two dependent loads a rule in the writer." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6897897097
- [2026-11-15T17:16Z] comment 6898249821 (#145 coordinator): "- The 16 → 12 B rule, its second candidate, it withdrew after reading `write_cells`: the writer needs both factor lengths, so recovering a short factor's length costs either an arena length byte (net saving ≈ 0.06 MiB, about half its first estimate, plus two dependent loads a rule) or offset bits t…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6898249821
- [2026-11-15T21:48Z] comment 6901897952 (#145 pathfinder): "- The other 11 points keep the fetch-ahead off the rules. It was hinting whatever word a letter a few slots on had; at one site 1.37 M of 11 M hints named a rule, 97% of them already sent. Left alone, the hints would read the rules straight back into the cache, and the change would not test its own…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901897952
- [2026-11-16T01:27Z] comment 6904790860 (#145 pathfinder): "One thing the table in §2 suggests that the rule does not cover: the lazy loop has **no fetch-ahead at all**, where the eager loop spends 16.7 instructions a letter on it, by choice. Its word copies and its slot reads through tagged entries are the candidates. That would *add* instructions, so it i…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904790860
- [2026-11-16T01:33Z] comment 6904862957 (#145 coordinator): "- fetch-ahead, which the lazy kernel lacks entirely and the eager kernel spends 16.7 instructions a letter on." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904862957
- [2026-11-16T01:33Z] comment 6904867571 (#145 pathfinder): "**The lazy loop has no fetch-ahead.** The eager loop spends 16.7 instructions a letter on hints, by choice, because they pay in time. The lazy loop's reads of a word's bytes and of a slot through a tagged entry are not hinted. That adds instructions, so the rule above does not cover it; it is a clo…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904867571
- [2026-11-16T01:59Z] comment 6905226406 (#145 pathfinder): "- **The gain is not known.** Callgrind does not model a hint. An indicative reading on a container whose caches are too large for the misses to cost had the hints at 1.05, 1.01 and 1.03 of plain at 1, 2 and 4 threads, inside its ±8% noise. So I expect a small loss at 4 threads on the i9 and have no…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6905226406

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #459, bench-lazy-hint) / Optimiser: unrecorded (no comment from the role naming #459, bench-lazy-hint)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** PR #459's earliest listed commit is 2026-11-16T01:48Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## lazy-ring: Ring of eight on the lazy kernel (shared words)

- **PRs:** [#464](https://github.com/lmmx/mibz2/pull/464)
- **Journal sources:** docs/journal/2026-11-15-shared-ring8.md, docs/journal/2026-11-16-lazy-ring.md
- **i9 recipes:** lazy-ring
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** the Pathfinder priced it at the Coordinator's request (shared-ring8)
- **commit_to_build:** 2026-11-16T02:18Z, PR #464 opened (https://github.com/lmmx/mibz2/pull/464)
- **Cost:** i9 3.4 over 1 bundle(s) min [lazy-ring/run-20261116T025027Z: 3.4 min (bundle start (name)→evidence commit)]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "CPU at 10 threads over 1 is 2.58 for the eager workers and 1.92 for lazy): build the ring of eight on the lazy kernel as a forced arm, default unchanged." (https://github.com/lmmx/mibz2/pull/464 (PR body, current text))
- "- **`just bench-lazy-ring`:** `-lazy` against `-lazy-ring` at 4, 8, 9 and 10 threads, ten paired rounds, Wikidata. ## What it saves and costs (Wikidata, per letter) \| \| the plan's lazy kernel \| with the ring \| \|---\|--:\|--:\| \| words kept \| 15.62 B \| 11.15 B (−28.6%) \| \| all the kernel t…" (https://github.com/lmmx/mibz2/pull/464 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-lazy-ring: "Ring of eight on the lazy kernel vs plain lazy, 4/8/9/10 threads (#464; rule LOG 6905472931)"
- "CPU at 10 threads over 1 is 2.58 for the eager workers and 1.92 for lazy): build the ring of eight on the lazy kernel as a forced arm, default unchanged." (https://github.com/lmmx/mibz2/pull/464 (PR body, current text))
- "**The decision rule is LOG 6905472931 on #145, posted before any i9 data.** The full record is `docs/journal/2026-11-16-lazy-ring.md`." (https://github.com/lmmx/mibz2/pull/464 (PR body, current text))
- "**This PR does not change any default.** It adds a bench arm and its recipe, and merges or closes on the i9's reading under that rule." (https://github.com/lmmx/mibz2/pull/464 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-16T02:18Z] "- **`just bench-lazy-ring`:** `-lazy` against `-lazy-ring` at 4, 8, 9 and 10 threads, ten paired rounds, Wikidata. ## What it saves and costs (Wikidata, per letter) \| \| the plan's lazy kernel \| with the ring \| \|---\|--:\|--:\| \| words kept \| 15.62 B \| 11.15 B (−28.6%) \| \| all the kernel touches beside the output \| 29.5 B \| 25.0 B (−15%) \| \| induction, instructions \| 201.7 \| 277.8 …" (https://github.com/lmmx/mibz2/pull/464 (PR body, current text))
- [2026-11-16T02:18Z] "New: every binary column to length 10 at every row against the plain walk, with the same cells as the plan's kernel and never more words; a Miri-listed test on repeating columns that requires fewer words and fewer rules; the folded search's unit test (two keys of one folded value are not a match)." (https://github.com/lmmx/mibz2/pull/464 (PR body, current text))

**same_quantity_facts:** 18 matching lines in 14 documents dated before commit_to_build, searched on `ring of eight|shared words?|word store` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-15T23:23Z] docs/journal/2026-11-15-memory-lever.md (doc): "\| word store (arena + rules) \| 15.64 B \| 14.28 B (−8.7%) \|" — https://github.com/lmmx/mibz2/blob/d7c26afb6ec7/docs/journal/2026-11-15-memory-lever.md
- [2026-11-15T23:59Z] comment 6903554082 (#145 monodromy): "**4. Fewer bits for the live set.** Nothing new beyond the equal-word ring (#446, 32% of the word store) and the lazy kernel's implicit originals." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903554082
- [2026-11-15T23:59Z] comment 6903554609 (#145 pathfinder): "\| word store (arena + rules) \| 15.64 B \| 11.15 B (−28.7%) \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903554609
- [2026-11-16T00:01Z] comment 6903591295 (#145 coordinator): "- **Accepted:** the ring of eight's revised verdict in #450, an open cut stacked on lazy letters (−20% of its live memory; −17% on #449's layout)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903591295
- [2026-11-16T00:35Z] docs/journal/monodromy/2026-11-15-equal-words.md (doc): "- *What it catches:* 0.315 of the word store on Wikidata, 83% of the neighbour check's 0.381." — https://github.com/lmmx/mibz2/blob/0647fbefc82a/docs/journal/monodromy/2026-11-15-equal-words.md
- [2026-11-16T00:35Z] docs/journal/monodromy/2026-11-15-least-width.md (doc): "The ring of 8 recent words shares about 32% of the word store, and applies to the live set" — https://github.com/lmmx/mibz2/blob/0647fbefc82a/docs/journal/monodromy/2026-11-15-least-width.md
- [2026-11-16T01:00Z] comment 6904409903 (#145 pathfinder): "**Callgrind's count of the lazy induction is not a gate.** Eight builds of the same loop, differing only in where a rule is written and whether the tail's offsets carry a constant, counted from 176 to 214 instructions a letter. One with fewer live values counted more. So my earlier "register-bound"…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904409903
- [2026-11-16T01:05Z] comment 6904481722 (#145 pathfinder): "\| **The ring of eight** (#450): a word equal to one of the last eight is not written again. \| −4.5 \| −15% \| A search a cell (two 256-bit compares). Callgrind had the eager induction at +76%, an instrument I have since found unreliable for these loops (6904409903 §4). Needs AVX2. \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904481722
- [2026-11-16T01:09Z] comment 6904540455 (#145 coordinator): "The Pathfinder's ring of eight (−15% of lazy's bytes, by de-duplicating twins) is the one word-side cut already priced. It is held until the release's memory-dose run says whether memory pressure is left. A census that says what share of the words could ever be freed early would tell us whether the…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904540455
- [2026-11-16T02:06Z] comment 6905312941 (#145 coordinator): "The penalty of 10 threads over 1 falls from 158% to 92%. That is well above 1, so memory pressure is far from spent. The ring of eight is unheld and goes to the Pathfinder as a memory cut." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6905312941

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #464, bench-lazy-ring) / Optimiser: unrecorded (no comment from the role naming #464, bench-lazy-ring)

**monodromy_involved:** yes: cited in the case's own documents

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** PR #464's earliest listed commit is 2026-11-16T01:48Z, 1 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## lazy-at-10: Lazy workers at 10 threads, teams only past the physical cores

- **PRs:** [#462](https://github.com/lmmx/mibz2/pull/462), [#465](https://github.com/lmmx/mibz2/pull/465)
- **Journal sources:** none listed
- **i9 recipes:** lazy-cap
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Coordinator (per LOG 6906090872)
- **commit_to_build:** 2026-11-16T02:15Z, PR #462 first commit (author date) (https://github.com/lmmx/mibz2/pull/462/commits)
- **Cost:** i9 5.8 over 1 bundle(s) min [lazy-cap/run-20261116T024424Z: 5.8 min (bundle start (name)→evidence commit)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- queue entry bench-lazy-cap: "Lazy workers capped at the physical cores vs shipped teams, 10-20 threads (#465; rule LOG 6905531834)"
- "Workers with lazy letters at 10 threads on a 10-core CPU; teams only past the cores" (https://github.com/lmmx/mibz2/pull/462 (first commit message))
- "At 10 threads on a 10-core CPU, the default becomes one block per thread with lazy letters." (https://github.com/lmmx/mibz2/pull/462 (PR body, current text))
- "- **The service core** was the default at exactly 10 threads on the i9 target (2 teams of 3 plus 2 service workers)." (https://github.com/lmmx/mibz2/pull/462 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-16T02:15Z] "**Tests:** - The plan tests now expect workers, lazy, at 6 to 10 threads on the i9 topology, and teams from 11." (https://github.com/lmmx/mibz2/pull/462 (PR body, current text))
- [2026-11-16T02:15Z] "- `the_service_default` and `tests/service_default.rs` now expect workers at 10 threads on the target, the scalar kernels and the same-shape CPU." (https://github.com/lmmx/mibz2/pull/462 (PR body, current text))

**same_quantity_facts:** 8 matching lines in 7 documents dated before commit_to_build, searched on `lazy (workers )?at 10|10 threads.{0,40}lazy|teams? (at|from) 10|teams only past` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-06T23:42Z] comment 6698812800 (#145 coordinator): "**mibz2 0.2.5 is released.** Its one change is the three-team policy (#234): 3×3 teams at 10 or more threads on a 10-core, 20 MiB-L3 machine." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6698812800
- [2026-11-15T15:27Z] comment 6896854245 (#145 coordinator): "Louis chose teams from 10 (this session), the raise LOG 6896798131 flagged for its 12.4% CPU at 9 threads. #425 makes it: workers whenever fewer than 10 threads each have a physical core of their own; more threads than physical cores keep the teams (workers on SMT siblings were 28–49% slower on the…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6896854245
- [2026-11-15T17:01Z] comment 6898054629 (#222 coordinator): "1. **The team threshold moved.** `bench-crossover` run 1 was invalid by its own sanity check (the forced arm kept a smaller in-flight window: a recipe error of the coordinator's, fixed by #424). Run 2: one block a thread beat the teams at 6–9 threads on the i9 (14%, 11%, 8%, 4% faster; teams of thr…" — https://github.com/lmmx/mibz2/pull/222#issuecomment-6898054629
- [2026-11-15T17:16Z] comment 6898249821 (#145 coordinator): "**What it suggested instead:** a cheap test of the L3 hypothesis on the corpus's own variation (block buffers 1.84–2.96 MiB): one block a thread against teams at 10 threads on the smallest-buffer third of blocks against the largest-buffer third. The hypothesis predicts workers win on the small thir…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6898249821
- [2026-11-15T23:40Z] comment 6903318530 (#145 coordinator): "- **10 threads:** one block a thread with lazy letters beats the shipped **teams** by 5.1% in wall time, at +10.6% CPU." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903318530
- [2026-11-15T23:48Z] comment 6903426903 (#145 coordinator): "**Context:** the i9 run (LOG 6903318530) says memory per core is the lever at high thread counts. Lazy letters, which keeps only live letters, beats the teams at 10 threads despite +28% instructions. So the size of the induction's live state at its busiest moment, your width, now maps directly onto…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903426903
- [2026-11-16T02:06Z] comment 6905312941 (#145 coordinator): "**Peak RSS at 10 threads:** lazy workers 189 MiB, eager 193 MiB, P (teams) 72 MiB." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6905312941

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** memory-lever/run-20261115T233002Z (9.1 min; Is memory per core the lever at 10 threads? Lazy letters and rules past the cache as opt-in arms, 4-12 threads (#447; r…)

**Review routing:** Researcher: unrecorded (no comment from the role naming #462, #465, bench-lazy-cap) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6926545736 (2026-11-16T19:18Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator

## lazy-one-a-core: Past the physical cores: lazy workers, one a core

- **PRs:** [#468](https://github.com/lmmx/mibz2/pull/468)
- **Journal sources:** none listed
- **i9 recipes:** lazy-cap, split-051
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Coordinator
- **commit_to_build:** 2026-11-16T03:02Z, PR #468 first commit (author date) (https://github.com/lmmx/mibz2/pull/468/commits)
- **Cost:** i9 18.9 over 2 bundle(s) min [lazy-cap/run-20261116T024424Z: 5.8 min (bundle start (name)→evidence commit); split-051/run-20261116T145423Z: 13.1 min (bundle start (name)→evidence commit)]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- queue entry bench-lazy-cap: "Lazy workers capped at the physical cores vs shipped teams, 10-20 threads (#465; rule LOG 6905531834)"
- queue entry bench-split-051: "0.5.1's +7% at 10-20 threads split between #462 and #468, plus the CI-i9 gap at 1/4 threads (#489; rule QUEUED 6916536279)"
- "Before, it ran teams (three teams of three on the i9)." (https://github.com/lmmx/mibz2/pull/468 (PR body, current text))
- "**Evidence:** `bench-lazy-cap` on the i9-10900K (#465;" (https://github.com/lmmx/mibz2/pull/468 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-16T03:02Z] "**Tests:** the plan tests (`the_policy`, the lazy test) and `tests/service_default.rs` now expect workers past the cores." (https://github.com/lmmx/mibz2/pull/468 (PR body, current text))

**same_quantity_facts:** 9 matching lines in 9 documents dated before commit_to_build, searched on `one a core|one worker a core|past the (physical )?cores` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md (doc): "\| 2: CPUs 0,2, one a core \| 414 \| 4.37 \| 1.06× (1.04–1.07) \| 85% \| 416 \| 4.34 \| 1.05× (1.04–1.06) \| 85% \| −0.3% (−1.4%, +0.8%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run1-ef28ad32/comment.md
- [2026-11-05T00:47Z] docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md (doc): "\| 2: CPUs 0,1, one a core \| 546 \| 3.19 \| 1.02× (1.01–1.03) \| 85% \| 557 \| 3.19 \| 1.02× (1.02–1.02) \| 86% \| −0.3% (−1.3%, +0.8%) \|" — https://github.com/lmmx/mibz2/blob/509f69e31f7a/docs/journal/2026-11-05-p87-production-validation/bench-run2-ee12bb9e/comment.md
- [2026-11-05T14:01Z] docs/journal/2026-11-05-compact30-staged/bench-run1-9v74.md (doc): "\| 2: CPUs 0,2, one a core \| 699 \| 2.82 \| 1.06× (1.04–1.08) \| 93% \| 638 \| 3.10 \| 1.04× (1.03–1.05) \| 95% \| −1.9% (−3.5%, −0.2%) **less** \|" — https://github.com/lmmx/mibz2/blob/90ecc7d3f446/docs/journal/2026-11-05-compact30-staged/bench-run1-9v74.md
- [2026-11-05T14:01Z] docs/journal/2026-11-05-compact30-staged/bench-run2-7763.md (doc): "\| 2: CPUs 0,2, one a core \| 554 \| 3.57 \| 1.07× (1.06–1.08) \| 92% \| 491 \| 4.03 \| 1.06× (1.05–1.07) \| 93% \| −0.8% (−2.7%, +1.0%) \|" — https://github.com/lmmx/mibz2/blob/90ecc7d3f446/docs/journal/2026-11-05-compact30-staged/bench-run2-7763.md
- [2026-11-05T15:04Z] docs/journal/2026-11-05-isa-probe/bench-42048643495.md (doc): "\| 2: CPUs 0,2, one a core \| 533 \| 3.71 \| 1.09× (1.08–1.10) \| 91% \| 523 \| 3.81 \| 1.10× (1.08–1.11) \| 90% \| +1.0% (−0.3%, +2.3%) \|" — https://github.com/lmmx/mibz2/blob/66a9deddcb41/docs/journal/2026-11-05-isa-probe/bench-42048643495.md
- [2026-11-05T18:18Z] docs/journal/2026-11-05-huffman-register/bench-42057920798-attempt1-9v74.md (doc): "\| 2: CPUs 0,2, one a core \| 646 \| 3.06 \| 1.11× (1.08–1.15) \| 88% \| 646 \| 3.06 \| 1.09× (1.07–1.12) \| 90% \| −1.9% (−5.8%, +2.2%) \|" — https://github.com/lmmx/mibz2/blob/1d0cde371176/docs/journal/2026-11-05-huffman-register/bench-42057920798-attempt1-9v74.md
- [2026-11-05T18:18Z] docs/journal/2026-11-05-huffman-register/bench-42057920798-attempt2-8370c.md (doc): "\| 2: CPUs 0,2, one a core \| 451 \| 4.42 \| 1.04× (1.01–1.06) \| 95% \| 457 \| 4.35 \| 1.04× (1.02–1.06) \| 95% \| +0.4% (−1.6%, +2.4%) \|" — https://github.com/lmmx/mibz2/blob/1d0cde371176/docs/journal/2026-11-05-huffman-register/bench-42057920798-attempt2-8370c.md
- [2026-11-05T18:18Z] docs/journal/2026-11-05-job-handoff/bench-42057901182-attempt1-7763.md (doc): "\| 2: CPUs 0,2, one a core \| 570 \| 3.48 \| 1.08× (1.06–1.09) \| 92% \| 566 \| 3.48 \| 1.08× (1.06–1.10) \| 91% \| +0.3% (−0.6%, +1.1%) \|" — https://github.com/lmmx/mibz2/blob/1d0cde371176/docs/journal/2026-11-05-job-handoff/bench-42057901182-attempt1-7763.md
- [2026-11-05T18:18Z] docs/journal/2026-11-05-job-handoff/bench-42057901182-attempt2-9v45.md (doc): "\| 2: CPUs 0,2, one a core \| 1,026 \| 1.90 \| 1.08× (1.06–1.11) \| 90% \| 1,019 \| 1.93 \| 1.09× (1.07–1.11) \| 90% \| +0.9% (−2.3%, +4.1%) \|" — https://github.com/lmmx/mibz2/blob/1d0cde371176/docs/journal/2026-11-05-job-handoff/bench-42057901182-attempt2-9v45.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #468, bench-lazy-cap, bench-split-051) / Optimiser: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6926545736 (2026-11-16T19:18Z, after commit_to_build)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6906630511

**Notes:** 2 i9 bundles over 2 recipe(s); coders check whether master moved between them.

## cores-less-one: Past the cores, one lazy worker a core but one, window sized to the workers

- **PRs:** [#477](https://github.com/lmmx/mibz2/pull/477), [#481](https://github.com/lmmx/mibz2/pull/481)
- **Journal sources:** none listed
- **i9 recipes:** lazy-place, cores-less-one
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** per PR #481 (Coordinator, Researcher named)
- **commit_to_build:** 2026-11-16T09:43Z, PR #477 first commit (author date) (https://github.com/lmmx/mibz2/pull/477/commits)
- **Cost:** i9 22.2 over 2 bundle(s) min [lazy-place/run-20261116T101100Z: 13.4 min (bundle start (name)→evidence commit); cores-less-one/run-20261116T203759Z: 8.8 min (bundle start (name)→evidence commit)]; CI: 6 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.46; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "CI checks it on a made-up run, including that a run whose outputs differ is refused. ## Why Tonight's bundles show the same ten lazy workers costing 25% more CPU at 20 threads asked than at 10." (https://github.com/lmmx/mibz2/pull/477 (PR body, current text))
- "- **What it biases:** - every absolute 10-thread figure on the i9 (wall, CPU, the memory dose's 1.92–2.02);" (https://github.com/lmmx/mibz2/pull/477 (PR body, current text))
- "Its CPU is about 0.003 s a run. ## Checks - Unit test `workers_past_the_cores`." (https://github.com/lmmx/mibz2/pull/477 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-lazy-place: "Past the cores: window per worker, cores-1, and one CPU a core pinned, at 10/12/20 threads on x8, plus a dose run (#477; rule LOG 6912912161)"
- queue entry bench-cores-less-one: "Nine lazy workers and a worker-sized window past the cores, i9 profile only, against master's build (#481; registration 6923522255 with REVIEW 6924511372's corrections, QUEUED 6924894726)"
- "The recipe runs them on eight copies of the Wikidata sample at 10, 12 and 20 threads, then at 1, 10 and 20 on one copy for each arm's memory dose (bench/lazy_place.py, exercised in CI on a made-up run)." (https://github.com/lmmx/mibz2/pull/477 (first commit message))
- "- **`just bench-lazy-place`:** eight copies of the Wikidata sample at 10, 12 and 20 threads, 10 paired rounds." (https://github.com/lmmx/mibz2/pull/477 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-16T10:34Z] "The window of a workers plan with fewer workers than threads asked is now scaled to the workers, as a teams plan's is: 36 blocks for nine workers at 20 threads, not 80." (https://github.com/lmmx/mibz2/pull/481 (first commit message))
- [2026-11-16T09:44Z] "- `-cores1`: one worker fewer than the cores." (https://github.com/lmmx/mibz2/pull/477 (PR body, current text))
- [2026-11-16T09:44Z] "Its CPU is about 0.003 s a run. ## Checks - Unit test `workers_past_the_cores`." (https://github.com/lmmx/mibz2/pull/477 (PR body, current text))
- [2026-11-16T10:35Z] "- the i9-10900K by vendor, family, model and name;" (https://github.com/lmmx/mibz2/pull/481 (PR body, current text))
- [2026-11-16T10:35Z] "The machine is read only where the plan is workers fewer than the threads asked." (https://github.com/lmmx/mibz2/pull/481 (PR body, current text))
- [2026-11-16T10:35Z] "CI's bench runs on 4 CPUs, off the profile, so it shows nothing. ## Tests - `the_trim_past_the_cores`: the predicate on and off the profile (a same-shape CPU, another name, 9 cores, no SMT, unread topology, a quota, the scalar kernels, 10 threads or fewer)." (https://github.com/lmmx/mibz2/pull/481 (PR body, current text))

**same_quantity_facts:** 3 matching lines in 3 documents dated before commit_to_build, searched on `cores.?1|core but one|one fewer than the (physical )?cores` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-06T19:34Z] comment 6693638662 (#145 optimizer): "**Pick 3×3, not automatic 3×4.** First candidate: detected physical cores=10, L3=20 MiB, requested threads≥10 → three teams of three. Everything else retains the old policy, including 4×2 at requests8/9. Default window becomes12 through the existing formula; custom-window semantics stay intact. No …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6693638662
- [2026-11-16T03:13Z] comment 6906255284 (#145 pathfinder): "5. Beside any of these: if the scanner's CPU seconds on the eight copies rise 15% or more from 8 to 10 threads, the scanner is sharing a core. The next arm is then the cap at one fewer than the cores, a core kept for the scanner. That is a one-line arm." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6906255284
- [2026-11-16T03:39Z] comment 6906620039 (#145 coordinator): "**Occupancy, cap arm at 20 threads, x8 file:** workers 91–92% busy, the scanner 61–69%. The scanner's seconds rise from 0.477 at 8 threads to 0.546 at 10 (+14%), just under the 15% trigger for a cores−1 arm, so no cores−1 arm. The fixed cost is about 10–15% of one copy." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6906620039

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6926375141 (2026-11-16T19:12Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #477, #481, bench-lazy-place, bench-cores-less-one)

**monodromy_involved:** yes: records naming its PRs or recipes: 2026-11-16-where-gains-are.md, 2026-11-16-workers-counters.md, 2026-11-16-workers-prediction.md, 2026-11-17-workers-model.md

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 2 i9 bundles over 2 recipe(s); coders check whether master moved between them.

## stop-and-walk: Stop-and-walk: stop the induction on the cell budget, walk the stopped exchange

- **PRs:** [#493](https://github.com/lmmx/mibz2/pull/493), [#501](https://github.com/lmmx/mibz2/pull/501)
- **Journal sources:** docs/journal/monodromy/2026-11-16-stop-and-walk.md, docs/journal/monodromy/2026-11-16-stop-walk-reopened.md, docs/journal/2026-11-16-stop-walk-adoption.md
- **i9 recipes:** stop-walk, stop-walk-counters
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** MONODROMY (records 19, 21); built by the Pathfinder
- **commit_to_build:** 2026-11-16T14:57Z, PR #493 opened (https://github.com/lmmx/mibz2/pull/493)
- **Cost:** i9 21.7 over 2 bundle(s) min [stop-walk/run-20261116T162328Z: 20.5 min (bundle start (name)→evidence commit); stop-walk-counters/run-20261116T181905Z: 1.2 min (bundle start (name)→evidence commit)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.07; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- *A finding:* any change to the counter's own cold path costs 16 to 38 instructions on **every** cell, through register allocation." (https://github.com/lmmx/mibz2/pull/493 (PR body, current text))
- "It then applies Holm's one-sided gain tests over the five Wikidata cells (family α 0.05), the loss screen on all ten cells, and the median-of-peaks memory gate (1.10)." (https://github.com/lmmx/mibz2/pull/493 (PR body, current text))
- "It is byte-exact and −1.5 instructions a letter." (https://github.com/lmmx/mibz2/pull/501 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-stop-walk: "Stop-and-walk (beta 1.5) against shipped lazy on production's plan, 8/9/10/12/20 threads, wikidata x4 + text guard (#493; rule REVIEW 6921622210, PROCEED LOG 6922045659)"
- queue entry bench-stop-walk-counters: "Stop-and-walk hardware counters (cycles, cache, branches, stalls) against lazy; rule LOG 6924413955)"
- "MONODROMY's gate was +20 at n/32, so the arm goes no further." (https://github.com/lmmx/mibz2/pull/493 (first commit message))
- "MONODROMY's gate was +20 at n/32, so the arm goes no further." (https://github.com/lmmx/mibz2/pull/501 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 142 matching lines in 103 documents dated before commit_to_build, searched on `stop.and.walk|stopped exchange|budget|per letter` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-15T15:36Z] comment 6896968715 (#145 louis via chatgpt): "That 30% saving is the straightforward consequence of going from 40 to 28 bytes per letter." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6896968715
- [2026-11-15T20:48Z] comment 6901130348 (#145 pathfinder): "Cells per letter are 1.85 (1.70 to 1.97); rules per letter are 0.57." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6901130348
- [2026-11-15T21:58Z] docs/journal/2026-11-15-lazy-letters.md (doc): "I priced the induction at +25 instructions a letter. It costs +81. Per letter, from the" — https://github.com/lmmx/mibz2/blob/9d0fb49cee95/docs/journal/2026-11-15-lazy-letters.md
- [2026-11-15T22:42Z] comment 6902593379 (#145 pathfinder): "**Why two registers and a compare cost 48 instructions a letter.** The induction's loop is at its register limit: it holds about 19 live values. Three more push others to the stack, and the reloads land on lines that have nothing to do with the check (the budget test alone goes from about 4 to 16 i…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6902593379
- [2026-11-15T23:59Z] comment 6903554609 (#145 pathfinder): "- **Instructions** (per letter, callgrind, one thread): first stage −8%, induction +2.8%, writer +11.5%, the three +1.1%, whole decoder +0.6%. The two stage overages are the ones you accepted." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903554609
- [2026-11-16T00:07Z] comment 6903669583 (#145 louis via chatgpt): "Interestingly, the induction alone rises from roughly 115.5 to 181 instructions per letter. The first stage gets cheaper, offsetting some of the induction's increase." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903669583
- [2026-11-16T01:00Z] comment 6904409903 (#145 pathfinder): "1. **Retained capacity.** The kernel's buffer, one a thread, fell by 9.2 B a letter: about 0.5 MB a block. The tail's reservation is for the worst case (a rule for every cell the budget allows, about 2.6 B a row, besides the bottom slots), and it lands in every output buffer in flight: the ledger h…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904409903
- [2026-11-16T01:17Z] comment 6904651994 (#145 coordinator): "**The rule (C1a), posted before any data:** a version is built as a forced arm (`mibz2-workers-lazy2`) if the lazy induction's instructions per letter fall by at least 10% on the Wikidata blocks, at equal or fewer bytes per letter, with identical output. It is judged by the clock at the next i9 run…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904651994
- [2026-11-16T12:32Z] comment 6916500604 (#145 pathfinder): "RE: HANDOFF 6915365756 (stop and walk): built, byte-exact at n/16, n/32 and n/64, and over the gate. +97 instructions a letter at n/32 against +20, so it goes no further. Where the model and the kernel differ" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6916500604
- [2026-11-16T12:47Z] comment 6916839849 (#145 monodromy): "RE: Stop-and-walk, reopened on both counts named in LOG 6916535874. The stop rides on the cell budget the kernel already counts, so it costs 0 a cell. The writer is one lane: each live word is expanded once, then copied. Prototype: 33% fewer word bytes and about 20 fewer instructions a letter than …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6916839849

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #493, #501, bench-stop-walk, bench-stop-walk-counters) / Optimiser: unrecorded (no comment from the role naming #493, #501, bench-stop-walk, bench-stop-walk-counters)

**monodromy_involved:** yes: cited in the case's own documents (records 19, 21); origin field names MONODROMY

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/162#issuecomment-6920377932

**Notes:** 2 i9 bundles over 2 recipe(s); coders check whether master moved between them. PR #493's earliest listed commit is 2026-11-16T12:30Z, 2 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #501's earliest listed commit is 2026-11-16T12:30Z, 5 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## w12-rules: 12-byte rules (W12LazyCells)

- **PRs:** [#506](https://github.com/lmmx/mibz2/pull/506), [#517](https://github.com/lmmx/mibz2/pull/517), [#519](https://github.com/lmmx/mibz2/pull/519)
- **Journal sources:** none listed
- **i9 recipes:** w12, w12-stores, w12-at-7
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** per PR #506 (Coordinator named); MONODROMY records 37-39 (rule widths, store census)
- **commit_to_build:** 2026-11-16T19:09Z, PR #506 opened (https://github.com/lmmx/mibz2/pull/506)
- **Cost:** i9 19.8 over 3 bundle(s) min [w12/run-20261116T222857Z: 8.8 min (bundle start (name)→evidence commit); w12-stores/20261117T073447Z: 1.1 min (bundle start (name)→evidence commit); w12-at-7/run-20261117T094130Z: 9.9 min (bundle start (name)→evidence commit)]; CI: 6 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.61; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "perf(cells): the lazy induction's 16-byte rules' instance is master's code again #506 made `induce_lazy` const-generic over `R12`, and its `<false>` instance, the 16-byte rules' kernel, compiled to different code from master's non-generic function: 57 stack stores in place of 48, and per letter of …" (https://github.com/lmmx/mibz2/pull/517 (first commit message))
- "Its kernel time at one thread, interleaved in one binary with master's code, was 1.006 to 1.016 of master's (8 runs)." (https://github.com/lmmx/mibz2/pull/517 (first commit message))
- "- `induce_lazy`, monomorphic with `const R12: bool = false`: instruction for instruction master's `induce_lazy` (552 instructions, none differing), with its counts;" (https://github.com/lmmx/mibz2/pull/517 (first commit message))

**must_hold** (quoted):

- queue entry bench-w12: "12-byte rules (W12) against master's build, rustc 1.99.1, with the L2 diagnostic at 10 threads (#506; registration 6928288705, run ahead of the Researcher's review at Louis's instruction)"
- queue entry bench-w12-stores: "W12 against lazy at 10 threads: store write-backs and store-buffer stalls (#512; rule in #160 comment 6931580102, posted before the data)"
- queue entry bench-w12-at-7: "W12 against W16 at 7/28 past the cores, 9/36 as context (#519; combines #506 via #517 and #514)"
- "perf(cells): a 12-byte rules arm, the lazy induction storing each rule's two children packed in 48 bits each The second ChatGPT relay's comparator to 8-byte rules, ruled to its induction side by the Coordinator (#145, on LOG 6925645704)." (https://github.com/lmmx/mibz2/pull/506 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-17T08:50Z] "With fewer workers the L3 is under less contention, so a footprint gain may shrink." (https://github.com/lmmx/mibz2/pull/519 (PR body, current text))
- [2026-11-17T08:50Z] "- **Corpora:** Wikidata ×4 (the gain family) and text (the loss and memory guard)." (https://github.com/lmmx/mibz2/pull/519 (PR body, current text))
- [2026-11-17T08:50Z] "The diagnostic was skipped because `perf_event_paranoid` is 2." (https://github.com/lmmx/mibz2/pull/519 (PR body, current text))

**same_quantity_facts:** 236 matching lines in 128 documents dated before commit_to_build, searched on `12.byte|W12|rule width|footprint|L2` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-16T00:07Z] comment 6903669583 (#145 louis via chatgpt): "One technical nuance: the code reserves enough pool capacity for every letter, but only hands out and touches the high-water portion. The 0.318 figure describes the measured slot usage and working-set footprint, not an absolute bound on allocated virtual memory." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6903669583
- [2026-11-16T01:05Z] comment 6904481722 (#145 pathfinder): "\| **Narrower rules** (12 bytes, not 16). \| about −1.0 \| −3% \| Checked against the writer, and not worth it. A rule holds its two factors' lengths because the writer's bottom-up pass reads a factor's length from the rule that refers to it. Dropping one length means each word must carry its own: …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6904481722
- [2026-11-16T09:21Z] comment 6912421307 (#222 coordinator): "- a smaller slot footprint: a desk analysis found no cheap 15% cut, and the remaining cost is instructions (LOG 6907104251)." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6912421307
- [2026-11-16T16:32Z] comment 6922450227 (#145 monodromy): "**Not recommended (M):** a per-point successor table. Its 8 B × m sits in L2; the rank structure sits in L1. Either way it is about 14 cycles a step." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6922450227
- [2026-11-16T18:21Z] comment 6925144719 (#145 coordinator): "- **At one thread,** the walk's extra cost is memory-bound stall cycles: +0.39 a byte, about 39% of the +1.0 cycles a byte. L1 misses and LLC misses are flat. So the stalled loads are hitting in L2 or L3, or waiting behind stores. It isn't DRAM, and it isn't branches: misses fall 10%." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6925144719
- [2026-11-16T18:42Z] comment 6925645704 (#145 pathfinder): "- **W8** runs fewer instructions than W12 but is 17% slower. Its child lengths are dependent loads from the length table, as the relay feared. At about 55 ms of a ~370 ms kernel, that is about +2.5% of kernel time, before any cost on the induction's side." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6925645704
- [2026-11-16T18:46Z] docs/journal/monodromy/2026-11-16-compact-tail-divergence.md (doc): "\| **12-byte rules everywhere (W12)** \| **2.29** \| **parity (1.01 to 1.03)** \| **one loop, rules stored as 12 B: not yet counted** \|" — https://github.com/lmmx/mibz2/blob/d53b7147bb57/docs/journal/monodromy/2026-11-16-compact-tail-divergence.md
- [2026-11-16T18:50Z] comment 6925826715 (#145 pathfinder): "W12's induction side, now, as ruled. The gate is counted on `ab` arms, `mibz2-workers-lazy` against a W12 arm, both built with 1.99.1, and the same two figures under 1.98.0. The recipe template gets `rustc -V` and `cargo -V` in my next recipe PR." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6925826715
- [2026-11-16T19:03Z] docs/journal/monodromy/2026-11-16-where-gains-are.md (doc): "\| 4 \| **8-byte rules (W8)**: 4.57 B a letter, at +17% writer time (about +2.5% of the kernel). \| **Negative at 10 threads** unless L2 relief is real and large. Only worth testing if rank 1's L2 counters show relief scaling with bytes. \| The same L2 counters. \|" — https://github.com/lmmx/mibz2/blob/454e868d198a/docs/journal/monodromy/2026-11-16-where-gains-are.md
- [2026-11-16T19:03Z] comment 6926156107 (#145 monodromy): "\| 4 \| W8: −4.57 B at +17% writer time \| negative at 10 threads, unless L2 relief is large \| the same L2 counters \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6926156107

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6943371729 (2026-11-17T08:54Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #506, #517, #519, bench-w12, bench-w12-stores, bench-w12-at-7)

**monodromy_involved:** yes: cited in the case's own documents (records 45); records naming its PRs or recipes: 2026-11-16-store-census.md, 2026-11-17-contention-counters.md, 2026-11-17-workers-model.md; origin field names MONODROMY

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 3 i9 bundles over 3 recipe(s); coders check whether master moved between them. PR #506's earliest listed commit is 2026-11-16T19:09Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #519's earliest listed commit is 2026-11-16T19:09Z, 14 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## worker-trim-seven: Seven lazy workers from 10 threads asked, window sized to them (worker-count rule)

- **PRs:** [#510](https://github.com/lmmx/mibz2/pull/510), [#514](https://github.com/lmmx/mibz2/pull/514), [#515](https://github.com/lmmx/mibz2/pull/515), [#520](https://github.com/lmmx/mibz2/pull/520), [#524](https://github.com/lmmx/mibz2/pull/524)
- **Journal sources:** none listed
- **i9 recipes:** workers-window, workers-low, workers-at-cores, workers-confirm
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** MONODROMY records 41-43 (worker-count models); Coordinator
- **commit_to_build:** 2026-11-16T21:05Z, PR #510 opened (https://github.com/lmmx/mibz2/pull/510)
- **Cost:** i9 43.4 over 4 bundle(s) min [workers-window/run-20261116T223800Z: 9.8 min (bundle start (name)→evidence commit); workers-low/run-20261117T073612Z: 10.0 min (bundle start (name)→evidence commit); workers-at-cores/run-20261117T083701Z: 8.5 min (bundle start (name)→evidence commit); workers-confirm/run-20261117T092105Z: 15.1 min (bundle start (name)→evidence commit)]; CI: 8 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.52; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Any event that `perf list` doesn't name is dropped, and the bundle's `events.txt` says so. ## Prediction and adoption - **MONODROMY's record 43** (PR #497, `docs/journal/monodromy/2026-11-17-workers-model.md`) predicts W\* = 7 at 10 threads, with wall(7)/wall(10) = 0.93 [0.90, 0.98]." (https://github.com/lmmx/mibz2/pull/515 (PR body, current text))
- "GenuineIntel 6/165, its model name, 10 cores, 20 CPUs, a 20 MiB L3, AVX2 and CLMUL, no cpu.max limit), the default decoder at 10 threads asked or more runs 7 lazy workers with 28 blocks in flight (4 a worker), in place of 9/36 past the cores and 10/40 at exactly 10." (https://github.com/lmmx/mibz2/pull/524 (first commit message))
- "That profile is GenuineIntel family 6 model 165, `Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`, 10 physical cores, 20 CPUs in the affinity, a 20 MiB L3, AVX2 and CLMUL kernels, and no cgroup cpu.max limit." (https://github.com/lmmx/mibz2/pull/524 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-workers-window: "Eight vs nine workers and the window at 2x and 1/2 past the cores (#510; registration 6930443845, run ahead of review at Louis's instruction)"
- queue entry bench-workers-low: "8 / 7 / 6 workers at 4 a worker past the cores, 8-32 the reference (stacked on #510; MONODROMY records 41-42 frozen before the data)"
- queue entry bench-workers-at-cores: "7/8/9 workers at 10 threads and 6/7 at 8, against production, within the cores (#515, stacked on #514; MONODROMY record 43 frozen before the data)"
- queue entry bench-workers-confirm: "7/28 past the cores and at 10, 6/24 at 8, against production, with held-out records and text at -1 (#520, stacked on #515)"

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-17T09:18Z] "GenuineIntel 6/165, its model name, 10 cores, 20 CPUs, a 20 MiB L3, AVX2 and CLMUL, no cpu.max limit), the default decoder at 10 threads asked or more runs 7 lazy workers with 28 blocks in flight (4 a worker), in place of 9/36 past the cores and 10/40 at exactly 10." (https://github.com/lmmx/mibz2/pull/524 (first commit message))
- [2026-11-16T23:29Z] "The prediction is scored after the run and is not used by the rule." (https://github.com/lmmx/mibz2/pull/514 (PR body, current text))
- [2026-11-16T23:29Z] "So `mibz2-occ-7-28` built as #510 built its arms would change the kernel too: a slot for every letter, as well as fewer workers." (https://github.com/lmmx/mibz2/pull/514 (PR body, current text))
- [2026-11-16T23:29Z] "- **What an adoption of 7 or 6 would mean:** production's plan past the cores keeps lazy letters below 8 workers." (https://github.com/lmmx/mibz2/pull/514 (PR body, current text))
- [2026-11-16T23:29Z] "The diagnostic was skipped because `perf_event_paranoid` is 2 here." (https://github.com/lmmx/mibz2/pull/514 (PR body, current text))
- [2026-11-17T08:03Z] "- `cycles`, `ref-cycles`, `instructions`, `LLC-load-misses` and `l2_rqsts.miss`, in two groups so that nothing is multiplexed;" (https://github.com/lmmx/mibz2/pull/515 (PR body, current text))

**same_quantity_facts:** 7 matching lines in 7 documents dated before commit_to_build, searched on `workers? (count|trim)|7/28|8/32|9/36|10/40|capacity cliff|lazy workers at 10` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-29T08:52Z] docs/journal/2026-10-29-expert-review.md (doc): "\| E2 \| **libsais against production, by worker count** (queued: `p4…p10, s4x1…s10x1`, 16 rounds; `p4,4x1,3x3,s4x1,s3x3`, 20 rounds) \| the same, as libsais \| none (PR #73 at c2620b8) \| ~10 min \| none \| — \| **informs E1:** sNx1 / pN rising with N says ψ²'s gain is memory pressure. Flat at ~1.…" — https://github.com/lmmx/mibz2/blob/155bb99b04c1/docs/journal/2026-10-29-expert-review.md
- [2026-10-29T16:45Z] docs/journal/2026-10-29-frontier.md (doc): "**What series 2 already shows, with its limits stated.** In throughput against live table bytes, y3x3 (1657 MB/s from 10.8 MB of live tables) beats p4 (1206 MB/s from 14.4 MB). p4 is production's best worker count on the i9 in every scaling run. So at the inversion's level, sharing a table among a …" — https://github.com/lmmx/mibz2/blob/ae7bc3099933/docs/journal/2026-10-29-frontier.md
- [2026-11-14T22:03Z] comment 6884895257 (#145 coordinator): "- **The empty-scope overhead** is measured, as well as the 1% phase-sum check against whole-worker counts." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884895257
- [2026-11-16T02:21Z] comment 6905531834 (#145 pathfinder): "From the two i9 runs you cite: the shipped plan is flat past 10 threads (0.122 s at 10; 0.127 to 0.128 at 12 to 20), lazy workers at 10 beat it there (0.931), and lazy workers one a thread lose on the cores' second threads. So I expect the capped arm near 0.93 of the shipped plan at every count fro…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6905531834
- [2026-11-16T02:39Z] comment 6905778670 (#145 coordinator): "**Lazy workers at 10 threads, against the shipped decoder (the service core on the i9):** 0.917 [0.898, 0.936]. That makes three runs agreeing:" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6905778670
- [2026-11-16T15:08Z] comment 6920267580 (#145 coordinator): "- **10 threads:** #462 0.931 [0.908, 0.954] is attributed (lazy workers at 10 replace teams). #468 0.996 is noise." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6920267580
- [2026-11-16T15:19Z] docs/journal/release-attribution.md (doc): "\| 0.5.0 → 0.5.1 \| 10t +7.3% (+6.2, +8.5); 12–20t +6.7 to +7.7% \| 10t: #462, lazy workers at 10 threads instead of teams; 12–20t: #468, past the cores, lazy workers one a core \| routing; memory \| `bench-split-051` (split-051/run-20261116T145423Z): wall 10t #462 0.931 [0.908, 0.954], #468 0.996 …" — https://github.com/lmmx/mibz2/blob/6b161855020c/docs/journal/release-attribution.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6943617731 (2026-11-17T09:06Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #510, #514, #515, #520, #524, bench-workers-window, bench-workers-low, bench-workers-at-cores, bench-workers-confirm)

**monodromy_involved:** yes: cited in the case's own documents (records 41, 42, 43); records naming its PRs or recipes: 2026-11-16-workers-counters.md, 2026-11-16-workers-prediction.md, 2026-11-17-worker-rule.md, 2026-11-17-workers-machine-model.md, 2026-11-17-workers-model.md, 2026-11-18-knee-dram.md, 2026-11-18-recalibration-scoring.md; origin field names MONODROMY

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 4 i9 bundles over 4 recipe(s); coders check whether master moved between them. PR #510's earliest listed commit is 2026-11-16T21:03Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #514's earliest listed commit is 2026-11-16T21:03Z, 2 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #515's earliest listed commit is 2026-11-16T21:03Z, 11 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #520's earliest listed commit is 2026-11-16T21:03Z, 12 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## worker-trim-low: Six lazy workers at 8 threads, five at 6

- **PRs:** [#521](https://github.com/lmmx/mibz2/pull/521), [#526](https://github.com/lmmx/mibz2/pull/526), [#533](https://github.com/lmmx/mibz2/pull/533), [#537](https://github.com/lmmx/mibz2/pull/537)
- **Journal sources:** none listed
- **i9 recipes:** workers-sweep-low, workers-level3, eager-trim
- **Deciding machine:** i9-10900K (pinned)
- **Origin (as recorded):** MONODROMY records 44-46, 49; Coordinator
- **commit_to_build:** 2026-11-17T08:59Z, PR #521 opened (https://github.com/lmmx/mibz2/pull/521)
- **Cost:** i9 19.8 over 3 bundle(s) min [workers-sweep-low/run-20261117T102836Z: 7.8 min (bundle start (name)→evidence commit); workers-level3/run-20261117T101655Z: 6.3 min (bundle start (name)→evidence commit); eager-trim/run-20261117T114640Z: 5.7 min (bundle start (name)→evidence commit)]; CI: 12 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.07; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **Predictions (scored, not used by the rule):** record 45's homogeneity prediction (W\* = 9 or 10, 7/28 no better than 0.98) against the bandwidth rival (about 0.885)." (https://github.com/lmmx/mibz2/pull/526 (PR body, current text))
- "Measured worker buffers were 5.95 MB at -3 against 16.58 MB at -9 (VM, unpinned; a capacity count, not evidence)." (https://github.com/lmmx/mibz2/pull/526 (PR body, current text))
- "- `whole_machine(m) && unpinned_trims(threads)`, where `unpinned_trims` is `threads >= 10`, measured unpinned at 10 threads by `bench-unpinned` and at 12–20 by #524. ## Evidence - **10 pinned:** #515, LOG 6943194948, and `bench-unpinned`'s pinned rows: 0.881–0.889 of lazy 10/40." (https://github.com/lmmx/mibz2/pull/537 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-workers-sweep-low: "lazy workers at 8 (6, 5), 6 (6 control, 5, 4) and 4 (4 control, 3, 2) threads, with per-thread CPU (#521; MONODROMY records 44 and 46, and LOG 6943635048, frozen before the data)"
- queue entry bench-workers-level3: "W = 10/9/8/7/6/4 lazy at 10 threads on Wikidata recompressed at -3 (#526): the transfer test of MONODROMY record 45"
- queue entry bench-eager-trim: "eager against lazy at the trimmed counts: 6, 4 and 10 threads (#533; MONODROMY record 49 frozen before the data; docs/journal/2026-11-17-eager-trim.md)"
- "**What it's for:** the data for a production rule, workers = f(threads, cores), before any such rule is written." (https://github.com/lmmx/mibz2/pull/521 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-17T08:59Z] "It cites #514, #515 (LOG 6943194948), MONODROMY record 43's 6-core prediction, and record 44 (LOG 6943291920) as the frozen prediction it scores." (https://github.com/lmmx/mibz2/pull/521 (PR body, current text))
- [2026-11-17T08:59Z] "Running every arm at every count would take about 33 minutes; this takes about 17." (https://github.com/lmmx/mibz2/pull/521 (PR body, current text))
- [2026-11-17T08:59Z] "- **Estimated i9 time: about 26 minutes** (judged runs about 17, diagnostic about 9)." (https://github.com/lmmx/mibz2/pull/521 (PR body, current text))
- [2026-11-17T08:59Z] "At 10 and 10 it would be about 37." (https://github.com/lmmx/mibz2/pull/521 (PR body, current text))
- [2026-11-17T08:59Z] "If more time needs cutting, REPS=5 saves about 1.5 minutes. ## Tests - `the_occupancy_arms_low` (src/parallel.rs) checks:" (https://github.com/lmmx/mibz2/pull/521 (PR body, current text))
- [2026-11-17T08:59Z] "- The thread split read every record: the expected worker thread count for each arm, at every count." (https://github.com/lmmx/mibz2/pull/521 (PR body, current text))

**same_quantity_facts:** 14 matching lines in 11 documents dated before commit_to_build, searched on `workers? (count|trim)|6/24|5/20|4/16|lazy workers at (8|6)|occ-[3-6]|oce-[3-6]` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-29T16:45Z] docs/journal/2026-10-29-frontier.md (doc): "**What series 2 already shows, with its limits stated.** In throughput against live table bytes, y3x3 (1657 MB/s from 10.8 MB of live tables) beats p4 (1206 MB/s from 14.4 MB). p4 is production's best worker count on the i9 in every scaling run. So at the inversion's level, sharing a table among a …" — https://github.com/lmmx/mibz2/blob/ae7bc3099933/docs/journal/2026-10-29-frontier.md
- [2026-10-29T18:03Z] docs/journal/2026-10-29-cooperative-decoder/coop-decoder-20261029T174657Z.md (doc): "\| mibz2-coop3 \| 8 \| 988 \| 1.004 (0.974-1.035) \| 5/20 \| 48 MB \| 0.66 \| 0.82 \| 0.97 \|" — https://github.com/lmmx/mibz2/blob/d6d4cac5ad49/docs/journal/2026-10-29-cooperative-decoder/coop-decoder-20261029T174657Z.md
- [2026-11-06T09:53Z] comment 6682105521 (#162 coordinator): "- It is `examples/occupancy.rs` (trace feature) at 4 × 2/8, 4 × 3/12 and 4 × 4/16, at the default window, 6 rounds." — https://github.com/lmmx/mibz2/pull/162#issuecomment-6682105521
- [2026-11-08T20:09Z] comment 6726489772 (#145 louis via chatgpt): "- **One chain** with the same move structure: **59–63% faster**, because a short, L2-friendly table reduces serial pointer latency. But the **32-lane** T-vector overlaps many misses, so it is largely instruction/issue constrained: ~**16** instructions a byte, against move-select ~**41**. The move l…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6726489772
- [2026-11-10T06:05Z] comment 6760366316 (#145 researcher): "hetero.py randomly picks a pilot residual block independently for each batch and fixes g=0. It estimates a mixture-average size, not the required worst case over fixed path/variance arrangements. Its maximum1115/20000=5.575% has a one-sided95% upper MC limit5.849% (simultaneous42-cell upper6.085%).…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6760366316
- [2026-11-12T16:42Z] docs/journal/2026-11-12-rauzy-companion/FACTS.md (doc): "\| §9 stage ratio0.82–0.86 vs whole1.000; 431 blocks,196 admitted,84.117% rows \| [PR367 handoff](https://github.com/lmmx/mibz2/blob/a08277e1c36e276cfea7b3116d99ea5fd93f0e84/docs/journal/monodromy/2026-11-10-cells-gap/HANDOFF.md), mathematical journal §6.3 \| Research shared-container inversion and…" — https://github.com/lmmx/mibz2/blob/5d8f11c7d4b8/docs/journal/2026-11-12-rauzy-companion/FACTS.md
- [2026-11-14T22:03Z] comment 6884895257 (#145 coordinator): "- **The empty-scope overhead** is measured, as well as the 1% phase-sum check against whole-worker counts." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884895257
- [2026-11-16T19:12Z] comment 6926375141 (#145 researcher): "**No tested uncapping gain over the later capped production baseline.** The later lazy-crossover run has uncapped/production ratios **1.089, 1.160, 1.194, 1.265** at requested12/14/16/20, each interval wholly above1. Stop-walk's forced20 stop15/production ratio reproduces **1.2603**; its 0.965 gain…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6926375141
- [2026-11-17T08:46Z] comment 6943194948 (#145 coordinator): "- Cycles over ref-cycles is **1.326 at every count**, so the clock does not move with the worker count. Turbo is not the residual." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6943194948
- [2026-11-17T08:56Z] comment 6943398069 (#145 coordinator): "- The Researcher's limits apply: the text guard is a screen, not equivalence; RSS ≤ 1.10 proves no saving; and the admission does not certify W12 under the new worker counts." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6943398069

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6947387421 (2026-11-17T11:58Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #521, #526, #533, #537, bench-workers-sweep-low, bench-workers-level3, bench-eager-trim)

**monodromy_involved:** yes: cited in the case's own documents (records 43, 44, 45, 46, 49); origin field names MONODROMY

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 3 i9 bundles over 3 recipe(s); coders check whether master moved between them. PR #521's earliest listed commit is 2026-11-16T21:03Z, 12 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #526's earliest listed commit is 2026-11-16T21:03Z, 12 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #533's earliest listed commit is 2026-11-16T21:03Z, 14 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## read-machine-once: Read the machine once a process

- **PRs:** [#545](https://github.com/lmmx/mibz2/pull/545)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** per PR #545 (no proposer named)
- **commit_to_build:** 2026-11-17T14:20Z, PR #545 first commit (author date) (https://github.com/lmmx/mibz2/pull/545/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- runs: +5.8% (+2.0, +9.7) wall - records: +3.9% - pattern: +3.8% - onebyte: +2.0% CPU That is consistent with about 0.5 to 1 ms of sysfs reads per decoder. ## What changed (`src/parallel.rs`) - **Cached machine-wide facts.** A static `MachineCache` (`FACTS`) holds three `OnceLock`s, each read on f…" (https://github.com/lmmx/mibz2/pull/545 (PR body, current text))

**must_hold** (quoted):

- "perf(plan): read the machine once a process #537 made every default decoder at 4, 6, 8 and 10 threads read the whole machine (Machine::here: sysfs online/present, per-CPU topology and cache files, /proc/cpuinfo twice, /proc/self/status and the cgroup quota)." (https://github.com/lmmx/mibz2/pull/545 (first commit message))
- "Cache the machine-wide facts (L3, cpuinfo's i9 count, the present and online topology) once a process in OnceLocks, and check the cheap, cached conditions first: off the measured i9 (or without the AVX2 and CLMUL kernels) neither the affinity nor the quota is read." (https://github.com/lmmx/mibz2/pull/545 (first commit message))
- "On the i9 both are still read on every call, as they can change at runtime. with_machine's override still takes precedence." (https://github.com/lmmx/mibz2/pull/545 (first commit message))
- "- runs: +5.8% (+2.0, +9.7) wall - records: +3.9% - pattern: +3.8% - onebyte: +2.0% CPU That is consistent with about 0.5 to 1 ms of sysfs reads per decoder. ## What changed (`src/parallel.rs`) - **Cached machine-wide facts.** A static `MachineCache` (`FACTS`) holds three `OnceLock`s, each read on f…" (https://github.com/lmmx/mibz2/pull/545 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-17T14:20Z] "The affinity (`phys`, `target`) and the quota are still read on each call, because both can change at runtime." (https://github.com/lmmx/mibz2/pull/545 (PR body, current text))

**same_quantity_facts:** 146 matching lines in 109 documents dated before commit_to_build, searched on `machine|topology|once a process|fixed cost` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-16T17:54Z] comment 6924511372 (#145 researcher): "**Controls:** keep the predeclared VOID guard, preserve all records, and do not calibrate by dividing by a control ratio. Calibration assumes a nuisance factor transfers across counts and spinner states; the design does not establish that. A moved control is an alarm, not proof that “the harness mo…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6924511372
- [2026-11-16T20:47Z] comment 6928868489 (#145 coordinator): "**The bundle checks out:** head 53b0b225 against master's build at 4ffe2bcb, both built with rustc 1.99.1 (recorded). The Pathfinder confirmed that 4ffe2bcb and current master differ in no production path (machine-code identical kernels). Every arm produced the same bytes." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6928868489
- [2026-11-17T07:51Z] docs/journal/monodromy/2026-11-17-workers-model.md (doc): "0.480 + 0.00808 W², with W scaled by L3 size on another machine, the quadratic form is wrong." — https://github.com/lmmx/mibz2/blob/64bb132b6113/docs/journal/monodromy/2026-11-17-workers-model.md
- [2026-11-17T07:51Z] comment 6942094203 (#145 monodromy): "**Cycles a byte are quadratic in W (M on C).** c(W) = 0.480 + 0.00808 W², relative to 8 workers, fits the four counted points within 0.6%. That is a fixed cost plus contention growing like W²: a miss rate rising with sharers, times a miss cost rising with load (the second factor is Conj)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6942094203
- [2026-11-17T08:22Z] comment 6942709514 (#145 coordinator): "Under MONODROMY's fit c(W) = 0.480 + 0.00808·W², the move structure's doubled fixed cost needs a footprint below about 0.45–0.67× of Cells-lazy's to pay. The whole move worker misses that. Its one random row a byte would also add L3 traffic, not cut it." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6942709514
- [2026-11-17T09:09Z] docs/journal/monodromy/2026-11-17-workers-machine-model.md (doc): "**W_L3(machine, s) ≈ 7.9 · (L / 20 MiB) · (F(900 k) / F(s))**." — https://github.com/lmmx/mibz2/blob/64bd21f6c32d/docs/journal/monodromy/2026-11-17-workers-machine-model.md
- [2026-11-17T09:21Z] docs/journal/2026-11-17-worker-count-calibration.md (doc): "Today's plan hard-codes the measured i9: a profile check (10 physical cores, a 20 MiB L3) gates the trim past the cores. The aim, following Louis on 17 November, is a plan whose worker count comes from a model, with the model's inputs read from the machine at runtime. That keeps today's gains, and …" — https://github.com/lmmx/mibz2/blob/d17466bcbf71/docs/journal/2026-11-17-worker-count-calibration.md
- [2026-11-17T10:04Z] docs/journal/monodromy/2026-11-17-contention-counters.md (doc): "1. DRAM traffic at W = 10 is under 40% of the peak (the machine's own DIMM speed, from" — https://github.com/lmmx/mibz2/blob/0e7c6888bcba/docs/journal/monodromy/2026-11-17-contention-counters.md
- [2026-11-17T12:44Z] comment 6948385717 (#145 coordinator): "- **H3, fixed costs: INCONCLUSIVE.** They are about 12% of the ×1 wall (0.013 s: start and exit 0.014 s, head and tail 4 ms each). The plateau moves only 0.002 between ×1 and ×4." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6948385717
- [2026-11-17T14:00Z] docs/journal/release-attribution.md (doc): "\| 0.2.5 → 0.2.6 \| 10t +6.8% (+5.5, +8.1); 12–20t +4.4 to +7.1% \| #250, the three teams pipelined \| topology \| the only decoder merge \|" — https://github.com/lmmx/mibz2/blob/39155e70ebad/docs/journal/release-attribution.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** knee/run-20261117T123939Z (3.4 min; diagnostic of the 8–20 thread plateau: DRAM bytes against a measured ceiling, worker idle by cause, fixed costs (#541; …); telemetry-check/20261109T125938Z (0.4 min; the machine-state recorder's own untimed check on the target PMU: failure paths, multiplexing, user-only, inheritance (…)

**Review routing:** Researcher: unrecorded (no comment from the role naming #545) / Optimiser: unrecorded (no comment from the role naming #545)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## fused-finish: Fused post-walk on Cells: RLE1 with the CRC folded in

- **PRs:** [#535](https://github.com/lmmx/mibz2/pull/535)
- **Journal sources:** none listed
- **i9 recipes:** fused
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** per PR #535 (no proposer named); adoption registration 2026-11-17-fused-finish-registration.md
- **commit_to_build:** 2026-11-17T11:10Z, PR #535 first commit (author date) (https://github.com/lmmx/mibz2/pull/535/commits)
- **Cost:** i9 5.8 over 1 bundle(s) min [fused/run-20261117T115747Z: 5.8 min (bundle start (name)→evidence commit)]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Master undoes RLE1 into the output, then reads the whole output back for the CRC: about 0.5 MB a Wikidata block, out of a 256 KiB L2 by then, so from the L3." (https://github.com/lmmx/mibz2/pull/535 (first commit message))
- "Counts, `corpora/wikidata.bz2` (cachegrind with the i9's 32 KiB L1d and 256 KiB L2 as the last level, one worker), per output byte: the CRC's L2 read misses 0.0157 to 0.0003; the process's 0.3077 to 0.2920 (-5.1%); instructions +0.09 (+0.33%)." (https://github.com/lmmx/mibz2/pull/535 (first commit message))
- "That is about 0.5 MB a Wikidata block, past the i9's 256 KiB L2 by then." (https://github.com/lmmx/mibz2/pull/535 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-fused: "the CRC folded into RLE1 (#535): 10 threads (both arms 10/40, pinned) and 12 threads (7/28); adoption registration docs/journal/2026-11-17-fused-finish-registration.md, Researcher PROCEED 6947023492"
- "- `just bench-fused`: master's decoder against `mibz2-fused` at 10 and 12 threads, the ten workers' pair for context, and an L2 diagnostic; its registration is on #145." (https://github.com/lmmx/mibz2/pull/535 (first commit message))
- "Counts, `corpora/wikidata.bz2` (cachegrind with the i9's 32 KiB L1d and 256 KiB L2 as the last level, one worker), per output byte: the CRC's L2 read misses 0.0157 to 0.0003; the process's 0.3077 to 0.2920 (-5.1%); instructions +0.09 (+0.33%)." (https://github.com/lmmx/mibz2/pull/535 (first commit message))
- "It answers #145 QUEUED 6946051925: #183's fusion ported to Cells' output, counted first, with an i9 run to follow. ## What it adds **The problem it targets.** Cells writes a block's RLE1-coded bytes contiguously, so #183's segment chain has nothing to fuse here." (https://github.com/lmmx/mibz2/pull/535 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-17T11:11Z] "That is about 0.5 MB a Wikidata block, past the i9's 256 KiB L2 by then." (https://github.com/lmmx/mibz2/pull/535 (PR body, current text))

**same_quantity_facts:** 165 matching lines in 90 documents dated before commit_to_build, searched on `RLE1|CRC|fused|post.walk` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-14T00:20Z] comment 6868355345 (#145 codex): "Before new data: one standard head/master CI comparison evaluates actual public BzReader at1t and unchanged parallel APIs on existing thread grid/corpora, existing estimator/gates; no custom experimental reader arm, filtering or retry-to-pass. Native parity includes original Segmented and independe…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6868355345
- [2026-11-14T15:29Z] docs/journal/monodromy/2026-11-10-rauzy-veech.md (doc): "up to 6 is refused (S ≈ n/p > 0.16 n; `(ab)^k a` is 0.5 n) and one of longer period is" — https://github.com/lmmx/mibz2/blob/7686bc3559b6/docs/journal/monodromy/2026-11-10-rauzy-veech.md
- [2026-11-14T19:52Z] comment 6883302716 (#145 coordinator): "- **The writer.** Measure before redesigning. The memory census already attributes 65% of Cells' lines and about 70% of modelled 2 MiB misses to the slot orders (6879785194; `research/cells-memory` 95b98eda), which ranks it below induction. The fused post-walk precedent (#2026-11-05) is noted. Not …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6883302716
- [2026-11-15T16:43Z] comment 6897826122 (#145 coordinator): "**The ask: a build, not a census.** #414's table gives the induction buffer per block as 1.97 / 2.27 / 3.22 MiB (min / median / max; slots + arena words + rules). Choose the reduction you judge cheapest per MiB saved (you know where the 2.27 MiB goes), build it on a branch from master with the usua…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6897826122
- [2026-11-16T10:04Z] docs/journal/2026-11-16-sibling-front-end.md (doc): "- with RLE1 and CRC moved too (+0.03): **1.13-1.23**." — https://github.com/lmmx/mibz2/blob/cc2147679e3a/docs/journal/2026-11-16-sibling-front-end.md
- [2026-11-16T17:09Z] docs/journal/monodromy/2026-11-16-staged-chains.md (doc): "\| eight staged chains \| about +13 to +16 more \| hidden behind the other chains \| RLE1 and the CRC read runs; the buffer is about 1.05 to 1.13 n \|" — https://github.com/lmmx/mibz2/blob/92ecc23e418e/docs/journal/monodromy/2026-11-16-staged-chains.md
- [2026-11-17T10:51Z] comment 6945934331 (#145 coordinator): "- **"70–85% of the miss cut on the walk's and expansion's dependent loads":** not supported as stated. The heaviest latency sites are the output-side CRC and unRLE passes, beside `write_cell` and `assemble`." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6945934331
- [2026-11-17T10:53Z] docs/journal/monodromy/2026-11-17-miss-cost-scoring.md (doc): "- *Estimate:* the post-walk passes were 6.4–9.3% of 4-thread CPU on CI, and fusion recovers about" — https://github.com/lmmx/mibz2/blob/b60960de9939/docs/journal/monodromy/2026-11-17-miss-cost-scoring.md
- [2026-11-17T10:54Z] comment 6946003036 (#145 monodromy): "1. **#183's fused post-walk**, re-measured at 7/28 on the i9: 2–5% of wall, kernel count needed. On CI it was NOGO on power, not mechanism (−1.5%, and −1.4% pooled). It needs porting to Cells." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6946003036
- [2026-11-17T10:56Z] comment 6946051925 (#145 coordinator): "**Why now.** The miss-cost counters put the contention's latency on the output-side passes: `crc::clmul_update`, `rle::unrle_avx2` and `SegScratch::assemble`. Each pass re-reads about 0.9 MB of fresh output from the L3. That runs at an L3-hit latency the misses inflate (record 51: hit latency ≈ 45 …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6946051925

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6947023492 (2026-11-17T11:41Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #535, bench-fused)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6946847293, https://github.com/lmmx/mibz2/pull/162#issuecomment-6946859150

## unpinned-trim-4: Lazy kernel at 4 threads, unpinned

- **PRs:** [#539](https://github.com/lmmx/mibz2/pull/539), [#678](https://github.com/lmmx/mibz2/pull/678)
- **Journal sources:** none listed
- **i9 recipes:** unpinned
- **Deciding machine:** i9-10900K (unpinned)
- **Origin (as recorded):** Coordinator; per PR #678 (MONODROMY named)
- **commit_to_build:** 2026-11-17T11:40Z, PR #539 opened (https://github.com/lmmx/mibz2/pull/539)
- **Cost:** i9 6.5 over 1 bundle(s) min [unpinned/run-20261117T130353Z: 6.5 min (bundle start (name)→evidence commit)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 3.64; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "This PR's own changes are the top three commits. ## The registration `docs/journal/2026-11-17-unpinned.md`, frozen before any data. \| threads \| reference (untrimmed) \| candidates \| pinned prediction \| \|--:\|---\|---\|---\| \| 10 \| lazy 10/40 `mibz2-occ-10-40` \| lazy 7/28 `mibz2-occ-7-28` \|…" (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))
- "**Prediction:** MONODROMY's record 87 predicted 1.02 for this cell, a near-tie." (https://github.com/lmmx/mibz2/pull/678 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-unpinned: "the trims unpinned on all 20 CPUs at 10, 8, 6 and 4 threads (#539; docs/journal/2026-11-17-unpinned.md, Researcher PROCEED 6948628221 after its correction; no repeat on a non-ADOPTS)"
- "After LOG 6946756938 on #145: every run behind the worker trims pinned its ≤10-thread arms one CPU a core (`ab run --spread`, CPUs 0..T−1), while production decoders usually run unpinned on all 20 CPUs." (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))
- "This is a frozen workstation recipe that measures the same explicit arms unpinned." (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))
- "- the candidate at 6 threads is now **eager 5/20**, with lazy 5/20 kept as a second candidate;" (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-17T11:40Z] "This PR's own changes are the top three commits. ## The registration `docs/journal/2026-11-17-unpinned.md`, frozen before any data. \| threads \| reference (untrimmed) \| candidates \| pinned prediction \| \|--:\|---\|---\|---\| \| 10 \| lazy 10/40 `mibz2-occ-10-40` \| lazy 7/28 `mibz2-occ-7-28` \| 0.881–0.889 \| \| 8 \| lazy 8/32 `mibz2-occ-8-32` \| lazy 6/24 `mibz2-occ-6-24` \| 0.925 \| \| 6 \|…" (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))
- [2026-11-17T11:40Z] "Its workers are counted from `observed_threads`, and 7 are expected on the i9." (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))
- [2026-11-17T11:40Z] "- 10 rounds with lazy 5/20 would be about 16 minutes, over budget." (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))
- [2026-11-17T11:40Z] "- 10 rounds without lazy 5/20 would be about 14.3 minutes." (https://github.com/lmmx/mibz2/pull/539 (PR body, current text))
- [2026-11-21T02:29Z] "**Prediction:** MONODROMY's record 87 predicted 1.02 for this cell, a near-tie." (https://github.com/lmmx/mibz2/pull/678 (PR body, current text))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `unpinned.{0,40}4 threads|4 threads.{0,40}unpinned|lazy (workers )?at 4` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** lazy-hint/run-20261116T021013Z (3.4 min; Fetch-ahead in the lazy kernel: lazy-hint vs lazy at 4/8/9/10 threads (#459; rule LOG 6905226406))

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6948628221 (2026-11-17T12:54Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #539, #678, bench-unpinned)

**monodromy_involved:** yes: cited in the case's own documents (records 87); records naming its PRs or recipes: 2026-11-18-knee-dram.md, 2026-11-18-recalibration-predictions.md, 2026-11-21-recalibration-diet-predictions.md; origin field names MONODROMY

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/162#issuecomment-6947285652

**Notes:** PR #539's earliest listed commit is 2026-11-16T21:03Z, 15 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## wide-admission: Wider admission for the lazy kernel (r/n 0.15, budget 0.32 n)

- **PRs:** [#556](https://github.com/lmmx/mibz2/pull/556), [#559](https://github.com/lmmx/mibz2/pull/559)
- **Journal sources:** none listed
- **i9 recipes:** admit
- **Deciding machine:** i9-10900K (unpinned)
- **Origin (as recorded):** per PR #556 (Coordinator, Pathfinder named)
- **commit_to_build:** 2026-11-18T09:12Z, PR #556 first commit (author date) (https://github.com/lmmx/mibz2/pull/556/commits)
- **Cost:** i9 5.9 over 1 bundle(s) min [admit/run-20261118T100717Z: 5.9 min (bundle start (name)→evidence commit)]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The census (cachegrind, a 3 MiB last level, matching the i9's memory controller to 0.2 B a byte) puts about a third of the decoder's DRAM traffic on the blocks Cells refuses, which the T-vector walk decodes: on Wikidata 13.7% of the bytes are 32 full blocks just past the density line." (https://github.com/lmmx/mibz2/pull/556 (first commit message))
- "DRAM in the model 11.41 to 9.27 B a byte of output (-18.8%), instructions +7.8%; text and random keep master's route." (https://github.com/lmmx/mibz2/pull/556 (first commit message))
- "- *Which blocks those are:* on Wikidata, 13.7% of the bytes are 32 full blocks just past the density line, at r/n 0.097–0.148, needing 0.161–0.241 cells a row against the budget's 0.16. ## What it adds - **`bwt::WideLazyCells`** (`src/cells_wide.rs`) is the lazy kernel, with its induction and write…" (https://github.com/lmmx/mibz2/pull/556 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-admit: "WideLazyCells admission (density < 0.15, budget 0.32n) against master f71de528, unpinned at 8 and 10 threads (#556; adoption registration docs/journal/2026-11-18-admit-registration.md, Researcher REVI"
- "bench(admit): wider admission for the lazy kernel, and its i9 run with the memory controller #145 QUEUED 6973253342, the Pathfinder's REVIEW 6973651863 and the Coordinator's go-ahead." (https://github.com/lmmx/mibz2/pull/556 (first commit message))
- "The census (cachegrind, a 3 MiB last level, matching the i9's memory controller to 0.2 B a byte) puts about a third of the decoder's DRAM traffic on the blocks Cells refuses, which the T-vector walk decodes: on Wikidata 13.7% of the bytes are 32 full blocks just past the density line." (https://github.com/lmmx/mibz2/pull/556 (first commit message))
- "- `bwt::WideLazyCells` (`cells_wide.rs`, a bench arm): the lazy kernel, its induction and writer master's own code, admitting to r/n 0.15 (3 records in 20 rows, not 1 in 10) with a budget of 0.32 n (not 0.16 n); the sparse gate Cells' own." (https://github.com/lmmx/mibz2/pull/556 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-18T09:12Z] "DRAM in the model 11.41 to 9.27 B a byte of output (-18.8%), instructions +7.8%; text and random keep master's route." (https://github.com/lmmx/mibz2/pull/556 (first commit message))
- [2026-11-18T10:38Z] "- `tests/lazy_admission.rs`: blocks between the density lines (Cells refuses them for density) and past Cells' budget decode to the same bytes through the lazy kernel, Cells and Segmented, and through lazy workers; every corpus at two levels; corrupt input (each block's stored CRC flipped, bits flipped through each block's body, the stream cut short) gives Segmented's output and exact error seque…" (https://github.com/lmmx/mibz2/pull/559 (first commit message))
- [2026-11-18T10:39Z] "It can't exercise this change, because lazy workers start at 8 threads and the runner's highest count is 4." (https://github.com/lmmx/mibz2/pull/559 (PR body, current text))
- [2026-11-18T10:39Z] "- *What it buys:* 12–16% less wall time on the primary corpus, and 25% fewer DRAM bytes per output byte." (https://github.com/lmmx/mibz2/pull/559 (PR body, current text))
- [2026-11-18T10:39Z] "Both are gains of at least 5%, with growth within twice the gain, +25 MB (under 64 MB), and no other corpus growing." (https://github.com/lmmx/mibz2/pull/559 (PR body, current text))
- [2026-11-18T10:39Z] "- *corrupt input:* each block's stored CRC flipped, bits flipped through each block's body, and the stream cut short." (https://github.com/lmmx/mibz2/pull/559 (PR body, current text))

**same_quantity_facts:** 209 matching lines in 131 documents dated before commit_to_build, searched on `admission|admit|r/n ?0\.|budget of 0\.|density <` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-16T19:18Z] comment 6926545736 (#145 optimizer): "All 29 archives used below were read from [`evidence` at 14b969f4](https://github.com/lmmx/mibz2/tree/14b969f4ef2d9c8da4c7ac77643babbddc28b6c6). Each compressed bundle matched its adjacent `.sha256`. A table label such as `lazy-place/run-20261116T101100Z` means `evidence/<label>.tar.gz`; each row n…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6926545736
- [2026-11-16T22:39Z] docs/journal/monodromy/2026-11-15-equal-words.md (doc): "they fit in n bytes up to r/n ≈ 0.083, under the admission limit of 0.10, so a buffer" — https://github.com/lmmx/mibz2/blob/9ea511bf7cde/docs/journal/monodromy/2026-11-15-equal-words.md
- [2026-11-16T23:01Z] docs/journal/monodromy/2026-11-16-store-census.md (doc): "**Per letter** means per admitted run of Wikidata: n/r ≈ 15.9 rows a letter (r/n ≈ 0.063)." — https://github.com/lmmx/mibz2/blob/70f0973a004d/docs/journal/monodromy/2026-11-16-store-census.md
- [2026-11-17T08:54Z] comment 6943371729 (#145 researcher): "**PROCEED on the frozen registration and its ADOPTS verdict.** Archive SHA-256 verified; pinned checker reproduces verdict.txt byte-for-byte from all200 records. Independent arithmetic confirms Wikidata20 **0.980076 [0.9710,0.9892]**, p0.00042758; Holm rejects20 only. No judged cell SLOWER; RSS rat…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6943371729
- [2026-11-17T08:56Z] comment 6943398069 (#145 coordinator): "- The Researcher's limits apply: the text guard is a screen, not equivalence; RSS ≤ 1.10 proves no saving; and the admission does not certify W12 under the new worker counts." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6943398069
- [2026-11-17T09:06Z] comment 6943617731 (#145 researcher): "**PROCEED on the frozen rules and their verdicts; proceed with seven lazy workers/28 slots past the cores and at ten requested threads, on the measured i9 profile.** Retain your deferral of eight and below. This is bounded evidence admission, not implementation/CI certification or a general 0.7 × t…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6943617731
- [2026-11-17T10:57Z] comment 6946065798 (#145 monodromy): "2. **k's transfer did use the buffer ratio (0.355) as a stand-in.** The hot-set ratio, now counted, is **0.377**: r = 26,046 at -3 (every 72nd block, the same offsets), with r/n 0.0868 against 0.0767." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6946065798
- [2026-11-17T16:15Z] comment 6953772428 (#145 researcher): "3. **Narrow the release summary to the real predicate.** “From 10 threads in any mode” is false for arbitrary masks. Release d27b6e7f admits exactly T distinct-core CPUs or full-machine affinity with T≥10; a ten-logical-CPU mask on five SMT cores fails both. Low trim counts are **4, 6, 8**, not the…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6953772428
- [2026-11-18T08:40Z] comment 6973462975 (#145 researcher): "**Measurements — VERIFIED within their scope.** The archive records direct equality of both fused outputs with libbzip2 over **209,700,196 bytes**. The frozen block was replayed here. The native full-stream cmp remains archival. Raw data reproduce **367/431** admitted blocks, all 233 large blocks, …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6973462975
- [2026-11-18T08:50Z] comment 6973651863 (#145 pathfinder): "- *The rule:* #506's two-arm rule, with an RSS gate of 1.10. Plus a scope note: an i9 ADOPTS qualifies the measured profile only. Forced admission lifts the sparse and budget gates, so adoption anywhere needs its own memory and adversarial-input review." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6973651863

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** service-lookahead/20261109T075939Z (6.7 min; the prepared-queue admission threshold at svc2 (#271))

**Review routing:** Researcher: unrecorded (no comment from the role naming #556, #559, bench-admit) / Optimiser: unrecorded (no comment from the role naming #556, #559, bench-admit)

**monodromy_involved:** yes: cited in the case's own documents (records 45); records naming its PRs or recipes: 2026-11-18-recalibration-predictions.md

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/162#issuecomment-6974215305

## one-scan-writer: Cells writes each full-period block in one scan of its rules (study 5)

- **PRs:** [#579](https://github.com/lmmx/mibz2/pull/579), [#581](https://github.com/lmmx/mibz2/pull/581)
- **Journal sources:** docs/journal/2026-11-18-one-scan-writer.md
- **i9 recipes:** none
- **Deciding machine:** CI
- **Origin (as recorded):** Cells study 5 (#579)
- **commit_to_build:** 2026-11-18T17:21Z, PR #579 first commit (author date) (https://github.com/lmmx/mibz2/pull/579/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.08; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "This saves 44.12% of the counted writer metadata accesses and 16.91% under the declared structural cost model." (https://github.com/lmmx/mibz2/pull/579 (PR body, current text))
- "Before every run the output is poisoned and the rules restored, uncharged. \| corpus \| eager \| lazy \| \|---\|--:\|--:\| \| Wikidata (196 / 228 blocks, all full period) \| **0.813** \| **0.785** \| \| the frozen block (28,710 rules, 9,628 commands) \| 0.797 \| 0.800 \| \| forced Cells: text, rand…" (https://github.com/lmmx/mibz2/pull/581 (PR body, current text))

**must_hold** (quoted):

- "On a full-period block, the cut that exhausts each rule child selects an occurrence from which the writer can copy." (https://github.com/lmmx/mibz2/pull/579 (PR body, current text))
- "The copies fit in the rule slots already consumed and their order makes every source complete." (https://github.com/lmmx/mibz2/pull/579 (PR body, current text))
- "Across the 233 large Wikidata blocks, the second scan of 8,176,313 rules becomes a scan of 2,528,876 deferred copies." (https://github.com/lmmx/mibz2/pull/579 (PR body, current text))
- "perf(cells): write each full-period block in one scan of its rules (study 5, #579)" (https://github.com/lmmx/mibz2/pull/581 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-18T17:22Z] "On a full-period block, the cut that exhausts each rule child selects an occurrence from which the writer can copy." (https://github.com/lmmx/mibz2/pull/579 (PR body, current text))
- [2026-11-18T17:22Z] "This saves 44.12% of the counted writer metadata accesses and 16.91% under the declared structural cost model." (https://github.com/lmmx/mibz2/pull/579 (PR body, current text))
- [2026-11-18T18:53Z] "A violation would leave bytes unwritten, which the CRC rejects." (https://github.com/lmmx/mibz2/pull/581 (PR body, current text))
- [2026-11-18T18:53Z] "It would never access out of bounds." (https://github.com/lmmx/mibz2/pull/581 (PR body, current text))

**same_quantity_facts:** 15 matching lines in 12 documents dated before commit_to_build, searched on `one.scan|full.period|Cells writer|write_cells` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T03:22Z] docs/journal/2026-11-10-par-scan-result.md (doc): "workers at 2.56 GB/s with nearly flat CPU, where one scanner capped at about 1.6 GB/s." — https://github.com/lmmx/mibz2/blob/4fe9f12004f7/docs/journal/2026-11-10-par-scan-result.md
- [2026-11-10T03:23Z] comment 6758059674 (#222 coordinator): "1. **The first workstation result of the night: par-scan** (#317, merged at 4fe9f120; LOG 6757860786). With level-9 tables, plain workers peak at 4 and fall by 10. With level-1 tables, the parallel finder reaches 2.56 GB/s at 10 workers with CPU almost flat, while one scanner caps at about 1.6. Thi…" — https://github.com/lmmx/mibz2/pull/222#issuecomment-6758059674
- [2026-11-10T05:51Z] comment 6760156549 (#145 pathfinder): "**What replaces them, measured.** The smallest table there is, -2's own, behind one scanner: 1,862. The default's sentinels in the same attempt: 1,499 (another block stream; context). So a scanner-fed small table is worth about 1.24× before its own cost, and with g of 1.18, 1.44, 1.7 on your surfac…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6760156549
- [2026-11-10T05:52Z] comment 6760167271 (#160 coordinator): "- **Its new reading:** the smallest table behind one scanner reaches 1,862 MB/s, against the default's 1,499. So a scanner-fed small table does not beat the default once its g is charged, and the small table and the parallel scan are one proposal." — https://github.com/lmmx/mibz2/pull/160#issuecomment-6760167271
- [2026-11-10T05:56Z] docs/journal/2026-11-10-share-nothing-pricing.md (doc): "- **The scanner binds near 5×** on small blocks (`2026-11-10-parallel-scan.md`). One scanner thread" — https://github.com/lmmx/mibz2/blob/8c40bde6b22a/docs/journal/2026-11-10-share-nothing-pricing.md
- [2026-11-14T16:04Z] docs/journal/monodromy/2026-11-14-feathered-screen/method.md (doc): "0.98 to 1.11 S: one scan of about 1.2 r cells and one rewrite of about 0.7 r letters. After" — https://github.com/lmmx/mibz2/blob/920cdf1b1958/docs/journal/monodromy/2026-11-14-feathered-screen/method.md
- [2026-11-15T17:16Z] comment 6898249821 (#145 coordinator): "- The 16 → 12 B rule, its second candidate, it withdrew after reading `write_cells`: the writer needs both factor lengths, so recovering a short factor's length costs either an arena length byte (net saving ≈ 0.06 MiB, about half its first estimate, plus two dependent loads a rule) or offset bits t…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6898249821
- [2026-11-16T17:28Z] docs/journal/monodromy/2026-11-16-compact-tail.md (doc): "# 2026-11-16 · MONODROMY 27 · A compact tail grammar keeps the whole induction and the Cells writer, with no walk: switch at 1.2 r to joins of two u16 IDs (u24 where they overflow) and 32% of the word bytes go, 4.9 B a letter. And the Ghrist relay's §3, §5 and §11, answered from records 24 and 26" — https://github.com/lmmx/mibz2/blob/51e1e7adc97d/docs/journal/monodromy/2026-11-16-compact-tail.md
- [2026-11-16T18:31Z] comment 6925397308 (#145 pathfinder): "**The writer was cut from +108 to +29.4.** Each join is decoded once into a 16-byte record, and joins and rules share one first-place array in the dead top slots, as `write_cells` keeps its own. All terminals sit in one table. The join records' bounds are guaranteed by construction, as `write_cells…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6925397308
- [2026-11-18T08:50Z] comment 6973651863 (#145 pathfinder): "\| zero-fill and text of the pre-RLE/slot buffer: `slots_lazy`'s resize 0.30, `write_cells` 0.14 \| 0.44 \| see §2 \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6973651863

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6986491194 (2026-11-18T17:37Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #579, #581)

**monodromy_involved:** yes: cited in the case's own documents (records 67); records naming its PRs or recipes: 2026-11-18-live-reuse-verification.md, 2026-11-18-owner-schedules-verification.md

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6986629119, https://github.com/lmmx/mibz2/pull/162#issuecomment-6986296845, https://github.com/lmmx/mibz2/pull/314#issuecomment-6986665685

## first-occurrence-writer: Cells study 6: first-occurrence writer

- **PRs:** [#587](https://github.com/lmmx/mibz2/pull/587)
- **Journal sources:** docs/journal/2026-11-18-first-occurrence-writer.md
- **i9 recipes:** none
- **Deciding machine:** local, against #581 on identical grammars
- **Origin (as recorded):** Cells study 6
- **commit_to_build:** 2026-11-18T19:39Z, PR #587 first commit (author date) (https://github.com/lmmx/mibz2/pull/587/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "**Callgrind (simulated), eager:** the walk executes 0.86× the instructions, with 0.55× the writes and 0.33× the D1 write misses." (https://github.com/lmmx/mibz2/pull/587 (PR body, current text))
- "But it has **2.5× the D1 read misses** and **6.0× the branch mispredictions**: 11.0 M against 1.8 M, about 1.7 a rule." (https://github.com/lmmx/mibz2/pull/587 (PR body, current text))

**must_hold** (quoted):

- "But it has **2.5× the D1 read misses** and **6.0× the branch mispredictions**: 11.0 M against 1.8 M, about 1.7 a rule." (https://github.com/lmmx/mibz2/pull/587 (PR body, current text))
- "It tags a rule's first field on first entry, writes a short left operand at once, pushes a rule left operand's right partner, and copies at once on a tagged visit." (https://github.com/lmmx/mibz2/pull/587 (PR body, current text))
- "The stack is bounded by the rule count." (https://github.com/lmmx/mibz2/pull/587 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-18T19:39Z] "Its per-visit four-way dispatch follows the grammar's shape, which the predictor cannot learn, and its visits chase records rather than scan them." (https://github.com/lmmx/mibz2/pull/587 (PR body, current text))

**same_quantity_facts:** 184 matching lines in 120 documents dated before commit_to_build, searched on `first.occurrence|writer|one.scan` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T17:37Z] comment 6986491194 (#145 researcher): "5. **VERIFIED, relevance.** Eager writer is 19.037%, 18.923% and 19.006% of one-thread i9 kernel time on the admitted population. I found no matching lazy phase-time share. The frozen lazy writer is 13.13% of the three attributed functions' instructions. Candidate time remains **UNPROVED**." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6986491194
- [2026-11-18T17:39Z] docs/journal/monodromy/2026-11-18-live-reuse-verification.md (doc): "gives writer 29%, slots 21%, induction 50%." — https://github.com/lmmx/mibz2/blob/2823064f2c88/docs/journal/monodromy/2026-11-18-live-reuse-verification.md
- [2026-11-18T17:42Z] docs/journal/2026-11-18-cells-live-reuse.md (doc): "There is a lookup-free arithmetic certificate for any two literals. Give the word $ab$ the address of entry $256a+b$ in a fixed catalogue of byte pairs. All 5,793,942 digram constructions would get canonical descriptors and the ordinary writer could use them directly. The two-byte test on 19,263,44…" — https://github.com/lmmx/mibz2/blob/284c03ded48f/docs/journal/2026-11-18-cells-live-reuse.md
- [2026-11-18T17:43Z] comment 6986629119 (#145 coordinator): "1. **Writer only:** the old and new writers on identical finished Cells grammars, the frozen block and a corpus sample, so the writer's own time is isolated. Count what the ledger leaves unpriced: branch mispredictions on the certificate (31% of long references survive), the store and copy order (3…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6986629119
- [2026-11-18T17:44Z] comment 6986665685 (#314 coordinator): "1. **Writer only:** the old and new writers on identical finished Cells grammars, the frozen block and a corpus sample, so the writer's own time is isolated. Count what the ledger leaves unpriced: branch mispredictions on the certificate (31% of long references survive), the store and copy order (3…" — https://github.com/lmmx/mibz2/pull/314#issuecomment-6986665685
- [2026-11-18T19:01Z] comment 6988550050 (#346 coordinator): "**Context, not a premise:** #581 (study 5's last-use writer) benched 4.8% faster on Wikidata at one thread, and is awaiting the Guardian's review. The question here is the alternative schedule's own claims, not #581." — https://github.com/lmmx/mibz2/issues/346#issuecomment-6988550050
- [2026-11-18T19:09Z] docs/journal/2026-11-18-cells-owner-schedules.md (doc): "The scalar ledger charges tag tests, tag encoding, cursor arithmetic, capacity checks, stack traffic and cold growth. Across ten blocks it favors the first-occurrence walk by **0.66%** at one unit per field or scalar operation and **5.23%** at two units per field. Equally, an extra cost of only 0.1…" — https://github.com/lmmx/mibz2/blob/fce68453aeb7/docs/journal/2026-11-18-cells-owner-schedules.md
- [2026-11-18T19:09Z] docs/journal/2026-11-18-cells-owner-schedules/README.md (doc): "**Line model.** The two traces start with empty, fully associative 32 KiB LRU data caches, 64-byte lines, write allocation and zero-mod-64-aligned regions. A short write reads the advertised byte span from the short arena and writes its destination. A copy references all its source lines then all i…" — https://github.com/lmmx/mibz2/blob/fce68453aeb7/docs/journal/2026-11-18-cells-owner-schedules/README.md
- [2026-11-18T19:23Z] docs/journal/2026-11-18-one-scan-writer.md (doc): "- **The writer's share of whole-kernel time** is study 5's i9 figure, 19% of eager kernel time." — https://github.com/lmmx/mibz2/blob/3502dd9bbc75/docs/journal/2026-11-18-one-scan-writer.md
- [2026-11-18T19:28Z] comment 6989227266 (#145 pathfinder): "1. **Writer-only, on identical grammars:** 0.813 eager and 0.785 lazy on Wikidata, and 0.80 on the frozen block. Bytes are identical everywhere. The record is `docs/journal/2026-11-18-one-scan-writer.md`." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6989227266

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #587) / Optimiser: unrecorded (no comment from the role naming #587)

**monodromy_involved:** yes: cited in the case's own documents (records 69)

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/314#issuecomment-6989549514

## astra-e: Astra E moonshot: exact marked block reconstruction (lookahead, narrow admission, edge budget, scalar cap, cap counter)

- **PRs:** [#607](https://github.com/lmmx/mibz2/pull/607)
- **Journal sources:** none listed
- **i9 recipes:** astra-e, astra-e-scalar, astra-e-cap-counter, scalar-vs-open
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Astra E (ChatGPT research agent), brief docs/astra-e-brief
- **commit_to_build:** 2026-11-18T23:41Z, PR #607 opened (https://github.com/lmmx/mibz2/pull/607)
- **Cost:** i9 127.5 over 11 bundle(s) min [astra-e/run-20261119T005641Z: 0.1 min (bundle start (name)→evidence commit); astra-e/run-20261119T012848Z: 2.5 min (bundle start (name)→evidence commit); astra-e/run-20261119T013135Z: 11.6 min (bundle start (name)→evidence commit); astra-e/run-20261119T021531Z: 5.5 min (bundle start (name)→evidence commit); astra-e/run-20261119T022117Z: 12.0 min (bundle start (name)→evidence commit); astra-e/run-20261119T083215Z: 2.5 min (bundle start (name)→evidence commit); astra-e/run-20261119T083507Z: 11.6 min (bundle start (name)→evidence commit); astra-e-scalar/run-20261119T045313Z: 5.5 min (bundle start (name)→evidence commit); astra-e-scalar/run-20261119T045900Z: 31.9 min (bundle start (name)→evidence commit); astra-e-cap-counter/run-20261119T060341Z: 6.7 min (bundle start (name)→evidence commit); astra-e-cap-counter/run-20261119T061036Z: 37.6 min (bundle start (name)→evidence commit)]; CI: 47 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: unrecorded; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The objective is 1.25× mibz2 0.5.6 throughput at each of 7–10 workers on the i9-10900K, with no increase in peak RSS." (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))
- "Its maximum peak in the new W10 run was 223,152 KiB versus scalar's 223,760 KiB and Lazy's 360,400 KiB. [Previous 256-sample i9 comparison](https://github.com/lmmx/mibz2/blob/79eadb9/research/astra-e/workstation/edge-i9.md) **Engineering.** The seven-arm independent CI gave W4 combined counter/scal…" (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))
- "On the frozen block, native full construction and emission took 1.384 [1.121,1.708] of direct LF and 6,368 vs 7,512 KiB of peak memory." (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-astra-e-cap-counter: "cap-counter against scalar-cap, edge-budget, LazyCells and opend, W = 7..10, unpinned, 40 rounds (800 samples); joint incremental rule against both i9 ancestors (#607; request 7001022850; rules in res"
- queue entry bench-scalar-vs-open: "scalar-cap against ungated open, opend and LazyCells in one run, W = 7..10, unpinned, 16 rounds; decides nothing in production (rule 7003825682 before the data; Louis approved the slot)"
- "The objective is 1.25× mibz2 0.5.6 throughput at each of 7–10 workers on the i9-10900K, with no increase in peak RSS." (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))
- "Its maximum peak in the new W10 run was 223,152 KiB versus scalar's 223,760 KiB and Lazy's 360,400 KiB. [Previous 256-sample i9 comparison](https://github.com/lmmx/mibz2/blob/79eadb9/research/astra-e/workstation/edge-i9.md) **Engineering.** The seven-arm independent CI gave W4 combined counter/scal…" (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-19T06:46Z] "Supply complete next source bases and marked LF from cut emission; audit every root, node, and row on identical histories" (https://github.com/lmmx/mibz2/pull/607 (first commit message))
- [2026-11-18T23:41Z] "On the ten original quote columns a maximum-weight compatible selection removed 402,876 of 1,357,067 rows." (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))
- [2026-11-18T23:41Z] "This removes the residual permutation on those blocks." (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))
- [2026-11-18T23:41Z] "It folds maximal same-type powers when needed and chooses a weighted cut that removes at least a quarter of the remaining active rows." (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))
- [2026-11-18T23:41Z] "Even the earlier constructor's first cut cost 1.693 [1.645,1.742] of complete scalar." (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))
- [2026-11-18T23:41Z] "A selective version now asks which source positions actually need recovery. [Formulation ledger](https://github.com/lmmx/mibz2/blob/research/astra-e-moonshot/research/astra-e/formulations/LEDGER.md) · [Direct original-byte grammar and native program](https://github.com/lmmx/mibz2/tree/research/astra-e-moonshot/research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/or…" (https://github.com/lmmx/mibz2/pull/607 (PR body, current text))

**same_quantity_facts:** 180 matching lines in 87 documents dated before commit_to_build, searched on `lookahead|edge budget|scalar.cap|capped.run|1\.25` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T11:24Z] docs/journal/monodromy/2026-11-18-recalibration-predictions.md (doc): "\| 4, unpinned \| eager 4 \| **4** (C) or 5 (N) \| 3: **1.21 [1.17, 1.25]** (C: 1.214) \| 5: 0.94 [0.90, 0.98] (**N**) \| = shipped \|" — https://github.com/lmmx/mibz2/blob/131c8a548a05/docs/journal/monodromy/2026-11-18-recalibration-predictions.md
- [2026-11-18T11:25Z] comment 6977179546 (#145 monodromy): "\| 4 \| unpinned \| eager 4 \| **4** \| 3: 1.21 [1.17, 1.25] (measured) · 5: 0.94 [0.90, 0.98], **no basis** \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6977179546
- [2026-11-18T12:45Z] docs/journal/monodromy/2026-11-18-recalibration-scoring.md (doc): "\| 4 \| 4 \| **4 ✓** \| 3: 1.21 [1.17, 1.25] → 1.225 ✓. 5 was not run. \|" — https://github.com/lmmx/mibz2/blob/567582854297/docs/journal/monodromy/2026-11-18-recalibration-scoring.md
- [2026-11-18T14:59Z] docs/journal/2026-11-18-switch-elimination/README.md (doc): "The study uses mibz2 0.5.5 at `f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71`, Rust 1.90.0 and Python 3. The three research kernels are installed into a disposable checkout. Their normal entry point is `mibz2::decode_with`, including the scanner, Huffman, MTF/RLE2, inverse BWT, RLE1 and the block and st…" — https://github.com/lmmx/mibz2/blob/9cd25610131b/docs/journal/2026-11-18-switch-elimination/README.md
- [2026-11-18T15:09Z] docs/journal/monodromy/2026-11-18-synthesis.md (doc): "\| **P4** \| Whether investigation 1's 3R − 2 is tight. The worst found is 1.25 R (record 55). \|" — https://github.com/lmmx/mibz2/blob/d25650265746/docs/journal/monodromy/2026-11-18-synthesis.md
- [2026-11-18T17:29Z] comment 6986296845 (#162 coordinator): "6. **The side results,** kept separate: 12,126,015 duplicate short words classified by cause; 1,214,986 further adjacent products certified by #575's lookahead, the family screening below 1%; and the forwarding of same-side boundaries." — https://github.com/lmmx/mibz2/pull/162#issuecomment-6986296845
- [2026-11-18T17:29Z] comment 6986297212 (#346 coordinator): "6. **The side results,** kept separate: 12,126,015 duplicate short words classified by cause; 1,214,986 further adjacent products certified by #575's lookahead, the family screening below 1%; and the forwarding of same-side boundaries." — https://github.com/lmmx/mibz2/issues/346#issuecomment-6986297212
- [2026-11-18T17:42Z] docs/journal/2026-11-18-cells-live-reuse.md (doc): "**Boundary forwarding.** During consecutive wins of one side the outer previous link of the winner's unconsumed interval is invariant. The scalar T writes that link into the next promised bottom slot, then the next T reloads the very field it just wrote. BB has the mirror identity on top. Retaining…" — https://github.com/lmmx/mibz2/blob/284c03ded48f/docs/journal/2026-11-18-cells-live-reuse.md
- [2026-11-18T17:42Z] docs/journal/2026-11-18-cells-live-reuse/cost.md (doc): "Eager Cells can wait for its normal bottom read and guard on the old operand's literal status. This costs one predicate on every T and up to three more operations on each T with a plain old literal. The pooled forced-trace balance is $7H-7,774,453-3(1,871,765)=-331,297$. Testing every refilled pair…" — https://github.com/lmmx/mibz2/blob/284c03ded48f/docs/journal/2026-11-18-cells-live-reuse/cost.md
- [2026-11-18T22:31Z] docs/journal/2026-11-18-astra-d-memory.md (doc): "\| Capped run records \| 5,290 \| 2,618 \| 0 \| 1,763 \| 1,763 \| 0.251 \|" — https://github.com/lmmx/mibz2/blob/4f7d02350259/docs/journal/2026-11-18-astra-d-memory.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** service-lookahead/20261109T075939Z (6.7 min; the prepared-queue admission threshold at svc2 (#271))

**Review routing:** Researcher: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6997277733 (2026-11-19T01:54Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #607, bench-astra-e, bench-astra-e-scalar, bench-astra-e-cap-counter, bench-scalar-vs-open)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

**Notes:** 11 i9 bundles over 3 recipe(s); coders check whether master moved between them.

## open-kernel-port: The Open kernel as the workers' kernel on LazyCells' admitted blocks (B5 port)

- **PRs:** [#590](https://github.com/lmmx/mibz2/pull/590), [#604](https://github.com/lmmx/mibz2/pull/604), [#631](https://github.com/lmmx/mibz2/pull/631), [#638](https://github.com/lmmx/mibz2/pull/638), [#630](https://github.com/lmmx/mibz2/pull/630)
- **Journal sources:** docs/journal/2026-11-19-open-port-registration.md, docs/journal/2026-11-19-open-port-b5.md
- **i9 recipes:** open-return, opend-sweep, open-port, open-port-3
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Astra's open kernel (Astra investigation 2/3; per PR #630 Astra named)
- **commit_to_build:** 2026-11-18T20:14Z, PR #590 opened (https://github.com/lmmx/mibz2/pull/590)
- **Cost:** i9 167.1 over 4 bundle(s) min [open-return/run-20261118T210422Z: 13.6 min (started→finished.txt); opend-sweep/run-20261118T225038Z: 20.7 min (started→finished.txt); open-port/run-20261119T125538Z: 54.4 min (started→finished.txt); open-port-3/run-20261119T171047Z: 78.4 min (started→finished.txt)]; CI: 19 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.93; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **The rule:** `two_arm.py`, one Holm family at α 0.05 over the 5 main-corpus cells, with `--check-workers`, reference admission, and rule 4's trade on `wikidata-x4,text`." (https://github.com/lmmx/mibz2/pull/590 (PR body, current text))
- "- **P1:** a wall gain on Wikidata at every count, Holm at α 0.05;" (https://github.com/lmmx/mibz2/pull/631 (PR body, current text))
- "- **P2:** Wikidata peak RSS ≤ 1.00 of v0.5.6's;" (https://github.com/lmmx/mibz2/pull/631 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-open-return: "the open kernel (open, opend) against 0.5.6's plan, unpinned at 10 and 8 threads (#590; C6 registration docs/journal/2026-11-18-open-return-comparison-registration.md frozen at 8045ce81 after the Rese"
- queue entry bench-opend-sweep: "opend against LazyCells, kernel-only, W = 2..10 workers unpinned, with stall counters and IMC/RAPL (#604; the B5 study's step 1, QUEUED 6992388066; rule LOG 6992528145 before the data, testing MONODRO"
- queue entry bench-open-port: "the open port's ONE confirmatory C6 attempt: port 9743bbe0 against LazyCells, W = 7..10, unpinned, about 65 min; decides 0.5.7 (#631/#630; registration docs/journal/2026-11-19-open-port-registration.m"
- queue entry bench-open-port-3: "open-port-3: the proportionate validation of the open port for 0.5.7 (#638 frozen at db004d00; one attempt, about 86 min; Louis's authority 19 Nov; decides 0.5.7"

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-19T14:46Z] "**The registration's own flaw, stated by the Coordinator (#145 LOG 7011104819):** - *The rule:* G compared each guard's point estimate with 1.02, over 39 guard cells whose 95% intervals reached about ±9%." (https://github.com/lmmx/mibz2/blob/ef25d39d3a3a/docs/journal/2026-11-19-open-port-registration.md (at its first commit))
- [2026-11-19T14:46Z] "A different experiment would need its own prospective review, and would leave this attempt and its outcome standing." (https://github.com/lmmx/mibz2/blob/ef25d39d3a3a/docs/journal/2026-11-19-open-port-registration.md (at its first commit))
- [2026-11-19T14:46Z] "- *Not on the evidence branch:* the one-attempt guard refuses any `open-port/run-*` bundle archived there, so a preflight there would block the attempt." (https://github.com/lmmx/mibz2/blob/ef25d39d3a3a/docs/journal/2026-11-19-open-port-registration.md (at its first commit))
- [2026-11-19T14:46Z] "Text is expected to show the gate's purpose." (https://github.com/lmmx/mibz2/blob/ef25d39d3a3a/docs/journal/2026-11-19-open-port-registration.md (at its first commit))
- [2026-11-18T20:14Z] "The recipe checks this: only `Cargo.toml`, `src/lib.rs` and the new `src/open_gate.rs` differ, with no line removed." (https://github.com/lmmx/mibz2/pull/590 (PR body, current text))
- [2026-11-18T20:14Z] "- **The recipe:** `--ref-workers 10=7 --ref-workers 8=8 --machine-model i9-10900K --lscpu "$O/lscpu.txt"`." (https://github.com/lmmx/mibz2/pull/590 (PR body, current text))

**same_quantity_facts:** 111 matching lines in 93 documents dated before commit_to_build, searched on `\bopen(d)? kernel|\bOpen\b|LazyCells|endpoint|thin set` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T15:20Z] comment 6982850361 (#145 researcher): "4. **Oracle replay specified and unrun.** Freeze the actual endpoint tape and coordinate contract before calling anything a 16,917-endpoint reproduction. A reproducible separate surrogate grants text-addressed Φ equalities: 10,954 spans plus 51,184 positioned literals. Time the compiler and emitter…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6982850361
- [2026-11-18T15:31Z] comment 6983166783 (#145 researcher): "- **Costs:** 0.22/1.21 ms are independently selected best-of-200 segments of an isolated probe. Its loop omits parts of native forest setup; the 35.30 ms denominator is the Xeon median. The 2.09 ms / 5.9% scaling is a cross-run estimate, so exact attribution needs native per-level phase instrumenta…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6983166783
- [2026-11-18T16:41Z] docs/journal/2026-11-18-cells-mixed-steps.md (doc): "Record 61's roughly 1.8 events per run compares three different counters. The independent review's forest tie witness changes 12 arcs to 15 on a fixed input. Controlling for both run density and normalized excess reduces the Cells–forest correlation from 0.906 to 0.393. Record 62 found increasing f…" — https://github.com/lmmx/mibz2/blob/82f2767d317a/docs/journal/2026-11-18-cells-mixed-steps.md
- [2026-11-18T17:38Z] comment 6986512740 (#162 coordinator): "- open return's 9–21%;" — https://github.com/lmmx/mibz2/pull/162#issuecomment-6986512740
- [2026-11-18T17:43Z] comment 6986629119 (#145 coordinator): "Open return lost 9–21% to eager Cells on the VM, so its expected value is lower." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6986629119
- [2026-11-18T17:44Z] comment 6986665685 (#314 coordinator): "Open return lost 9–21% to eager Cells on the VM, so its expected value is lower." — https://github.com/lmmx/mibz2/pull/314#issuecomment-6986665685
- [2026-11-18T17:46Z] comment 6986707297 (#145 researcher): "2. **Correct the timing table and its figure.** Open Return's 9–21% covers calling thread, two pool workers and four pool workers. The calling-thread figure is +17.1%. Investigation 1's 8.370/0.729 s reproduces on the four-CPU cloud VM, one calling thread. The figure's 8–12× range spans worker mode…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6986707297
- [2026-11-18T18:54Z] comment 6988388052 (#162 coordinator): "- **The timing labels:** calling thread against worker pools, open return 17.1% / 21.1% / 9.1%, and adjacent-rank 11.5× and 7.8×." — https://github.com/lmmx/mibz2/pull/162#issuecomment-6988388052
- [2026-11-18T19:02Z] comment 6988577904 (#145 researcher): "Record 65's d₀ split of 229 at depth one and four at depth two resolves the conflict with record 63’s universal chord claim. The internal count equals the forest excess, and the historical count equals the chords only at d₀ = 1. The revised timing labels and ratios check out, including 17.1% / 21.1…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6988577904
- [2026-11-18T19:42Z] comment 6989549514 (#314 coordinator): "**The question:** does the open decoder's smaller state (6.29 MB of peak requested heap against Cells' 10.35 MB on the frozen block) give a throughput gain in the regime where contention limits us? Its own measurements, on a 4-CPU Xeon VM with no SMT siblings, were +17% wall against eager Cells at …" — https://github.com/lmmx/mibz2/pull/314#issuecomment-6989549514

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** admit/run-20261118T100717Z (5.9 min; WideLazyCells admission (density < 0.15, budget 0.32n) against master f71de528, unpinned at 8 and 10 threads (#556; ado…)

**Review routing:** Researcher: 9 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6990885206 (2026-11-18T20:35Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #590, #604, #631, #638, #630, bench-open-return, bench-opend-sweep, bench-open-port, bench-open-port-3)

**monodromy_involved:** yes: cited in the case's own documents (records 54, 71); records naming its PRs or recipes: 2026-11-18-open-return-scoring.md, 2026-11-19-recalibration-057-predictions.md

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6990679695, https://github.com/lmmx/mibz2/pull/162#issuecomment-6990728619

**Notes:** 4 i9 bundles over 4 recipe(s); coders check whether master moved between them. PR #590's earliest listed commit is 2026-11-18T11:35Z, 9 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #604's earliest listed commit is 2026-11-18T11:35Z, 10 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #631's earliest listed commit is 2026-11-18T11:35Z, 23 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #638's earliest listed commit is 2026-11-18T11:35Z, 29 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## trim-057: Lazy kernel at 6 threads (0.5.7's recalibration)

- **PRs:** [#640](https://github.com/lmmx/mibz2/pull/640), [#642](https://github.com/lmmx/mibz2/pull/642)
- **Journal sources:** none listed
- **i9 recipes:** recalibrate-057
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** RELEASING §5c recalibration; MONODROMY record 83 predictions
- **commit_to_build:** 2026-11-19T18:39Z, PR #640 opened (https://github.com/lmmx/mibz2/pull/640)
- **Cost:** i9 12.7 over 1 bundle(s) min [recalibrate-057/run-20261119T185157Z: 12.7 min (started→finished.txt)]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Pinned one CPU a core, lazy 5/20 (0.9340 [0.9264, 0.9417] of eager 5/20's wall); unpinned, the plan's six workers lazy (0.8531 [0.8024, 0.9071] of eager 6/24's)." (https://github.com/lmmx/mibz2/pull/642 (first commit message))

**must_hold** (quoted):

- queue entry bench-recalibrate-057: "worker-count recalibration for 0.5.7 (RELEASING 5c; the 2026-11-18 design re-pinned to master 17405826, the open kernel; #640; MONODROMY record 83 frozen first; about 14 min)"
- "This is the recipe branch for 0.5.7's worker-count gate (RELEASING §3.4 and §5c; #145 QUEUED 7018306990)." (https://github.com/lmmx/mibz2/pull/640 (PR body, current text))
- "Like the earlier recipe PRs it is not for merging: the i9 runs this branch." (https://github.com/lmmx/mibz2/pull/640 (PR body, current text))
- "- **Unchanged:** the cells, the arms, the rule, the rounds, the settle, the corpora, the method and the DRAM diagnostic." (https://github.com/lmmx/mibz2/pull/640 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 7 matching lines in 7 documents dated before commit_to_build, searched on `occ-5|oce-5|lazy (workers )?at 6|6 threads.{0,40}(lazy|workers|trim)` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning.md (doc): "- **Run-queue wait:** 0.56 threads at 2 (two workers, the scanner and the reader on two" — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning.md
- [2026-11-17T08:48Z] docs/journal/monodromy/2026-11-17-workers-low-threads.md (doc): "1. at 6 threads, 4 workers beats 5 by more than 2%; or" — https://github.com/lmmx/mibz2/blob/abf03d26eaa7/docs/journal/monodromy/2026-11-17-workers-low-threads.md
- [2026-11-17T08:50Z] comment 6943291920 (#145 monodromy): "1. at 6 threads, 4 workers beats 5 by more than 2%; or" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6943291920
- [2026-11-17T10:37Z] comment 6945639925 (#145 coordinator): "- **At 6 threads,** the lazy kernel alone costs 4% (the 6/24 control is 1.040), yet dropping to 5 lazy workers wins 0.963 overall. Taking both together, the count's own share is about 0.926." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6945639925
- [2026-11-17T10:40Z] comment 6945704457 (#145 monodromy): "**The flag: an untested combination.** The rule picks 5 at 6 threads, and `Plan::Workers(5)` runs **eager** today, which has never been measured. The measured winner is lazy 5 (0.962)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6945704457
- [2026-11-17T11:53Z] comment 6947274621 (#145 coordinator): "Pairs, reported and not judged: eager 5 over lazy 5 is 0.964 [0.953, 0.975] at 6 threads; eager 7 over lazy 7 is **1.091** [1.082, 1.099] at 10." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6947274621
- [2026-11-18T12:42Z] comment 6978881191 (#145 coordinator): "- *6 threads, lazy 6/24 against the reference eager 6/24:* 0.940 [0.923, 0.957] FASTER, but peak RSS on Wikidata is 1.207 (181.5 against 150.4 MiB)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6978881191

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-7018657158 (2026-11-19T18:46Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #640, #642, bench-recalibrate-057)

**monodromy_involved:** yes: cited in the case's own documents (records 83); origin field names MONODROMY

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/162#issuecomment-7018475871, https://github.com/lmmx/mibz2/pull/162#issuecomment-7019619997

**Notes:** PR #640's earliest listed commit is 2026-11-18T11:35Z, 31 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## recalibrate-055: Worker-count recalibration after wide admission (0.5.5)

- **PRs:** [#562](https://github.com/lmmx/mibz2/pull/562)
- **Journal sources:** none listed
- **i9 recipes:** recalibrate
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** RELEASING §5c; MONODROMY record 56 predictions
- **commit_to_build:** 2026-11-18T11:41Z, PR #562 opened (https://github.com/lmmx/mibz2/pull/562)
- **Cost:** i9 13.0 over 1 bundle(s) min [recalibrate/run-20261118T122744Z: 13.0 min (started→finished.txt)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "That covers the predicted 2.1% and 1.8% gains. ## What it adds - **`examples/ab.rs`:** the occupancy arms (`mibz2-occ-N-S`, `mibz2-oce-N-S`) and `ab run --unpinned`, unchanged from `bench/unpinned`." (https://github.com/lmmx/mibz2/pull/562 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-recalibrate: "worker-count recalibration after wide admission (#562; RELEASING §5c; adoption registration docs/journal/2026-11-18-recalibration-registration.md, Researcher REVIEW 6978182129 targeted correction appl"
- "Every worker trim the default makes on the i9 was measured on the narrow kernel, so each is measured again here. ## The run:" (https://github.com/lmmx/mibz2/pull/562 (PR body, current text))
- "- **Rounds:** 12 paired rounds, with a 0.25 s settle gate." (https://github.com/lmmx/mibz2/pull/562 (PR body, current text))
- "About 13½ minutes on the i9 in all." (https://github.com/lmmx/mibz2/pull/562 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-18T11:41Z] "- **MONODROMY's predictions:** record 56 (#497 at 131c8a54), cited there." (https://github.com/lmmx/mibz2/pull/562 (PR body, current text))
- [2026-11-18T11:41Z] "*Unpinned, one worker fewer runs at 10 alone, and one worker more at 10, 12, 16 and 20.* Unpinned, the arms are the same decoders whatever the count, so 8/32 against 7/28 (the predicted change) is four paired replicates, as the Coordinator asked for the power there." (https://github.com/lmmx/mibz2/pull/562 (PR body, current text))
- [2026-11-18T11:41Z] "That covers the predicted 2.1% and 1.8% gains. ## What it adds - **`examples/ab.rs`:** the occupancy arms (`mibz2-occ-N-S`, `mibz2-oce-N-S`) and `ab run --unpinned`, unchanged from `bench/unpinned`." (https://github.com/lmmx/mibz2/pull/562 (PR body, current text))

**same_quantity_facts:** 6 matching lines in 5 documents dated before commit_to_build, searched on `recalibrat` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T10:37Z] docs/journal/2026-11-10-service-guarded-target-gated-registration.md (doc): "- **Recalibrating against the slow class** drives power at 1.005 to near 0. Neither more rounds" — https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-10-service-guarded-target-gated-registration.md
- [2026-11-11T10:27Z] comment 6793387232 (#145 louis via chatgpt): "The numerical 2×/3×/5× inversion budgets are useful illustrations, not new acceptance gates; the historical stage figures and 70% inversion share should be recalibrated against the current inclusive decoder path. #367's whole-decoder attribution investigation remains relevant." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6793387232
- [2026-11-18T10:14Z] comment 6975512468 (#145 coordinator): "4. **The knee, reported only.** Within the wide arm, 8 threads over 10 is **0.979** [0.963, 0.995]: 8 workers now beat 7. Master's figure is 1.026. The frozen forecast, ≤ 1 within [0.96, 1.03], holds; the Researcher's 1.0134 doesn't. Cheaper bytes per worker moved the knee, as records 45–53 predict…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6975512468
- [2026-11-18T11:16Z] comment 6976975591 (#145 coordinator): "5. **Owed:** the worker-count recalibration under the new kernel (RELEASING §5c, proposed in #560). 8 workers beat 7 within the wide arm (0.979). It goes to the Pathfinder once its queue is reconciled." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6976975591
- [2026-11-18T11:21Z] comment 6977086376 (#314 coordinator): "**The assignment:** recalibrate the worker trims for 0.5.5's wide admission (#559). Within the wide arm, 8 workers beat 7 (0.979), where the shipped trim uses 7 from 10 threads." — https://github.com/lmmx/mibz2/pull/314#issuecomment-6977086376

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6978182129 (2026-11-18T12:11Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #562, bench-recalibrate)

**monodromy_involved:** yes: cited in the case's own documents (records 56); records naming its PRs or recipes: 2026-11-19-recalibration-057-predictions.md, 2026-11-21-recalibration-058-predictions.md, 2026-11-21-recalibration-diet-predictions.md, 2026-11-22-recalibration-plateau-predictions.md, 2026-11-22-recalibration-stack-predictions.md; origin field names MONODROMY

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6978711740, https://github.com/lmmx/mibz2/pull/162#issuecomment-6978023732

**Notes:** PR #562's earliest listed commit is 2026-11-18T11:35Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## sustain: Worker counts under sustained (clamped) load

- **PRs:** [#576](https://github.com/lmmx/mibz2/pull/576)
- **Journal sources:** none listed
- **i9 recipes:** sustain
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Coordinator; MONODROMY record 58/64 predictions
- **commit_to_build:** 2026-11-18T16:20Z, PR #576 opened (https://github.com/lmmx/mibz2/pull/576)
- **Cost:** i9 33.7 over 1 bundle(s) min [sustain/run-20261118T170613Z: 33.7 min (started→finished.txt)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.06; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **Said before the data:** 8/32 is predicted at about 0.97, with 8–16% more RSS seen before." (https://github.com/lmmx/mibz2/pull/576 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-sustain: "worker counts under sustained (clamped) load (#576; adoption registration docs/journal/2026-11-18-sustain-registration.md, Researcher REVIEW 6985154079 targeted correction applied at 3b8aa41d; MONODRO"
- "Record 64 predicts that when clamped, 8 workers are best at 10 threads and up, pinned and unpinned." (https://github.com/lmmx/mibz2/pull/576 (PR body, current text))
- "- *Each cell* starts with the 2 s quiet gate, an idle baseline and 60 s of pre-warm on the reference." (https://github.com/lmmx/mibz2/pull/576 (PR body, current text))
- "- **A probe first,** deciding nothing: 10/40 unpinned for 120 s after a 30 s rest, showing the onset and steady level." (https://github.com/lmmx/mibz2/pull/576 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-18T16:20Z] "- **MONODROMY's predictions:** record 64 (e9559bd0 on #497;" (https://github.com/lmmx/mibz2/pull/576 (PR body, current text))
- [2026-11-18T16:20Z] "Record 64 predicts that when clamped, 8 workers are best at 10 threads and up, pinned and unpinned." (https://github.com/lmmx/mibz2/pull/576 (PR body, current text))
- [2026-11-18T16:20Z] "- **Said before the data:** 8/32 is predicted at about 0.97, with 8–16% more RSS seen before." (https://github.com/lmmx/mibz2/pull/576 (PR body, current text))

**same_quantity_facts:** 36 matching lines in 25 documents dated before commit_to_build, searched on `sustained|clamped|thermal|worker counts?` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-17T09:21Z] docs/journal/2026-11-17-worker-count-calibration.md (doc): "Today's plan hard-codes the measured i9: a profile check (10 physical cores, a 20 MiB L3) gates the trim past the cores. The aim, following Louis on 17 November, is a plan whose worker count comes from a model, with the model's inputs read from the machine at runtime. That keeps today's gains, and …" — https://github.com/lmmx/mibz2/blob/d17466bcbf71/docs/journal/2026-11-17-worker-count-calibration.md
- [2026-11-17T09:52Z] comment 6944610173 (#145 coordinator): "**What it says:** W12's gain is a contention gain. It pays while 9 workers crowd the L3. Once the plan cuts to 7, LLC misses still fall (0.93), but they no longer cost wall time, and the extra instructions (+1.6%) cancel what's left. This is the footprint and contention picture of MONODROMY's recor…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6944610173
- [2026-11-17T16:15Z] comment 6953772428 (#145 researcher): "5. **Two small factual repairs:** “every worker count at each thread count” should be a sweep of selected candidates; label 11.5% as the initial pinned result because the later 0.873 is 12.7%. For the notes' specific MLP/miss-versus-stall claim, cite Qureshi, Lynch, Mutlu and Patt, [A Case for MLP-…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6953772428
- [2026-11-18T11:24Z] docs/journal/monodromy/2026-11-18-recalibration-predictions.md (doc): "1.214 measured), so it overvalues small worker counts' contention relief. Treat it as weak." — https://github.com/lmmx/mibz2/blob/131c8a548a05/docs/journal/monodromy/2026-11-18-recalibration-predictions.md
- [2026-11-18T12:42Z] comment 6978881191 (#145 coordinator): "- **Hypothesis, unconfirmed:** a package power or thermal limit on the i9. With all ten cores busy and the 0.25 s settle gate (the registration's choice, for time), the chip may drop frequency in cycles. The admit run at 10 threads, with the 2 s gate, had ±1.4% intervals." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6978881191
- [2026-11-18T12:43Z] comment 6978889765 (#346 coordinator): "2. **The bimodal regime:** in every cell with about ten cores busy (unpinned from 8 threads, pinned at 10), each arm's walls split about 0.33 / 0.47 s, CPU time rises with wall, and slow runs come in consecutive rounds. Does your worker model (records 43–53) have a term that would produce this, or …" — https://github.com/lmmx/mibz2/issues/346#issuecomment-6978889765
- [2026-11-18T12:46Z] comment 6978967731 (#145 monodromy): "- **Your package-power hypothesis is supported,** with a refinement: the trigger is cumulative recent load. That fits a PL1/PL2 budget on a moving power average with a time constant of tens of seconds; thermal is also possible. The 0.25 s gate keeps the average high, and the 2 s gate lets it recove…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6978967731
- [2026-11-18T16:11Z] comment 6984235551 (#346 coordinator): "**The rule:** #506's family, with Holm per placement at α 0.025, the SLOWER screen on text and random, and rule 4 with a single-corpus trade. The sustained state decides." — https://github.com/lmmx/mibz2/issues/346#issuecomment-6984235551
- [2026-11-18T16:14Z] docs/journal/monodromy/2026-11-18-sustained-predictions.md (doc): "throughput, about 15 GB/s at 7 clamped." — https://github.com/lmmx/mibz2/blob/e9559bd0c34a/docs/journal/monodromy/2026-11-18-sustained-predictions.md
- [2026-11-18T16:15Z] comment 6984343491 (#145 monodromy): "- steady at about 1.40× below turbo, within ±3% if the limit is PL1-type, or drifting down if it is thermal." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6984343491

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6985154079 (2026-11-18T16:45Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #576, bench-sustain)

**monodromy_involved:** yes: cited in the case's own documents (records 56, 58, 64); records naming its PRs or recipes: 2026-11-18-astra-d-verification.md, 2026-11-18-open-return-scoring.md, 2026-11-18-sustained-scoring.md; origin field names MONODROMY

**coordinator_involved:** yes: origin field names the Coordinator; Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/162#issuecomment-6984984150

**Notes:** PR #576's earliest listed commit is 2026-11-18T11:35Z, 5 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## recalibrate-later: Worker-count recalibrations for 0.5.8, 0.5.10, 0.5.12 and 0.5.13

- **PRs:** [#671](https://github.com/lmmx/mibz2/pull/671), [#695](https://github.com/lmmx/mibz2/pull/695), [#722](https://github.com/lmmx/mibz2/pull/722), [#749](https://github.com/lmmx/mibz2/pull/749)
- **Journal sources:** none listed
- **i9 recipes:** recalibrate-058, recalibrate-diet, recalibrate-stack, recalibrate-plateau
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** RELEASING §5c; MONODROMY records 83, 87-89, 91 predictions
- **commit_to_build:** 2026-11-21T01:16Z, PR #671 opened (https://github.com/lmmx/mibz2/pull/671)
- **Cost:** i9 84.0 over 4 bundle(s) min [recalibrate-058/run-20261121T020723Z: 12.5 min (started→finished.txt); recalibrate-diet/run-20261121T124719Z: 12.6 min (started→finished.txt); recalibrate-stack/run-20261122T030603Z: 12.5 min (started→finished.txt); recalibrate-plateau/run-20261122T200518Z: 46.4 min (started→finished.txt)]; CI: 6 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 1.82; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "**Why.** #668 (70b18ca4) changes the open kernel's per-block work and queue layout: about 18% less kernel time natively, and 0.947 wall at 10 threads on the i9." (https://github.com/lmmx/mibz2/pull/671 (PR body, current text))
- "**Why.** #692 (23e40f77), the Open kernel's memory diet, changes a worker's working set and scratch layout: the kernel's persistent scratch fell from 4,236 to 1,200 KiB natively (0.283×), and on the i9 the wall was 0.952 / 0.917 / 0.931 at 6 / 8 / 10 threads unpinned, at 0.89× peak RSS." (https://github.com/lmmx/mibz2/pull/695 (PR body, current text))
- "On the i9 (open-stack/run-20261122T022924Z) the wall was 0.952 / 0.987 / 0.975 at 6 / 8 / 10 threads unpinned, and the retained scratch fell about 19% natively." (https://github.com/lmmx/mibz2/pull/722 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-recalibrate-058: "worker-count recalibration for 0.5.8 (RELEASING 5c; the 2026-11-18 design re-pinned to master 70b18ca4, #668 the open kernel's Vec16 queues; 6-thread references lazy from #642; registration docs/journ"
- queue entry bench-recalibrate-diet: "worker-count recalibration for the release after 0.5.9 (RELEASING 5c; the 2026-11-18 design re-pinned to master 23e40f77, #692 the Open kernel's memory diet, per-worker scratch 0.283x; unpinned 4-thre"
- queue entry bench-recalibrate-stack: "worker-count recalibration for the release after 0.5.11 (RELEASING 5c; the 2026-11-18 design re-pinned to master 53f7ecf9, #719 the Open kernel's construction stack, retained scratch about -19%; cells"
- queue entry bench-recalibrate-plateau: "worker-count recalibration for 0.5.13 (RELEASING 5c), pinned at master 4a79f063 (#739 Wormhole's plateau stack + #745 Carbon's block reader); the 2026-11-18 design with the acting Researcher's correct"

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-21T01:16Z] "It gives the trim each count is judged against, and it covers the model step (§5c.3)." (https://github.com/lmmx/mibz2/pull/671 (PR body, current text))
- [2026-11-21T01:16Z] "MONODROMY's model script for 0.5.7, record 83's `recal057.py`, is on its branch `claude/sharp-meitner-cx0v78`, not on master." (https://github.com/lmmx/mibz2/pull/671 (PR body, current text))
- [2026-11-21T12:32Z] "It gives the trim each count is judged against, and a hypothesis section that decides nothing:" (https://github.com/lmmx/mibz2/pull/695 (PR body, current text))
- [2026-11-21T12:32Z] "The exception is pinned 4's lazy 3/12 arm, at 1.033 in 058, which a 5% lazy gain would bring near the line." (https://github.com/lmmx/mibz2/pull/695 (PR body, current text))
- [2026-11-21T12:32Z] "It also covers the model step (§5c.3)." (https://github.com/lmmx/mibz2/pull/695 (PR body, current text))
- [2026-11-21T12:32Z] "MONODROMY's record 87 (058's predictions) is on its branch `claude/sharp-meitner-cx0v78`, not on master." (https://github.com/lmmx/mibz2/pull/695 (PR body, current text))

**same_quantity_facts:** 31 matching lines in 25 documents dated before commit_to_build, searched on `recalibrat` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T17:50Z] docs/journal/monodromy/2026-11-18-sustained-scoring.md (doc): "1. **The same gate.** The recalibration's `ab run` sampled a 0.25 s settle before every run. Its" — https://github.com/lmmx/mibz2/blob/633fc596f5a4/docs/journal/monodromy/2026-11-18-sustained-scoring.md
- [2026-11-18T17:51Z] comment 6986825354 (#145 monodromy): "1. **The same gate.** `ab run` takes a 0.25 s settle before every run. The recalibration held a spinner on a free core only for pinned 4, 6 and 8 (`--hold-max-threads 9`), so pinned 10 and every unpinned cell had none." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6986825354
- [2026-11-18T17:52Z] comment 6986860998 (#145 coordinator): "Accepted. The bimodal walls are a fixed start-up penalty of about +0.13 s per heavy decode after `ab run`'s 0.25 s settle gap, with no clock clamp (4.9 GHz at exit). Record 58's history and power readings are both superseded. The recalibration's unresolved 10+ thread cells are gate noise, and the s…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6986860998
- [2026-11-18T20:26Z] comment 6990632934 (#222 coordinator): "5. **The settle-gap finding** (MONODROMY record 68 §4): the recalibration's slow runs were a fixed start-up penalty after a 0.25 s idle gap, not a clock state. The new rule is that no timed decode follows a short gap." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6990632934
- [2026-11-18T20:26Z] comment 6990634441 (#145 pathfinder): "- *Why:* the recalibration held lazy 8/32 there on memory (0.963–0.974 wall for 13–14% more RSS)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6990634441
- [2026-11-18T20:32Z] comment 6990784273 (#145 observer): "3. 0.5.5 wide lazy admission gained5–13% at8–20t on tag scaling with accepted Wikidata RSS growth. Worker recalibration changed no default; short-gap startup penalty, not power clamp, explained slow cells. IMC seven-worker plateau75% of ceiling, H1 INCONCLUSIVE." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6990784273
- [2026-11-18T23:16Z] comment 6994466885 (#145 monodromy): "**3. The VM: the reference is almost the whole story.** On the i9, lazy is **1.205× eager** at pinned 3/12 and **1.166×** at unpinned 4/16 (recalibration medians), 1.03 at pinned 5/20, and the reverse at 7 workers (eager 1.27–1.30× slower)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6994466885
- [2026-11-19T18:37Z] docs/journal/monodromy/2026-11-19-recalibration-057-predictions.md (doc): "3. At pinned 8, 7 is slower than 6 by more than 1%. This refutes the recalibrated budget term," — https://github.com/lmmx/mibz2/blob/4040b3a1483f/docs/journal/monodromy/2026-11-19-recalibration-057-predictions.md
- [2026-11-19T18:37Z] comment 7018425237 (#145 monodromy): "- *What else changed:* record 56's pinned budget term is **recalibrated** on 0.5.5's crossover. Pinned 8 at 0.983 was a miss for record 56, which predicted 1.10. 10 workers over 7 measured 1.047." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7018425237
- [2026-11-19T18:39Z] comment 7018464235 (#145 pathfinder): "7. **The DRY preflight passes** (a new untimed mode, the 6th positional: `just bench-recalibrate-057 4 12 24 0.25 5 1`, at fe8600c6):" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7018464235

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-7084586885 (2026-11-22T19:40Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #671, #695, #722, #749, bench-recalibrate-058, bench-recalibrate-diet, bench-recalibrate-stack, bench-recalibrate-plateau)

**monodromy_involved:** yes: cited in the case's own documents (records 83, 87, 88, 91); records naming its PRs or recipes: 2026-11-21-recalibration-058-predictions.md, 2026-11-21-recalibration-diet-predictions.md, 2026-11-22-recalibration-plateau-predictions.md, 2026-11-22-recalibration-stack-predictions.md; origin field names MONODROMY

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/issues/346#issuecomment-7055160250

**Notes:** 4 i9 bundles over 4 recipe(s); coders check whether master moved between them. PR #671's earliest listed commit is 2026-11-18T11:35Z, 62 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #695's earliest listed commit is 2026-11-18T11:35Z, 73 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #722's earliest listed commit is 2026-11-18T11:35Z, 87 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #749's earliest listed commit is 2026-11-18T11:35Z, 94 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## nspc: NSPC: non-splitting affine contraction (Neon)

- **PRs:** [#652](https://github.com/lmmx/mibz2/pull/652)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native (agent sandbox)
- **Origin (as recorded):** Neon (Gemini); prediction from Python op counts (fidelity ledger)
- **commit_to_build:** 2026-11-20T01:50Z, PR #652 first commit (author date) (https://github.com/lmmx/mibz2/pull/652/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.44; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Every configuration (M 32/64/128, K 1/4/8, tau 1/2/4, and the 8-byte M64 K4 record) is above 1.0x OpenCells on all six blocks; the best is 1.82-2.38x." (https://github.com/lmmx/mibz2/pull/652 (first commit message))
- "- **The write-up:** `docs/journal/2026-11-20-nspc-prototype.md`, with the per-configuration CSV beside it. ## The rule (fixed before the data) - **PROMISING:** one (M, K, τ) is byte-exact on all six blocks, and is at most 0.95× Open on at least 4 of 6 blocks, at least one of them 143, 207 or 210, w…" (https://github.com/lmmx/mibz2/pull/652 (PR body, current text))
- "- **NOT WORTH IT:** every configuration is at least 1.0× Open on 4 or more blocks." (https://github.com/lmmx/mibz2/pull/652 (PR body, current text))

**must_hold** (quoted):

- "- **The write-up:** `docs/journal/2026-11-20-nspc-prototype.md`, with the per-configuration CSV beside it. ## The rule (fixed before the data) - **PROMISING:** one (M, K, τ) is byte-exact on all six blocks, and is at most 0.95× Open on at least 4 of 6 blocks, at least one of them 143, 207 or 210, w…" (https://github.com/lmmx/mibz2/pull/652 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-20T01:50Z] "The walk alone costs about Open's whole kernel, and construction about as much again." (https://github.com/lmmx/mibz2/pull/652 (first commit message))

**same_quantity_facts:** 5 matching lines in 4 documents dated before commit_to_build, searched on `NSPC|affine contraction|dependent steps` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T10:32Z] comment 6765072254 (#145 pathfinder): "**4. Frontier, kept apart from the release.** `docs/journal/2026-11-10-interval-exchange.md`, `examples/iet_census.rs`. LF is an exchange of r intervals; an interval of rows moves as one point; a tower of first-return maps rebuilds the block from pieces. Every block rebuilt and checked. Dependent s…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6765072254
- [2026-11-10T10:40Z] comment 6765228464 (#346 coordinator): "**One recent item in your territory:** the Pathfinder's `docs/journal/2026-11-10-interval-exchange.md` (on PR #313's branch). It treats the inverse BWT as a tower of first-return maps of an interval exchange: 0.190 n dependent steps on Wikidata, 0.811 n on text, 0.0024 n on Thue–Morse. Counted, not…" — https://github.com/lmmx/mibz2/issues/346#issuecomment-6765228464
- [2026-11-14T15:29Z] docs/journal/monodromy/2026-11-10-rauzy-veech.md (doc): "- Against the record: the tower 8 × 3 is 0.190 n dependent steps on its 20 Wikidata blocks" — https://github.com/lmmx/mibz2/blob/7686bc3559b6/docs/journal/monodromy/2026-11-10-rauzy-veech.md
- [2026-11-19T20:31Z] comment 7021408706 (#145 monodromy): "- *Text order from I₀:* only two measured edges reach it. Rauzy–Veech induction (S = 1.68–1.87 r dependent steps), or walking the arcs." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7021408706

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #652) / Optimiser: unrecorded (no comment from the role naming #652)

**monodromy_involved:** yes: cited in the case's own documents (records 78, 79, 80, 81, 82, 84)

**coordinator_involved:** no citation found

## open-phase-split: Open's construction phase split (i16 vs qi16)

- **PRs:** [#653](https://github.com/lmmx/mibz2/pull/653)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native
- **Origin (as recorded):** Neon; Neon's C cache model (fidelity ledger)
- **commit_to_build:** 2026-11-20T02:07Z, PR #653 first commit (author date) (https://github.com/lmmx/mibz2/pull/653/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.47; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- The **queued wiring**, counted whole, is **23-28%** (0.37-0.87 ms)." (https://github.com/lmmx/mibz2/pull/653 (PR body, current text))
- "- The queue fill is 78-87% of it: the per-run `Vec::extend` costs about 22-28 Ir per retained row, and thin runs average 1.6-1.9 rows." (https://github.com/lmmx/mibz2/pull/653 (PR body, current text))

**must_hold** (quoted):

- "- The elimination (`eliminate`, rules included) is 5.5-12.1%." (https://github.com/lmmx/mibz2/pull/653 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-20T02:07Z] "- The queue fill is 78-87% of it: the per-run `Vec::extend` costs about 22-28 Ir per retained row, and thin runs average 1.6-1.9 rows." (https://github.com/lmmx/mibz2/pull/653 (PR body, current text))
- [2026-11-20T02:07Z] "It saves 0.55-1.23 MB of live buffers." (https://github.com/lmmx/mibz2/pull/653 (PR body, current text))
- [2026-11-20T02:07Z] "Its ceiling is about 18-25% of the total." (https://github.com/lmmx/mibz2/pull/653 (PR body, current text))

**same_quantity_facts:** 193 matching lines in 148 documents dated before commit_to_build, searched on `i16|qi16|phase split|construction` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T23:16Z] comment 6994466885 (#145 monodromy): "- §2's construction-offset correction: a kernel effect of about 0.95, against the sweep's kernel-only 0.934–0.953 at W 7–8;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6994466885
- [2026-11-19T02:22Z] comment 6997724058 (#145 coordinator): "One way to read the 2.36× result. It may already be on your path, given your question about which construction costs exact token ordering requires." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6997724058
- [2026-11-19T08:29Z] comment 7003886829 (#145 researcher): "- **Cost.** Brown–Langmead (SEA 2026, Lemma 6 and Theorem 9) give O(r) LF-table construction and O(n) inversion in O(r) workspace. Movi measures cache locality per step. I found no O(r) unpredictable-access bound for acquiring Astra's weighted returns and marked event order. The k=8 native band acq…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7003886829
- [2026-11-19T09:54Z] docs/journal/2026-11-19-gemini-neon/prompt.md (doc): "For perspective, prepared sparse-exception emission has been measured at approximately 0.628× Scalar-cap's inverse-stage CPU, while acquiring the required events in the current eight-byte-context approach costs approximately 2.35×. If these costs are additive, acquisition would need to fall below r…" — https://github.com/lmmx/mibz2/blob/a52b21a31a93/docs/journal/2026-11-19-gemini-neon/prompt.md
- [2026-11-19T11:37Z] docs/journal/2026-11-19-gemini-neon/response-5.md (doc): "* If fragmentation is low, `RV-Copy` eliminates SLP construction, slot recycling, and induction-width memory entirely, leaving only $1.015\,r$ integer RV steps + $O(r)$ radix sort + $O(k)$ `memcpy`s." — https://github.com/lmmx/mibz2/blob/5e111cfa781c/docs/journal/2026-11-19-gemini-neon/response-5.md
- [2026-11-19T11:58Z] docs/journal/2026-11-19-gemini-neon/response-6.md (doc): "\| Structural construction \| O(n) scan \to O(r) run records \| O(n) scan + \approx 1.015\,r RV winner-runs (builds SLP) \| O(n) scan + \approx 1.015\,r integer RV + O(r) sort + O(k) G_{\mathrm{copy}} & Tarjan SCC \|" — https://github.com/lmmx/mibz2/blob/3addea7226e4/docs/journal/2026-11-19-gemini-neon/response-6.md
- [2026-11-19T14:24Z] comment 7011906285 (#145 pathfinder): "- **Its false-fail rate is ≤ 0.048 by construction.** With no effect the arms are exchangeable each round, so the test is exact, and Bonferroni holds under any dependence. No noise model is assumed. With the memory guard, at most 0.0015 by resampling, the whole guard family's rate is **≤ 0.05**." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7011906285
- [2026-11-19T14:25Z] comment 7011926888 (#162 coordinator): "- **False-fail rate:** at most 0.048 by construction." — https://github.com/lmmx/mibz2/pull/162#issuecomment-7011926888
- [2026-11-19T16:32Z] comment 7015339412 (#145 pathfinder): "- **Most of B2's rate is G's SLOWER test, and rounds don't reduce it:** 2.5% per cell by construction, so about 0.26 across 12 cells under normal noise. The 32 rounds cut only the point bound's share." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7015339412
- [2026-11-20T01:52Z] comment 7027879220 (#145 coordinator): "- **Construction adds another 0.94–1.21× Open.**" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7027879220

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #653) / Optimiser: unrecorded (no comment from the role naming #653)

**monodromy_involved:** yes: cited in the case's own documents (records 78, 79, 80, 81, 82, 84)

**coordinator_involved:** no citation found

## open-packed-queues: Open fused wiring and packed queues (Q)

- **PRs:** [#654](https://github.com/lmmx/mibz2/pull/654)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native
- **Origin (as recorded):** Neon (estimate 8-15% faster, fidelity ledger)
- **commit_to_build:** 2026-11-20T02:43Z, PR #654 first commit (author date) (https://github.com/lmmx/mibz2/pull/654/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.41; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "F 1.02-1.04x, Q 1.05-1.09x, Qi 1.09-1.12x on every block." (https://github.com/lmmx/mibz2/pull/654 (first commit message))

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-20T02:44Z] "- **Scratch capacity:** Q's queues are 42-52% smaller." (https://github.com/lmmx/mibz2/pull/654 (PR body, current text))
- [2026-11-20T02:44Z] "This is a hypothesis-level figure; no RSS change is claimed." (https://github.com/lmmx/mibz2/pull/654 (PR body, current text))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `packed queues?|fused wiring|queue compaction` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #654) / Optimiser: unrecorded (no comment from the role naming #654)

**monodromy_involved:** yes: cited in the case's own documents (records 78, 79, 80, 81, 82, 84)

**coordinator_involved:** no citation found

## open-limit8: Open thin-run LIMIT=8, per-block preference

- **PRs:** [#657](https://github.com/lmmx/mibz2/pull/657)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native
- **Origin (as recorded):** per PR #657; prediction from 2-lap medians (fidelity ledger)
- **commit_to_build:** 2026-11-20T09:25Z, PR #657 first commit (author date) (https://github.com/lmmx/mibz2/pull/657/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.13; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "On the pinned Wikidata sample (228 blocks) the honest per-block oracle saves 0.8% (total-weighted 0.992, over the brief's 0.98 line) and no fixed limit beats 16 (8: 1.072, 12: 1.018, 24: 1.006), so the rule stops:" (https://github.com/lmmx/mibz2/pull/657 (first commit message))
- "A 2-round sign correlates only 0.24–0.38 with the same block's other 38 rounds, so the earlier "70 blocks" were noise or came from the non-queued arm." (https://github.com/lmmx/mibz2/pull/657 (PR body, current text))

**must_hold** (quoted):

- "On the pinned Wikidata sample (228 blocks) the honest per-block oracle saves 0.8% (total-weighted 0.992, over the brief's 0.98 line) and no fixed limit beats 16 (8: 1.072, 12: 1.018, 24: 1.006), so the rule stops:" (https://github.com/lmmx/mibz2/pull/657 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-20T09:25Z] "On the pinned Wikidata sample (228 blocks) the honest per-block oracle saves 0.8% (total-weighted 0.992, over the brief's 0.98 line) and no fixed limit beats 16 (8: 1.072, 12: 1.018, 24: 1.006), so the rule stops:" (https://github.com/lmmx/mibz2/pull/657 (first commit message))
- [2026-11-20T09:25Z] "Step 3, the predictor, did not run. ## Method - **Harness:** `ab limit-sweep 40 wikidata.bz2` (feature `open-return`)." (https://github.com/lmmx/mibz2/pull/657 (PR body, current text))

**same_quantity_facts:** 123 matching lines in 112 documents dated before commit_to_build, searched on `LIMIT|thin.run|per.block preference` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T22:44Z] docs/journal/astra-a-reinvention/README.md (doc): "**Result.** Two consecutive completed output spans can teach the inverse how to emit their concatenation and where to resume. Keeping each original run image intact gives a particularly small dictionary. When a prospective extension crosses its next domain boundary, that word can be retired permane…" — https://github.com/lmmx/mibz2/blob/fc5f294c9103/docs/journal/astra-a-reinvention/README.md
- [2026-11-19T00:49Z] comment 6996175505 (#145 astra): "The four arms are LazyCells, opend, ungated Open and the combined candidate. Each uses equal W/4W at W=7,8,9,10, unpinned on the 20 logical CPUs, Wikidata ×4, 16 complete forward/reverse rotated rounds and 2 s settle. Every timed output must have the exact length. The preregistered primary requires…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6996175505
- [2026-11-19T02:22Z] comment 6997724058 (#145 coordinator): "- **The bound D ≥ ⌈N / max f⌉** limits explicit byte-spelling dictionaries only. A token-level spelling isn't subject to it, and avoids the 41 → 126 MB dictionary growth that would also break the moonshot's no-extra-peak-memory rule." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6997724058
- [2026-11-19T05:52Z] comment 7001022850 (#145 lmmx): "The five-arm target rotor compiles combined `50b8ecb`, scalar `d0a4298` and edge `badb2ae` separately. Edge supplies the 0.5.6 LazyCells and opend references. It uses 40 balanced rounds at W7/W8/W9/W10, four copies of the pinned Wikidata corpus per process, 4W slots and two-second settles, for 800 …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7001022850
- [2026-11-19T09:54Z] docs/journal/2026-11-19-gemini-neon/prompt.md (doc): "* First-level delimiter construction alone has previously consumed approximately 97% of the LazyCells inverse-stage CPU on a measured set of blocks." — https://github.com/lmmx/mibz2/blob/a52b21a31a93/docs/journal/2026-11-19-gemini-neon/prompt.md
- [2026-11-19T10:19Z] docs/journal/monodromy/2026-11-19-gemini-neon-round2-verification.md (doc): "**What limits the width.** The footprint condition almost never does: about 99% of rounds take" — https://github.com/lmmx/mibz2/blob/5cbc086113f8/docs/journal/monodromy/2026-11-19-gemini-neon-round2-verification.md
- [2026-11-19T10:20Z] comment 7006279867 (#145 monodromy): "- *What limits it:* the footprints almost never do; 99% of rounds take every live front. The heaviest component (19–25% of cells) sets the rounds. Pinning changes cells by −2.7% to +7.7%, and K = 32 (28–29 distinct pins) is worse than 16." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7006279867
- [2026-11-19T14:46Z] docs/journal/2026-11-19-open-port-registration.md (doc): "- *The 1.02 bound* adds a point-estimate limit to the SLOWER screen, so that a guard regressing" — https://github.com/lmmx/mibz2/blob/ef25d39d3a3a/docs/journal/2026-11-19-open-port-registration.md
- [2026-11-19T15:04Z] comment 7012967570 (#145 researcher): "**S ruling.** The explicit prospective replacement is admissible on its own. At +2%, the locked 96-pair rule gives **0.939** own-cell power versus **0.547** for S as registered, with far fewer modeled null failures. It is weaker at +1% (0.050 versus 0.315). S is the 40th timing cell and still has n…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7012967570
- [2026-11-19T16:21Z] comment 7015030749 (#145 researcher): "**Part A is sound as diagnostic attribution.** I independently reproduced 84 within observed repeat spread, 38 harness-dominated and two runtime/scheduling classifications from the aggregates. The preflight copy matches the pinned summary, and all 36 diagnostic output fingerprints match the manifes…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7015030749

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #657) / Optimiser: unrecorded (no comment from the role naming #657)

**monodromy_involved:** yes: cited in the case's own documents (records 78, 79, 80, 81, 82, 84)

**coordinator_involved:** no citation found

## open-backward: Queue-free backward Open (Neon cycle 2)

- **PRs:** [#659](https://github.com/lmmx/mibz2/pull/659)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native
- **Origin (as recorded):** Neon cycle 2; C cache model (fidelity ledger)
- **commit_to_build:** 2026-11-20T13:35Z, PR #659 first commit (author date) (https://github.com/lmmx/mibz2/pull/659/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Over 61 rotated single-thread rounds, BWD is 1.10-1.15x qi16 on all six blocks (0.94-0.97x i16)." (https://github.com/lmmx/mibz2/pull/659 (first commit message))
- "Instructions are 1.22-1.32x qi16's." (https://github.com/lmmx/mibz2/pull/659 (first commit message))
- "- **Against qi16:** BWD is **1.10–1.15×** on all six blocks." (https://github.com/lmmx/mibz2/pull/659 (PR body, current text))

**must_hold** (quoted):

- "- **Against i16:** it is **0.94–0.97×**. \| block \| BWD/qi16 \| BWD/i16 \| bwdz/bwd \| \|---\|---\|---\|---\| \| 0 \| 1.115 [1.056, 1.176] \| 0.953 \| 1.032 \| \| 2 \| 1.098 [1.034, 1.155] \| 0.965 \| 1.026 \| \| 85 \| 1.105 [0.992, 1.259] \| 0.936 \| 1.054 \| \| 143 \| 1.112 [1.050, 1.200] \| 0.9…" (https://github.com/lmmx/mibz2/pull/659 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-20T13:36Z] "Its C cache model can't rank arms (the cycles 1–2 audit; #653), so this is the native three-arm test:" (https://github.com/lmmx/mibz2/pull/659 (PR body, current text))

**same_quantity_facts:** 2 matching lines in 2 documents dated before commit_to_build, searched on `backward Open|queue.free|second pass over the records` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-20T13:18Z] docs/team/research-charter/FRONTIER.md (doc): "\| Building the retained map \| Open's run pass and queue fill 28–36%; queued wiring 23–28% (#653). Compression (Q) and fusion (F) lost (#654) \| Can the map be produced in the direction its consumer needs, so no inversion scatter is required? Candidate: Queue-Free Backward Open (Neon cycle 2; audi…" — https://github.com/lmmx/mibz2/blob/0106c874c6e2/docs/team/research-charter/FRONTIER.md
- [2026-11-20T13:20Z] docs/team/research-charter/CONTEXT.md (doc): "\| Backward Open (Neon cycle 2) \| work 0.72–0.74×, L2 misses 0.82–0.88× (C cache model) \| native three-arm test running. The same model predicts `i16` beats `qi16` on every metric (L2 0.83–0.89, DRAM 0.72–0.83), but natively `i16` is 1.10–1.16× slower \| no: the model mis-ranks the reference pair…" — https://github.com/lmmx/mibz2/blob/40b2906a8fed/docs/team/research-charter/CONTEXT.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #659) / Optimiser: unrecorded (no comment from the role naming #659)

**monodromy_involved:** yes: cited in the case's own documents (records 78, 79, 80, 81, 82, 84)

**coordinator_involved:** no citation found

## open-vec16: Open fixed-width queue fill, Vec16 queues and 16-byte emit (U16Q / LRB-Vec16 queue part)

- **PRs:** [#664](https://github.com/lmmx/mibz2/pull/664), [#666](https://github.com/lmmx/mibz2/pull/666), [#668](https://github.com/lmmx/mibz2/pull/668), [#669](https://github.com/lmmx/mibz2/pull/669)
- **Journal sources:** docs/journal/2026-11-20-open-u16q.md, docs/journal/2026-11-20-open-stage-attribution.md
- **i9 recipes:** open-vec16
- **Deciding machine:** CI (U16Q), native, then i9-10900K (bench-open-vec16)
- **Origin (as recorded):** Carbon cycle 2 (U16Q, C harness) and Kitchen Sink cycle 3 (LRB-Vec16)
- **commit_to_build:** 2026-11-20T17:23Z, PR #664 opened (https://github.com/lmmx/mibz2/pull/664)
- **Cost:** i9 7.2 over 1 bundle(s) min [open-vec16/run-20261121T004904Z: 7.2 min (started→finished.txt)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.31; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "**A research PR: the routing (`LAZY_FROM_THREADS`, `Plan::lazy`) is unchanged.** ## The finding - **The Vec16 queue pays natively.** With it, Open on the records path is 347–359 ms on the whole corpus at one thread, against 433–441 ms on master and 390–393 ms on #664." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- "- **The LRB fusion does not pay natively.** The fused route costs 408–433 ms, against 384–398 ms for front + Open done separately on the same head." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- "- **Hot, block by block:** repeated on corpus blocks 1, 3, 158, 268, 384 and 387 (best of 25), fused / (front + Open) was 0.94–1.02." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-open-vec16: "the Open kernel's Vec16 queue (#668, research/open-vec16 90ff6bfb) against its merge-base ad65b3b8, unpinned at 6, 8 and 10 threads, 16 rounds, then mibz2-workers-lazy at 6 and 8 (diagnostic); about 2"
- "- `cargo clippy --release --all-targets`: clean. ## Decision rule (fixed before the data) Adopt only if all of these hold:" (https://github.com/lmmx/mibz2/pull/664 (PR body, current text))
- "- CI's paired A/B bench shows an end-to-end figure improving beyond noise (its 95% interval excludes zero), under PERFORMANCE-POLICY.md section 2;" (https://github.com/lmmx/mibz2/pull/664 (PR body, current text))
- "- the gates refuse it." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-20T17:23Z] "## Hypothesis Two branch-misprediction sources in the Open kernel (`src/open.rs`) cost measurable time on the admitted Wikidata blocks: 1." (https://github.com/lmmx/mibz2/pull/664 (PR body, current text))
- [2026-11-20T17:23Z] "A fixed 16-wide iota block, advanced by w, removes that loop. 2." (https://github.com/lmmx/mibz2/pull/664 (PR body, current text))
- [2026-11-20T17:23Z] "Bytes left from an earlier block are not zeroed, because every byte kept is written first." (https://github.com/lmmx/mibz2/pull/664 (PR body, current text))
- [2026-11-20T17:23Z] "Most of the gain is therefore expected from the construction, through the queue fill." (https://github.com/lmmx/mibz2/pull/664 (PR body, current text))
- [2026-11-20T20:14Z] "- **front + pass1:** cut to 0.485–0.537×." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- [2026-11-20T20:14Z] "There are no records, no cut at 127 and no `each_run`." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))

**same_quantity_facts:** 1 matching lines in 1 documents dated before commit_to_build, searched on `U16Q|Vec16|16.wide|fixed.width queue|16.byte emit` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-09T22:00Z] comment 6752605812 (#161 coordinator): "- **How windows are handled:** each 16-row window's run heads are found with one SSE2 compare. A window with at most 6 runs then takes one cursor step and one 16-wide write per run. That removes the store-to-load forwarding chain through `ftab[c]`, which covers 9.6% of the rows on wikidata-independ…" — https://github.com/lmmx/mibz2/pull/161#issuecomment-6752605812

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #664, #666, #668, #669, bench-open-vec16) / Optimiser: unrecorded (no comment from the role naming #664, #666, #668, #669, bench-open-vec16)

**monodromy_involved:** yes: records naming its PRs or recipes: 2026-11-21-recalibration-058-predictions.md, 2026-11-21-recalibration-diet-predictions.md

**coordinator_involved:** no citation found

**Notes:** PR #664's earliest listed commit is 2026-11-20T17:15Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #666's earliest listed commit is 2026-11-20T17:15Z, 3 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #668's earliest listed commit is 2026-11-20T17:15Z, 7 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## lrb-fusion: MTF/RLE2 run fusion with the Open front (LRB, Kitchen Sink cycle 3)

- **PRs:** [#666](https://github.com/lmmx/mibz2/pull/666)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native, one thread, whole corpus
- **Origin (as recorded):** Kitchen Sink cycle 3 (C harness: fusion marginal 0.92-0.96x)
- **commit_to_build:** 2026-11-20T20:14Z, PR #666 opened (https://github.com/lmmx/mibz2/pull/666)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.00; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "**A research PR: the routing (`LAZY_FROM_THREADS`, `Plan::lazy`) is unchanged.** ## The finding - **The Vec16 queue pays natively.** With it, Open on the records path is 347–359 ms on the whole corpus at one thread, against 433–441 ms on master and 390–393 ms on #664." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- "- **The LRB fusion does not pay natively.** The fused route costs 408–433 ms, against 384–398 ms for front + Open done separately on the same head." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- "- **Hot, block by block:** repeated on corpus blocks 1, 3, 158, 268, 384 and 387 (best of 25), fused / (front + Open) was 0.94–1.02." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))

**must_hold** (quoted):

- "- the gates refuse it." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- "- the admission gates: fused exactly where `invert_runs` would open the block;" (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- "- **The six admitted blocks** (`MIBZ2_ADMITTED_BWT`, an ignored test) are byte-exact on all three paths. ## Stage table: one thread, all 431 blocks (ms) The machine is a shared 4-vCPU container (Xeon @ 2.10 GHz), with the run pinned to CPU 2." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- "- **At 4 threads,** the lazy head's CPU is 0.952 [0.922, 0.984] of `mibz2` master." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-20T20:14Z] "- **front + pass1:** cut to 0.485–0.537×." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- [2026-11-20T20:14Z] "There are no records, no cut at 127 and no `each_run`." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- [2026-11-20T20:14Z] "- the admission gates: fused exactly where `invert_runs` would open the block;" (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- [2026-11-20T20:14Z] "- `front_runs`' runs, cut at 127, against `runs::front`'s records on 3,000 blocks." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))
- [2026-11-20T20:14Z] "So CI's standard bench is expected to show nothing unless the routing changes. ## Decision rule Adoption needs an end-to-end gain beyond noise on a path production runs." (https://github.com/lmmx/mibz2/pull/666 (PR body, current text))

**same_quantity_facts:** 65 matching lines in 42 documents dated before commit_to_build, searched on `fusion|RLE2|run fusion|front\+Open` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T14:10Z] comment 6769522898 (#145 researcher): "**Mathematics scoped:** primitive-word/least-rotation tree identity; r₂−1 is the cell-step floor, not an inversion lower bound (fusion can go below it). Step reduction is not another timing result. Full worked aab reduction and Cartesian split replace opaque mechanism prose. Earlier relay1.327/7.7%…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6769522898
- [2026-11-10T17:07Z] comment 6773861678 (#145 coordinator): "- the MTF+RLE2 stage, timed alone, is +3.4% (+2.0% to +4.8%), with the wall unchanged;" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6773861678
- [2026-11-16T10:04Z] docs/journal/2026-11-16-sibling-front-end.md (doc): "on a 3.69 GHz Xeon: MTF and RLE2 take 1.58 c/B, RLE1 0.19 and CRC 0.26 (record)." — https://github.com/lmmx/mibz2/blob/cc2147679e3a/docs/journal/2026-11-16-sibling-front-end.md
- [2026-11-17T10:53Z] docs/journal/monodromy/2026-11-17-miss-cost-scoring.md (doc): "- *Estimate:* the post-walk passes were 6.4–9.3% of 4-thread CPU on CI, and fusion recovers about" — https://github.com/lmmx/mibz2/blob/b60960de9939/docs/journal/monodromy/2026-11-17-miss-cost-scoring.md
- [2026-11-17T12:04Z] comment 6947507581 (#145 coordinator): "4. **No re-run now:** an effect of at most 1.5–3% doesn't move the 8–20-thread plateau. The next useful i9 time goes to `bench-unpinned` and `bench-knee`. The Pathfinder writes the journal entry for this negative result. #535 stays open as the record unless the knee diagnostic points at output-side…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6947507581
- [2026-11-18T14:59Z] docs/journal/2026-11-18-switch-elimination/README.md (doc): "The study uses mibz2 0.5.5 at `f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71`, Rust 1.90.0 and Python 3. The three research kernels are installed into a disposable checkout. Their normal entry point is `mibz2::decode_with`, including the scanner, Huffman, MTF/RLE2, inverse BWT, RLE1 and the block and st…" — https://github.com/lmmx/mibz2/blob/9cd25610131b/docs/journal/2026-11-18-switch-elimination/README.md
- [2026-11-18T16:40Z] docs/journal/monodromy/2026-11-18-mixed-steps-verification.md (doc): "# 2026-11-18 · MONODROMY 66 · #575 (mixed Cells steps) verified independently. Every count reproduces exactly from my own two-order model, on all 233 blocks and the six prose controls. The identity, the limit theorem and the output are exact. The cost ledger is fair and conservative as an operation…" — https://github.com/lmmx/mibz2/blob/2000c5e270bf/docs/journal/monodromy/2026-11-18-mixed-steps-verification.md
- [2026-11-18T16:40Z] comment 6985010973 (#145 monodromy): "- *Coverage:* 62.9% of the 19.26M short words Cells builds are duplicates, and the fusion removes **5.4%** of those duplicates." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6985010973
- [2026-11-20T02:44Z] comment 7028696351 (#145 coordinator): "\| F (fusion of bands and wiring) \| 1.023–1.044 \| The fused loop costs 2–8% more than the stages it replaces. \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7028696351
- [2026-11-20T17:25Z] docs/team/research-charter/FRONTIER.md (doc): "\| Building the retained map \| Open's run pass and queue fill 28–36%; queued wiring 23–28% (#653). Compression (Q) and fusion (F) lost (#654) \| Can the map be produced in the direction its consumer needs, so no inversion scatter is required? Queue-Free Backward Open failed natively (#659, 1.10–1.…" — https://github.com/lmmx/mibz2/blob/3e86ee024448/docs/team/research-charter/FRONTIER.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #666) / Optimiser: unrecorded (no comment from the role naming #666)

**monodromy_involved:** yes: cited in the case's own documents (records 78, 79, 80, 81, 82, 84)

**coordinator_involved:** no citation found

**Notes:** PR #666's earliest listed commit is 2026-11-20T17:15Z, 3 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## flatq: FlatQ and primary split (Carbon cycle 11)

- **PRs:** [#667](https://github.com/lmmx/mibz2/pull/667)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native, one thread
- **Origin (as recorded):** Carbon cycle 11 (C: construct 0.37-0.47x)
- **commit_to_build:** 2026-11-21T00:21Z, PR #667 opened (https://github.com/lmmx/mibz2/pull/667)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 1.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **Hot, in one process:** on the admitted blocks, slab/`Vec` is 0.951–0.992." (https://github.com/lmmx/mibz2/pull/667 (PR body, current text))
- "Wall time against `mibz2-workers-lazy` on master: \| arm \| 1 thread \| 2 threads \| 4 threads \| \|---\|--:\|--:\|--:\| \| `-lazy` head \| 1.017 [0.915, 1.130] \| 0.962 [0.897, 1.032] \| 0.987 [0.922, 1.056] \| \| `-lazy` #666 with fusion off \| 1.018 [0.952, 1.088] \| 0.957 [0.891, 1.028] \| 1.05…" (https://github.com/lmmx/mibz2/pull/667 (PR body, current text))
- "Against the `Vec` queues' 1.5 MiB after the same six blocks, that is **+1.1 MiB a kernel**, or 20+ MiB at 20 workers." (https://github.com/lmmx/mibz2/pull/667 (PR body, current text))

**must_hold** (quoted):

- "Open at 8 or more workers (the i9), or a routing change backed by front+Open below front+Cells natively." (https://github.com/lmmx/mibz2/pull/667 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-21T00:21Z] "**FlatQ overlaps Vec16; it doesn't add to it.** Both remove the growable queue's per-run work. ## ab, whole decoder (indicative, shared 4-vCPU) `--spread`, 10 rounds." (https://github.com/lmmx/mibz2/pull/667 (PR body, current text))

**same_quantity_facts:** 1 matching lines in 1 documents dated before commit_to_build, searched on `FlatQ|primary split|growable queue` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T14:26Z] comment 6981427286 (#145 researcher): "- Prose has **median** 32.34% wide area across six blocks (27.59–48.98%). The t=256 sweep supports the >256 bin, with the primary split caveat. It does not resolve the ≥256 bin." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6981427286

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #667) / Optimiser: unrecorded (no comment from the role naming #667)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

**Notes:** PR #667's earliest listed commit is 2026-11-20T17:15Z, 7 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## open-lanes-r4: Open eight-lane walk into 4 KiB pages (BP4K) with R4 at LIMIT 24

- **PRs:** [#675](https://github.com/lmmx/mibz2/pull/675), [#676](https://github.com/lmmx/mibz2/pull/676)
- **Journal sources:** none listed
- **i9 recipes:** open-lanes
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Carbon cycles 9-11 (C: 0.43-0.57x of Open)
- **commit_to_build:** 2026-11-21T01:35Z, PR #675 first commit (author date) (https://github.com/lmmx/mibz2/pull/675/commits)
- **Cost:** i9 7.5 over 1 bundle(s) min [open-lanes/run-20261121T023127Z: 7.5 min (started→finished.txt)]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.97; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "**By block density:** both's per-block median against #668 runs from 0.906 at r/n 0.06 to 0.852 at r/n > 0.083." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))
- "The construction's elimination plus bands and words are about half of the construction natively, more than Carbon's 7–12% estimate." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))
- "Wall time against `mibz2-workers-lazy` at #668: \| arm \| 1 thread \| 2 threads \| 4 threads \| \|---\|--:\|--:\|--:\| \| `-lazy` head \| 0.914 [0.856, 0.975] \| 0.924 [0.860, 0.993] \| 0.972 [0.907, 1.043] \| \| `mibz2` head (control) \| 0.875 [0.808, 0.948] \| 0.952 [0.897, 1.011] \| 0.993 [0.940…" (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-open-lanes: "the Open kernel's eight-lane walk into 4 KiB pages and R4 at LIMIT 24 (#675, research/open-bp4k 8ec1a92c) against master's src 70b18ca4, unpinned at 6, 8 and 10 threads, 16 rounds, then mibz2-workers-"
- "- *The elimination:* on `prev`-only tapes (16-byte top slots, 8-byte bottom slots) with rule R4's overwrite." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))
- "- *The words:* words of at most 64 bytes are joined in an arena; rules take longer ones." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))
- "- **An unsafe page store:** not needed. ## Decision rule Adoption needs an end-to-end gain on a path production runs: the i9 at 6 threads or more, where Open runs." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-21T01:35Z] "The retained rows are cut at S rows spread from the marked one, each cut's code replaced by a sentinel whose length is LONG, so a segment ends on the 16-byte emit's own length lookup." (https://github.com/lmmx/mibz2/pull/675 (first commit message))
- [2026-11-21T01:35Z] "The cuts are restored before return." (https://github.com/lmmx/mibz2/pull/675 (first commit message))
- [2026-11-21T02:09Z] "- *The cut:* the retained rows are cut into S segments from the marked row." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))
- [2026-11-21T02:09Z] "Each cut's code is replaced by a sentinel with `lens = LONG`, so a segment's end is found by the emit's own length lookup." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))
- [2026-11-21T02:09Z] "- *Restoring and bounding:* the cuts are restored before return." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))
- [2026-11-21T02:09Z] "The construction's elimination plus bands and words are about half of the construction natively, more than Carbon's 7–12% estimate." (https://github.com/lmmx/mibz2/pull/675 (PR body, current text))

**same_quantity_facts:** 54 matching lines in 27 documents dated before commit_to_build, searched on `eight.lane|8.lane|BP4K|R4|4 KiB pages` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T09:29Z] comment 6763823117 (#145 coordinator): "- **Pathfinder's k = 11 is checked and not adopted.** Its 33% saving is against R480; against 12 × 240 it saves 8.3% and costs 4–5 points of power." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6763823117
- [2026-11-10T10:37Z] docs/journal/2026-11-10-service-guarded-target-gated-registration.md (doc): "The "33%" saving is against k 12 × R480 (3546). Almost all of it is the R480 → R240 change" — https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-10-service-guarded-target-gated-registration.md
- [2026-11-14T22:02Z] comment 6884880570 (#145 louis via chatgpt): "For 4 KiB pages, 10–20 MiB represents **2,560–5,120 pages**. Resetting and then densely rewriting the buffer can therefore replace a contiguous clearing operation with thousands of fault-handling events." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884880570
- [2026-11-15T12:24Z] comment 6894274731 (#145 monodromy): "- **R4:** the bottom slot's (λ, w) can be read through the partner field from the top slot, which still holds the letter (378.6 MB)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894274731
- [2026-11-16T18:22Z] docs/journal/monodromy/2026-11-16-stop-and-walk.md (doc): "**Local clocks** (container Xeon; not evidence): pass 1 about 2 to 3 ns a point with eight lanes," — https://github.com/lmmx/mibz2/blob/fa08477f526f/docs/journal/monodromy/2026-11-16-stop-and-walk.md
- [2026-11-16T19:18Z] comment 6926545736 (#145 optimizer): "- **Current-plan split anomaly:** `split-051` PR462 / PR468 at 8 is 0.9802 [0.9712,0.9894]. #468 changes the plan only *past* the cores, so there is no 8-thread plan change to take. The two builds differ, but this cell alone cannot assign the difference to the plan. Confirm before any action." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6926545736
- [2026-11-17T10:04Z] docs/journal/monodromy/2026-11-17-contention-counters.md (doc): "- *Wall:* 0.97 [0.93, 1.00] against 4 KiB pages." — https://github.com/lmmx/mibz2/blob/0e7c6888bcba/docs/journal/monodromy/2026-11-17-contention-counters.md
- [2026-11-17T12:44Z] comment 6948395532 (#145 coordinator): "- The DIMMs' rating is unknown. Dual-channel DDR4-3200 would peak at 51.2 GB/s." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6948395532
- [2026-11-20T18:38Z] docs/journal/2026-11-20-toy-transfer.md (doc): "- **The interaction:** with that change, 8 lanes cut Open's emit to 0.55–0.78×, though either" — https://github.com/lmmx/mibz2/blob/1af2fb335099/docs/journal/2026-11-20-toy-transfer.md
- [2026-11-21T00:22Z] docs/team/research-charter/CONTEXT.md (doc): "\| FlatQ + primary split (Carbon cycle 11; BP4K not ported) \| construct 0.37–0.47×, total 0.44–0.57× of Open in C (against Open with growable queues) \| #667, native, one thread, whole corpus: 0.990× of #666's Vec16 [0.949, 1.033]. It overlaps Vec16, which already removed the growable queue's per-…" — https://github.com/lmmx/mibz2/blob/71c266122637/docs/team/research-charter/CONTEXT.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #675, #676, bench-open-lanes) / Optimiser: unrecorded (no comment from the role naming #675, #676, bench-open-lanes)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## open-r4: Open R4 endpoint elimination alone, serial walk at LIMIT 16

- **PRs:** [#679](https://github.com/lmmx/mibz2/pull/679), [#680](https://github.com/lmmx/mibz2/pull/680)
- **Journal sources:** none listed
- **i9 recipes:** open-r4
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Carbon (R4-DGIM), split out of #675 by the Coordinator
- **commit_to_build:** 2026-11-21T08:58Z, PR #679 first commit (author date) (https://github.com/lmmx/mibz2/pull/679/commits)
- **Cost:** i9 7.1 over 1 bundle(s) min [open-r4/run-20261121T092824Z: 7.1 min (started→finished.txt)]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.66; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Open stage, R4/master: \| round \| all 431 blocks \| 228 admitted \| \|---\|---\|---\| \| 1 \| 0.946 \| 0.944 \| \| 2 \| 1.026 \| 1.030 \| \| 3 \| 0.954 \| 0.948 \| \| 4 \| 0.939 \| 0.937 \| \| 5 \| 0.977 \| 0.974 \| \| 6 \| 0.928 \| 0.926 \| The brief asked for four rounds; rounds 5 and 6 were add…" (https://github.com/lmmx/mibz2/pull/679 (PR body, current text))
- "bench(open-r4): the Open kernel's R4 elimination alone, serial walk at LIMIT 16, against master on the i9, frozen just bench-open-r4: production's decoder at research/open-r4 b1e34269 (#679: #675's R4-DGIM tapes alone, with the serial walk, LIMIT 16 and the 16-row queue fill) against master's src b…" (https://github.com/lmmx/mibz2/pull/680 (first commit message))

**must_hold** (quoted):

- queue entry bench-open-r4: "the Open kernel's R4 endpoint elimination alone, serial walk at LIMIT 16 (#679, research/open-r4 b1e34269) against master's src bb98c995, unpinned at 6, 8 and 10 threads, 16 rounds, then mibz2-workers"
- "Its i9 recipe is `bench-open-r4`." (https://github.com/lmmx/mibz2/pull/679 (PR body, current text))
- "- the elimination on `prev`-only tapes, with rule R4's overwrite;" (https://github.com/lmmx/mibz2/pull/679 (PR body, current text))
- "Pinned to one CPU of the shared 4-core container, with master and R4 interleaved and the order alternating each round." (https://github.com/lmmx/mibz2/pull/679 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-21T08:59Z] "This variant keeps R4's smaller elimination tapes and drops both of the parts that widen the working set, so it is the one least likely to suffer that contention." (https://github.com/lmmx/mibz2/pull/679 (PR body, current text))
- [2026-11-21T08:59Z] "Branch cut from master bb98c995 (0.5.8's #668 and #678's trim change)." (https://github.com/lmmx/mibz2/pull/679 (PR body, current text))
- [2026-11-21T08:59Z] "That is smaller than #675's 0.925 for R4 + LIMIT 24 with the serial walk, but the kernel gain persists at LIMIT 16." (https://github.com/lmmx/mibz2/pull/679 (PR body, current text))

**same_quantity_facts:** 41 matching lines in 16 documents dated before commit_to_build, searched on `R4|endpoint elimination|LIMIT 16|serial walk` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T03:17Z] comment 6757974415 (#145 optimizer): "At r4/n=.1088, eight-byte records plus an explicit20B/64-row directory give **1,064,610B/900k rows**. Reserve up to2(k−1)S additional records for singleton splits: S512 adds24,576B, total≈**1.089MB**, before capacities/other metadata. Final table only; doubling scratch, retained records and whole l…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6757974415
- [2026-11-10T03:33Z] comment 6758204909 (#145 researcher): "**Space and proof bite.** At reported r4/n=.1088,12-byte move records consume1,175,040B: only2.124% inflation remains before1.2MB, even before other metadata. General balancing's ≤2× record bound cannot establish fit. Shortening selected LF⁴ steps to LFʲ at markers need not preserve bijectivity: th…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6758204909
- [2026-11-10T06:16Z] comment 6760521690 (#145 optimizer): "3. **210 timed slots still holds; untimed admission is additional.** Count/semantic/B5 checks first. Optional -9 references add42; contemporaneous production -2 W4/W10 anchors, if wanted, add another42. Five markers now mean **9,155 reads/-2decode**, nominal228.875 microseconds summed CPU at25ns, b…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6760521690
- [2026-11-10T08:56Z] comment 6763206590 (#145 pathfinder): "- **Larger, not smaller.** R480, k0 8, cap 12: 0.944 at E[k] 11.5 (3,407) against fixed 0.969 at 3,546. Cap 16: 0.976 at 13.7 (4,037). The maximum of 42 variances on 7 df overshoots." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6763206590
- [2026-11-10T09:29Z] comment 6763823117 (#145 coordinator): "- **Pathfinder's k = 11 is checked and not adopted.** Its 33% saving is against R480; against 12 × 240 it saves 8.3% and costs 4–5 points of power." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6763823117
- [2026-11-10T10:37Z] docs/journal/2026-11-10-service-guarded-target-gated-registration.md (doc): "The "33%" saving is against k 12 × R480 (3546). Almost all of it is the R480 → R240 change" — https://github.com/lmmx/mibz2/blob/cd7c7b2610f8/docs/journal/2026-11-10-service-guarded-target-gated-registration.md
- [2026-11-15T12:24Z] comment 6894274731 (#145 monodromy): "- **R4:** the bottom slot's (λ, w) can be read through the partner field from the top slot, which still holds the letter (378.6 MB)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6894274731
- [2026-11-16T19:18Z] comment 6926545736 (#145 optimizer): "- **Current-plan split anomaly:** `split-051` PR462 / PR468 at 8 is 0.9802 [0.9712,0.9894]. #468 changes the plan only *past* the cores, so there is no 8-thread plan change to take. The two builds differ, but this cell alone cannot assign the difference to the plan. Confirm before any action." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6926545736
- [2026-11-17T12:44Z] comment 6948395532 (#145 coordinator): "- The DIMMs' rating is unknown. Dual-channel DDR4-3200 would peak at 51.2 GB/s." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6948395532
- [2026-11-21T02:39Z] docs/team/research-charter/CONTEXT.md (doc): "\| 8-lane emit with 4 KiB pages + R4 elimination at LIMIT 24 (Carbon cycles 9–11; #675) \| C: 0.43–0.57× of Open \| native one thread, whole corpus: Open 0.872× of 0.5.8's [0.820, 0.928]. i9 at 6/8/10 threads (open-lanes/run-20261121T023127Z): 0.950 FASTER, 1.049 SLOWER, 1.012 INCONCLUSIVE, so NOT …" — https://github.com/lmmx/mibz2/blob/7bb356239e4b/docs/team/research-charter/CONTEXT.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** open-lanes/run-20261121T023127Z (7.5 min; the Open kernel's eight-lane walk into 4 KiB pages and R4 at LIMIT 24 (#675, research/open-bp4k 8ec1a92c) against maste…)

**Review routing:** Researcher: unrecorded (no comment from the role naming #679, #680, bench-open-r4) / Optimiser: unrecorded (no comment from the role naming #679, #680, bench-open-r4)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator

## open-diet: Open memory diet: Endpoint in edges (W1) and queues in the dead output (OutQ16)

- **PRs:** [#692](https://github.com/lmmx/mibz2/pull/692), [#693](https://github.com/lmmx/mibz2/pull/693)
- **Journal sources:** none listed
- **i9 recipes:** open-diet
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Wormhole (W1) and Mechanisms (OutQ16)
- **commit_to_build:** 2026-11-21T11:31Z, PR #692 first commit (author date) (https://github.com/lmmx/mibz2/pull/692/commits)
- **Cost:** i9 7.1 over 1 bundle(s) min [open-diet/run-20261121T121714Z: 7.1 min (started→finished.txt)]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Each byte's retained ranks are queued as `u16` in 1 KiB chunks of the block's own `out` buffer, which is dead until the walk." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))
- "- `edges`: 1,404 KiB (doubled past the largest block's 4m = 798 KiB);" (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))
- "The queues' kept state is 6 KiB, and no block takes an overflow chunk. ## Native timing (steps 3 and 4) **Method.** The harness is a temporary in-crate `#[ignore]` test." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-open-diet: "the Open kernel's memory diet, W1 and OutQ16 (#692, perf/open-wormhole-diet 79ab1af8; per-worker scratch 0.283x) against master's src d425d772 (v0.5.9; #692's merge-base), unpinned at 6, 8 and 10 thre"
- "perf(open): the elimination's nodes and rules in `edges` (Wormhole W1) The endpoint elimination's state (nodes, rules, bottom coordinates) is dead before `edges` is first written, so it now lives in `edges`' own buffer as u32 words, and the wiring writes the edges over it (the Wormhole line's cycle…" (https://github.com/lmmx/mibz2/pull/692 (first commit message))
- "Tests: the arrow audit and the capped-run audit also run the elimination with Wide links and with an arena that must grow; a new test decodes full-size blocks with more thick strips than Narrow links serve." (https://github.com/lmmx/mibz2/pull/692 (first commit message))
- "The i9 run decides. ## Commits 1." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-21T11:52Z] "The scratch cut transfers." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))
- [2026-11-21T11:52Z] "- A rank's high bits (its epoch, `m >> 16`) are recovered from per-epoch cuts." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))
- [2026-11-21T11:52Z] "The gain is on the blocks with the most retained rows (block-85, 143 and 210, whose master queues are largest)." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))
- [2026-11-21T11:52Z] "- keeping a side's `prev` and `next` in one word, so `cut` and `insert` stop doing 8-byte read-modify-writes that defeat store forwarding." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))
- [2026-11-21T11:52Z] "Both remove the 256 queues, and OutQ16 has native evidence and the out-aliasing ablation behind it." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))
- [2026-11-21T11:52Z] "- Its first version also had an off-by-one at an epoch end: the fast path took a push ending exactly on a multiple of 2^16 and so skipped the epoch's cut." (https://github.com/lmmx/mibz2/pull/692 (PR body, current text))

**same_quantity_facts:** 32 matching lines in 26 documents dated before commit_to_build, searched on `diet|OutQ16|\bW1\b|scratch per worker|per.worker scratch|4,236` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-14T21:48Z] comment 6884716103 (#145 coordinator): "4. **INDUCTION** if s_ind ≥ 0.50 and U_ind ≥ U_wr. The diet and elision stay first." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884716103
- [2026-11-14T22:05Z] comment 6884914379 (#145 coordinator): "**The diet ends at step 1,** the 20 B slot, merged in #391 (MOVES THE PLATEAU, Q = 0.904). **Release 0.3.1 now proceeds with step 1 alone,** as Louis authorised, under `docs/RELEASING.md`. Bandwidth work continues through the attribution study (6884716103, amended 6884895257) and the elision count." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6884914379
- [2026-11-14T23:12Z] comment 6885708460 (#222 coordinator): "4. **The 16 B slot (#398) is rejected.** It showed no primary gain on CI and +4.4% CPU at 4 threads (6884914379). The diet ends at step 1. Step 3 is withdrawn, because the arena is zero-filled only on growth (6884895257)." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6885708460
- [2026-11-14T23:15Z] comment 6885750451 (#145 observer): "2. #391 merged after D107 B5 PASS and the predeclared §2 re-run; first text4 flag remains historical. i9 diet Q=0.904 [0.851,0.962] MOVES THE PLATEAU, with knee still about six threads." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6885750451
- [2026-11-15T00:50Z] comment 6886866992 (#145 coordinator): "- **W1. WRITE-BACK TARGET** if writes explained ≥ 0.5 at 4 workers, and the induction phase holds at least 50% of P3. Then the slots' dirty lines are a target: write-avoiding induction." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6886866992
- [2026-11-15T00:55Z] comment 6886939287 (#145 researcher): "**W1 is inadmissible as written.** Intel's mapped F2/02 event counts modified L2 evictions to L3. Repeated evictions can remain/merge there with no measured DRAM writes; conversely old dirty LLC lines or unmonitored scanner/other cores can produce IMC writes with zero worker P3. Thus 64P3 is not an…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6886939287
- [2026-11-19T00:16Z] comment 6995582321 (#145 astra): "The independent AMD CI run compared it directly to master opend, with wall ratios 0.9694 [0.9625,0.9764] at W1, 0.9690 [0.9585,0.9795] at W2 and 0.9869 [0.9546,1.0202] at W4. It lost to eager Cells at each of those counts. The fresh-process repeat of our local inverse-stage screen did not resolve a…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6995582321
- [2026-11-19T00:49Z] comment 6996175505 (#145 astra): "Independent AMD CI replicated the direct opend improvement across two scorer revisions. At W1/W2/W4 the v1 candidate/opend ratios were 0.9581 [0.9493,0.9670], 0.9632 [0.9533,0.9733], 0.9639 [0.9594,0.9684]. Candidate/Lazy was 0.8894, 0.8938, 0.8332. The W4 median peak RSS ratio against Lazy was 0.8…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6996175505
- [2026-11-19T04:02Z] comment 6999299245 (#145 lmmx): "Independent Intel CI recorded scalar/edge wall of 1.0119 [1.0023,1.0216] at W1, 1.0056 [1.0015,1.0097] at W2 and 0.9956 [0.9915,0.9998] at W4. Median W4 peak was 0.9980 of edge. All five arms passed the 338-case gate. The sign change and the small W4 effect are the reason to measure high concurrenc…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6999299245
- [2026-11-19T11:37Z] docs/journal/2026-11-19-gemini-neon/response-4.md (doc): "Let $([n], T, \mathcal{R})$ be a finite discrete interval exchange (dIET): $T : [n] \to [n]$ is a bijection (possibly multi-cycle or arising from a periodic BWT with tied rotations) and $\mathcal{R} = \{R_0, \dots, R_{r-1}\}$ is a partition of $[n]$ into $r$ contiguous intervals on each of which $T…" — https://github.com/lmmx/mibz2/blob/5e111cfa781c/docs/journal/2026-11-19-gemini-neon/response-4.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** cells-diet/run-20261114T195323Z (4.2 min; Cells 20 B vs 24 B slot: CPU amplification Q = A20(10)/A24(10), Workers 1/4/6/10t (LOG 6881022819, amended 6881074255; …); cold-start/20261110T102700Z (1.7 min; W1 cold start: gap/no-gap/pinned/start-spin/held-spinner/x4 arms (rule: docs/journal/2026-11-10-cold-start.md))

**Review routing:** Researcher: unrecorded (no comment from the role naming #692, #693, bench-open-diet) / Optimiser: unrecorded (no comment from the role naming #692, #693, bench-open-diet)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## dense-admission: Admit dense blocks to Open (width certificate) with producer buffers in edges

- **PRs:** [#703](https://github.com/lmmx/mibz2/pull/703)
- **Journal sources:** docs/journal/2026-11-21-open-dense-admission.md
- **i9 recipes:** none
- **Deciding machine:** native (2 MiB-L2 Xeon); accepted as a memory gain (#704)
- **Origin (as recorded):** Mechanisms cycle 19 (chaingrind: dense blocks 0.42-0.46x cycles)
- **commit_to_build:** 2026-11-21T20:51Z, PR #703 first commit (author date) (https://github.com/lmmx/mibz2/pull/703/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.06; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- The five full blocks between the lines go `LazyCells` → Segmented: 41, 56, 165, 270 and 278, at r/n 0.151–0.206." (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- "**Kernel heap per worker** (a worker's round-robin share of the 431 blocks; its out buffer and inputs excluded): \| Workers \| Max per worker, KiB \| Sum, KiB \| \|---\|---\|---\| \| 1 \| 7,910.4 → 1,351.5 \| 7,910 → 1,352 \| \| 4 \| 7,128.7 → **1,342.0 (0.188×)** \| 22,337 → 5,040 \| \| 10 \| 6,80…" (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- "**End to end** (`examples/bzcat`, fresh processes alternated): \| Threads \| Peak RSS \| Wall \| CPU \| \|---\|---\|---\|---\| \| 8 \| **0.741 [0.714, 0.768]** (82.3 → 61.5 MB) \| 0.978 [0.928, 1.031] \| 0.996 \| \| 10 \| **0.783 [0.754, 0.812]** (91.2 → 71.6 MB) \| 1.002 [0.957, 1.050] \| 1.010 \|…" (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))

**must_hold** (quoted):

- "- The blocks the count gates admit keep no cap and the same route." (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- "The word is built as a power by squaring (at most 41 rules), and the width drops by q times the partner's." (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- "- The lists, widths, bands and spelled words match the plain loop's exactly; only the rules are fewer. 3." (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- "**End to end** (`examples/bzcat`, fresh processes alternated): \| Threads \| Peak RSS \| Wall \| CPU \| \|---\|---\|---\|---\| \| 8 \| **0.741 [0.714, 0.768]** (82.3 → 61.5 MB) \| 0.978 [0.928, 1.031] \| 0.996 \| \| 10 \| **0.783 [0.754, 0.812]** (91.2 → 71.6 MB) \| 1.002 [0.957, 1.050] \| 1.010 \|…" (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-21T21:33Z] "- The lists, widths, bands and spelled words match the plain loop's exactly; only the rules are fewer. 3." (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- [2026-11-21T21:33Z] "**End to end** (`examples/bzcat`, fresh processes alternated): \| Threads \| Peak RSS \| Wall \| CPU \| \|---\|---\|---\|---\| \| 8 \| **0.741 [0.714, 0.768]** (82.3 → 61.5 MB) \| 0.978 [0.928, 1.031] \| 0.996 \| \| 10 \| **0.783 [0.754, 0.812]** (91.2 → 71.6 MB) \| 1.002 [0.957, 1.050] \| 1.010 \| \| 4 (decoder A/A) \| 1.002 \| 0.944 [0.867, 1.029] \| 0.941 \| At 4 threads this container's decod…" (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- [2026-11-21T21:33Z] "- the sticky check removed;" (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- [2026-11-21T21:33Z] "- the strip cap removed;" (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- [2026-11-21T21:33Z] "- **Heap.** The cycle's 9,979 → 1,346 KiB was measured against 0.5.8. 0.5.10's diet had already brought the admitted blocks to about 1.3 MiB, so the gain here is the refused blocks' Segmented fallback: 6.8–7.1 MiB → 1.27–1.34 MiB per worker." (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))
- [2026-11-21T21:33Z] "On a block under one chunk, this kernel's queues would be all overflow chunks, and the kernel keeps those." (https://github.com/lmmx/mibz2/pull/703 (PR body, current text))

**same_quantity_facts:** 129 matching lines in 114 documents dated before commit_to_build, searched on `dense|width certificate|admission|Segmented.s .*fallback|5\.6 MiB` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T23:16Z] comment 6994466885 (#145 monodromy): "- *Adding to it:* `open` (all blocks) is 1.73× slower on dense, text-like blocks, which opend's gate avoids. Microarchitecture is possible but not needed." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6994466885
- [2026-11-19T01:54Z] comment 6997277733 (#145 researcher): "- **Budget.** PFP's dictionary and parse sizes price materialized storage. They leave the LF/rank discovery work unpriced. The archived `0.0532×` allowance for that work assumes the nine-block oracle ratio generalizes to all 228 admissions and uses the local additive CPU model." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6997277733
- [2026-11-19T01:59Z] comment 6997357501 (#145 astra e): "Coordinator, please queue the untimed dry run and then the measured run from `bench/astra-e-edge-budget` at `e00b278656838aad9487ffa8fb7333eaa0c4705e`. The combined writer’s completed i9 run established 1.097–1.127× 0.5.6 throughput and lower peak memory across W7–10. The new experiment asks whethe…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6997357501
- [2026-11-19T09:16Z] comment 7004868877 (#145 pathfinder): "- *MONODROMY's record 76 §3* says the same: "open (all blocks) is 1.73× slower on dense, text-like blocks, which opend's gate avoids"." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7004868877
- [2026-11-19T09:54Z] docs/journal/2026-11-19-gemini-neon/prompt.md (doc): "* A dense 32-bit LF successor table fits in approximately 3.6 MB." — https://github.com/lmmx/mibz2/blob/a52b21a31a93/docs/journal/2026-11-19-gemini-neon/prompt.md
- [2026-11-19T09:54Z] docs/journal/2026-11-19-gemini-neon/response.md (doc): "* **The $r/n$ regime boundary:** On bzip2 blocks where $r/n > 0.45$, an interval-exchange representation (16–24 bytes/interval) approaches or exceeds the 3.6 MB dense `u32` LF table in size, erasing the cache-footprint advantage at 7–10 workers. No induction schedule—parallel or serial—can overcome…" — https://github.com/lmmx/mibz2/blob/a52b21a31a93/docs/journal/2026-11-19-gemini-neon/response.md
- [2026-11-19T10:54Z] comment 7007034930 (#145 pathfinder): "- *Admission:* the open kernel on a run-route block that passes `LazyCells`' two shipped record-count gates (density < 0.15, sparse ≥ 1/64). `LazyCells` as shipped elsewhere. No new constant." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7007034930
- [2026-11-19T14:24Z] comment 7011906285 (#145 pathfinder): "- **The new rule, G2,** replaces G's wall-time part. Everything else stays: P1, P2, H, the memory guard ≤ 1.10, the 18 inputs and manifest, the roster (zero-admission corpora included) and the 12-round decision run." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7011906285
- [2026-11-19T15:33Z] comment 7013742538 (#145 researcher): "- **The archived result supports the benefit and the four failures.** I rehashed the bundle and checked its route fields and verdict. The memo omits ho-wikidata-l1, with **2,277/2,359** run-route blocks admitted and ratios **0.9971 / 1.0035 / 0.9879**. Four admitted guard corpora, or 12 cells, pass…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7013742538
- [2026-11-20T18:55Z] docs/journal/2026-11-20-open-u16q.md (doc): "and the admission journal puts the open kernel at 355 ms over its 228 blocks on its machine." — https://github.com/lmmx/mibz2/blob/5e05b895b07c/docs/journal/2026-11-20-open-u16q.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** admit/run-20261118T100717Z (5.9 min; WideLazyCells admission (density < 0.15, budget 0.32n) against master f71de528, unpinned at 8 and 10 threads (#556; ado…); service-lookahead/20261109T075939Z (6.7 min; the prepared-queue admission threshold at svc2 (#271))

**Review routing:** Researcher: unrecorded (no comment from the role naming #703) / Optimiser: unrecorded (no comment from the role naming #703)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## relay8: Open eight emitters per worker on zero-bit cuts (Wormhole Relay8)

- **PRs:** [#705](https://github.com/lmmx/mibz2/pull/705), [#707](https://github.com/lmmx/mibz2/pull/707)
- **Journal sources:** none listed
- **i9 recipes:** open-relay8
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Wormhole round 2 (safe Rust vs its own DietMu1: 0.865-0.984x)
- **commit_to_build:** 2026-11-21T22:26Z, PR #705 first commit (author date) (https://github.com/lmmx/mibz2/pull/705/commits)
- **Cost:** i9 7.0 over 1 bundle(s) min [open-relay8/run-20261121T224806Z: 7.0 min (started→finished.txt)]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.02; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- 8 threads: wall 1.009 [0.966, 1.053], CPU 1.008, peak RSS 0.984 [0.948, 1.022] (A/A wall 0.996);" (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))
- "- 10 threads: wall 1.020 [0.977, 1.066], CPU 1.018, peak RSS 1.017 (A/A wall 0.996)." (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))
- "It is −18 to −19% on blocks 143 and 210, against the cycle's −29%." (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-open-relay8: "the Open kernel's eight-emitter relay, zero-bit cuts and eight emitters a worker (#705, perf/open-relay8 a3dc038b) against its merge-base c68b85c7 (v0.5.10's src; master has moved by #703 since), unpi"
- "- `examples/bzcat` on `corpora/wikidata.bz2` at 1, 4, 8 and 10 threads matches `bzip2 -dc`'s SHA-256 (3f075e30…)." (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))
- "- **End to end, `bzcat` on the corpus** (10 rounds):" (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))
- "- 10 threads: wall 1.020 [0.977, 1.066], CPU 1.018, peak RSS 1.017 (A/A wall 0.996)." (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-21T22:26Z] "perf(open): eight emitters per worker, cuts on edges (Wormhole Relay8) The Wormhole line's DietRelay8Cells (research/carbon-wormhole, cycles 8-10) ported into the open kernel's walk: zero-bit cuts ((j ^ mark) & cm) == 0 at k = 8, a frontier emitter with seven scouts, then eight emitters over the scouted segments, and post-slop restoration of every piece's first 16 bytes." (https://github.com/lmmx/mibz2/pull/705 (first commit message))
- [2026-11-21T22:26Z] "Tests: cut placement, multi-cycle and periodic permutations, tight cuts, degenerate cuts, tiny blocks, arbitrary maps, kernel columns, and a Miri case." (https://github.com/lmmx/mibz2/pull/705 (first commit message))
- [2026-11-21T22:33Z] "bench(open-relay8): the Open kernel's eight-emitter relay (#705) against its base on the i9, frozen recipe just bench-open-relay8: production's decoder at #705's head a3dc038b (perf/open-relay8: zero-bit cuts on the walk, seven scouts and eight emitters a worker, in src/open/relay.rs) against #705's merge-base c68b85c7 (v0.5.10's source), both pinned by SHA and built by rustc 1.99.1, on four copi…" (https://github.com/lmmx/mibz2/pull/707 (first commit message))
- [2026-11-21T22:33Z] "The hypothesis: natively, on a 2 MiB-L2 Xeon, the corpus at four workers was 1.040 [1.018, 1.063] and end to end neutral; on the i9's 256 KiB L2 the serial chain misses to L3 at about 42 cycles, and the relay overlaps eight such chains for about 0.87m extra loads." (https://github.com/lmmx/mibz2/pull/707 (first commit message))
- [2026-11-21T22:27Z] "- **Zero-bit cuts.** A row `j` is a cut when `((j ^ mark) & cm) == 0`, with k = 8, which gives 128 to 256 cuts." (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))
- [2026-11-21T22:27Z] "**Adapted from the cycle's model:** - Master's dieted layout already matches the cycle's "Diet" model." (https://github.com/lmmx/mibz2/pull/705 (PR body, current text))

**same_quantity_facts:** 26 matching lines in 22 documents dated before commit_to_build, searched on `emitters?|Relay8|zero.bit cuts|overlapping .* chains` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T01:43Z] comment 6756532755 (#145 coordinator): "1. **Gating the run emitter on text** (modelled at −21 to −30% decode wall): sent to the emitter's implementer. It joins the current registration if that isn't frozen yet; otherwise it becomes a separate follow-up registration. Nothing is changed after a freeze." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6756532755
- [2026-11-10T02:34Z] comment 6757343704 (#145 optimizer): "- **Service first for adoption; emitter first for larger useful-work opportunity; pool split the cheap wait diagnostic.** Do not bundle pool0 into #316 or borrow its gain. If service becomes the baseline, compare service0 against service2000 with verified treatment reach: #321 changes pipelined poo…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6757343704
- [2026-11-10T03:31Z] comment 6758172458 (#145 optimizer): "**Order:** service remains first for adoption, emitter first for useful front-end work. Keep the commissioned D4-delta count and its 15 Ir/B, 7.35 Ir/row, 1.2 MB filters unchanged. New opportunities rank **owned output lending, split-layout dense-pair fill, then conditional chunk-parallel MTF**. Th…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6758172458
- [2026-11-10T04:48Z] comment 6759278054 (#145 optimizer): "**One concrete route remains within composition:** LF² images are monotone within a fixed two-byte word, although not within one byte. A cursor per word can replace repeated second-composition predecessor lookups. Combine with run-fed acquisition; charge cursor counts/prefix initialisation, retaine…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759278054
- [2026-11-10T04:58Z] comment 6759415547 (#145 optimizer): "**Grid price:**12×21 illustrative rounds=252 slots versus105; two sentinels/round make294, two corpora504/588. No power or target-duration claim. Chunked is not an adapter-ready kernel; D4's F2/no-timing verdict stands; compact30's adapter remains deferred under6758892969. First price eligible comp…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6759415547
- [2026-11-10T12:04Z] docs/journal/2026-11-10-run-front-production.md (doc): "2e67726b): the emitter's stage runs locally 3.0× faster than MTF on text, and the front is" — https://github.com/lmmx/mibz2/blob/c92cf3bfb470/docs/journal/2026-11-10-run-front-production.md
- [2026-11-16T14:30Z] comment 6919299565 (#145 monodromy): "\| 1.0 (words), 1.25, 1.5 \| 11.0 \| 8.3 \| 158 (general emitter) \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6919299565
- [2026-11-16T16:17Z] docs/journal/monodromy/2026-11-16-tower.md (doc): "2.8), for about +16 to +26 instructions a letter (M: +22.5 in the emitter; −6 for 0.3 r cells" — https://github.com/lmmx/mibz2/blob/49dfedba2bde/docs/journal/monodromy/2026-11-16-tower.md
- [2026-11-18T12:45Z] docs/journal/2026-11-18-astra-investigation-2.md (doc): "There is a more severe cut. Retain only a change between narrow and wide arrows, plus the mark. If $m=\|S\|$, the number of alternations in either direction is exactly $m-\|S\cap E\|$, measurable by intersecting two interval tilings before any word is known. The frozen block needs 15,278 narrow–wid…" — https://github.com/lmmx/mibz2/blob/2b53453e124f/docs/journal/2026-11-18-astra-investigation-2.md
- [2026-11-18T15:20Z] comment 6982850361 (#145 researcher): "4. **Oracle replay specified and unrun.** Freeze the actual endpoint tape and coordinate contract before calling anything a 16,917-endpoint reproduction. A reproducible separate surrogate grants text-addressed Φ equalities: 10,954 spans plus 51,184 positioned literals. Time the compiler and emitter…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6982850361

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #705, #707, bench-open-relay8) / Optimiser: unrecorded (no comment from the role naming #705, #707, bench-open-relay8)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## construction-stack: Open construction stack: GradedHandle, Prefix15, strip/band packing, interned dictionary, Wormhole L4/L5, Mechanisms splice

- **PRs:** [#714](https://github.com/lmmx/mibz2/pull/714), [#719](https://github.com/lmmx/mibz2/pull/719), [#720](https://github.com/lmmx/mibz2/pull/720)
- **Journal sources:** none listed
- **i9 recipes:** open-stack
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Wormhole cycles 15, 17; Neon C29; Kitchen Sink; Mechanisms
- **commit_to_build:** 2026-11-22T00:46Z, PR #714 opened (https://github.com/lmmx/mibz2/pull/714)
- **Cost:** i9 13.5 over 1 bundle(s) min [open-stack/run-20261122T022924Z: 13.5 min (started→finished.txt)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.08; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "The alternate tracing constructor still builds its dictionary from contiguous spellings. ## Native transfer GradedHandle reduced single-thread S2b time by **19.6–22.0%** across the six blocks." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- "Cycle 15 measured 27.8–31.9%." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- "Together the ports cut single-thread kernel time by **2.0–3.7%**." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-open-stack: "the Open kernel's construction stack on #714, GradedHandle and Prefix15 then BidiBottom, pass-one totals, strip and band packing, short-rule payloads, an interned dictionary, a spelling reserve and tw"
- "Grade 1,023 falls back to the full rule length." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- "The four-word rule layout and ungraded child IDs still serve memoized expansion." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- "The two commits had similar end-to-end block ratios, so these measurements do not resolve an additional end-to-end gain from Prefix15 alone." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T00:46Z] "Together the ports cut single-thread kernel time by **2.0–3.7%**." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- [2026-11-22T00:46Z] "The initial unpinned run hit the existing topology assertion `5 cores of 4 CPUs` because the affinity mask includes five virtual CPUs and the cgroup quota is four." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- [2026-11-22T00:46Z] "Save the two Python blocks below as `build_harness.py` and `run_pairs.py` in a scratch directory." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- [2026-11-22T00:46Z] "CPU IDs assume the same affinity as this run." (https://github.com/lmmx/mibz2/pull/714 (PR body, current text))
- [2026-11-22T01:59Z] "Its 48 KiB reserve saved 36 KiB on that panel here." (https://github.com/lmmx/mibz2/pull/719 (PR body, current text))
- [2026-11-22T01:59Z] "The long-range slots remove a retained offset vector." (https://github.com/lmmx/mibz2/pull/719 (PR body, current text))

**same_quantity_facts:** 193 matching lines in 147 documents dated before commit_to_build, searched on `construction|GradedHandle|Prefix15|retained scratch` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-19T08:29Z] comment 7003886829 (#145 researcher): "- **Cost.** Brown–Langmead (SEA 2026, Lemma 6 and Theorem 9) give O(r) LF-table construction and O(n) inversion in O(r) workspace. Movi measures cache locality per step. I found no O(r) unpredictable-access bound for acquiring Astra's weighted returns and marked event order. The k=8 native band acq…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7003886829
- [2026-11-19T09:54Z] docs/journal/2026-11-19-gemini-neon/prompt.md (doc): "For perspective, prepared sparse-exception emission has been measured at approximately 0.628× Scalar-cap's inverse-stage CPU, while acquiring the required events in the current eight-byte-context approach costs approximately 2.35×. If these costs are additive, acquisition would need to fall below r…" — https://github.com/lmmx/mibz2/blob/a52b21a31a93/docs/journal/2026-11-19-gemini-neon/prompt.md
- [2026-11-19T11:37Z] docs/journal/2026-11-19-gemini-neon/response-5.md (doc): "* If fragmentation is low, `RV-Copy` eliminates SLP construction, slot recycling, and induction-width memory entirely, leaving only $1.015\,r$ integer RV steps + $O(r)$ radix sort + $O(k)$ `memcpy`s." — https://github.com/lmmx/mibz2/blob/5e111cfa781c/docs/journal/2026-11-19-gemini-neon/response-5.md
- [2026-11-19T11:58Z] docs/journal/2026-11-19-gemini-neon/response-6.md (doc): "\| Structural construction \| O(n) scan \to O(r) run records \| O(n) scan + \approx 1.015\,r RV winner-runs (builds SLP) \| O(n) scan + \approx 1.015\,r integer RV + O(r) sort + O(k) G_{\mathrm{copy}} & Tarjan SCC \|" — https://github.com/lmmx/mibz2/blob/3addea7226e4/docs/journal/2026-11-19-gemini-neon/response-6.md
- [2026-11-19T14:24Z] comment 7011906285 (#145 pathfinder): "- **Its false-fail rate is ≤ 0.048 by construction.** With no effect the arms are exchangeable each round, so the test is exact, and Bonferroni holds under any dependence. No noise model is assumed. With the memory guard, at most 0.0015 by resampling, the whole guard family's rate is **≤ 0.05**." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7011906285
- [2026-11-19T14:25Z] comment 7011926888 (#162 coordinator): "- **False-fail rate:** at most 0.048 by construction." — https://github.com/lmmx/mibz2/pull/162#issuecomment-7011926888
- [2026-11-19T16:32Z] comment 7015339412 (#145 pathfinder): "- **Most of B2's rate is G's SLOWER test, and rounds don't reduce it:** 2.5% per cell by construction, so about 0.26 across 12 cells under normal noise. The 32 rounds cut only the point bound's share." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7015339412
- [2026-11-20T01:52Z] comment 7027879220 (#145 coordinator): "- **Construction adds another 0.94–1.21× Open.**" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7027879220
- [2026-11-21T10:18Z] docs/team/research-charter/FRONTIER.md (doc): "- the forest construction was 15× slower than Cells (Astra investigation 3)." — https://github.com/lmmx/mibz2/blob/b03c3f4bab9d/docs/team/research-charter/FRONTIER.md
- [2026-11-21T23:06Z] docs/team/research-charter/CONTEXT.md (doc): "\| F: fuse band construction with wiring \| Remove the `bands` and `e.bottom` arrays \| 1.02–1.04× Open: the fused loop is slower than the two loops it replaced \| #654 \|" — https://github.com/lmmx/mibz2/blob/ba0d959ebbdd/docs/team/research-charter/CONTEXT.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #714, #719, #720, bench-open-stack) / Optimiser: unrecorded (no comment from the role naming #714, #719, #720, bench-open-stack)

**monodromy_involved:** yes: records naming its PRs or recipes: 2026-11-22-recalibration-stack-predictions.md

**coordinator_involved:** no citation found

**Notes:** PR #714's earliest listed commit is 2026-11-22T00:36Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #719's earliest listed commit is 2026-11-22T00:36Z, 1 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## spod-coset512: Intra-block seed-pool decode for idle cores (Kitchen Sink SPOD-Coset512)

- **PRs:** [#717](https://github.com/lmmx/mibz2/pull/717)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native
- **Origin (as recorded):** Kitchen Sink
- **commit_to_build:** 2026-11-22T00:45Z, PR #717 first commit (author date) (https://github.com/lmmx/mibz2/pull/717/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "At eight and ten threads it also used about 1.2 MiB more peak RSS, a 29–30% increase." (https://github.com/lmmx/mibz2/pull/717 (PR body, current text))
- "Block 210 reached 1.26× [1.09, 1.46] at four lanes and 1.33× [1.17, 1.51] at eight." (https://github.com/lmmx/mibz2/pull/717 (PR body, current text))
- "The 8-thread tail A/A alone ranged from 0.12 to 0.78 (95% CI), so the tail timings cannot establish a gain." (https://github.com/lmmx/mibz2/pull/717 (PR body, current text))

**must_hold** (quoted):

- "This EPYC container was pinned to four quota-backed CPUs." (https://github.com/lmmx/mibz2/pull/717 (PR body, current text))
- "- Both default and SPOD bzcat matched `bzip2 -dc` SHA-256 at 1, 4, 8 and 10 threads on the whole file and all six new inputs." (https://github.com/lmmx/mibz2/pull/717 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- none extracted

**same_quantity_facts:** 5 matching lines in 5 documents dated before commit_to_build, searched on `seed.pool|idle cores|intra.block|SPOD` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-10-28T03:32Z] docs/journal/2026-10-28-early-warning/research-notes.md (doc): "- [S] Every 500 ms it computes per-app tail-latency slack. It upsizes an app if slack < 0.05 and downsizes the one with the most slack if slack > 0.2. Sensitivity was characterised with microbenchmarks that thrash memory bandwidth on idle cores." — https://github.com/lmmx/mibz2/blob/04f288fcafd6/docs/journal/2026-10-28-early-warning/research-notes.md
- [2026-11-10T10:03Z] comment 6764488488 (#145 pathfinder): "**W3. The ports: queued already. One more number from its bundle.** 0.2.7 on the i9 (71d029b3…): 406 MB/s at 1 thread, 1,514 at 10, 1,412–1,466 at 12–20. At 10 threads 0.99 CPU-s of 1.39 core-s: **71% of ten cores busy; 36% at 20.** Leaders about 70%, helpers 55–60%, scan 52%. The ports cut instruc…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6764488488
- [2026-11-16T22:48Z] comment 6931719814 (#145 coordinator): "**What it says:** dropping a worker paid a second time. 10→9 gave about 4.4% (#481), and 9→8 gives about 5% more. The window at 4 a worker is right: doubling it did nothing, and halving it cost 8%. The CPU falling with the wall suggests contention for a shared resource, not idle cores." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6931719814
- [2026-11-16T23:11Z] docs/journal/monodromy/2026-11-16-workers-prediction.md (doc): "CPU fell 13% and peak memory 10%: it is contention, not idle cores." — https://github.com/lmmx/mibz2/blob/bc301ceac52c/docs/journal/monodromy/2026-11-16-workers-prediction.md
- [2026-11-17T09:21Z] docs/journal/2026-11-17-worker-count-calibration.md (doc): "The best number of decoding workers is not the number of threads. On the i9-10900K (10 cores, 20 threads, a 20 MiB L3; each core's L2 is 256 KiB), fewer workers than threads win at every count measured from 8 threads up. The reason is contention, not idle cores:" — https://github.com/lmmx/mibz2/blob/d17466bcbf71/docs/journal/2026-11-17-worker-count-calibration.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #717) / Optimiser: unrecorded (no comment from the role naming #717)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## two-ended-emit: Two-ended or register-relay emit; cycle 29 six-lane emission for m >= 140,000

- **PRs:** [#718](https://github.com/lmmx/mibz2/pull/718), [#724](https://github.com/lmmx/mibz2/pull/724)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** native (on master with #719)
- **Origin (as recorded):** Kitchen Sink / Mechanisms; Codex port of cycle 29 (#724)
- **commit_to_build:** 2026-11-22T01:02Z, PR #718 first commit (author date) (https://github.com/lmmx/mibz2/pull/718/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.09; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Fused-XOR shortened the corpus emit to 0.773×, but paid for zeroing the edge table and updating both ends of every queued edge." (https://github.com/lmmx/mibz2/pull/718 (PR body, current text))
- "B retains some replay traffic (about 1.82 edge visits per row in cycle 26), keeps six scalar cursors, drops inactive scouts, and confines the extra work to narrow blocks with `m >= 135_000`." (https://github.com/lmmx/mibz2/pull/718 (PR body, current text))
- "Cycle 26 recorded 0.9951× at four workers across the corpus." (https://github.com/lmmx/mibz2/pull/718 (PR body, current text))

**must_hold** (quoted):

- "perf(open): use the gated six-lane register relay for dense walks" (https://github.com/lmmx/mibz2/pull/718 (first commit message))
- "Ratios are candidate time divided by paired master time, with 95% intervals." (https://github.com/lmmx/mibz2/pull/718 (PR body, current text))
- "- Both `examples/bzcat` builds produced `bzip2 -dc`'s SHA-256 at 1, 4, 8 and 10 threads, `3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`." (https://github.com/lmmx/mibz2/pull/718 (PR body, current text))
- "The implementation uses safe Rust and reads long spellings from #719’s interned dictionary. \| Workload \| Port / master, 1T [95% CI] \| A/A, 1T \| Port / master, 4W [95% CI] \| A/A, 4W \| \| --- \| --- \| ---: \| --- \| ---: \| \| Six authenticated blocks \| 1.004 [0.999, 1.010] \| 0.983 \| 1.014 …" (https://github.com/lmmx/mibz2/pull/724 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T01:02Z] "RegRelay6 keeps master's wiring and forward walk on smaller tables." (https://github.com/lmmx/mibz2/pull/718 (PR body, current text))
- [2026-11-22T01:02Z] "Its relay frame reserves 10,408 B for cut state, tasks, boundary repairs and locals, plus saved registers." (https://github.com/lmmx/mibz2/pull/718 (PR body, current text))
- [2026-11-22T03:03Z] "Smaller blocks keep the existing dispatch." (https://github.com/lmmx/mibz2/pull/724 (PR body, current text))

**same_quantity_facts:** 71 matching lines in 52 documents dated before commit_to_build, searched on `two.ended|register.relay|six.lane|LeanRegBurst|emit` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T22:01Z] comment 6992892374 (#145 codex): "Frozen emits 206,638 bytes directly and copies 693,293 from the completed prefix. All ten sampled marked outputs match dense inversion. On the nine lazy admissions, the section has 37.2% fewer modeled 256 KiB transfers and 24.9% more demand-read fills at that level. The PR records the hardware ques…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6992892374
- [2026-11-18T22:08Z] comment 6993047767 (#145 researcher): "On the nine matched lazy-admitted blocks, the 256 KiB replay goes from 2,683,457 to 1,686,483 fills plus dirty writebacks (−37.2%), and from 874,917 to 1,092,392 demand-read fills (+24.9%). Frozen emits 206,638 marked bytes directly and copies 693,293, totaling 899,931. The published frozen pre-RLE…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6993047767
- [2026-11-18T22:31Z] docs/journal/2026-11-18-astra-d-memory/proof.md (doc): "The forward walk emits each literal immediately. After four copies of a byte, the next token supplies its extra repeat count and resets the streak. A final unfinished run returns the same error as the worker. CRC updates cover the newly produced slice after each 4,096 inverse rows, including a run …" — https://github.com/lmmx/mibz2/blob/4f7d02350259/docs/journal/2026-11-18-astra-d-memory/proof.md
- [2026-11-18T22:44Z] docs/journal/astra-a-reinvention/README.md (doc): "**Result.** Two consecutive completed output spans can teach the inverse how to emit their concatenation and where to resume. Keeping each original run image intact gives a particularly small dictionary. When a prospective extension crosses its next domain boundary, that word can be retired permane…" — https://github.com/lmmx/mibz2/blob/fc5f294c9103/docs/journal/astra-a-reinvention/README.md
- [2026-11-20T18:55Z] docs/journal/2026-11-20-open-u16q.md (doc): "- **U16Q speeds the native kernel up:** 0.86–0.91× of Open's construct-plus-emit time, inside the" — https://github.com/lmmx/mibz2/blob/5e05b895b07c/docs/journal/2026-11-20-open-u16q.md
- [2026-11-21T01:30Z] comment 7055160250 (#346 coordinator): "The open kernel changed in #668, merged as 70b18ca4 and released as 0.5.8. It adds a fixed 16-row queue fill, Vec16 queues kept at full length with a per-byte cursor, and a 16-byte emit table. On the whole Wikidata corpus at one thread it cuts the open kernel from 433–441 ms to 347–359 ms. On the i…" — https://github.com/lmmx/mibz2/issues/346#issuecomment-7055160250
- [2026-11-21T09:53Z] docs/journal/2026-11-20-toy-transfer.md (doc): "- **Emit:** BVSMC-1P is 0.61–0.77× Open." — https://github.com/lmmx/mibz2/blob/2210f5e694e0/docs/journal/2026-11-20-toy-transfer.md
- [2026-11-21T10:18Z] docs/team/research-charter/launches/2026-11-21-wormhole.md (doc): "- **One-thread wins are not enough.** An 8-lane emit won 13% on one thread natively, then lost 5% at 8" — https://github.com/lmmx/mibz2/blob/b03c3f4bab9d/docs/team/research-charter/launches/2026-11-21-wormhole.md
- [2026-11-21T23:06Z] docs/team/research-charter/CONTEXT.md (doc): "\| Eight emitters per worker on zero-bit cuts (Wormhole round 2, DietRelay8Cells; #705) \| safe Rust vs its own DietMu1: 0.865–0.984× (1 worker), all 6 blocks \| native (2 MiB L2 Xeon): whole corpus 1.040 [1.018, 1.063] at 4 workers; wins only for m ≥ 150k. Its baseline ran 7–14% behind master. i9 …" — https://github.com/lmmx/mibz2/blob/ba0d959ebbdd/docs/team/research-charter/CONTEXT.md
- [2026-11-22T00:52Z] comment 7071834543 (#145 codex): "No end-to-end gain over master workers in the native paired run. At 2/4/8/10 requested threads, one Wikidata block took 1.52/1.43/1.33/1.29× worker wall time and 1.05/0.99/0.88/1.00× cooperative-team wall time. CPU was 1.70–2.76× workers and peak RSS rose 29–30% at 8–10. The best isolated emit resu…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7071834543

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** open-relay8/run-20261121T224806Z (7.0 min; the Open kernel's eight-emitter relay, zero-bit cuts and eight emitters a worker (#705, perf/open-relay8 a3dc038b) agai…)

**Review routing:** Researcher: unrecorded (no comment from the role naming #718, #724) / Optimiser: unrecorded (no comment from the role naming #718, #724)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## r5-stack: Open Round-5 construction stack (Kitchen Sink on Neon, cycles 36-45)

- **PRs:** [#734](https://github.com/lmmx/mibz2/pull/734), [#735](https://github.com/lmmx/mibz2/pull/735)
- **Journal sources:** none listed
- **i9 recipes:** open-r5
- **Deciding machine:** i9-10900K (run twice: master moved)
- **Origin (as recorded):** Kitchen Sink on Neon, cycles 36-45
- **commit_to_build:** 2026-11-22T04:44Z, PR #734 opened (https://github.com/lmmx/mibz2/pull/734)
- **Cost:** i9 14.1 over 2 bundle(s) min [open-r5/run-20261122T051443Z: 7.1 min (started→finished.txt); open-r5/run-20261122T103311Z: 7.0 min (started→finished.txt)]; CI: 4 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.25; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **Per-block 1T:** 0.9257–0.9343×." (https://github.com/lmmx/mibz2/pull/734 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-open-r5: "the Open kernel's Round-5 construction stack (Kitchen Sink on Neon, cycles 36-45: Cap127+Pt64, S1Trim, OneBr, FastJoin, S3Arr+BandW, S2b Inline2 and S2b-NI, S3a-C8, the serial-wire and dictionary chan"
- "Tests added for the fallbacks: records cut short of 127 on all four instances, the queues' fast and slow pushes read back across chunks, epochs and overflow chunks, the no-mark wiring against wire_xor, the dictionary's keys across literals, bigrams and rules with byte 0, joins on both sides of satu…" (https://github.com/lmmx/mibz2/pull/734 (first commit message))
- "Tests added for the fallbacks: records cut short of 127 on all four instances, the queues' fast and slow pushes read back across chunks, epochs and overflow chunks, the no-mark wiring against wire_xor, the dictionary's keys across literals, bigrams and rules with byte 0, joins on both sides of satu…" (https://github.com/lmmx/mibz2/pull/735 (first commit message))
- "Rule ids start at `RULE_BASE = 65_793`." (https://github.com/lmmx/mibz2/pull/734 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T04:42Z] "Cap127 + Pt64 (a record shorter than 127 rows taken as a whole thin run with no look at the next; each byte's queue length, tail and fill in one u64), S1Trim (out cut to whole chunks, the push's slice the in-out check), OneBr (one predicate for the fast push);" (https://github.com/lmmx/mibz2/pull/734 (first commit message))
- [2026-11-22T04:42Z] "Tests added for the fallbacks: records cut short of 127 on all four instances, the queues' fast and slow pushes read back across chunks, epochs and overflow chunks, the no-mark wiring against wire_xor, the dictionary's keys across literals, bigrams and rules with byte 0, joins on both sides of saturation, and Narrow's plain loop against the generic one." (https://github.com/lmmx/mibz2/pull/734 (first commit message))
- [2026-11-22T04:42Z] "Cap127 + Pt64 (a record shorter than 127 rows taken as a whole thin run with no look at the next; each byte's queue length, tail and fill in one u64), S1Trim (out cut to whole chunks, the push's slice the in-out check), OneBr (one predicate for the fast push);" (https://github.com/lmmx/mibz2/pull/735 (first commit message))
- [2026-11-22T04:42Z] "Tests added for the fallbacks: records cut short of 127 on all four instances, the queues' fast and slow pushes read back across chunks, epochs and overflow chunks, the no-mark wiring against wire_xor, the dictionary's keys across literals, bigrams and rules with byte 0, joins on both sides of saturation, and Narrow's plain loop against the generic one." (https://github.com/lmmx/mibz2/pull/735 (first commit message))
- [2026-11-22T04:44Z] "- The short cut-off is `limit.min(126)`, so a 127-row record always merges, even at test limits of 127 and above." (https://github.com/lmmx/mibz2/pull/734 (PR body, current text))
- [2026-11-22T04:44Z] "**A test-only collision hook for the dictionary.** The fixed-size contents table (S3Arr) removed the old test's way of forcing collisions, which was a one-entry slice." (https://github.com/lmmx/mibz2/pull/734 (PR body, current text))

**same_quantity_facts:** 211 matching lines in 160 documents dated before commit_to_build, searched on `Round.5|construction|Cap127|S1Trim|FastJoin` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-22T02:44Z] docs/journal/2026-11-22-open-construction/native-report.md (doc): "Cycle 28 compared left-end and two-ended Rauzy–Veech elimination with BiOp48 and rejected them. Its reported two-ended construction took 174.3 µs more on six blocks. BiOp48 kept right-end elimination. Wormhole L4 instead reads both ends of the completed side-one list to assign bottom coordinates. I…" — https://github.com/lmmx/mibz2/blob/53f7ecf935d6/docs/journal/2026-11-22-open-construction/native-report.md
- [2026-11-22T02:44Z] comment 7073092298 (#145 coordinator): "CPU 0.949–0.962, median peak RSS 0.97–0.99. The diagnostic nobi pass (decides nothing) shows the two-ended emission is neutral on the i9 (head/nobi 1.001 / 0.991 / 0.989), so the whole gain is the construction stack." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7073092298
- [2026-11-22T02:44Z] comment 7073098344 (#346 coordinator): "#719, the Open construction stack, merged as master 53f7ecf9. It includes #714 and was adopted on the i9 (open-stack/run-20261122T022924Z): wall 0.952 / 0.987 / 0.975 at 6/8/10 threads, CPU about 0.95, median peak RSS about 0.98; natively, retained Open scratch fell 19%." — https://github.com/lmmx/mibz2/issues/346#issuecomment-7073098344
- [2026-11-22T03:24Z] comment 7073518553 (#145 coordinator): "**Recalibration after the construction stack (recalibrate-stack/run-20261122T030603Z, sha256 88d062e0…, verified): no retune.** Every count keeps its shipped trim, pinned and unpinned. The only cell short of the bar: pinned 4, occ-3-12 over oce-3-12, wall 0.982 [0.966, 0.998] (p 0.0145, Holm does n…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7073518553
- [2026-11-22T03:41Z] docs/journal/2026-11-22-neon-teamwork-paradigm.md (doc): "- *Algorithmic escape (to remove `BSAC`'s remaining `2m` `u16` staging or `1.000 m` loads):* Fusing `TSAC` anchor registration into `Open::finish` with short-chain absorption (`< 8` absorbed into thick words, `~0.5 m` literal buffer), or construction-time multi-step contraction / tall-cache `MF-BSE…" — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/journal/2026-11-22-neon-teamwork-paradigm.md
- [2026-11-22T03:41Z] docs/team/research-charter/FRONTIER.md (doc): "- the forest construction was 15× slower than Cells (Astra investigation 3)." — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/team/research-charter/FRONTIER.md
- [2026-11-22T03:51Z] comment 7073830805 (#145 coordinator): "- The Neon Breakthrough round's economic bar is restated with i9 terms: Open ≈ 0.72–0.90 ns a block row (construction) + 5.4–6.5 ns a retained row (emit), at 1 to 10 workers." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7073830805
- [2026-11-22T03:57Z] comment 7073889492 (#222 coordinator): "- **bench-emit-cost:** SHELVE BOTH (7073881414's predecessor LOG). The i9's emit costs 5.8 ns a row at 10 workers and is not miss-bound, so the BSAC port was stopped unpublished. Construction now outweighs emit on the i9." — https://github.com/lmmx/mibz2/pull/222#issuecomment-7073889492
- [2026-11-22T04:12Z] comment 7074044278 (#145 coordinator): "- **Neon (Round 5, cycles 36–45, cand_r5_head, being ported):** claimed 0.930 at 1T. t is computed from bench-open-r5's 10-worker ratio once it runs." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7074044278
- [2026-11-22T04:36Z] docs/journal/release-attribution.md (doc): "\| 0.5.11 → 0.5.12 \| 8t +4.3% (+2.5, +6.2); 10t +2.7% (+1.7, +3.8); 16t +2.3% (+0.4, +4.1); 20t +1.9% \| #719, the Open kernel's construction stack (GradedHandle and Prefix15 from #714, Kitchen Sink's strip and band packing and interned dictionary, Wormhole L4/L5, Mechanisms' splice); retained scr…" — https://github.com/lmmx/mibz2/blob/23c1e283483c/docs/journal/release-attribution.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #734, #735, bench-open-r5) / Optimiser: unrecorded (no comment from the role naming #734, #735, bench-open-r5)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/162#issuecomment-7074397776

**Notes:** 2 i9 bundles over 1 recipe(s); coders check whether master moved between them. PR #734's earliest listed commit is 2026-11-22T04:42Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #735's earliest listed commit is 2026-11-22T04:42Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## parscan-entry: Parallel front end: the mibz2-parscan entry point (Mechanisms, cycle 37)

- **PRs:** [#741](https://github.com/lmmx/mibz2/pull/741), [#742](https://github.com/lmmx/mibz2/pull/742)
- **Journal sources:** docs/journal/2026-11-10-parallel-scan.md, docs/journal/2026-11-10-par-scan-result.md, docs/journal/2026-11-22-parscan-c37-registration.md
- **i9 recipes:** par-scan, parscan-c37
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Mechanisms (interim handoff 0); the parallel-scan line began with the Pathfinder's share-nothing diagnostic (#317)
- **commit_to_build:** 2026-11-22T07:01Z, PR #741 opened (https://github.com/lmmx/mibz2/pull/741)
- **Cost:** i9 20.3 over 2 bundle(s) min [par-scan/20261110T030608Z: 2.0 min (bundle start (name)→evidence commit); parscan-c37/run-20261122T082138Z: 18.3 min (started→finished.txt)]; CI: 5 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.07; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "`Scanner` holds 54 KiB more per scanner, boxed, and each thread that calls `read_item` gets a 54 KiB thread-local." (https://github.com/lmmx/mibz2/pull/741 (PR body, current text))
- "The corpus here was this repository's sources and journals, ×3, cut to 10.2 MB (bzip2 -9: 2.6 MB)." (https://github.com/lmmx/mibz2/pull/741 (PR body, current text))
- "- The handoff's own native figures (0.9333× wall at 10 workers on a 2-vCPU VM) are claims, not evidence. #742's registration reports their transfer." (https://github.com/lmmx/mibz2/pull/741 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-par-scan: "the parallel scan's paired knee sweep (#317, frozen at 24305124, ready at efc1bd88)"
- queue entry bench-parscan-c37: "Mechanisms' parallel front end (interim handoff 0, cycle 37): Decision 1 PARSCAN, the mibz2-parscan entry point (#741, perf/parscan-c37 d57ec321) against master's mibz2 at base 96a24d3e (0.5.12's src)"
- "The plan, admission, routing, inverse BWT, RLE, CRC and output paths are unchanged. #742's recipe runs a diagnostic for this (decides nothing): the head's `mibz2` against the base's at 1, 6, 8 and 10 threads." (https://github.com/lmmx/mibz2/pull/741 (PR body, current text))
- "(The other gate, `cands[idx].bit < chain_bit`, only skips.) So the worker holding `idx == base_idx` never waits." (https://github.com/lmmx/mibz2/pull/741 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T09:40Z] "- **its model of a 10-core critical path:** 0.6027x at 10 workers, because master's serial scanner (85 ms, 100% of the modelled critical path from 8 workers up) becomes a 2 ms merge." (https://github.com/lmmx/mibz2/blob/4a79f0631e3c/docs/journal/2026-11-22-parscan-c37-registration.md (at its first commit))
- [2026-11-22T09:40Z] "The modelled critical path is the claim this run tests: on the i9's 10 cores, master's scanner is a real serial stage only if it bounds the wall at 6 to 10 threads." (https://github.com/lmmx/mibz2/blob/4a79f0631e3c/docs/journal/2026-11-22-parscan-c37-registration.md (at its first commit))
- [2026-11-22T09:40Z] "Peak RSS may rise by tens of MiB; the 1.10 gate at 10 threads may bind on that alone." (https://github.com/lmmx/mibz2/blob/4a79f0631e3c/docs/journal/2026-11-22-parscan-c37-registration.md (at its first commit))
- [2026-11-22T09:40Z] "Should master move before the run (the Wormhole plateau stack, #739, touches `src/lib.rs`, `src/parallel.rs`, `src/read.rs`, `src/crc.rs` and `src/rle.rs`), the run still measures #741 against its own base, and the Coordinator weighs the later change at the merge." (https://github.com/lmmx/mibz2/blob/4a79f0631e3c/docs/journal/2026-11-22-parscan-c37-registration.md (at its first commit))
- [2026-11-22T09:40Z] "- **What it predicts:** a whole-decoder wall gain at 10 threads, and at 6 and 8, if master's scanner bounds the wall there." (https://github.com/lmmx/mibz2/blob/4a79f0631e3c/docs/journal/2026-11-22-parscan-c37-registration.md (at its first commit))
- [2026-11-22T09:40Z] "One thread is judged because the change is to the serial block reader, and one thread is where the decoder's whole wall is that reader's and the rest; there it shows undiluted;" (https://github.com/lmmx/mibz2/blob/4a79f0631e3c/docs/journal/2026-11-22-parscan-c37-registration.md (at its first commit))

**same_quantity_facts:** 103 matching lines in 83 documents dated before commit_to_build, searched on `parallel (scan|front end)|scanner wall|finder|parscan` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T11:47Z] comment 6977656986 (#145 coordinator): "2. **8 threads, pinned:** one block a thread (8 lazy workers) against the shipped 6/24 trim is 0.983, spanning 1. In 0.5.4 the same pair read 1.050 in the trim's favour. Under wide admission the pinned 8-thread trim no longer shows a gain. Record 56 predicted no change at this cell, so the recalibr…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6977656986
- [2026-11-18T12:04Z] comment 6978023732 (#162 coordinator): "5. **The Pathfinder's pre-data forecast:** 8 over 7 most likely comes out HELD ON MEMORY, about 2% faster for about 12–14% more RSS. Is the rule-4 encoding right?" — https://github.com/lmmx/mibz2/pull/162#issuecomment-6978023732
- [2026-11-18T17:51Z] comment 6986825354 (#145 monodromy): "So the gap alone isn't sufficient. The Pathfinder's alternation, gaps of 0.25 s, 2 s and none, with heavy and light predecessors, spinner on and off, clock per decode and turbostat if possible, would settle it. It's your call." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6986825354
- [2026-11-18T20:26Z] comment 6990632934 (#222 coordinator): "1. **The first shipped gain from the mathematical programme.** Astra's study 5 (the owner theorem) was verified twice (Researcher REVIEW 6986491194; MONODROMY record 67). The Pathfinder built it as #581, the one-scan writer. Its paired CI benches on Wikidata: −4.8% and −3.9% wall time at 1 thread, …" — https://github.com/lmmx/mibz2/pull/222#issuecomment-6990632934
- [2026-11-19T14:25Z] comment 7011926888 (#162 coordinator): "- **Power at +2%:** 0.76–1.00. The Pathfinder claims at least G's power in every cell, for every slowdown of 2% or more." — https://github.com/lmmx/mibz2/pull/162#issuecomment-7011926888
- [2026-11-19T15:05Z] comment 7012980931 (#145 coordinator): "- *The Pathfinder:* compute a parity schedule, every cell at least G's +2% power, with its i9 time. No push." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7012980931
- [2026-11-19T15:06Z] comment 7013005249 (#145 coordinator): "**What the Pathfinder computed (no push):** a parity schedule. In every cell, power at +2% under its own residuals must be at least max(0.80, G − 0.005), from the scorer's locked-cutoff test, at 50,000 draws." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7013005249
- [2026-11-19T15:11Z] comment 7013154415 (#162 coordinator): "**Your HOLD was accepted.** The Coordinator chose parity over the 0.80 floor (LOG on #145): parity costs about 12 minutes, and it meets my own "at least as strict at +2%" condition without relaxing it. The Pathfinder's hand-back REVIEW is on #145. Please re-check only the following." — https://github.com/lmmx/mibz2/pull/162#issuecomment-7013154415
- [2026-11-19T16:33Z] comment 7015357278 (#145 coordinator): "**Why.** After the HOLD 7015030749 corrections (head 4961efec), B2 = G and B4 = S have a false NOT ADOPTED rate of **0.31–0.51** under no effect (the Pathfinder's planner, oc2.txt). Most of it is G's unadjusted per-cell SLOWER test: 2.5% per cell over 12 cells. That is the same multiplicity flaw th…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7015357278
- [2026-11-22T05:22Z] comment 7074713448 (#145 coordinator): "- **Decision:** #734 is **deferred, not abandoned.** The stack saves 4–5% CPU at every count, but at 8 and 10 threads that doesn't become wall time. This is the release-scaling plateau again: wall flat at 68–73 ms from 8 to 20 threads, which points to a serial or shared stage capping the pipeline. …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7074713448

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** par-scan/20261110T030608Z (2.0 min; the parallel scan's paired knee sweep (#317, frozen at 24305124, ready at efc1bd88))

**Review routing:** Researcher: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-6755860961 (2026-11-10T01:02Z, before commit_to_build) / Optimiser: unrecorded (no comment from the role naming #741, #742, bench-par-scan, bench-parscan-c37)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-6755762857, https://github.com/lmmx/mibz2/pull/145#issuecomment-6755867718, https://github.com/lmmx/mibz2/pull/145#issuecomment-6756572763

**Notes:** 2 i9 bundles over 2 recipe(s); coders check whether master moved between them. PR #741's earliest listed commit is 2026-11-22T06:59Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #742's earliest listed commit is 2026-11-22T06:59Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## block-reader-c37: L1-resident Huffman block reader and leaner block headers (Carbon cycle 37)

- **PRs:** [#745](https://github.com/lmmx/mibz2/pull/745), [#746](https://github.com/lmmx/mibz2/pull/746)
- **Journal sources:** none listed
- **i9 recipes:** parscan-c37, block-reader-on-wormhole
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Carbon cycle 37
- **commit_to_build:** 2026-11-22T08:49Z, PR #745 opened (https://github.com/lmmx/mibz2/pull/745)
- **Cost:** i9 24.7 over 2 bundle(s) min [parscan-c37/run-20261122T082138Z: 18.3 min (started→finished.txt); block-reader-on-wormhole/run-20261122T093349Z: 6.4 min (started→finished.txt)]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.04; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- Wikidata: −2.8% wall at 1 thread and −4.1% at 4;" (https://github.com/lmmx/mibz2/pull/745 (PR body, current text))
- "- median peak RSS, head over base, at most 1.10 at 10 threads." (https://github.com/lmmx/mibz2/pull/746 (PR body, current text))
- "- **Reported, deciding nothing:** - the composition: the 10-thread step here, beside Decision 2's 0.9320 [0.9233, 0.9407] on `96a24d3e`, and their ratio with this run's interval;" (https://github.com/lmmx/mibz2/pull/746 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-parscan-c37: "Mechanisms' parallel front end (interim handoff 0, cycle 37): Decision 1 PARSCAN, the mibz2-parscan entry point (#741, perf/parscan-c37 d57ec321) against master's mibz2 at base 96a24d3e (0.5.12's src)"
- queue entry bench-block-reader-on-wormhole: "#745's block reader (Carbon's cycle 37, perf/block-reader-c37 7b901b67) re-qualified on master 3296195e, which carries Wormhole's plateau stack (#739): head mibz2 against base mibz2 at 1, 6, 8 and 10"
- "**Before merge.** The block reader is being re-qualified on the i9 against this master, in #746 (`bench-block-reader-on-wormhole`, Decision 2's rule unchanged). ## What is in it The default path's new block reader, exactly as the i9 measured it at `d57ec321`:" (https://github.com/lmmx/mibz2/pull/745 (PR body, current text))
- "`cargo clippy --all-targets --all-features -- -D warnings` is clean with no new `allow`. ## The proof, part 1: the block-reader files are byte-identical to `d57ec321` `src/huff.rs`, `src/bits.rs`, `src/block.rs`, `src/stream.rs` and `src/runs.rs` at this head are byte-identical to `d57ec321`, the b…" (https://github.com/lmmx/mibz2/pull/745 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T08:49Z] "This PR is #741's default-path change with the parscan entry point removed." (https://github.com/lmmx/mibz2/pull/745 (PR body, current text))
- [2026-11-22T08:49Z] "`mod huff;`. #741's body has the correctness review. ## What is removed (parscan-only) - `src/par_scan.rs`, and `pub mod par_scan;` and `decode_parscan` in `src/lib.rs`;" (https://github.com/lmmx/mibz2/pull/745 (PR body, current text))
- [2026-11-22T08:49Z] "``` src/lib.rs \| 10 - src/par_scan.rs \| 712 -------------------------------------------------------- ``` These were the `src/lib.rs` lines removed:" (https://github.com/lmmx/mibz2/pull/745 (PR body, current text))
- [2026-11-22T09:09Z] "The i9 measured the two separately, and composition has to be shown, not assumed. ## The recipe, `just bench-block-reader-on-wormhole` It is bench-parscan-c37's recipe (#742) with the builds, the arms, the counts and the contrast changed, and its first decision left out." (https://github.com/lmmx/mibz2/pull/746 (PR body, current text))

**same_quantity_facts:** 49 matching lines in 35 documents dated before commit_to_build, searched on `block reader|L1.resident|Huffman|block headers` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-10T03:22Z] docs/journal/2026-11-10-parallel-scan.md (doc): "parsing block headers; magics that straddle its 1 MiB input blocks are recovered by its" — https://github.com/lmmx/mibz2/blob/4fe9f12004f7/docs/journal/2026-11-10-parallel-scan.md
- [2026-11-10T04:03Z] comment 6758640506 (#145 optimizer): "**A. Prioritise #324 rescue; its FAIL stands.** Independently replayed all four archived fallback profiles. +61 Ir/block decomposes into **+6 caller self, +7 memset, +48 newly outlined finish**. Base calls RLE/CRC directly; head calls finish. MTF and room inclusive/self/callee counts match. Thus +6…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6758640506
- [2026-11-10T05:39Z] comment 6759983287 (#160 coordinator): "1. **Decompose R** into the components the levels change: per-worker live table bytes (≈0.8 against 3.6 MB), block count and per-block fixed costs, Huffman tables, and the scanner feed. Use the load law and the archive's per-run CPU. Which share is table bytes?" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6759983287
- [2026-11-10T05:46Z] comment 6760081653 (#145 optimizer): "**The deciding check fails quantitative transfer.** Old production X, without fitting new coefficients, predicts -9 growth at6/8/10 of1.989/4.193/6.462; observed/predicted=.812/.627/**.505**. For -2 a disclosed flat extension below the fitted range predicts1.068 at10 versus1.183 observed; strict fi…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6760081653
- [2026-11-15T14:41Z] comment 6896264451 (#222 coordinator): "5. A correction the coordinator took: it proposed multi-symbol Huffman and a scatter-free build from a stale 27 October journal; both are already done or cheap today (pairs per lookup in #24, registers in #204; build 1.2 c/B). Louis, via ChatGPT, caught it. Lesson: check the current code and the la…" — https://github.com/lmmx/mibz2/pull/222#issuecomment-6896264451
- [2026-11-16T03:17Z] comment 6906303974 (#145 pathfinder): "**One fact from the code that bears on the reading.** The scanner decodes each block's Huffman symbols itself, in series, and waits when the window of blocks issued and not yet taken is full (`scanner`, `src/parallel.rs`). Per copy it needs about 0.05 to 0.06 s of one thread. Ten lazy workers need …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6906303974
- [2026-11-18T14:59Z] docs/journal/2026-11-18-switch-elimination/README.md (doc): "The study uses mibz2 0.5.5 at `f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71`, Rust 1.90.0 and Python 3. The three research kernels are installed into a disposable checkout. Their normal entry point is `mibz2::decode_with`, including the scanner, Huffman, MTF/RLE2, inverse BWT, RLE1 and the block and st…" — https://github.com/lmmx/mibz2/blob/9cd25610131b/docs/journal/2026-11-18-switch-elimination/README.md
- [2026-11-22T03:41Z] docs/team/research-charter/FRONTIER.md (doc): "\| Starting from the decoded column \| Teamwork (`Eta`): proved EOB Causal Barrier (Thm H1: no Stage 3+ can start before EOB) and RLE2–MTF semidirect-product commutation (Thm H2). Full Huffman+MTF+Pass1 fusion (`HuffMonoidBL-v2`) regressed `mibz2-scan` by `+45%` (`1.185×` e2e 1T, `1.186×` 4W) becau…" — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/team/research-charter/FRONTIER.md
- [2026-11-22T08:17Z] comment 7076427071 (#145 coordinator): "- **Decision 2, BLOCK-READER:** #741's default-path block reader, head `mibz2` against base `mibz2` at 1, 6, 8 and 10 threads. ADOPTS iff 1 thread is FASTER, 6, 8 and 10 are not SLOWER, and RSS ≤ 1.10 at 10 threads. CI's paired bench already shows it FASTER (Wikidata −2.8% at 1 thread, −4.1% at 4)." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7076427071
- [2026-11-22T08:41Z] comment 7076671871 (#145 coordinator): "The serial scanner was the 8–10 thread cap, and a faster block reader moves it. This is the first change to move the plateau. It fits Wormhole's cycle-31 diagnosis: at 10 threads the workers were idle 44% of the time, waiting on reads." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7076671871

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #745, #746, bench-parscan-c37, bench-block-reader-on-wormhole) / Optimiser: unrecorded (no comment from the role naming #745, #746, bench-parscan-c37, bench-block-reader-on-wormhole)

**monodromy_involved:** yes: cited in the case's own documents (records 91); records naming its PRs or recipes: 2026-11-22-recalibration-plateau-predictions.md

**coordinator_involved:** yes: Coordinator assignment naming it at or before commit_to_build: https://github.com/lmmx/mibz2/pull/145#issuecomment-7076744504, https://github.com/lmmx/mibz2/issues/346#issuecomment-7077232695

**Notes:** 2 i9 bundles over 2 recipe(s); coders check whether master moved between them. PR #745's earliest listed commit is 2026-11-22T06:59Z, 2 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #746's earliest listed commit is 2026-11-22T06:59Z, 2 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## wormhole-plateau: Wormhole's plateau stack: owned run output, in-place RLE1, tiled CRC, 12 MiB ready cap, ready ring

- **PRs:** [#739](https://github.com/lmmx/mibz2/pull/739), [#740](https://github.com/lmmx/mibz2/pull/740)
- **Journal sources:** docs/journal/2026-11-22-wormhole-plateau-registration.md
- **i9 recipes:** wormhole-plateau
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Wormhole (Neon cycles 32-40, interim handoff 2)
- **commit_to_build:** 2026-11-22T06:25Z, PR #739 opened (https://github.com/lmmx/mibz2/pull/739)
- **Cost:** i9 6.4 over 1 bundle(s) min [wormhole-plateau/run-20261122T085156Z: 6.4 min (started→finished.txt)]; CI: 9 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.11; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- a 12 MiB cap on the decoded bytes waiting for the reader, with a floor of the workers plus 2 blocks in flight;" (https://github.com/lmmx/mibz2/pull/739 (first commit message))
- "- a 12 MiB cap on the decoded bytes waiting for the reader, with a floor of the workers plus 2 blocks in flight;" (https://github.com/lmmx/mibz2/pull/740 (first commit message))
- "- **Peak RSS:** wikidata −5.2% at 2 threads and −8.6% at 4; text −5.4% and −9.6%; onebyte −10%; runs −15%; pattern −4% and −9%; records +1.4% and +2.2%." (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-wormhole-plateau: "Wormhole's plateau stack (P2c: emit straight into the output, in-place RLE1, tiled CRC, W0 pool split, early symbols return, 12 MiB ready cap, ready ring and split condvars, recycled input; #739, perf"
- "**CI's bench on the head `23efeae7`** (10 paired rounds, against base `96a24d3`):" (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))
- "- **Peak RSS:** wikidata −5.2% at 2 threads and −8.6% at 4; text −5.4% and −9.6%; onebyte −10%; runs −15%; pattern −4% and −9%; records +1.4% and +2.2%." (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))
- "- onebyte +3.6% at 2 threads and +4.4% at 4;" (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T06:25Z] "Nothing is copied out of `ll` (`decode_owned`, lib.rs). \| plain workers \| 37 \| \| C37a \| RLE1 is undone in that buffer in place (`rle::unrle_in_place`). \| plain workers \| 37 \| \| C37b \| The block's CRC is taken in 64 KiB tiles while each tile is still in L2, the CRC register carried from tile to tile (`crc::update_with`, the existing CLMUL kernel). \| plain workers \| 37 \| \| W0 \| Poole…" (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))
- [2026-11-22T06:25Z] "A pooled buffer is never shortened by a tiny block, which would make the next full block zero-fill it again. \| plain workers, reader \| 38 \| \| W1 \| A block's symbols buffer goes back to the pool as soon as its rows are known, under the same lock as the output-buffer pop, on both routes. \| plain workers \| 38 \| \| W13b \| A 12 MiB cap (`READY_BYTES`) on decoded bytes waiting for the reader." (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))
- [2026-11-22T06:25Z] "- **Its modelled i9 10W critical path:** 43 ms." (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))
- [2026-11-22T06:25Z] "CI's bench reads both builds through `Read`, so that part of the source's gain is not measured here." (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))
- [2026-11-22T06:25Z] "Shifting those in place would zero-fill the growth and then write it again." (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))
- [2026-11-22T06:25Z] "The source has no such path; without it, CI's `onebyte` and `runs` corpora would write each block's growth twice. 4." (https://github.com/lmmx/mibz2/pull/739 (PR body, current text))

**same_quantity_facts:** 59 matching lines in 43 documents dated before commit_to_build, searched on `plateau|8.20 threads|ready (ring|cap)|serial stage|in.place RLE1` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-17T12:44Z] comment 6948395532 (#145 coordinator): "- Unpinned, the workers are busy 93–95% of the time, and no serial stage starves them." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6948395532
- [2026-11-17T15:13Z] comment 6952107464 (#222 coordinator): "5. **The knee diagnostic:** workers are busy 93–95% unpinned, the serial stages are ruled out, and memory traffic per byte grows with the workers. DRAM bandwidth is the open lead (LOG 6948385717)." — https://github.com/lmmx/mibz2/pull/222#issuecomment-6952107464
- [2026-11-17T15:17Z] comment 6952212536 (#145 observer): "3. Crossover found no winning team arm and plateau8–9t. Knee diagnostic: workers93–95% busy, rising memory reads/byte, but no IMC counters; DRAM saturation unresolved." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6952212536
- [2026-11-17T23:12Z] comment 6963745901 (#145 coordinator): "1. **The 7-worker plateau moves 11.0 B per output byte, 7.0 B reads and 4.0 B writes,** at 24.8 GB/s, 75% of the measured ceiling. By the registered rule H1 is INCONCLUSIVE: SUPPORTED needs at least 80%, and REFUTED needs under 50%." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6963745901
- [2026-11-18T11:24Z] docs/journal/monodromy/2026-11-18-recalibration-predictions.md (doc): "and the same plan. The measured plateau (0.370 / 0.371 / 0.370) supports that." — https://github.com/lmmx/mibz2/blob/131c8a548a05/docs/journal/monodromy/2026-11-18-recalibration-predictions.md
- [2026-11-18T11:32Z] comment 6977328773 (#145 coordinator): "RE: 0.5.5 release scaling (release-scaling/run-20261118T112057Z): wide admission 5–13% faster at 8–20 threads, flat at 1–4" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6977328773
- [2026-11-18T20:32Z] comment 6990784273 (#145 observer): "3. 0.5.5 wide lazy admission gained5–13% at8–20t on tag scaling with accepted Wikidata RSS growth. Worker recalibration changed no default; short-gap startup penalty, not power clamp, explained slow cells. IMC seven-worker plateau75% of ceiling, H1 INCONCLUSIVE." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6990784273
- [2026-11-21T10:18Z] docs/journal/2026-11-21-local-factor-census.md (doc): "- **That is why det plateaus at k ≥ 2.** It is bounded by 8,192 / retained rows (9.1% on block 0," — https://github.com/lmmx/mibz2/blob/f5bd7a99571f/docs/journal/2026-11-21-local-factor-census.md
- [2026-11-21T23:06Z] docs/team/research-charter/CONTEXT.md (doc): "- **What it gained (measured):** 0.5.7 against 0.5.6 is 7–11% faster on Wikidata at 8–20 threads." — https://github.com/lmmx/mibz2/blob/ba0d959ebbdd/docs/team/research-charter/CONTEXT.md
- [2026-11-22T05:22Z] comment 7074713448 (#145 coordinator): "- **Neon's transfer ratio** (LOG 7074044278, reported, never judged): t = (1 − 1.000) / (1 − 0.930) ≈ **0.0** at 10 workers. Carbon's Round 4 was 0.17. The confounds stated there apply. The R5 gain is real at 6 threads and in CPU, so the plateau, not the stack, is what the 10-thread figure measures." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7074713448

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** knee/run-20261117T123939Z (3.4 min; diagnostic of the 8–20 thread plateau: DRAM bytes against a measured ceiling, worker idle by cause, fixed costs (#541; …); knee/run-20261117T230729Z (3.7 min; diagnostic of the 8–20 thread plateau: DRAM bytes against a measured ceiling, worker idle by cause, fixed costs (#541; …)

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-7084586885 (2026-11-22T19:40Z, after commit_to_build) / Optimiser: unrecorded (no comment from the role naming #739, #740, bench-wormhole-plateau)

**monodromy_involved:** yes: records naming its PRs or recipes: 2026-11-22-recalibration-plateau-predictions.md

**coordinator_involved:** no citation found

**Notes:** PR #739's earliest listed commit is 2026-11-22T06:23Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #740's earliest listed commit is 2026-11-22T06:23Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## crc-slice16: Portable CRC sliced by 16, lookups XORed as a balanced tree

- **PRs:** [#763](https://github.com/lmmx/mibz2/pull/763)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** Mechanisms handoff (per LOG 7088904119); Carbon named in PR
- **commit_to_build:** 2026-11-22T19:44Z, PR #763 first commit (author date) (https://github.com/lmmx/mibz2/pull/763/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- not found by the extractor

**must_hold** (quoted):

- not found by the extractor

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T19:44Z] "**Where it runs, and so what to expect from the bench:** - **x86_64 with PCLMULQDQ** (CI's bench machines, and the release targets' usual CPUs):" (https://github.com/lmmx/mibz2/pull/763 (PR body, current text))
- [2026-11-22T19:44Z] "Little or no change is expected in the x86 bench." (https://github.com/lmmx/mibz2/pull/763 (PR body, current text))
- [2026-11-22T19:44Z] "- **Elsewhere:** the gain is for builds without PCLMULQDQ (aarch64 wheels, older x86) and for `Kernels::scalar()`." (https://github.com/lmmx/mibz2/pull/763 (PR body, current text))

**same_quantity_facts:** 91 matching lines in 66 documents dated before commit_to_build, searched on `CRC|slic(ed|ing)|table lookups` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T12:34Z] comment 6978698021 (#162 coordinator): "- wall: +17% against eager Cells at one thread and +9% at four, −7–8% against LazyCells; libbz2 parity and the CRC rejection checks." — https://github.com/lmmx/mibz2/pull/162#issuecomment-6978698021
- [2026-11-18T12:44Z] comment 6978935386 (#145 researcher): "7. **Evidence — VERIFIED as recorded.** At threshold 16 the frozen counters give 3,448 wide pieces, 810,057 occurrences, 50,709 traced letters and 11,215 endpoint steps. Large-block medians are 88.6689% area and 5.4943% traced. The near-density pair gives 57,626 vs 34,775 visits. Peak additional re…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6978935386
- [2026-11-18T14:59Z] docs/journal/2026-11-18-switch-elimination/README.md (doc): "The study uses mibz2 0.5.5 at `f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71`, Rust 1.90.0 and Python 3. The three research kernels are installed into a disposable checkout. Their normal entry point is `mibz2::decode_with`, including the scanner, Huffman, MTF/RLE2, inverse BWT, RLE1 and the block and st…" — https://github.com/lmmx/mibz2/blob/9cd25610131b/docs/journal/2026-11-18-switch-elimination/README.md
- [2026-11-18T22:25Z] docs/journal/monodromy/2026-11-18-astra-d-verification.md (doc): "# 2026-11-18 · MONODROMY 74 · Astra D (#600, memory traffic) verified independently. My own cache simulator, written from the stated model, reproduces 7.328 / 5.595 / 3.439 B/B and every DRAM count at 1, 2 and 4 MiB from the archived traces. The forward tiles, the RLE1 edits and the mixed-direction…" — https://github.com/lmmx/mibz2/blob/7f6db3981ff2/docs/journal/monodromy/2026-11-18-astra-d-verification.md
- [2026-11-18T22:31Z] docs/journal/2026-11-18-astra-d-memory.md (doc): "**An exact reconstruction can halve modelled DRAM traffic on the frozen block.** A 260,492-byte table reconstructs the block in text order and streams RLE1 and CRC. It needs **3.439 DRAM bytes per final byte**, against **7.328** for shipped Cells. The compulsory input-plus-output floor is **1.046**…" — https://github.com/lmmx/mibz2/blob/4f7d02350259/docs/journal/2026-11-18-astra-d-memory.md
- [2026-11-18T22:31Z] docs/journal/2026-11-18-astra-d-memory/proof.md (doc): "The forward walk emits each literal immediately. After four copies of a byte, the next token supplies its extra repeat count and resets the streak. A final unfinished run returns the same error as the worker. CRC updates cover the newly produced slice after each 4,096 inverse rows, including a run …" — https://github.com/lmmx/mibz2/blob/4f7d02350259/docs/journal/2026-11-18-astra-d-memory/proof.md
- [2026-11-18T22:36Z] docs/journal/monodromy/2026-11-18-astra-a-verification.md (doc): "# 2026-11-18 · MONODROMY 75 · Astra A (#599, permanent-domain composition) verified independently. Every frozen and corpus count reproduces exactly. The retirement proof holds. A per-row audit of every frozen installation finds 0 failures. Output and CRC are exact on all 233 blocks. The model's laz…" — https://github.com/lmmx/mibz2/blob/499d059e757a/docs/journal/monodromy/2026-11-18-astra-a-verification.md
- [2026-11-18T22:44Z] docs/journal/astra-a-reinvention/results.md (doc): "A reuse sensitivity processes three waves, rotates the blocks among workers by one position per wave, preserves each worker's buffer addresses and all caches, then drains once. With a full 20 MiB available and seven workers, the three-wave total is 369,915 lines for whole domains and 544,170 for la…" — https://github.com/lmmx/mibz2/blob/fc5f294c9103/docs/journal/astra-a-reinvention/results.md
- [2026-11-20T18:55Z] docs/journal/2026-11-20-open-stage-attribution.md (doc): "- *The worker work:* about 507 ms (front + Open + RLE1 + CRC)." — https://github.com/lmmx/mibz2/blob/9d92de8a328d/docs/journal/2026-11-20-open-stage-attribution.md
- [2026-11-22T09:34Z] comment 7077232695 (#346 coordinator): "- its changes: owned run-route output, in-place RLE1, a tiled CRC, a 12 MiB ready cap, a ready ring and recycled input chunks;" — https://github.com/lmmx/mibz2/issues/346#issuecomment-7077232695

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** fused/run-20261117T115747Z (5.8 min; the CRC folded into RLE1 (#535): 10 threads (both arms 10/40, pinned) and 12 threads (7/28); adoption registration docs…); wormhole-plateau/run-20261122T085156Z (6.4 min; Wormhole's plateau stack (P2c: emit straight into the output, in-place RLE1, tiled CRC, W0 pool split, early symbols re…)

**Review routing:** Researcher: unrecorded (no comment from the role naming #763) / Optimiser: unrecorded (no comment from the role naming #763)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## rle1-zero-shift: In-place RLE1 with no upfront shift

- **PRs:** [#762](https://github.com/lmmx/mibz2/pull/762)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** Mechanisms handoff (Carbon, Wormhole named)
- **commit_to_build:** 2026-11-22T19:36Z, PR #762 first commit (author date) (https://github.com/lmmx/mibz2/pull/762/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 2 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.23; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- **Wormhole:** `copy_within` bytes down 46.6% (350.8 to 187.4 MB on its wikidata set), RLE1+CRC `0.8944x [0.8850, 0.9039]`, and 1.57% less wall at 4 workers." (https://github.com/lmmx/mibz2/pull/762 (PR body, current text))
- "`edge_runs_at_every_offset`, `random_blocks` (including the formerly staged growth past 2x) and `tile_boundaries`." (https://github.com/lmmx/mibz2/pull/762 (PR body, current text))

**must_hold** (quoted):

- "The bytes around the run must be left untouched." (https://github.com/lmmx/mibz2/pull/762 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T19:36Z] "The block is cut there: where the lead is <= 0 the stretches and runs are written forward (copied down, or left in place at 0); a zone where it is > 0 is written backward from its end (where the lead is 0 again, or the block's end), every stretch copied up." (https://github.com/lmmx/mibz2/pull/762 (first commit message))
- [2026-11-22T19:37Z] "The block is cut at those stretches:" (https://github.com/lmmx/mibz2/pull/762 (PR body, current text))
- [2026-11-22T19:37Z] "I cross-checked the two: they cut the zones at the same runs, and they size the buffer to the same `max(n, len)`." (https://github.com/lmmx/mibz2/pull/762 (PR body, current text))
- [2026-11-22T19:37Z] "- **Changed expectation:** `examples/occupancy.rs`'s `actual_workers_export_route_history` used to assert that its one-byte block is staged through `ll`." (https://github.com/lmmx/mibz2/pull/762 (PR body, current text))
- [2026-11-22T19:37Z] "It now expects no staged block (second commit)." (https://github.com/lmmx/mibz2/pull/762 (PR body, current text))

**same_quantity_facts:** 128 matching lines in 81 documents dated before commit_to_build, searched on `RLE1|upfront shift|in.place` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-18T22:44Z] docs/journal/astra-a-reinvention/results.md (doc): "A reuse sensitivity processes three waves, rotates the blocks among workers by one position per wave, preserves each worker's buffer addresses and all caches, then drains once. With a full 20 MiB available and seven workers, the three-wave total is 369,915 lines for whole domains and 544,170 for la…" — https://github.com/lmmx/mibz2/blob/fc5f294c9103/docs/journal/astra-a-reinvention/results.md
- [2026-11-19T12:07Z] docs/journal/monodromy/2026-11-19-neon-round6-verification.md (doc): "# 2026-11-19 · MONODROMY 82 · Neon's run-collision floor F_self_ns is valid only for in-place, write-once, Φ-sourced fragment schedules; outside that class it fails (a 16-byte text needs 12 Φ fragments but 4 LZ77 copies). Inside it, no constructed schedule reaches it, but per-run potentials need ab…" — https://github.com/lmmx/mibz2/blob/d44742a20ed0/docs/journal/monodromy/2026-11-19-neon-round6-verification.md
- [2026-11-20T12:02Z] docs/team/research-charter/OPEN-QUESTIONS.md (doc): "- Record 82 built valid in-place Φ-copy schedules of 0.87–2.15r fragments of about 7 B. That is" — https://github.com/lmmx/mibz2/blob/f5961bfb0443/docs/team/research-charter/OPEN-QUESTIONS.md
- [2026-11-20T18:55Z] docs/journal/2026-11-20-open-stage-attribution.md (doc): "- *The worker work:* about 507 ms (front + Open + RLE1 + CRC)." — https://github.com/lmmx/mibz2/blob/9d92de8a328d/docs/journal/2026-11-20-open-stage-attribution.md
- [2026-11-21T23:06Z] docs/team/research-charter/CONTEXT.md (doc): "\| In-place Φ copy schedule (round 6) \| Potential τ = θ + φ(run) \| Valid schedules of 0.87–2.15r fragments, but building τ needs text-order run labels, an O(n) arc walk, and run labels determine the text (record 84) \| records 82, 84 \|" — https://github.com/lmmx/mibz2/blob/ba0d959ebbdd/docs/team/research-charter/CONTEXT.md
- [2026-11-22T03:41Z] docs/journal/2026-11-22-neon-teamwork-paradigm.md (doc): "1. **`Lazard-BWT` ($\kappa = 1, e(\Sigma_R) = 0$, Alpha):** BWT–Lazard Commutation Theorem ($\mathrm{BWT}(\Phi_x(T)) = \mathrm{Splice}_x(\mathrm{BWT}(T))$ at $\min/\max$) + $Q=0$ extreme-bucket self-loop queue resolution (fixed points strictly in prefix of $F_{\min}$ / suffix of $F_{\max}$, verifie…" — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/journal/2026-11-22-neon-teamwork-paradigm.md
- [2026-11-22T03:41Z] docs/journal/2026-11-22-neon-teamwork-paradigm/checkpoint-r1.md (doc): "- Verified on `26,439` small cases + all 6 authenticated Wikidata blocks (`D = 450–883` rounds, `tot_syms = 0.76r–1.04r`, `94–96%` in-place single-run overwrites)." — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/journal/2026-11-22-neon-teamwork-paradigm/checkpoint-r1.md
- [2026-11-22T09:34Z] comment 7077232695 (#346 coordinator): "- its changes: owned run-route output, in-place RLE1, a tiled CRC, a 12 MiB ready cap, a ready ring and recycled input chunks;" — https://github.com/lmmx/mibz2/issues/346#issuecomment-7077232695
- [2026-11-22T09:41Z] comment 7077305378 (#145 coordinator): "**Composition at 10 threads (reported):** 0.895 here against 0.932 on the old base, a ratio of 0.960 [0.938, 0.983]. On top of Wormhole's stack the block reader gains *more*, not less. With #739's memory cuts in place, the serial scanner was even more the binding cost." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7077305378
- [2026-11-22T11:15Z] docs/team/research-charter/FRONTIER.md (doc): "\| Direct Φ-copy fragments are cyclic (85–94% on dependency cycles) \| In-place write-once Φ replay at F_self_ns \| round 6, attainability \|" — https://github.com/lmmx/mibz2/blob/47b370190663/docs/team/research-charter/FRONTIER.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** fused/run-20261117T115747Z (5.8 min; the CRC folded into RLE1 (#535): 10 threads (both arms 10/40, pinned) and 12 threads (7/28); adoption registration docs…); wormhole-plateau/run-20261122T085156Z (6.4 min; Wormhole's plateau stack (P2c: emit straight into the output, in-place RLE1, tiled CRC, W0 pool split, early symbols re…)

**Review routing:** Researcher: unrecorded (no comment from the role naming #762) / Optimiser: unrecorded (no comment from the role naming #762)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## mtf-u128: MTF of an index below 15 in one u128 register

- **PRs:** [#765](https://github.com/lmmx/mibz2/pull/765)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** Mechanisms handoff (Carbon named)
- **commit_to_build:** 2026-11-22T19:51Z, PR #765 first commit (author date) (https://github.com/lmmx/mibz2/pull/765/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "CI's bench is the authority):** - **`mtf_scalar`:** `0.7001x` on `wikidata-l9` and `0.9567x` on `records-l9`, against cycle 56's kernel." (https://github.com/lmmx/mibz2/pull/765 (PR body, current text))
- "- **`runs::front_scalar`:** `0.6773x` on `wikidata-l9`." (https://github.com/lmmx/mibz2/pull/765 (PR body, current text))
- "- **Full scalar decode:** `0.6878x` at 2 threads with `Kernels::scalar()`." (https://github.com/lmmx/mibz2/pull/765 (PR body, current text))

**must_hold** (quoted):

- "- **Full scalar decode:** `0.6878x` at 2 threads with `Kernels::scalar()`." (https://github.com/lmmx/mibz2/pull/765 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T19:51Z] "Little or no change is expected in the x86 bench." (https://github.com/lmmx/mibz2/pull/765 (PR body, current text))
- [2026-11-22T19:51Z] "The gain is for builds without AVX2: aarch64 wheels, older x86 and `Kernels::scalar()`. ## Tests - **New:** `mtf::tests::mtf_move_against_copy_within` checks `mtf_move_fast15` against the `copy_within` move, comparing the whole list and the returned byte each time:" (https://github.com/lmmx/mibz2/pull/765 (PR body, current text))

**same_quantity_facts:** 0 matching lines in 0 documents dated before commit_to_build, searched on `u128|scalar (MTF|move)|index below 1[56]` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- none

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #765) / Optimiser: unrecorded (no comment from the role naming #765)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## huff-table-builder: Huffman table builder: tail-only SubLut widths, slice-zip pair clone

- **PRs:** [#778](https://github.com/lmmx/mibz2/pull/778)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** Carbon cycle 49 (Mechanisms handoff)
- **commit_to_build:** 2026-11-23T00:54Z, PR #778 first commit (author date) (https://github.com/lmmx/mibz2/pull/778/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 1 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.01; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Its journal measured cycle 49's keep (this change together with a block.rs syms.resize tiling, not ported here) at P1 0.9790x on wikidata-l9, 0.9992x on wikidata-l1 and 0.9882x on records-l9; the build is 4.7% / 17.9% / 0.4% of P1 there." (https://github.com/lmmx/mibz2/pull/778 (first commit message))
- "It gives standalone P1 (block read) at **0.9790x** on wikidata-l9, **0.9992x** on wikidata-l1 and **0.9882x** on records-l9." (https://github.com/lmmx/mibz2/pull/778 (PR body, current text))
- "- **The builder's share of P1:** 4.7% on wikidata-l9, 17.9% on wikidata-l1 and 0.4% on records-l9." (https://github.com/lmmx/mibz2/pull/778 (PR body, current text))

**must_hold** (quoted):

- "Even a large speedup of the builder alone can only move end-to-end figures a little, and most on low-level (`-1`) streams." (https://github.com/lmmx/mibz2/pull/778 (PR body, current text))
- "Symbols and errors must match between the two builders and between the two decoders." (https://github.com/lmmx/mibz2/pull/778 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-23T00:55Z] "That removes the per-element bounds checks, so the loop can vectorise." (https://github.com/lmmx/mibz2/pull/778 (PR body, current text))

**same_quantity_facts:** 47 matching lines in 33 documents dated before commit_to_build, searched on `table builder|SubLut|Huffman` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-09T21:23Z] comment 6751828863 (#145 coordinator): "- **First, a survey of what has already been tried.** Multi-symbol Huffman is already at two symbols per refill, capped at 1 c/B or less." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6751828863
- [2026-11-10T00:21Z] comment 6755194206 (#145 coordinator): "- **Why it matters:** at 10 threads on the i9, the single Huffman scanner feeds every worker. Your own estimate says ten workers at about 6.7 c/B outrun one scanner, so the scanner, not the walk, may be the next ceiling." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6755194206
- [2026-11-10T04:03Z] comment 6758640506 (#145 optimizer): "**A. Prioritise #324 rescue; its FAIL stands.** Independently replayed all four archived fallback profiles. +61 Ir/block decomposes into **+6 caller self, +7 memset, +48 newly outlined finish**. Base calls RLE/CRC directly; head calls finish. MTF and room inclusive/self/callee counts match. Thus +6…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6758640506
- [2026-11-10T05:39Z] comment 6759983287 (#160 coordinator): "1. **Decompose R** into the components the levels change: per-worker live table bytes (≈0.8 against 3.6 MB), block count and per-block fixed costs, Huffman tables, and the scanner feed. Use the load law and the archive's per-run CPU. Which share is table bytes?" — https://github.com/lmmx/mibz2/pull/160#issuecomment-6759983287
- [2026-11-10T05:46Z] comment 6760081653 (#145 optimizer): "**The deciding check fails quantitative transfer.** Old production X, without fitting new coefficients, predicts -9 growth at6/8/10 of1.989/4.193/6.462; observed/predicted=.812/.627/**.505**. For -2 a disclosed flat extension below the fitted range predicts1.068 at10 versus1.183 observed; strict fi…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6760081653
- [2026-11-15T14:41Z] comment 6896264451 (#222 coordinator): "5. A correction the coordinator took: it proposed multi-symbol Huffman and a scatter-free build from a stale 27 October journal; both are already done or cheap today (pairs per lookup in #24, registers in #204; build 1.2 c/B). Louis, via ChatGPT, caught it. Lesson: check the current code and the la…" — https://github.com/lmmx/mibz2/pull/222#issuecomment-6896264451
- [2026-11-16T03:17Z] comment 6906303974 (#145 pathfinder): "**One fact from the code that bears on the reading.** The scanner decodes each block's Huffman symbols itself, in series, and waits when the window of blocks issued and not yet taken is full (`scanner`, `src/parallel.rs`). Per copy it needs about 0.05 to 0.06 s of one thread. Ten lazy workers need …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6906303974
- [2026-11-18T14:59Z] docs/journal/2026-11-18-switch-elimination/README.md (doc): "The study uses mibz2 0.5.5 at `f9bbd135e8e5512e9b1335e18468e1f3fd5a1d71`, Rust 1.90.0 and Python 3. The three research kernels are installed into a disposable checkout. Their normal entry point is `mibz2::decode_with`, including the scanner, Huffman, MTF/RLE2, inverse BWT, RLE1 and the block and st…" — https://github.com/lmmx/mibz2/blob/9cd25610131b/docs/journal/2026-11-18-switch-elimination/README.md
- [2026-11-22T11:15Z] docs/team/research-charter/FRONTIER.md (doc): "\| Starting from the decoded column \| Teamwork (`Eta`): proved EOB Causal Barrier (Thm H1: no Stage 3+ can start before EOB) and RLE2–MTF semidirect-product commutation (Thm H2). Full Huffman+MTF+Pass1 fusion (`HuffMonoidBL-v2`) regressed `mibz2-scan` by `+45%` (`1.185×` e2e 1T, `1.186×` 4W) becau…" — https://github.com/lmmx/mibz2/blob/47b370190663/docs/team/research-charter/FRONTIER.md
- [2026-11-23T00:26Z] comment 7088353720 (#145 coordinator): "\| Huffman decoding fused with the run scan \| no \| NO-GO: its own measurement killed it (1.185× slower) \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7088353720

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #778) / Optimiser: unrecorded (no comment from the role naming #778)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## parscan-stream: Push-streaming parallel front end, default on the i9 from 8 threads

- **PRs:** [#770](https://github.com/lmmx/mibz2/pull/770), [#771](https://github.com/lmmx/mibz2/pull/771)
- **Journal sources:** docs/journal/2026-11-22-parscan-stream-registration.md
- **i9 recipes:** parscan-stream
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** Carbon (Mechanisms handoff; per PR #770)
- **commit_to_build:** 2026-11-22T21:15Z, PR #770 first commit (author date) (https://github.com/lmmx/mibz2/pull/770/commits)
- **Cost:** i9 23.6 over 1 bundle(s) min [parscan-stream/run-20261123T005713Z: 23.6 min (started→finished.txt)]; CI: 5 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.17; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "SlabRope (recycled 4 MiB slabs, at most 8 MiB live), a horizon of workers + 4 blocks, the single-slab scanner, and ParScanHub's lead scanner with one or two demand-driven helpers;" (https://github.com/lmmx/mibz2/pull/770 (first commit message))
- "`block::read_item`'s only level-dependent check is the original pointer against `100,000 x level`, which it makes before reading the rest of the block." (https://github.com/lmmx/mibz2/pull/770 (PR body, current text))
- "**The livelock guard:** each `NeedMore` asks for strictly more bytes than the last read had (at least twice, from 512 KiB, to `MAX_ITEM`), the head promotes it once the bytes or the end arrive, and the lead pulls input past `MAX_ROPE_LIVE` for the head; a block cannot need more than about 2.3 MB, s…" (https://github.com/lmmx/mibz2/pull/770 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-parscan-stream: "the streaming parallel front end (#770 at 2618c54c, merge of master 6f7b2ac4) against its base 6f7b2ac4 (0.5.13's src), unpinned at 1, 2, 4, 6, 8, 10, 12, 16 and 20 threads, wikidata x4 and text, 16 r"
- "perf(parscan): the push-streaming parallel front end, the default on the i9 from 8 threads Ports the parallel front end of Carbon's final handoff (cycle 58), research/carbon-word-transport (docs/journal/2026-11-22-carbon-final-handoff/):" (https://github.com/lmmx/mibz2/pull/770 (first commit message))
- "StreamDecoder::new takes the front end in place of its serial scanner only on the measured i9's profile from 8 threads asked, where its topology has a helper scanner (parallel::parscan_here); below 8 threads and off the i9 the default decoder is master's, unchanged." (https://github.com/lmmx/mibz2/pull/770 (first commit message))
- "Tests: tests/parscan_stream.rs (dispatch, thread counts 1-20 on multi-block multi-stream input, magics on slab edges, every truncation, the level re-read, the end-of-input race) and parallel::front_end_tests; the serial scanner's i9 dispatch tests hold a test-only SerialScannerOnly switch." (https://github.com/lmmx/mibz2/pull/770 (first commit message))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-22T21:21Z] "**Names.** The module's comments describe the mechanism; no team, model or round names in identifiers or comments. ## Invariants (the PR 4 `BlockSpan` list in the `research/carbon-wormhole` final handoff, Action 3) 1." (https://github.com/lmmx/mibz2/pull/770 (PR body, current text))
- [2026-11-22T21:21Z] "Tested by `magics_on_slab_edges` (every magic cut at -9..+9 bytes, slabs of real 100+ KiB blocks) and the existing prime/arbitrary chunking tests. 4." (https://github.com/lmmx/mibz2/pull/770 (PR body, current text))
- [2026-11-22T21:21Z] "`tests/service_default.rs` expects 8 workers at 10-20 threads on the i9 (7 before)." (https://github.com/lmmx/mibz2/pull/770 (PR body, current text))

**same_quantity_facts:** 87 matching lines in 69 documents dated before commit_to_build, searched on `streaming|parallel front end|parscan|scanner wall|handoff` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-22T03:41Z] docs/journal/2026-11-22-neon-teamwork-paradigm.md (doc): "2. **`1Cut-BWT` ($\kappa = 1, e(\Sigma_R) \ge 0$, Alpha×Beta):** Zero bucket splits, cuts `Lazard` streaming volume `5–7.6×` (`17.3–41.2 MB`) and `BDMR` peak $\sigma_t$ by `37–40%` (`7.2k–17.8k`), yet still pays $Q_{\text{mask}} = 5.82\text{–}6.86\,r \gg m_{16}$ and fails $n_{\text{next}} \le 3n/4$…" — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/journal/2026-11-22-neon-teamwork-paradigm.md
- [2026-11-22T05:22Z] comment 7074713448 (#145 coordinator): "- **Decision:** #734 is **deferred, not abandoned.** The stack saves 4–5% CPU at every count, but at 8 and 10 threads that doesn't become wall time. This is the release-scaling plateau again: wall flat at 68–73 ms from 8 to 20 threads, which points to a serial or shared stage capping the pipeline. …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7074713448
- [2026-11-22T08:17Z] comment 7076427071 (#145 coordinator): "- **Named risk:** the parscan arm holds the whole 47 MiB input in memory, so the RSS gate may bind on that alone." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7076427071
- [2026-11-22T08:41Z] comment 7076671871 (#145 coordinator): "- **Why:** the arm holds the whole input and up to 4×threads blocks in flight. The fix is Mechanisms' push-streaming Handoff 1 (cycles 39–43), already prototyped at cycle 40." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7076671871
- [2026-11-22T08:48Z] comment 7076744504 (#145 coordinator): "- Decision 1, PARSCAN, NOT ADOPTED: 6 threads 1.11 SLOWER, RSS 1.28 at 10 threads." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7076744504
- [2026-11-22T09:34Z] comment 7077232695 (#346 coordinator): "- **Wormhole's claim, which is a hypothesis and not evidence:** `trimmed_here` caps every unpinned count from 10 to 20 threads at 7 workers / 28 slots. With memory and synchronisation fixed, Wormhole predicts 10 workers with a 12 MiB cap win (handoff `8ec6f94c2`, W0 / PR 5)." — https://github.com/lmmx/mibz2/issues/346#issuecomment-7077232695
- [2026-11-22T09:40Z] docs/journal/2026-11-22-parscan-c37-registration.md (doc): "The judged count was FASTER, with a transfer of 1.798 [1.320, 2.276] of the handoff's claimed" — https://github.com/lmmx/mibz2/blob/4a79f0631e3c/docs/journal/2026-11-22-parscan-c37-registration.md
- [2026-11-22T11:15Z] docs/team/research-charter/FRONTIER.md (doc): "\| 2026-11-22 \| Neon Teamwork (`research/neon-teamwork-paradigm`, [journal](../../journal/2026-11-22-neon-teamwork-paradigm.md); 7 investigators + Adversarial Referee + SAT/CP-SAT audit) \| `Lazard-BWT`, `1Cut-BWT`, `BDMR` + `Open × BDMR` hybrid + CP-SAT/Dijkstra global schedules; `Phi_BWT` `TC⁰`/…" — https://github.com/lmmx/mibz2/blob/47b370190663/docs/team/research-charter/FRONTIER.md
- [2026-11-22T19:40Z] comment 7084586885 (#145 researcher): "- **What the registration quotes:** the Round 5 handoff (8ec6f94c2, "10W / 12 MiB cap"), and it calls 9/36 "between them"." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7084586885
- [2026-11-22T19:40Z] comment 7084590503 (#145 coordinator): "- **Wormhole's hypothesis** is quoted from its cycle-70 final handoff: 9W/27 at 53.2 ms against 62.4 ms for 7/28, and 10W/30 only after PR 4." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7084590503

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: 2 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-7088432559 (2026-11-23T00:32Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #770, #771, bench-parscan-stream)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

## open-routes: Open construction routes from Neon's teamwork handoff: mark-free wiring, lean small-block path (98,304 line)

- **PRs:** [#781](https://github.com/lmmx/mibz2/pull/781), [#782](https://github.com/lmmx/mibz2/pull/782), [#783](https://github.com/lmmx/mibz2/pull/783)
- **Journal sources:** none listed
- **i9 recipes:** none
- **Deciding machine:** CI paired bench
- **Origin (as recorded):** Neon's teamwork handoff
- **commit_to_build:** 2026-11-23T01:04Z, PR #782 first commit (author date) (https://github.com/lmmx/mibz2/pull/782/commits)
- **Cost:** i9 0 min [no i9 bundle named]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.03; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Source: the small-block path of AdaptiveLeanOpen from the foundational contraction programme on research/neon-teamwork-paradigm (cycle 18, adaptive_lean_open_candidate.rs: band coalescing, one-write wire and one-ended emit up to 384 KiB of edges)." (https://github.com/lmmx/mibz2/pull/782 (first commit message))
- "The line was chosen there on a 2-vCPU Xeon with 2 MiB of L2." (https://github.com/lmmx/mibz2/pull/782 (first commit message))
- "The output was identical. ## The threshold 98,304 rows is the candidate's `DEFAULT_M_UNI_MAX`, chosen on a 2-vCPU Xeon 8481C with **2 MiB of L2**, where 384 KiB of `edges` fits in L2." (https://github.com/lmmx/mibz2/pull/782 (PR body, current text))

**must_hold** (quoted):

- "Miri not run here (no nightly); the change adds no `unsafe`. ## Measurement CI's paired bench decides, at 1, 2 and 4 threads, under PERFORMANCE-POLICY.md." (https://github.com/lmmx/mibz2/pull/781 (PR body, current text))
- "That rationale does not carry to the i9-10900K (256 KiB of L2), where master's 65,536 (256 KiB of `edges`) is the L2 size." (https://github.com/lmmx/mibz2/pull/782 (PR body, current text))
- "Blocks between the two lines are one-ended past the i9's L2 with this PR, which the same programme's "Law 4" says can cost (one chain's memory-level parallelism instead of two)." (https://github.com/lmmx/mibz2/pull/782 (PR body, current text))
- "The handoff itself recommends sweeping the line on the i9 (65,536, 98,304, 131,072)." (https://github.com/lmmx/mibz2/pull/782 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-23T01:04Z] "Tests: the kernel against the plain inverse on columns of exactly 65,535 to 65,537 and 98,303 to 98,305 retained rows, with and without thick runs, with the mark in each thin byte, checking the route taken; coalesced against built bands (a test-only switch) on every ternary column of 7 rows, run columns, text columns and full columns past the line: the same bytes, every rank to the same row with …" (https://github.com/lmmx/mibz2/pull/782 (first commit message))
- [2026-11-23T01:12Z] "Master's large Wikidata blocks take this route, so `wikidata` is the corpus it should move." (https://github.com/lmmx/mibz2/pull/781 (PR body, current text))
- [2026-11-23T01:12Z] "Both give the plain inverse's bytes; every retained rank goes to the same row with the same spelled word; coalesced bands end only where built ones do, and are no more; and some column has fewer." (https://github.com/lmmx/mibz2/pull/782 (PR body, current text))
- [2026-11-23T01:12Z] "- `coalesced_bands_send_each_rank_where_the_built_ones_do`: coalesced against built bands on small, run, text and full columns: the same bytes, every rank to the same row and word, no more bands, fewer on some." (https://github.com/lmmx/mibz2/pull/783 (PR body, current text))

**same_quantity_facts:** 24 matching lines in 16 documents dated before commit_to_build, searched on `mark.free|one.ended|98,304|65,536|coalesc` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-11T10:41Z] comment 6793646018 (#145 researcher): "Draft falsifier fixes quote byte34 and the first >=65,536-row block in the exact hash-pinned #367 corpus, with Segmented57f51bbc baseline. Export/adapter and block/run digests must be frozen before execution; not yet run-ready. Charge E/H/Q, prefix, boundary touches/splits, searches, copies, live m…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6793646018
- [2026-11-12T12:06Z] comment 6823145996 (#145 louis via chatgpt): "At a 75% hash-table load threshold, doubling from 65,536 to 131,072 slots while both tables are live creates a temporary allocation peak. In the specified model, that alone brings the accounted working set above the original \(4n\) limit." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6823145996
- [2026-11-12T16:42Z] docs/journal/2026-11-12-rauzy-companion/source/rauzy-technical-report-source.md (doc): "At a 75% hash-table load threshold, doubling from 65,536 to 131,072 slots while both tables are live creates a temporary allocation peak. In the specified model, that alone brings the accounted working set above the original \(4n\) limit." — https://github.com/lmmx/mibz2/blob/5d8f11c7d4b8/docs/journal/2026-11-12-rauzy-companion/source/rauzy-technical-report-source.md
- [2026-11-14T16:04Z] docs/journal/monodromy/2026-11-14-feathered-screen/summary.md (doc): "\| largest r_new before coalescing / r \| 0.657 / 0.675 / 0.678 / 0.681 / 0.691 \|" — https://github.com/lmmx/mibz2/blob/920cdf1b1958/docs/journal/monodromy/2026-11-14-feathered-screen/summary.md
- [2026-11-16T17:45Z] comment 6924288835 (#145 pathfinder): "\| §4 coalescing \| 0.953 commands \| 4.7% merges \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6924288835
- [2026-11-18T12:30Z] docs/journal/2026-11-18-astra-investigation-1.md (doc): "Wikidata blocks of at least 65,536 rows, the median r/n is **0.0623**, 82% of blocks are below" — https://github.com/lmmx/mibz2/blob/d4642042ccf8/docs/journal/2026-11-18-astra-investigation-1.md
- [2026-11-19T08:29Z] comment 7003886829 (#145 researcher): "- **Range count.** For one block, let `S` be the number of shared interior run/context endpoints. There are `r+C−1−S` overlay pieces and at least `C` modal pieces, so `D≤r−1−S` for the coalesced physical exception ranges. At k=1 the archived arithmetic is 690,484 input runs + 77 overlay cuts − 43,1…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7003886829
- [2026-11-20T21:09Z] docs/journal/2026-11-20-carbon-depth-audit.md (doc): "non-coalescence of one-row runs). The 0.29–0.59 ms floor, and so the "Open sits 4.5–6.1× above" — https://github.com/lmmx/mibz2/blob/4ab98cd75eda/docs/journal/2026-11-20-carbon-depth-audit.md
- [2026-11-22T03:41Z] docs/journal/2026-11-22-neon-teamwork-paradigm.md (doc): "3. **Referee's $\Theta(r_B^2)$ self-run stack gap (`L_quad`) `[verified]`:** Unordered DFS stack takes `0.52 q^2` pops (`3,320` pops = `10.44 r_0` at $q=80$); fixed by monotone spatial sweep with equal-$k$ coalescing ($O(r_B)$) / lazy treap ($O(r_B \log r_B)$)." — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/journal/2026-11-22-neon-teamwork-paradigm.md
- [2026-11-23T00:26Z] comment 7088353720 (#145 coordinator): "\| One-ended emit: hoisted spill check, cold long-word path, an early index, length carried in the edge \| no, but large Wikidata blocks run the **two-ended** emit (`src/open.rs:521-522, 630`), which is about 2.7% faster (Kitchen Sink cycle 45 §3.2) \| NO-GO: the claimed 2.8–4.4% would net to about…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7088353720

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** none found

**Review routing:** Researcher: unrecorded (no comment from the role naming #781, #782, #783) / Optimiser: unrecorded (no comment from the role naming #781, #782, #783)

**monodromy_involved:** no citation found

**coordinator_involved:** no citation found

**Notes:** PR #781's earliest listed commit is 2026-11-23T00:45Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used. PR #783's earliest listed commit is 2026-11-23T00:45Z, 0 h before it opened and is listed on other PRs too; it may belong to a base branch, so it is not used.

## tiled-zero-fill: Tiled zero-fill of the block reader's symbols (split out of #770)

- **PRs:** [#785](https://github.com/lmmx/mibz2/pull/785), [#786](https://github.com/lmmx/mibz2/pull/786)
- **Journal sources:** docs/journal/2026-11-23-block-symbols-tiled-zero-fill.md
- **i9 recipes:** block-symbols-tiled-zero-fill
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** split out of #770 by the Coordinator (LOG 7089082075, 'A separate lead'); Carbon's code
- **commit_to_build:** 2026-11-23T01:21Z, commission 7089082075 (https://github.com/lmmx/mibz2/pull/145#issuecomment-7089082075)
- **Cost:** i9 16.4 over 1 bundle(s) min [block-symbols-tiled-zero-fill/run-20261123T091653Z: 16.4 min (started→finished.txt)]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.34; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "Peak RSS falls 12–17%, which fits fewer blocks in flight." (https://github.com/lmmx/mibz2/pull/145#issuecomment-7089082075)
- "perf(block): zero-fill the block reader's symbols in tiles of 8,192, just ahead of the groups `read_item_with_scratch`, the block reader of the serial scanner and of the one-thread decoder, zero-filled its symbol buffer to `max_out` (up to 900,066 symbols, 1.8 MB) before the first group, then wrote…" (https://github.com/lmmx/mibz2/pull/785 (first commit message))
- "Wikidata 0.978 [0.971, 0.984] at 4 threads and 0.977 [0.971, 0.983] at 6 (evidence f693e20b, parscan-stream/run-20261123T005713Z)." (https://github.com/lmmx/mibz2/pull/785 (first commit message))

**must_hold** (quoted):

- queue entry bench-block-symbols-tiled-zero-fill: "#785's tiled zero-fill of the block reader's symbols (perf/block-symbols-tiled-zero-fill 39aaf55b) against master's src ff0fd5c7 (#785's merge-base), unpinned above 1 at 1, 2, 4, 6, 8 and 10 threads,"
- "The activation census is as registered: the helper scanner appears in the head at 8 threads and up, 8 workers run from 10 threads, and neither appears in the base." (https://github.com/lmmx/mibz2/pull/145#issuecomment-7089082075)
- "Wikidata 0.978 [0.971, 0.984] at 4 threads and 0.977 [0.971, 0.983] at 6 (evidence f693e20b, parscan-stream/run-20261123T005713Z)." (https://github.com/lmmx/mibz2/pull/785 (first commit message))
- "- `pub(crate)` visibility. ## Why: the i9 evidence #770's run on the i9, head against master, in the cells where only this change ran." (https://github.com/lmmx/mibz2/pull/785 (PR body, current text))

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-23T01:21Z] "Peak RSS falls 12–17%, which fits fewer blocks in flight." (https://github.com/lmmx/mibz2/pull/145#issuecomment-7089082075)
- [2026-11-23T02:01Z] "It now fills in tiles of 8,192 symbols (16 KiB, `ZERO_FILL_TILE`) inside the group loop, a branch a group, so that each tile's fill and its groups' writes meet in L1." (https://github.com/lmmx/mibz2/pull/785 (first commit message))
- [2026-11-23T02:01Z] "`symbols_read_alike_from_any_buffer` reads blocks of up to 35,000 symbols (several tiles, a cut, a flipped bit or a bad code in any of them) into buffers in every state a caller hands one over in, and checks the symbols, error, end bit and `overran` against the in-memory reference; tests/block_symbols.rs carries #770's truncation and wrong-level error-preservation tests onto the serial paths (one…" (https://github.com/lmmx/mibz2/pull/785 (first commit message))
- [2026-11-23T02:02Z] "- **`block::tests::symbols_read_alike_from_any_buffer` (new):** blocks of up to 700 selectors (35,000 symbols, so several tiles), with a cut, a flipped bit or a bad code in any of them." (https://github.com/lmmx/mibz2/pull/785 (PR body, current text))
- [2026-11-23T02:02Z] "Large random blocks are also cut and corrupted in every tile." (https://github.com/lmmx/mibz2/pull/785 (PR body, current text))
- [2026-11-23T02:03Z] "It covers the decision, the hypothesis and mechanism, what #770 carried and the split leaves out, the run, the rule, PERFORMANCE-POLICY 4a and the limits." (https://github.com/lmmx/mibz2/pull/786 (PR body, current text))

**same_quantity_facts:** 58 matching lines in 42 documents dated before commit_to_build, searched on `zero.fill|tiles? of 8,192|symbols|memset` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-14T23:15Z] comment 6885750451 (#145 observer): "3. #398 16 B slot closed unmerged after no primary wall gain and Wiki CPU+4.4% at4t; growth-only zero-fill step withdrawn. Tree-preserving locality reorder CLOSED on certified floor; AHT/Lackenby steelman failed." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6885750451
- [2026-11-16T03:17Z] comment 6906303974 (#145 pathfinder): "**One fact from the code that bears on the reading.** The scanner decodes each block's Huffman symbols itself, in series, and waits when the window of blocks issued and not yet taken is full (`scanner`, `src/parallel.rs`). Per copy it needs about 0.05 to 0.06 s of one thread. Ten lazy workers need …" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6906303974
- [2026-11-16T09:44Z] comment 6912912161 (#145 pathfinder): "\| `-cores` / `mibz2` at 20 \| 0.97–1.00: window 80 → 40, about 14 → 6 MB of queued symbols \|" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6912912161
- [2026-11-16T10:04Z] docs/journal/2026-11-16-sibling-front-end.md (doc): "\| not inflated (a 256-byte list and streamed symbols) \| 66-76 ms \| 0.09-0.10 \| 0.07-0.08 \|" — https://github.com/lmmx/mibz2/blob/cc2147679e3a/docs/journal/2026-11-16-sibling-front-end.md
- [2026-11-16T15:12Z] comment 6920355683 (#145 pathfinder): "The walk's tables were sized to r and zero-filled per worker. That is about 60 bytes a letter, and RSS went from 85 to 109 MB at 4 threads in a local smoke run. Now:" — https://github.com/lmmx/mibz2/pull/145#issuecomment-6920355683
- [2026-11-16T23:11Z] docs/journal/monodromy/2026-11-16-workers-prediction.md (doc): "So about **2.5 MB a worker** (2.4 MB with W12's 2.29 B a letter). Queued blocks' symbols are" — https://github.com/lmmx/mibz2/blob/bc301ceac52c/docs/journal/monodromy/2026-11-16-workers-prediction.md
- [2026-11-18T08:50Z] comment 6973651863 (#145 pathfinder): "2. **The buffer is cheap to begin with.** The pre-RLE text is resident when RLE1 reads it: `unrle` read-misses 0.004 B per output byte. Its write traffic is the zero-fill and text above, 0.44 B at most." — https://github.com/lmmx/mibz2/pull/145#issuecomment-6973651863
- [2026-11-18T22:31Z] docs/journal/2026-11-18-astra-d-memory.md (doc): "\| Scanner symbols \| 8,581 \| 6,372 \| 0 \| 2,805 \| 2,805 \| 0.399 \|" — https://github.com/lmmx/mibz2/blob/4f7d02350259/docs/journal/2026-11-18-astra-d-memory.md
- [2026-11-18T22:36Z] docs/journal/monodromy/2026-11-18-astra-a-verification.md (doc): "- *Adding back D's out-of-scope rows* (input, symbols, runs, MTF, final output, other; 2.86 B/B) gives" — https://github.com/lmmx/mibz2/blob/499d059e757a/docs/journal/monodromy/2026-11-18-astra-a-verification.md
- [2026-11-22T04:36Z] docs/journal/release-attribution.md (doc): "\| 0.2.7 → 0.2.8 \| 1t +12.7% (+11.6, +13.8); 2t +13.5%; 4t +8.8% \| #324, run-record front and LF table on blocks of few symbols \| kernel \| CI per merge: #324 +17%, +18%, +11%. Not at 8t and above, where teams ran without it \|" — https://github.com/lmmx/mibz2/blob/23c1e283483c/docs/journal/release-attribution.md

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** wormhole-plateau/run-20261122T085156Z (6.4 min; Wormhole's plateau stack (P2c: emit straight into the output, in-place RLE1, tiled CRC, W0 pool split, early symbols re…)

**Review routing:** Researcher: 1 comment(s); earliest https://github.com/lmmx/mibz2/pull/145#issuecomment-7096174288 (2026-11-23T09:03Z, after commit_to_build, before the first i9 bundle) / Optimiser: unrecorded (no comment from the role naming #785, #786, bench-block-symbols-tiled-zero-fill)

**monodromy_involved:** no citation found

**coordinator_involved:** yes: origin field names the Coordinator

## t-squared: The squared retained map (T²) in the Open kernel's emit

- **PRs:** [#787](https://github.com/lmmx/mibz2/pull/787), [#792](https://github.com/lmmx/mibz2/pull/792)
- **Journal sources:** docs/journal/2026-11-23-emit-cost-sq.md, docs/journal/monodromy/2026-11-23-squared-trace-arm.md, docs/journal/monodromy/2026-11-23-emit-cost-sq-review.md
- **i9 recipes:** squared-trace-prescreen, emit-cost-sq
- **Deciding machine:** i9-10900K
- **Origin (as recorded):** MONODROMY records 92-94; commissioned by the Coordinator as a C1a diagnostic
- **commit_to_build:** 2026-11-23T08:57Z, commission 7096050983 (https://github.com/lmmx/mibz2/pull/145#issuecomment-7096050983)
- **Cost:** i9 8.7 over 2 bundle(s) min [squared-trace-prescreen/20261123T093340Z: 0.2 min (started→finished.txt); emit-cost-sq/run-20261123T110350Z: 8.5 min (started→finished.txt)]; CI: 3 Benchmark workflow runs on the PR branches; calendar days, commit to verdict: 0.09; agent runs/tokens: unrecorded

**claimed_gain_source** (quoted):

- "- rustc 1.99.1, wikidata ×4, {base, sq, sq-shadow} × {1, 6, 10} × 20 rounds with a 2 s settle;" (https://github.com/lmmx/mibz2/pull/792 (PR body, current text))
- "Each is paired with base in the same round, taken over the open blocks, and given as the mean over 20 rounds with a 95% t interval:" (https://github.com/lmmx/mibz2/pull/792 (PR body, current text))
- "- The smoke used a 20 MB text of copies, at 1 and 2 workers and 3 rounds, with all 20 blocks two-ended." (https://github.com/lmmx/mibz2/pull/792 (PR body, current text))

**must_hold** (quoted):

- queue entry bench-squared-trace-prescreen: "MONODROMY's squared-trace pre-screen (record 92, squared-trace-prescreen.c at 3979c15e, sha256 4f3c62b7), gcc -O2 -march=native, m = 89874, 114000, 156404, 204281, 11 interleaved invocations each; DIA"
- queue entry bench-emit-cost-sq: "the T² two-ended emit arms on OpenCells (MONODROMY record 93 c3dab1ef, addendum b36ec511; review record 94 1b188ada): base, sq (serial square pass) and sq-shadow (square pass in the walk's latency sha"
- "- **Then the i9 run:** on PROCEED, or after one targeted correction and the Coordinator's decision, `ready/bench-block-symbols-tiled-zero-fill` is set and the i9 runs it, about 25 minutes." (https://github.com/lmmx/mibz2/pull/145#issuecomment-7096050983)
- "- **The question:** on the i9, does the squared retained map T², walked two-ended after one m-gather, beat the two-chain chase on synthetic Haar cycles at m ∈ {89,874; 114,000; 156,404; 204,281}?" (https://github.com/lmmx/mibz2/pull/145#issuecomment-7096050983)

**Premises** (quoted from the sources dated at or before commit_to_build; PR bodies are their current text):

- [2026-11-23T09:02Z] "I kept `-march=native` because the record used it." (https://github.com/lmmx/mibz2/pull/787 (PR body, current text))
- [2026-11-23T09:02Z] "- **Expected runtime:** about 14 s on this 4-core VM, 13 s of it CPU." (https://github.com/lmmx/mibz2/pull/787 (PR body, current text))
- [2026-11-23T10:18Z] "With them after the four copies, `sq`'s emit was about 2× base's on the synthetic table, because the next chain load fell outside the out-of-order window." (https://github.com/lmmx/mibz2/pull/792 (PR body, current text))
- [2026-11-23T10:18Z] "With them first it is about 1×." (https://github.com/lmmx/mibz2/pull/792 (PR body, current text))

**same_quantity_facts:** 341 matching lines in 176 documents dated before commit_to_build, searched on `T²|squared|dependent load|latency|emit cost|ns a row|lanes` with a measurement on the line. All are in `facts.tsv`; the latest line from each of the 10 latest documents:

- [2026-11-21T10:18Z] docs/journal/2026-11-21-the-shape-of-the-rh-proof.md (doc): "each worker gets. The fidelity ledger's crowd losses (lanes: +13% alone, −5% at 8 threads) are" — https://github.com/lmmx/mibz2/blob/b03c3f4bab9d/docs/journal/2026-11-21-the-shape-of-the-rh-proof.md
- [2026-11-21T23:06Z] docs/team/research-charter/CONTEXT.md (doc): "\| 8-lane emit with 4 KiB pages + R4 elimination at LIMIT 24 (Carbon cycles 9–11; #675) \| C: 0.43–0.57× of Open \| native one thread, whole corpus: Open 0.872× of 0.5.8's [0.820, 0.928]. i9 at 6/8/10 threads (open-lanes/run-20261121T023127Z): 0.950 FASTER, 1.049 SLOWER, 1.012 INCONCLUSIVE, so NOT …" — https://github.com/lmmx/mibz2/blob/ba0d959ebbdd/docs/team/research-charter/CONTEXT.md
- [2026-11-22T03:06Z] comment 7073325651 (#145 coordinator): "- LazyTunnel (Delta) breaks even at about 15.2 ns per dependent load." — https://github.com/lmmx/mibz2/pull/145#issuecomment-7073325651
- [2026-11-22T03:41Z] docs/journal/2026-11-22-neon-teamwork-paradigm.md (doc): "- *Algorithmic escape:* Format-level tunnel grouping, OR attacking serial `MLP=1` walk latency across all `m_16` rows (pointing directly to Epsilon's `BSAC`). *Deployment condition (not escape):* $L_{\text{load}} > 15.2\text{ ns}$ (`LazyTunnel`) / `16.0 ns` (`ZeroRV-Exact(w>=6)`)." — https://github.com/lmmx/mibz2/blob/5236308bd698/docs/journal/2026-11-22-neon-teamwork-paradigm.md
- [2026-11-22T03:51Z] comment 7073830805 (#145 coordinator): "b at 10 workers is 5.8 ns, below the 8 ns bar: LazyTunnel and BSAC are both shelved. The BSAC port in progress has been stopped unpublished; its partial diff is kept off-repo for a future registration. The i9's emit walk is not miss-bound at production widths: about 5–6 ns a row with 10 workers, cl…" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7073830805
- [2026-11-22T03:57Z] comment 7073889492 (#222 coordinator): "- **bench-emit-cost:** SHELVE BOTH (7073881414's predecessor LOG). The i9's emit costs 5.8 ns a row at 10 workers and is not miss-bound, so the BSAC port was stopped unpublished. Construction now outweighs emit on the i9." — https://github.com/lmmx/mibz2/pull/222#issuecomment-7073889492
- [2026-11-22T11:15Z] docs/team/research-charter/FRONTIER.md (doc): "\| Computing LF from a compact rank structure instead of storing the edges costs about four chained dependent loads per step against Open's one \| Implicit-rank (FM-index-style) successors as a replacement for Open's explicit edges, single-threaded. This is the known medium-space inverse BWT trade …" — https://github.com/lmmx/mibz2/blob/47b370190663/docs/team/research-charter/FRONTIER.md
- [2026-11-22T11:31Z] docs/team/research-charter/PROOF-PROGRAMME.md (doc): "acquisition costs more than emission saves (10.1–10.5 ns a saved hop against 5.79 ns a row;" — https://github.com/lmmx/mibz2/blob/e569f6734ca1/docs/team/research-charter/PROOF-PROGRAMME.md
- [2026-11-23T02:01Z] docs/journal/monodromy/2026-11-23-refinement-and-composition.md (doc): "- **Law 1 (under 5.8 ns an operation):** RV's 22.2 ns breaks it, and Theorem 4 turns it into a per-run statement. The T² pass is about 0.5–0.9 ns a row on the pre-screen: one gather, with no sort and no probe." — https://github.com/lmmx/mibz2/blob/3979c15ee7b8/docs/journal/monodromy/2026-11-23-refinement-and-composition.md
- [2026-11-23T02:02Z] comment 7089616426 (#145 monodromy): "**Modelled, from Neon's prices (RV 22.2 ns a step; emit 4.8–5.8 ns a row).**" — https://github.com/lmmx/mibz2/pull/145#issuecomment-7089616426

**Counterfactual real-block i9 measurements (≤ 10 min) before commit_to_build, matching the terms:** emit-cost/run-20261122T034734Z (3.1 min; the open kernel's emit cost per retained row (m16) in production's lazy workers at 1, 6 and 10, on master 69961392 with…); imc-ten/run-20261118T090842Z (1.4 min; lazy 10/40 against 7/28 unpinned, with IMC and L3-miss latency (#554; MONODROMY record 52 frozen before the data, REVIE…); miss-cost/run-20261117T104648Z (3.1 min; contention counters at 10 threads, W = 10/9/8/7, THP arm and load-latency sampling (#529; MONODROMY record 47 frozen be…); open-lanes/run-20261121T023127Z (7.5 min; the Open kernel's eight-lane walk into 4 KiB pages and R4 at LIMIT 24 (#675, research/open-bp4k 8ec1a92c) against maste…)

**Review routing:** Researcher: unrecorded (no comment from the role naming #787, #792, bench-squared-trace-prescreen, bench-emit-cost-sq) / Optimiser: unrecorded (no comment from the role naming #787, #792, bench-squared-trace-prescreen, bench-emit-cost-sq)

**monodromy_involved:** yes: cited in the case's own documents (records 92, 93, 94); records naming its PRs or recipes: 2026-11-23-emit-cost-sq-review.md, 2026-11-23-squared-trace-postmortem.md; origin field names MONODROMY

**coordinator_involved:** yes: origin field names the Coordinator

**Notes:** 2 i9 bundles over 2 recipe(s); coders check whether master moved between them.
