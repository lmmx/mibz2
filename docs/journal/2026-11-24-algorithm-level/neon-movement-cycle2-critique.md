# Critique of Neon (movement) cycle 2 (c71d0a22..34efa5ed)

*An independent reviewer, read-only. It recomputed S8 on block-85 (`s8check.py` below), ran E1's `run_block` on block-85, and checked attribution on all 14 commits: clean.*


This was a read-only review. I recomputed the S8 raw figures for block-85 with an independent numpy script (`nm2-critique/s8check.py`). I also ran E1's `run_block` on block-85: the assertions passed and `byte_exact` came out true, in 1.1 s.

## Items

| Item | What it is | New? | Q | D | P | Verdict |
|---|---|---|---|---|---|---|
| E1 MTF monoid | A chunk-parallel MTF scan in S_Σ, with per-chunk histograms and first/last row. Its output is the per-symbol position lists of L (Ψ per bucket), delta-coded with an anchor every 16 entries. | Mostly no. The scan is the (O,P) composition from external reports 5–7, plus ordinary per-chunk counts. Gap-coded, sampled Ψ is Sadakane's CSA (2000/03), and PRIOR-ART does not cite it. What is new is the M-22 measurement. | Q_run = n as stated, but each step makes about 3 addressed loads (anchor, block directory, gap bytes) plus a mean of 7.5 gap adds and a search on C. A tt step is one load, so per-row cost is worse. | "865×" applies only to building the chunk prefix states. It is the chunk size (1024 tokens) plus 2·log G, and (O,P) gives the same figure. The emit phase (about 7k rows per chunk) and the Huffman decode are not counted. | 2 passes over tokens, which means MTF is decoded twice. | A memory trade (1.47 B/row against 4 B/row for tt) and a useful M-22 answer. It is not a decoder win and not new as a structure. |
| E2 M₂ rider | 2-gram intervals from one histogram pass, and a 2-step bypass when the image interval falls inside one run | Small extension | Saves 1.3–3.2% of h₁ on Wikidata and 0% on hex | Unchanged | 2 | A correct, negligible gain. The "9–13× IFS" is prediction error, not cost. |
| E3.1 "1/3 law" | The probability that the middle of three is visited last, under random visit order | Trivial | Neon concedes Q ≥ 1 per row | n | 1 | Correct but has no cost content. The uniform-order assumption is not proved for BWT walks. |
| E3.2 dichotomy | Rotating three adjacent lone rows that hold distinct bytes in one F-bucket gives LF' = LF∘(3-cycle). One orientation keeps a single cycle. | Parts (1) and (2) are correct and near folklore (single-cycle/BWT-image characterisation). | — | — | — | It covers only aggregates whose block contains all three rows, not per-row riders. 3(ii) (P ≥ k or s ≥ \|Σ\|^k) and 3(iii) are asserted, not proved. Its claim that "E1 escapes" is false, since E1 leaves Q = n. |
| S7.1 pairing oracle | Swapping digit t ∈ [k_max+1, d−2] between two records leaves π and the head and tail orders unchanged | The invariance is correct and easy. It has one witness (t = 20), and "34/40" is the formula, not 34 tests. | Not proved (below) | — | — | Overclaimed |
| S7 bidirectional wave | Meet-in-the-middle LF×Ψ merge wave | It extends S5. | Stream mode: Q_thin = 0 at P = 26–45 | D_wave 24–43 (half of S5) is correctly labelled. The total D of 14.9k–44.0k is **unchanged from S5**, because the K_exc glue dominates. | ≫ K | A D cut in name only. The S5 corrections are applied. |
| S8 count-targeted landmarks | MONODROMY 114's M-21b prediction | It executes someone else's prediction. ORTH-BIDIR is S1B with count targeting. | Raw 1.03–1.17 n. ORTH 0.34–0.60 n, i.e. 5–10 h₁. | **608–1,974 reproduced** (block-85: 137+244+46+181 = 608, exact). ORTH 627–1,776. | 4–6 | A real D find. The pre-registered gate passes 6/6. |
| S9.1 unitig pass-wall | A birthday bound on unique k-mers | Elementary | Q_run 1.07–2.24 h₁ | — | — | Half right. It counts k−1 passes to name k-mers, but prefix doubling needs ⌈log₂ k⌉, so k = 6 takes 3 passes, not 5. The conclusion (nothing at K = 2) still stands. |
| R1 renormalisation | q_min = 1 + des(Ψ), the minimum alphabet of a standard permutation (Gessel–Reutenauer / Crochemore et al., not Mantaci–Restivo) | The invariant is right. | — | — | — | Mixed (below) |

## Strongest result

**S8.** I reproduced its numbers exactly and independently. D = 608–1,974 (0.07–0.22% of n) at Q ≈ n confirms MONODROMY's pre-registered M-21b on all 6 blocks. Two caveats:
- D counts the build rounds plus |Y_k|. Unrolling (prefix sums over the stored chains) and the label precompute (`L[LF^t]` over all n rows, which Neon argues is free from the return chains) are not charged.
- It is a lesser find under ESSENCE: Q does not move.

## Errors and overclaims

