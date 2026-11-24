# Brief: the four-arm whole-decoder oracle (scanner × emission)

*Commissioned by Louis on 24 November 2026 ("yes commission the four-arm test! … I think pathfinder is the session best suited to this"). Assigned to the Pathfinder. The design is from ChatGPT Pro (journal §42, `chatgpt-closure.md` §6).*

## The question

Does removing **two** costs together free whole-decoder time that removing either alone does not? The two costs are the Huffman/MTF scanner's work and Open's code-ordering in emission. Only one cell has been measured: the supplied-codes run (journal §30; ready tag `ready/bench-supplied-codes` at 5e5495c5 on branch `bench/supplied-codes`; decision rule LOG 7131301798, verdict LOG 7132792060). It halved the emitter: 0.53 at 1 worker, 0.55 at 10. The whole decoder moved only to 0.838 at 1 worker and 0.988 at 10 (CI [0.957, 1.020]).

## The arms

| Arm | Scanner output | Open emission |
|---|---|---|
| A | normal scanner | normal |
| B | normal scanner | supplied codes (the existing intervention) |
| C | **precomputed ordinary Symbols output** | normal |
| D | precomputed Symbols output | supplied codes |

- **Precomputation sits outside the timer.** This is a diagnostic of potential, not an implementation.
- **Delivery stays timed.** The precomputed symbols and codes go through the production bounded queue.
- **Everything else is kept and timed:** construction, word handling, RLE1, CRC and output.

## What to build

A frozen workstation recipe, following CLAUDE.md "Workstation recipes": one new directory per attempt, a console log, `bench/bundle.sh`, and `just submit` for evidence.
- Base it on the supplied-codes recipe. B must reproduce §30 within noise, as the check that the harness is unchanged.
- Workers 1, 6 and 10; wikidata ×4. The repeats, rounds, settle and arm permutation follow the supplied-codes recipe unless there is a reason to change them.
- Record peak RSS: precomputed symbols cost memory, so report it.
- Any change to `src/` goes behind a research feature or bench-only path. Production defaults are untouched.

## The decision rule, registered before the data

This is a DIAGNOSTIC (RULES C1a), so the rule is posted on #145 before the run. At 10 workers, with ratios to A (95% intervals from the paired rounds):
- **INTERACTION** iff D ≤ 0.90, and D is below min(B, C) − 0.03 with that difference's interval excluding 0. The two capabilities together are worth an algorithm that changes both interfaces.
- **ADDITIVE** iff D ≤ 0.90 without the interaction condition. Pursue the larger single arm first.
- **NO POTENTIAL** iff D > 0.95. This pair of capabilities is closed as a joint target.
- Otherwise **MARGINAL**, reported as such.

Also report the same figures at 1 and 6 workers, without a decision.

The Pathfinder may refine these thresholds once, in the registration, with a reason, before any data.

## Process

1. The Pathfinder builds the recipe on a branch, runs it locally only for correctness, and opens a PR.
2. It posts the registration and the decision rule on #145, then reports back. The Coordinator reviews the PR, merges it on green, and sets `ready/<recipe>` under the standing authorisation. The workstation queue runs it.
3. The Coordinator reads the bundle and posts the verdict.

The rules throughout: commits as Louis Maddox <louismmx@gmail.com>, no AI attribution, no allocator changes, no Hub access.
