# Antigravity (Neon): the next task, in two parts: the Rauzy census, then two pre-port tests

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
