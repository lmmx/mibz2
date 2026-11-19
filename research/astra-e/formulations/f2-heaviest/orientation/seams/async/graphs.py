"""Embed bipartite cover constraints into an actual one-delimiter cyclic return dictionary."""
from itertools import combinations
from math import lcm
from schedule import grow, optimize, price


def code(i, size):
    return bytes(2 + int(x) for x in f"{i:0{max(1,(size-1).bit_length())}b}")


for layers in (2, 3, 4):
    left = lcm(*range(1, layers+1))
    edges = []
    columns = 0
    for k in range(1, layers+1):
        for start in range(0,left,k):
            for i in range(start,start+k):
                edges.append((i,columns,k))
            columns += 1
    for length in (16,64,256):
        # 0 is the only delimiter; 1 is the long interior context; 2,3 code the endpoints.
        words = {b'\0'+code(a,left)+b'\1'*length+code(b,columns):3**(k-1) for a,b,k in edges}
        opt = optimize(list(words))["queries"]
        greedy,_,_,_ = grow(words,False)
        worst_len = sum(len(w)*m for w,m in words.items())
        assert opt >= left*length  # the k=1 edges give a perfect matching on the left
        print(f"layers={layers} left={left} right={columns} interior_length={length} "
              f"cyclic_text_rows={worst_len} edge_cover_bound={left*length} optimal={opt} "
              f"maximum_mass={greedy} ratio={greedy/opt:.5f}")
