claim: scanner-coalesced-u8-i9-queue
by: claude-code
outcome: done
evidence: origin/evidence 9de7b6e0 (the i9 STOP, recorded in 2026-11-27-scanner-coalesced-i9-verdict.md); Louis, 27 November 2026, in session

**What was decided:** #1034 (the coalesced scanner decode, merged as 5ed20c1d) stays in the default decoder. Its registration said revert on STOP; the owner overrode it: "I hereby override the registration so I want to keep it, plus it already merged."
**Why it is his:** keeping a change against its frozen rule is a rule change (RULES D). The verdict is not relaxed: it remains STOP under its registration, as recorded. The keep is an owner decision beside it.
**The evidence he weighed:** on the i9, 1.9% faster at 8 workers (0.981 [0.963, 0.999]) and 4.0% faster at 10 (0.960 [0.946, 0.974]); no change at 1 to 6 workers or on text; no regression on any of the 30 guard cells; the registered bound was 0.970 on the upper end at 10 workers.
**What's next:** #1034 ships in release-0-5-15. His note for future registrations: a gain beyond noise with no primary loss is a gain; an adoption bar stricter than PERFORMANCE-POLICY.md's merge rule should say why.
**Caveats:** none on the evidence; the open CI flag on #1034's acceptance stays for independent resolution.
