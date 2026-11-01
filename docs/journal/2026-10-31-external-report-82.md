# External research report 82: P74-C's stage result: both behaviours shown, not yet combined (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 10:23 UTC.
- **Source:** unattributed in the relay.
- **In reply to:** the P74-C i9 stage trial (NEGATIVE as registered).

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
```
This is a more interesting outcome than the negative verdict alone suggests. P74-C has demonstrated the two behaviours we wanted, but hasn't combined them into a sufficiently reliable general-purpose optimization.

The distinction is important:

- The Wikidata gain is real and replicated: approximately 18–21% faster MTF-stage execution on your i9, with larger gains on the AMD CI runners.
- The bypass works: random and records went from substantial regressions under P74-Free to approximately baseline performance under P74-C.
- The online guard works: even adversarial blocks with a rank-local prefix followed by a high-rank suffix benefited from switching.
- The registered experiment is negative: the mixed-independent control and seven adversarial rows failed their criteria.

We shouldn't reinterpret the last finding merely because the first three are encouraging.

## What I find most significant

The Intel results make the architectural contribution unusually clear.

| Corpus                        | P74-Free versus shipped | P74-C versus shipped             |
| ----------------------------- | ----------------------- | -------------------------------- |
| Wikidata checkpoints          | Substantial improvement | \~20% faster                     |
| Random                        | 21–23% slower           | Approximately unchanged          |
| Records                       | 39–44% slower           | Approximately unchanged          |
| Mixed-independent checkpoints | —                       | 1.7% slower, interval up to 3.3% |

That is strong evidence that the mechanism solves much of P74's original workload-selection problem.

Previously, the census selector failed because inspecting every encoded symbol imposed a significant cost even when it chose the shipped kernel.

Now the remaining overhead appears substantially narrower: per-block dispatch, layout effects and particular adversarial cases.

That is scientific progress, even though the engineering gate failed.

## Two things I'd challenge before choosing the next experiment

First, the approximately 1 µs small-block dispatch cost deserves a precise breakdown.

If that's an unavoidable architectural cost, it may limit the applicability of the design. If it's caused by how the experimental harness invokes the kernels, a production-equivalent implementation could behave differently.

But we should not assume the latter. The next investigation should identify the actual machine-code mechanism rather than label the overhead as merely dispatch.

Second, the code-layout effect is large relative to the remaining regressions.

Claude reports approximately ±5% depending on kernel configuration. That's potentially larger than the control regression we are trying to eliminate.

A new experiment must therefore control function placement and distinguish a genuine guard improvement from an advantageous binary layout. Otherwise we risk optimizing against compiler placement noise.

## I would not immediately authorize another P74 experiment

We now have two successive negative registered gates, despite increasingly convincing evidence for the underlying technique.

That's not a reason to abandon P74 intellectually. It is a reason to avoid letting it become an indefinite sequence of small adjustments designed to clear increasingly specific test failures.

My preference would be:

1. Preserve the P74-C results and close the confirmatory experiment under its registered verdict.
2. Perform a bounded, read-only investigation of the small-block dispatch cost and layout sensitivity.
3. Decide whether a new architecture-level implementation is justified, rather than changing thresholds or dispatch rules merely to pass the existing controls.

I would also retain the speculative BWT traversal proposal and N1's partial-upgrade construction as independent research directions. Neither should be displaced indefinitely by P74 tuning.

### One practical point

The output-path correction is appropriate. Defaulting to `target/p74c/` is much cleaner and makes generated artifacts easier to find and remove. The pinned benchmark remains reproducible because the old runner was not modified.

I would keep the original result archive and journal intact, then clean the old home-directory outputs if you've confirmed the archive is preserved.

My overall assessment: P74-C is a successful mechanistic experiment and an unsuccessful acceptance test. We're closer to a deployable optimization than before, but the remaining work must demonstrate an actual economic advantage rather than merely refine the classifier around known failing cases.

The next scientific question is whether those residual costs are intrinsic to online selection—or artifacts of the particular experimental implementation.
```


