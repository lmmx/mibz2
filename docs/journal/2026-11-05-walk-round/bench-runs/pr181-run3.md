# #181 run 3 (merged head): head e3ea69f (master eb5363c, #180, merged in) vs base eb5363c, run 41979947814, AMD EPYC 9V45 (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t -0.9% (-3.2,+1.5); 2t -0.1%; 4t wall 0.207->0.199 -3.2% (-5.6,-0.7) FASTER cpu -3.5% (-6.0,-0.8) FASTER
no wall row flagged slower; pattern 1t -3.6% faster; text 4t -5.4% faster; all same bytes; assessment: Better (speed, CPU cost), no measure worse
stages (1t c/B): walk 2.90->2.74 -6.8% (-9.8,-3.8) FASTER; build -2.6% faster; total -3.8% (-5.9,-1.7) faster
rule: PASS (benefit runs 1-2 on eebe33f; run 3 on the merged head consistent and faster at 4t; no harm in any run)
