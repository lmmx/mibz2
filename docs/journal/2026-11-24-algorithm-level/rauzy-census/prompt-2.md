# Antigravity: the Rauzy census, round 2. Rescue it, don't re-run it

*26 November 2026. Louis: "I feel like we gave up on the Rauzy census idea too easily … not override it … use this as feedback on further ideas. Often your first attempt wasn't phrased properly, or you missed something obvious that could rescue it."*

Work in lmmx/mibz2 on a new branch `research/rauzy-census-2`, from `origin/research/rauzy-census` (PR #933, which holds your harness `examples/rauzy_census.rs` and round 1's results), or from master once #933 has merged. Commit as Louis Maddox <louismmx@gmail.com>, with no AI attribution anywhere, trailers included. Never put a PR or issue number straight after fix, close or resolve.

## Round 1's verdict stands. This round asks a different question

Round 1 (`results/README.md` on #933) asked: **would a cheaper induction *order* cut time?**
- **On Open's own exchange:** the best order cuts steps by only 16.7%.
- **The ceiling:** that is 1.41% of the 1-worker wall, against a 3% gate.
- **The cost:** every cleverer order loses on the clock. The online scheduler costs 160 ns per step, against a 4.9 ns break-even.

That verdict is not reopened. But it was one phrasing of the opportunity. Its own data point at others. **The question for round 2: can anything that touches A6b's machinery (setup, the steps, the dictionary), or the choice of what is eliminated at all, cut ≥ 3% of the 1-worker wall (about 39 ms of 1,306), at a cost that fits?**

Treat round 1's numbers as feedback. Here is what they say, and the rephrasings they suggest. You are free to find better ones.

1. **Cheaper steps, not fewer.**
   - Production's 24.6 ns per step is about 57 cycles at 2.3 GHz, for a step that reads two neighbours and relinks one node in a linked list (Narrow, 16 B a node).
   - That looks like a dependent-load chain, latency rather than work. If so, the cost lever is the chain, not the count.
   - Measure it: instructions per step, IPC, and the critical-path loads (perf on the VM is fine, as ratios). Then try latency hiding:
     - interleave the inductions of 2–4 blocks in one worker, round-robin, as coroutines or AMAC-style;
     - or prefetch the next step's nodes.
   - The steps are 51% of A6b, about 110 ms at 1 worker, so halving their cost would clear the gate on its own.
2. **Does the induction have width?** Round 1 treated it as a sequence. Find out whether it is one dependency chain or a DAG with independent parts:
   - Do eliminations in different regions of `[m, m+n)` commute?
   - Does the auxiliary exchange split into invariant pieces?
   - If there is width, several independent lanes in one worker give memory-level parallelism.

   Report the width or the critical-path length per block.
3. **A cheap rule that takes only free steps.**
   - The online scheduler lost on random probes into two hash maps.
   - **Rephrase:** mark the boundary positions once, in a bitmap over `[m, m+n)` (n/8 bytes, L2-resident), during A6a's pass, which already has the data. Then run right-end production, but take a free step when one is visible at a probe of a bit or two.
   - The budget is about 5 ns a step. Measure how much of round 1's 16.7% such a rule keeps, and at what cost.
4. **The other half of A6b.** It is 49%: setup 19.3% (17.7% is linked-list building) and dictionary/expansion 29.6% (hash-and-dedup).
   - **Setup:** can the step run on implicit arrays instead of built lists?
   - **The dictionary:** can its deduplication be replaced by a structural canonical form, or deferred to emission (lazily, only for words that are emitted), or batched?

   Each is its own budget. The dictionary is about 64 ms at 1 worker, so a 60% cut clears the gate.
5. **What is eliminated at all.** Band count is invariant under order (827,858 on wikidata). But the thin/thick cut (`LIMIT = 16`) decides how much reaches A6b against A6c and A7.
   - Measure A6a + A6b + A6c + A7 together as `LIMIT` varies: 8, 12, 16, 24, 32, 64.
   - Then try a per-block or per-byte limit chosen from A6a's counts.

   This trades between C and E. Count only the net.
6. **Anything round 1 missed.** If you see a better framing, try it. Say why it's better.

## For every idea: the same rules

- **A prototype in `examples/` only.** No `src/` changes, no new `unsafe`, no allocator changes. Don't touch `bench/`, `.just/` or `examples/ab.rs`.
- **Byte-identical to `bwt::walk`** on every block of wikidata, wikidata ×4 and the `ab gen` corpora.
- **Measured as round 1 was:** sub-phase ns per row; median and min..max over ≥ 5 repeats; ratios against round 1's production mirror on the same machine. VM timings are indicative.
- **Frozen gate, the same for every idea:** the net saving across A6a + A6b + A6c + A7 must reach ≥ 3% of the 1-worker wall, about 39 ms (that is, a ≥ 1.84% cut of C + E's 2.65 ns per row, or the ratio equivalent), with its cost included.
  - Ideas that pass go forward as engineering candidates, with a production design sketched.
  - Ideas that fail are written up as dead ends with their numbers, and the next idea is tried.
- **Also report, without gating,** each idea's effect on C's share at 8 workers. Construction binds there once the scan is pooled (journal §92: with every worker-side part free, the scan alone is 179 ms at 8 workers), so a gain small at 1 worker may matter more later.

## Deliver

- One PR adding `rauzy-census/results-2/README.md`:
  - each idea: hypothesis, measurement, verdict;
  - the winners, ranked;
  - the dead ends, with numbers.
- The harness changes.
- `plan.md` §2 thread 2's state updated.
- A short REVIEW on #145 linking the PR at its head.