guardian:


```
I think we may be slightly too strict about what counts as a useful optimization—but I would not merge P74-C into production just yet. And your news about Astra's machine-code optimization is important enough that I would reconsider what experiment comes next.

The central issue isn't whether P74-C works. It demonstrably does. It's whether we can ship it without introducing regressions that outweigh its benefits.

## What the evidence supports

| Question                                                  | Current answer                                                              |
| --------------------------------------------------------- | --------------------------------------------------------------------------- |
| Does register-resident MTF accelerate Wikidata?           | Yes. Replicated on Intel and AMD                                            |
| Does bypassing P74 protect random and records?            | Largely yes. Near-baseline performance on the i9                            |
| Does the online guard handle changing rank distributions? | Yes, on the adversarial cases tested, though not all performance gates pass |
| Did P74-C meet its frozen acceptance criteria?            | No                                                                          |
| Is P74-C ready for an unconditional production merge?     | Not established                                                             |

The distinction between a failed statistical gate and a demonstrated harmful regression matters.

For example, the i9's mixed-independent checkpoint result was +1.7%, with a 95% interval of approximately +0.1% to +3.3%. That fails the registered requirement to establish a regression below 3%, but doesn't demonstrate that the actual regression exceeds 3%.

Similarly, some adversarial failures were caused by wide multiplicity-adjusted intervals rather than clear performance losses.

So yes, there is arguably more economic promise here than the word negative suggests.

But we deliberately made the criteria strict to prevent ourselves from rationalizing failures after seeing results. We shouldn't retroactively call the gate passed.

## Why Astra's code-size result matters

Astra reportedly reduced the combined checkpoint function from 4,741 bytes to 4,091 bytes.

That's a reduction of:

\\[ \frac{4741-4091}{4741}\approx13.7\\%. \\]

There are three distinct possibilities:

- Less code to fetch and decode. Smaller hot functions can reduce instruction-cache and instruction-TLB pressure.
- Better code layout. Rearranging a loop can alter branch placement, alignment, and instruction delivery.
- Fewer executed operations. If the changes also simplify the dynamic instruction stream, execution may become faster independently of code size.

But these are possibilities, not demonstrated performance mechanisms.

The 13.7% figure by itself doesn't establish a speedup. A smaller function can execute the same number of instructions, or even more instructions, on its hot path.

Given that P74-C already exhibited sizeable code-layout sensitivity, however, I would take this result seriously as a specific lead worth measuring.

One clarification: I haven't verified that Astra's 4,091-byte function is the same P74-C kernel and compiled revision used in the negative confirmatory run. That provenance matters.

## What should we do next?

My recommendation is one new, focused production-candidate experiment, not another selector-threshold search.

I'd make the next hypothesis:

> Reducing P74-C's generated-code footprint and refactor/dispatch overhead will preserve its measured Wikidata improvements while bringing mixed and adversarial workloads within the existing regression limits.

That is a meaningful new mechanism if Astra's changes actually affect the P74-C hot path or its bypass. It is not an excuse to retune a guard around known failing cases.

Before preregistering, Claude should establish exactly what changed in machine code. The next comparison should use the original frozen P74-C as the control, hold the selection policy constant, and use fresh validation data rather than treating previously examined adversarial cases as untouched evidence.

We should also make the gate more directly relevant to shipping: test the complete decoder at one, two and four workers, with CPU-seconds/GB and peak RSS recorded. The original stage result is strong enough to justify investigating full-decoder economics, but not to assume them.

That gives us a bounded path toward merging rather than an indefinite programme of small stage-level experiments.

## What I'd tell Claude

I want to evaluate whether P74-C can become a production-worthy optimization, without reopening or overriding its negative confirmatory verdict.

Before proposing the next preregistration, please inspect the recent Astra/Codex machine-code optimization and establish its exact provenance: commit, affected functions, compiler, architecture, and whether the reported checkpoint function shrinking from 4,741 to 4,091 bytes applies to the P74-C implementation or a different decoder path.

Compare the generated hot paths with the frozen P74-C implementation. Identify whether the change reduces dynamic instructions, improves layout, eliminates dispatch overhead, or merely reduces total function size. Separate observed disassembly facts from performance hypotheses.

If it genuinely addresses the measured P74-C costs, propose one bounded new preregistration: original P74-C versus the optimized implementation, with the policy held fixed, fresh held-out controls, and a production-equivalent full-decoder comparison including wall time, CPU-seconds/GB and peak RSS at 1, 2 and 4 workers.

Do not relax the old gate, change P74-C's thresholds, or merge production code before the new evidence. Stop after presenting the proposed protocol for authorization.

## My decision

I would choose another experiment, but make it the last P74-C experiment unless it produces convincing production-level evidence.

I wouldn't merge the existing research implementation wholesale. It contains experimental arms, harness machinery and a policy whose registered acceptance test failed.

But I also wouldn't discard an optimization that is approximately 20% faster on your i9's Wikidata checkpoint stage and has nearly eliminated the original random/records regressions.

If Astra has genuinely improved the production-relevant machine code, that may be the missing piece.

We're close enough to a plausible shipping candidate to justify one more well-targeted test. We're not yet close enough to claim it's safe to merge.
```


