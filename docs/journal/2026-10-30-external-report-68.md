# External research report 68: event-triggered budget switching (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**What it proposes:** a one-way switch tested only inside P74's rank ≥ 32 fallback branch,
F + 1 > τn + b, with n the output cursor.
- **Invariant:** between high-rank events, F is constant and n non-decreasing, so the
  budget can only be crossed at a high-rank event. There is no fast-path check, no
  monitoring delay and no overshoot.
- **Bound:** R ≤ (uτ + v)/c_min + (u·b + H + M)/(c_min·n), conditional on d_high ≤ u and
  D_other ≤ v·n.
- **Short-block guard:** syms.len() < S₀ → shipped, using n ≥ S − 1.
- **Go/no-go:** b_min(τ) = max over Wikidata blocks and positions of (F_t − τn_t); then the
  feasibility inequality.
- **Illustrative:** τ = 1/64 sits about 16× above Wikidata's aggregate rate and 48–58×
  below the controls' (estimated).

No implementation or CI follows from it.

**The coordinator's action:** passed to the option C analysis, with three engineering
obligations to check from the asm and source:
- **(a)** the handoff cost H: whether P74's tail already sits in the shipped list's memory
  layout, so that materialising is one 32-byte head store;
- **(b)** zero new fast-path µops, spills or loop-carried dependencies;
- **(c)** whether the shipped checkpoint kernel can resume mid-interval. If it can't, the
  zero-delay property is lost for up to one checkpoint interval, which must be bounded.

## The message, as received

P74-C: eliminate monitoring windows entirely

New candidate: event-triggered budget switching. The existing output cursor is the
denominator. The high-rank fallback branch is the only place requiring a decision. No
explicit-token counter, periodic checkpoint or fitted low-rank credit.

Let count executed ranks . Maintain:

\boxed{F\le \tau n+b}

At a high-rank token, before executing its MTF update, test:

\boxed{F+1>\tau n+b}

If true, materialize the 256-byte list and execute that token with shipped MTF. Otherwise
increment and continue P74. The switch is permanent.

1. The mathematical advantage

Between high-rank events, is constant and cannot decrease. Therefore the inequality can
become violated only at a high-rank event.

This proves that checking solely inside the existing fallback branch is sufficient.
Monitoring delay is zero; no per-token debt maintenance is needed.

Assume conditional cost bounds:

d_{\rm high}\le u,\qquad
D_{\rm other}\le vn,

including context effects in . Let handoff cost , monitoring cost , and shipped time .

Because switching occurs before exceeding the budget,

D_{\rm total}\le (u\tau+v)n+ub+H+M.

Hence:

\boxed{
R\le\frac{u\tau+v}{c_{\min}}
+\frac{ub+H+M}{c_{\min}n}
}

No exposure-window assumption. No threshold evasion. No post-trigger overshoot. This is a
conditional cost-model proof, not a universal CPU-time guarantee; still need defensible
bounds.

2. The workload separation is enormous

Using observed token densities:

Corpus	, approximately
Wikidata	0.00096
Random	0.75
Records	0.90

Try as an analytical candidate, not a frozen experimental parameter.

It lies roughly 16× above Wikidata's aggregate rate and 48–58× below the controls.

Choose illustrative startup allowance . On random data, the threshold is reached after
approximately

n\approx\frac{8}{0.75-1/64}\approx11

decoded bytes, rather than waiting 4,096 tokens.

The potential control penalty becomes the cost of roughly eleven P74 tokens plus handoff,
not thousands.

Critical caveat: Wikidata's aggregate rate does not bound its early-prefix bursts. Evaluate
the maximum observed for every block before choosing .

3. Short-block hardening

The constant dominates as .

Use a pre-dispatch short-block guard based on Symbols.syms.len(), which is already known.
For valid decoded blocks, , where includes EOB, because each RLE2 codeword sequence
expands to at least as many output bytes as its RUNA/RUNB digits.

Route directly to shipped MTF. Then the fixed-overhead term is bounded by:

\frac{ub+H+M_0}{c_{\min}(S_0-1)}.

Choose from the overhead budget, not by trial-and-error benchmarking.

4. Engineering implementation constraint

The branch must live inside P74's existing path. Check after flushing any pending RLE2 run
but before modifying the MTF list for the current token. Preserve that token for the
shipped continuation.

The state is then clean: list, counts and output agree; no MTF update requires rollback.

Disassembly must establish zero new fast-path spills or additional loop-carried
dependencies. An extra register that forces head spills would invalidate the architecture.

5. Exact go/no-go analysis

Compute, from existing traces:

b_{\min}(\tau)
=\max_{\text{Wikidata blocks}}\max_t
\left(F_t-\tau n_t\right).

This gives the smallest startup allowance that never switches a measured Wikidata block
prematurely.

Then evaluate feasibility:

\boxed{
u\tau+v+
\frac{ub_{\min}(\tau)+H+M}{n_{\min}}
\le0.03c_{\min}
}

using consistent time units and a defensible minimum block size after the short-block
guard.

If no satisfies the conditional cost envelope, formal worst-case safety is unavailable. An
empirical pre-registered gate may still be viable.

Decision: ask the agent to evaluate this event-triggered policy against its windowed/debt
alternatives. It removes the two hardest costs—per-token accounting and monitoring
delay—by exploiting an exact monotonicity invariant in the actual decoder. No
implementation or CI authorization follows from the derivation.
