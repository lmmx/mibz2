claim: migrate-rules
by: claude-code
outcome: partial
evidence: PR #1065 head ec7bbc09bcd5 (base da48997293; master 37dc02bc5b has no RULES.md change since); frontier/README.md; CLAUDE.md

**What was asked:** take one: finish integration of `migrate-rules` (#1065), merging what is meaning-preserving and sending authority changes to `needs_louis`.
**What I found:**
- **Nothing in #1065 is mine to merge.** The claim's frozen `stop_test` reserves the merge to Louis (RULES D1: amending these rules); `done_when` is "reviewed by a different agent, awaiting Louis". `take-review` refuses ("requires an owner decision": the PR's claim card carries `needs_louis`) and `claim` refuses (pending PR), so I worked read-only; no hold taken. The only purely mechanical parts (archive/ link repairs, tense) are not worth a separate RULES.md PR racing Louis's decision.
- **The gap was visibility:** `needs_louis` lived only on the PR branch, so master's claim was `open` with no owner item and #944 never showed it. This PR puts it on master.
- **Cold review, row by row (original text vs #1065):** faithful to the original or to already-adopted frontier/CLAUDE.md wording: header and history; A1 (flag resolved by a different agent, as frontier/README.md); A3; A4 (owner session for "Coordinator's session"); A5; A6 (= O1); B2-B4, B7; C1a (= CLAUDE.md "in its claim or a registration file"); C2; C3 (rescue text = CLAUDE.md 27 Nov); C4; C6 (adoption review by a different agent = CLAUDE.md); D1-D8 lists; E1-E4; S; M; F; archive/ links.
- **Six drifts that change meaning** (fix before approval):
  1. O3 drops "only": "binds only under B1, B5, B6 and B7" becomes "binds under …", widening binding objections.
  2. B5 drops "Nobody's approval is a precondition", turning a hold right into a possible approval gate.
  3. C1 drops "Registration is permission to experiment", removing an authority grant.
  4. C5 adds "any agent holding a striated claim on `bench/workstation-queue.txt` may queue"; CLAUDE.md's standing authority (9 Nov) is any agent, for any frozen, reviewed, green recipe.
  5. D's preamble makes a verbatim decisions.md record a condition of Louis's decision counting; C5 makes his running the recipe the authorisation. Keep recording as A4's duty, not a validity test.
  6. D's B5-objection clause: resolution moves from "the reviewer accepts … or withdraws" to "a different agent from the implementer". Consistent with frontier/README.md's flag rule but weakens a safety gate: a third agent could override the objector.
- Also substantive but plausibly intended: A2's performance-HOLD lapse now needs "a different agent" to concur (was the Coordinator); B6 now needs a different agent to resolve the stop (stricter).
**What changed:** `frontier/claims/migrate-rules.toml` on master: `status = "review"`, `pr = 1065`, `needs_louis` with the decision and recommendation; this delta.
**What's next:** Louis decides (Needs Louis / #944). If approved with amendments, whoever holds the claim fixes drifts 1-5 on `claim/migrate-rules`, rebases on master (this PR's claim-file lines will conflict; take master's `needs_louis` cleared on approval) and merges with his words in docs/journal/decisions.md.
**Caveats:** review is of #1065's text only; OWNERSHIP.md's stale COORDINATOR-FAILOVER.md link (noted by the author) is still open and owner-only.
