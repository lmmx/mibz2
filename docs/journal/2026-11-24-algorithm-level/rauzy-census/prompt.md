# Antigravity (Neon): the next task, in three parts: the Rauzy census, two pre-port tests, and a reading of the compact GRH proof

Work in the lmmx/mibz2 repository on a new branch `research/rauzy-census`, from `origin/master`. Use Louis's identity (`git config user.name "Louis Maddox"`, `git config user.email louismmx@gmail.com`), with no AI attribution anywhere. Never put a PR or issue number straight after fix, close or resolve.

**First read** `docs/journal/2026-11-24-algorithm-level/plan.md` and `rauzy-census/README.md`. The question, the reason and the stop test are there. This is characterisation only.

**Do, in order:**
1. **Time split.** On the production Open kernel (`src/open.rs` `Kernel::construct`, `src/open/endpoint.rs`), measure A6b's time split into setup, the induction steps and the dictionary/expansion, at one thread on wikidata, wikidata ×4, prose and hex (`ab gen corpora`).
   - Instrument outside `src/` if you can: a copy of the kernel under `examples/`, or counters behind an off-by-default feature.
   - Report each part's ns per row and its share, with the step count.
2. **The census of orders.** Extend `examples/rauzy.rs`, or a new example beside it, to run alternative induction orders on the same interval exchange the Open kernel eliminates. The orders:
   - right end (the baseline, which must match production's step count);
   - left end;
   - two-sided;
   - MONODROMY's 1.279 order (`docs/journal/monodromy/2026-11-10-rauzy-veech.md` §5.6);
   - Zorich-accelerated right end (division instead of repeated subtraction).

   For each, per block, report the steps per run, the ns per row, and whether the output is byte-identical to `bwt::walk`.
3. **Online choice.** For the best order, say whether it can be decided from information available during the decode, at what cost per step, and measure that cost if you build it.
4. **Apply the stop test** in `rauzy-census/README.md`, and write the verdict plainly.

**Rules:**
- no change to production behaviour;
- don't touch `bench/`, `.just/` or `examples/ab.rs`;
- no allocator changes;
- no new `unsafe`;
- `cargo test`, `clippy --all-targets --all-features -D warnings` and `fmt --check` clean;
- timings at one thread are indicative, so give ratios with spreads over at least 5 repeats.

**Deliver (part 1):**
- one PR against master, adding the code and `rauzy-census/results/README.md` (tables, verdict, negative findings);
- the status line in `plan.md` §2 updated;
- a short `FROM: neon / TO: coordinator / TYPE: REVIEW` on #145 linking the PR at its head.

---

## Part 2: the two pre-port tests from your software survey

Do these after part 1, on a second branch `research/pre-port-tests` from `origin/master`, as a second PR. They are the 1–2 day tests your `research/neon-movement/SOFTWARE-SURVEY.md` §3 sets before any port. Each needs only the stock tools. Its pass/kill gate is frozen as written in §3; don't change it after the data.

1. **Network calculus on stage 1's traces** (Deep Dive #1).
   - **The data:** extract arrival and service curves from `docs/journal/2026-11-24-algorithm-level/math-targets/data/character-stage1/` at 1, 6, 8 and 10 workers.
   - **The bound:** compute U_NC with stock Panco (and Nancy if .NET installs; if it doesn't, say so and use Panco alone).
   - **The gate:** U_NC / T_wall ≤ 1.5, the knee near 7–8, and two policies separated (for example W = 4k against W = 2k).
2. **uiCA on our kernels** (Deep Dive #2).
   - **The tools:** run stock `uiCA.py -arch CFL` and `llvm-mca -mcpu=skylake` on the hot loops you list:
     - `src/huff.rs`'s decode loop;
     - the 3-symbol LUT loop from PR 870;
     - A6b's step in `src/open/endpoint.rs`;
     - `src/runs.rs`'s MTF/RLE2 loop.
   - **The measured side:** compare both predictions with the cycles already measured (journal §82, `character-stage1`).
     - Don't run the workstation. If a loop has no measured figure, say so rather than estimating one.
   - **The gate:** ≤ 5% on the Huffman loop, the right sign and size on PR 870's regression, and clearly better than llvm-mca.
   - **The licence:** uiCA is AGPL-3.0. Use it as an external tool only, and commit none of its code.

**Deliver (part 2):**
- a PR adding `docs/journal/2026-11-24-algorithm-level/leads/pre-port-tests/README.md`, with each test's numbers and its verdict (PASS, KILL or, if a tool couldn't run, UNRESOLVED with why), plus the scripts;
- `plan.md` §2 thread 6's state updated;
- a short REVIEW on #145 naming both PRs at their heads.

---

## Part 3: how the compact GRH proof closes its squeeze (a reading task)

Do this last, as a third PR from `origin/master`, on a branch `research/grh-squeeze`. This is thread 7 of `plan.md`. Writing only: no code.

**The source:** https://github.com/lsernau/grh-lean, the compact edition `grh-compact/` and the guide `grh/GUIDE.md`.
- It is a roughly 76,000-line Lean rewrite of Levent Alpöge's roughly 17-million-line development, proving RH and GRH for Dirichlet L-functions.
- It is machine-checked only: no mathematician has read it through.

**Our earlier reading:** `docs/journal/2026-11-21-the-shape-of-the-rh-proof.md` worked from a second-hand overview of the proof's shape. It recorded that the overview's author had "not yet understood how these combine to exclude zeros". Read that note first.

**Read:**
- `grh/GUIDE.md`;
- `grh-compact/README.md` and `src/Statements.lean`;
- `src/New/Main.lean` (`star_inequality`, `New.grh`);
- the files for theorems A (short forms), B (enough forms) and C (integral forms are long).

Follow imports only as far as each step needs.

**Write** `docs/journal/2026-11-24-algorithm-level/leads/grh-squeeze.md`, answering:
1. **How the squeeze closes:**
   - what each of A, B and C bounds, in which direction, and from which theory;
   - how `star_inequality` combines them;
   - how a hypothetical zero off the line contradicts it, step by step, at a level a programmer can follow.

   Quote the Lean names you rely on, with file paths.
2. **What the rewrite dropped.** Compare with the 21 November note's six moves (A–F) and twelve levers:
   - which moves survived the 200-fold shortening and which went;
   - which steps were proved "by other arguments".

   Tie each claim to a file. If you can't tell, say so.
3. **The mapping onto mibz2.** We hold three kinds of bound on the decoder's wall:
   - the lower bounds of journal §70;
   - the measured rooms of the ideal-parts matrix (`ideal-parts/results/full/README.md`);
   - the proved upper bounds network calculus would give (part 2).

   For each move in (1), say whether it has an exact counterpart in those quantities: what would play "short", "enough" and "long", and what would play the exclusion. Mark each answer `[PROVED]`, `[ARGUED]` or `[CONJECTURED]`.
4. **The stop test, frozen:** if no move in (1) maps onto a quantity we measure or can compute, say so plainly. The thread then ends as this note. Otherwise, name the one experiment or derivation it suggests and what it would cost.

**Deliver (part 3):**
- a PR adding `leads/grh-squeeze.md`;
- `plan.md` §2 thread 7's state updated;
- a short REVIEW on #145, which may be the same post as part 2's.
