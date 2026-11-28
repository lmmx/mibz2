claim: runs-front-i9-rescue
by: claude-code
outcome: done
evidence: origin/evidence 95edea2a (runs-front-i9/run-20261128T030132Z.tar.gz, sha256 51dd2d6da9589065cf899b5170c72e18e66792a18f0133969289e649a2602b56, equal to the #145 post); run at ready/bench-runs-front-i9 = 18d4d6c4 against BASE 4bb65910; PR #1274

**What was asked:** read the frozen C6 i9 adoption run of #1238's `runs::front_avx2` against registration §4, and act on §5.
**What I found: PASS (rules 1–4 hold), as the recipe scored it and as I re-read it from `summary.json`.**
- **Provenance:** 16 rounds, not smoke, the registered BASE, and a decoder diff of exactly `src/runs.rs` and `src/runs/tests.rs`.
  - The checkout was shallow (`shallow: true`, `ancestry_checked: null`). That is the C6 review's R1 case, so the fix decided whether this run was VOID.
  - Identity: the candidate has 1 `vptest` against the base's 0.
  - Parity: every cell matches libbzip2.
- **Rule 1, one primary gain:** Wikidata 6T at 0.9892 [0.9853, 0.9931].
- **Not significant:** Wikidata 1T 0.9939 [0.9825, 1.0053], text 1T 0.9822 [0.9593, 1.0056], Wikidata 8T 0.9908 [0.9712, 1.0108]. Wikidata 10T 1.0015 and text 8T 1.0002 are flat.
- **Rule 2:** no primary loss beyond noise.
- **Rule 3:** the largest secondary point is `runs` 8T at 1.0165 [0.9966, 1.0368], under the 1.05 allowance. Every guard interval includes 1.
- **Rule 4:** median peak-RSS ratios are 0.9919–1.0012.
- **Size:** the gain is real but small, about 1% at 6 workers. It is below the registration's 0.97 prediction and at the low end of the C6 review's revised 1.7–2.3% at 1T. This matches `front`'s small share of an i9 1T wall of 1.38 s on Wikidata ×4. At 8–10 workers the scanner/worker tie masks it, as DECODER.md predicts.
- **Rule 1's false-pass risk:** the C6 review noted that one significant cell among six can arise by chance. The 6T interval excludes 1 by about 0.7%, and the other primary point estimates all lie at or below about 1.002. C2 forbids re-scoring either way, so PASS stands.
**What changed:** claim `done`; surface row M records the adoption. #1274 merges under its exact-head gates: CI green, CI's paired bench with no primary loss after its one policy re-run, and the B5 and C6 reviews PASS.
**What's next:**
- The default path changes, so a release becomes due (P1) once this merges. RELEASING §3.5's i9 gate applies there.
- The B5 review's follow-ups are open for anyone: an AVX2 Miri step in `miri.yml`, and adopting the fuzz into `src/runs/tests.rs`.
- The `YMM_STEPS` arm (`d5e95794`) stays an untested rescue. It is a distinct mechanism for Intel's single shuffle port, and it is not needed now.
**Caveats:** one machine, one run. The gain at 1T is not established on the i9.
