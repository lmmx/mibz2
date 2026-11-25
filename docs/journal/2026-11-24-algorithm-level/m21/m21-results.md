# M-21: landmark induction on the six admitted blocks (MONODROMY 114 §3, §5)

**Checks.** The SHA-256 of each L matches the manifest, and the run count r matches. The LF built from L and C, walked from origPtr, is a single n-cycle. The Y rows for the byte landmark are exactly the F-interval [C[c], C[c]+cnt[c]). Script and data: m21.py and m21.json in this directory. L is the pre-RLE1 column, so the "text" here is the RLE1-encoded text.

**Readings.**
- **Pair:** the most frequent cyclic, overlapping 2-gram of the text. Y is its occurrences, which form one F-interval (rows whose suffix begins with the pair). A lane stops on reaching a Y row.
- **Recursion:** Y_j = {y ∈ Y_{j-1} whose R_{j-1} return word, in emission (backward) order, has byte c_j at position j-1}. Equivalently, the landmark string grows by one byte to the left at each level, and c_j is the most frequent such extension.
  - Read literally ("begins with"), k=3 would be degenerate: every y in Y₂ has the same first byte L[y].
  - The greedy choice keeps |Y_j| as large as possible. A rarer c_j would cut the final walk but lengthen the rounds.
- **Gaps:** cyclic text distances between consecutive level-j landmark positions.

