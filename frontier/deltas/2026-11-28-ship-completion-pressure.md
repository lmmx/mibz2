claim: none
by: chatgpt
outcome: partial
evidence: board #943 at 07ab2e7; #1256; #1241; #1303; frontier/tools/frontier.py release_due and release_pipeline_state; docs/RELEASING.md

**What was asked:** releases must receive reliable priority through stigmergy, without repeated owner commands.

**What I found:** release_due suppresses the signal when a nonterminal release claim exists; release_pipeline_state marks an untagged existing release derived_p1=False. The board says Release due: nothing despite a blocked 0.5.18, accepted default changes, unreviewed #1256 and a starved repository queue. The release was not merely waiting for a new algorithm.

**What changed:** runbook §2a and the shared agent loop require persistent release-critical pressure, explicit dependencies, resume on evidence, stable release inputs and priority in the existing queue. Reopened the existing 0.5.18 preparation claim with concrete next steps; publication remains gated. Added P1 ship-completion-pressure for the small tested extension to existing frontier tooling.

**What's next:** complete #1256 review, prepare and run the applicable current-candidate §5c recalibration, freeze the candidate and run final paired CI/i9 preflight (including recovery versus 0.5.16), read the result, publish on pass and record scaling. Use a reviewed equivalent harness rather than await helper tooling alone. No need to wait for #1324's optional adoption. Respect existing holds including #1282 on src/parallel.rs and coordinate tooling with #1303.

**Caveats:** no code changes to the frontier projector, hardware dispatch, or publication in this contribution. Normal integration gates apply; no frozen threshold, spending authority or safety gate changed.
