import itertools,json
def check(s):
 n=len(s); runs=[(i,c) for i,c in enumerate(s) if i==0 or c!=s[i-1]]
 if not n:return
 lens=[(runs[i+1][0] if i+1<len(runs) else n)-p for i,(p,c) in enumerate(runs)]
 order=sorted(range(len(runs)),key=lambda i:runs[i][1])
 fo=[]; a=0
 for i in order:fo.append((a,runs[i][0]));a+=lens[i]
 fo.append((n,0)); rs=[p for p,c in runs]+[n]
 oracle=[(p<<8)|s[q] for q,p in enumerate(sorted(range(n),key=lambda p:s[p]))]
 out=[None]*(n+8); f=l=q=slots=0
 while q<n:
  af,pf=fo[f]; fe=fo[f+1][0]; le=rs[l+1]; end=min(fe,le)
  assert q<end<=n and af<=q and rs[l]<=q
  base=(pf+q-af)*256+runs[l][1]
  for w in range(q,end,8):
   assert w+8<=n+7
   for j in range(8):out[w+j]=(base+256*(w-q+j))&0xffffffff
   slots+=8
  f+=end==fe;l+=end==le;q=end
 assert out[:n]==oracle
 assert n<=slots<=n+7*(len(set(rs[:-1]+[x[0] for x in fo[:-1]])))
count=0
for n in range(9):
 for s in itertools.product(range(3),repeat=n):check(s);count+=1
# Private whole-table overstore cannot be used on disjoint output slices as-is:
# one-row left share stores slots [0,8), intersecting right share [1,3).
assert set(range(8)) & set(range(1,3)) == {1,2}
print(json.dumps({"ternary_columns_checked":count,"max_length":8,"whole_table_model":"PASS","parallel_unmasked_store_counterexample":{"left_owned":[0,1],"left_written":[0,8],"right_owned":[1,3],"overlap":[1,2]},"scope":"Mathematical sweep model only; not Rust, AVX2 execution, Miri, fuzz or a memory-model proof."},indent=2))
