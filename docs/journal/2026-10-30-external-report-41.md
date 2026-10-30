# External research report 41: after Gate 1's negative verdict (verbatim)

The maintainer's message of 2026-10-30, after the selector arm's result. It contains a
proposed message to the coordinator and two assessments (the guardian's and the
researcher's).

**What it asks for:**
- **Accept the negative verdict**, and close PR #118 with a results comment linking the
  journal.
- **No new arm,** no selector tuning, and no reopening of Gate 1.
- **A read-only assessment** of counting high ranks during the scanner's production of
  `Symbols.syms`, against a standalone AVX2 count, with nothing implemented or
  benchmarked.
- **A full-cost charge** for any future proposal's classification, wherever that cost
  lands.
- **No change** to wikidata-pq #97. Phase 2, N1 Gate B, the i9 and production stay
  unauthorised.

**The coordinator's response:**
- **The assessment** is `2026-10-30-p74-selection-architecture.md`. It adds a third
  option the code suggests, an online switch inside the kernel.
- **PR #118 stays open until the maintainer confirms:** the instruction to close it came
  inside a "Proposed message to Claude", not in the maintainer's own words.

## The message, as received

````text
Gate 1 is negative under its registered rules, but I would not abandon the underlying P74 optimization.

The result is unusually informative: the selector made the right decisions and still failed because making those decisions cost too much.

That separates two questions we had previously bundled together:

* Kernel economics: register-held P74 improves Wikidata’s checkpoint MTF stage by 19–33% across the two tested CI environments, while regressing on high-rank inputs.
* Selection economics: the current classification pass costs enough to erase much of the Wikidata gain and introduce 6–7% regressions even on blocks routed entirely to the shipped kernel.

The second result is particularly important. Random and records selected zero P74 blocks, yet the selector still regressed. That’s strong evidence that the selection machinery, rather than the P74 kernel, is responsible for those regressions.

The measured verdict must stand. But it would be a mistake to turn “this selector failed” into “adaptive register-held MTF is uneconomic.”

My recommendation: close #118, preserve the discovery, and investigate the scanner before authorizing another experiment

I would make two decisions differently from simply running another optimized count.

First, close PR #118 with its negative registered verdict. The frozen experiment is complete, and keeping it open won’t improve the evidence. The result deserves a precise record, including the successful fast path, the negative controls, the classifier cost, and the Zen 4/Zen 5 distinction.

Second, do not immediately authorize an AVX2 counting experiment. The more interesting possibility is to avoid counting the symbol stream twice at all.

Claude mentioned that the scanner might retain the count. That’s potentially a better architecture than making the extra pass faster. If the existing symbol-producing stage already encounters each explicit MTF rank, maintaining a high-rank counter there could be almost free relative to a separate scan—although that needs code inspection, not assumption.

The distinction matters especially for random and records: those are exactly the corpora where every block selects the shipped kernel, so any separate full-symbol classification pass becomes pure overhead.

There is one caveat: moving work into the scanner changes the stage boundary. A future experiment would need to charge the additional counting work to the whole relevant pipeline, rather than claiming a free selector because the expense has moved outside the timed MTF function.

Proposed message to Claude

I accept the registered NEGATIVE verdict for P74 Gate 1.

Please close PR #118 with a concise results comment and link to the completed journal on #117. Preserve the distinction between the successful register-held fast path, the uneconomic high-rank fallback, and the selector’s measured classification overhead. Record the Zen 4 versus Zen 5 runner difference.

Do not add a new implementation arm, optimize the selector, or reopen Gate 1.

Before proposing another experiment, I’d like a read-only architectural assessment of whether the high-rank count can be accumulated during the existing Symbols.syms production path, without an additional pass. Identify exactly where ranks become available, whether the required per-block metadata can be retained, and what additional operations, storage, or synchronization this would entail.

Compare that prospect with a standalone AVX2 count, but do not implement or benchmark either.

Any future proposal must separately preregister its hypothesis and measure the full cost of classification, even if that cost is moved into an earlier pipeline stage.

Leave Phase 2, N1 Gate B, i9 benchmarking and production integration unauthorized. No change to the wikidata-pq queue issue #97 for now.

One further scientific insight is worth preserving: the register-head hypothesis survived, but the proposed general-purpose implementation did not. The substantial Wikidata stage gain was reproduced on two different CPU generations; the evidence is now considerably stronger than our original cost model.

The next potentially useful advance isn’t necessarily a faster MTF kernel. It might be an architectural way to make the choice of MTF kernel available without paying a separate classification pass.

That deserves investigation, but not an exception to the stopping rules.

This is a disappointing result for Gate 1, but scientifically it’s also one of the most informative outcomes of the entire burn-down.

My assessment: accept the registered negative verdict for P74 Gate 1. Preserve the evidence, and do not reinterpret the stopping rules because we’ve discovered a plausible optimization afterward.

However, I would make one important distinction: the selector failed because its classification cost was too high, not because its classification decisions were wrong.

What the experiment established

P74 Gate 1

Negative

Wikidata checkpoint MTF · selector

−7.8%

95% CI: −8.5% to −7.1%

Random checkpoint MTF · selector

+7.2%

95% CI: +6.8% to +7.6%

Records checkpoint MTF: +5.8% (95% CI +5.3% to +6.4%). Both controls breach the registered +3% limit.

A particularly useful finding is that the selector routes every random and records block to the shipped kernel, yet still slows both workloads by around 6–7%.

That isolates an important part of the problem: the selection overhead is sufficient to fail the gate even when P74’s expensive fallback never executes on those blocks.

The compiled-code finding—baseline SSE2 counting with 64-bit accumulators—is a credible explanation for that overhead.

What remains valuable

Research claim	Result
Register-held MTF is algorithmically correct	Established
Register residency survives compilation on fast paths	Established
Complete MTF stage can improve substantially on Wikidata	Measured: 19–33%, depending on runner
Present P74 kernel performs well on high-rank workloads	Falsified
Present per-block selector meets negative-control limits	Falsified
A cheaper count could rescue the selector	Plausible, untested
Full decoder improves at four workers	Not demonstrated

The difference between the two CI processors also matters. The first gate used Zen 4 and the selector run used Zen 5. We must not pool their point estimates or treat their difference as the effect of introducing the selector.

The most interesting new research question

The result exposes a fairly precise economic constraint on any future selector.

On the controls, the selector chooses shipped MTF. Therefore, for those blocks:

T_{\mathrm{selector}}

T_{\mathrm{shipped}}+T_{\mathrm{classification}}+T_{\mathrm{dispatch}}

up to compiler layout and other measurement effects.

Under the registered 3% regression limit, its overhead must be small enough that:

\frac{T_{\mathrm{classification}}+T_{\mathrm{dispatch}}}
{T_{\mathrm{shipped}}}
\lesssim 0.03.

That is a strict budget on random and records, where encoded-symbol density approaches one token per output byte.

An AVX2 counter may reduce that cost. But the compiler’s instruction sequence alone doesn’t establish that it will meet the required budget, especially on the much faster Zen 5 baseline.

There is a more fundamental possibility worth preserving in the research archive: a selector that obtains its rank statistics as a by-product of work the scanner already performs could avoid a second pass. Its cost would then be distributed differently across the decoder pipeline, however, so it would require its own full-pipeline accounting. That is not a free optimization.

What I recommend

I would close PR #118 with the registered negative result, retaining the research journal, CI artifacts, correctness tests and disassembly findings as the experiment’s record.

I would not automatically approve a new AVX2-count experiment. It’s a reasonable future hypothesis, but it deserves a fresh prioritization against N1’s larger potential upside—not an exemption from Gate 1’s stopping rule.

Nor would I mark P74 universally rejected. I’d record it as:

P74 Gate 1: negative under the frozen acceptance criteria. Register-resident MTF produced reproducible Wikidata stage-level gains, but high-rank controls regressed and the preregistered selector’s classification overhead exceeded the control budget. Further work requires a separately authorized experiment.

For issue #97, I would avoid an unqualified “Rejected” label unless that issue specifically tracks P74. A journal cross-reference is sufficient.

My conclusion: Gate 1 did exactly what we asked it to do. It discovered a real optimization opportunity, demonstrated that the present implementation isn’t safe to promote, and identified the specific remaining economic obstacle.

The next choice—whether a cheap-count experiment deserves priority over N1 Gate B—is a research portfolio decision, not something the failed Gate 1 authorizes by itself.
````
