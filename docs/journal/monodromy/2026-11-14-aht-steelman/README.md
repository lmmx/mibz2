# AHT / Lackenby steelman (QUEUED 6882319427): the record behind REVIEW on #145

Paper only; no timing. Verdict: STEELMAN FAILS (the REVIEW on #145 gives the argument).

- `reading-notes.md`: a sub-agent's first-hand reading, with locators, of Agol–Hass–Thurston
  (arXiv math/0205057v2, §4, §6), Lackenby (DCG 76 (2026) 539–588) and Erickson–Nayyeri
  (DCG resubmission, 17 Mar 2013). Schaefer–Sedgwick–Štefankovič, Štefankovič's thesis, Bell
  and Dynnikov(–Wiest) were not read first-hand.
- `aht_mibz2_orbit_experiment.py.txt`: the program relayed in #145 6882615225, recovered from
  the comment's raw JSON; SHA-256 8c9dd9e30dcec5ca59c408ef0c3e38c888a84d241ec654fc709258810f541c28,
  equal to the relay's. Its rerun: `aht_mibz2_orbit_experiment.out.txt` (exit 0).
- `count_batches.py.txt` and its output: the program's batching on two 20,000-byte texts
  (1 fold in 16,803 steps; 0 in 9,354).
- On the census cohort (the 196 admitted Wikidata blocks of `research/cells-census` b5617a8,
  matched one for one by block order and n to the per-block counts of entry 1's census): steps
  whose loser already sits next to the winner, the only steps the program's batching folds:
  28 of S = 20,456,267, in 25 streaks; at most 3 steps foldable.
