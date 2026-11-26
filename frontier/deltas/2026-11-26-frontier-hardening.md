claim: none
by: claude-code
outcome: done
evidence: this PR; reviews in docs/journal/2026-11-26-rhizome/06 and 07; live race test on origin (hold/selftest-race)

**What was asked:** harden the frontier after ChatGPT's and Gemini's reviews of #945. In #945, claiming only edited a local file, so two agents could both "hold" one claim.
**What I found:** git gives a real lock. `push --force-with-lease=ref:<expected>` is a compare-and-swap that the server enforces.
- **On GitHub:** two simultaneous `claim`s on one id gave one "acquired" and one "reference already exists". The wrong session was refused for verify and release. Renew worked. A released hold could be re-claimed.
- **Deletions:** this session's proxy refuses branch deletions, so release is a "released" commit, not a delete.
**What changed:**
- holds live on `hold/<id>`; the claim files keep only idea/open/blocked/done/dropped;
- new commands: `verify` (before anything consequential), `renew`, `holds` and `mark`;
- `claim` refuses: not open, needs_louis, a flag, an owned_paths overlap, a full budget;
- the hold stays until the PR merges (completion is not integration);
- five invariants are written down; flags (`frontier/flags/`) hold only their scope; deltas are append-only (check);
- protected paths are never self-merged; ideas wait beyond `max_open_claims`;
- `explore` and `synthesise` are recurring claims, about 1 session in 5;
- PR bodies start `Claim: <id>`;
- the canary cap is `max_live_holds = 3`.
- Also: `mechanisms` (semantic overlap, refused like owned_paths, tested live), the takeover-recovery rule, the governance metric replaced, the three-arm claim scored on ChatGPT's nine dimensions.
- From Antigravity's review (rhizome/08):
  - STATUS.md is local and gitignored (no conflict storm), and #943 publishes it;
  - path matching by segment; the check runs on every PR; blog review by a different agent;
  - a `review` state listed first; recent evidence commits shown in `brief`;
  - `dead-ends.tsv`; mtf split into census and build; i9 dropped from build needs;
  - the ops claims made striated; AGENTS.md and GEMINI.md carry the hard invariants and "only when asked".
**What's next:**
- `stress-test-frontier` (P1) must pass before the cap is raised;
- `migrate-rules` rewrites RULES.md by meaning (it needs Louis to merge);
- `branch-protection` needs Louis.
**Caveats:** the test left debris branches, `hold/selftest-cas` and `hold/selftest-race`; delete them from an environment that allows deletions. The protected list warns but doesn't block; binding protection is the `branch-protection` claim.
