"""D049 supplementary Python model; no Rust, SIMD, threads or raw pointers executed."""
import bisect, itertools, json, pathlib, random
HERE = pathlib.Path(__file__).parent
source = (HERE / 'run_team.rs').read_text()
coop = (HERE / 'coop.rs').read_text()
def function(text, name, next_name):
    return text.split(name, 1)[1].split(next_name, 1)[0]
assert 'k.max(2)' in function(source, 'pub fn pipelined_run_fed(', '#[cfg(test)]')
assert 'let k = k.max(1)' in function(coop, 'fn named(', 'pub(crate) fn threads')
assert 'let shares = helpers.min(MAX_SHARES)' in source
assert 'helper(j - 1)' in source
count = function(source, 'fn count(&self, j: usize)', 'fn merge(&self)')
assert '&mut *self.hist.add(j)' in count and 'j >=' not in count
assert 'hist.resize(shares, [[0; 256]; 3])' in source
assert 'w + 8 <= end' in source
assert 'for b in held.into_iter().flatten()' in source
assert 'if j >= self.shares' in function(coop, 'pub(crate) fn share(&self, j:', 'let bit =')

def check(ll, h, weighted, avx, mutant=False):
    n=len(ll)
    recs=[(i,c) for i,c in enumerate(ll) if i==0 or c!=ll[i-1]]
    r=len(recs); starts=[p for p,c in recs]+[n]
    cuts=[0]
    if weighted:
        keys=[p+8*i for i,p in enumerate(starts)]
        for j in range(1,h):
            cuts.append(bisect.bisect_left(keys,(n+8*r)*j//h,cuts[-1],r))
    else: cuts=[r*j//h for j in range(h)]
    cuts.append(r)
    assert cuts[0]==0 and cuts[-1]==r and all(a<=b for a,b in zip(cuts,cuts[1:]))
    # Independent stable-sort oracle: table low byte from L, high bits from Psi.
    psi=sorted(range(n),key=lambda p:ll[p]); want=[ll[q]|(psi[q]<<8) for q in range(n)]
    hist=[[0]*3 for _ in range(h)]
    for j in range(h):
        for i in range(cuts[j],cuts[j+1]):hist[j][recs[i][1]]+=starts[i+1]-starts[i]
    owned=[[None]*3 for _ in range(h)]; at=0
    for c in range(3):
        for j in range(h):owned[j][c]=(at,at+hist[j][c]);at+=hist[j][c]
    assert at==n
    coverage=[0]*n
    for j in range(h):
        buf={}; curs=[a for a,e in owned[j]]
        def cell(q,stop,end,base):
            w=q
            while avx and w<stop and w+8<=end+(8 if mutant else 0):
                for z in range(w,w+8):
                    assert any(a<=z<e for a,e in owned[j]), ('unowned', n,h,j,z,owned[j])
                    buf[z]=base+((z-q)<<8)
                w+=8
            while w<stop:buf[w]=base+((w-q)<<8);w+=1
        for i in range(cuts[j],cuts[j+1]):
            p,c=recs[i];a=curs[c];end=owned[j][c][1];length=starts[i+1]-p
            assert length<=end-a
            q=a;qe=a+length
            while q<qe:
                l=bisect.bisect_right(starts,q)-1
                stop=min(starts[l+1],qe)
                cell(q,stop,end,ll[q]|((p+q-a)<<8));q=stop
            curs[c]=qe
        for c,(a,e) in enumerate(owned[j]):
            assert curs[c]==e
            for q in range(a,e):assert buf[q]==want[q];coverage[q]+=1
        assert set(buf)=={q for a,e in owned[j] for q in range(a,e)}
    assert coverage==[1]*n

cases=0
columns=[]
for n in range(1,8):columns.extend(itertools.product(range(3),repeat=n))
rng=random.Random(49)
for n in [8,9,15,16,17,31,40,65,129]:
    columns += [tuple(rng.randrange(3) for _ in range(n)),tuple([1]*n),tuple((i//9)%3 for i in range(n))]
for ll in columns:
    for h in [1,2,3,5]:
        for weighted in [False,True]:
            for avx in [False,True]:check(ll,h,weighted,avx);cases+=1
try:check((0,),1,True,True,True)
except AssertionError as e:mutant=str(e)
else:raise AssertionError('model mutant survived')
# The metadata route is reached before count_share's safe slice could panic.
k=66;helpers=k-1;shares=min(helpers,64);j=64
assert j in range(helpers) and not j<shares
witness={'k':k,'helpers':helpers,'allocated_hist_entries':shares,'helper_j':j,
         'hist_entry_bytes':3*256*4,'byte_offset':j*3*256*4,
         'initialized_hist_bytes':shares*3*256*4,
         'finding':'one-past initialized histogram; raw mutable reference formed before safe slicing'}
# Conditional on entering the real outer Err path, dropping held publications stalls ordered read.
ready={7:'ok'};taken=5;end=8
assert taken not in ready and taken!=end
for place in [5,6]:ready[place]='Internal'
assert ready[taken]=='Internal'
out={'model_cases':cases,'columns':len(columns),'failures':0,'end_plus_8_mutant':mutant,
     'share_index_witness':witness,'outer_publication_model':'baseline supplies place 5; missing held 5,6 stalls',
     'scope':'finite Python semantic/source model only, not Rust/Miri/concurrency execution'}
print(json.dumps(out,indent=2))
(HERE/'results.json').write_text(json.dumps(out,indent=2)+'\n')
