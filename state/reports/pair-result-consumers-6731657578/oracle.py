"""Untimed mathematical checks, not a decoder or candidate implementation."""
import itertools,json,bisect

def model(L):
    n=len(L); P=sorted(range(n),key=lambda q:L[q]); F=sorted(L)
    starts=[0]+[i for i in range(1,n) if L[i]!=L[i-1]]+[n]
    runs=[]; counts={c:L.count(c) for c in set(L)}
    cur={c:sum(v for k,v in counts.items() if k<c) for c in counts}
    for p,e in zip(starts,starts[1:]):
        c=L[p];a=cur[c];runs.append((a,p,e-p,c));cur[c]+=e-p
    fstarts=sorted(a for a,p,l,c in runs)+[n]
    coarse={0,n}
    for a,p,l,c in runs:
        coarse.update((a,a+l))
        coarse.update(a+b-p for b in fstarts if p<b<p+l)
    cuts=sorted(coarse|set(starts)); r=len(runs)
    assert len(coarse)-1<=2*r-1
    assert len(cuts)-1<=3*r-2
    return P,F,cuts,r

def check(L):
    P,F,cuts,r=model(L);n=len(L);checks=0;skips=0
    for u,v in zip(cuts,cuts[1:]):
        d=P[P[u]]-u; pair=[L[u],F[u]]
        for q in range(u,v):
            assert P[P[q]]==q+d and [L[q],F[q]]==pair
            assert F[q]==L[P[q]]
            m=n if d==0 else (v-1-q)//d+1 if d>0 else (q-u)//(-d)+1
            m=min(m,n)
            row=q;out=[]
            for j in range(m):
                assert u<=row<v
                out += [L[row],L[P[row]]];row=P[P[row]]
            assert out==pair*m and row==q+m*d
            # Independently cap at the first flagged source on the arithmetic progression.
            flags={0,n-1,n//2}
            if d==0:
                stop=0 if q in flags else m
            else:
                hits=[(f-q)//d for f in flags if (f-q)%d==0 and 0<=(f-q)//d<m]
                stop=min(hits,default=m)
            row=q;actual=0
            while actual<m and row not in flags:
                row=P[P[row]];actual+=1
            assert actual==stop and row==q+stop*d
            checks+=1;skips+=max(0,m-1)
    return checks,skips

def check_composed(L):
    P,F,cuts,r=model(L);n=len(L);checks=0
    for q in range(n):
        for k in range(1,5):
            row=q;shift=0;lo=0;hi=n;word=[]
            for j in range(k):
                c=bisect.bisect_right(cuts,row)-1;u,v=cuts[c:c+2]
                lo=max(lo,u-shift);hi=min(hi,v-shift)
                word += [L[row],F[row]]
                row=P[P[row]];shift=row-q
            assert lo<=q<hi
            m=n if shift==0 else (hi-1-q)//shift+1 if shift>0 else (q-lo)//(-shift)+1
            m=min(m,n);row=q;actual=[]
            for j in range(m*k):
                actual += [L[row],L[P[row]]];row=P[P[row]]
            assert actual==word*m and row==q+m*shift
            checks+=1
    return checks

columns=rows=saved=composed=0
for n in range(1,9):
    for L in itertools.product(range(3),repeat=n):
        a,b=check(list(L));columns+=1;rows+=a;saved+=b;composed+=check_composed(list(L))

def bwt(s):
    order=sorted(range(len(s)),key=lambda i:s[i:]+s[:i])
    return [s[(i-1)%len(s)] for i in order]

witnesses=[]
for s in ['ababab','abcabcabc','aaaaaaaa','aabca']:
    L=bwt(s);P,F,cuts,r=model(L)
    cells=[]
    for u,v in zip(cuts,cuts[1:]):
        d=P[P[u]]-u
        m=len(L) if d==0 else (v-1-u)//d+1 if d>0 else 1
        cells.append({'interval':[u,v],'delta':d,'pair':L[u]+F[u],'batch_at_start':m})
    witnesses.append({'text':s,'L':''.join(L),'psi':P,'cells':cells})

k=209697662/209700196;B=5.668*k;C=11.705-B;P=6.184
result={'columns':columns,'source_rows_checked':rows,'composed_packet_checks':composed,'formula_reused_steps_sum':saved,
 'pair_cell_bound':'at most 3r-2 after refining by L boundaries',
 'witnesses':witnesses,'arithmetic_only':{'paired_rows_per_output_byte':k,
 'run_build_per_output_byte':B,'R_residual':C,'O_residual':14.427-8.246*k,
 'P_residual':P-2.213*k,'free_build_R_over_P':C/P,
 'five_percent_target':.95*P,'residual_budget_current_builder':.95*P-B,
 'builder_budget_if_consumer_equals_P_residual':.95*P-(P-2.213*k),
 'consumer_budget_if_builder_equals_onehop':.95*P-2.213*k,
 'onebyte_residual':.158-1.812*98040/5000000}}
print(json.dumps(result,indent=2))
