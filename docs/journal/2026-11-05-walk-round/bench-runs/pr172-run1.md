# #172 run 1: head 0ccf706 vs base e7c30a8, run 41955025317 attempt 1, AMD EPYC 7763 (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall +0.2% (-1.6,+2.1); 2t +0.9% (-1.3,+3.1); 4t wall -2.2% (-4.7,+0.3) cpu -2.2% (-3.5,-0.8) FASTER
text 2t cpu faster; no wall row slower; all same bytes
stages (1t c/B): MTF+RLE2 2.03->2.00 -1.3% (-1.8,-0.8) FASTER; walk +2.4% (-3.8,+9.0); total +1.1% (-2.4,+4.7)
assessment: Better (CPU cost), no measure worse
