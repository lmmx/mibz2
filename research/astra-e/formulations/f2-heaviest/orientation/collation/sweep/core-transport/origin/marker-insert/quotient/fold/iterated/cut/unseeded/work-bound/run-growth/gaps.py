"""Apply the actual coalesced unselected alias rule to both six-run witnesses."""
from fractions import Fraction
import csv
import sys
from witness import LAST, test
writer=csv.writer(sys.stdout,lineterminator='\n')
writer.writerow(('column','n','initial_runs','qualified','best_productive_runs','mixed_floor','mask_lo','mask_hi','q_lo','r_lo','q_hi','r_hi'))
for s in [LAST,"1111111122222222344444444555555550000000000"]:
    word,rows=test(s,True)
    n=len(word);good=[(m,q,r) for m,q,r in rows if 4*q>=n]
    pts=[(Fraction(r),i,i) for i,_,r in good]
    for i,x,a in rows:
        for j,y,b in rows:
            if 4*x<n<4*y:
                pts.append((a+(Fraction(n,4)-x)/(y-x)*(b-a),i,j))
    v,i,j=min(pts)
    writer.writerow((s,n,6,len(good),min(r for _,_,r in good),v,i,j,*rows[i][1:],*rows[j][1:]))
