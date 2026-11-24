# Critique of Neon (movement) cycle 3, the closure problem (34efa5ed..36e9dbca)

*An independent reviewer, read-only. It recomputed the figures on blocks 0, 2 and 85; its check script is appended. Attribution on all 9 commits is clean.*


Range 34efa5ed..36e9dbca (9 commits). Attribution: all author/committer "Louis Maddox <louismmx@gmail.com>"; no AI trailers, model names or session links in any message. Clean.

## Bottom line
A partial result on the closure problem, and a weaker one than claimed. Nothing brings total Q below h1. With the docs' own figures, acquisition plus the emit links (CL4 table) gives **1.55–4.99 h1**: block-0 142,917 + 29,295 = 4.90 h1; block-2 4.99; block-85 1.92; block-143 3.14; block-210 1.55; block-207 2.70. That total charges the sweeps nothing and assumes flat payloads. The emit links alone are 0.67–1.08 h1, never at or below h1/2. Uncapped M-17 alone already gives 0.76–1.20 h1 on the same accounting.

## Recomputed figures (block-0; blocks 2 and 85 agree)
My script recomputes CL1 independently, without payloads: `chk/check.py`.

| Metric | M-17 (1 sweep) | CL1 (2 sweeps) |
|---|---:|---:|
| Read schedule, exact +1 | 2,148 (= B_P) | S1 1,159 + S2 859 (S2 c16 753) |
| Output map, exact +1 over all m rows (ChatGPT's metric) | 52,319 | **54,438** |
| Output map, exact +1 over survivors only | 19,467 | 17,562 |
| q_min of output (CL2) | 6,658 | 6,373 |
| Emit links | 32,833 | 29,295 = valleys + 1 (29,294 valleys) |

## Errors and overclaims
1. **"44–133× less fragmented" compares different things.** CL1's 650–1,939 is the *read* schedule of sweep 2, and that schedule is the original R by construction. M-17's 52,319 is the *output* map that the next composition would read. M-17's own read schedule is also B_P = 2,148. On the same metric (the output map), CL1 is *more* fragmented: 54,438 against 52,319. Restricted to survivors it is 10% less, which is just the link ratio. c16 against exact barely matters (753 against 859); the distortion comes from comparing reads with outputs. CL2's q_min(R_2) ≈ q_min(R_gs) agrees. **The map after composition does not stay compact. Closure is not achieved.** CL1 never composes the composed map: it runs a second first-order sweep in the other direction.
2. **CL1 is not new.** Ascending Gauss–Seidel on descents, then descending on ascents, contracts each arch from valley to valley. That is the N-20 C4 unimodal-arch / SA-IS LMS cut, already in PRIOR-ART.md, now applied to the retained map R. The survivors are exactly the valleys of the cycle, so links ≈ m/3, the expected number of peaks in a random permutation (0.320–0.330 m measured). That explains the "universal 0.326 h1 floor" at L_min=2, where m = h1. It is a known statistic, not a floor.
3. **The "8m zero-copy DAG" defers the work; it does not remove it.** Emitting visits every composition node: 89,873 on block-0, plus 835–871k internal nodes in total per block (JSON `emit_dag_internal_visits`). That is about one dependent pointer load per output byte. CL4's "sequential DAG expansion of n bytes" is false. With the DAG, emit Q ≥ links + (m−1), which exceeds walking R.
4. **CL2 theorem.** The proof is correct and short: +1 segments are intervals of target values, and no descent of T⁻¹ can lie inside one. It is the textbook rising-sequence count, 1+des(π⁻¹) (Bayer–Diaconis 1992), and the doc gives it no such citation. "Representation-independent" is too strong: it assumes an ascending sweep in source order, exact +1 steps and the fixed target layout. A different sweep order, a relaid target array or a tolerance breaks it. "Third-composition wall" also overreads it: q_min(R_2) = 6,373 = **0.18 h1** on block-0. An oracle third sweep would therefore be cheap; the real cost is acquiring the rising-sequence labelling, which is not charged.
5. **CL4.** The "Q_acq_glue = m saved" is measured against the team's own Python strawman in `common.py`, which builds R[0..m), not against Open. Its own acquisition is 56.5k–163.7k probes, **0.88–4.07 h1**, mostly dependent binary searches over strip endpoints. That cost dominates every block. Its "satisfies all three requirements of §5" does not follow from finding 1.
6. **CL3.** The L_min sweep is a sound negative: eager thin-strip contraction costs 3.2–5.5× more probes than the rows it saves. κ-FAS gives −15.5% links on synthetic i.i.d. hex only, with raw LF; it does not transfer to 256-byte alphabets and was not tested on Wikidata. Theorem O comes from record 116.
7. **P is understated.** P = 2 counts only the sweeps. Acquisition adds a stream over the runs, strip contraction rounds, the merge pass and the emit walk.

## Item table
| Item | New? | Q (block-0) | D | P | S | Verdict |
|---|---|---|---|---|---|---|
| CL1 two-sweep | No (N-20 C4 arches on R) | m−1 reads, cursor-predictable (~2k segments); emit 0.83 h1 links + 2.56 h1 DAG nodes | emit chain 29k + DAG depth | 2 sweeps + emit | 4m next + 8m DAG + 25 KB schedule (≈1.25 MB) | −10.8% links against M-17; **no closure** |
| CL2 Thm | Known (rising sequences) | n/a | — | — | — | Correct, trivial, overread as a "wall" |
| CL2 R² under (v, v_R) | Mild | 6.3k segments (2.9× B_P) | — | — | Classes grow as \|Σ\|^k | Most interesting number; not costed |
| CL3 κ-FAS | Confirms rec. 116 | hex only | — | 1 | 256 B + 16 KB M3 | Niche |
| CL3 L_min sweep | Useful negative | Acquisition 3.3–45 h1 | log (binary search) | — | — | Sound |
| CL4 endpoint schedule | Bookkeeping | 0.88–4.07 h1 acquisition | binary searches | ≥3 | 24–65 KB | Correct; "glue saved" is against a strawman |

## Byte-exact checks
`T_ref` is a textbook LF walk in the same `common.py`, sharing `build_lf_array` and `compute_counts_and_runs` with the code under test. On the synthetic blocks `T_ref` is asserted equal to the true text, which anchors the LF convention. On the admitted blocks nothing is checked against an external decoder or against bzip2 RLE1/CRC. This is adequate for inverse-BWT exactness but not independent of the shared LF code. The multi-cycle cases (n ≤ 120, raw LF) use the same helper.

## Most promising thread
Acquisition is the binding term (0.9–4 h1), not composition. Emission is pinned at about m/3 links by valley statistics, so the lever is m. The cheapest m available (L_min=2) blows up the eager strip probes. The thread worth pursuing is lazy or shared acquisition for thin runs, for instance reusing the B_P cursor intersection inside strip contraction itself, charged in full. Second: the (v, v_R) depth-2 class schedule (6.3k segments), costed with payloads.

## Appendix: check.py

```python
import sys, time, json
sys.path.insert(0, sys.argv[1])
import common, cl1_mirror_factored_closure as cl1
blks = common.load_admitted_blocks()
names = sys.argv[2].split(',')
for blk in blks:
    if blk.name not in names: continue
    t=time.time()
    ret = common.build_retained_system_genuine(blk, L_min=17)
    v = [blk.L[x] for x in ret.rows]
    R = ret.R; m = ret.m; mark = ret.mark_rank
    # independent recompute of CL1 two sweeps (no payloads)
    nxt = R[:]
    for j in range(m):
        y=R[j]
        if y<j and y!=mark: nxt[j]=nxt[y]
    for j in range(m-1,-1,-1):
        y=R[j]
        if y>j and y!=mark: nxt[j]=nxt[y]
    # survivors along walk
    surv=[]; cur=mark
    while True:
        surv.append(cur); cur=nxt[cur]
        if cur==mark: break
    # walk sequence of R
    seq=[]; cur=mark
    while True:
        seq.append(cur); cur=R[cur]
        if cur==mark: break
    assert len(seq)==m
    k=len(seq)
    # survivors predicted: mark + rows x whose predecessor-in-walk ... count local minima
    valleys=sum(1 for i in range(k) if seq[i]<seq[i-1] and seq[i]<seq[(i+1)%k])
    peaks=sum(1 for i in range(k) if seq[i]>seq[i-1] and seq[i]>seq[(i+1)%k])
    sset=set(surv)
    # metrics: exact +1 per v-label of composed map restricted to survivors (what sweep 3 reads)
    def seg(targets, labels, idx):
        last={}; s=0
        for j in idx:
            c=labels[j]; t=targets[j]
            if last.get(c) is None or t!=last[c]+1: s+=1
            last[c]=t
        return s
    def c16(targets, labels, idx, step=1):
        last={}; s=0
        for j in idx:
            c=labels[j]; t=targets[j]
            if c not in last or not (1 <= (t-last[c])*step <= 16): s+=1
            last[c]=t
        return s
    D=[j for j in range(m) if R[j]<j and R[j]!=mark]
    A=[j for j in range(m-1,-1,-1) if R[j]>j and R[j]!=mark]
    survs=sorted(sset)
    out={
     'name':blk.name,'m':m,'h1':blk.h1,'B_P':seg(R,v,range(m)),
     'links':len(surv),'links/h1':round(len(surv)/blk.h1,3),'links/m':round(len(surv)/m,4),
     'valleys':valleys,'peaks':peaks,
     'S1_read_exact':seg(R,v,D),'S1_read_c16':c16(R,v,D),
     'S2_read_exact_desc': sum(1 for _ in []) ,
     'S2_out_exact_all':seg(nxt,v,range(m)),'S2_out_c16_all':c16(nxt,v,range(m)),
     'S2_out_exact_survivors':seg(nxt,v,survs),'S2_out_c16_survivors':c16(nxt,v,survs),
     'M17_read_exact':seg(R,v,range(m)),
     'secs':round(time.time()-t,1)}
    # S2 read in descending order, step -1
    last={}; s=0
    for j in A:
        c=v[j]; t=R[j]
        if c not in last or t!=last[c]-1: s+=1
        last[c]=t
    out['S2_read_exact_desc']=s; out['S2_read_c16_desc']=c16(R,v,A,-1)
    print(json.dumps(out)); sys.stdout.flush()
    # M-17 uncapped GS, survivors-restricted output schedule
    n2=R[:]
    for j in range(m):
        y=n2[j]
        if y==j or y==mark: continue
        n2[j]=n2[y]
    sv=[];cur=mark
    while True:
        sv.append(cur);cur=n2[cur]
        if cur==mark:break
    print(json.dumps({'M17_links':len(sv),'M17_out_exact_all':seg(n2,v,range(m)),'M17_out_exact_survivors':seg(n2,v,sorted(sv))}))
```
