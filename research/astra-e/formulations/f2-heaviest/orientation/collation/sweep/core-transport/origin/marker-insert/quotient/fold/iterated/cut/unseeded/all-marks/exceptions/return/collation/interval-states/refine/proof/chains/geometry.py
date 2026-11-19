"""Original-run construction of mandatory seam paths, periodic fibers and neutral choices."""
from bisect import bisect_left, bisect_right
from collections import Counter
from itertools import accumulate, chain, product


class Geometry:
    def __init__(self, runs):
        self.byte, widths = zip(*runs)
        assert all(w>0 for w in widths) and all(a!=b for a,b in zip(self.byte,self.byte[1:]))
        self.src = [0,*accumulate(widths)]
        self.n = self.src[-1]
        self.order = sorted(range(len(runs)),key=lambda i:self.byte[i])
        self.dst = [0,*accumulate(widths[i] for i in self.order)]
        self.image = [0]*len(runs)
        for i,s in enumerate(self.order):
            self.image[s] = self.dst[i]
        self.fseams = {self.dst[i] for i in range(1,len(runs))
                       if self.byte[self.order[i-1]] != self.byte[self.order[i]]}

    def psi(self,z):
        i=bisect_right(self.dst,z)-1
        return self.src[self.order[i]]+z-self.dst[i]

    def lf(self,x):
        i=bisect_right(self.src,x)-1
        return self.image[i]+x-self.src[i]

    def mandatory(self,z):
        i=bisect_right(self.dst,z)-1
        if z!=self.dst[i]:
            return self.src[self.order[i]]+z-self.dst[i]
        return None

    def support(self,z):
        assert 0<z<self.n
        if z in self.fseams:
            return None
        # Adjacent target ranks share their F byte. Both endpoints inclusive.
        return self.psi(z-1)+1,self.psi(z)

    def paths(self,old):
        starts=list(self.src[1:-1])
        for z in old:
            u=self.mandatory(z)
            if u is not None:
                starts.append(u)
        paths=[]
        for z in sorted(starts):
            path=[]
            while z is not None:
                path.append(z)
                z=None if z in old else self.mandatory(z)
            paths.append(path)
        return paths

    def fibers(self):
        # Positive gaps on these paths are precisely the boundaries between distinct futures.
        ends=[0,*sorted(chain.from_iterable(self.paths(set()))),self.n]
        size={lo:hi-lo for lo,hi in zip(ends,ends[1:])}
        necklaces=[]
        unvisited=set(size)
        while unvisited:
            start=next(iter(unvisited))
            group=[]
            z=start
            while z in unvisited:
                unvisited.remove(z)
                group.append(z)
                z=self.psi(z)
            assert z==start
            g=size[start]
            assert all(size[x]==g for x in group)
            necklaces.append((sorted(group),g))
        return sorted(necklaces)

    def scores(self,src_cuts,dst_cuts):
        """Physical mass using run/source/target contacts, including very wide fibers."""
        src=[*sorted(src_cuts),self.n]
        dst=[*sorted(dst_cuts),self.n]
        total=i=x=state=0
        hist=Counter()
        while x<self.n:
            a=self.byte[i]
            y=self.image[i]+x-self.src[i]
            j=bisect_right(dst,y)
            w=min(self.src[i+1]-x,src[state]-x,dst[j]-y)
            hist[a,j]+=w
            x+=w
            if x==src[state]:
                total+=max(hist.values())
                hist.clear()
                state+=1
            if x==self.src[i+1]:
                i+=1
        return total


class Choices:
    def __init__(self,geo,old,necklaces):
        self.geo,self.old=geo,set(old)
        self.paths=geo.paths(self.old)
        self.slabs=[]
        self.flex=[]
        for path in self.paths:
            z=path[-1]
            if z not in self.old and z not in geo.fseams:
                lo,hi=geo.support(z)
                assert lo<hi
                self.flex.append((z,lo,hi))
        for group,g in necklaces:
            special=set()
            for z in self.old:
                ix=bisect_right(group,z)-1
                if ix>=0 and 0<z-group[ix]<g:
                    special.add(z-group[ix])
            levels=[0,*sorted(special),g]
            for lo,hi in zip(levels,levels[1:]):
                if lo+1<hi:
                    self.slabs.append((group,lo+1,hi-1))

    def enumerate(self,canonical):
        paths=[]
        for p in self.paths:
            # A path containing a held seam terminates at that seam.
            max_start=len(p)-int(p[-1] in self.old)
            paths.append([tuple(p[i:]) for i in range(max_start+1)])
        cycles=[]
        for group,a,b in self.slabs:
            offsets=sorted({a,b}) if canonical else range(a,b+1)
            choices=[]
            for mask in range(1<<len(offsets)):
                choices.append(tuple(x+j for i,j in enumerate(offsets) if mask>>i&1 for x in group))
            cycles.append(choices)
        for choice in product(*paths,*cycles):
            selected=tuple(sorted(chain.from_iterable(choice)))
            assert len(set(selected))==len(selected) and self.old.issubset(selected)
            available=set(selected)
            if all(z not in available or
                   ((at:=bisect_left(selected,lo))<len(selected) and selected[at]<=hi)
                   for z,lo,hi in self.flex):
                yield selected
