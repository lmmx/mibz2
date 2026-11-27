claim: none
by: codex
outcome: partial
evidence: #1053 faa9df70 merged 07cf47a4; #1085 02e492a7; recipe review 6084253348; evidence c71c4a56

**What changed:** Admit the P1 queue handoff for A6b. Master at 534bc65 has room under the open-claim ceiling. The source implementation is integrated behind its feature flag. The runnable branch pins the original reviewed source, with the feature off and on. Four synthetic recipe tests and the independent fixture retest passed. The recipe is reviewed at `02e492a7aa9980fa710ae93c1b6e761f2214d18e`; its exact-head CI was still running at admission.
**Outstanding:** `scanner-seam-window-queue` currently holds the shared queue path. Its holder can finish normally. Recheck master, queue history, evidence, ready/hold tags, independent review and the exact recipe checks before acquiring or using this handoff.
**Queue execution:** After acquiring and verifying `a6b-endpoint-queue`, the command for the reviewed head is `gh workflow run run-signal.yml --ref master -f signal=ready -f recipe=bench-a6b-endpoint -f sha=02e492a7aa9980fa710ae93c1b6e761f2214d18e`. Verify with `git ls-remote --tags origin refs/tags/ready/bench-a6b-endpoint`. The queue line is `bench-a6b-endpoint  bench/a6b-endpoint-redesign  Registered A6b redesign of Open endpoint construction (#1053, recipe #1085), 15 paired rounds on Wikidata x4 and text, including the eight-worker ideal-scanner comparison.` An existing result or signal must first be reconciled. No run dispatched by this admission.
