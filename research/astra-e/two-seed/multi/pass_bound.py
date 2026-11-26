"""Verify an extremal family for the equal-remaining-space scheduler."""
def witness(k):
    base=4*k+1
    ceiling=base**(k-1)+1
    lengths=[ceiling-base**i for i in range(k-1,-1,-1)]
    remaining=lengths[:]
    for i in range(k):
        free=sum(remaining)
        cells=[free//len(remaining)+(j<free%len(remaining)) for j in range(len(remaining))]
        fitting=[j for j,(w,c) in enumerate(zip(remaining,cells)) if w<=c]
        assert fitting==[0]
        remaining=remaining[1:]
    return lengths

for k in range(2,11):
    lengths=witness(k)
    print(k,k,sum(lengths),lengths[0],lengths[-1])
