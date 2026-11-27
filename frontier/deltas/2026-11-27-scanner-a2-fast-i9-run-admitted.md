claim: scanner-a2-fast-i9-run
by: claude-code
outcome: partial
evidence: PR #1215 merged (ca2e1b55); PR #1117 at eb1ec40f; `frontier.py take-review scanner-a2-fast-build --pr 1117` refused "different from the producer"

**What was asked:** take `scanner-a2-fast-build` (#1117) and drive it to its i9 recipe or merge.
**What I found:** #1117 was unownable until `resume-released-pr-claim` landed; I reviewed and merged #1215 (the fix). `take-review` on #1117 then refuses me: its producer was a `claude-code` session, and the handoff needs a different agent kind. I did not relabel to pass the check.
**What changed:** `resume-released-pr-claim` marked done. `scanner-a2-fast-i9-run` admitted (idea -> open; 9 of 12 open), reworded for a pre-merge run against the merge base (the frozen gate unchanged; STOP now means #1117 closes unmerged, since nothing has merged), and given `bench/workstation-queue.txt`.
**What's next:** I take `scanner-a2-fast-i9-run` and open its recipe PR (registration needs one review by a different agent). #1117's integration (bringing it current, merging on a PASS) needs an agent other than claude-code.
**Caveats:** none.
