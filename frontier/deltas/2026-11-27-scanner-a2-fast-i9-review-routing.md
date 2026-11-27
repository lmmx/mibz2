claim: none
by: claude-code
outcome: blocked
evidence: `frontier.py take-review scanner-a2-fast-i9-run --pr 1237 --head d7544ed10e4578ef95d1db90440d127126cbafa4 --by claude-code` at ~23:00Z refused ("integration handoff requires a participant different from the producer"); hold/scanner-a2-fast-i9-run f604aaf4 (holder claude-code, released = true, base ac708c52)

**What was asked:** `take one`: the RULES C6 review of #1237 (the i9 adoption registration and recipe gating #1117).
**What I found:**
- **The tool identifies the reviewer by agent kind, not by session.** #1237's producer is recorded as `claude-code`, so every claude-code session counts as the author for C6, even a fresh session with no part in writing the recipe. I did not pass a different `--by` to get around the check. Doing so would be self-review in effect, which is what C6 forbids.
- **Routing:** the C6 review of #1237 needs a non-claude-code participant (antigravity, chatgpt, gemini, ...). Nothing else blocks it: the claim is at `review` with `pr = 1237`, the lease is parked and released, and `take-review` will transfer it without a new slot.
- **For whoever reviews it, from a read I deliberately left unfinished so as not to anchor the review:** the recipe's real-decode cells run Wikidata at 1/8/9/10 workers. The fifteenth synthesis pass notes that the shipped i9 default trims 8 and 10 threads asked to 6 and 7 workers (`trim_cell`, `src/parallel.rs`). Check which plan `scanner_coalesced.measure()` actually runs at those cells, and whether that matters to the registration. The frozen gate is S′ alone, so it bears on how the walls are read, not on the verdict.
- **Nothing else was eligible:** every other awaiting-review claim is under a live antigravity work hold. The only new delta since the eighteenth synthesis pass is `forward-emission-run-queued`, so another synthesis would be busywork.
**What changed:** this delta only.
**What's next:** a non-claude-code agent takes the review of #1237 (`take-review ... --by <its kind>`), then merges, sets `ready/bench-scanner-a2-fast` and adds the queue entry under the standing authorisation.
**Caveats:** a later rule could let a different session of the same kind count as a different agent. That is a rule change (RULES D), not mine to make.
