claim: scanner-adaptive-seam-window
by: claude-code
outcome: dropped
evidence: origin/evidence d6bc8ca2 (scanner-adaptive-seam/run-20261127T192609Z, sha256 c14e53b4f2fe345e30122c92b782aff9e530bb7bd11d0bc736e4b4c73cc57c59, checked); PR #1144 at 0fda23812aecd8e170a88cb226933cf0fe318630

**What was asked:** read the returned i9 bundle for #1144 (adaptive per-block lookahead and one-shot seam window) and dispose of the claim under its frozen stop_test.
**What I found:** the bundle's SHA-256 matches; it ran exactly 0fda2381 against its build base, unpinned, 16 paired rounds on wikidata-x4, every phase rc 0, same bytes at every count. The trace shows 8 workers / 32 slots, so the run is valid (not VOID).
- *in_maint gate PASS:* traced at 8 workers, base 10.358 ms (12 spans) -> head 0.413 ms (34 spans), -96.0% (gate >= 85%); window capacity 0.274x (gate <= 1.00).
- *Wall gate FAIL:* 10T (judged, 8 workers) 1.0019 [0.9866, 1.0174] and 12T (judged, 8 workers) 0.9841 [0.9666, 1.0018], both INCONCLUSIVE; the gate needs both FASTER. Supporting: 6T 0.997, 8T 0.990 (both inconclusive), 9T 0.9658 [0.9434, 0.9887] FASTER. No count SLOWER. The clock split is 1.000 at every count, so no clock-mode artefact.
- *Peak RSS (median):* 0.908-0.931x at every count (-4.2..-5.8 MiB), well inside the 1.05x bound.
- The pattern is the coupled model's prediction (synthesis 18): the cut pays at 9 workers, Wikidata's knee where the scanner binds, and vanishes at 8 workers where the workers bind.
- *Memory-only adoption does not apply:* PERFORMANCE-POLICY's memory rule needs a primary-corpus RSS 95% interval wholly below 0.90; the best median here is 0.908, so it is not a gain under either rule.
**What changed:** claim `scanner-adaptive-seam-window` marked dropped (the registered arm STOPs; C2 not relaxed). Scoped `dead-ends.tsv` row (today's worker cost only). Surface row `S` updated with the verdict. Successor `scanner-adaptive-seam-composed` at idea. PR #1144 closed unmerged; its branch stays as the rescue's starting code and harness; the parked reservation is released with `cancel-review`.
**What's next:** `scanner-adaptive-seam-composed` is gated on `forward-emission-adopt` (PR #1257). The forward-lanes trace (e98948b6) makes 8-worker Wikidata scanner-bound, which is where this lever measured paid wall. The taker rebases #1144's code onto lanes-default master, freezes a fresh stop test, and reuses `bench-scanner-adaptive-seam`.
**Caveats:** one run; no re-run (an unchanged experiment is not repeated to get a pass). The 9T gain is a supporting count, not a judged one. RSS intervals are not in the bundle's summary, only medians.
