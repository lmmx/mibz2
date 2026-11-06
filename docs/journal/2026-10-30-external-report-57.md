# External research report 57: "avoid putting into practice stealing the gain" (verbatim)

The maintainer's remark, quoted, and the reply it was given, relayed together. The reply is
unattributed in the relay.

**What it says:**
- **Two mechanisms with demonstrable advantages.** The challenge is to make them available
  without overhead that erases them. P74's speed is measured; N1's saving is in operations
  and correctness, not yet net runtime.
- **P74:** avoid paying for the decision. Start with P74, watch the first few thousand
  tokens' ranks, and switch to the shipped MTF if unfavourable (option C). The open
  problems are a correct, cheap switch and a distribution that changes mid-block.
- **N1:** avoid building information that won't be used. The open research question is
  whether jump information can be produced more directly: fewer passes, less temporary
  storage, bounded allocation. No such shortcut has evidence comparable to option C's, and
  the one-pass shipped build is a formidable baseline.
- **The principle:** net benefit = work eliminated − work introduced. Future candidates
  should reuse work the decoder already does, not add a preparatory phase. Keep the
  whole-cost discipline: memory retention and negative controls before celebrating a
  stage-local result.

## The message, as received

> mhm so we have found two things which *can* help in practice, we just need to find how to avoid said putting into practice degrading them disproportionately and stealing the gain…

Yes. That's exactly the opportunity we've uncovered. And I think it's a more productive way
to view the results than treating the two candidates as failures.

We've found two mechanisms with demonstrable advantages. The challenge is to make those
advantages available without introducing enough overhead to erase them.

There is, however, one important distinction.

P74 has demonstrated actual speed. N1 has demonstrated fewer navigation operations and
strong correctness, but still needs to prove that those savings translate into a net
runtime improvement.

What interests me is that they suggest two rather different ways forward.

P74: avoid paying to make the decision

The fast register-held MTF kernel already exists and works.

The expensive part was scanning every block's symbols to decide whether to use it. But
decoding itself reveals the rank distribution.

So instead of scanning first, we could:

* Begin with P74.
* Observe the rank distribution during the first few thousand tokens.
* Switch to shipped MTF if the input appears unfavorable.
* Avoid scanning the full symbol stream separately.

That addresses the specific overhead that killed the selector.

The existing 4,096-token prefix analysis is encouraging, though not a guarantee on
arbitrary inputs. The remaining challenge is to make switching correct and cheap,
particularly when the rank distribution changes partway through a block.

N1: avoid building information we won't use

N1 takes the opposite approach. It performs less work during navigation but spends too much
preparing the navigation records.

The partial-depth designs already move in the right direction: construct fewer records and
make longer jumps only where the existing intervals allow them.

The more ambitious research question is whether the useful jump information can be
produced much more directly, with fewer intermediate passes, less temporary storage and
bounded allocation.

But I would be cautious here. We haven't identified an N1 construction shortcut with the
same level of evidence as P74's online selector idea. And the one-pass shipped builder is a
formidable baseline.

The common insight

Both designs currently violate a useful optimization principle:

\text{Net benefit}

\text{work eliminated}

\text{work introduced}.

For P74, we need to minimize the cost of selecting the optimization.

For N1, we need to minimize the cost of preparing the optimization.

The strongest future candidates are therefore not necessarily faster versions of the
existing kernels. They're designs that reuse work the decoder must already perform, instead
of adding a separate preparatory phase.

That is where I would focus any fresh theoretical investigation.

And I would keep one discipline from this campaign: evaluate the entire cost, including
memory retention and the negative-control workloads, before celebrating a stage-local
result.

So yes: we have two genuine pieces of algorithmic leverage. P74 is much closer to practical
realization; N1 has a harder construction problem. Neither should be discarded
intellectually simply because the first integration approach was too expensive.

The next breakthrough, if one exists, is likely to come from making the acceleration nearly
free to enable, rather than making its already-fast inner loop faster.
