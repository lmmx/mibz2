# #180 run 3: head c90b362 vs base 17daef2, run 41976148934 attempt 3, Intel Xeon Platinum 8370C (4 vCPU), bench 3.2.1, 10 rounds
wikidata 1t -0.9% (-2.1,+0.3) (A/A control); 2t wall 0.486->0.433 -11.5% (-13.0,-10.0) FASTER; 4t wall 0.407->0.374 -8.9% (-10.5,-7.3) FASTER cpu +5.7% (+4.1,+7.3) SLOWER
peak RSS 4t 66->74 MB (+8 MB; cap 21.6 MB); amplification 4t +6.7% (+4.6,+8.9) more
text 2t +1.1% (-4.0,+6.6) NOT flagged -> run-2 harm signal unresolved, not confirmed
NEW harm signal: onebyte 4t wall +2.2% (+0.3,+4.1) slower (runs 1-2: -6.3%, -0.6%, unflagged) -> at the three-run cap: unresolved, reported
onebyte 2t -1.6% faster; runs 4t cpu faster; all same bytes; stages (sequential A/A) all flat
RULE: PASS. Benefit: 4t wall faster beyond noise in all three runs (-8.1, -10.3, -8.9). No harm confirmed (text 2t, onebyte 4t each flagged once, unresolved). RSS within cap. Trade-off: 4t CPU +4.3 to +6.1%.
