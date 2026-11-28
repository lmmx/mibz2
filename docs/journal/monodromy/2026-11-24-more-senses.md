# 2026-11-24 · MONODROMY 114 · More senses of "moving together", one principle that orders them, and one candidate worked up: landmark induction, which leaves Q where it is and takes the dependent depth D from n to about n/|Σ|^k

Answers channel entry #21 (all senses, none leads). Adds to record 113's A–D. Labels as before. **No breakthrough in Q is claimed.**

## 1. The principle that ranks every sense [P for the stated form; record 112 Theorem B2]

In one sweep, a row can be joined to its successor only if the successor's fact is already final when the row is processed. So for any processing order ≺ a link must start at least at every second ≺-ascent along the orbit. **Every sense of "moving together" induces an order on rows, and its figure of merit is how often that order agrees with text order.** Text order itself gives one link; it is the answer. The question "what else can move together" is the question "what cheap order on rows correlates with text position".

## 2. Further senses

| | sense | prior art | on novel random digits | where the cost goes |
|---|---|---|---|---|
| **E** | *same landmark*: rows between two consecutive text occurrences of a chosen symbol c travel as one lane; the lanes start at the F-interval of c, which is contiguous and free | lanes (toy-transfer); the frontier (record 104); wave inversion of collection BWTs (Bauer–Cox–Rosone, from memory, not re-read) | lanes of mean length \|Σ\| regardless of content | §3 |
| **F** | *order by distance to the next landmark* | this is E seen as an order (§1) | descents 15/16 of the time, against 1/2 for row order | labelling rows with that distance is a chain ranking: rounds, or addressed accesses |
| **G** | *alphabet reordered*: sweep in (τ(byte), row) order for a chosen permutation τ of the alphabet | none found in my records | still 1/2: a random digit is above or below its neighbour equally often under any τ [A] | none, and no gain; it does make a whole template segment one chain where the template's bytes are τ-monotone |
| **H** | *co-lexicographic neighbours*: in the order of reversed prefixes, every same-word class of sense A is an interval for all k at once, so closure (S4) holds | the bidirectional BWT (literature) | classes are singletons past k ≈ 5 all the same | transport between the two orders is not O(1) |
| **I** | *bit planes*: LF as four stable binary partitions | wavelet-tree LF (literature) | runs of mean 2 per plane, four planes: 2 steps per row | worse than the walk |
| **J** | *together up to a predicted address*: on a random region LF on a symbol class is nearly linear with slope 1/\|Σ\| | interpolation and learned indexes (literature) | predicts the target to within about √w, not exactly | exactness needs the correction, which is the access |
| **K** | *together in the output*: bytes at one template offset are written as one strided stream | none found | needs the wave in text order | the same pairing as relation C |
| **L** | *together up to a fingerprint*: hashes of words compose in O(1) | Karp–Rabin (literature); record 105 §4 on checksums | can check a copy, cannot transport | exactness |

## 3. Sense E worked up: landmark induction

**Construction** [P that it decodes; standard pieces]. Fix a symbol c and let Y = I_c, the rows whose suffix begins with c. Y is one interval, known from the symbol counts.
1. *Build.* Start one lane at every row of Y. In rounds, every live lane takes one LF step and appends the byte it emits; a lane stops when the byte is c, and records the row of Y it has reached. Lanes are kept in row order, which a stable partition by byte preserves (record 104 Lemma F).
2. *Result.* The first-return map R : Y → Y with its return words. This is Rauzy–Veech's own operation, inducing on a subinterval, done pointwise for all of Y at once.
3. *Recurse.* Apply the same step to (Y, R) with a second landmark: Y₂ = the rows of Y whose return word begins with c′, and so on, k times.
4. *Emit.* Walk the last induced map and write the words.

**Its vector under Π** [P for Q and D given the gap statistics; P and S depend on layout]:

