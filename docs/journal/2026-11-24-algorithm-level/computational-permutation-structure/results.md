# Results: two labelled-composition operators, priced; a scoped obstruction and a sharper question

Claim `computational-permutation-structure`, under [`registration.md`](registration.md) (committed `deb5401`
before any data). Harness [`examples/permutation_operator_probe.rs`](../../../../examples/permutation_operator_probe.rs)
(zero `src/` edits, `#![forbid(unsafe_code)]`). Corpus `corpora/wikidata.bz2` (SHA-256 checked), 431 blocks,
**233 admitted** by Open's rule (the same 233 as #992/#998); train = first 117, held-out = last 116.
Machine: a 4-vCPU Xeon @ 2.10 GHz VM (2 MiB L2 per core, 260 MiB L3), pinned to CPU 0, one thread, warm.
It is not #992's machine (256 KiB L2): compare within this table only. Numbers for the i9 are ratio
projections, never a PASS.

**Verdict: both operators STOP.** Neither gets near the gate. Together with #992, #998 and the
Open × BDMR density-ratio result (#933 era, `2026-11-22-neon-teamwork-paradigm.md`), they give a
scoped obstruction: on this workload, any operator that costs one more pass over the retained rows
costs about as much as the whole matched 4-lane walk. What is left is a sharper missing primitive,
stated at the end.

## Reference and exactness

`T` = first return of raw LF to Open's retained rows (`LIMIT = 16`, the primary split out of a thick
run), indexed by retained rank. It was built offline from LF as a stand-in for Open's A6a–A6c output.
Its census agrees with #998: median `P_1(T)/m = 0.5517` (#998: `nLSRuns/m = 0.5517`), 82.0% literal visits.
Raw outputs: [`data/baseline.txt`](data/baseline.txt), [`data/native.txt`](data/native.txt). Every arm is **byte-identical to `bwt::walk` on 233/233 blocks**, at one lane and at four lanes, for
`r = 0..3` and in both builds. All 233 are single-cycle.

## O1: labelled powers `J_k = (p^k, W_k)` as interval maps

Rows `j, j+1` share a labelled translation piece of `p^k` iff their preceding `k`-mers are equal. So
`P_k(p)` is the run count of the preceding-`k`-mer column, with `P_k <= min(n, k·r)`. This was checked by
the recurrence `eq_k[j] = (L[j] = L[j+1]) ∧ eq_{k-1}[p(j)]`. Train, 117 blocks, median `m/n = 0.110`:

| k | `P_k(p)/n` median [p10..p90] | blocks with `n/k <= m` | blocks with `P_k(k+8) B <= 4m + dict` |
|---:|---|---:|---:|
| 1 | 0.0644 [0.0579..0.1069] | 0 | 0 |
| 2 | 0.0781 [0.0704..0.1304] | 0 | 0 |
| 4 | 0.0964 [0.0856..0.1593] | 2 | 0 |
| 8 | 0.1259 [0.1116..0.2058] | 25 | 0 |
| 16 | 0.1746 [0.1520..0.2817] | 117 | 0 |
| 32 | 0.2502 [0.2207..0.3882] | 117 | 0 |

On `T`, the median `P_k(T)/m` is **0.552, 0.631, 0.684 and 0.736** for `k = 1, 2, 4, 8`.

**Reading:** on raw LF, pieces grow far below the `k·r` bound (`P_32/P_1 ≈ 3.9`, not 32), so bundles
shatter slowly. But they start at 6.4% of rows. A labelled power needs `k >= 9` to make no more visits
than Open, and at that `k` its piece table is about 5× Open's whole representation. On `T`, the
shattering has already happened: a translation piece averages 1.8 rows, and composing only adds pieces.
**O1 is structurally excluded** on both criteria in the registration, so it was not timed. This is the
precise form of axis 1 of the brief. Composition of interval translations is exact and cheap per piece,
but on this map the pieces are rows.

## O2: word-capped labelled doubling of `T` by independent gathers

Each round: `J'(x) = J(J(x))` with the words joined while they fit in 12 inline bytes. It is one
sequential read, one *independent* gather and one sequential write per row, with no dependent chase, no
hashing and no renumbering. This is the form of a post-Open pass that the stopped `mat_f3b_post_open`
(a dependent cycle chase with renumbering) did not test. Visits: `J_1` 0.555×, `J_2` 0.342×, `J_3` 0.246×
of `m` (all 233 blocks).

On held-out (116 blocks, 7 interleaved repeats, ms median (min..max)), `C` is pass 0 (edges → 16-byte
records) plus `r` rounds:

