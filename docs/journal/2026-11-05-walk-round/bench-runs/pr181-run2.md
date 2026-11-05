# #181 run 2: head eebe33f vs base c943331, run 41977512992 attempt 2, Intel Xeon Platinum 8573C (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t wall 0.817->0.814 -0.5% (-0.9,-0.1) FASTER cpu same; 2t wall -1.8% (-3.3,-0.3) FASTER cpu -1.1% FASTER; 4t wall -3.2% (-6.6,+0.2) cpu -0.9%
no wall row flagged slower (text 4t +5.7% (-1.1,+12.9), runs 2t +6.6% (-8.6,+24.4) not flagged); faster: pattern 1t, text 1t/2t, records 4t cpu, pattern 2t cpu; all same bytes
amplification 2t -0.6% less; assessment: Better (contention sensitivity), no measure worse
stages (1t c/B): walk -0.9% (-2.3,+0.5); MTF +0.7% slower (untouched by the diff); total -0.3%
rule: benefit met (run 1 1t wall -2.0% beyond noise; run 2 1t wall point faster, -0.5% beyond noise); no wall row flagged slower in either run -> PASS
