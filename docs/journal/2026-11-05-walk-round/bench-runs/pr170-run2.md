# #170 run 2: head 55dff0a vs base 4d4709d, run 41952047551 attempt 2, AMD EPYC 7763 (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall 0.780->0.765 -2.1% (-3.0,-1.1) FASTER cpu -2.0% FASTER
wikidata 2t wall -1.0% (-2.5,+0.5) cpu -2.0% (-3.5,-0.4) FASTER
wikidata 4t wall 0.410->0.397 -1.9% (-3.6,-0.2) FASTER cpu -2.0% (-3.3,-0.7) FASTER
random 4t wall +0.5% (-2.1,+3.2) not flagged (run-1 harm signal not repeated -> unresolved)
text 1t/2t/4t faster; onebyte 1t faster; no wall row slower; all same bytes
stages (1t c/B): MTF+RLE2 2.01->1.82 -9.7% (-10.0,-9.3) FASTER; total -3.0% (-4.3,-1.6) faster
assessment: Better (speed, CPU cost), no measure worse
