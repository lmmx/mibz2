# External research report 59: answers to the three questions, and the prefix's worst case (verbatim)

A relayed message from the maintainer, unattributed in the relay, answering the
coordinator's three questions after report 57.

**What it says:**
- **P74's remaining work is not merely mechanical:** correctness and performance under
  changing rank distributions still need proof.
- **The three answers:**
  1. **P74 for a projected 4–6%:** yes, if decoding materially affects release elapsed
     time; otherwise leave option C archived.
  2. **N1 only on cheaper doublings:** essentially yes. Archive N1 after this round unless
     a specific way to reduce the doubling work appears, not the same work with better
     instructions.
  3. **Direct level-1 emission from MTF (N3):** worth a separate read-only question, not an
     N1 rescue. It challenges the claim that L's runs are exactly the explicit MTF tokens:
     the 20-block agreement may rest on a particular interpretation, so it wants the
     definition made precise before treating it as an invariant. Removing the build's input
     scan helps only if it saves more than it adds to MTF.
- **An added theoretical task** (analysis, not reopening Gate 1): the maximum avoidable loss
  from a wrong prefix decision, under bounded continued monitoring. A block can start
  with low ranks and later become fallback-heavy.
- **Priorities unchanged until the release-level result:**
  - P74: option C preserved;
  - N1: archived unless the doubling improves;
  - N3: a separate question;
  - release: the exposed cost of decoding first.

  One tightly bounded P74 online-switch experiment only if decoding is on the critical
  path; otherwise other release bottlenecks.

**The coordinator's reply on the runs and tokens (deduced, exact):** in bzip2's MTF, a
symbol is moved to the front of the list as it is coded. So the rank at position i ≥ 1 is
0 exactly when L[i] = L[i − 1]. A run of L therefore starts exactly at a nonzero rank. The
one exception is position 0, which starts a run even when its rank is 0 (L[0] is
already at the front of the initial list).

So r = (nonzero ranks) + [rank at position 0 is 0]. The nonzero ranks are the explicit
symbols 2 ≤ s < EOB, the same E P74's selector counts, since the zero ranks are what
RUNA/RUNB code. The invariant is universal, off by at most one at the block's start, and
the 20 of 20 measured matches agree with it.

**The coordinator's action:** the prefix worst-case analysis is queued behind the
release-level result, with the other priorities.

## The message, as received

Claude's analysis sharpens the distinction considerably. P74 has a plausible route to
preserving a measured speedup. N1 still needs a substantial reduction in work that the
existing decoder never had to perform.

I agree with the direction of the assessment, with one qualification: I'd be cautious
about calling P74's remaining work merely mechanical. Mid-block switching is implementable
in principle, but correctness and performance under changing rank distributions still need
proof.

My answers to Claude's three questions

1. Would I pursue P74 for a projected 4–6% decoder improvement?

Yes, if decoding materially affects release elapsed time.

A 4–6% reduction in decoder time is worthwhile for a mature decompression library,
especially if the specialization preserves performance on other inputs. But a mid-block
switch introduces complexity, and its estimated 0.09% probe cost isn't yet a measurement.

I'd want the release-level analysis before deciding whether that complexity is justified.
If decompression is mostly hidden by other pipeline stages, I'd leave Option C archived.

2. Should N1 research focus exclusively on cheaper doublings?

Essentially, yes.

The expensive counting, prefix and merge passes are the fundamental obstacle. Optimizing
level 1 cannot close a 2–3× construction-cost gap by itself.

I would not keep searching indefinitely, though. Unless someone can identify a specific way
to reduce the doubling work—not simply implement the same work with better
instructions—I would archive N1 after the current round.

Its correctness and memory results are worth preserving even if its performance case
closes.

3. Is direct level-1 emission from MTF worth investigating?

Yes, but as a separate read-only question, not as another N1 rescue attempt.

There's an interesting possibility here: instead of constructing a complete intermediate
representation and then scanning it, perhaps the decoder could emit the information needed
for the shipped BWT construction while producing its last column.

But I would challenge one detail in the proposed reasoning. The assertion that L's runs are
exactly the block's explicit MTF tokens needs a precise definition. MTF tokens describe
positions in a changing symbol list; adjacent repeated symbols in the BWT last column are
not generally in one-to-one correspondence with explicit MTF tokens. The reported 20-block
agreement may involve a particular interpretation of run boundaries or token counts. I'd
want that clarified before treating the observation as an architectural invariant.

Even if the correspondence holds in the relevant representation, the shipped one-pass
T-vector builder is already very efficient. Removing its input scan is useful only if it
avoids more work than it adds to MTF.

One question I'd add

For P74's online switch, the 4,096-token prefix matching whole-block classification on all
70 sampled blocks is reassuring, but classification agreement is not the ultimate
correctness or performance criterion.

A block can begin with low ranks and later become fallback-heavy. The online switch must
either cope with that change or accept a measurable worst-case regression.

That suggests a useful theoretical task before any implementation: derive the maximum
avoidable loss from a wrong prefix decision, subject to a bounded amount of continued
monitoring. It could determine whether Option C can meet the controls without requiring an
expensive full-stream count.

This is analysis, not a proposal to reopen Gate 1.

Where I stand

I would keep the priorities unchanged until the release-level result arrives.

* P74: preserve Option C as the strongest unshipped candidate.
* N1: archive unless a concrete improvement to doubling construction emerges.
* N3: retain direct emission as a separate research question, but don't assume the claimed
  token/run equivalence is universal.
* Release pipeline: determine the exposed cost of decoding before authorizing further
  optimization.

The biggest change in perspective is that we no longer need another theoretically faster
decoder stage. We need evidence that making a stage faster will improve the actual release.

If the upcoming analysis shows that decompression is on the critical path, I'd be willing
to support one tightly bounded P74 online-switch experiment. If it isn't, I'd prioritize
other release bottlenecks rather than continue optimizing bzip2 in isolation.
