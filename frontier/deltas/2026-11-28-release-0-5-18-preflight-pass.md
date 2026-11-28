claim: release-0-5-18
by: antigravity
outcome: partial
evidence: origin/evidence c8489a45f5b704e8c8cdc327475a5cb7a2403ad6 (release-preflight/20261128T111128Z.tar.gz, sha256 64a382d293d62e1456648a48d1e0c53b9ebef5f425abf9e83fb009f9c3ad4604); origin/evidence f8b21e4c (recalibrate-0518/run-20261128T100510Z.tar.gz, sha256 51fb951ba23ee9b5d327974ae60d125630ee39a3d80a4b96697af8a495b597f2); workflow_dispatch bench.yml 46131501937 on 7be50513

**What was asked:** read back, independently judge, and record the verified §3.5 physical i9 pre-release regression gate and §5c recalibration receipt for `v0.5.18` (candidate `7be505133b3f5e00e9de983e3177aa619bfb2ccc` vs `v0.5.17` `01acd74d` and `v0.5.16` `4e261140` recovery), and dequeue the two completed workstation entries.

**Pre-release i9 gate receipt (`RELEASING.md` §3.5 & §5c):**
- `preflight_candidate`: `7be505133b3f5e00e9de983e3177aa619bfb2ccc`
- `preflight_base`: `01acd74d17bfea0ab891e75eb4fc2ef3838f5cd8`
- `preflight_evidence_commit`: `c8489a45f5b704e8c8cdc327475a5cb7a2403ad6`
- `preflight_bundle`: `release-preflight/20261128T111128Z.tar.gz`
- `preflight_sha256`: `64a382d293d62e1456648a48d1e0c53b9ebef5f425abf9e83fb009f9c3ad4604`
- `preflight_recalibration`: `completed`
- `preflight_recalibration_commit`: `7be505133b3f5e00e9de983e3177aa619bfb2ccc`
- `preflight_recalibration_evidence`: `f8b21e4c recalibrate-0518/run-20261128T100510Z.tar.gz (sha256 51fb951ba23ee9b5d327974ae60d125630ee39a3d80a4b96697af8a495b597f2)`
- `preflight_recovery_base`: `4e261140986f998a5d0c29122b27f7d106dd23f6`
- `preflight_recovery_cells`: `wikidata:8,wikidata:10`

**Independent judge verdict (`release_preflight.py judge`): VALID PASS (`sha256 64a382d293d6`, 12 paired rounds, 100% byte parity):**
- **Wikidata vs `v0.5.17`:** 1T `1.0073 [0.9628, 1.0538]`, 2T `0.9784 [0.9550, 1.0023]`, 4T `0.9925 [0.9648, 1.0209]`, **8T `0.9668 [0.9597, 0.9739]` (−3.3%)**, **10T `0.9691 [0.9593, 0.9789]` (−3.1%)**, **12T `0.9579 [0.9253, 0.9917]` (−4.2%)**, **16T `0.9514 [0.9206, 0.9831]` (−4.9%)**, **20T `0.9415 [0.9086, 0.9755]` (−5.9%)**.
- **Recovery vs `v0.5.16` (`4e261140`, last known good):** **8T `0.9906 [0.9840, 0.9972]` (yes, −0.9%)**, **10T `0.9869 [0.9772, 0.9966]` (yes, −1.3%)** — full recovery and net faster beyond noise than `v0.5.16`.
- **Text vs `v0.5.17`:** 1T `0.9918`, 2T `0.9965`, 4T `1.0092 [0.9845, 1.0345]`, 8T `0.9822 [0.9597, 1.0051]`, **10T `0.9851 [0.9733, 0.9971]` (−1.5%)**, 12T `0.9680`, 16T `0.9907`, 20T `0.9905`. Zero primary regressions.
- **Secondary guards & memory:** every secondary cell `<= 1.0122` (`records:10` `+1.2%`, `runs:10` `+1.2%`, far below 5% tier); every peak RSS ratio `<= 1.0232` (`+1.0 MB` max, far below 1.10x).

**§5c worker-count recalibration disposition (independently audited from `f8b21e4c`):**
- Pinned family (648/648 rows): strictly valid under `origin/bench/open-port` `two_arm.py` with the recipe's exact arguments; shipped trims hold at 10/8/6/4T (`occ-8-32` @ 10T is −8.9% Wikidata but +6.4% slower text -> `A FINDING, NOT ADOPTED`).
- Unpinned family (1,152/1,152 real rows, `bad = 0`, verified `Cpus_allowed_list: 0-19`, `cpu_quota: null`, `pin.txt` = `7be50513`): strict scorer VOIDed only on field-format mismatch (`cpus: "all"` from master `ab` vs `"0-19"` expected by `open-port`'s `--unpinned`; `#1353` already ported the multi-ref scorer and explicit `--unpinned` CPU list to master for future runs). Labelled diagnostic rewriting `"all"` -> `0-19` rejects 0/25 Holm cells (nearest two exceed the 1.10x RSS gate). Shipped trims stay unchanged; deviation recorded transparently here and for the `v0.5.18` CHANGELOG.

**§3.3 research feature retention review (`Cargo.toml` `[features]`):**
- Reviewed all 14 research features. Active or still referenced by open claims/recipes: `wait-stats`, `walk-stats`, `dose`, `phase-counters`, `induction-trace`, `grammar-replay`, `owned-writer`, `moonshot`, `census`, `ideal-construction`, `ideal-mtf`, `ideal-rle1`, `ideal-delivery`, `ideal-parts`, `mtf-fold`, `scan-oracle`.
- Lapsed candidates for separate follow-up removal PRs (never removed in the release commit, per §3.3): `a6b-endpoint-redesign` (i9 STOPped; replaced by shipped `#1294`) and `forward-anchor-lanes` (adopted into default in `#1312`).

**What changed:** added this receipt delta and removed completed `bench-recalibrate-0518` and `bench-release-preflight` from `bench/workstation-queue.txt` (both non-performance paths; zero staling of `7be50513`).

**What's next:** merge this receipt PR, re-confirm `release_preflight.py verify` passes on `master`, then dispatch `Deploy Release` (`deploy.yml`, patch bump `0.5.18`), verify crates.io/PyPI/docs.rs, queue `bench-release-scaling` & `bench-crossover`, and publish the `v0.5.18` CHANGELOG entry.
