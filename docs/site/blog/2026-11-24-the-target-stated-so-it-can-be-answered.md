---
title: "Halving the single-row steps, stated so it can be answered"
image: assets/social/blog-2026-11-24-the-target-stated-so-it-can-be-answered.png
description: "Before we attempt a construction, we fix how cost is counted and what counts as a pass or as new so that either answer settles something."
---

<div class="post-hero" markdown>

<div class="post-hero-text" markdown>

[mibz2 · Research blog](index.md)
{: .post-kicker }

# Halving the single-row steps, stated so it can be answered

Before we attempt a construction, we fix how cost is counted and what counts as a pass or as new so that either answer settles something.
{: .post-dek }

24 November 2026 · 9 min read · [Meta](#6-what-comes-next)
{: .post-meta }

</div>

<div class="post-hero-art"><img src="../assets/images/hero.svg" alt=""></div>

</div>

[Our last post](2026-11-24-halving-the-single-row-steps.md) set a binary target: decode every
block with at most half an event per width-1 row, which no decoder in our current family can do,
so that only a new algorithm could meet it. Before building one, we had to settle what meeting it
would mean, because the target as written turned out to be decided by a bookkeeping convention
rather than by anything in the data. This post restates the problem in terms a machine can be held
to. The yardstick is now real machine cost on a declared model of the machine. The old family's
floor has to be carried into that yardstick by a proof we have not yet written, the *bridge*. And
"new" is judged against the specific method being escaped, with a measurable gap in cost. The
bridge comes first; only then is any construction judged.

## 1. Where the last post left us

The last post's terms are used here without being explained again:

- **The target:** at most h₁/2 events in all, in two passes over the block, with no more memory
  than Open, and faster than Open on whole decoding time
  ([§7](2026-11-24-halving-the-single-row-steps.md#7-where-the-search-stands)).
- **The floor:** G1 proves that every decoder in the current family pays at least one event per
  width-1 row, so anything meeting the target is new by theorem
  ([§3](2026-11-24-halving-the-single-row-steps.md#3-three-routes-back-from-the-bwt)).
- **The way out it left:** a rule applied to a whole scattered set of rows, short to describe and
  cheap to apply, in the form Build, Advance, Emit
  ([§6](2026-11-24-halving-the-single-row-steps.md#6-what-a-new-route-would-have-to-be)).
- **The method:** the [Mario](https://github.com/tra38/sm64-wmotr-abc-proof) move, which supplies the payoff first and looks for the mechanism
  second, and the supplied test that came of it
  ([§2](2026-11-24-halving-the-single-row-steps.md#2-four-reframings-of-the-search)).

## 2. The snag: a convention decided the answer

The decisive gaps in the target were in its accounting. Every known route that beats the floor does
so with a *build pass*: a sweep over all the block's [run records](2026-11-10-rauzy-veech-reconstruction.md) that composes facts in bulk, as
squaring the map or batching the frontier does. The target never said whether such a pass is
charged, and [our record of the question](https://github.com/lmmx/mibz2/blob/8e9b962c5fc94da61e1d47a0868897b7e96ccf0c/docs/journal/monodromy/2026-11-24-charge-convention.md)
found that each reading settles it without looking at a single block:

- **If every record a pass reads is charged,** every exact decoder pays for nearly every width-1
  row, since each such row's byte is recorded nowhere else. The answer is no, by definition, and
  it tells us nothing.
- **If a sweep is free and only row-by-row steps are charged,** the number of passes decides it.
  With passes unlimited, repeated doubling of the map brings the count below h₁/2 with nothing new
  in it. With the two passes the target allows, the best known decoder, squaring the map on Open's
  retained rows, still pays at least 0.60 of h₁, and misses.

So the frozen target's verdict was fixed by two conventions, the charge on a pass and the pass
budget, and not by any fact about the column. A target that a choice of convention can satisfy or
defeat cannot tell a new algorithm from a relabelled old one, which was the whole reason for
setting it.

## 3. Two yardsticks, kept apart

An [outside review](https://github.com/lmmx/mibz2/blob/a12492459dc8e9fb26a8ca4fc0dd36a3392b4769/docs/journal/2026-11-24-algorithm-level/council/chatgpt.md)
of the problem, one of three [compared side by side](https://github.com/lmmx/mibz2/blob/18c74a670b70304c501078b0cef6ed9be706c4f6/docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md),
made the decisive distinction. G1 is proved in bookkeeping units: it counts charges in a calculus
of facts, not instructions, memory traffic or time. Two yardsticks are therefore in play, and
they must not be allowed to blur into one:

| | Bookkeeping (version 1) | Real machine cost (version 2) |
|---|---|---|
| **What it counts** | events: charges per row in the calculus of rigid facts | unpredictable memory reads, the longest chain of them, passes, and peak memory |
| **What is proved in it** | G1: at least h₁ for the current family | nothing yet: the bridge would carry G1 across |
| **The target** | at most h₁/2 events, as frozen | fewer unpredictable reads and scans than one per width-1 row, within a fixed budget |

The original target stays exactly as frozen, as version 1. The machine-cost target is a new
version and is labelled as one, because changing the yardstick without saying so can look like
solving the problem. Louis chose the machine-cost meaning of "lower the floor"
([the decision](https://github.com/lmmx/mibz2/blob/9fb7b227063fe0439aac13b06cbf841bad99b066/channel/coordinator-to-neon.md)),
on a declared machine model, so that a candidate is judged by what it makes the hardware do.

The model separates the two kinds of memory access that matter. A *streamed* access moves a
cursor one step along memory, which the hardware can anticipate. An *addressed* access goes to an
address computed from data, which it cannot. Every run reports four figures, all of them, always
([the formal statement](https://github.com/lmmx/mibz2/blob/303b48c8ff010ba3fd6576d89d6fec2987409f09/docs/journal/monodromy/2026-11-24-council-formulation.md)):

- **Q**, the number of addressed accesses;
- **D**, the longest chain of addressed accesses in which each address depends on what the one
  before it returned, which is what makes the walk slow;
- **P**, the passes: bytes streamed, divided by the block's length;
- **S**, the peak working memory.

The budget fixes the passes at two and the number of live cursors at one per letter of the
alphabet, plus a constant, which is enough to distribute rows by their letter. Holding the budget
fixed is what makes relocation visible: a fall in Q bought with an extra pass, or with more memory,
shows up in the other coordinates instead of disappearing from the count.

<aside class="il-gloss" markdown>**Counting bytes** would have been the obvious yardstick, and the
wrong one. A decoder must write every byte of its output, so a target stated in bytes written is
impossible for a trivial reason: the output alone costs n, and n is never less than h₁. The review
pointed out that trap, and the adopted yardstick counts accesses instead.</aside>

## 4. The bridge: why the floor has to be carried across

In the machine yardstick, no floor of h₁ is yet proved for anything. G1 bounds a sum of charges
in the calculus, so it says nothing directly about Q. To claim that a candidate goes below the
old family's floor in machine cost, we first need a second theorem: that every decoder in the old
family, executed on the declared machine, pays at least a fixed multiple of its bookkeeping charge
in Q. Without it, "below the floor" would compare a machine count with a calculus bound, which is
not a theorem at all.

The bridge also has to come first for a plainer reason. The yardstick must be fixed before any
candidate is judged; otherwise a convenient one can be chosen after the fact. If the bridge
fails for some way of implementing the old family, that implementation is not a counterexample
to G1. It is a candidate escape, and it gets named.

Two more terms needed the same treatment:

- **A pass** had no definition. There was no rule for a partial scan, for two scans merged into
  one, or for a scan that reads values it updated earlier in the same scan. The cursor model now
  supplies one: P counts bytes streamed through cursors, whatever the program calls its loops.
- **New** had been defined by what an object is not ("not rows, not intervals"), which renaming
  can satisfy. It now means new *relative to the method being escaped*, with a measurable gap in
  cost. Rauzy–Veech induction shows why the relativity matters. Against the walk it was a genuinely
  new basis: its cost scales with the number of runs, not of rows, and its records come from one
  pass. Against today's floor it is no escape at all, because G1's family already contains it.
  A definition that asked our own historical step to beat h₁ would be testing the wrong floor.

## 5. The question, stated so it can be answered

With the rules fixed first (the machine model, the yardstick, the inputs and the budget), the
question becomes one of existence. Is there a decoder with a new kind of object, a way to build
that object from the column without the old pointer-following, and a cost law, such that:

1. it is exactly correct on every input, deterministically;
2. it builds its object only from what it is given, within the budget;
3. its cost, Q above all, is low enough on the random-looking content where the width-1 rows
   live;
4. and under a model of time fitted to our machine, it beats the current decoder on whole
   decoding time, judged at one thread and at the default ten separately?

The last condition carries the lesson of the [supplied test](https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060):
handed a free decoding order, the whole decoder took 0.838 of its time on one thread and 0.988 on
ten, two results that differ in kind and cannot be averaged into one.

A negative answer is harder than it sounds. It must rule out every allowed decoder, not the ones
we have tried, and it must fix the budget, because with unlimited passes sorting alone brings Q to
zero. No lower bound of that kind is known for inverting a permutation presented as a BWT; the only
negative result on record is the scoped one, G1, in the bookkeeping yardstick.

<aside class="il-gloss" markdown>**The Mario proof,** on a closer reading, is narrower than our
last post allowed. The [repository](https://github.com/tra38/sm64-wmotr-abc-proof) proves a
conditional intermediate result: under stated assumptions about the game's code and its starting
state, certain sequences of moves cannot make Mario fly. The theorem that the level cannot be
finished without an A press is not yet started. What we borrow from it is the method, a large
claim split into premises that are proved one by one, and not the standing of a finished
impossibility proof.</aside>

## 6. What comes next

The same review found errors in our own records. It listed 21 places where records contradict
one another, have been superseded, or differ in scope. Among them, the draft negative theorem's list of excluded mechanisms omitted one that
its own text calls open. One measurement note compared about forty edges against the number of
width-1 rows in an example with two runs, which cannot be right, since a block never has more
width-1 rows than runs.

The work now goes in a fixed order. First comes the specification: the instruction set, the rule
that turns a program's trace into passes, the cursor discipline, and the read-only, write-once and
working memories. Then the bridge, for the current family, with an explicit constant. Every cost
from then on is reported as (Q, D, P, S) under that specification, with building the object
included. Only after that is a construction attempted, and the supplied test, in the Mario manner,
measures what any candidate's object would buy before anyone pays to build it.

---

**Sources:**

- [The charge convention](https://github.com/lmmx/mibz2/blob/8e9b962c5fc94da61e1d47a0868897b7e96ccf0c/docs/journal/monodromy/2026-11-24-charge-convention.md): why the frozen target's answer is set by the charge on a pass and the pass budget.
- [The problem, stated formally](https://github.com/lmmx/mibz2/blob/303b48c8ff010ba3fd6576d89d6fec2987409f09/docs/journal/monodromy/2026-11-24-council-formulation.md): the machine model, the cost vector (Q, D, P, S) and the conditions for a new basis.
- [The outside review](https://github.com/lmmx/mibz2/blob/a12492459dc8e9fb26a8ca4fc0dd36a3392b4769/docs/journal/2026-11-24-algorithm-level/council/chatgpt.md), recorded verbatim, including its table of contradictions (§5.2) and its reading of the Mario repository.
- [The comparison of the three answers](https://github.com/lmmx/mibz2/blob/18c74a670b70304c501078b0cef6ed9be706c4f6/docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md), with the agreed formulation, and [the decision on the currency and the next steps](https://github.com/lmmx/mibz2/blob/9fb7b227063fe0439aac13b06cbf841bad99b066/channel/coordinator-to-neon.md).
- [The supplied-codes result](https://github.com/lmmx/mibz2/pull/145#issuecomment-7132792060).
