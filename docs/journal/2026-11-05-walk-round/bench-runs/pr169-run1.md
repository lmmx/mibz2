# #169 run 1: head f0a768a vs base 4d4709d, run 41951281465 attempt 1, Intel Xeon Platinum 8573C (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall 0.684->0.679 +0.1% (-1.9,+2.3) cpu +0.1% (-2.0,+2.3)
wikidata 2t wall 0.408->0.406 -2.4% (-5.7,+1.1) cpu -2.5% (-5.3,+0.3)
wikidata 4t wall 0.311->0.310 -0.5% (-4.2,+3.4) cpu -0.3% (-4.5,+4.0)
records 4t wall -1.9% (-3.7,-0.1) FASTER; no wall row slower; all same bytes
stages (1t c/B): scanner -1.1% faster; MTF+RLE2 +1.1% slower; build 1.73->1.68 -3.1% (-3.5,-2.7) FASTER; walk +0.1%; total -0.3% (-0.5,-0.2) faster
assessment: no change detected
