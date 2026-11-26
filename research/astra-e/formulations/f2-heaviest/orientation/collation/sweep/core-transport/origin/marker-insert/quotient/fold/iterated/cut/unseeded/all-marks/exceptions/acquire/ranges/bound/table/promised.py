"""Build actual sorted rotations including ties and check every unanimous one-text model."""
from collections import Counter
from itertools import product
import json


def verify():
    total=Counter()
    for n in range(1,11):
        for text in product((0,1),repeat=n):
            order=sorted(range(n),key=lambda i:text[i:]+text[:i])
            last=[text[(i-1)%n] for i in order]
            count=[0,0];base=[0,last.count(0)];lf=[]
            for b in last:
                lf.append(base[b]+count[b]);count[b]+=1
            lap=[]
            for start in range(n):
                x=lf[start];d=1
                while x!=start:x=lf[x];d+=1
                lap.append(d)
            for k in range(n+2):
                contexts=[tuple(text[(i+j)%n] for j in range(k)) for i in order]
                incoming={}
                for word,b in zip(contexts,last):
                    incoming.setdefault(word,set()).add(b)
                if any(len(values)!=1 for values in incoming.values()):continue
                K=len(incoming)
                assert lap==[K]*n
                transitions={}
                for x,word in enumerate(contexts):
                    target=contexts[lf[x]]
                    assert transitions.setdefault(word,target)==target
                assert len(set(transitions.values()))==K
                total['unanimous_models']+=1
                total['physical_mark_checks']+=n
                total['proper_power_models']+=K<n
                total['proper_power_with_multiple_contexts']+=1<K<n
                total['unanimous_depth_zero']+=k==0
    return total
if __name__=='__main__':print(json.dumps(verify(),sort_keys=True,indent=2))
