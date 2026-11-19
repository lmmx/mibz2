"""Find source-slice split and coalescence cases independently from the dense contracted map."""
from itertools import groupby, product
import json
def examples():
    found={}
    for n in range(1,10):
        for last in product(range(3),repeat=n):
            k=len(set(last))
            if set(last)!=set(range(k)):continue
            first=sorted(last)
            nxt=sorted(range(n),key=last.__getitem__)
            pred=[nxt.index(i) for i in range(n)]
            for mask in range(1,2**k-1):
                color=lambda c:mask>>c&1
                dead={nxt[p] for p in range(n) if not color(first[p]) and color(first[nxt[p]])}
                if not dead:continue
                kept=[p for p in range(n) if p not in dead]
                alias=lambda p:(first[p],-1 if color(first[p]) else first[nxt[p]])
                old_to_new={x:i for i,x in enumerate(kept)}
                labels=[alias(pred[pred[x]] if pred[x] in dead else pred[x]) for x in kept]
                for x in range(n):
                    z=pred[x] if x in dead else x
                    j=z
                    while j>0 and (last[j-1],first[j-1])==(last[z],first[z]):j-=1
                    lo=old_to_new[j];hi=old_to_new[z]
                    pieces=[a for a,_ in groupby(labels[lo:hi+1])]
                    merged=lo>0 and labels[lo-1]==labels[lo]
                    row=dict(last=last,mark=x,target_mask=mask,moved=x in dead,
                        retained=z,source_slice_start=j,new_ordinal=hi,labels_through_mark=pieces,
                        merges_with_previous=merged)
                    if len(pieces)>1:found.setdefault('interior_map_boundary',row)
                    if merged:found.setdefault('cross_slice_coalescence',row)
                    if len(pieces)>1 and merged:found.setdefault('boundary_and_coalescence',row)
                    if last[x]==first[x] and pred[x]!=x and x>0:
                        found.setdefault('equal_types_without_a_fixed_row',row)
                if len(found)==4:return found
    raise AssertionError(found)
if __name__=='__main__':print(json.dumps(examples(),indent=2))
