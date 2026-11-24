# Source: the next step, in plain terms (24 November 2026)

Louis's material for the post after "Halving the single-row steps", recorded verbatim before
any rework (his standing direction: keep the lineage of ideas). It is a Claude session's plain
summary of ChatGPT Pro's council answer, which is recorded in full on
`docs/journal-algorithm-level` at a12492459
(`docs/journal/2026-11-24-algorithm-level/council/chatgpt.md`). The comparison of the three
council answers, and Louis's choice of currency, are at 18c74a670
(`docs/journal/2026-11-24-algorithm-level/council/COMPARISON.md`, journal §33).

Louis's note on it: "(it begins with claude apologising for using shorthand i couldnt understand
to refer to the records etc)"

---

Sorry about that. Here's ChatGPT's answer in plain terms, starting with the background.

## The background

The slowest part of bzip2 decompression is undoing a scramble. The compressor stores the text as a shuffled column of bytes. To recover the text, the classic decoder follows a chain of pointers one row at a time. Each step yields one byte, and each step is an unpredictable jump in memory, which is slow.

**Your faster method.** Where the column has several identical bytes in a row (a "run"), those rows all move together, so a whole run can be handled at once. It's the same move, from points to stretches, that the mathematicians' Rauzy–Veech induction makes. On real data runs are long, so this is a big win.

**Where it stalls.** Some rows are a run of length one: a byte that differs from both its neighbours. These are common in random-looking content such as hex IDs and hashes. I'll call them **lone rows**.

**The proved floor.** We've proved that any method built purely from "this stretch of rows moves together" facts must pay at least one unit per lone row. Your current decoder already pays about that much. So to go faster you need a method of a genuinely different kind, as runs were different from rows. That's the "new basis" the campaign is after. The frozen target asks for at most half a unit per lone row, in at most two passes over the data, with no more memory than today.

**The snag found earlier today.** Whether that target can be met turned out to depend on accounting choices, such as whether a bulk pass over the data counts as cost. It didn't depend on the data.

## What ChatGPT says

**1. The floor is proved in bookkeeping units, not in machine time.** What's proved is that the old family of methods must log at least one bookkeeping charge per lone row. What isn't proved is that each charge corresponds to real cost: instructions, memory traffic, time. To claim "we beat the floor" in real cost, you first need a second theorem, a bridge, that ties the old methods' real cost to their bookkeeping charges. That theorem doesn't exist yet. The yardstick has to be fixed before any candidate is judged. Otherwise someone can choose a convenient yardstick after the fact.

**2. Keep two yardsticks apart.**
- One is the strict bookkeeping rule: any pass that learns something about a row is charged for that row.
- The other is real machine cost: work, memory traffic, passes and memory used.

ChatGPT says to keep the original target exactly as it is. Any target stated in real machine cost is a new version and should be labelled as one. Otherwise, changing the yardstick can look like solving the problem.

**3. "Two passes" isn't defined yet.** There's no rule for counting a partial scan, two scans merged into one, or a scan that reads values it updated earlier in the same scan. The rule has to be fixed before "at most two passes" means anything.

**4. The historical example passes one test and fails the other.** Handling runs instead of rows was a genuinely new basis compared with the row-by-row walk. But judged against today's floor it doesn't escape, because that floor's family already includes runs. So "new" must always mean "new relative to the specific old method being escaped", and it must show a measurable cost gap, not just a new name.

**5. The Mario proof is narrower than we've been treating it.** The repository proves a partial result: under stated assumptions, certain move sequences can't make Mario fly. The full "the level can't be finished" theorem isn't done. What we should borrow is the method: break a big claim into premises and prove each one separately. We shouldn't suggest it's a finished impossibility proof. Neon made the same point on its own.

**6. The shape of the main question.** First fix all the rules: the machine model, the yardstick, the inputs and the budgets. Then ask whether there exists a decoder, with a new kind of object, a way to build that object from the raw column without the old pointer-following, and a cost law, such that:
- the decoder is exactly correct on every input;
- it builds the object only from what it's given;
- it's cheap enough on the hard, random-looking content.

A "no" answer would have to rule out every allowed decoder, not just the ones we've tried.

**7. Errors it found.**
- It lists about twenty places where our own records contradict each other. For example, the target's draft "no" theorem leaves out one of the very mechanisms it says is still open.
- One of Neon's measurement notes compares about 40 against the number of lone rows in an example with only 2 runs. That can't be right, because there can't be more lone rows than runs.
- If cost were counted in bytes written, the target would be impossible for a trivial reason: writing the output already costs one byte per byte of output.

**8. Its recommendations on the choices still open.** Keep both targets, the original and the machine-cost one. Track the full set of costs rather than one number. Keep the decoder deterministic. Let the decoder use output space as scratch if it's accounted for. Judge single-thread gains and 10-thread whole-decoder gains separately.

## What happens next

A reviewer is reading all three answers side by side and checking any disputed points against our records. Then I'll write up one agreed statement of the problem, and bring you only the choices that need your decision, in plain terms. The next pass, which actually attempts a construction, starts from that statement.
