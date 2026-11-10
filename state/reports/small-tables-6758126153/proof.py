import itertools,bisect,json
cases=steps=0
maxscan=0
for n in range(1,9):
 for L in itertools.product(range(3),repeat=n):
  freq=[L.count(c) for c in range(3)]
  C=[0,freq[0],freq[0]+freq[1]]
  seen=[0]*3; lf=[]
  for c in L:
   lf.append(C[c]+seen[c]);seen[c]+=1
  for k in (1,2,4):
   targets=[];words=[]
   for i in range(n):
    j=i;w=[]
    for _ in range(k):w.append(L[j]);j=lf[j]
    targets.append(j);words.append(tuple(w))
   p=[0]+[i for i in range(1,n) if targets[i]!=targets[i-1]+1 or words[i]!=words[i-1]]
   q=[targets[x] for x in p]
   assert sorted(targets)==list(range(n))
   for i in range(n):
    x=bisect.bisect_right(p,i)-1
    j=q[x]+i-p[x]
    b=bisect.bisect_right(p,q[x])-1
    y=b
    while y+1<len(p) and p[y+1]<=j:y+=1
    assert j==targets[i] and y==bisect.bisect_right(p,j)-1
    maxscan=max(maxscan,y-b);steps+=1
   cases+=1
out={"sequence_k_cases":cases,"row_queries":steps,"largest_unbalanced_forward_scan":maxscan,
"g_break_even":{str(A):round((A/1.5-.3)/.7,6) for A in (2.4,2.563,2.7,3.0,3.3)},
"wm_one_counter_per_64bit_bytes":900000*8/8+((900000+63)//64)*8*4,
"D4_12byte_records":.1088*900000*12,
"D4_max_record_inflation_before_1_2MB":1200000/(.1088*900000*12)}
print(json.dumps(out,indent=2))
