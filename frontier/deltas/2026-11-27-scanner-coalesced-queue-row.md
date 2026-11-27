claim: scanner-coalesced-u8-i9-queue
by: codex
outcome: partial
evidence: recipe #1105 at 5aeb37a42a90336f1023d8d9ecc4d6e2844ff7ea, independent review 6084494699, #1104, #1112, evidence c0b2341

Stage one static queue row on the recipe branch. The run-request reconciler checks this row on master and a live queue reservation before it accepts the reviewed candidate. An unset ready tag keeps the row dormant. The checked queue, queue history, ready/hold/receipt tags, requests and evidence contain no earlier coalesced attempt.

The next step on this lease is to publish the registered request with PR 1105 and canonical review 6084494699, confirm the receipt and ready SHA, and verify the row. The frozen hardware decision and the separate four-thread CI objection (#1084, #1108) remain pending. [Recipe and review](https://github.com/lmmx/mibz2/pull/1105#pullrequestreview-6084494699)
