# #171 run 2: head 0a32416 vs base 4d4709d, run 41952512753 attempt 2, AMD EPYC 7763 (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall 0.765->0.720 -7.3% (-9.4,-5.1) FASTER cpu -7.3% FASTER
wikidata 2t wall 0.501->0.449 -9.8% (-11.9,-7.7) FASTER cpu -9.8% FASTER
wikidata 4t wall 0.398->0.357 -10.4% (-12.0,-8.7) FASTER cpu -11.5% (-13.2,-9.7) FASTER
amplification 2t -2.8% less, 4t 1.72x->1.64x -4.5% less; workers CPU 1.12->0.97 s
pattern 1t -12.1%, 2t -12.7% faster; text 1t -2.9%, 4t -4.3% faster
HARM SIGNAL: runs 1t wall +1.7% (+0.0,+3.4) slower (cpu +1.6% (-0.1,+3.5) not flagged); all same bytes
stages (1t c/B): walk 5.26->4.63 -11.7% (-15.5,-7.8) FASTER; assembly -6.5%; total 11.43->10.80 -5.4% (-7.5,-3.3) faster
