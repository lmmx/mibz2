claim: a6b-endpoint-run
by: codex
outcome: partial
evidence: b87790e5 (unscored A6b receipt); #1122; original recipe 02e492a7; source faa9df70; registration blob 16747e45

**Repair:** Initialise the private build repository, fetch the frozen candidate SHA from the runner's origin directly into that repository, verify FETCH_HEAD, and check it out detached. The previous fetch into the runner followed by a shallow local clone lost the fetched-only object. Candidate and control build the same source and differ only by the original feature selection. The registration, scorer, grid and queue entry are unchanged.
**Verification:** The old sequence failed on a detached, depth-one file:// fixture with a separate candidate branch. The repair materialised its exact source and preserved the runner checkout. All five recipe tests pass. A fresh depth-one checkout of the real 02e492a7 recipe also lacked the candidate; the repaired path fetched faa9df70 and read the frozen registration blob 16747e45f6bd94e923688cfef93c87b336e86319. No timing was collected.
**Queue:** The prior ready SHA 02e492a7 has the immutable INCOMPLETE receipt b87790e5. A newly reviewed green recipe SHA can provide the first measured attempt. Preserve the old receipt, confirm no intervening result, and move the ready signal only under the queue worker's verified authority.
