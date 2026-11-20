"""Q1 control: METIS level-0 bisection of the real core against a degree-preserving random rewiring.

Usage: BLOCKS=<dir>/ python baseline.py   (same environment as census.py)
"""
import os, sys, warnings
import numpy as np, pymetis
from scipy.sparse import csr_matrix
warnings.filterwarnings('ignore')
sys.argv = ['census']
exec(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'census.py')).read().split('\nfor b in M')[0])


def bis(nv, u, v):
    A = csr_matrix((np.ones(2 * len(u), dtype=np.int64), (np.r_[u, v], np.r_[v, u])), shape=(nv, nv))
    A.sum_duplicates()
    return pymetis.part_graph(2, xadj=A.indptr, adjncy=A.indices, eweights=A.data)[0]


rng = np.random.default_rng(1)
for f in ['block-0.L', 'block-210.L']:
    G = build(open(D + f, 'rb').read()); series, ca = reduce_series(G)
    core = np.flatnonzero(~series); cid = np.full(G['r'], -1); cid[core] = np.arange(len(core))
    u = np.array([cid[c[0]] for c in ca]); v = np.array([cid[c[1]] for c in ca]); k = u != v; u, v = u[k], v[k]
    v2 = rng.permutation(v); k2 = u != v2
    print(f, 'core level-0 cut', bis(len(core), u, v), 'degree-preserving random rewire', bis(len(core), u[k2], v2[k2]))
