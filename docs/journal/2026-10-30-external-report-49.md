# External research report 49: "what is left?" (verbatim)

The maintainer's question, quoted, and the reply it was given, relayed together. The reply
is not attributed in the relay; it reads as the researcher's.

**What it says:**
- **Change direction.** P74 was a real speed-up that couldn't be transferred economically.
  N1's engineering economics are worsening. Don't invent another hypothesis to keep the
  programme alive.
- **What's left:**

  | Avenue | Verdict |
  |---|---|
  | P74, the current implementation | Close |
  | N1, partial depth | One final feasibility decision |
  | N1, full Ψ⁸ | Deprioritised |
  | The shipped decoder (the one-pass build, landed) | Preserve and consolidate |
  | A new architecture-level idea | Needs a new premise |

- **Stop reasoning from one-core stage models.** Measure what actually stops the shipped
  decoder going faster at four workers on the target hardware: an end-to-end bottleneck
  profile with exact CPU identification, worker placement, throughput, CPU-seconds per GB
  and peak RSS. This does not authorise the i9 now.
- **N1 must show all three before Gate B is considered:**
  - a correct construction and mixed-length walker;
  - a credible high-water memory plan within the existing RSS limit;
  - a whole-corpus construction budget with meaningful headroom against the one-pass
    shipped build.

  Otherwise close N1 without Gate B.
- **P74:** a free classifier or a safely bounded online switch is a possible future route.
  Neither is demonstrated, and P74 is not to be reopened because N1 got hard.
- **The verdict:** the portfolio is thin. Give N1 a final paper feasibility review. If it
  fails, close this campaign, keep the production improvements, and restart only from a
  fresh end-to-end profile. The campaign may be ending; the decoder's limit is not
  established.

## The message, as received

> it sure looks like a dead gate walking… before this P74 failed to transfer, now this, what is left?

I think we're approaching the point where the research programme needs a change of
direction.

P74 demonstrated a real performance improvement that couldn't be transferred economically
across workloads. N1 has an elegant mathematical foundation, but its engineering economics
are looking increasingly unfavorable.

And I don't think we should invent another optimization hypothesis simply to keep the
programme alive.

But I also wouldn't conclude that nothing valuable remains.

What is actually left?

Avenue	Evidence today	My assessment
P74 register-held MTF	19–33% faster Wikidata MTF stage; selector failed the control limits	Close current implementation
N1 partial-depth Ψ	Fewer dependent lookups; increasingly difficult construction and memory economics	One final feasibility decision
N1 full Ψ⁸	High construction cost and retained memory	Effectively deprioritized
Existing shipped decoder improvements	One-pass BWT construction already landed; functioning production implementation	Preserve and consolidate
New architecture-level optimization	No surviving measured candidate from this portfolio	Requires a new premise, not another variation on a failed one

The last two rows are important.

The goal wasn't to produce a novel algorithm at any cost. It was to make the decoder
materially faster on four-core hardware without sacrificing CPU efficiency or memory.

If the existing implementation is already close to its practical performance frontier,
discovering that through disciplined experiments is a legitimate research result.

My strongest recommendation: stop chasing the one-core stage models

We've spent a lot of effort reasoning from cycles per record, cache-line traffic,
estimated IPC and inferred instruction dependencies.

Those models have been useful for rejecting bad ideas, but they have also repeatedly
turned out to be more fragile than we hoped.

P74 taught us that a very large stage-local speedup can be undermined by the cost of
choosing when to use it.

N1 is teaching us that reducing dependent navigation steps doesn't guarantee an
economical construction process or acceptable retained memory.

The practical question should now be:

What actually prevents the shipped decoder from going faster at four workers on the
hardware that matters?

Not what looks theoretically expensive. What is measured to prevent it.

We have some profiling and stage measurements, but the four-copy investigation exposed how
easily SMT contention, CPU frequency and runner changes can masquerade as algorithmic
behavior.

Before opening another optimization programme, I would want a reproducible end-to-end
bottleneck profile on the actual target class of hardware, with precise CPU
identification, worker placement, throughput, CPU-seconds/GB and peak RSS.

That is a different research strategy: measure the bottleneck first, then generate
candidates that directly address it.

It doesn't require authorizing the i9 now.

Is N1 worth one final builder experiment?

Possibly. But I'd set a much stricter condition than before.

Once the correctness and memory reconciliation finish, N1 should have to show all three
things before Gate B is considered:

* A correct construction and mixed-length walker.
* A credible high-water memory plan within the existing RSS limit.
* A whole-corpus construction budget with meaningful headroom against the current
  one-pass shipped decoder.

If those aren't present, I'd recommend closing N1 without implementing Gate B.

I would not approve another elaborate builder simply to resolve a theoretical uncertainty
that cannot lead to a deployable decoder.

The result I don't want us to overlook

The programme has already yielded an improved shipped decoder through the one-pass BWT
build, as well as substantial knowledge about where further optimization becomes
counterproductive.

It has also established something valuable about the performance landscape: a
specialization can be spectacular on one stage and still fail the requirements of a
general-purpose decompressor.

Those findings are worth keeping even if every remaining speculative candidate closes.

And there is still a narrower route for P74 in the future: a genuinely free classifier or
a safely bounded online switch. But neither is presently demonstrated, and I would not
reopen P74 just because N1 has become difficult.

My current research verdict: the remaining portfolio is thin. N1 deserves a final paper
feasibility review, not automatic Gate B authorization. If it fails that review, close
this optimization campaign, retain the production improvements, and restart only when a
fresh end-to-end profile identifies a specific bottleneck that justifies new work.

I know that's less exciting than finding a breakthrough, but it is a much better outcome
than spending weeks engineering candidates whose acceptance conditions have already become
implausible.

And one distinction matters: the optimization campaign may be nearing its end; that does
not mean the decoder itself has reached its ultimate performance limit. It means this
particular set of hypotheses may be exhausted.
