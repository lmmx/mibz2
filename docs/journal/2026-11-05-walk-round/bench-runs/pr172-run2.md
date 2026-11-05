# #172 run 2: head 0ccf706 vs base e7c30a8, run 41955025317 attempt 2, AMD EPYC 9V74 (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall -0.6% (-1.7,+0.6) cpu -0.6%; 2t -0.6% (-2.8,+1.7); 4t wall -0.1% (-1.9,+1.7) cpu -1.3% (-2.8,+0.2)
no wall row slower; text 4t cpu -3.4% faster, pattern 2t cpu faster, onebyte 1t cpu faster; all same bytes
stages (1t c/B): MTF+RLE2 2.00->1.99 +0.2% (-0.6,+1.0); scanner +0.7% (0.0,+1.3) slower (untouched by the diff); walk +0.0%; total +0.1%
assessment: no change detected
rule: benefit met (run 1 4t CPU -2.2% (-3.5,-0.8) faster; run 2 point -1.3% not slower); no harm met (no wall row slower in 2 runs)
