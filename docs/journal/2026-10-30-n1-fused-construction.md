# N1: a fused P^{2|4} construction, its work count, and the decision

**Status:** read-only analysis; nothing built or run. Counts: census (233 blocks, median
unless noted). Instruction costs: model, the per-operation figures of
`2026-10-30-n1-construction-economics.md` §3.3.

**Inputs (measured, median / p90 / max):** r/n = m₁/n = 0.0623 / 0.111 / 0.200;
m₂/n = 0.0756 / 0.136 / 0.227; upgraded u/n ≈ 0.94·m₂/n.

## Design (deduced)

Layout C′ builds P^{2|4} in five passes: level 1, count, prefix, merge (writes S₂), upgrade
(reads S₂, writes U). The fused design has three, and never materialises S₂:

1. **Level 1** from L's runs (or from MTF: r = #nonzero ranks + [q₀ = 0], report 59):
   records S₁ {D, byte}, bitvector B₁ and its directory.
2. **B₂ by blit:** B₂ = B₁ ∨ ⋁_I shift_{−D_I}(B₁[s_I+D_I+1, e_I+D_I)). Word operations per
   parent: 2 + ℓ_I/64. This is exact, because the level-2 starts are the level-1 starts
   plus the preimages of the level-1 starts strictly inside each image.
   m₂ = popcount(B₂), which is known before anything else is allocated, so the staged
   fallback (reconciliation §4.7) costs nothing extra.
3. **Emit, in source order:**
   - for each I: j₀ = rank₁(s_I + D_I), j₁ = rank₁(e_I − 1 + D_I);
   - for each J ∈ [j₀, j₁], the child K = I ∩ (J − D_I), with D_K = D_I + D_J and
     w_K = w_I‖w_J;
   - upgrade K iff nextset₂(k₀ + D_K) ≥ k₁ + D_K. If so, compose with
     J′ = (I′ = rank₁(k₀ + D_K), J″ = rank₁(k₀ + D_K + D_{I′})), all read from S₁;
   - write the final 12 B record (length in D's spare bits) sequentially.

   This is correct by Lemmas 1–3 (`2026-10-30-n1-upgrade-correctness.md`): every
   composition reads S₁ and B₂ only, so nothing is read after being overwritten.

## Work count (model, instructions per output byte)

| Term | Count/B | Instructions each | Instructions/B |
|---|--:|--:|--:|
| Level 1 (L scan + runs) | 1 + r/n | 0.3–1.0, 10–20 | 0.92–2.25 |
| B₂ blit | 2r/n + 1/64 | 3 | 0.42 |
| Emit, per parent (2 ranks + loop) | r/n | 16 | 1.00 |
| Emit, per child (nextset, compose, store, loop) | m₂/n | 19–25 | 1.44–1.89 |
| Upgrade (2 ranks, 2 gathers, compose) | u/n | 18 | 1.28 |
| Directories (B₁, B₂) | 2/64 | 6–10 | 0.19–0.31 |
| **Total, fused** | | | **5.3–7.2** |
| Layout C′, P^{2\|4} (economics note §3.3) | | | 4.5–8.4 |

**Reading (deduced):** instructions are about equal at the low end and 15% lower at the
high end. The rank queries replace the counting pass's popcounts; they don't remove them.

**What the fused design does remove:**
- two passes;
- S₂ and U;
- the peak: level-1 records 8r + B₁ + B₂ + directories + output 12m₂ ≈ 1.5 MB at the
  median, excluding L, against the reconciled 1.94 (L recycled) and 2.84 (L kept). At the
  maximum block it is about 4.1 MB > 4n, so the staged fallback is still needed for the
  high-r tail.

**The emit pass's random working set:** S₁ + B₁ + B₂ + directories ≈ 0.7 MB at the median.
That is L2 (1 MB) on Zen 5 and L3 on Zen 3 and 4 (512 KB L2). There are 4–6 independent
queries per output record, with no dependence between records, so memory-level parallelism is
permitted; the achieved parallelism is not measured.

## Break-even (deduced from the Gate B draft's ns/B thresholds)

- **Budget:** the P^{2|4} pass threshold is 1.03–1.18 ns/B on the two post-#114 7763 runs,
  that is 3.3–3.8 c/B at the reported 3.24 GHz.
- **Required IPC:** with 5.3–7.2 instructions per byte, IPC ≥ 1.4–2.2.
- **Mispredicts:** the upgrade test is 94/6 biased, so at most about 0.07 c/B.
- **The only measured relative,** #42's untuned run-native build, implies 16–27 cycles
  per element. This is weak comparative evidence, not an architectural limit (report 63).
  Its instruction counts and record density are not normalised to this design's, its loops
  branched on run length and divided by the cap, and this design needs neither.
- **MLP is permitted, not measured:** the throughput depends on the actual outstanding
  loads and cache locality.

## Decision

- **No substantial reduction in operations:** 5.3–7.2 against 4.5–8.4 instructions per
  byte. By report 62's rule, N1 does not earn a construction experiment on work count. The
  fused builder is not experimentally falsified: the required IPC is feasible in principle,
  but unsupported for this rank-query workload.
- **N1 reduces to one falsifiable number:** the achieved IPC of an L2/L3-resident,
  rank-heavy emit loop, which must be ≥ about 1.7 (1.4–2.2) on the 7763.
- **Archived:**
  - the fused design, as the best-known organisation: three passes, no intermediate
    arrays, about −0.45 MB of peak;
  - the IPC criterion, as the single question any future Gate B must answer first.

  Gate B stays unauthorised.
