"""Sharp one-shift projected-descriptor family, with and without clipped end runs."""
from itertools import groupby
import csv
import sys
from quotient import Section, original, shifted, table


def solve(last,clip):
    r=[(c,len(list(w))) for c,w in groupby(last)]
    n=len(last);k=last.count(3);u=Section([(n-k+clip,n-clip)])
    lf,psi=original(r)
    v,c=shifted(u,psi,[x[0] for x in psi]);assert c==3
    _,_,_,stats=table(lf,[u,v])
    return stats


def main():
    writer=csv.writer(sys.stdout,lineterminator="\n")
    writer.writerow(("components","partial_r","partial_descriptors","full_r","full_descriptors"))
    for v in range(1,258):
        prefix=[3,3]
        for i in range(v-1):prefix.extend([1+i%2,3])
        last=prefix+[3]+[0]*(2*v+1)
        partial=solve(last,1)
        assert partial["r"]==2*v and partial["target_components"]==v
        assert partial["normalized_descriptors"]==3*v+2
        # An alternating non-3 prefix separates all deleted 3s and survives intact.
        full_prefix=[1]
        for i in range(v):full_prefix.extend([3,2-i%2])
        full=solve(full_prefix+[0]*(2*v+1),0)
        assert full["r"]==2*v+2 and full["target_components"]==v
        assert full["normalized_descriptors"]==full["r"]+v
        writer.writerow((v,partial["r"],partial["normalized_descriptors"],full["r"],full["normalized_descriptors"]))


if __name__=="__main__":main()
