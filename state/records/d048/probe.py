"""D048 finite, independent mathematical checks; NOT Rust/Miri execution."""
import itertools,json
def column(syms,limit):
    lst=[0,1,2]; out=[]; run=0; w=1
    for s in syms:
        if s<2:
            run+=(s+1)*w; w*=2
            continue
        if len(out)+run>limit: return "BlockTooLong",out
        out.extend([lst[0]]*run);run=0;w=1
        if s==4: break
        if len(out)>=limit:return "BlockTooLong",out
        val=lst.pop(s-1);lst.insert(0,val);out.append(val)
    return "ok",out
def emitted(syms,limit):
    lst=[0,1,2]; rec=[];n=0;run=0;w=1
    if syms and syms[0]<2:rec.append((0,0))
    for s in syms:
        if s<2:run+=(s+1)*w;w*=2;continue
        if n+run>limit:return "BlockTooLong",n,rec
        n+=run;run=0;w=1
        if s==4:break
        if n>=limit:return "BlockTooLong",n,rec
        val=lst.pop(s-1);lst.insert(0,val);rec.append((n,val));n+=1
    rec.append((n,0));return "ok",n,rec
cases=success=errors=refused=0
for length in range(7):
    for syms in itertools.product(range(5),repeat=length):
        for limit in [0,1,3,12,128]:
            cases+=1
            a,col=column(syms,limit);b,n,rec=emitted(syms,limit)
            assert a==b and len(col)==n
            assert len(rec)<=len(syms)+1
            if a!="ok":errors+=1;continue
            success+=1
            tiling=(rec==[(0,0)] if n==0 else rec[0][0]==0 and all(x[0]<y[0] for x,y in zip(rec,rec[1:])))
            if not tiling:
                refused+=1
                assert n==0 and len(rec)==2 and rec[1]==(0,0)
            else:
                rebuilt=[]
                for (p,c),(end,_) in zip(rec,rec[1:]):rebuilt.extend([c]*(end-p))
                assert rebuilt==col
_,_,m3=emitted([2,2,2],100)
assert len(m3)==4 # reserve(syms.len()) may provide only 3 words
_,_,empty=emitted([],100)
assert len(empty)==1 and len(empty)+1>1 # M4 violates record bound
maxrun=2*((1<<21)-1)
assert maxrun==4194302 and 4*maxrun+3==(1<<24)-5
assert 5*maxrun+4==20971514
fixed=[5507,3878,214,2061,4629,95]
bounded=[655778,457727,27902,238086,553516,12591]
assert sum(fixed)==4*2048*2
assert sum(bounded)==(4+471)*2048*2
r={"model":"finite scalar column versus emitted-record semantics, not native code",
   "cases":cases,"successful":success,"block_errors":errors,"refused_leading_no_eob":refused,
   "max_symbols":6,"limits":[0,1,3,12,128],"M3_witness":{"symbols":[2,2,2],"required_words":4,"mutant_reserve":3,"closing_byte_offset":12},
   "M4_empty_bound":{"baseline_records":1,"mutated_length":2,"bound":1},
   "longest_run":maxrun,"under24":4*maxrun+3,"over24":5*maxrun+4,
   "fixed_kernel_runs":sum(fixed),"bounded_kernel_runs":sum(bounded),
   "miri_count_correction":"80 scalar / 160 combined include the 4 / 8 longest-run calls; path classes exclude those calls"}
print(json.dumps(r,indent=2))
