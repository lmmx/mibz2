"""A stable one-cycle token BWT where row-max and query-max choices differ strictly."""
from collections import Counter,defaultdict
from itertools import groupby
from solve import solve
w=tuple(map(int,'0123023403034'))
n=len(w);d=0
rotation=sorted(range(n),key=lambda i:w[i:]+w[:i])
last=[w[(i-1)%n] for i in rotation];freq=Counter(w)
assert max(freq,key=lambda k:(freq[k],-k))==d
targets=defaultdict(set);states=defaultdict(set)
for i,k in enumerate(w):
    targets[k].add(w[(i+1)%n]);u=[];j=i
    while w[j]!=d:u.append(w[j]);j=(j+1)%n
    states[tuple(u)].add(w[(i-1)%n])
g={k:next(iter(t)) for k,t in targets.items() if len(t)==1 and k!=d and k not in t and d not in t}
supports=Counter(tuple(sorted(e)) for e in states.values() if e<=g.keys())
best,p,_=solve(g,dict.fromkeys(g,2),supports)
assert g=={1:2,2:3} and freq[2]==2 and freq[1]==1 and p=={1} and best[0]==1
assert all(not set(e)<={2} for e in supports)
print('token_cycle', ''.join(map(str,w)), 'stable_last', ''.join(map(str,last)), 'old_suffix_states',len(states),
      'unique_max_row_choice 2, removes 2 rows and 0 queries; unique_max_query_choice 1, removes 1 row and 1 query')
