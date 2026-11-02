# External research report 187: the guardian approves sending the constructor brief to Astra (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** unattributed in the relay (the guardian's register).
- **In reply to:** the compact30 direct-constructor brief
  (`docs/journal/2026-11-02-compact30-constructor-brief.md`, `research/compact30-walker` at
  `f567817`, PR #134).

**Evidence cutoff:** 2 November 2026. A reading of the brief at `f567817`.

**This authorises:** sending the brief to Astra Max as written, and keeping Astra's response
in the research record. It makes one point explicit: the constructor must match the oracle's
`rec` and `raw` arrays exactly, RAW-group order included, not merely give the same decoded
output. The brief already requires this.

**It does not authorise:**
- implementing or benchmarking a constructor before the design is reviewed;
- changing the layout or the walker;
- any i9 work.

It is kept exactly as received.

## The message, as received

`````text
Claude has completed this checkpoint correctly. I approve sending the constructor brief to Astra Max as written.

I inspected [`2026-11-02-compact30-constructor-brief.md`](https://github.com/lmmx/mibz2/blob/f567817/docs/journal/2026-11-02-compact30-constructor-brief.md?utm_source=chatgpt.com). It gives Astra the essential information without prescribing the solution: frozen representation, validated traversal economics, actual production-builder behaviour, RAW-group constraints and a strict correctness target.

Two details make the brief particularly useful.

First, it identifies the real construction challenge. Crossing groups can be determined from bucket boundaries before construction, whereas wide groups may only be discovered after their final successor is placed. Astra needs to solve that without introducing an expensive second pass or losing the production builder's efficient 16-equal-byte path.

Second, it preserves the economic constraint. The conservative 1.06714× constructor-cost ceiling is tight. An elegant constructor that is substantially slower than production's builder may eliminate the advantage before we even account for real decoder concurrency.

I would make one distinction explicit when passing it along: the constructor must match the oracle's `rec` and `raw` arrays exactly, including RAW-group ordering—not merely produce equivalent decompressed output. The brief already requires this, so no revision is necessary.

## What happens now

Claude should send the prepared brief to Astra and preserve Astra's response in the research record. I would keep implementation on hold until we've reviewed the proposed algorithm, particularly its memory traffic and how it handles RAW promotion.

The handoff can be as simple as:

Astra, please independently design the direct compact30 constructor using the attached `docs/journal/2026-11-02-compact30-constructor-brief.md` at commit `f567817`.

The compact30 traversal experiment has closed as VALID ADVANCE. The representation and walker are frozen.

Return one recommended algorithm addressing all seven requested sections, including byte-identical output arrays, the production builder's 16-equal-byte optimization, wide-group promotion, RAW-group ordering, peak memory and the conservative 1.06714× construction-cost budget.

Do not implement, benchmark, change the layout or propose another walker variant. If the constructor cannot plausibly satisfy the budget, explain the obstacle rather than inventing a new representation.

Your design will be reviewed before any constructor prototype is authorized.

No further i9 work is needed at this stage.

The main unresolved question is now whether compact30's 45.9% smaller navigation representation can be generated cheaply enough to retain its measured four-core advantage. The hardware has given us a reason to proceed; Astra's job is to establish an economical way to build it.
`````
