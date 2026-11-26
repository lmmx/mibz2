"""Direct compressed Ψ + forward word ID using only the original bands and cross hits."""
from bisect import bisect_right
from itertools import accumulate
from constructor import incident

class Implicit:
    def __init__(self,bands,image,mark,colors,rules):
        f,end,hits,zero,words,width,_,_,below=incident(bands,image,mark,colors,rules)
        self.old=bands;self.f=f;self.end=end;self.zero=zero;self.words=words
        self.lo=[l for l,h in hits];self.hi=[h for l,h in hits]
        self.mass=[0,*accumulate(width)];self.closed=[l-self.mass[j] for j,l in enumerate(self.lo)]
        self.color=[colors[b[3]] for b in bands]
        self.mark=mark-below;self.source=[];self.image=[];self.istart=[];self.work={}
        s=i=0
        for e,(_,_,w,_) in enumerate(bands):
            self.source.append(s)
            d=self.mass[end[e]]-self.mass[f[e]] if f[e]>=0 else 0
            s+=w-(d if self.color[e] else 0)
        for e in image:
            count=end[e]-f[e] if f[e]>=0 else 0
            d=self.mass[end[e]]-self.mass[f[e]] if count else 0
            w=bands[e][2]-(0 if self.color[e] else d)
            if w:
                self.image.append(e);self.istart.append(i)
                self.work[count]=self.work.get(count,0)+w
            i+=w
        assert i==s;self.n=i
    def at(self,v):
        index=bisect_right(self.istart,v)-1
        z,w,_=self.at_owner(v,index);return z,w
    def at_owner(self,v,index):
        e=self.image[index];off=v-self.istart[index]
        x,y,_,word=self.old[e];f=self.f[e]
        if f<0:return self.source[e]+off,word,e
        end=self.end[e];p=self.mass
        if self.color[e]:
            middle=x+off
            j=bisect_right(self.lo,middle,f,end)
            if j>f and middle<self.hi[j-1]:
                a=self.zero[j-1];return self.source[a]+middle-self.old[a][1],self.words[j-1],a
            return self.source[e]+off-(p[j]-p[f]),word,e
        j=bisect_right(self.closed,y+off-p[f],f,end)
        return self.source[e]+off+p[j]-p[f],word,e
