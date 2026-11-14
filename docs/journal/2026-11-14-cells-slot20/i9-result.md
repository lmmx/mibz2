# The 20-byte slot on the i9: MOVES THE PLATEAU

The `bench-cells-diet` run registered in #145 LOG 6881022819 (amended 6881074255), frozen at master 52e539345658d9f8e9dad915594f9f2dbb3f6357 (LOG 6882968471). Verdict LOG: #145 6883380160.

- **Bundle:** `cells-diet/run-20261114T195323Z.tar.gz` on the `evidence` branch at 88f2428cbc2630c4ea124254b0f66f5fc59ab82b, sha256 `52275b76ee0937eb20cd006e544ef87f2c5d654afb3088b4c15e6598e63c9374`, checked.
- **Forecast before the data** (Optimiser, #145 6881070503): Q 0.96 (0.91 to 1.01), 4-thread wall 0.97 (0.94 to 1.02), 1-thread wall 0.99 (0.97 to 1.02).
- **Decision:** diet step 2 (the 16-byte slot) is built, with its own scaling question Q2 = A16(10)/A20(10) registered in the same LOG.

The analysis as the recipe wrote it (`diet.md`), verbatim:

    # The Cells buffer diet's scaling question (#145, LOG 6881022819, amended 6881074255)
    
    - runs: /home/louis/dev/mibz2/target/cells-diet/run-20261114T195323Z/runs.jsonl (80 records)
    - grid: threads 1,4,6,10, 10 rounds, wikidata
    - A20 (20 B slot): `mibz2-workers@/home/louis/dev/mibz2/target/cells-diet/target-s20/release/examples/ab`
    - A24 (24 B slot): `mibz2-workers@/home/louis/dev/mibz2/target/cells-diet/target-s24/release/examples/ab`
    
    ## Validity: passed
    
    a complete roster, no duplicates, every child successful, one output hash, Cells engaged on all 196 blocks in both builds (dose_blocks, dose 0 MB)
    
    ## Reported, not gating
    
    | t | wall A20 s | wall A24 s | wall 20/24 [95% CI] | CPU 20/24 [95% CI] | A20(t) = CPU(t)/CPU(1) | A24(t) |
    |---|---|---|---|---|---|---|
    | 1 | 0.4043 [0.3878, 0.4216] | 0.4125 [0.3979, 0.4276] | 0.9803 [0.9207, 1.0437] | 0.9805 [0.9211, 1.0438] | 1.0000 [1.0000, 1.0000] | 1.0000 [1.0000, 1.0000] |
    | 4 (cap) | 0.1862 [0.1841, 0.1883] | 0.1945 [0.1925, 0.1965] | 0.9574 [0.9453, 0.9695] | 0.9531 [0.9439, 0.9625] | 1.4494 [1.3889, 1.5126] | 1.4911 [1.4381, 1.5460] |
    | 6 | 0.1579 [0.1553, 0.1607] | 0.1705 [0.1688, 0.1723] | 0.9261 [0.9090, 0.9436] | 0.9080 [0.9015, 0.9146] | 1.7907 [1.7231, 1.8609] | 1.9336 [1.8645, 2.0053] |
    | 10 | 0.1621 [0.1603, 0.1639] | 0.1779 [0.1760, 0.1798] | 0.9111 [0.8993, 0.9230] | 0.8868 [0.8689, 0.9050] | 3.0139 [2.8983, 3.1341] | 3.3326 [3.1977, 3.4731] |
    
    4-thread wall ratio 20 B / 24 B: 0.9574 [0.9453, 0.9695]
    
    ## Verdict
    
    VERDICT: MOVES THE PLATEAU: Q = A20(10)/A24(10) = 0.9044 [0.8505, 0.9617]
