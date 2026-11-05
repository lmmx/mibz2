# #180 run 1: head c90b362 vs base 17daef2, run 41976148934 attempt 1, Intel Xeon Platinum 8370C (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t wall 0.853->0.866 +1.2% (-0.7,+3.2) (A/A control: BzReader::new path, unchanged); 2t wall 0.535->0.467 -12.4% (-14.2,-10.7) FASTER cpu -2.5% (-3.9,-1.1) FASTER; 4t wall 0.441->0.407 -8.1% (-10.5,-5.5) FASTER cpu +6.1% (+3.1,+9.1) SLOWER
peak RSS: 2t 45->48 MB (+7.5%), 4t 66->74 MB (+13.1%, +8 MB; registered cap +21.6 MB); random/records/text 2t RSS +8-9%
amplification: 2t -3.7% (-6.0,-1.3) less; 4t +4.8% (+0.5,+9.3) more; workers' CPU 4t 1.24->1.32 s; efficiency 4t 48%->53%
no wall row flagged slower; pattern 4t CPU +7.3% (+0.9,+14.2) slower (CPU only); all same bytes
stages (1t c/B): all unchanged (total -0.3%)
rule: benefit met in run 1 (wikidata 4t wall); no harm signal; RSS within cap -> run 2 needed (point estimate faster)
