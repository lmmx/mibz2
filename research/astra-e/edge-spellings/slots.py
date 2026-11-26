"""Cheap handle-map falsifier over exact paid-width band owners."""
import csv,sys
from pathlib import Path
from handles import words
w=csv.writer(sys.stdout,lineterminator='\n')
w.writerow(['block','slots','reuse_words','reuse_bytes'])
for (b,_,_,_,_),records in words(Path(sys.argv[1])):
    for bits in [0,4,6,8,10]:
        tab=[None]*(1<<bits)
        hits=spelled=0
        for h,_,word in records:
            key=(h,word[-1])
            i=0 if bits==0 else (h*2654435761 & 0xffffffff)>>(32-bits)
            if tab[i]==key:
                hits+=1
                spelled+=len(word)
            tab[i]=key
        w.writerow([b,1<<bits,hits,spelled])
