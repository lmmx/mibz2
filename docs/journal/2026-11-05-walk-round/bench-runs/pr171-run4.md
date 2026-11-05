# #171 run 4 (head run 1): head 2301189 (master e7c30a8 merged) vs base e7c30a8, run 41956321455 attempt 1, INTEL XEON PLATINUM 8573C (4 vCPU, 2 cores), bench 3.2.0, 10 rounds
wikidata 1t wall 0.871->0.834 -4.7% (-5.4,-4.0) FASTER cpu -4.7% FASTER
wikidata 2t wall 0.514->0.488 -4.8% (-6.0,-3.5) FASTER cpu -5.9% FASTER
wikidata 4t wall 0.395->0.372 -5.2% (-6.5,-3.9) FASTER cpu -7.0% (-7.8,-6.2) FASTER
amplification 2t -1.2% less, 4t 1.53x->1.49x -2.4% (-3.0,-1.8) less; workers CPU 1.10->1.02 s
pattern 1/2/4t -10.9/-11.9/-9.8% faster; records 1t -4.0% faster; text 1/2/4t faster; random 2t/4t cpu faster
HARM SIGNAL: runs 1t wall +1.9% (+0.1,+3.7) slower, cpu +2.1% (+0.3,+3.9) slower; runs 2t +0.0%, 4t +0.6%; all same bytes
stages (1t c/B): walk 6.33->5.70 -9.7% (-11.8,-7.6) FASTER; build +0.4% slower (untouched); scanner -0.9% faster (untouched); total -5.1% faster
assessment: Better (speed, CPU cost, contention sensitivity), no measure worse
runs 1t history: run1 (9V45) not flagged; run2 (7763) +1.7% (+0.0,+3.4) flagged; run3 (7763) -0.5% (-1.9,+1.0); run4 (8573C) +1.9% (+0.1,+3.7) flagged