**Formulas.** Y₀ = all n rows. With k levels:
- Q = Σ_{j=1..k} |Y_{j-1}| + |Y_k|. Building level j steps every point of Y_{j-1} once; the final walk on R_k takes |Y_k| steps.
- D = Σ_{j=1..k} rounds_j + |Y_k|, where rounds_j = the largest number of R_{j-1} steps in one level-j gap. At j=1 this is the longest text gap.
- The walk has Q = D = n. Its Q/h₁ = n/h₁ = 25.6, 26.3, 16.2, 12.4, 9.27, 14.4 (blocks in the table's order).

**Landmarks (the same in all six blocks).**
- Byte: `"` at k=1, then `,"` at k=2 and `","` at k=3 (text order).
- Pair: `":` at k=1, then `e":` at k=2 and `pe":` at k=3. Block 207's k=3 is `ue":`.

| block | n | h₁ | σ | lm | k | \|Y_k\| | gap mean/p99/max | rounds | Q | D | Q/h₁ | D/n | lone∈Y |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 899931 | 35116 | 89 | byte | 1/2/3 | 135716/30581/15844 | 6.6/47/272 · 29.4/145/284 · 56.8/175/347 | 272/10/9 | 1.036/1.066/1.082 M | 135988/30863/16135 | 29.5/30.4/30.8 | .151/.034/.018 | 7.7% |
| | | | | pair | 1/2/3 | 41219/23294/11831 | 21.8/129/286 · 38.6/176/331 · 76.1/205/6475 | 286/8/134 | 0.941/0.964/0.976 M | 41505/23588/12259 | 26.8/27.5/27.8 | .046/.026/.014 | 0.3% |
| 2 | 900000 | 34262 | 86 | byte | | 135062/30167/15720 | 6.7/47/272 · 29.8/133/283 · 57.3/175/346 | 272/10/9 | 1.035/1.065/1.081 M | 135334/30449/16011 | 30.2/31.1/31.6 | .150/.034/.018 | 8.9% |
| | | | | pair | | 40939/23415/10868 | 22.0/106/285 · 38.4/205/330 · 82.8/205/13820 | 285/8/336 | 0.941/0.964/0.975 M | 41224/23708/11497 | 27.5/28.2/28.5 | .046/.026/.013 | 0.3% |
| 85 | 899935 | 55504 | 94 | byte | | 138920/30044/16773 | 6.5/48/239 · 30.0/89/250 · 53.7/180/272 | 239/12/9 | 1.039/1.069/1.086 M | 139159/30295/17033 | 18.7/19.3/19.6 | .155/.034/.019 | 9.7% |
| | | | | pair | | 42025/21403/13035 | 21.4/73/247 · 42.0/206/248 · 69.0/241/943 | 247/8/28 | 0.942/0.963/0.976 M | 42272/21658/13318 | 17.0/17.4/17.6 | .047/.024/.015 | 0.8% |
| 143 | 899869 | 72578 | 96 | byte | | 128934/27655/14279 | 7.0/49/419 · 32.5/149/430 · 63.0/231/452 | 419/10/12 | 1.029/1.056/1.071 M | 129353/28084/14720 | 14.2/14.6/14.8 | .144/.031/.016 | 8.2% |
| | | | | pair | | 39402/21619/8788 | 22.8/131/427 · 41.6/187/439 · 102/508/17579 | 427/8/425 | 0.939/0.961/0.970 M | 39829/22054/9648 | 12.9/13.2/13.4 | .044/.025/.011 | 0.6% |
| 210 | 899959 | 97054 | 96 | byte | | 136448/29978/15587 | 6.6/48/484 · 30.0/115/496 · 57.7/179/530 | 484/16/9 | 1.036/1.066/1.082 M | 136932/30478/16096 | 10.7/11.0/11.1 | .152/.034/.018 | 7.6% |
| | | | | pair | | 41681/21663/11607 | 21.6/87/498 · 41.5/177/504 · 77.5/251/6999 | 498/8/157 | 0.942/0.963/0.975 M | 42179/22169/12270 | 9.70/9.93/10.05 | .047/.025/.014 | 0.7% |
| 207 | 899998 | 62407 | 94 | byte | | 129704/27598/14374 | 6.9/50/902 · 32.6/147/933 · 62.6/230/952 | 902/10/9 | 1.030/1.057/1.072 M | 130606/28510/15295 | 16.5/16.9/17.2 | .145/.032/.017 | 9.9% |
| | | | | pair | | 39244/22732/8929 | 22.9/129/921 · 39.6/180/939 · 101/342/1147 | 921/8/14 | 0.939/0.962/0.971 M | 40165/23661/9872 | 15.1/15.4/15.6 | .045/.025/.011 | 1.3% |

**Lone rows.**
- The lanes partition the cycle, so every row, and therefore every lone row, lies on some landmark lane. That is trivially all of them.
- The non-trivial figure is the lone∈Y column, given at k=1: the share of the h₁ lone rows that are themselves landmark rows.
  - Byte: 7.7–9.9% of lone rows, against Y's 14–15% of all rows. Lone rows are under-represented in Y by about 0.6×.
  - Pair: 0.3–1.3%, against Y's 4.5%.
  - At k=3 the share is at most 0.1% for both.

**Findings.**
1. **Q does not fall; it rises.** The Q column is absolute, in millions of steps.
   - Byte: Q/n ≈ 1.15 / 1.18 / 1.20 at k = 1/2/3.
   - Pair: Q/n ≈ 1.046 / 1.07 / 1.08.
   - Q/h₁ is therefore 4–20% above the walk's n/h₁, across 10–32.
2. **D falls, but not geometrically as n/|Σ|^k.** At k = 1/2/3:
   - Byte: D/n ≈ 0.15 / 0.033 / 0.018.
   - Pair: D/n ≈ 0.046 / 0.025 / 0.013.
   - That is about 10–17k dependent steps at k=3, against the "few hundred" record 114 projects for hex-dense content. On this JSON the greedy landmark strings (`","`, `pe":`) stay frequent, so the final walk |Y_k| dominates D. The rounds are at most about 900.
3. **The long tail is in the gaps.**
   - The k=1 maximum gaps (239–921) are 5–19× the 99th percentile (47–50).
   - The pair's k=3 text-gap maximum reaches 17579, from clustered `pe":`-free stretches. Even so, its rounds stay at most 425.