the research programme chatbot agrees:


```
My decision: pursue shipping P74-C, but not another selector-tuning experiment. We have enough evidence to move to a full-decoder trial.

We are being too strict in interpreting the failed gate, but we must retain its recorded negative verdict. Four hosts show substantial Wikidata gains, including held-out windows; random/records are approximately unchanged; the online guard demonstrably protects against distribution shifts.

The failure is narrower than “P74-C doesn't work”:

- i9 mixed checkpoint: +1.7%, upper confidence bound +3.3% against a +3% limit.
- Seven adversarial rows fail, several because 54-way multiplicity correction cannot establish very tight bounds, not because their measured runtime exceeds the limit.
- Three Zen CI jobs pass the joint negative controls; their adversarial failures concentrate on bypassed edge blocks, tested against an unusually strict +1% bound. [CI results](https://github.com/lmmx/mibz2/blob/research/p74-c-implementation/docs/journal/2026-10-31-p74c-ci-41190706676/results.md?utm_source=chatgpt.com)

A failed proof of non-regression is not proof of an economically important regression. Equally, it is not permission to relabel the frozen gate a pass.

### Why the remaining failures don't justify another selector redesign

I checked `p74_c.rs`: when the bypass fires, T2 immediately calls `super::mtf` or `super::mtf_checkpoints`—the same shipped implementation used by T0. A measured \~1 µs discrepancy on those blocks is thousands of CPU cycles, far too large to attribute to a few dispatch comparisons alone. Its source could be code placement, cache state or timing variance. Do not tune the selector to cure it.

### The machine-code discovery is important

Reducing the checkpoint function from 4,741 to 4,091 bytes is a 13.7% static-code reduction, not a 13.7% speedup. The more consequential finding is that the error-state refactor changed register allocation: the output cursor could stay in a register instead of repeatedly loading and storing stack state. That can shorten an actual loop-carried dependency.

The i9's T1/T0 measurements corroborate that code generation matters: approximately 5% faster checkpoint execution but 5% slower plain execution, with no P74 algorithmic change. This is evidence for register-allocation sensitivity, not proof that smaller binaries are inherently faster. Any newer Astra fix that changes executable code needs its own binary audit and measurement; the pinned i9 result cannot validate it.

### Next hypothesis to preregister

> Production-equivalent P74-C dispatch improves whole-decoder throughput on 1, 2 and 4 physical cores, without material CPU-s/GB, memory or negative-control regression.

Freeze one implementation, retain the existing inclusive guard, verify correctness on all adversarial files, and measure full decoding against shipped in a same-binary A/B comparison. Make four physical cores the primary target; report one-core performance separately. Include Wikidata fitted/held-out, random, records, mixed and text. Predefine a ≤3% upper regression bound for controls, CPU efficiency and peak RSS limits, and a positive four-core throughput criterion. No threshold search, no repeated stage qualification.

Merge recommendation: merge the reviewed research implementation behind an off-by-default experimental path now, if desired. Do not enable it as the production default yet. One full-decoder experiment is the remaining decision, not another sequence of P74 tuning rounds.

If that experiment shows a material four-core win with acceptable CPU and memory costs, ship it. If not, archive P74-C and return to the dominant BWT walk.

My position: P74-C has earned an integration trial. It has not earned indefinite experimentation, and the marginal statistical failures should not veto a properly controlled decoder-level evaluation.
```

