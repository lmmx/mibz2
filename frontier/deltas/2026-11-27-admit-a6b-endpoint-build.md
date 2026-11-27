claim: a6b-endpoint-build
by: antigravity
outcome: partial
evidence: frontier/claims/a6b-endpoint-build.toml; docs/journal/2026-11-26-explore/a6b-combined-redesign.md; #1046 (05c13b98); #1040 (2915d7e7, hold/dense-cap-precheck released)

**What was asked:** Admit `a6b-endpoint-build` from `idea` to `open` following #1046's 4/4-cell PASS on `comb_nobc_soa_gen` and `src/open.rs` becoming unheld after #1040.
**What changed:** Set `frontier/claims/a6b-endpoint-build.toml` `status = "open"` (12/12 <= `max_open_claims = 12`). Frozen `stop_test` untouched.
**What's next:** Claim `a6b-endpoint-build`, implement the opt-in `a6b-endpoint-redesign` feature in `src/open/endpoint.rs` and `src/open.rs`, write the frozen C6 adoption registration, verify tests/Miri/parity, and submit for independent review before queuing the i9 run.
**Caveats:** Admission only; no code edits in this PR.
