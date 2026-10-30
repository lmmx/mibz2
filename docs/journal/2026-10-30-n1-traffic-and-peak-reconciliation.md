# N1: the builder's traffic and peak memory, reconciled

**Status:** read-only analysis. Nothing was built, benchmarked or run on CI. It reconciles two
models of the same construction and corrects both. It authorises nothing.

**What it reconciles:**
- the portfolio's layout C (`2026-10-30-research-portfolio.md` §3.1(i)–(j): 17.0 B/B,
  dup/dup/idx 17.5, peak 4.23 MB against the T-vector's 4.5);
- the construction-economics note's layout C′ (`2026-10-30-n1-construction-economics.md`
  §3.2: 22.5 B/B, 25.5 with caps as a separate pass; peak 3.79 MB excluding L).

**Boundaries:** every byte figure is per output byte of the inverse BWT (n, the RLE1-coded
block), one block, one thread. Input: L (n bytes, written by MTF) and `counts[256]`. Output:
the walk's table and its directory. Memory figures are MB (10⁶ B) per worker.

**Labels:**
- **measured:** census counts (`data/census.json`, 233 full bench blocks), or recorded runs and
  journals, cited;
- **read from code:** mibz2 `origin/master` at **`ae942bd`** (`src/bwt.rs`, `src/lib.rs`,
  `src/parallel.rs`, `src/coop.rs`), read with `git show` and nothing checked out. Every
  shipped-side "from code" figure is from `ae942bd` unless marked `155bb99`. That is the stale
  local checkout, from before #114 (`6a341f6`, the one-pass build), and it is kept only where
  the old build is the subject;
- **deduced:** follows from measured counts and the code, with no further assumption;
- **model:** an estimate, with its assumptions listed (§7).

**The byte convention (both source models use it, and so does this note):** each access is
counted at its element size (4, 8, 12 B), and each pass over an n-bit bitvector as n/8 bytes,
whatever its pattern. There's no cache-line amplification. §3.5 gives a word-access sensitivity.

## 1. Summary

1. **The portfolio's 17.0 B/B leaves out four of the accesses it needs** (deduced). At the
   median block:
   - the counting pass's read of the parents (+2.77 B/B);
   - the F prefix's image-order reads (+1.28 net);
   - the merge's reads of the offsets it writes: E[i] gathers (+0.92) and F[j] reads (+0.55);
   - it reads an 8 B/run list rather than L (+0.50). That is N3's boundary, set against a
     T-vector figure that reads L.

   It over-sizes the final records (16 B where 12 suffice, −0.49).
2. **C′'s 22.5 B/B is right to within 0.2 B/B** (deduced). Its two slips nearly cancel:
   - it counts f, F and an image sweep at the last doubling, which needs none of them (−0.37);
   - it leaves out the merges' bitvector marks and the final clear (+0.38).

   Its explanation of the gap is half right. The S_a[i] gather is in both models. The real
   differences are those in point 1.
3. **The reconciled P⁸ figure** (deduced; design choices as a range):

   | | min / p10 / median / p90 / max, B/B |
   |---|---|
   | low (8 B SoA count read, caps fused) | 18.9 / 20.1 / **21.7** / 35.7 / 56.4 |
   | like-for-like with C′ (12 B count read, caps fused) | 19.7 / 21.0 / **22.6** / 37.4 / 59.1 |
   | high (12 B count read, caps as a separate copy) | 22.1 / 23.6 / **25.6** / 42.4 / 66.1 |
