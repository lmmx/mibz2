# External research report 56: "we also made some demo of gains here but couldn't use them?" (verbatim)

The maintainer's question, quoted, and the reply it was given, relayed together. The reply
is unattributed in the relay.

**What it says:**
- **Two kinds of improvement, neither yet usable:**
  - P74 delivered a measured stage speed-up: Wikidata's checkpoint MTF stage 33.2%
    faster on Zen 4 and 19.2% on Zen 5. The controls were 9–20% slower with P74 alone,
    and the selector, though correct, cost 6–7% on them.
  - N1 demonstrated less navigation work: 0.296 and 0.150 lookups per byte against the
    shipped walk's 1.000, correctness-tested. These are operation counts, not a measured
    speed-up, and construction looks expensive.
- **The recurring pattern:** a faster core operation, extra preparation, an uncertain net
  gain. P74 pays to decide; N1 pays to construct.
- **If either is revisited,** P74's online switch first, scoped narrowly, and only if
  decoding matters enough to the release.

**The coordinator's notes on the reply, against the record:**
- **"A pre-allocation memory selector is being investigated":** finished
  (`2026-10-30-n1-traffic-and-peak-reconciliation.md` §4.7). A staged check after
  P^{2|4}'s first count pass, before any level-2 allocation, falls back to the T-vector in
  the same arena. It keeps 221 of 233 blocks (94.8% of bytes) on N1, with no false
  fallbacks and no overshoot.
- **"Retaining the construction buffers may violate the memory requirements":** under
  that rule, retained capacity equals the shipped path's, 7.23 MB a worker: memory-neutral,
  never better. Without a rule, every plan retains more.

## The message, as received

> so I recall we won 33% speedup on wikidata but it harmed random data etc… it sounds like maybe we also made some demo of gains here but again somehow couldnt use them?

Yes, you're remembering correctly. We've demonstrated two different kinds of improvement,
but neither has yet become a usable general-purpose decoder optimization.

The distinction is that P74 delivered a measured speedup; N1 has demonstrated a
substantial reduction in algorithmic work, but not yet a measured speedup.

Here's how they compare.

	P74 — Register-held MTF	N1 — Multi-byte Ψ
What changed?	Kept the frequently used MTF list entries in CPU registers	Replaced one-byte BWT navigation with variable-length jumps
Demonstrated benefit	33.2% faster Wikidata checkpoint MTF stage on Zen 4; 19.2% on Zen 5	Approximately 0.296 lookups per byte with , or 0.150 with
Was that a measured speedup?	Yes	No — these are operation counts
What went wrong?	Random and records became 9–20% slower with P74 alone	Constructing the jump tables looks expensive and may increase retained memory
Could we select when to use it?	The selector chose correctly but cost 6–7% on controls	A pre-allocation memory selector is being investigated
Status	Gate 1 closed, negative	Correctness established within tested scope; performance viability unresolved

P74: we built something faster but couldn't deploy it safely

We really did measure a 33.2% reduction in the complete checkpoint-aware MTF stage on
Wikidata.

The trouble was that random and records data frequently access entries beyond the
register-held MTF head, where our implementation was slower.

We tried to select the fast implementation only when appropriate. It made the correct
choice for every tested block, but determining that choice required scanning the encoded
ranks. The cost of that scan alone made the controls 6–7% slower.

So we didn't fail to find a fast algorithm. We failed to find a sufficiently cheap way of
choosing when to use it.

There is a potential solution, Option C, which starts decoding and makes the choice from
the first few thousand tokens without a separate scan. It remains unimplemented and
unbenchmarked.

Also, the 33% is a stage-level improvement. Its projected contribution to the complete
decoder was only about 5.8% less decoder time on Zen 4, before selection overhead.

N1: we proved a way to do much less navigation work

This is a different story.

The shipped BWT walk essentially advances one byte per lookup. N1 can advance several
bytes using a single record.

Navigation lookups per decoded byte

Shipped

1.000

N1 P²|⁴

0.296

N1 P⁴|⁸

0.150

Lookup counts from the tested navigation designs, not relative execution times.

That's approximately 70% or 85% fewer navigation lookups.

And the correctness tests are remarkably strong: 5.65 million walks and 7.7 million record
checks with no failures.

But there is a catch: before you can make those jumps, you must construct the tables
describing them.

That construction currently looks substantially more expensive than building the shipped
T-vector. Worse, retaining the construction buffers may violate the memory requirements.

So N1 has proved that we can do less work during traversal, but hasn't proved that the
cost of preparing for that traversal is worth paying.

The recurring pattern

Both findings expose the same broad engineering difficulty:

Faster core operation

Extra preparation

Net gain uncertain

* P74 pays to decide when to use the faster kernel.
* N1 pays to construct the structure that makes navigation faster.

In both cases, the overhead threatens to consume the benefit.

The crucial difference: P74's speedup was measured in executable code. N1's reduction in
lookups was demonstrated and correctness-tested, but its runtime advantage is still a
model.

My assessment

We didn't discover two production-ready accelerations and somehow lose them. We discovered
one demonstrably faster specialization and one demonstrably correct way of reducing
navigation operations.

Those are real findings. Neither has yet met the requirements for integration.

If we eventually return to either, I would prioritize P74's online switch for a narrowly
scoped experiment, because its underlying fast-path improvement is already measured. But
whether even that is worth the work depends on how much decompression matters to the
complete release pipeline.

So your intuition is right: we keep finding ways to make the expensive part faster, only
to discover that making the optimization usable costs us much of the gain. That's the
central lesson of this research round.
