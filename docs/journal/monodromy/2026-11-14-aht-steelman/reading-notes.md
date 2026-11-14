# AHT / Lackenby / Erickson–Nayyeri: what they offer the inverse-BWT itinerary problem

## Sources read first-hand (PDFs in aht/dl/, text via pdftotext)
1. Agol–Hass–Thurston, arXiv math/0205057 **v2** (10 Jul 2004), PDF from https://arxiv.org/pdf/math/0205057. Read §4 and §6 in full. This is the arXiv version, not the Trans. AMS typeset copy.
2. Lackenby, "Some Fast Algorithms for Curves in Surfaces", DCG 76:539–588 (2026), doi 10.1007/s00454-026-00845-7. **Published open-access version**, from the Springer PDF link. Read §1–§5 opening and §3 (AHT) in full. I did not look for a separate arXiv version: the published text was available.
3. Erickson–Nayyeri, "Tracing Compressed Curves in Triangulated Surfaces", DCG resubmission dated 17 Mar 2013, from http://jeffe.cs.illinois.edu/pubs/pdf/tracing.pdf. Read §1, §3.2–§4.3.
   Not read: Schaefer–Sedgwick–Štefankovič (COCOON 2002, CCCG 2008), Štefankovič's thesis, Bell, Dynnikov, Dynnikov–Wiest. The statements below about them are as Erickson–Nayyeri and Lackenby report them.

## 1. AHT (arXiv v2), §4 "Orbits of interval isometries"
- **Pairing system.** On {1..N} there are k bijections g_i:[a_i,b_i]→[c_i,d_i]. Each is "orientation preserving" (a→c, b→d) or "orientation reversing" (a→d, b→c).
  - Domain and range are **positional**: "If a < c we refer to [a,b] as the domain and [c,d] as the range". They are not set by the dynamics.
  - Width is w = b−a+1. The translation distance is t = c−a.
  - A pairing is **periodic** if it preserves orientation and a < c = a+t ≤ b+1. Overlaps between pairings are arbitrary, and the pairings generate a pseudogroup.
  - **Maximal** pairing: d_i = N and its range is the widest among those ending at N (lexicographic order (d,−c,−a,−orientation)).
- **Lemmas.**
  - Lemma 7: a periodic pairing has t orbits on [a,d].
  - Lemmas 8–10: two overlapping periodic pairings whose overlap is at least t1+t2 have the same orbits as one periodic pairing of period gcd(t1,t2).
  - Lemma 11: contraction lowers the orbit count by the width removed. The other moves preserve it.