4. **The shipped build moves 6.0 B/B** (master `ae942bd`, read from code; deduced).
   - Since #114, `bwt::build` writes each entry whole, `ll[s] | i << 8`, in one pass. It
     reads L sequentially (1 B), reads ll[s] at each slot written (1 B), and writes the
     4 B entry. There is no init pass and no read-modify-write.
   - **10.0 B/B with write-allocate** (each scattered tt line read before it's written).
   - The two-pass build (14.0 B/B) now runs only when the counts aren't L's own, which is
     never on the decoder's path.
   - **The portfolio's 5.0 was 1.0 B/B low:** it left out the ll[s] read. The construction
     note gave no figure.
   - **What N1 moves against the one-pass build** (element convention, median):
     - P⁸: 3.62–4.26×;
     - P^{4|8}: 2.89–3.60×;
     - P^{2|4}: 1.66–2.20×.
   - **With write-allocate on both sides:** P⁸ 3.07–3.61×, P^{4|8} 2.37–2.92×,
     P^{2|4} 1.35–1.77×.
   - So the portfolio's 3.4× for P⁸ was about right, by offsetting errors. The traffic is as
     adverse as recorded. Bytes still don't decide it (both source notes agree): N1's arrays
     are L2/L3-sized, and the T-vector isn't.
5. **The upgrade pass (C′) leaves out its rank query's directory reads** (deduced), 16–32 B per
   parent. Reconciled medians:
   - P^{2|4}: 10.0–13.2 B/B (C′ 9.24);
   - P^{4|8}: 17.3–21.6 B/B (C′ 16.48).
6. **Peak memory, accounted the same way on both sides** (deduced; §4):
   - **The shipped path's peak is its walk, not its build.** During the walk, the 3.6 MB
     T-vector and the segment buffers (0.9–1.83 MB) are live together. With L kept, that is
     5.40–6.33 MB against the build's 4.50.
   - **N1's peak, phase by phase, under ideal freeing.** P⁸ is 4.03 MB at the median with L
     kept, and 3.13 with L recycled. That is below the shipped path on 196–209 of 233 blocks,
     and above it from about the p85 block (p90 6.36 MB, max 9.54).
   - P^{2|4} peaks at 2.84 MB (L kept), and P^{4|8} at 3.46.
7. **The corrected portfolio comparison.**
   - 4.23 MB left L out and counted three arrays that aren't live: 16 B final records, F
     at the last doubling, and the directory during the merge.
   - On its own build-phase boundary with L kept, P⁸ is **4.03 against 4.50 MB** at the
     median, and 6.36 against 4.50 at p90.
   - C′'s 3.79 (excluding L) becomes 4.69 with L. Reconciled, it is 4.03.
8. **No resident-memory (RSS) claim for N1 survives the way mibz2 keeps its buffers**
   (deduced).
   - Every buffer is kept from block to block (`Scratch`, `InverseBwt`). So a worker's
     footprint is its high-water mark over the blocks it meets, not the median block's
     peak.
   - The high-r tail sets N1's high-water: P⁸ 9.54 MB, P^{2|4} 6.24 (one arena, L kept),
     against 4.50 for L plus the T-vector.
   - **Per worker on master's boundary** (common buffers 3.63 MB included), when every worker
     meets the census's max block, against the shipped 7.23 MB (§4.6):
     - P² and P^{2|4}: 8.98 MB with one arena; separate buffers 9.31 and 9.60;
     - P⁴ and P^{4|8}: 11.33 MB with one arena; separate 11.76 and 12.04;
     - P⁸: 12.27 MB with one arena; 13.48 separate.
   - **Four workers:**
     - every worker meeting the max block: 35.9 (P^{2|4}, arena) to 53.9 MB (P⁸, separate),
       against 28.9;
     - the bench corpus dealt round-robin: 31.9 to 50.7 MB.
   - **A fallback must be decided before allocation.** A staged rule needs no new counter: r
     from the L scan, then the exact m₂ from the 1→2 count, checked before the merge
     allocates. On the census (§4.7) it keeps every N1 block inside the T-vector's 4n arena:
     - P^{2|4}: 94.8% of bytes on N1, no false fallbacks, no overshoot;
     - P^{4|8}: 84.1% of bytes.
     That makes N1 memory-neutral by construction, and never better.
   - What survives is a **walk-phase cache-footprint** advantage: P⁸'s table is 1.54 MB,
     P^{4|8}'s 1.24 and P^{2|4}'s 1.04, against 3.6 MB. That is §5.4's four-copy argument
     (L3 pressure), not a memory saving, and it is unmeasured.

**Revised against mibz2 `origin/master` `ae942bd`.** The first draft read the stale local
checkout `155bb99`, from before #114.

| Item | Before (`155bb99`) | Now (`ae942bd`) |
|---|---|---|
| The shipped build | 14.0 B/B (two pass) | **6.0 B/B** (one pass); 10.0 with write-allocate |
| N1 against the shipped build, median | P⁸ 1.55–1.83×, P^{4\|8} 1.24–1.54×, P^{2\|4} 0.71–0.94× | **P⁸ 3.62–4.26×, P^{4\|8} 2.89–3.60×, P^{2\|4} 1.66–2.20×** |
| The same, write-allocate on both sides | not given | P⁸ 3.07–3.61×, P^{4\|8} 2.37–2.92×, P^{2\|4} 1.35–1.77× |

**What #114 didn't change:**
- the shipped buffers;
- the peaks (§4.2);
- the corrected 4.03 against 4.50 MB (§4.3);
- the retained high-water table (§4.4).

**Unchanged because they don't touch the shipped side:**
- N1's traffic;
- the portfolio-against-C′ reconciliation.

**The stage figures** (§7.8):
- **The 7763's build_T of 2.16 c/B already includes #114.** It is PR #114's own head run.
- **The Zen 4 and Zen 5 rows** that C′ §5.1 uses are untraced.
- **No post-#114 Zen 5 stage figure exists** in the bench comments or the journals.

## 2. The two models, reproduced

- **The portfolio's layout C** was computed inline in round 20 and never saved under
  `theory/`. None of `doubling.py`, `children.py`, `census.py`, `twoviews.py`, `bvcount.py`
  or `summarise.py` produces it.
  - Those scripts check correctness (doubling, the two views, the bitvector counts) or
    summarise the census.
  - The model was recovered verbatim from the session record as
    `portfolio_layoutC_model.py`. It reproduces the table exactly: idx/idx/idx 17.0 B/B,
    peak 4.23 / 7.19 / 10.12; dup/dup/idx 17.5 B/B, 4.23 / 7.19 / 11.10.
- **C′** is `n1-analysis/model.py`, `plan_bytes`. `reconcile.py` re-implements it term by
  term and reproduces 19.5 / 20.8 / 22.5 / 37.5 / 59.6 and, with separate caps, 25.5 at the
  median.
- **Counts per output byte** (measured, census; median, with min–max):
  - r: 0.0623 (0.0533–0.200);
  - m₂: 0.0756 (0.064–0.227);
  - m₄: 0.0937 (0.078–0.252);
  - m₈: 0.1219 (0.099–0.298);
  - one bitvector pass: 0.125.

## 3. Task 1: traffic, access by access

### 3.1 What the construction must do (the reconciled access list)

The rules, for a doubling a → 2a with P = m_a parents and Q = m_2a children:
- **Level a's image view** (a u32 permutation) exists only if a → 2a is done.
- **f (and F) are needed at a → 2a only if level 2a has an image view.** That means only if
  2a → 4a is also done. Level a's image-start bitvector is needed only for f.
- **The counting pass must be a pass of its own.** E is a prefix over every parent's child
  count, and no child can be placed before E is known.
- **It can't fold into the previous merge.** e_i is a popcount over the level-a source-start
  bitvector, which that merge is still marking.
- **So the counting pass reads the parents again,** with at least {start, D}.

### 3.2 P⁸, row by row (median block)

Element sizes are given as portfolio / C′ / reconciled. "—" means the model leaves the access
out. R and W mark reads and writes. The four right-hand columns are B/B.

| Access | Element | Count per B | R/W | Portfolio C | C′ | Reconciled | Verdict |
|---|---|--:|:-:|--:|--:|--:|---|
| **Level 1** | | | | | | | |
| read L | — / 1 / 1 | 1 | R | — (8 B × r of a run list: 0.50) | 1.00 | 1.00 | boundary: the portfolio is on N3's boundary (no L); with MTF writing L, the L read is required |
| write S₁ (bucket-scattered via `counts`) | 12 | r 0.062 | W | 0.75 | 0.75 | 0.75 | agree |
| write image view 1 (L order) | 4 | r | W | 0.25 | 0.25 | 0.25 | agree |
| mark src- and image-start bitvectors | bit | 2 passes | W | in its 6 passes per doubling | 0.25 | 0.25 | agree |
| **1→2** (P = m₁, Q = m₂) | | | | | | | |
| count: read S₁ {start, D} | — / 12 / 8–12 | 0.062 | R | **—** | 0.75 | 0.50–0.75 | **portfolio omission**; 8 vs 12 is a design choice (SoA/AoS) |
| count: popcount sweeps (e, f) | bit | 2 passes | R | in its 6 passes | 0.25 | 0.25 | agree |
| count: write E (running prefix) | 4 | 0.062 | W | 0.25 | 0.25 | 0.25 | agree |
| count: write f | 4 | 0.062 | W | 0.25 ("F" as offsets) | 0.25 | 0.25 | agree in bytes |
| F prefix: read idx, gather f, write F (in place) | 4+4+4 | 0.062 | R/W | **—** | 0.75 | 0.75 | **portfolio omission** |
| merge: read idx₁ | 4 | 0.062 | R | 0.25 | 0.25 | 0.25 | agree |
| merge: gather S₁[i] | 12 | 0.062 | R | 0.75 | 0.75 | 0.75 | agree (both have it) |
| merge: gather E[i] | 4 | 0.062 | R | **—** | 0.25 | 0.25 | **portfolio omission** |
| merge: read S₁[j], source order | 12 | 0.062 | R | 0.75 | 0.75 | 0.75 | agree |
| merge: read F[j] | 4 | 0.062 | R | **—** | 0.25 | 0.25 | **portfolio omission** |
| merge: write children S₂ (group-scattered) | 12 | m₂ 0.076 | W | 0.91 | 0.91 | 0.91 | agree |
| merge: write child image view | 4 | 0.076 | W | 0.30 | 0.30 | 0.30 | agree |
| merge: mark level-2 bitvectors | bit | 2 passes | W | in its 6 passes | **—** | 0.25 | **C′ omission** |
| clear level-1 bitvectors | bit | 2 passes | W | in its 6 passes | 0.25 | 0.25 | agree |
| **2→4** (P = m₂, Q = m₄) | | | | | | | |
| count read S₂ / sweeps / E / f | 8–12; bit; 4; 4 | 0.076 | R/W | — / in 6 / 0.30 / 0.30 | 0.91 / 0.25 / 0.30 / 0.30 | 0.60–0.91 / 0.25 / 0.30 / 0.30 | as 1→2 |
| F prefix | 12 | 0.076 | R/W | **—** | 0.91 | 0.91 | portfolio omission |
| merge: idx, S₂[i], E[i], S₂[j], F[j] | 4, 12, 4, 12, 4 | 0.076 | R | 0.30, 0.91, —, 0.91, — | 0.30, 0.91, 0.30, 0.91, 0.30 | same as C′ | portfolio omits E[i] and F[j] |
| merge: write S₄, child image view | 12, 4 | m₄ 0.094 | W | 1.12, 0.37 | 1.12, 0.37 | 1.12, 0.37 | agree |
| merge: mark level-4 source starts (no image starts: 4→8 needs no f) | bit | 1 pass | W | in its 6 passes | — | 0.125 | C′ omission |
| clear level-2 bitvectors | bit | 2 | W | in 6 | 0.25 | 0.25 | agree |
| **4→8, final** (P = m₄, Q = m₈) | | | | | | | |
| count: read S₄ | 8–12 | 0.094 | R | **—** | 1.12 | 0.75–1.12 | portfolio omission |
| count: sweeps | bit | 1 (e only) | R | in its 6 passes | 0.25 (2 passes) | 0.125 | **C′ overcount**: no f at the final level |
| count: write E | 4 | 0.094 | W | 0.37 | 0.37 | 0.37 | agree |
| count: write f | 4 | 0.094 | W | 0.37 ("F") | 0.37 | **0** | **both overcount**: level 8 has no image view, so f and F aren't needed |
| merge: idx, S₄[i], E[i], S₄[j] | 4, 12, 4, 12 | 0.094 | R | 0.37, 1.12, —, 1.12 | 0.37, 1.12, 0.37, 1.12 | same as C′ | portfolio omits E[i] |
| merge: write S₈ | 16 / 12 / 12 | m₈ 0.122 | W | 1.95 | 1.46 | 1.46 | **portfolio over-sizes it**: walk format {D, 8-byte word} needs no start, since the directory gives the index. The length (8, or less at a cap) goes in D's spare bits (§3.3) |
| merge: mark B₈ | bit | 1 | W | — | — | 0.125 | C′ omission (the portfolio has no final-level bitvector) |
| clear level-4 bitvector | bit | 1 | W | in 6 | 0.25 (2) | 0.125 | C′ overcount (one bitvector, not two) |
| **Final** | | | | | | | |
| directory: read B₈, write {count, bits} per 64 rows | bit + 16 B per 64 rows | 1 + 0.25 | R/W | 0.25 | 0.375 | 0.375 | the portfolio leaves out the bitvector read |
| clear B₈ | bit | 1 | W | — | — | 0.125 | both omit it (needed when buffers are reused) |
| caps | — | ≤ 512 × 14 records | R/W | **—** | 0 (fused) or 2.93 (separate) | 0.125 (fused: one cap-bitvector popcount in the 4→8 count) to 3.02 (separate copy of S₈) | **design choice**; "fused" isn't free: the cap breaks must enter e_i |
| **Total, median** | | | | **17.0** | **22.5** | **21.7–25.6** (22.6 like-for-like with C′) | |

**The differences by cause** (`reconcile.py`, median B/B; reconciled with the 12 B count read
and fused caps, minus each model):

| Cause | vs portfolio | vs C′ | Who is right |
|---|--:|--:|---|
| Counting pass re-reads the parents | +2.77 | 0 | C′. The counts must exist before the merge. "Folding the count into the previous scatter" (§3.1(j)) removes the second join, not the pass |
| f, F prefix, F as offsets | +1.28 | −0.37 | Neither. The portfolio omits the prefix's image-order reads; both count f and F at the final doubling |
| Merge gathers E[i] | +0.92 | 0 | C′. The portfolio writes E and never reads it |
| Merge reads F[j] | +0.55 | 0 | C′. Same |
| L read against a run list | +0.50 | 0 | Boundary. The portfolio's N1 side assumes N3; its T-vector side reads L |
| Final records, 16 → 12 B | −0.49 | 0 | C′. Walk format suffices |
| Bitvector marks, sweeps, clears | −0.12 | +0.38 | The portfolio is right in total (18 passes); C′ omits the merges' marks and the final clear |
| Directory | +0.13 | 0 | C′. It reads the bitvector |
| Caps | +0.12 | +0.12 | Neither. Fused caps cost a popcount pass |
| **Net** | **+5.6** | **+0.1** | |

**The note's attribution, checked:** C′ says the difference is "explicit gathers (S_a[i] and E[i]
through the index view) and its F prefix pass". The S_a[i] gather is not a difference: the
portfolio's idx layout charges m_a·(4 + 12) for exactly that gather. The E[i] gathers and the F
prefix are differences, and so are the counting re-read (the largest), the F[j] reads and the
boundary.

