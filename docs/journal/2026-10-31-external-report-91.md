# External research report 91: use the next ten hours for Astra's BWT-walker investigation (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 14:07 UTC.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** planning while the i9 was unavailable.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
My recommendation: use the next ten hours for Astra's BWT-walker investigation. Let Claude coordinate it, while leaving the frozen P74-C experiment untouched until you get home.

You have a particularly good opportunity here: the P74-C full-decoder trial is ready, and the preliminary CI results are encouraging. Meanwhile, the BWT walker offers substantially more room for a breakthrough.

I'd divide responsibilities like this:

Astra Codex — start now

Investigate the BWT walk's machine-code dependency graph using the prepared evidence pack. Ask for one concrete, falsifiable transformation, not a collection of speculative optimizations.

Claude Code — work during the day

Review Astra's findings when available, validate the claimed dependency mechanism against the disassembly, and prepare a narrowly scoped experimental protocol if warranted. It can also prepare the i9 benchmark handoff.

Your i9 — tonight

Run the frozen P74-C full-decoder trial at `3870c12`. No changes to its code or thresholds.

### 1\. Send this to Astra now

The evidence pack is committed at:

[BWT walker evidence pack](https://github.com/lmmx/mibz2/tree/c4018b54594f2ba7a6e50ad364b328f53a8980f6/docs/research/2026-10-31-astra-room-walk?utm_source=chatgpt.com)

Review the machine-code evidence pack in lmmx/mibz2 at commit c4018b54594f2ba7a6e50ad364b328f53a8980f6, directory docs/research/2026-10-31-astra-room-walk/.

This is a focused, READ-ONLY performance investigation of `bwt::room_segments::<32, PF_L1>` on the EPYC 7763.

The central question is why 32 independent logical BWT traversal lanes achieve only approximately 8.6 effective chains of overlap under the historical latency/throughput model.

Examine the annotated disassembly, lane-state stack traffic, dependent T-vector loads, output-pointer handling, prefetch instructions, register pressure and instruction-window limitations.

Pay particular attention to whether four interleaved groups of eight scalar lanes would retain concurrency across all 32 chains, and whether removing lane-loop bookkeeping could plausibly save approximately 1.15 cycles per decoded byte against the 10.78 c/B baseline.

Distinguish measured evidence from machine-code deductions and performance models. Do not assume the 8.6 figure is a literal hardware count of outstanding requests.

Return at most ONE supported transformation, with:
- the exact source-level mechanism;
- the expected assembly signature;
- a conservative cycles-per-byte prediction;
- the main risk to memory-level parallelism;
- a falsification criterion.

If no credible transformation exists, explain the limiting hardware mechanism instead.

Do not modify code, run benchmarks, open a PR, or change production decoding. Keep the response concise and technically substantive.

This is the best first use of Astra's limited quota.

### 2\. Give Claude one instruction for the rest of the day

I'm at work and won't have access to my i9 for approximately ten hours.

Please leave the P74-C full-decoder trial frozen at 3870c1225d4cfb1a87ade57651eb1165ce451692. No further P74-C changes, benchmark reruns or threshold adjustments. The RSS allocation finding is documented and we'll interpret it against the frozen i9 result tonight.

I am sending the existing BWT walker evidence pack to Astra for read-only diagnosis.

While I'm away, please prepare for a possible follow-up BWT experiment: check that the existing correctness suites, baseline stage benchmarks, lane-count sweep and assembly-audit tooling can support a separate research arm. You may review the proposed implementation protocol, but do not implement or time a walker modification until we have Astra's finding and have approved a specific hypothesis.

Please also make sure the P74-C i9 one-command runner and its expected SHA remain documented and accessible.

The next decision I want is whether Astra has identified a credible machine-code improvement worth testing—not another round of P74 tuning.

### 3\. Tonight: one meaningful i9 run

When you're back, the task is straightforward:

```bash
cd ~/dev/mibz2
git fetch origin
git switch research/p74c-decoder
git pull --ff-only
git rev-parse HEAD
```

Verify the output is:

`3870c1225d4cfb1a87ade57651eb1165ce451692`

Then:

```bash
just bench-p74c-decoder i9-10900K
```

The estimated duration is 30–60 minutes. Keep the workstation quiet, and preserve the resulting archive.

If the run shows approximately 2–4% full-decoder throughput improvement with lower CPU-seconds/GB, that's a meaningful engineering outcome, even if the strict RSS rule needs a separately validated production fix.

The larger strategy: let P74-C reach its shipping decision tonight, while Astra investigates whether the BWT walk can unlock the next 10% or more.

I wouldn't spend any more of your working day coordinating multiple research agents. One Astra prompt, one Claude instruction, then return to the results.
````