- **Moves.**
  - *Periodic merger* (Lemma 9; gcd).
  - *Contraction* of a static interval, one in no domain or range. This adds its width to the orbit counter.
  - *Trimming*: an orientation-reversing pairing whose domain and range overlap is cut to [a,(a+d)/2) → ((a+d)/2,d].
  - *Truncation*: a top segment [N'+1,N] that lies in the range of only one pairing g is "peeled off". The segment is removed and g shortens.
  - *Transmission*: when range(g2) ⊂ range(g1), g2 is replaced by g1^{-r}∘g2, or by g1^{-r}∘g2∘g1^{s} if domain(g2) ⊂ range(g1) as well. Here r and s are the **largest** powers that keep the image inside range(g1). This is the Euclid-division (Zorich-like) acceleration.
- **Orbit counting algorithm** (one pass through the steps is a "cycle"):
  1. Delete identity restrictions.
  2. Contract static intervals and add their widths to the counter.
  3. Trim.
  4. Merge periodic pairings.
  5. Find a maximal g_i and transmit every g_j whose range lies in [c_i,N'].
  6. Truncate at the smallest c such that [c,N'] meets exactly one range.
- **Theorem 12.** The algorithm "gives the number of orbits … in time bounded by a polynomial in k log N".
  - Proof: the complexity X = 4^k ∏ w_i at least halves every 5k cycles (Claims 3 and 4), so there are at most **5k²(2 + log₂ N) cycles**.
  - Each cycle costs "linear in k² log N" operations (comparisons, additions and subtractions, gcds).
  - Erickson–Nayyeri read this as O(n⁴ log³(X/n)) overall.
- **Output of Theorem 12:** a **single integer**, the number of orbits.
- **§6, Theorem 16 (weighted).** The input adds a weight function [1,N]→Z^d that is piecewise constant on m pieces, with total weight ≤ D.
  - The algorithm "outputs a list with one point for each orbit and corresponding orbit weights", in time poly(k m d log D log N).
  - The new move is a "transfer" of weights. It moves weights from the range to orbit representatives in the domain, using mod-t arithmetic in the periodic case.
  - Orbit weights are **sums**, so they are commutative.
- **Data structures assumed:** plain lists of pairings given by integer endpoints, plus a list of constant-weight intervals. Cost is counted in arithmetic operations on log N-bit integers. Each cycle rescans every pairing.
- **Ordered information:** none. No word, itinerary or SLP is produced.
- **Periodic merger cannot carry words.** The proof of Lemma 11 swaps g′ for "some word in g1 and g2" chosen point by point, so the merger keeps only the orbit partition.

## 2. Lackenby, DCG 2026 (published version)
- **Objects.** Normal and standard 1-manifolds in triangulated or handle-structured compact surfaces, closed surfaces included.
  - A standard 1-manifold is given as one pairing system per polygon: arcs of a parallelism class pair two integer intervals.
  - Lemma 2.18: there are at most 9|H1| + 2 simp(C) pairings.
- **Main theorems.**
  - **Thm 1.1:** the geometric intersection number i(C1,C2) in time poly(|T|, log w(C1), log w(C2)).
  - **Thm 1.2:** isotopy of essential 1-manifolds, with the same bound.
  - **Thm 1.3 / 2.15:** normalise a standard 1-manifold in poly(|T| or ||H||, simp(C), log w(C)).
  - **Thm 4.6:** cut along C.
  - **Thm 4.8:** locate a parallelity-bundle component.
  - **Thm 6.3:** minimal position.
  - Every bound is stated only as "a polynomial function". No exponents are given.
- **§3 restates AHT.**
  - **Thm 3.1 (Basic AHT):** the output is "the number of orbits", in poly(r, log N).
  - **Thm 3.3 (AHT):** with φ:[1,N]→Z^m, the output is "a list of orbits together with their φ-values", in poly(r, s, m, log N, log M).
  - **Thm 3.4:** the components of C, as normal-coordinate vectors.
- **How AHT is used.** Purely as a black box that counts orbits and sums commutative φ-values (Thms 3.2, 3.4, 4.6, 4.8). The φ coordinates are indicator functions such as arc type, boundary component and "contains p".
- **Ordered information:** none. Lackenby does not compute words, itineraries or SLPs, and gives **no bound on compressed itinerary size**.
  - He mentions SSS's straight-line programs and Schleimer's compressed words only in the related-work section (§1.1).
  - His own method modifies the triangulation, in the line of Bell–Webb, plus AHT.

## 3. Ordered and compressed itineraries in the cited literature
- **Erickson–Nayyeri (2013 DCG version)**
  - **Thm 4.4:** a connected normal curve can be traced in **O(n² log X)** time. Here n is the triangulation size and X the total crossings.
  - **Thm 4.7:** the same holds for all components of a reduced curve.
  - **Thm 4.8:** in O(n² log X) time one gets a **straight-line program of length O(n² log X)** encoding each component's ordered crossing sequence.
  - **Mechanism (§3.2–3.3):** a "street complex" of size O(n). The curve is traced in *phases*, each a maximal run of same-side turns. A *spiral* of depth d through m streets is done in O(m) by division, which they call "the use of division in Euclid's algorithm instead of repeated subtraction".
  - **Grammar rule per phase:** X_a → X_b (X_{i0}…X_{i(m−1)})^d X_{i0}…X_{i((ℓ−1) mod m)}. In Chomsky normal form that is O(m + log d) productions.
  - **Phase bound:** Lemma 4.2 gives at most O(N log X) phases (N = number of streets). Lemma 4.5 shows this is tight in the worst case.
  - **Lineage:** they state that their analysis is "nearly identical" to Dynnikov–Wiest's transmission-relaxation method, which is a special case of AHT, and that the potential function matches the "AHT-complexity".
- **Štefankovič [thesis Lemma 3.4.2; SSS COCOON'02 Lemma 3.1], as Erickson–Nayyeri report it:** the crossing sequence of a connected normal curve has an SLP of length **O(n log X)**, computable in O(n log X) time. I did not read this source first-hand.
- **SSS (COCOON 2002; CCCG 2008):** algorithms on normal curves through compressed intersection sequences, word equations and Plandowski–Rytter. These do produce ordered compressed sequences. I did not read them.

## 4. My own assessment (not the papers' words)
Map the IET onto AHT pairings. Each BWT run is one orientation-preserving pairing between its interval and the LF image of that interval, with no flips. So k = r and N = n.

### Point (a): can it carry ordered, noncommutative words W_{g∘f}(x) = W_f(x)·W_g(f(x))?
Partly.
- **Transmission and truncation are compositions,** so they can carry words.
  - In an IET each point lies in exactly one LF-domain and one LF-range. So if range(g2) and range(g1) overlap positionally, the upper interval of g1 must be its LF-*domain*.
  - Then g1^{-1} in AHT's positional sense is forward LF. The composite g1^{-r}∘g2 is a consistently forward map, and its word is W_{g2}·W_{g1}^r. The two-sided form adds W_{g1}^s in front.
  - Truncating the top segment is then first-return induction onto [1,N'].
- **So on a no-flip IET, AHT's transmission and truncation core is essentially right-endpoint Rauzy–Veech induction with Zorich acceleration.** That is the same elimination Cells already runs.
- **Periodic merger (the gcd step) is not a composition.** It keeps only the orbit partition, so it cannot carry ordered words. In AHT it is also what guarantees the log N bound when periodic stretches overlap.
- **Contraction only counts.**
- **Weighted AHT (Thm 16) and Lackenby's Thm 3.3** carry commutative orbit *sums* only. That is the abelianisation of the word, which is useless for the text.
- **So the only thing beyond Rauzy–Veech that word-carrying could gain from AHT is the "largest power r, s" in transmission,** the multiplicative step.
- If an ordered SLP of the itinerary is wanted, the right literature is Erickson–Nayyeri Thm 4.8 and Štefankovič's O(n log X)-length SLP, not AHT or Lackenby.

### Point (b): fewer compositions or fewer memory touches than S ≈ 1.8r?
No, as far as I can see.
- **The acceleration is all from the multiplicative step.** Every log N factor in AHT, and in Erickson–Nayyeri's phase bound, comes from Euclid-division acceleration of repeated subtraction: the powers r and s, spirals of depth d, and the gcd merger.
  - Remove those and AHT's progress is one truncation per cycle: worst case N, which is Theorem 12's naive bound.
  - Your data has about 94 streak steps in 27 million, so the multiplicative step essentially never fires. Rauzy–Veech is already near-linear in r there: S ≈ 1.8r is far below the r log n regime that these worst-case bounds target.
- **Step (5) does not reduce compositions.** It transmits every g_j inside the maximal pairing's range, but each of those is still one composition, matching one Rauzy step. Batching doesn't reduce their number.
- **Every elimination has a floor.** It must fold all r generators into one SLP of binary rules, so it needs at least r−1 rules. The headroom over 1.8r is therefore at most about 1.8×, and nothing in AHT, Lackenby or Erickson–Nayyeri targets that constant.
- **Memory touches are worse in AHT as written.** Each cycle rescans all k pairings (k² log N operations per cycle, up to 5k²(2+log₂N) cycles), so it is polynomial in k, not linear.
  - Erickson–Nayyeri cost O(m) per phase with m up to O(n), giving O(n² log X). Lemma 4.6 says this is tight for their abstract model.
  - Both are asymptotically worse in r than an O(S) Rauzy–Veech with a sorted endpoint structure.
- **Constant-factor wins would come from elsewhere.** Layout of the per-step state, or fusing steps, might help. None of these papers offers a route to them.
- **Bottom line:** AHT and Lackenby give a strong theoretical guarantee (polylog in n) for an adversarial regime of long periodic and spiral stretches that the BWT data doesn't show. They give no ordered output and no constant-factor gain on non-periodic data.