**No double counting was found in C′** (deduced). It reads the level-a records three times per
doubling: in the count pass, as gathered parents, and as sequential second parents. Each read
is needed. Two of them could merge only if the count and the merge ran in the same order,
which they don't (source order against image order).

### 3.3 The other plans (median; the full quantiles are in `reconcile.json`)

| Plan | C′ (model.py) | Reconciled low–high | min / p10 / median / p90 / max (low) | Main differences from C′ |
|---|--:|--:|---|---|
| P^{2\|4} | 9.24 | **10.0–13.2** | 8.8 / 9.2 / 10.0 / 15.9 / 26.0 | the rank query's directory reads, +1.21–2.42; caps; C′ counts f and two sweeps at 1→2, where level 2 has no image view (−0.37) |
| P^{4\|8} | 16.48 | **17.3–21.6** | 15.2 / 16.1 / 17.3 / 28.6 / 45.7 | directory reads +1.50–3.00; caps; the 1→2 marks need only the source starts; C′'s 2→4 count has f and two sweeps |
| P⁸ | 22.5 | **21.7–25.6** | 18.9 / 20.1 / 21.7 / 35.7 / 56.4 | §3.2 |

**The upgrade pass, access by access** (P = m_a; reconciled):

| Access | Element | R/W | B/B, P^{2\|4} (a = 2) | B/B, P^{4\|8} (a = 4) | C′ |
|---|--:|:-:|--:|--:|---|
| read S_a {D, word} in source order | 8 | R | 0.60 | 0.75 | yes |
| scan the starts (B_a bitvector) | bit | R | 0.125 | 0.125 | yes |
| rank: 1–2 directory words {count, bits} | 16–32 | R | 1.21–2.42 | 1.50–3.00 | **omitted** |
| gather S_a[J] | 8 | R | 0.60 | 0.75 | yes |
| write U, 12 B {D with the length in its spare bits, 8-byte word} | 12 | W | 0.91 | 1.12 | yes |

**Notes on the upgrade:**
- **Two directory words** are read when the parent's image crosses a 64-row word, since the
  unsplit test needs rank(hi − 1) − rank(lo).
- **U is a separate array.** S_a[J] may itself be upgraded (C′ §4.2), and the correctness
  check has since shown that the upgrade can't be done in place.
