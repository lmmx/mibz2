import math,statistics as st,sys
# paired.py REF R BUILD,BUILD..  : per-run sums over the blocks (best of 5 each), paired log-ratio vs REF, t(R-1) 95% CI
T={2:12.706,3:4.303,4:3.182,5:2.776,6:2.571,7:2.447,8:2.365,9:2.306,10:2.262,11:2.228,12:2.201,16:2.131}
def tot(b,r):
    fr=c=o=0
    for line in open(f'blk-{b}-{r}.tsv'):
        x=line.split('\t')
        if x[0]=='blk':
            if x[4]=='1': fr+=float(x[7])
            c+=float(x[8]); o+=float(x[10])
    return fr,c,o
ref=sys.argv[1]; R=range(1,int(sys.argv[2])+1); tq=T[len(R)]
builds=sys.argv[3].split(',')
print("build\tOpen ms (min-max)\tfront+Open\tfront+Cells")
for b in [ref]+builds:
    ts=[tot(b,r) for r in R]
    f=lambda i: f"{min(t[i] for t in ts):.1f}-{max(t[i] for t in ts):.1f}"
    fo=[t[0]+t[2] for t in ts]; fc=[t[0]+t[1] for t in ts]
    print(f"{b}\t{f(2)}\t{min(fo):.1f}-{max(fo):.1f}\t{min(fc):.1f}-{max(fc):.1f}")
for b in builds:
    for name,g in [('Open',lambda t:t[2]),('front+Open',lambda t:t[0]+t[2])]:
        l=[math.log(g(tot(b,r))/g(tot(ref,r))) for r in R]
        m=st.mean(l); se=st.stdev(l)/math.sqrt(len(l))
        print(f"{b}/{ref}\t{name}\t{math.exp(m):.3f} [{math.exp(m-tq*se):.3f}, {math.exp(m+tq*se):.3f}]")
# front+Open / front+Cells within each run
for b in [ref]+builds:
    l=[math.log((t[0]+t[2])/(t[0]+t[1])) for t in [tot(b,r) for r in R]]
    m=st.mean(l); se=st.stdev(l)/math.sqrt(len(l))
    print(f"{b}\t(front+Open)/(front+Cells)\t{math.exp(m):.3f} [{math.exp(m-tq*se):.3f}, {math.exp(m+tq*se):.3f}]")
for b in [ref]+builds:
    l=[math.log(t[2]/t[1]) for t in [tot(b,r) for r in R]]
    m=st.mean(l); se=st.stdev(l)/math.sqrt(len(l))
    print(f"{b}\tOpen/Cells\t{math.exp(m):.3f} [{math.exp(m-tq*se):.3f}, {math.exp(m+tq*se):.3f}]")
# Open over blocks, min across runs per block
def blks(b,r):
    d={}
    for line in open(f'blk-{b}-{r}.tsv'):
        x=line.split('\t')
        if x[0]=='blk': d[int(x[1])]=(float(x[7]) if x[4]=='1' else 0.0, float(x[8]), float(x[10]))
    return d
for b in [ref]+builds:
    ds=[blks(b,r) for r in R]
    print(f"{b}\tmin-per-block: Open {sum(min(d[i][2] for d in ds) for i in ds[0]):.1f}, front+Open {sum(min(d[i][0] for d in ds)+min(d[i][2] for d in ds) for i in ds[0]):.1f}, Cells {sum(min(d[i][1] for d in ds) for i in ds[0]):.1f}")
