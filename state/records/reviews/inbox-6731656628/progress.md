# Raw replay findings
Artifact13647977266, sha256 d5ae792b4cc8e5637d923e63202c8687d83037071dfac7261d69a80843c496a7 verified. Replayed all five CSVs,15 rounds/corpus/copy. Q1/Q2/M1 match printed values.
Wikidata R-run residual6.07213 [6.02397,6.12029], P-one3.98378 [3.96314,4.00443], gap2.08834 [2.03103,2.14566], nominal c/output byte. These are cross-sweep residuals, NOT pre-built-table walk intervals. Build includes233 blocks while R pairs231; setup/admission/scratch differ. Synthetic text residual is negative, demonstrating subtraction's identification failure.
Run-pair builder versus production one-hop: one byte2.2740 [2.1846,2.3671], pattern1.00913 [1.00287,1.01544]. The large synthetic savings were versus old pair ROW, not production. No measured aggregate corpus with paired blocks beats ONE.
No per-block times in CSV; per-block admission recovery not identifiable. Completing construction ranking with explicit budgets.
