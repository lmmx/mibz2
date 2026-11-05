# #171 run 5 (head run 2, the settling run): head 2301189 vs base e7c30a8, run 41956321455 attempt 2, AMD EPYC 9V45 (4 vCPU, 2 cores), bench 3.2.0, 10 rounds
wikidata 1t wall 0.359->0.367 +0.4% (-3.2,+4.2); 2t -1.1% (-2.5,+0.2); 4t wall 0.235->0.205 -13.9% (-17.0,-10.7) FASTER cpu -16.3% (-19.7,-12.8) FASTER
amplification 4t 2.05x->1.71x -16.6% (-18.4,-14.8) less; workers CPU at 4t 0.60->0.49 s
runs 1t +0.2% (-3.4,+4.0): run-4 harm signal NOT repeated on the same head -> unresolved, reported
no wall row slower; text 4t faster; records 4t cpu faster; all same bytes
stages (1t c/B): build -5.9% faster (untouched code), walk +3.4% (-2.2,+9.2); total -0.1%
assessment: Better (speed, CPU cost, contention sensitivity), no measure worse
RULE: benefit met (wikidata 4t wall and CPU faster beyond noise in runs 1-5 bar 1t on 9V45 runs; never slower); no harm: runs 1t flagged in runs 2 and 4, each not repeated on its own head's next run -> unresolved; PASS
