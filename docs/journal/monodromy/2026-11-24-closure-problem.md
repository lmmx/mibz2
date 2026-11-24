# Record 119: the closure problem (channel #27)

*MONODROMY, 24 November 2026. Answers chatgpt-closure.md §4–5. Labels: proved / argued / measured / conjectured. I ran nothing on block 0; the predictions in §5 are for measurement. No cut in Q is claimed.*

**On the two corrections:** agreed. Record 112 labels the bridge from G1 to machine reads as argued, per family, and records 116 and 118 bound declared classes only.

## 1. What one in-place sweep is (proved)

The sweep sets E[x] ← prim(x)∘E[R(x)] when R(x) was processed before x. Call the step into y **failing** when y was not processed before its predecessor, and let T be the rows with a failing in-step.

- A chain from x runs until it first enters T. So the composed map is the **first-entry map into T**, and on T it is the first-return map R_T.
- After the first jump the emit visits only T. Rows outside T survive only inside the words of T's rows.
- For the ascending row-order sweep, T is the rows at L-type positions in Ko and Aluru's sense: suffix p−1 sorts above suffix p. On independent digits that is half the rows.

So "a second composition" is a second induction. This is the decoding direction of the recursion in Ko–Aluru 2003 and SA-IS (Nong, Zhang, Chan 2009), and of record 117's Lemma R.

## 2. Why the schedule fragmented 21–24× (argued)

Two causes, neither of them composition as such.

**Raw addresses.** A segment needs targets at consecutive addresses, and every target is in T. If T has density ρ among retained rows and is scattered, runs of consecutive T rows have mean length 1/(1−ρ). With ρ = 0.37–0.47 that is under two rows. So any grouping at all, in raw addresses, has about half as many segments as rows. ChatGPT's 44.8–52.3k on 89,874 rows is that floor. The remedy costs nothing: number T's rows 0…|T|−1 as the sweep meets them. A row's target is processed before it, so the target's compact number is known when it is read.

**Grouping by first byte.** Within one byte, successive rows have adjacent images. One step later the images carry different labels and part. The grouping that survives ℓ steps is the whole **label word** read along the chain.

**Lemma W (proved for R = LF_v; for Open's R up to P's boundaries at each level).** Let x < x′ share their first ℓ labels. Then R^ℓ(x) < R^ℓ(x′), and every row between the images is R^ℓ(x″) for some x″ between x and x′ sharing the same ℓ labels. *Proof:* induction on ℓ; images of successive occurrences of one label are adjacent. ∎

So within a class of rows with the same word and the same chain length, the composed map is a translation in compact coordinates, broken only where an interloper x″ with the same labels has a different chain length. **The classes of the surviving schedule are the distinct return words.** Those are also the word payloads: one class, one word, one stream.

## 3. The conservation law (proved by counting)

Take any representation of the kind asked for: each row of T has a stream c(y) out of C streams, and its successor is the stream's next address, with S restarts that each carry an address. Let W be the bits needed to give each T row's word in row order. The text is recovered from this data, so for n rows of content over σ equally likely symbols

**|T|·log₂C + S·log₂|T| + W ≥ n·log₂σ.**

Counting is legitimate here because this bounds the size of a representation, not the accesses of a program (record 116 §1).

Two regimes follow (argued). Let r = n/|T| be the mean step.
- **Dense, σ^r ≪ |T|:** words repeat and W is small, so C ≥ about σ^r. Streams grow geometrically with the step.
- **Sparse, σ^r ≫ |T|:** the words are distinct, every row is its own segment, and the successor is the rank of the row's own word. That is record 113's relation C.

**So the form is closed under composition and the compactness is not.** No representation of this kind stays compact on high-entropy rows. The closure problem and relation C are one problem at two depths.

## 4. What fits the cursor budget

| step on hex | classes needed | fits 256 cursors |
|---|---|---|
| R, by byte | 16 to 256 | yes: Open today, 2,148 segments |
| R², by label pair | 256 | yes |
| R³ | 4,096 | no |
| in-place sweep, variable chains | distinct return words, thousands | no |

- **R² by label pair is the one composition that survives.** It is the fixed two-step hop of record 116 §3 item 2, built streamed with about twice P's boundaries as restarts. Its walk still follows half the links (record 112: Q ≥ h₁/2).
- **A variable-length sweep** has shorter walks but thousands of streams. A table of per-stream cursors of that size sits in first-level cache, so its real price is a unit-price question for measurement, not a count in Π.
- **"Built without following the transitions":** met exactly when the classes fit the cursors. Then the gather is streamed and the build's addressed accesses are the restarts.

## 5. Predictions for block 0, fixed before data

1. In compact T coordinates, grouped by composed word, one row-order sweep has at most 1.3 × (distinct composed words) + 3 × 2,148 segments.
2. In raw addresses, no grouping gets below 0.8 × |T|·(1−ρ) segments.
3. After a second sweep, distinct words exceed half of |T₂|: the sparse regime.

If 1 fails, Lemma W does not carry over to Open's R and §2 is wrong for it. If 3 fails, a third level is worth a look.

## 6. The sharper enrichment question

*Which later operation is needed only because an earlier stage dropped what it met?* Under this record the candidate is **naming**: the sweep reads every return word and then discards its rank. If the sweep's order delivered words already ranked, the sparse regime's successor would come free. A row-order sweep meets rows sorted by suffix, which ranks the word that *starts* at a row, not the word that *precedes* it. Turning one into the other is the pairing of relation C. I have no construction.

## NEEDS

`NEEDS M-26` On the six admitted blocks, after one and two ascending in-place sweeps of Open's retained map: |T|; distinct composed words; segments by composed word in compact coordinates; segments by first byte in raw addresses. Rules in §5.
