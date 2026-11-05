# #170 run 1: head 55dff0a vs base 4d4709d, run 41952047551 attempt 1, AMD EPYC 9V74 (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall 0.593->0.583 -1.4% (-2.7,-0.1) FASTER cpu -1.4% (-2.7,-0.1) FASTER
wikidata 2t wall 0.376->0.372 -0.5% (-3.4,+2.5) cpu -1.7% (-3.6,+0.2)
wikidata 4t wall 0.334->0.326 -3.3% (-7.6,+1.2) cpu -2.1% (-4.1,-0.0) FASTER
random 4t wall +2.7% (+0.4,+5.0) SLOWER (harm signal); text 1t/2t wall faster; all same bytes
stages (1t c/B): MTF+RLE2 2.09->2.00 -4.4% (-4.8,-4.0) FASTER; build -0.2%; walk -2.2% (-8.7,+4.8); total -2.0% (-5.2,+1.3)
assessment: Better (CPU cost), no measure worse