| | Q | D | notes |
|---|---|---|---|
| walk | n | n | |
| landmark induction, depth k | ≈ n·(1 + 1/\|Σ\| + …) | (longest gap) + n/\|Σ\|^k | every build access is independent and in ascending row order within its round |

**Where it stands, said plainly.**
- **Q does not fall.** Under Π as written this is not a lowered floor. It is record 113's relation C with the acquisition problem removed: the anchors are free.
- **D falls geometrically**, from n to a few hundred at k = 3 on hex-dense content. If "unpredictable access" in Louis's decision means *dependent* access, this is the quantity that moves; if it means *addressed* access, nothing moves. **That reading is the decision this candidate turns on.**
- **Price.** Each lane step moves a record. The team's only figure is lanes with branchless appends at 0.55–0.78× of emit, with 8 lanes. Whether thousands of lanes in sorted order cost more or less per step is unmeasured.
- **Ceiling on utility.** The supplied-codes diagnostic (main-theorem §7) bounds any emitter at about 0.84 of the whole decoder at 1 worker and 0.99 at 10.
- **Not new in its parts.** Lanes, the frontier and wave inversion exist. What I have not found in our records is the choice of anchors (a symbol's F-interval, so nothing is acquired) and the recursion on the induced map. I have not searched the `research/*` branches; toy-transfer's own choice of lane starts I have not read.

## 4. Ranking by how far each can be taken next

1. **E / F, landmark induction.** Fully specified, exact, measurable today. One reading of the referee decides it.
2. **C, the pairing** (record 113). The only route to a lower Q I can name; no construction.
3. **K, output-side streaming.** Reduces to C's pairing.
4. **H, co-lex order.** Fixes closure for sense A, which random digits defeat anyway.
5. **A, D, G, I, J, L.** Each stops for the reason in its row.

## 5. NEEDS

- **NEEDS M-21** on the six admitted blocks, for c = the most frequent byte and for the most frequent pair: the number of landmark rows; the gap distribution (mean, 99th percentile, maximum); and (Q, D) of landmark induction at k = 1, 2, 3 computed from those.
- **Question for the Coordinator, not a measurement:** does "unpredictable access" in the adopted currency mean addressed or dependent? Record 111's vector reports both; the verdict on sense E depends on which is the target.

## 6. Addendum after NEEDS M-21 (channel entry #23; the Coordinator's figures, cited, not rechecked)

**Measured:** Q = 1.05–1.20·n; D at k = 3 is 1.3–1.9% of n (10–17 thousand steps); at most about 900 rounds per level.

**BREAK, own §3.** I wrote "a few hundred at k = 3". That used n/|Σ|^k, which assumes the landmark's count falls by |Σ| per level. My NEEDS line then asked for the *most frequent* byte and pair, which keeps the count high, and on templated text the frequent strings stay frequent. The estimate and the request were inconsistent; the fault is mine, and the Coordinator's readings of the recursion were reasonable.

**The selection rule the construction needs** [P for the identity; A for the optimum]. D = (rounds, summed over levels) + |Y_k|. The final walk is |Y_k|, the count of the last landmark string, so the landmark must be chosen by **count**, not by frequency rank: at level j take the extension whose count is nearest |Y_{j−1}|/ρ for a fixed ratio ρ. Rounds at a level are the longest gap between consecutive landmark rows on the induced orbit, about ρ·ln|Y_{j−1}| if the gaps are geometric and shorter if the text is periodic. With ρ = 16 and k = 4 that predicts D of the order of 10³, against the measured 10⁴. **This is a prediction, not a result.**

**What does not change:** Q rises slightly. Landmark induction is a find in D only.

**NEEDS M-21b** on the same six blocks: landmarks chosen by count, ratio ρ ∈ {8, 16, 32} per level, k up to 4; report |Y_j|, rounds per level, and D. Rule fixed before data: if D at the best (ρ, k) is not under 0.5% of n on four or more blocks, the count-targeted prediction is withdrawn and the measured 1.3–1.9% stands as the method's figure.
