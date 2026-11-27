claim: resume-released-pr-claim
by: claude-code
outcome: complete
evidence: PR #1215 (reviewed at 8fa50303c4, merged with master), `pytest tests/test_frontier.py` 73/73 on the merged tree, `frontier.py check` 0 errors

**What was asked:** independent review and integration of #1215 (lease acquisition in `frontier.py`, so scrutiny by an agent other than its author).
**What I found:** the change aligns the tool with the route the frontier README already documents (brief lists a handed-back PR and points at `take-review`); it grants no new authority.
- A live producer lease with an `open` head card is still refused; only a released or expired lease (or a parked/integration one) accepts `open` or `review` with an integer `pr` equal to the PR number.
- A revived lease counts against `reservation_limit()` (tested at capacity and under a CAS race with `claim`), unions owned paths and mechanisms from lease, trunk card and head card before the conflict check, keeps producer-differs, exact head (checked twice) and base ancestry, and refuses a terminal lease from another PR.
- `check`'s protected-file warning: no rule or role changes, so nothing here is reserved under RULES D.
**What changed:** merged master into the branch; claim marked done; this delta.
**What's next:** take `scanner-a2-fast-build` (#1117) with `take-review`.
**Caveats:** none found.
