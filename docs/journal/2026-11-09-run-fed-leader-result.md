# The run-fed leader: the registered CI run

9 November 2026. This is the result of the run registered in
`docs/journal/2026-11-09-run-fed-leader.md` (§6), on #270.
- **The frozen head** is `03e52206ac248737bc856f064928adf238265917`. Its implementation pin is
  `f67085670ee696340337aa37377afab886dded3a`, and the head adds docs only.
- **Guardian D056** gave PROCEED at that head. D054's scoped B5 clearance transferred across the
  empty-block repair.
- **CI on the head** was 24 of 24 successful.

**The run:** `sweep.yml` run [42564217298](https://github.com/lmmx/mibz2/actions/runs/42564217298),
`tool: run_fed_leader`, `rounds: 15`, `registered_head` 03e52206. Its artifact is
`sweep-03e52206…`.
- **The binary's SHA-256** was `4bcbe399…f81b` before and after the run.
- **The corpus's SHA-256** was `82f65ace…c29915` before and after.
- **The compiler** was rustc 1.99.1, on 4 vCPUs (2 cores).

**Validity:** `bench/run_fed_leader.py report` gives **VALID (JUDGED)**: 15 rounds at 9 threads,
against `mibz2-default33`. All 210 runs gave the same output hash, `a8830caee5c511a6`, over
209,700,196 bytes.

## The judged reading

The primary is `mibz2-pipe3-runfed-dir` against `mibz2-default33`, at 9 threads, on Wikidata:

| | wall ratio [95%] | CPU ratio [95%] |
|---|---|---|
| **primary, dir** | **0.9669 [0.9594, 0.9744]** | 0.9640 [0.9602, 0.9677] |

- **The verdict is LEAD.** The registered rule is: the wall ratio ≤ 0.97, its high end < 1, and the
  CPU ratio's high end ≤ 1.05. All three hold.
- **Attributed.** `ctl` over the incumbent is 1.0006 [0.9931, 1.0081], wholly inside
  [0.98, 1.02]. `dir/ctl`'s wall high end, 0.9725, is below 1.

This is **the first positive registered reading on the run-fed line.**

## Every arm, and the contrasts

Wall ratios at 9 threads against `default33`, paired by round:

| arm | wall [95%] | CPU [95%] |
|---|---|---|
| pipe3-both | 0.9936 [0.9866, 1.0006] | 1.0009 [0.9935, 1.0083] |
| runfed-ctl | 1.0006 [0.9931, 1.0081] | 1.0017 [0.9959, 1.0076] |
| runfed-reg (C29 control) | 0.9745 [0.9675, 0.9816] | 0.9719 [0.9659, 0.9780] |
| runfed-ls (secondary) | **0.9498 [0.9435, 0.9562]** | 0.9521 [0.9474, 0.9568] |
| runfed-hs (secondary) | 0.9794 [0.9698, 0.9891] | 0.9780 [0.9728, 0.9832] |
| **runfed-dir (primary)** | **0.9669 [0.9594, 0.9744]** | 0.9640 [0.9602, 0.9677] |

**What drives it.** The report prints contrasts against `ctl`, which is the registration's rule
for attribution. I also computed two contrasts against `reg`, the register-list control, from
`rfl-runs.jsonl` with the same paired log-ratio t-interval over the 15 rounds:
- **dir / reg: 0.9921 [0.9865, 0.9977]**;
- **ls / reg: 0.9746 [0.9697, 0.9796]**.

These are post-run descriptive contrasts. They neither replace the registered reference nor add a
retrospective gate. The Researcher reproduced both independently (6732761085): dir/reg
0.992134 [0.986550, 0.997749] and ls/reg 0.974647 [0.969726, 0.979592].

At the registered attribution's own level, the LEAD is attributed. What the contrasts do and
don't show:
1. **`reg` alone gets 2.5%** (0.9745). It is the C29 register-list MTF checkpoint kernel, with no
   records.
2. **"The direct fill adds 0.8%" would be too causal.** `reg` uses C29 on every block. The record
   arms keep the old checkpoint kernel on blocks the rule doesn't take, and they also change
   several construction and scheduling paths. So dir/reg is a contrast between two whole
   configurations; it does not isolate the direct fill's benefit. The registered attribution
   passes, but it doesn't credit the whole 3.3% to the direct fill either.
3. **The leader-sort arm `ls` is the strongest at 9 threads:** 5.0% against the incumbent, and
   2.5% beyond `reg`. It is a secondary, so it is never judged, and the registration chose `dir`
   as its primary before the run.
   - **The phases:** `ls` has the shortest `run_ns` (4.435 ms against ctl's 4.769) and the
     shortest `join_wait`.
   - **The sort:** its leader does it (0.215 ms), and its helpers have no sort step.
   - **dir, by comparison:** it pays 0.754 ms of helper sort and 163 lookups a block.
   - Helper phase totals can't identify critical-path savings by subtraction.
4. **The 3-thread cells don't agree with the 9-thread ranking.** These are one team, reported
   only and never judged. Against `pipe3-both`:
   - dir 0.9609;
   - reg 0.9615;
   - ls 0.9761;
   - hs 0.9836;
   - ctl 1.0028.

   The Researcher's paired recomputation at one team:
   - dir/reg 0.999362 [0.992310, 1.006464];
   - ls/reg 1.015168 [1.001756, 1.028760];
   - ls/dir 1.015817 [1.001113, 1.030736].

   So at one team `ls` is *slower* than both `reg` and `dir`, and `dir` is level with `reg`. The
   cells agree only in that every record arm gains against both controls. This argues for
   measuring the target machine, not for predicting that `ls` will win there.

These phase clocks include descheduling on an oversubscribed runner (9 threads on 4 vCPUs), so
they are CI-specific. The registration says so in its §7.

## What the registration permits, and what follows (PROTOCOL §11)

- **The workstation run.** An attributed LEAD on CI permits freezing §8's workstation recipe as the
  decision's evidence. It permits no production change. The Researcher's review (6732761085)
  asks for a REVISE of §8 before the freeze. `dir` stays the primary, with `ls` and `hs` as
  secondaries. The freeze must have:
  - all seven arms, their order, and the `default33` reference;
  - exactly 20 paired rounds at a requested 10 threads (140 timed records), with three leaders
    and six pipelined helpers observed;
  - a separate, fail-closed admission, not the CI reporter's hard-coded 15 rounds at [9, 3] or
    its UNJUDGED mode;
  - the thresholds and attribution stated prospectively: §6's, with df = 19 intervals;
  - frozen source, recipe and admission identities, and stable binary and corpus digests;
  - the ctl and dir occupancy captures, defined and admitted separately;
  - partial or failing evidence kept.

  Louis's own run of the frozen recipe, submitted with a verified digest, is still required
  (C5).
- **C29 is a separate lead, with its own registration** (§6: "if `reg` is faster than `ctl` …
  that is a separate lead"). `reg/ctl` is 0.9739 [0.9682, 0.9797].
  - The register-list kernel is a candidate replacement for the shipped `mtf_checkpoints_avx2`
    in the shipped pipelined path, independent of records.
  - That is a production prototype: its own registration, CI's paired bench, and B5.
- **`ls` as a primary.** The leader-sort arm's 2.5% beyond `reg` at 9 threads is the largest
  record effect seen, but it reverses at one team. Making it a primary needs a new registration.
  - **A narrower angle (the Researcher):** fuse the run-count and row histograms into emission,
    to remove the later count pass. `ls` already does a stable counting sort in the next block's
    leader front stage.
  - **The obstacle:** final F placement still needs lower-byte totals, which are not known until
    emission ends. Keep the prefix and scatter, or cost the buffering and fix-ups.
  - **The comparison:** fused against unfused `ls`, with the rest fixed, under its own
    registration.
- **Not shown:** the whole-decoder gain on the target machine, and any production benefit. CI's
  9-on-4 oversubscription is not the i9's.

## Record corrections from D056

These are carried from D056 and are non-blocking:
- The empty-block prefixes accept `[c]` for any 0 ≤ c < 256 at n = 0 with zero counts, not only
  `[0]`. All of them give empty semantics and no table stores. `[1]` and `[255]` are negative
  cases if the contract is tightened.
- `r3-case596-replay.txt` shows 20 post-fix repeats. The pre-fix failure evidence is mutant R3a.

**Revision (same day):** the mechanism reading and the 3-thread sentence were corrected from the
Researcher's review (6732761085). The verdict is unchanged.
