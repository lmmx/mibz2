# The essence: what we are looking for, and how

*Set by Louis Maddox on 24 November 2026. Every brief, prompt and review in this campaign starts from this page. It is here so the campaign does not slide back into performance tuning, incremental improvements and partial results.*

## The aim

A breakthrough, not a partial result. In Louis's words: "Enough of partial results, we need to do a breakthrough."

## Where the breakthrough comes from

Rauzy–Veech induction was not found by counting machine cost. It came from seeing that a whole stretch of rows takes the same step, so their steps can be composed once and shared. Compare the two algorithms, the walk and the induction, and ask what carries over (the Hilbert's 24th problem question). The answer is **contraction along composition**, in two parts:

1. **A notion of "these move together".** For Rauzy–Veech, that is adjacent rows holding the same byte.
2. **A rule for composing what moves together,** then recursing on what is left.

The proved floor says that lone rows have nothing to move together with *in that sense*. *(Scope, corrected 24 November, journal §42: G1 is a floor on charged operations in the rigid-interval calculus. It is **not** a proved lower bound on unpredictable machine reads; the bridge between the two is unproved. An implementation may keep the same interval facts and acquire, arrange or consume them more cheaply.)* So the way forward is not to contract harder in the same sense. **Change what "moving together" means.** Louis: "a kind of a looseness of language, a generalization, work smarter, not harder … Find a different sense for something that can occur. It requires a kind of ability to think flexibly, laterally."

**All directions at once, and more.** The term is deliberately overloaded. Louis (24 November): "I don't want to frame it as one way only … the Hilbert's 24th problem movement. And however you call it, we overload it, right, with the multiple ways in which it might occur. The recurrence, the moving together, the contracting along the text. Like, we want to look at all possible ones. In fact, we might want to look at even more." Pursue every sense in parallel. None is the lead, and each is a reading of the same movement. Finding further senses is part of the task.

Another phrasing of the same movement, from the outside verdict on Louis's JAX idea (`jax-idea-verdict.md`): "Don't just look for rows that move together. Look for transformations that are secretly the same computation, expressed in different coordinates." It says the same thing as Louis's "replace the state of a computation with a representation of what the computation does": compose the transformations, not the states.

The senses named so far:
- **Contract along the text, not along the rows.** A repeated stretch of text is a run in the other direction. Copy plans are the start of this.
- **Moving together up to a consistent relabelling,** not exact equality.
- **A recurrence among the induction's own steps.** This is the Pell rule in doomslide's essay: a rule that generates new instances, where the oracle could only hand them over.

## A second principle beside it: enrichment along composition

From Louis's page-dewarp history (journal §§36–37). Louis: "we had a gradient based problem… and we improved the quality of the gradients involved". Automatic differentiation did not group anything. It **lifted the computation into a richer algebra**: pairs (a, a′), with (a, a′)(b, b′) = (ab, a′b + ab′). Extra information then came out alongside the ordinary result, and that information made a different, far cheaper algorithm possible. Keeping more of it (`maxcor` 10 → 100) removed most of the remaining work.

- **Contraction along composition:** do less work, by recognising that computations are the same.
- **Enrichment along composition:** make existing work yield more information, so that later work becomes unnecessary.

Pursue both. For enrichment, ask what extra quantity could ride along an existing step (an LF step, a run step, the Huffman, MTF or construction stages) under a richer algebra, at little cost. Then ask whether it makes a later stage cheap or unnecessary. Starting points, not answers: algebraic lifting, AD as a construction, discrete derivatives, semirings, auxiliary invariants.

The test that separates it from a restatement: the rider must be **cheaply acquired** and must **not just restate the inversion**. A fingerprint of the input that "separates every flip and decodes nothing" (N-18) fails. So does "the text position p(i)", which is the answer itself, with p(LF(i)) = p(i) − 1.

## A third question, sharper than "find a richer algebra"

From the page-dewarp lesson as ChatGPT Pro restated it (journal §42): "Obtain information that makes a more capable algorithm possible; then retain enough of that information that the algorithm can avoid expensive work." So ask: **which later operation is necessary only because an earlier stage failed to retain, or cannot cheaply exploit, information it already encountered?**

And the property any representation must have: **closure**. After composition, the result must still be cheap to use. The access description, the successor lookup and the word payload must all survive. Otherwise the work has only been deferred.

## The cost rules are the referee, not the generator

The agreed problem statement (`council/COMPARISON.md`) and Louis's choice of real machine cost decide whether a find is real: no uncharged passes, no hidden walk. They are applied after a candidate exists. They do not produce candidates, and they must not set the order of work.

The failure mode to avoid is the one both Antigravity chats fell into on 24 November (see the two critiques in this directory). Every candidate squeezed the *same* notion of moving together harder, and paid for it with extra passes or a hidden walk.

## What has been tried before

- **Earlier attempts are not banned.** Louis: "it's not necessarily the case that an idea we tried has to be disallowed because it's already been tried, but rather that we can perhaps … think again and redouble our efforts if we think there might be something in it."
- **But know them, and don't feign novelty.** "We shouldn't feign surprise at novelty if the thing turns out to already exist."

Before proposing an object, search the records: the `research/*` branches, the kitchen-sink and Astra investigations, MONODROMY's records and the journals. Cite what is found. Then say either "new", or "revisiting X because Y was missed".

The breadth of what has been tried is not the limit of what can be tried. "The fact that we've tried so many things already doesn't mean that that's the set we're limited to."