guardian:

```
Yes, I agree with the researcher here. And I think its recommendation is better than my previous proposal.

P74-C has earned a full-decoder trial. It does not need to pass another stage-level optimization experiment first.

The researcher makes a distinction that I should have given more weight: failure to demonstrate a regression below a strict statistical limit is not the same as demonstrating a practically significant regression.

The i9's mixed checkpoint result illustrates this perfectly. A measured 1.7% slowdown with an upper confidence bound of 3.3% fails the registered 3% criterion. But it would be unreasonable to treat that as equivalent to the original P74's 20–80% control regressions.

Likewise, applying a stringent multiplicity correction across 54 adversarial comparisons provides strong protection against false declarations of success, but can make it difficult to establish extremely small performance differences with confidence.

That doesn't make the original gates mistakes. They were useful for preventing premature promotion, and they've already exposed important problems. But a stage-level statistical gate shouldn't become a permanent barrier to testing the complete decoder, particularly when the observed failures are narrow and the proposed integration changes what we're measuring.

### Where I now stand

| Decision                                                       | My recommendation                             |
| -------------------------------------------------------------- | --------------------------------------------- |
| Preserve P74-C's negative confirmatory verdict                 | Yes                                           |
| Another selector or fallback tuning experiment                 | No                                            |
| Integrate behind an off-by-default experimental path           | Reasonable, after correctness and code review |
| Measure the complete decoder on four physical cores            | Yes, prioritize this                          |
| Enable P74-C by default                                        | Not yet                                       |
| Continue P74-C indefinitely if full-decoder results disappoint | No                                            |

The machine-code findings make the full-decoder trial more important, too.

The output cursor remaining in a register rather than repeatedly reloading from stack memory is a plausible mechanism for a genuine runtime improvement. That is more consequential than the 13.7% reduction in static function size.

But it also tells us something about the experiment: we should freeze the exact optimized implementation that will be tested, rather than implicitly transferring performance results from the earlier pinned binary to newer code.

I particularly agree with making four physical cores the primary target. The shared GitHub runners expose only two physical cores with SMT, and the performance economics of your intended four-core workflow cannot be established from those machines alone.

One distinction I'd retain: merging a research implementation behind an off-by-default feature or dispatch option is not the same as shipping it as the default. The former can be a controlled engineering step; the latter requires the whole-decoder evidence.

And I would explicitly budget CPU-seconds per GB, not just wall time. We have already seen that faster decoding can consume disproportionately more CPU without producing an equally valuable end-to-end improvement.

My revised recommendation is therefore straightforward: one frozen, production-equivalent full-decoder experiment, with a clear stopping rule.

If it produces a material four-core improvement while preserving CPU efficiency, memory and control-corpus performance, proceed toward release. If not, archive P74-C and redirect the programme toward the BWT walk.

I also think your decision to return leadership of the research programme to the researcher makes sense. It has continuity with the original hypotheses, constraints and experimental design, while I can remain useful as an independent reviewer of the results.

The aim was never to collect perfect stage-level confidence intervals. It was to make the decoder meaningfully faster without unacceptable trade-offs.

We now have enough evidence to test whether P74-C actually achieves that.
```
````
