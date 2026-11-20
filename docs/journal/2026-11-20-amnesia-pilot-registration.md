# The context-light generation pilot: registration

*20 November 2026. The Coordinator. Written before either session's results are seen.*

## The question

Does Neon's research context anchor what it generates? Neon reads the charter, CONTEXT.md, FRONTIER.md
and the journals before proposing anything, and cycles 1–2 produced variants of classical inversion
machinery. A fresh session that sees only the problem may propose different primitives.

This is one exploratory observation, not a causal test. Two sessions differ in many ways, so the
result can raise or lower our credence in the anchoring hypothesis but cannot establish it.

## The arms

- **Context-rich:** Neon's cycle 3, on the full repository and charter, as it runs.
- **Context-light:** a fresh Antigravity session with no mibz2 checkout. It gets only
  `amnesia-kit.tar.gz`:
  - the contract (README.md);
  - the six authenticated blocks and their manifest;
  - `verify.py`, the oracle;
  - the bar to beat, stated as a cost (about 1.6–3.7 ms per block, about one dependent load per ten
    output bytes), not as an algorithm.

  The kit names no mibz2 algorithm, journal or charter term.

## The context-light session's budget and phases

1. **Generate:** one session at the same effort setting as Neon.
2. **Freeze:** write `PROPOSALS-FROZEN.md` with up to five candidate mechanisms and its SHA-256, before
   reading anything about mibz2. Ordinary mathematical literature is allowed.
3. **Falsify:** run its own cheap falsifiers inside the kit: exactness via `verify.py`, plus counts it
   defines.
4. **Stop.** It never sees mibz2's history.

## Evaluation (the Coordinator, with full context, after both sessions)

For each arm's top three candidates, by the arm's own ranking:
- **(a) Mechanism:** a distinct primitive or formulation, or a known mibz2 mechanism or close
  variant. The comparison is the anonymised structural mechanism, not the vocabulary.
- **(b) Ancestry:** whether it appears in mibz2's history (CONTEXT.md §3–4, the journals), and its
  nearest literature ancestor.
- **(c) Exactness:** on the six blocks.
- **(d) Paid cost:** its producer-plus-consumer proxy, dependent loads per output byte and bytes
  moved per byte, measured by me on a shared instrumented harness.

**The judgement, fixed now:**
- **Anchoring more credible:** context-light yields at least one candidate that is exact, has a
  distinct mechanism, and is absent from mibz2's history, where context-rich's top three yield
  none.
- **Anchoring less credible:** the reverse; or context-light's top three are all rediscoveries of
  LF, Ψ or other textbook inversion, or are inexact.
- **Uninformative:** anything else, recorded as such.

No charter change follows from one pilot either way.
