# Oracle précis: stop at a supported claim, not an artifact
Researcher, 2026-11-06. [Assignment 6684137403](https://github.com/lmmx/mibz2/pull/162#issuecomment-6684137403).

## Recommendation
Keep discrete versions and the short L0. Add three narrow qualifications: a demonstration supports a **particular claim at a particular revision**; settlement records the channel watermark **and the evidence actually read**; L0 must acknowledge materially parallel directions rather than imply a single team-wide step. No workflow inventory, continuous polling, deeper fixed hierarchy or new approval gate is needed.

The reviewed pins are [SPEC.ORACLE at 604ea822](https://github.com/lmmx/mibz2/blob/604ea822ba3f65bf964dfddec542941fe7af03fb/docs/journal/compact30-channel/SPEC.ORACLE.md) and [seed at c42c3fa7](https://github.com/lmmx/mibz2/blob/c42c3fa78c0634fc478d5b47cf578a35049974f1/docs/journal/oracle/intention.md). This is advice on those versions, not an assertion about subsequent edits. Governing Researcher/common spec pin remains 0479b380f5d6309f3fd0f23e0ad6dfade4b7a5c0. Read current constructor-branch ROLES §3. I contributed the earlier purpose/premise/dispatch distinctions; this is not a blind review.

## 1. The concrete failure is already in the seed
The seed is stamped settled at #145 comment **6684041836**. That comment expressly records acceptance of the counterexample and **retraction** of the threshold in [6683965024](https://github.com/lmmx/mibz2/pull/145#issuecomment-6683965024). The latter says the >=25% touched-T reopening requirement is retracted; the dispatch pause until #220 remains; footprint research is to be priced on effective reduction and recoverable time without a fixed threshold.

Yet seed 1.2 still says the decision stands and the premise is open. That wording leaves the rejected reopening condition apparently operative. This is not simply a reasonable later event that the snapshot has yet to cover: the purported cutoff already includes the correction. Its status should be **threshold retracted; dispatch pause retained**, linked to 6683965024. The scientific counterexample remains historical support.

This does not require another scientific judgment or another level of detail. It requires checking that the sentence selected for compression follows the latest relevant decision inside the stated coverage. A source link can demonstrate the old decision perfectly while failing to support its present applicability.

## 2. Stopping at a demonstration
The reasonable-misreading exception already covers directional divergence in principle. Clarify its application rather than mandating children below every PR:

| Counterexample | What the artifact actually demonstrates | Minimum sufficient display/check |
|---|---|---|
| PR originally serves speed; a later revision knowingly trades speed for memory | An original registered purpose and a changed implementation, not automatically an authorized new purpose | Check the relevant revision and stated purpose together; flag mismatch or unstated change. Do not infer a new intention from code |
| A dispatched brief remains linked after execution stops | Dispatch occurred; it is not perpetual evidence of a live run | Use a current terminal/checkpoint record when available. Otherwise say dispatched, execution status unconfirmed; age alone does not establish death |
| Two attempts pursue incompatible mechanisms or trade-offs | Both belong to one PR/front, but the competition may be the current direction | One aggregate line identifying competing directions and the unresolved choice; link details. No list of process IDs or every attempt |
| Parent purpose changes after an old result | The old result may still be valid, but may no longer answer the new question | Recheck applicability to the changed purpose/premise; retain valid evidence instead of imposing a date-based expiry |
| Child changes or fails while parent text remains identical | Parent-age comparison detects nothing | Check relevant child/source changes affecting the asserted status or direction, not just parent edits |

**Present purpose does not fix a numeric depth.** One front may be demonstrated by a registration; another may need one more discriminating sentence. Stop when the relevant alternative interpretation is resolved or explicitly marked unresolved, not when any recognized artifact type appears.

Suggested minimal rule:
> A demonstration supports the particular purpose, status or result claimed at its cited revision. Recheck its applicability after a relevant change to that item, its purpose/premise, or supporting work. If the evidence cannot establish current execution, state the last confirmed state and the uncertainty. Expand only enough to expose a material divergence.

This is change-sensitive, not an arbitrary time-to-live. A months-old registration can remain binding, while a minute-old brief can already be superseded. During each authorized rebuild, inspect the sources necessary to support changed or still-active claims. Do not introduce background monitoring or demand agent heartbeats. No observer can detect work that leaves no accessible record; admit that limit.

### Austin: the relevant alternative, not every conceivable alternative
The primary text actually says, “It does not mean, for example, enough to show it isn’t a stuffed goldfinch.” See [Other Minds, printed p.84](https://web.stanford.edu/~paulsko/papers/Austin_OM.pdf#page=29). The assignment's stuffed-goldfinch paraphrase reverses that local example. His contextual standard still supports the useful principle here: identify a definite relevant shortcoming rather than require exhaustive assurance. A superseded decision or divergent active purpose is relevant to Louis's directional judgment; process trivia normally is not. These engineering rules are my application, not a claim that Austin specified snapshot semantics.

## 3. Discrete settling: watermark plus evidence set
A #145 comment is a **channel coverage watermark**, not a version of every branch, PR or run. Replacing it with a set of branch heads alone is also insufficient: comments, PR state and external run status can change without Git commits. Sequentially reading several heads does not create an atomic snapshot.

Concrete counterexample: read a branch's old status, then read a newer channel decision that depends on a new branch commit. A page mixing those observations may describe a state that never existed. Another case is an unannounced branch commit after the watermark: either the page uses it and is not strictly as-of the comment, or excludes it and cannot claim to describe all currently visible work.

Keep last_rebuilt_from for channel coverage and queue progress. Add a small evidence manifest, preferably behind the page:
- each relevant repository/ref and exact commit/file revision actually used;
- comment IDs and observed mutable status needed for claims, with observation time (preserve body/version or digest when edits matter);
- material unavailable or inconsistent sources.

Use wording such as **“Channel covered through C; evidence inspected at [time], pins linked; later changes not reflected.”** Do not call the set globally simultaneous. Read cross-references at the named pins: a decision dependent on a commit must not be paired with an older file. If the relevant source changes during collection, refresh the affected claim once or mark that item unsettled; do not restart indefinitely or block all publication.

An unannounced commit can be evidence of work done. It cannot silently authorize a changed intention or dispatch order. Label that distinction. A narrow unresolved item can coexist with a settled, useful page.

The seed's actual retraction error requires neither a new capture nor more heads: it is a reconciliation failure within its own stated channel coverage. The evidence manifest helps reproduction, but does not substitute for reconciliation.

## 4. L0 with parallel fronts
One line is a compression constraint, not proof that the team has one current step. The present L0 names only the #220 wait. L1 does name the open downstream bump, so the whole page is not concealing it; nevertheless a reader taking L0 as the shortest **team-wide** answer receives an incomplete account.

Keep one line if it can mention the distinct states faithfully. For this seed, for example:

> Awaiting the i9 test of three-table scaling, with the downstream 0.2.4 update open alongside, to improve and deliver decoding performance.

This reports an open update, not verified active execution. It avoids pretending that every front serves the same immediate experiment. If several material fronts cannot fit, one line per front is preferable to silently selecting a main direction that nobody authorized. Alternatively label the single line “Performance front” so its scope is honest, and explicitly point to the parallel fronts. Do not invent an overarching immediate step or duplicate the entire why chain.

At higher levels, carry the direction-changing exception: a disputed premise, retracted constraint, competing approach or independent blocked front must survive compression if omitting it changes what Louis would reasonably intervene on. It may be enough to say “two alternatives under review,” with their evidence one level below.

## 5. A minimal acceptance test and novel consequence
Consider two underlying records: both link the same original brief; in one it is still the live plan, in the other its threshold has been retracted. If the précis renders them identically while that difference changes which work Louis may direct, it has stopped too early or retained a stale predicate. Conversely, changing a worker ID without changing purpose, state or choice need not change the précis.

This is a **direction-preserving compression test**, not a requirement that Oracle choose the right direction. Applied to the three examples, the required discriminators are applicability, execution confidence and competing purposes—not another fixed level.

Thus “settled” should mean **reconciled against the declared evidence set**, not “everything finished” or “the world frozen at comment C.” This permits short, discrete updates without either false liveness or a full task inventory.

## Forwardable correction
ORACLE: 1.2 / settled-at stamp — the seed says the 25% decision stands, but the covered record retracts that reopening threshold and retains only the dispatch pause. Record the threshold as retracted — Coordinator 6683965024, reaffirmed by cutoff 6684041836; seed c42c3fa7.

The other recommendations are scoped specification advice for the requested review, not repeated corrections to already closed findings. No code, Oracle page, spec, dispatch, registration, task or subscription changed. Research only on the Researcher branch; one REVIEW on #145.
