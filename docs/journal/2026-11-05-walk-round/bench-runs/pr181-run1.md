# #181 run 1: head eebe33f vs base c943331, run 41977512992 attempt 1, AMD EPYC 9V74 (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t wall 0.571->0.557 -2.0% (-2.7,-1.2) FASTER cpu same; 2t wall -2.4% (-5.1,+0.2) cpu -2.2% (-2.8,-1.5) FASTER; 4t wall +0.2% (-1.5,+1.9) cpu -0.3%
no wall row flagged slower; faster: onebyte 4t, pattern 1t, records 1t, text 1t -2.1%, text 2t -8.6%; all same bytes
amplification 4t +1.7% (+0.6,+2.8) more (CPU figure); RSS flat
stages (1t c/B): walk 4.23->4.16 -2.6% (-4.3,-0.8) FASTER with +4% Ir; build +0.4% slower; total -1.0% (-1.8,-0.2) faster
rule: benefit met in run 1 (wikidata 1t wall); no harm -> run 2 (1t wall point estimate must be faster)
