# #177 run 2: head 3a7162b vs base 17daef2, run 41974498254 attempt 2, AMD EPYC 9V74 (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t +0.1% (-0.2,+0.5); 2t -0.5% (-2.0,+1.0); 4t wall +0.8% (-0.4,+2.0) cpu +0.6% (-0.1,+1.3)
pattern 1t wall +1.3% (+0.2,+2.4) SLOWER (flagged in run 1 too: +1.6% (+0.8,+2.4)) -> HARM CONFIRMED
onebyte 2t +1.9% (-0.2,+4.1) not flagged (run 1 +4.2%) -> unresolved; onebyte 4t cpu +1.4% slower; pattern 4t wall -2.8% faster; text 1t -0.5% faster
rule: benefit not shown in either run (wikidata 1t/4t never faster beyond noise); harm confirmed on pattern 1t wall -> FAILS; close
