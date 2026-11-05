# #168 run 1: head eb3e857 vs base 4d4709d, run 41950036366 attempt 1, AMD EPYC 7763 (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall 0.767->0.776 +0.3% (-1.3,+1.9) cpu +0.3% (-1.3,+1.9)
wikidata 2t wall 0.498->0.496 +1.3% (-1.1,+3.8) cpu +2.3% (-0.6,+5.3)
wikidata 4t wall 0.395->0.397 +1.0% (-0.8,+2.9) cpu +1.7% (+0.0,+3.3) SLOWER
pattern 1t wall -2.2% (-3.3,-1.0) FASTER; no wall row slower; all same bytes
stages (1t c/B): scanner +0.4%; MTF+RLE2 +1.6% (+1.4,+1.8) slower; build +0.2%; walk 3.62->3.78 +4.0% (-0.7,+9.0); total +2.2% (+0.0,+4.3) slower
assessment: Worse (CPU cost), no measure better
