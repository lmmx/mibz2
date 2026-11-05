# #178 run 2: head e04e95c vs base 17daef2, run 41974509600 attempt 2, AMD EPYC 9V74 (4 vCPU; Zen 4, 32 KiB L1D), bench 3.2.1, 10 rounds
wikidata 1t wall 0.584->0.724 +23.5% (+21.9,+25.2) SLOWER; 2t +17.2% (+14.2,+20.2) SLOWER; 4t wall 0.305->0.329 +8.2% (+6.0,+10.5) SLOWER cpu +9.0% SLOWER
pattern 1t +56.2% (+54.7,+57.6), 2t +37.7%, 4t +13.2% SLOWER; random 1t +3.2%, 2t +6.6% SLOWER; text 1t +8.7% SLOWER; onebyte 4t +2.2% SLOWER
rule: wikidata 4t wall flagged slower in run 1 (+2.3%) and run 2 (+8.2%) -> HARM CONFIRMED -> FAILS; close. A 32 KiB tile is the whole of a 32 KiB L1D: even 1 thread loses 23.5%.