| arm | baseline `C` | baseline `E` 4-lane | baseline `C + E4` | native `C` | native `E` 4-lane | native `C + E4` |
|---|---:|---:|---:|---:|---:|---:|
| `ctrl4` (Open format, 4 lanes) | 0 | **66.51** (65.07..68.73) | 66.51 | 0 | **62.20** (61.36..64.00) | 62.20 |
| `J_0` | 25.56 | 126.58 | 152.57 | 22.98 | 125.23 | 147.75 |
| `J_1` | 91.21 | 76.67 | 167.88 | 83.26 | 77.26 | 161.04 |
| `J_2` | 169.36 | 53.00 | 223.29 | 154.48 | 51.49 | 205.92 |
| `J_3` | 283.99 | **42.59** (0.640×) | 325.94 | 257.34 | **40.36** (0.649×) | 298.48 |

`ctrl1` is 78.23 ms baseline and 80.97 ms native. Train selected `r = 0`, the least-bad arm. Every arm is
slower than `ctrl4` in both builds, with disjoint spreads: the net is −86 to −259 ms against a 39.18 ms
gate, and its ratio projection is −421 to −1,269 ms.

**Mechanism.** One round costs 66–78 ms over 13.53M rows, **4.9–5.8 ns per row**. The whole 4-lane walk
costs 4.9 ns per visit. An independent gather is no cheaper per row than a visit of a 4-lane walk,
because that walk already has the memory-level parallelism a gather would add. Even `J_3`'s consumer gain
(24 ms) is 12× smaller than its construction. A break-even needs a round of at most about
0.6 ns per row. The record format also costs: `J_0` makes the same visits as `ctrl4` but its 16-byte
records (1.8 MB per block) spill L2, so it runs 1.9–2.0× slower. This VM's `ctrl4` is weaker than #992's
(0.80–0.85× of one lane, against 0.73×), which favours O2. It still loses.

## Scoped obstruction

On `wikidata` at `LIMIT = 16`, the following are all now priced:
- re-encoding the same visits (#998);
- a dependent post-Open row pass (#992);
- band-space grammar (density ratio `w̄ = 1.6–2.2`);
- labelled powers as interval maps (O1: pieces are rows);
- an independent row-gather composition (O2: about one 4-lane walk per round).

The common cause is quantitative. Literal visits (82%) sit where bundles have already shattered, so
`T`'s pieces average 1.8 rows, and any per-row pass costs about the walk it would shorten. **A
contraction constructor can pay only if it is sublinear in `m`, or fused into a pass that already touches
every row group** (A6a's run pass, A6b's band trace). It must also obtain *shared* words, as F1's RePair
words are, rather than per-row ones. This does not rule out composition that a prior pass gets for free,
nor other corpora or limits.

## The sharper missing primitive

*Given the run-length BWT and Open's bands, produce the frequent code digrams `(w(x), w(T x))`, each with
its set of rows as `O(1)`-describable intervals, at cost `O(B + #digrams)` rather than `O(m)`.*

Shared words exist (F1: 2.78–4.53M visits with 118–338 KiB of dictionary), but rows sharing a word pair
are BWT-non-adjacent, so no interval operator on `p` or `T` finds them. Sources checked:
- Policriti–Prezza (*LZ77 computation based on the run-length encoded BWT*, Algorithmica 2018) walks all
  `n` positions.
- Kempa (*Optimal construction of compressed indexes for highly repetitive texts*, SODA 2019) is
  sublinear in `n` in theory, through synchronizing sets, with no implementation at `n = 900k`.
- The lexicographic parse (Navarro–Ochoa–Prezza, IEEE TIT 2021) has `O(r)` phrases but bidirectional
  sources, so decoding it is again a dependency chase.

None supplies the primitive. The BWT/F-bucket/digram constructors of `bwt-digram-return-map`, merged
while this PR was open, also STOP ([its results](../bwt-digram-return-map/results.md)). Their contraction
costs 108–113 ms against a 69–72 ms consumer gain. They found that 76.5% of thin runs are `w = 1`
singletons, and that bands before A6c do not know target ranks. Both findings agree with this note's
1.8-row pieces and its per-row-pass floor, and independently. The question left open is narrower:
*does A6b's band trace, which already visits every band, see `(band, image band)` pairs whose rows are
contiguous enough to carry a shared word?* The singleton share suggests not on this corpus, which makes
the obstruction a corpus-level fact about wikidata at `LIMIT = 16`. It is not a theorem. **No new claim.**

## Smallest falsifying experiment

The obstruction rests on two measured ratios:
- an independent-gather round costs about one 4-lane walk visit;
- `T`'s translation pieces average 1.8 rows.

It is falsified by a corpus or limit where `P_1(T)/m <= 0.25`, or by a machine where one gather round
costs at most 0.15 of a 4-lane visit. `census` and `bench` measure both in under a minute per build.
