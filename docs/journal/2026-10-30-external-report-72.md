# External research report 72: P74-H, the shipped copy strategy in P74's fallback (verbatim)

A relayed message from the maintainer, unattributed in the relay.

**What it proposes:** keep P74's low-rank kernel, and replace the rank ≥ 32 masked chunked
tail update with an overlapping `ptr::copy` on the tail alone. There is no classifier,
counter, handoff or refactor.
- **The hurdle on Zen 5:** to keep the controls within +3%, H must save
  Δt_high ≳ 1.27 ns per high-rank event (random) or 1.08 ns (records).
- **Kill rule:** kill H if the predicted savings can't approach 1.1–1.3 ns per event.
  Otherwise prefer a separately authorised H experiment to C's refactor.

**The coordinator's action:** a read-only evaluation, from the extracted asm and glibc's
memmove source. It covers:
- the instruction, load and store counts by c and j;
- store-forwarding risks;
- reconciliation with the fit, whose per-fallback excess is +6–9 cycles and per-chunk
  term only 0–0.5. If the excess is fixed per fallback, H recovers little;
- the c-weighted predicted saving per runner;
- Gate 1's history: LLVM's memmove lowering was replaced by blended chunks as the one
  permitted disassembly fix, and "memmove beats the chunked tail shift" is recorded as
  a hypothesis, not a finding.

## The message, as received

New candidate P74-H: eliminate the expensive fallback instead of selecting around it. This
could make Option C unnecessary.

For , P74 currently performs masked AVX2 tail updates. Replace those with the shipped
kernel's overlapping ptr::copy strategy, operating on the tail alone.

Let head , tail . Exact transition:

v=Uj,\quad h=H31U1..j+1\leftarrow U0..j,\quad U0\leftarrow hH\leftarrow\[v,H\[0\],\ldots,H30].

The copy length is , not . Valid for every , including endpoints.

Consequences: unchanged P74 low-rank kernel; no classifier, counters, handoff, extra
allocation or checkpoint restructuring. Correctness follows directly from MTF rotation.

Quantitative hurdle. Zen 5 controls regress approximately 20% with current P74. To satisfy
+3%, fallback optimization must recover:

\Delta t_{\mathrm{high}}\ge
\frac{(r-0.03)T_{\mathrm{shipped}}}{F/n}.

Using ns/B:

\Delta t_{\mathrm{high}}\gtrsim
\begin{cases}
1.27\text{ ns/event}&\text{random}\\
1.08\text{ ns/event}&\text{records}
\end{cases}

That is substantial but plausible if eliminating repeated masked loads, blends and stores
saves several cycles per fallback. Not measured.

Decision: have the agent evaluate P74-H's instruction-count and memory-dependency model
against current P74 fallback and shipped memmove, including -weighted corpus distributions.
If the predicted savings cannot approach 1.1–1.3 ns/high-rank event, kill H. Otherwise
prefer a separately authorized P74-H experiment before implementing C's state-machine
refactor.

This is the more aggressive engineering simplification: make the fast kernel competitive
on unfavorable inputs rather than pay to detect and escape them. It targets a shippable
4–6% improvement; the ≥20% breakthrough still requires the independent walk work.
