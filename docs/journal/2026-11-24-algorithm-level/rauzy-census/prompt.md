# Antigravity (Neon): the Rauzy census (thread 2 of `plan.md`)

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

**Deliver:**
- one PR against master, adding the code and `rauzy-census/results/README.md` (tables, verdict, negative findings);
- the status line in `plan.md` §2 updated;
- a short `FROM: neon / TO: coordinator / TYPE: REVIEW` on #145 linking the PR at its head.
