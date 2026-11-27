claim: scanner-seam-window
by: codex
outcome: done
evidence: origin/evidence c0b2341cd073b354b36d3fb1ba658820e245ec2b, scanner-seam-window/run-20261127T070349Z.tar.gz, SHA-256 9ca9135a68104a15e7dc4e5509bc97f3b4938f133c3323bd718dcc223384e5d3; #1031; #1106

**NOT ADOPTED.** The i9-10900K measured candidate 3ae44390 against 95f4637b on Wikidata x4, 16 paired rounds. Both judged thread counts used eight workers. At 10 threads the wall ratio was 0.9811 (95% CI 0.9670–0.9954). At 12 threads it was 0.9898 (0.9726–1.0073). The registration requires an upper bound below 1.000 at both counts. Supporting counts 6 and 8 were inconclusive and 9 was faster. All five median peak RSS ratios were below 1.00.

**Integrity.** The archive matches its published SHA-256. All phases exited zero. Sixteen untimed checks matched libbz2's 838,800,784 bytes and decoded SHA-256 02949df2160357245d83e635cb155950ad0d31d2b4b8703d8e2d1be0bd3577bf. Independent replay of 160 main, 96 lazy and 18 occupancy records reproduced the verdict. At eight workers and 32 slots, median scanner maintenance fell from 10.689 to 7.490 ms (29.93%). The diagnostic target was 80%.

**Disposition.** Close #1031 and its superseded standalone prototype #857 without merging their code. This completes the registered attempt. Preserve the branches, ready tag and bundle. #846's symbol-buffer change is represented in #1034, whose separate CI objection and i9 keep/revert study remain on the frontier. #861 is the historical combined candidate measured against a different control. Its evidence and salvage disposition are preserved; this run isolates the seam on a baseline already containing #1034.
