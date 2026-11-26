"""Exhaustive short-safe policies using individual Psi rows as an independent packet oracle."""
from collections import Counter, defaultdict
from itertools import product
import json


def suffixes(words):
    return {w[i:] for w in words for i in range(1,len(w)+1)}


class Instance:
    def __init__(self,last,a,b,H):
        self.last,self.a,self.b,self.H=last,a,b,H
        self.psi=sorted(range(len(last)),key=last.__getitem__)
        self.front=bytes(last[y] for y in self.psi)
        back=[0]*len(last)
        for x,y in enumerate(self.psi):back[y]=x
        self.back=back
        self.words={}
        self.dest={}
        for x in range(a,b):
            row,w=x,bytearray()
            while True:
                w.append(self.front[row]);row=self.psi[row]
                if a<=row<b:break
            self.words[x]=bytes(w);self.dest[x]=row
        self.vocab=set(self.words.values())
        self.actions={w[:i] for w in self.vocab for i in range(min(len(w),H))}
        self.obligations=defaultdict(set)
        for w in self.vocab:
            for i in range(1,len(w)):
                self.obligations[w[i:]].add(w[:i-1])
        # Goals require all predecessor actions, including impossible >H lengths.
        self.obligations={s:ps for s,ps in self.obligations.items() if ps<=self.actions}
        self.baseline=suffixes(self.vocab)
        _,self.fullcost=self.execute(self.actions)
        self.r=1+sum(x!=y for x,y in zip(last,last[1:]))

    def execute(self,enabled):
        """No suffix model: split each source-affine packet by its actual consecutive row steps."""
        queue=[(self.a,self.a,self.b-self.a,b'')];leaves=[]
        # [input packets, original select intersections, resulting fragments, short fragments]
        cost=defaultdict(lambda:[0,0,0,0])
        while queue:
            origin,row,width,p=queue.pop()
            if p not in enabled:
                leaves.append((origin,row,width,p));continue
            c=cost[p];c[0]+=1
            prev_raw=None
            start=0
            for i in range(width):
                q=row+i;y=self.psi[q]
                raw=(self.front[q],y-q)
                if raw!=prev_raw:c[1]+=1;prev_raw=raw
                item=(raw,self.a<=y<self.b)
                if i==0:
                    carry=item
                if carry!=item:
                    self.post(carry,origin,row,start,i,p,queue,leaves,c);start=i;carry=item
            self.post(carry,origin,row,start,width,p,queue,leaves,c)
        return leaves,dict(cost)

    def post(self,part,origin,row,start,stop,p,queue,leaves,c):
        (byte,delta),early=part
        if early:
            leaves.append((origin+start,row+start,stop-start,p));c[3]+=1
        else:
            queue.append((origin+start,row+start+delta,stop-start,p+bytes([byte])))
        c[2]+=1

    def validate(self,A):
        leaves,cost=self.execute(A)
        assert cost=={p:c for p,c in self.fullcost.items() if p in A}
        f=sum(x[1] for x in cost.values())
        b=1+sum(x[2]-x[0] for x in cost.values())
        assert b==len(leaves) and b<=self.r+2
        # Recover each original source word from a backward core and the published cut identity.
        atlas={}
        for origin,row,w,p in leaves:
            for i in range(w):
                assert row+i not in atlas
                atlas[row+i]=(origin+i,p)
        assert len(atlas)==self.b-self.a
        cores=set();cuts={}
        for dest in range(self.a,self.b):
            x=dest;rev=bytearray()
            while True:
                x=self.back[x];rev.append(self.front[x])
                if x in atlas:break
            origin,p=atlas[x]
            core=bytes(reversed(rev));w=self.words[origin]
            assert self.dest[origin]==dest and p+core==w and core
            # Short-safe lookahead from a word's last row is paid but does not remove it.
            t=0
            while t<len(w)-1 and w[:t] in A:t+=1
            assert p==w[:t]
            if w in cuts:assert cuts[w]==t
            cuts[w]=t;cores.add(core)
        direct=suffixes(cores)
        removed={s for s,ob in self.obligations.items() if ob<=A}
        assert self.baseline-direct==removed and direct<=self.baseline and b'' in direct
        reduced={s for s,ob in self.obligations.items()
                 if all(p in A for p in ob if not any(p!=q and q.startswith(p) for q in ob))}
        assert reduced==removed
        return len(direct),f,b

    def policies(self):
        parents=sorted(self.actions,key=lambda p:(len(p),p))
        def iterate(i,chosen):
            if i==len(parents):
                yield frozenset(chosen);return
            p=parents[i]
            yield from iterate(i+1,chosen)
            if not p or p[:-1] in chosen:
                chosen.add(p);yield from iterate(i+1,chosen);chosen.remove(p)
        return iterate(0,set())


def instance_from_cyclic(raw,symbol,H):
    n=len(raw)
    order=sorted(range(n),key=lambda i:raw[i:]+raw[:i])
    last=bytes(raw[(i-1)%n] for i in order)
    a=sum(c<symbol for c in raw);b=a+raw.count(symbol)
    return Instance(last,a,b,H)


def textkey(s):return ''.join(map(str,s))


def named():
    x=instance_from_cyclic(bytes((0,1,2,1,0,2,2,1)),0,3)
    e=b'';z=bytes([0]);b1=bytes([0,1]);b2=bytes([0,2])
    base={e,z};A=base|{b1};B=base|{b2}
    vals=[x.validate(k) for k in (base,A,B,A|B)]
    assert [v[0] for v in vals]==[2,2,2,1]
    complementary=dict(last=textkey(x.last),U=[x.a,x.b],return_types=sorted(map(textkey,x.vocab)),
                 policies=[sorted(map(textkey,k)) for k in (base,A,B,A|B)],Q_F_B=vals,
                 suffix_one_obligations=sorted(map(textkey,x.obligations[bytes([1])])))
    no_step=Instance(bytes([0,1]),0,2,1)
    none=no_step.validate(set());early=no_step.validate({b''})
    assert none==(1,0,1) and early==(1,2,2)
    return dict(complementarity=complementary,all_length_one=dict(last="01",U=[0,2],default=none,
                  enabled_root=early,coalesced_identity_chart=1))


def main():
    tally=Counter();examples=named()
    for n in range(1,8):
        for last in product((0,1),repeat=n):
            for a in range(n):
                for b in range(a+1,n+1):
                    for H in (0,1,3,5):
                        x=Instance(bytes(last),a,b,H)
                        tally["section_depths"]+=1
                        results={A:x.validate(A) for A in x.policies()}
                        tally["policies"]+=len(results)
                        tally["source_arcs_checked"]+=(b-a)*len(results)
                        tally["policies_with_charged_short_fragments"]+=sum(
                            any(x.fullcost[p][3] for p in A) for A in results)
                        tally["policy_instances_with_depth_impossible_suffixes"]+=len(x.obligations)<len(x.baseline)-1
                        tally["policy_instances_with_shared_obligation"]+=any(len(o)>1 for o in x.obligations.values())
                        # Distinct proper-suffix savings are supermodular on prefix-closed sets.
                        for A,(qA,_,_) in results.items():
                            for B,(qB,_,_) in results.items():
                                if A<=B or B<=A:continue
                                tally["incomparable_policy_pairs"]+=1
                                d=qA+qB-results[A|B][0]-results[A&B][0]
                                assert d>=0
                                tally["strict_complement_pairs"]+=d>0
    print(json.dumps(dict(counts=tally,witnesses=examples),indent=2,sort_keys=True))


if __name__=="__main__":main()
