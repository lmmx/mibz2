"""A full surviving-bucket rank needs linear new information if only the marked incoming piece was saved."""
from itertools import product
import json

def column(bits):
    # Source symbols a<b<c; b separates arbitrary choices and fixes every raw width.
    values=[v for bit in bits for v in [1,2*bit]]
    values.extend([1,0,1])
    values.extend(v for bit in bits for v in [2*(1-bit),1])
    x=2*len(bits)+1
    return values,x

def check():
    cases=0;neighbors=0
    for n in range(1,11):
        for v in product((0,1),repeat=n):
            last,x=column(v)
            freq=[last.count(k) for k in range(3)]
            assert len(last)==4*n+3 and all(a!=b for a,b in zip(last,last[1:]))
            assert freq==[n+1,2*n+2,n] and sorted(last)[x]==1 and last[x]==0
            p=last[:x].count(0);assert p==n-sum(v)
            for j in range(n):
                flipped=list(v);flipped[j]^=1
                other,_=column(flipped)
                assert [k for k,(a,b) in enumerate(zip(last,other)) if a!=b]==[2*j+1,2*n+3+2*j]
                assert abs(other[:x].count(0)-p)==1
                neighbors+=1
            cases+=1
    return dict(all_bit_vectors=cases,one_bit_neighbors=neighbors,largest_pairs=10)
if __name__=='__main__':print(json.dumps(check(),indent=2))
