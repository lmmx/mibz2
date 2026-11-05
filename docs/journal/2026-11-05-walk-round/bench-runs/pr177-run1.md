# #177 run 1: head 3a7162b vs base 17daef2, run 41974498254 attempt 1, Intel Xeon Platinum 8573C (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t wall 0.762->0.758 -0.4% (-0.8,+0.0) cpu -0.4%; 2t -1.2% (-3.5,+1.1); 4t wall 0.360->0.359 -0.8% (-2.1,+0.6) cpu -0.8% (-2.1,+0.6)
HARM SIGNALS: onebyte 2t wall +4.2% (+0.8,+7.8) slower; pattern 1t wall +1.6% (+0.8,+2.4) slower; pattern 2t wall +8.3% (+1.8,+15.3) slower
amplification 4t -0.4% (-1.9,+1.2); RSS flat; all same bytes
stages (1t c/B): walk 5.34->5.28 -1.4% (-2.9,+0.1); scanner +1.9% (+0.8,+3.0) slower (untouched); total -0.6%
rule: benefit not shown in run 1; harm signals -> run 2 on the same head
