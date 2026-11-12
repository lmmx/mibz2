# External research report 97: accept the W4(c) inlining difference; approve exploratory CI (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 19:38 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the walker decoder trial's dry run.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
I'd accept the W4(c) inlining difference and approve exploratory CI, while keeping the i9 trial separate.

Claude has reached a sensible review checkpoint. The dry run passed correctness, corpus reproducibility, affinity, and the structural checks apart from an explicitly disclosed compiler-layout difference.

The important distinction is that different inlining is not evidence of a different decoding algorithm. Rust has generated separate monomorphizations of the same decoder pipeline, and LLVM has chosen different function boundaries.

However, I wouldn't accept Claude's estimate that the cost is necessarily just a few calls per block. Inlining can also affect register allocation, instruction-cache pressure and optimization across function boundaries. That means W4(c) is a legitimate performance confounder, even though it isn't a correctness problem.

My judgment is that we can measure the integrated implementation as it would actually run, provided that difference is documented and its contribution isn't misrepresented as purely the walker algorithm.

### My decisions on Claude's four review points

| Item                      | Decision               | Reason                                                                                 |
| ------------------------- | ---------------------- | -------------------------------------------------------------------------------------- |
| W4(c): different inlining | Accept with disclosure | Same functional pipeline; compiler differences remain part of the measured integration |
| rustc 1.97.0              | Accept                 | Matches the successful stage experiment and was chosen before measurement              |
| zeros exercises fallback  | Keep it                | Useful control, but correct its description                                            |
| Audit-tool corrections    | Accept                 | Necessary to identify the actual code paths; report their limitations                  |

There are two operational details worth addressing.

First, do not alter the unrolled kernel to force matching assembly. That would turn this into another optimization experiment. We already have a correct, stage-validated implementation; the goal now is to assess its complete-decoder economics.

Second, I would avoid rewriting Git history merely to remove an incidental process name from an environment snapshot. A process called `claude` is not itself an AI-attribution trailer. For future public records, Claude can exclude process command lines or retain only numerical resource information. It should still preserve CPU-affinity and workload evidence.

### What to send Claude

I've reviewed the frozen walk-decoder preparation report and approve W4(c) as a documented integration-level code-generation difference.

The source-level stages and their reachable build, MTF, RLE1 and CRC functions are equivalent, and W1–W4(b) pass. The different `decode_block` and segment-assembly inlining does not invalidate the experiment. However, don't assume it costs only a few calls: acknowledge that layout, register allocation and instruction-cache effects can contribute to the observed A/B performance.

Please record my acceptance of the exact W4(c) signature in a documented, versioned pre-timing amendment. Preserve the existing protocol and original frozen records, and rehash or repin the harness as required. Do not alter the kernels or other performance-critical code to force identical disassembly.

I also accept rustc 1.97.0, the disclosed audit-tool corrections, and retaining `zeros` as a short-block fallback control. Correct its descriptive classification without changing the corpus or its hash.

I authorize the three-job exploratory CI timing once the amended runner passes its pre-timing checks. Use the frozen thresholds, correctness checks, same-binary A/B/A′ comparisons and existing stopping rules. Do not run or rerun the i9 trial, enable production defaults, merge, or combine with P74-C.

Preserve the exact timed binary and results. For future environment snapshots, avoid incidental process names or command lines where they aren't needed. There's no need to rewrite published history solely to remove the previously captured `claude` process name.

Please report the exploratory CI outcomes, including CPU efficiency, RSS and any possible layout effects, then stop for review. The frozen P74-C i9 trial remains our next local workstation priority.

### Why I think we should proceed

The research question is now sufficiently well isolated:

Does a decoder using the constant-index BWT walker outperform the otherwise identical shipped decoder when scheduling, worker allocation and MTF are held constant?

The successful stage experiment showed approximately 12% less walk time on the counted Zen 3 host. We want to learn how much survives in the parallel decoder, where scanner contention, buffers and other stages matter.

The W4(c) difference slightly complicates attribution but does not prevent a meaningful production-equivalent comparison.

One caution for the eventual results: if CI passes, that still doesn't establish a four-core i9 improvement, and a CI negative should not automatically overrule the i9 primary gate because the hosts expose different core configurations.

So my call is: accept W4(c), document it before timing, run the exploratory CI, and keep the i9 for the definitive separate trial after P74-C.
````