- **Every figure here counts both arrays while both are live:**
  - **Traffic:** the upgrade reads S_a (8 B per parent), gathers S_a[J] (8 B), and writes U
    (12 B) to its own array.
  - **Peak (§4.2):** the upgrade phase holds S_a (8 m_a) and U (12 m_a) together, plus the
    bitvector and the directory.
  - **Retained (§4.6):** the separate-buffer variant puts U in the record buffer that is dead
    at the upgrade (R0 for P^{2|4}, R1 for P^{4|8}), not in S_a's. The arena variant takes the
    upgrade phase's full live set.
  - **No correction was needed.** The upgrade phase is never the peak on the census: the 1→2
    merge is larger for both plans, because m₂/r ≤ 1.13 is far below the 1.67 at which
    20m₂ would pass 20r + 8m₂.
- **The record has no field for its length** (1–8: a or 2a, or shorter at a cap), so it goes
  in D's spare bits. The check found this. D is stored as a displacement modulo n, below
  n ≤ 900,000 < 2²⁰, so a u32 has 12 spare bits; `build_one_pass` refuses n ≥ 2²⁴, which
  still leaves 8. Three bits hold the length minus one.
  - This costs no memory: the record stays 12 B.
  - It costs a mask and a shift per lookup in the walk. That is inside the model's 16–24
    instructions per lookup, and unmeasured.
  - The same applies to P⁸'s 12 B walk-format records, whose cap records carry lengths below
    8.

### 3.4 The shipped build on the same boundary (read from code, `ae942bd`)

`bwt::build` (`src/bwt.rs` at `ae942bd`, lines 241–321) calls `build_one_pass`. It falls back
to `build_two_pass` only if the counts don't sum to n, or if a bucket's cursor would pass its
end. That is never the case on the decoder's path, whose MTF stage computes the counts from L
itself.

**Case A, `build_one_pass`: the decoder's path, on every one-core build** (`Plan::Workers`,
which includes the 4-core laptop target):

| Access | Element | Count per B | R/W | B/B |
|---|--:|--:|:-:|--:|
| read L at row i, sequential (`read_u64` 16 B at a time on runs) | 1 | 1 | R | 1.0 |
| read ll[s] at each slot written: 256 bucket cursors, each sequential (16 contiguous bytes on the run path) | 1 | 1 | R | 1.0 |
| write tt[s] = ll[s] \| i << 8, whole 4 B, bucket-scattered | 4 | 1 | W | 4.0 |
| `ftab` and `lim` from `counts` | 4 × 512 | — | R/W | 0 |
| **Total, element convention** | | | | **6.0** |
| + write-allocate: each tt line read before it's written (ordinary stores, not streaming) | 4 | 1 | R | +4.0 → **10.0** |

**Case B, `coop::build_fused`** (`Plan::Teams`, from 6 physical cores):
- it moves the same 6.0 B/B, split over the team's K threads, using the MTF stage's count
  checkpoints;
- it is not the 4-core target's build.

**Case C, `build_two_pass`** (the pre-#114 build, and master's fallback for foreign counts):
- **What it moves:** pass 1 reads L and writes tt (5 B); pass 2 reads L, then reads and
  writes tt[k] (9 B). That is **14.0 B/B**.
- **Where it was used:** this is what `155bb99` shipped, and what the first version of this
  note counted.

**The other accesses:** the walk adds a START-flag read-modify-write on 512 rows
(`room_segments`). That is negligible, and belongs to the walk.

**Ratios of N1's traffic to the one-pass build** (deduced; low–high; min / p10 / median / p90 /
max of the low end):

| Plan | Element convention, median | Low end's quantiles | Both sides with write-allocate, median |
|---|--:|---|--:|
| P^{2\|4} | 1.66–2.20× | 1.46 / 1.54 / 1.66 / 2.66 / 4.34 | 1.35–1.77× |
| P^{4\|8} | 2.89–3.60× | 2.53 / 2.68 / 2.89 / 4.76 / 7.62 | 2.37–2.92× |
| P⁸ | 3.62–4.26× | 3.15 / 3.36 / 3.62 / 5.95 / 9.40 | 3.07–3.61× |

**The write-allocate convention** applies to both sides: every byte N1 writes is read first.
- **N1's totals under it:**
  - P^{2|4}: 13.5–17.7 B/B;
  - P^{4|8}: 23.7–29.2 B/B;
  - P⁸: 30.7–36.1 B/B.
- **Bitvector clears** are counted as written too, though a `memset` may avoid the read.

**Against the two-pass build (14.0)** the ratios would be P⁸ 1.55–1.83× and P^{2|4}
0.71–0.94×. That is the old comparison, and it no longer applies to the shipped decoder.

### 3.5 Two sensitivities

- **N3's boundary** (MTF emits an 8 B/run list and L is never written): P⁸ is 21.2–25.1 B/B
  at the median.
  - The shipped build has no such variant: it needs L.
  - So compare on L's boundary, and charge N3's saving to the MTF stage, where it belongs.
- **Word access** (each bit mark a 16 B read-modify-write of a word, and each random range
  popcount for e_i two 8 B edge words): P⁸ is 33.4–37.1 B/B at the median, 5.6–6.2× the
  one-pass build.
  - The shipped build's accesses are already whole elements.
  - Under this convention the bitvector work is about 40% of N1's accesses. That is a
    reason for Gate B's per-pass counters to report the marking and counting passes
    apart.

## 4. Task 2: symmetric peak memory

### 4.1 What the shipped path holds (read from code, `ae942bd`)

The buffers are the same at `155bb99` and at `ae942bd` on the one-core path, so the peak
figures below hold for both. What #114 changed is how tt is written: `reserve(n)` and then
one whole write per entry, where the old code did `extend` and then an RMW. Its size and
lifetime are unchanged.

