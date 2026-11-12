# External research report 52: the CI topology finding and the release-level question (verbatim)

A relayed message from the maintainer, unattributed in the relay, after report 51 and the
decoder-level scaling of the salvage table.

**What it says:**
- **Good news, though not a rescue for N1:** much of the apparent four-worker scaling
  problem may be an artefact of CI's hardware. All three runner types are 4 vCPUs on 2
  physical cores, so the measured 1.74–2.21× amplification at four threads can't be
  assumed for a four-physical-core laptop. That is not yet evidence of better scaling; it
  needs measurement.
- **The walk** is bound by dependent execution and per-core resources, not by bandwidth or
  table size. Be sceptical of proposals whose main advantage is a smaller table.
- **P74's online switch** remains the strongest dormant candidate. The prefix agreement
  (70 of 70) and the 0.09% probe cost are supporting but unmeasured.
- **N1 hasn't recovered.** The pre-allocation memory rule could remove one obstacle for
  P^{2|4}, but not the construction time.
- **The most important question:** how much of the release's actual wall time is exposed
  bzip2 decoding? Examine the existing wikidata-pq end-to-end measurements for the maximum
  plausible release-level benefit before restarting P74, authorising N1 Gate B or
  generating ideas.
- **Physical-core topology** should be explicit in future bench reports.
- **The order:** let the final N1 analyses finish, then the release-level examination.

**The coordinator's reading:**
- **The release-level examination** is read-only, over existing data, and comes after
  the N1 analyses finish.
- **Topology in bench reports** changes mibz2's bench. That is a proposal for the
  maintainer's approval, not started.

## The message, as received

Yes. And this update contains one genuinely important piece of good news, although it
isn't a rescue for N1.

We've discovered that much of the apparent four-worker scaling problem may be an artifact
of the CI hardware rather than a fundamental limitation of the decoder.

That's more consequential than it initially sounds.

Three findings that change the picture

1. The four-worker CI results were not measuring four physical cores.

All three CI runner types have four virtual CPUs on two physical cores.

That means we've been observing substantial SMT contention: two decompression workers
competing for one physical core's execution resources.

The measured 1.74–2.21× CPU amplification at four threads consequently cannot be assumed
to describe the intended four-physical-core laptop environment.

This is potentially encouraging for the shipped decoder. It may scale better on the actual
target than the CI results suggest.

But it is not yet evidence of better four-core scaling. That still needs measurement.

2. We've identified what makes the BWT walk expensive.

The results indicate that the shipped walk is mainly constrained by dependent execution
and per-core resources, not by memory bandwidth or the total size of its T-vectors.

That's a useful narrowing of the problem.

It means we should be skeptical of proposals whose principal advantage is a smaller table.
To outperform shipped inversion, a replacement probably needs to reduce dependent
operations per output byte without adding too much construction work.

N1 attempted exactly that, but its construction economics remain unfavorable.

3. P74's online switch remains the most interesting dormant candidate.

The 4,096-token prefix agreed with whole-block classification on all 70 sampled blocks.

That's supporting evidence for the design, but it doesn't establish reliability on unseen
block distributions.

The estimated 0.09% probe penalty is promising, but again, unmeasured.

Unlike the failed selector, an online switch potentially avoids the costly second pass over
the symbol stream. I still consider it the strongest unshipped opportunity from the
portfolio.

What hasn't improved

The N1 situation hasn't meaningfully recovered.

The correction to shipped construction traffic, the tighter whole-corpus builder budget and
the retained-memory problem all remain.

The new memory-selection analysis is useful because it asks whether high-memory blocks can
be rejected before allocation. If it succeeds, that could remove one obstacle to .

But it cannot fix construction time by itself.

The question I now consider most important

Claude raises the right one:

How much of the release's actual wall time is exposed bzip2 decoding?

Suppose decoding accounts for only 20% of elapsed time once the rest of the pipeline
overlaps with it. Even making the decoder infinitely fast could eliminate at most that
20%, assuming the remaining work cannot be shortened.

If decoding instead determines the pipeline's critical path, the opportunity is much
greater.

The exact answer depends on pipeline overlap and scheduling, so simple stage-time
percentages may be misleading.

Before restarting P74, authorizing N1 Gate B or generating more ideas, I'd examine the
existing wikidata-pq end-to-end measurements to establish the maximum plausible
release-level benefit.

My updated assessment

Research question	Status
Does N1 have an economical builder?	Still doubtful
Can stay within memory limits?	Pending
Do four-worker CI results represent four physical cores?	No — important finding
Is the shipped decoder necessarily poor at four-core scaling?	Not established
Is P74's online selector worth preserving?	Yes
Do we know the release-level payoff from faster decoding?	Not yet

My recommendation: let the final N1 analyses finish, then examine the existing end-to-end
release data before deciding on any further optimization work.

I would also make physical-core topology explicit in future benchmark reports. That's a
measurement-quality improvement with lasting value.

So there is a change: the outlook for N1 hasn't improved, but our understanding of the
shipped decoder has. We may have been trying to solve a four-core contention problem that
doesn't actually exist to the same degree on the target hardware.

And that is a useful discovery.
