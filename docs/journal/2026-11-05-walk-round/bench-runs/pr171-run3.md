# #171 run 3: head 0a32416 vs base 4d4709d, run 41952512753 attempt 3, AMD EPYC 7763 (4 vCPU), bench 3.2.0, 10 rounds
wikidata 1t wall 0.858->0.748 -12.6% (-13.5,-11.6) FASTER cpu -12.5% FASTER
wikidata 2t wall 0.536->0.487 -9.8% (-12.4,-7.2) FASTER cpu -13.0% FASTER (RSS +1.9% more)
wikidata 4t wall 0.437->0.382 -13.0% (-14.8,-11.3) FASTER cpu -15.2% (-16.5,-13.9) FASTER
runs 1t wall -0.5% (-1.9,+1.0): run-2 harm signal NOT repeated -> unresolved, reported
pattern, random 2t/4t, records, text all faster or unflagged; no wall row slower; all same bytes
stages (1t c/B): walk 6.47->5.08 -20.2% (-27.6,-11.9) FASTER; scanner +0.7% and CRC +1.9% flagged (stage diagnostic, unchanged code); total 12.74->11.35 -10.0% (-14.6,-5.2) faster
amplification 4t -3.0% less; workers CPU 1.26->1.05 s
