# #183 attribution, fusion alone: head beef4e8 vs refactor arm 5761a32 (differs only in src/lib.rs), dispatch run 41986908473, Intel Xeon Platinum 8370C (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t 0.864->0.849 -2.3% (-4.2,-0.4) FASTER cpu -2.3% FASTER; 2t -2.7% (-5.3,-0.1) FASTER cpu -3.4% FASTER; 4t 0.392->0.387 -1.5% (-2.4,-0.6) FASTER cpu -1.9% (-2.8,-1.0) FASTER
wall rows flagged slower: text 2t +3.4% (+0.0,+6.8) (cpu +0.2% ns)
faster: onebyte 1/2/4t -5.0/-3.5/-6.7%; pattern 1/2t -3.1/-8.9%; runs 1/2/4t -3.4/-4.7/-2.7%; text 1t -2.0%; all same bytes; assessment: Better (speed, CPU cost), no measure worse
amplification 4t 1.76x both (+0.4% ns); stages (phases.rs, separate stage functions: blind to the fusion) all ns