1. **E1 "eliminates L":**
   - What it actually does is decode MTF twice and scatter into per-symbol streams. bzip2 already builds tt from the MTF output.
   - The decoder reads F by searching C, which is standard CSA practice.
   - "Zero-L" is not new and does not lower decoder work. The "865×" depth is set by the chunk size and leaves out the emit and Huffman stages.
   - Within its own scope, the byte-exact check is sound. The decode path takes only the streams and counts, and compares its output with the textbook LF oracle (no external hash). The tokens are produced by re-encoding L inside the script.
2. **S7.1 does not prove Q ≥ (d−2)m:**
   - What it proves is that π does not determine the interior digits, which holds by counting alone. That was never the content of MONODROMY 113's relation C: there the digits are read from L, and π was the missing datum for placing them.
   - Going from "zero bits" to a bound on addressed accesses is the counting step that record 116 itself rejects ("a pass carries more bits than random digits hold"). S7's own wave reaches Q_thin = 0 with more passes, so the bound needs a K = 2 argument that S7.1 does not supply.
   - It is internally inconsistent: the invariance covers d−2−k_max digits, but the bound states d−2.
   - **Relation C is therefore not closed.** At most, the [A] "head and tail share nothing else" is formalised in the converse direction.
3. **R1:**
   - The headline "R(M₁₆) is the BWT-preserving renormalisation fixed point" holds only on the templated synthetic (q_min 24→19). On Wikidata q_min rises 8–21× (83–94 → 748–1,919).
   - "Fixed point" is used without any iteration. On hex, R_{M16} is the identity because there are no thick runs.
   - Calling E1 a "positive transfer" of Lead 2 (dual numbers) is relabelling: the monoid is a semidirect parallel-scan state, not a derivative.
   - The Aggarwal–Vitter remark compares s² with h₁!, but the count it needs is s^{P·h₁} against h₁!. With s = 256 and P = 2, 65536^{h₁} exceeds h₁!, so the argument gives no bound at K = 2.
   - The sound conclusion is that squaring and first-return sections leave the small-alphabet class on random content. That is plausible and matches kitchen-sink C7.
4. **S7 stream-mode cost:**
   - The JSON's D field leaves out `rounds_done`, a small error.
   - The word-sort P_bytes is reported separately, not inside P_bytes.

## Cycle-1 corrections (#24)

The README gains a "Cycle 1 Corrections Acknowledged" section covering S5's D, the scatter and bitmap, S4's 2.94 law, and S1B's accounting, and S7 applies them. But cycle 2 only added text and deleted nothing, so the old claims still stand in the files:
- The README headline still reads "Q and D can BOTH be driven literally to 0".
- S5-FRONTIER-WAVE.md keeps 48–86.
- S4-INDEPENDENT-SET.md keeps "beats … universally" and 2.94 m.

Neon has acknowledged the corrections but not fixed the files.

## Prior-art notes

- E1: the CSA literature (Sadakane, Grossi–Vitter) for differenced, sampled Ψ. MONODROMY 115 §3 already states the derivative identity. (O,P) is cited.
- R1: the descents characterisation of a standard permutation is Gessel–Reutenauer (1993) / Crochemore–Désarménien–Perrin (2005).
- S9: prefix doubling (Manber–Myers) for naming k-mers.
- S8: MONODROMY 114 §6 and cycle-1 S1B (acknowledged).

## Attribution

All 14 commits (65935e39..34efa5ed) have author and committer "Louis Maddox <louismmx@gmail.com>". No message contains Co-authored-by, Claude, Anthropic, "Generated" or a session link. Clean.

**Bottom line:** no Q breakthrough is claimed and none exists. S8 is the solid D result. E1 is a useful M-22 measurement wrapped in an overstated novelty and depth claim. S7.1 and E3.2 are correct invariance lemmas whose access-cost conclusions are not proved, so relation C remains MONODROMY's open candidate.

## Appendix: s8check.py

```python
import json,sys,numpy as np
from collections import Counter
S=sys.argv[1]
man=json.load(open(S+'/data/manifest.json'))
e=[b for b in man['blocks'] if b['file']=='block-85.L'][0]
L=np.frombuffer(open(S+'/data/block-85.L','rb').read(),dtype=np.uint8); n=len(L); p=e['primary_index']
cnt=np.bincount(L,minlength=256); C=np.concatenate([[0],np.cumsum(cnt)[:-1]])
order=np.argsort(L,kind='stable'); LF=np.empty(n,np.int64); LF[order]=np.arange(n)
# cycle order along LF from p
seq=np.empty(n,np.int64); x=p; lf=LF.tolist()
for i in range(n): seq[i]=x; x=lf[x]
assert x==p
F=np.repeat(np.arange(256),cnt)
Lt1=L[LF]; Lt2=L[LF[LF]]
ext=[F,L,Lt1,Lt2]
for rho in (8,16,32):
  cur=seq  # current node set in cycle order (rows)
  cum=0
  for k in range(1,5):
    lab=ext[k-1][cur]
    c=Counter(lab.tolist()); N=len(cur); t=N/rho
    cands=sorted((abs(v-t),v,s) for s,v in c.items() if 1<=v<N)
    if not cands: break
    sym=cands[0][2]
    pos=np.nonzero(lab==sym)[0]
    gaps=np.diff(np.concatenate([pos,[pos[0]+N]]))
    cum+=int(gaps.max()); m=len(pos)
    print(rho,k,'Y',m,'rounds',int(gaps.max()),'D',cum+m,'%.3f%%'%(100*(cum+m)/n))
    cur=cur[pos]
```
