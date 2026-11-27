claim: workstation-pressure
by: claude-code
outcome: done
evidence: #1093 (producer antigravity, rebased onto 72333f9 after #1191); integration review fix in this PR; tests/test_frontier.py 69/69; live brief on origin/master 37dc02bc; origin/evidence bundles jcc-aligned-release/run-20261127T135104Z, scanner-coalesced/run-20261127T082548Z, text-knee/run-20261127T172728Z

**Independent review (integrator ≠ producer).** The change only reads state: no lease, tag, queue, dispatch or claim-file write; declared priorities stay unchanged, physical i9 state is always reported unknown, and every fetch/parse failure fails closed to `unknown`. Authority and gates are untouched, so the protected-path warning needs no owner decision.

**Blocker found and fixed.** Evidence detection read only `commit.txt`/`commits.txt` inside bundles. The newer recipes write `provenance.json` instead (`recipe` for jcc-aligned-release and scanner-coalesced, `head` for text-knee). On live state the PR therefore reported the queue as "replenished" with three runs that already have archived evidence, the exact false negative the claim exists to prevent. Fix: also accept a 40-hex value under the top-level `recipe`, `head` or `commit` keys of `provenance.json`, never `source`, base or tree/blob ids. Malformed JSON is a `ValueError` and still fails closed. Regression test `test_wp_provenance_json_counts_as_archived_evidence` fails without the fix and passes with it.

**After the fix, live brief:** "Repository queue: starved (all 55 ready/* queue entries already have archived evidence)"; pressure active; shortest path lists the review-ready workstation PRs (#1168, #1132, #1117, #1144).

**Residual (for `workstation-runner-telemetry` or a later pass):** recipe bundles have no common provenance schema; a future recipe using another key would again read as un-evidenced (fails toward "replenished", i.e. pressure off, never toward a dispatch). A shared `commit.txt` written by `bench/bundle.sh` would remove the guesswork.
