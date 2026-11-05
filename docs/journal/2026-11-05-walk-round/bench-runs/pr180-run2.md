# #180 run 2: head c90b362 vs base 17daef2, run 41976148934 attempt 2, AMD EPYC 7763 (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t wall 0.727->0.725 -0.1% (-1.5,+1.4) (A/A control); 2t wall 0.464->0.403 -12.2% (-16.1,-8.1) FASTER cpu +0.0%; 4t wall 0.367->0.327 -10.3% (-11.7,-8.9) FASTER cpu +4.3% (+2.8,+5.9) SLOWER
peak RSS: 2t 46->49 MB, 4t 69->77 MB (+8 MB; cap +21.6 MB)
amplification 4t +4.4% (+2.4,+6.4) more; workers' CPU 4t 0.99->1.02 s
HARM SIGNAL: text 2t wall +3.0% (+0.7,+5.3) slower (run 1: +0.2% (-3.0,+3.4), not flagged) -> run 3 settles
CPU-only flags: random 2t +2.3%, records 2t +1.2%, text 2t +2.7%; records 1t -1.2% faster; runs 4t cpu faster; all same bytes
stages (1t, sequential path untouched by the diff, so an A/A): walk +5.8% (+0.4,+11.6) "slower", total +2.8% (-0.3,+6.1)
rule: benefit met (run 1 4t wall faster beyond noise; run 2 4t wall faster beyond noise); RSS within cap; text 2t harm signal -> run 3
