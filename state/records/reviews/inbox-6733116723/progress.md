# Progress: inbox6733116723

- Downloaded artifacts13651923399 and13645275344; SHA256 matches00b0e008bdd8a90108ccaa87e21467dd35d70df35e1b71b86e687078a64a3e94 and9be443df35b8aa51c08c4b9491557b0d077cb6b78a11167625e2f7195e88d539.
- Pinned reporters reproduce D/A1.052451, D/B1.078033, B/A.976270 and oldB/A1.159179. Raw arithmetic replay, not full independent validity/corpus reconstruction.
- Both measured heads have identical src/mtf.rs2610fa7e and src/bwt.rscb971c9f blobs. B's original src/run_stage.rs runtime prefix is unchanged; D is appended. Both declared master baselines likewise have identical mtf/bwt blobs to each other. Compiler logs match rustc1.99.1. Harness arms and native layout remain confounded with host.
- New B/A synthetic random1.2842, records1.3749, runs1.3831; Wikidata fallback1.02107. B is not a production-qualified win and does not leapfrog C29.
- Direct-placement loss is not decomposed into seeds, cursor dependencies, write locality and loop/check overhead. Equal stored slots do not identify a unique cause.
- Developing C43 stable u32 run-index buckets: two grouping passes plus sequential global fill, half-size ordering metadata, paying dependent record gathers. No constructor implemented.
