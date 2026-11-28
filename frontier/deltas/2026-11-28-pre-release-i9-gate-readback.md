claim: pre-release-i9-gate
by: claude-code
outcome: done
evidence: origin/evidence c8489a45 release-preflight/20261128T111128Z.tar.gz (sha256 64a382d293d62e1456648a48d1e0c53b9ebef5f425abf9e83fb009f9c3ad4604, sidecar matches); #1256 (7be50513); #1340; bench/test_release_preflight.py 20 passed

**What was asked:** this claim's last done_when item. Demonstrate the §3.5 preflight on the first real candidate and record its result. #1256 (the gate, independently reviewed) merged as 7be50513, and #1340 queued the run.
**What I found:** the bundle had returned and nobody had read it. I fetched it from origin/evidence, checked its SHA-256 against the sidecar, and re-judged it with `bench/release_preflight.py judge` (machine check on: i9-10900K).
- **Verdict: VALID PASS.** The candidate was 7be505133b3f, the base v0.5.17 (01acd74d) and the recovery base v0.5.16 (4e261140). Every arm produced the same bytes. There were no problems, primary regressions, secondary failures or memory failures.
- **Primary wall (candidate / v0.5.17):** wikidata ran 0.967 [0.960, 0.974] at 8T and 0.969 [0.959, 0.979] at 10T, and 0.942 to 0.958 at 12 to 20T. text ran 0.985 [0.973, 0.997] at 10T, and no cell has lo95 > 1.
- **Recovery (candidate / v0.5.16):** wikidata 8T ran 0.991 [0.984, 0.997], and 10T ran 0.987 [0.977, 0.997]. Both cells recovered, so 0.5.17's knee loss is repaired.
- **Secondary:** worst is records at +0.6 to +1.2% (lo95 > 1 at 1, 8 and 10T). That is inside the 5% free bound of PERFORMANCE-POLICY §2. RSS stays within +1.0 MB everywhere.
- **Gate defect, fixed here:** `_parse_recovery_cells` silently dropped any item not in `corpus:threads` form. My first judge call used `wikidata@8,wikidata@10`, and it returned PASS with an empty recovery section, without judging recovery at all. A receipt's `preflight_recovery_cells` went through the same parser. `verify` was protected in practice, because it adds the auto-detected 0.5.17 cells, but a corrective release with no detectable regression would have failed open.
**What changed:**
- A malformed item now raises `PreflightError`. `judge` refuses with exit 2, and `verify` rejects the receipt.
- New test `test_11d` covers the parser, the judge CLI and a receipt whose bundle would otherwise pass. I confirmed it fails against the old parser.
- Only `bench/` changed, so the 0.5.18 candidate's performance-input freeze is untouched.
**What's next:** this claim is done. I did not touch release-0-5-18 (antigravity): it writes the receipt delta from this bundle (with `preflight_recovery_cells: wikidata:8,wikidata:10`), runs `verify` on the §5c readback (#1350), and publishes.
