# Simulated first-time reader, round 1

The post at master `ba52e4d6`, read once, top to bottom, by a fresh reader given only the
post and this profile: a working programmer who knows caches, pointers and lookup tables,
and not bzip2's internals, the BWT, LF, Lyndon words, grammars or this project's vocabulary.
Its report, condensed; quotations are the post's words.

## Where to start
At the top, with one detour: opening paragraphs 1 to 3, then §1, then back to the opening.
"Working on intervals" did not land until the `aab` figure had been seen.

## Not followed in the opening
- "the last interval of the runs' order in the last column with the last interval of their
  order after LF": two orders could not be pictured.
- "gives the shorter interval a longer word": nothing had said intervals carry words.
- "`X → AB`": are A and B runs or words?
- "After a bug fix": what bug, and how bad?
- "the older column-based reference": undefined there.
- "No order of these steps can take fewer than 1.220": no reason given.
- Why it matters that the tree is a Lyndon tree.

## The goal questions
1. Why can rules replace the walk? Answered; confidence medium ("I can replay `aab` but not
   say why it is right in general").
2. What improved, and against what? Answered; high on the headline, medium on what each
   side's time includes.
3. What is a Lyndon tree, and what does the identity say? Answered; medium.
4. Why are fewer steps not less time? Answered; high. The reader did the multiplication the
   post leaves out (steps a run × instructions a step).
5. What stands before deployment? Answered; high.

## Stuck points
| level | where | what |
|---|---|---|
| step | §1 ¶1 | "Matching equal letters in occurrence order gives LF mappings": worked out on paper |
| step | §1 ¶3 | why this order of concatenation, and why the shorter interval gets the word |
| chain | §1 ¶4 | "keep only the rows where the path returns": returns to what? |
| symbol | §1, §4 caption | "the two rules of §1": §1 never writes its steps as rules |
| chain | opening, §1 | "One step covers many rows": in the example each step covers one row |
| statement | §2 | "Everything below was timed before the fix": are those numbers still valid? |
| statement | §2 ¶1 | does 2.049 include making the records, on either side? |
| symbol | §2 caption | "no confidence intervals; not CI": two meanings of CI |
| step | §3 | "rows that read the same pair of run letters in succession" |
| symbol | §4 | "expanded derivation tree" |
| step | §4 | "ranks of rotations", then a cited result about "suffix ranks" |
| chain | §4 | why the smallest rotation, and not the block |
| beyond | §4 | the Cartesian-tree example: ranks 0, 1, 2 are already sorted, so it shows nothing |
| symbol | §5 caption | r₂ − 1 before r₂ is defined |
| step | §5 | "A digram's occurrences fill one cell": read three times |
| symbol | §5 | "segments" |
| statement | §5 | "We have since counted a worklist": counted or timed, slower by how much? |
| statement | §6 | "All but the referee's reading have reported": reported what? |

## Numbers that confused
- Three ratios for the kernel against a reference: 0.818, 0.738 and an implied 0.96. The
  headline is after the fix and the whole table before it.
- "higher than the 0.738 in the table below": a table not yet seen.
- "moved from 1.604 to 1.677, partly from the fix": partly what else?
- Denominators move between a byte, a row, a run and a step.
- "233 blocks": of how many? "126,658 bytes": of how many?
- "one cloud machine" in the opening; "Local" in the caption.
- `ab` repeated is timed in §2; §4 says `abab` "needs separate treatment".
- "53–60% of a decode" gives no way to turn 0.818 into a whole-decoder expectation.

## Hints where a fact was wanted
"another operation, outside it" (§5); "a different reduction" (§6); "needs separate
treatment" (§4); "from another kernel" (§2); "records where later work goes" (§3); "uses more
memory than the table" (§3); "finds enough of them" (§5); "partly from the fix" (§2); review
parts that "have reported" (§6).

## What worked
Bold definitions at first use; candour about its own table; `aab` carried through §1, §4
and §5.
