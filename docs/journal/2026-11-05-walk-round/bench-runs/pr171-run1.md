# #171 run 1: head 0a32416 vs base 4d4709d, run 41952512753 attempt 1, AMD EPYC 9V45 (4 vCPU, 2 cores), bench 3.2.0, 10 rounds
wikidata 1t wall 0.416->0.415 -0.1% (-2.4,+2.2) cpu -0.1%
wikidata 2t wall 0.286->0.278 -3.2% (-5.5,-0.8) FASTER cpu -5.4% (-7.6,-3.1) FASTER
wikidata 4t wall 0.280->0.241 -14.4% (-19.2,-9.5) FASTER cpu -17.6% (-21.7,-13.3) FASTER
amplification 2t 1.17x->1.11x (-5.3% less); 4t 2.23x->1.84x (-17.5% (-21.6,-13.2) less)
workers summed CPU at 4t: 0.77 -> 0.62 s
other rows: random 1t -2.7% faster, random 4t -3.0% faster, text 4t -4.6% faster; no wall row slower; all same bytes
stages (1t c/B): walk 3.06->3.08 +0.3% (-1.3,+1.9); RLE1 -2.1% faster; total +0.0%
assessment: Better (speed, CPU cost, contention sensitivity), no measure worse
