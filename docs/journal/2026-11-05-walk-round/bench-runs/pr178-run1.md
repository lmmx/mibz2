# #178 run 1: head e04e95c vs base 17daef2, run 41974509600 attempt 1, Intel Xeon Platinum 8573C (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t wall 0.801->0.798 -0.4% (-1.1,+0.3); 2t +0.3% (-1.0,+1.5); 4t wall 0.366->0.377 +2.3% (+1.3,+3.4) SLOWER cpu +1.4% (+0.1,+2.6) SLOWER
HARM SIGNALS: wikidata 4t wall +2.3%; pattern 1t +11.2% (+0.5,+23.2); pattern 4t +9.3% (+6.4,+12.3); amplification 4t +1.8% (+0.6,+3.0) more
text 2t -5.0% faster; all same bytes
stages (1t c/B): walk -0.2%; build -1.0% faster; assembly -1.6% faster
rule: benefit not shown in run 1; wikidata 4t wall harm signal -> run 2 on the same head (registered prediction: larger tile harms at 4t under SMT; 8573C L1D 48 KiB < 2 x 32 KiB)
