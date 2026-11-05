# #183 run 1: head beef4e8 vs base 0769b52, run 41986885355, AMD EPYC 7763 (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t wall 0.692->0.683 -1.1% (-1.6,-0.7) FASTER cpu -1.2% (-1.6,-0.8) FASTER; 2t -2.4% (-5.8,+1.1); 4t wall 0.314->0.305 -4.2% (-10.0,+1.8) cpu -1.4% (-2.5,-0.3) FASTER
wall rows flagged slower: runs 4t +1.3% (+0.2,+2.4) (cpu +1.3% (+0.0,+2.7), RSS +0.5% more). CPU-only flag: random 4t cpu +2.0% (+0.0,+4.0) (wall +0.7%, ns)
faster flags: pattern 1t -1.7%, pattern 4t wall -4.4% (-8.5,-0.1); all same bytes; assessment: Better (CPU cost), no measure worse
amplification 4t 1.69x both; stages (phases.rs, separate stage functions, not the fused path): walk -1.3% ns, RLE1 -0.3% ns, CRC -0.5% faster, assembly -0.8% ns, total -0.7% ns
model: 4t dump 46m37s -> 44m31s (-4.2%, ns); 1t 1h40m -> 1h39m (-1.1% faster)