- **Per worker, in `Scratch` (`src/lib.rs` 265 at `ae942bd`),** kept from block to block:
  - `ll`, the last column: n bytes, 0.90 MB;
  - `counts`: 1 KB;
  - `checkpoints` (new since #101/#104): the MTF stage's count checkpoints, filled only for a
    cooperative kernel. That is 1 KB per 16,384 rows, about 56 KB a block on teams, and empty
    on `Plan::Workers`;
  - the inverse kernel's buffers (`Segmented` under `Plan::Workers`):
    - `tt`: 4n, 3.60 MB;
    - `seg.bufs`: 512 segment buffers, each `reserve(2n/512 + 64)`, which is 1.83 MB
      reserved. They hold n bytes written, and grow by doubling on a long segment;
    - `seg.starts` and `seg.next`: about 6 KB;
  - `rle`, the assembled walk output: n bytes, 0.90 MB.
- **Per job, outside the kernel:** the block's symbols (`Vec<u16>`, ≤ 1.8 MB per slot) and the
  decoded output (`free_out`). Both paths hold these identically, so they are left out below.
- **L is not freed or recycled.** `invert` takes `ll: &[u8]`. So today L is "kept" for the
  whole invert, on both paths.
- **The T-vector doesn't reuse L's buffer.** It is a separate `Vec<u32>`, which `build`
  clears and extends.
- **Nothing is shared between workers.** Each worker builds its own `Scratch` (`worker()` in
  `parallel.rs`), so four workers hold four copies of everything below.
- **The plan decides which kernel runs.** `Plan::for_threads` (default since #106) plans
  `Workers(threads)` below 6 physical cores, including the 4-core laptop target. From 6 cores
  it plans `Teams { teams, k: 2 }`.
  - Under teams, `Cooperative<K>`'s K threads share one tt (`coop::build_fused`) and one
    `SegScratch`, with 512·K segments of 2n/(512K) + 64 bytes each.
  - So a team holds one table per K threads. The per-thread figures halve at K = 2.
  - This note's comparisons are per worker under `Plan::Workers`. An N1 team would share
    the same way.
- **Documented:** "each worker keeps ~7 MB of buffers", and 113 MB peak RSS at four threads
  (`2026-10-27-bz2-decoder.md` lines 162–166). Both predate #114 and the cooperative
  default; #114 left the buffers unchanged.

### 4.2 Live bytes per phase, under ideal freeing (deduced; median block, MB)

Under ideal freeing, an array counts from its first write to its last read. "L kept" adds L's
0.90 to every phase, as today's code does. "L recycled" frees it after its last read: for N1
that is level 1, for the shipped path the build. Segment buffers are counted at the bytes
written (n); the reserved figure is in brackets.

| Phase | Shipped | P^{2\|4} | P^{4\|8} | P⁸ |
|---|--:|--:|--:|--:|
| build: level 1 (L, S₁, idx₁, bitvectors) | L + tt 4.50 | 1.91 | 2.02 | 2.02 |
| 1→2 merge | | 2.68 | 3.46 | 3.57 |
| 2→4 merge | | | 3.05 | 4.00 |
| 4→8 merge | | | | 4.02 |
| directory / upgrade | | 1.78 / 2.60 | 1.91 / 2.92 | 2.55 |
| walk (table + directory + segment buffers) | 5.40 (6.33) | 2.84 (3.77) | 3.04 (3.97) | 3.34 (4.27) |
| assemble (segment buffers + rle) | 2.70 (3.63) | 2.70 | 2.70 | 2.70 |
| **Peak, L kept** | **5.40 (6.33)** | **2.84 (3.77)** | **3.46 (3.97)** | **4.03 (4.27)** |
| **Peak, L recycled** | **4.50 (5.43)** | **1.94 (2.87)** | **2.56 (3.07)** | **3.13 (3.37)** |

**Quantiles of the peak** (min / p10 / median / p90 / max), with segment buffers at n:

| Plan | L kept | L recycled | Blocks below the shipped peak |
|---|---|---|--:|
| shipped | 5.40 on every full block | 4.50 | — |
| P^{2\|4} | 2.72 / 2.77 / 2.84 / 3.98 / 6.24 | 1.82 / 1.87 / 1.94 / 3.08 / 5.34 | 232 / 233 |
| P^{4\|8} | 3.09 / 3.23 / 3.46 / 5.35 / 8.59 | 2.19 / 2.33 / 2.56 / 4.45 / 7.69 | 210 / 233 |
| P⁸ | 3.52 / 3.76 / 4.03 / 6.36 / 9.54 | 2.62 / 2.86 / 3.13 / 5.46 / 8.64 | 196 / 233 |

**What sets each peak:**
- **The shipped path peaks during the walk.** The T-vector (3.6 MB) and the segment buffers
  are live together. The 4.5 MB figure is its build phase only.
- **P⁸ peaks at its last two merges,** which are within 0.02 MB of each other.
- **P^{4|8} peaks at its 1→2 merge.** Its 2→4 merge writes 8 B records with no image view,
  and needs no F.
- **P^{2|4}'s walk phase is its largest.**

**What each N1 phase holds at its last merge** (P⁸, 4→8):
- S₄ (12 m₄);
- idx₄, E₄ (4 m₄ each);
- S₈ in walk format (12 m₈);
- B₈ (n/8).

It holds no F (level 8 has no image view) and no directory yet.

**Recycling L** saves up to 0.90 MB on both paths:
- **N1:** the arrays live at P⁸'s peak that fit in n bytes (idx₄, E₄ and B₈, 0.79 MB) can
  take L's buffer.
- **Shipped:** `rle` can take it.
- **Neither exists today.** Both need the trait or `Scratch` to hand L's buffer over.

### 4.3 The portfolio's 4.23 against 4.5 MB, corrected

This is unchanged at `ae942bd`. L plus tt is still 0.90 + 3.60 MB during the build.


**The portfolio's 4.23 MB at the median** (deduced from its model) is made of:
- S₄ 1.01;
- idx₄ 0.34;
- E₄ + F₄ 0.67;
- S₈ at 16 B 1.76;
- two bitvectors 0.23;
- the directory 0.23.

**Against the reconciled build-phase peak:**
- **leaves out L:** +0.90;
- **F₄, not needed at the final level:** −0.34;
- **16 → 12 B final records:** −0.44;
- **the directory, not live during the merge:** −0.23;
- **one live bitvector, not two:** −0.11.
- **Result:** 4.03 MB with L kept, against the shipped build phase's 4.50 (ll + tt).

**On the portfolio's own build-phase boundary:**

| Plan | L kept | L recycled | Blocks below 4.50 (kept / recycled) |
|---|---|---|---|
| P⁸ | 3.52 / 3.76 / **4.03** / 6.36 / 9.54 | 2.62 / 2.86 / 3.13 / 5.46 / 8.64 | 168 / 196 |
| P^{4\|8} | 3.09 / 3.23 / **3.46** / 5.35 / 8.59 | 2.19 / 2.33 / 2.56 / 4.45 / 7.69 | 196 / 210 |
| P^{2\|4} | 2.43 / 2.53 / **2.68** / 3.98 / 6.24 | 1.78 / 1.83 / 1.91 / 3.08 / 5.34 | 221 / 232 |

**C′'s 3.79 MB (P⁸, excluding L)** becomes 4.69 with L kept. It also counts F₄ and four
bitvectors at the last merge, so the reconciled figure is 4.03. C′'s §4.2 peaks are high for
every plan by the same items: P^{4|8} 2.76 against 2.56 reconciled, excluding L.

### 4.4 Retained footprint: what the worker actually keeps (deduced)

This is unchanged at `ae942bd` under `Plan::Workers`: the same buffers, kept the same way.


**What a worker keeps.** mibz2 keeps every buffer from block to block, and the trait demands
it. So a worker's footprint is the sum of its buffers' high-water capacities over all the
blocks it has met. It is not one block's peak.

**Per worker, L plus the kernel's own arrays** (segment buffers and `rle` are common to both
paths: 1.8–2.7 MB more):

| Discipline | Shipped (L + tt) | P^{2\|4} | P^{4\|8} | P⁸ |
|---|--:|--:|--:|--:|
| median block only, one arena, L kept | 4.50 | 2.68 | 3.46 | 4.03 |
| corpus high-water (the max block), one arena, L kept | 4.50 | 6.24 | 8.59 | 9.54 |
| corpus high-water, separate buffers per role (ping-pong records and views, E, F, U) | 4.50 | 6.87 | 9.31 | 10.69 |
| the same, with a T-vector fallback met in the same worker (tt kept as well) | 4.50 | 10.47 | 12.91 | 14.29 |
| one arena shared with the T-vector, selector sends any block over 4.50 MB to the T-vector | 4.50 | 4.50 | 4.50 | 4.50 |

**The reading:**
- **A worker over a dump meets the high-r blocks.** The census's p90 r is about 99,000, and
  its highest are blocks 145 (r = 179,972), 90 (136,178), 33 (134,923), 150 (132,011) and 24
  (131,363). A worker that has met them keeps their arrays.
- **A budgeted selector makes N1 memory-neutral, not better.** Any block sent to the
  T-vector restores the shipped footprint, so the high-water can't fall below it.
- **Freeing per block would cost page faults** (model). About 900 faults per 3.6 MB at 4 KB
  pages, at 0.25–1 µs each, is 0.2–0.9 ms per block. That is about 0.5–2.5 c/B at
  2.5–3 GHz, the same order as the build itself.

### 4.5 Four workers (deduced; `Plan::Workers`, four copies, nothing shared)

| | Shipped | P⁸, median block only | P⁸, corpus high-water | P⁸, budgeted shared arena |
|---|--:|--:|--:|--:|
| L + kernel arrays, × 4 | 18.0 MB | 16.1 | 38.2 | 18.0 |
| + segment buffers and `rle`, × 4 | 7.2–10.9 MB | same | same | same |

**At the laptop scale (16 GB):** the inverse-BWT buffers are a small part of the documented
113 MB at four threads. A saving of 0.5–2 MB per worker would be 2–8 MB.

### 4.6 Retained capacity per worker, every plan, on master's decoder boundary (deduced)

**The boundary:** what a worker keeps under `Plan::Workers` at `ae942bd`.
- **Common to both paths:** `ll`, `counts`, the segment buffers at their reserved
  512 × (2n/512 + 64), and `rle`. That is 3.63 MB. It is a floor: the segment buffers grow by
  doubling on a long segment.
- **The kernel's buffers:** the T-vector, or N1's arrays.
- **Left out:** the per-job symbols and output, which are the same on both paths.

**The rules:**
- **Every buffer is kept from block to block.** So a buffer's capacity is its maximum over the
  blocks the worker has met.
- **One arena:** one allocation for the kernel, sized at the largest phase-live set of any
  block met.
- **Separate buffers:** each role keeps its own high-water capacity. The roles are the records
  R0/R1, the image views X0/X1, E, F, two bitvectors, the directory, and U in the dead record
  buffer. Their maxima can come from different blocks, so this is never below the arena.
- **The shipped path** is 3.60 + 3.63 = **7.23 MB per worker, 28.9 MB for four** (full blocks).

**Per worker, MB** (kernel high-water + 3.63 common; `retained.py`):

| Plan | Median block alone, arena / separate | Every worker meets the max block: arena / separate | Over shipped (7.23) |
|---|---|---|--:|
| P² | 5.41 / 5.74 | **8.98 / 9.31** | +1.75 / +2.08 |
| P^{2\|4} | 5.41 / 5.89 | **8.98 / 9.60** | +1.75 / +2.37 |
| P⁴ | 6.19 / 6.57 | **11.33 / 11.76** | +4.10 / +4.53 |
| P^{4\|8} | 6.19 / 6.77 | **11.33 / 12.04** | +4.10 / +4.81 |
| P⁸ | 6.76 / 7.63 | **12.27 / 13.48** | +5.04 / +6.25 |

**What sets the arena figures:**
- **P^{2|4}'s arena equals P²'s.** The upgrade's phase (8 m₂ + 12 m₂) is never the largest;
  the 1→2 merge is.
- **P^{4|8}'s arena equals P⁴'s** for the same reason.
- **Their separate-buffer figures differ.** U needs room in the dead record buffer.

**Four workers, MB:**

| Plan | Every worker meets the max block: arena / separate | Census dealt round-robin in stream order: arena / separate | Dealt at random, median (p10–p90), arena |
|---|---|---|---|
| shipped | 28.9 | 28.9 | 28.9 |
| P² | 35.9 / 37.3 | 31.9 / 33.2 | 31.9 (31.5–32.2) |
| P^{2\|4} | 35.9 / 38.4 | 31.9 / 34.5 | 31.9 (31.5–32.2) |
| P⁴ | 45.3 / 47.1 | 39.6 / 41.5 | 39.7 (39.2–40.1) |
| P^{4\|8} | 45.3 / 48.2 | 39.6 / 43.1 | 39.7 (39.2–40.1) |
| P⁸ | 49.1 / 53.9 | 44.7 / 50.7 | 44.6 (44.0–45.3) |

**The "realistic" case and what it assumes:**
- **The assumption:** the census's 233 full blocks are dealt to 4 workers, about 58 each. That
  is one bench-corpus decode.
  - Round-robin in stream order stands for a shared job queue with equal block times.
  - Random dealing (2,000 deals, seed 1) shows the spread.
- **Every worker meets a high-r block.**
  - The chance that 58 random blocks include one of the 5 highest-r blocks is 0.72.
  - The chance they include one of the 23 highest (the top 10%) is 1.00.
  - **High-r blocks cluster in the stream.** Of the eight highest, blocks 144, 145, 150 and
    151 are neighbours. Round-robin spreads them, so each worker gets one of the top eight.
  - The least-loaded worker's arena under round-robin is still 7.41 MB for P^{2|4}, above the
    shipped 7.23.
- **A dump has about 2 million blocks** (1.83 TB of JSON / 0.9 MB). There every worker meets
  the census's maximum rank many times over, so "every worker meets the max block" is the
  realistic case for a release. If the dump's tail goes beyond the census's maximum
  (r = 179,972), the figures rise with it.

**So no plan's retained capacity is at or below the shipped path's** (deduced, on the census),
in either variant, without a fallback rule. P^{2|4} is the closest: +1.75 MB per worker
(arena) when every worker meets the max block, and +3.0 MB for four in the bench-corpus
case. The selector (§4.7) is what can bound it.

### 4.7 Deciding the fallback before allocating (census; deduced)

**The guardian's point holds.** Choosing the T-vector after building a large table doesn't
bound retained memory. The capacity is already taken and kept. The decision has to come before
the allocation it would avoid.

**What is known when:**
- **r is the number of L's runs.** It equals the block's explicit MTF tokens: measured on 20
  sampled blocks (`data/mtf.json`, 20 / 20 equal to the census's r). Those are the symbols
  other than RUNA, RUNB and EOB.
  - So r is countable in the MTF stage, or in `Symbols.syms`, before L is written.
  - Neither counts it today: `mtf()` returns n; `Symbols` has no such count.
  - The level-1 L scan counts it exactly as a by-product.
- **m₂ is known after the 1→2 counting pass** (Σe_i, E's final value), before any level-2
  array is written.
- **m₄ is known after the 2→4 count.**
- **The proven bounds:**
  - |C₂ₐ| ≤ 2|C_a|, so m₂ ≤ 2r − 1 and m₄ ≤ 2m₂ − 1;
  - so P^{2|4}'s arena ≤ 40r + n/8 + dir, and P^{4|8}'s ≤ 80r + n/8 + dir.

**Two budgets on the kernel arena:**
- **B36: arena ≤ 4n = 3.60 MB.** This is the retained boundary. With one arena shared with the
  T-vector, no block can grow it. It is the same as "L + kernel ≤ 4.50 MB".
- **B45: arena ≤ 4.5 MB.** This is the per-block, ideal-freeing equivalent of the shipped
  walk-phase peak (L + tt + segment buffers = 5.40 MB).
  - It isn't a retained bound. Under retention the segment buffers and `rle` are kept on both
    paths and cancel, so the equivalent boundary is the arena against tt, which is B36.

**Rules decided on r before level 1** (233 blocks):

| Plan, budget | Rule | Admitted to N1 | False fallbacks (would have fitted) | Bytes on N1 | Census blocks admitted but over | Census worst margin | Overshoot if m_k reached its bound |
|---|---|--:|--:|--:|--:|--:|--:|
| P^{2\|4}, B36 (221 blocks fit, 94.8% of bytes) | safe: 40r + n/8 + dir ≤ 4n (r ≤ 79,375) | 196 | 25 (11.3%) | 84.1% | 0 | −1.13 MB | none (proven) |
| | calibrated: r × 32.1 B ≤ 4n (r ≤ 110,566) | 218 | 3 (1.4%) | 93.6% | 0 | −0.19 MB | +1.16 MB |
| | the census's best threshold (r ≤ 117,195) | 221 | 0 | 94.8% | 0 | −0.01 MB | +1.43 MB |
| P^{2\|4}, B45 (232 fit, 99.6%) | safe (r ≤ 102,105) | 212 | 20 (8.6%) | 91.0% | 0 | −1.37 MB | none |
| | calibrated (r ≤ 136,178) | 232 | 0 | 99.6% | 0 | −0.33 MB | +1.28 MB |
| P^{4\|8}, B36 (196 fit, 84.1%) | safe: 80r + … ≤ 4n | **0** (r would have to be ≤ 41,000; the census's minimum is 47,936) | 196 (100%) | 0% | 0 | — | none |
| | calibrated: r × 46.1 B ≤ 4n (r ≤ 74,024) | 195 | 1 (0.5%) | 83.7% | 0 | −0.26 MB | +2.66 MB |
| P^{4\|8}, B45 (210 fit, 90.1%) | safe (r ≤ 52,026) | 44 | 166 (79%) | 18.9% | 0 | −2.11 MB | none |
| | calibrated (r ≤ 96,981) | 206 | 4 (1.9%) | 88.4% | 0 | −0.13 MB | +3.60 MB |

**Why the calibrated rule works so well:**
- **The peak is almost exactly linear in r.** corr(peak, r) is 0.9998 for P^{2|4} and 0.9997
  for P^{4|8}.
- **The bytes per run are narrow.** P^{2|4} takes 29.7–32.1 B per run; P^{4|8} 42.8–46.1.
- **The calibrated rule** uses the census's largest bytes per run.
- **Its risk is out of sample:** a block with more level-2 (or level-4) records per run than
  any census block. If m_k reached its proven bound, it would overshoot by the last column.

**A staged rule removes both the risk and most of the cost** (deduced). It needs no new
counter.
1. **Build level 1 into the shared 4n arena.**
   - Level 1's arrays (16r + n/8 ≤ 3.0 MB at the census's maximum r) always fit.
   - The L scan yields r exactly, as it already must.
2. **After the 1→2 counting pass,** m₂ is known exactly (E's final sum). So the merge's and
   the upgrade's live sets are known exactly before they're allocated: 20r + 8m₂ + n/8, and
   20m₂ + n/8 + dir.
   - If either exceeds the budget, abandon N1 and build the T-vector in the same arena from
     L, which is still there.
   - For P^{4|8}, check again after the 2→4 count, with m₄.
3. **The result on the census at B36:**
   - **P^{2|4}:** 221 blocks on N1 (94.8% of bytes), zero false fallbacks, zero overshoot.
     The worker's retained capacity stays the shipped 7.23 MB. So the memory question is
     neutral by construction.
   - **P^{4|8}:** 196 blocks on N1 (84.1% of bytes), on the same terms.

**What the staged rule costs:**
- **Wasted work** on each block that falls back: level 1 plus one counting pass. That is 12
  blocks (5.2% of bytes) for P^{2|4}, and 37 blocks for P^{4|8} at B36.
- **Its time is unmeasured.** It is bounded by those blocks' level-1 and count cycles, a
  fraction of their build.
- **A pure r rule before level 1** would avoid that waste, but needs a new counter: one
  increment per explicit symbol in the MTF stage (or in the scanner). It then trades 3 false
  fallbacks and an unbounded out-of-sample overshoot (calibrated), or 25 false fallbacks and
  no overshoot (safe).

### 4.8 Does any memory-efficiency claim survive?

| Claim | Survives? | Under which accounting |
|---|---|---|
| A smaller peak during the build | Yes at the median block: P⁸ 4.03 against 4.50 (L kept), 3.13 (recycled); P^{2\|4} 2.68. No above about p72 (L kept) or p84 (recycled) for P⁸, and p95 for P^{2\|4} | per block, ideal freeing |
| A smaller peak over the whole invert | Yes at the median, by more than the build phase suggests: −1.37 MB (P⁸) to −2.56 MB (P^{2\|4}), because the shipped walk holds tt and the segment buffers together | per block, ideal freeing, L kept or recycled alike |
| A smaller resident footprint per worker (RSS) | **No.** Every plan's retained capacity exceeds the shipped 7.23 MB per worker without a selector (§4.6). With the staged rule of §4.7 and an arena shared with the T-vector, it is held at exactly 7.23: neutral, never below | mibz2's buffer discipline, every variant |
| A smaller working set during the walk | **Yes:** 1.04 (P^{2\|4}), 1.24 (P^{4\|8}) and 1.54 (P⁸) MB of table and directory, against 3.6. Unmeasured as a speed effect | cache footprint, the four-copy argument of C′ §5.4 |

So **the only memory-side benefit is cache residency during the walk.** It belongs to Gate C's
four-copy question, and isn't a memory saving.

## 5. Which model was wrong where (summary)

| Item | Portfolio C | C′ | Reconciled |
|---|---|---|---|
| Counting pass's parent read | omitted (−2.77) | right | needed; 8 or 12 B |
| F prefix, image-order reads | omitted | right | needed at nonfinal levels |
| E[i] gathers, F[j] reads | omitted | right | needed |
| f and F at the final doubling | counted (F) | counted (f, F and a sweep) | not needed |
| Final record | 16 B (wrong) | 12 B | 12 B |
| Merge bitvector marks | counted | omitted | needed |
| L read | absent (N3 boundary) | counted | counted on L's boundary |
| Caps | absent | 0 or 2.93 | 0.125–3.02, a design choice |
| Upgrade's rank reads | — | omitted | 16–32 B per parent |
| Shipped build (`ae942bd`, one pass) | 5.0 B/B (1.0 low: no ll[s] read) | not counted ("1 entry per byte") | 6.0 B/B (10.0 with write-allocate); 14.0 only for the two-pass fallback |
| Peak: L on N1's side | left out | flagged, left out of its figure | counted |
| Peak: the shipped path's peak phase | build only (4.5) | build only | walk: 5.40–6.33 with L kept |
| Peak: arrays not live | F₄, the directory, 16 B S₈ | F₄, two extra bitvectors | removed |
| Footprint over blocks | not considered | not considered | high-water, set by the tail |

## 6. Measured, deduced, estimated

- **Measured:**
  - the census counts (233 blocks): n, r, m₂, m₄, m₈;
  - the documented ~7 MB per worker and 113 MB at four threads (earlier version);
  - C′'s sample lookups, used only in the Gate B revision.
- **Read from code (`ae942bd`):** `build_one_pass`'s accesses and its fallback;
  `build_fused`'s use under teams; `Scratch` and `SegScratch`'s buffers and their reuse;
  per-worker ownership; the plan rule; segment-buffer reservation of 2n/512 + 64 bytes each.
  The two-pass build's 14.0 B/B is read from `155bb99` and from master's fallback.
- **Deduced:**
  - every byte and MB figure above, from the counts and the access lists;
  - the differences by cause;
  - that the count pass can't fold into the previous merge;
  - the retained high-water and the neutrality under a budgeted selector.
- **Estimated (model):**
  - **segment-buffer residency** (n written to 1.83 MB reserved; the allocator decides);
  - **the page-fault cost** of freeing per block;
  - **the cost of fused caps** (one popcount pass over a cap bitvector);
  - **the upgrade's rank cost** of 1–2 directory words. That depends on how often an image
    crosses a 64-row word. It wasn't measured.

## 7. Assumptions, and which would reverse a conclusion

1. **Ideal freeing.**
   - **What rests on it:** every per-block peak in §4.2.
   - **If it fails:** with mibz2's retained buffers, the peak advantage can't be banked (§4.4).
     This is the assumption that decides the memory question, and it fails for today's code.
2. **The segment buffers are a separate allocation, live during the shipped walk.** True in
   the code. If the shipped walk wrote into L's dead buffer, its walk phase would fall to
   4.50 and N1's per-block advantage would shrink by 0.9 MB.
3. **The element-size convention.**
   - **The shipped one-pass build's 6.0 B/B** counts each access at its element size. With
     write-allocate it is 10.0, and N1 rises by its write volume (§3.4).
   - **On a cache-line view** both totals change, and this model doesn't capture how:
     write-allocate, and which arrays stay in L2 or L3. N1's arrays are mostly L2/L3-sized;
     the T-vector is not.
   - **The ratios of §3.4** are byte ratios, not cycle ratios. That is consistent with both
     source notes: bytes don't decide.
4. **The design choices:**
   - SoA against AoS for the count read: 0.93 B/B for P⁸;
   - fused against separate caps: 2.9 B/B.
   Neither changes a conclusion.
5. **Caps fused into the 4→8 count.** Argued, not checked. The counting gate (portfolio
   §3.1(k)) counted uncapped partitions only.
6. **The upgrade's correctness** is still argued, not brute-force checked (C′ §8).
7. **The census is the bench's 233 blocks.** A dump's r distribution beyond them isn't
   measured. The retained high-water depends on its tail.

8. **Which CI stage figures include #114** (checked against the bench comments):
   - **C′ §5.1's EPYC 7763 row includes #114.** Its 1.23 / 2.29 / 2.16 / 6.69 / 12.93 c/B is
     PR #114's own head column (`f121097`, run 40970005807, 3239 MHz). There the build went
     from 2.31 to 2.16 c/B, −6.0% (−8.5%, −3.5%).
   - **Two later runs also include #114:**
     - PR #116, base `7e7cce4`, 7763: build 2.12, walk 5.31 c/B;
     - PR #115, base `1291566`, 9V74: build 1.75, walk 4.77 c/B.
   - **C′'s Zen 4 (1.88 / 7.01) and Zen 5 (1.52 / 4.18) rows match no saved bench comment.**
     The saved 9V74 tables give builds of 1.56–1.75 c/B, and the 9V45 tables 0.83–1.50. They
     are untraced, so treat them as unverified.
   - **No post-#114 9V45 (Zen 5) stage table was found.** The latest is PR #107 (base
     `8ed29b0`, pre-#114).
   - **The instruction counts are pre-#114:** the i9 profile's 6.65 instructions per byte
     for the build (C′ §5.1, §8) is the two-pass build's. The walk's 16.9 is unaffected.
   - **The same runner model varies from run to run.** The 7763's walk spans 3.5–7.6 c/B
     across the saved runs, at 2.45–3.26 GHz. So the Gate B draft takes its thresholds from
     the same run only, in ns per decoded byte.

## 8. Negative findings and limits

- **The portfolio's layout-C model was never saved as a script.** Its figures were
  reproducible only from the session record (now `portfolio_layoutC_model.py`).
- **The portfolio's §3.1(j) claim, "folding the counting pass is possible without a second
  join", is true but was costed as folding the pass away.** The pass remains, with a read of
  every parent.
- **Neither source accounted for the shipped path's peak phase, which is the walk,** or for
  buffer retention.
- **No memory claim for N1 survives at the RSS level.**
- **The first draft read the stale `155bb99`** and counted the two-pass build (14.0 B/B). On
  master, the shipped build moves 6.0 B/B. N1's traffic disadvantage is as large as the
  portfolio recorded (P⁸ 3.6–4.3×), not the 1.6–1.8× the first draft gave.
- **Not resolved here:**
  - whether a smaller walk working set helps at four copies (Gate C);
  - the real segment-buffer residency (a footprint-ledger run, `trace` feature);
  - the frequency of 64-row crossings in the upgrade's rank.

## 9. Files

All in `/tmp/claude-0/-home-user/a3c6f4e5-fac5-5911-974c-cd24e5aa55fb/scratchpad/n1-reconcile/`:
- **`reconcile.py`:** both models reproduced, the reconciled access list, the shipped build,
  and the phase and retained peaks. `python3 -I reconcile.py data/census.json reconcile.json`,
  under a second.
- **`reconcile.json`:** its output, all quantiles.
- **`portfolio_layoutC_model.py`:** the portfolio's round-20 model, recovered verbatim. Run it
  from the scratchpad directory.
- **`aggregate.py` and `aggregate.json`:** corpus-weighted against median-block elements,
  per-plan budgets, and the highest-r blocks, for the Gate B revision.
- **`gate-b-criteria-revision.md`:** the draft replacement for C′ §6's Gate B sketch.
- **`retained.py` and `retained.json`:** retained capacity per worker and for four workers,
  every plan, arena and separate buffers (§4.6).
- **`prealloc_rule.py` and `prealloc_rule.json`:** the before-allocation fallback rules on the
  census (§4.7).
- **`gate_ns.py` and `gate_ns.json`:** Gate B's whole-corpus ns/B thresholds from the labelled
  CI runs.

Inputs: `data/census.json`, `n1-analysis/sample.json` (lookups only), mibz2 `origin/master`
`ae942bd` (`src/bwt.rs`, `src/lib.rs`, `src/parallel.rs`, `src/coop.rs`, read with `git show`).
Also `155bb99` (the stale local checkout), for the two-pass build only.
