# Exchange identities of true near-threshold strips

The historical Open experiment at 2b53453e124ffdc75cc4d2141b833a319e3cfd73 already swept queued widths 8,16,24,32. The measured writer badb2aec3624bd65606c429b2e6f79e9d9d0050f retains whole true runs up to 16 except for its mandatory primary handling. At the same omitted run count, choosing the longest true runs minimizes retained rows. With the primary run fixed, every nontrivial 16-for-17 exchange costs one extra row. The three feature policies and neutral intervention here had identical per-block table lengths and omitted run counts. They spent at most 128 extra rows compared with the unchanged writer, kept the same successor width, and obeyed its 1 MiB bound for previously guarded admissions. [Frozen rules and split](protocol.md)

The endpoint initializes one terminal node for each positive gap between omitted strips on each of two orderings. This motivates omitting a length-16 candidate adjacent to more omitted strips while retaining a length-17 candidate adjacent to fewer. The selector observes the preceding and following run in original order and both neighbors in the same-byte stable LF order. A single additional scan fills a small candidate list; a counting pass over nine score bins avoids a sort. The bucket alternative ranks by the fraction of each byte's rows currently omitted. The neutral control takes the same number from each width in input order. All primary-containing runs retain the inherited classification.

## Prespecified training and held-out structural replay

In the four training streams (93 blocks), adjacency removed 0.390% of elimination steps and 0.990% of dictionary spelling bytes against equal-budget neutral, winning steps in each stream. Antiadjacency and bucket selection cost 0.216% and 0.150% more steps than neutral. Adjacency did not reach the frozen 1% step threshold. We committed the training winner at 8946a6f before opening the other eight streams. All 9,116 independent short marked-policy checks had active swaps. [Training geometry and exact choice](train.json) · [Forced frequent-width marks](small.txt)

On the 140 held-out admissions, adjacency made 10,482 exchanges and again removed 0.374% of neutral's steps, with fewer steps in all eight unseen streams. Its dictionary bytes fell 0.577% against neutral. Relative to the original champion, the picture changed: nodes 0.99344, elimination steps 0.99745, dictionary bytes 0.99632, word visits 1.00422, and retained rows 1.00060. Per-block table bytes were exact by construction. The selector retained up to 17,408 bytes. The real edge-vector capacities exceeded the champion by 436–872 bytes on these blocks, although no bit width changed. All 233 large actual admissions matched the Segmented inverse across both splits, totaling 209,697,662 coded bytes. [Held-out rows](held.csv) · [Separate structural metrics](held.json)

The limited paid diagnostic used the first two large admissions in each held-out stream. All 24 three-arm rounds replayed the same 16 blocks with each arm in each position eight times per block. It timed the frozen admission check, classification, actual capped input scanning, endpoint construction, and complete marked byte output on a pinned low-priority thread. The adjacency/control CPU ratio was 1.1422 [1.0955,1.1910]. Adjacency/neutral was 1.0137 [0.9849,1.0434]. Classification alone took a median 8.303 ms per 16-block lap. Subtracting its measured interval on the *same* calls leaves 1.0042 [0.9627,1.0474] vs the champion. This last subtraction is a diagnostic excluding real work. No held-out speed or worker-memory advantage supports a scorer timing or i9 request for these swaps. The structural result constrains this affordable neighborhood and choice of features; other section geometries remain open. [Paid phase samples](price.csv) · [Ratios](price-summary.csv)

```sh
d=research/astra-e/strip-selection
PAGE_DEPS=/path/to/release/deps bash "$d/build.sh" /tmp/astra-e-strips
o=/tmp/astra-e-strips
nice -n 16 taskset -c 4 "$o/strip" train corpora/wikidata.bz2 > "$o/train.csv"
python3 "$d/assess.py" train "$o/train.csv"
nice -n 16 taskset -c 4 "$o/strip" held corpora/wikidata.bz2 2 > "$o/held.csv"
nice -n 16 taskset -c 4 "$o/strip" price corpora/wikidata.bz2 2 > "$o/price.csv"
python3 "$d/assess.py" held "$o/held.csv" 2
PAGE_DEPS=/path/to/release/deps bash "$d/small.sh" /tmp/astra-e-strip-marks
```

The corpus SHA-256 is 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915. The shared host can inflate variability. These measurements exclude the entropy scanner, RLE1, CRC and delivery, and have no process-RSS or target-machine interpretation.

## Bound for all identity choices at the same swap budget

Represent the marked split's pieces in original row order and stable byte/image order. In either sequence put a virtual omitted piece at both ends. Twice the number of retained gaps equals the number of unequal retained/omitted adjacencies. Each positive gap contributes exactly one initial terminal node. An eligible length-16 or length-17 piece has four incident adjacencies counting both orders and multiplicities. Its individual gain in gap nodes when flipped is one half times (number of initially unequal incidences minus number initially equal). For a simultaneous flip set sum these individual gains. Each adjacency with both ends flipped has zero actual change. Correct the sum by +1 for an initially equal pair and −1 for an initially unequal pair. This identity is exact even when the same pair is adjacent in both permutations.

An upper bound for any q-for-q policy takes the largest q individual gains in each width class, allows every eligible equal adjacency to contribute its possible +1, discards all negative pair interactions, and caps the result at 4q (two gap nodes per flip). It uses the complete measured run geometry and does not need to know an optimal selection. Per-block crossing counts independently reproduced every unmodified endpoint's terminal-node count. Exhaustive tests checked all 23,596 feasible simultaneous exchanges across 7,692 finite marked-class budget problems against the bound.

On the held-out 140 blocks, the actual policy removed 6,985 of 1,064,543 initial endpoint nodes. No whole-run identity choice at these same q values could remove more than 11,619 initial nodes (1.0915%). The training bound was 6,727 of 738,431 (0.9110%), of which adjacency realized 4,269. This result bounds the initial node count only. It does not bound downstream elimination steps, dictionary spellings, or machine time. Those use the separate paid and structural observations above. [Per-block relaxation](gaps.csv) · [Validated totals](gap-summary.txt) · [Bound implementation](gap_bound.rs)

```sh
PAGE_DEPS=/path/to/release/deps bash research/astra-e/strip-selection/bound.sh /tmp/astra-e-gap-bound
python3 research/astra-e/strip-selection/bound_summary.py
```
