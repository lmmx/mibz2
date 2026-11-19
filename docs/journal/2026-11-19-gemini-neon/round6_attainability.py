"""Test whether Neon's minimal duplicate-free fragmentation admits a valid replay order.

Builds the F_self_ns partition (each copy split greedily wherever a BWT run repeats), adds an edge
from the fragment holding each byte's Φ-source to the fragment holding the byte, and runs Kahn's
algorithm. ACYCLIC would mean the floor is attained. Loads Neon's verbatim verifier from
verify_phi_obstruction.py.txt. Run: python3 round6_attainability.py <block.L> <primary_index>
"""

import pathlib
import sys
import tempfile
import runpy

_src = (pathlib.Path(__file__).resolve().parent / "verify_phi_obstruction.py.txt").read_text()
with tempfile.NamedTemporaryFile("w", suffix=".py", delete=False) as _fh:
    _fh.write(_src)
from array import array
from collections import deque

g = runpy.run_path(_fh.name, run_name="lib")
path, primary = sys.argv[1], int(sys.argv[2])
L = open(path, "rb").read()
n = len(L)
r, run_start, run_len, cod_order, row_to_run, LF = g["build_runs_and_lf"](L)
NONE = 0xFFFFFFFF
row_to_pos = array("I", [NONE]) * n
cur = primary
for t in range(n):
    row_to_pos[cur] = t
    cur = LF[cur]
pos_head = [row_to_pos[run_start[a]] for a in range(r)]
pos_tail = [row_to_pos[run_start[a] + run_len[a] - 1] for a in range(r)]
order = sorted(range(r), key=pos_head.__getitem__)
tau0 = pos_head[order[0]]
text_run = array("I", [0]) * n
for x in range(n):
    text_run[(row_to_pos[x] - tau0) % n] = row_to_run[x]
cod_pred = [0] * r
for i in range(1, r):
    cod_pred[cod_order[i]] = cod_order[i - 1]
copies = []
for i in range(r):
    rid = order[i]
    tau = pos_head[rid] - tau0
    nxt = (pos_head[order[i + 1]] - tau0) if i + 1 < r else n
    m = nxt - tau - 1
    if m > 0:
        src = (pos_tail[cod_pred[rid]] + 1 - tau0) % n
        dst = tau + 1
        if src + m <= n:
            copies.append((dst, src, m))
        else:
            f = n - src
            copies.append((dst, src, f))
            copies.append((dst + f, 0, m - f))
# greedy duplicate-free fragments; frag id per destination position, source per position
frag = array("i", [-1]) * n
srcpos = array("i", [-1]) * n
fid = 0
flen = []
seen = array("I", [0]) * r
gen = 1
for dst, src, m in copies:
    gen += 1
    start = fid
    flen.append(0)
    for u in range(m):
        t = dst + u
        a = text_run[t]
        if seen[a] == gen:
            gen += 1
            fid += 1
            flen.append(0)
        seen[a] = gen
        frag[t] = fid
        srcpos[t] = src + u
        flen[fid] += 1
    fid += 1
F = fid
indeg = [0] * F
adj = [[] for _ in range(F)]
selfdep = 0
edges = set()
for t in range(n):
    f = frag[t]
    if f < 0:
        continue
    s = srcpos[t]
    gsrc = frag[s]
    if gsrc < 0:
        continue  # source is a literal: filled at start
    if gsrc == f:
        selfdep += 1
        continue
    if (gsrc, f) not in edges:
        edges.add((gsrc, f))
        adj[gsrc].append(f)
        indeg[f] += 1
q = deque(i for i in range(F) if indeg[i] == 0)
done = 0
while q:
    v = q.popleft()
    done += 1
    for w in adj[v]:
        indeg[w] -= 1
        if indeg[w] == 0:
            q.append(w)
copy_bytes = sum(m for _, _, m in copies)
print(
    f"{path.split('/')[-1]}: fragments={F} ({F / r:.2f}r, mean {copy_bytes / F:.1f} B), "
    f"fragment-graph edges={len(edges)}, self-deps={selfdep}, "
    f"topologically ordered={done}/{F} -> {'ACYCLIC: floor attained' if done == F and selfdep == 0 else 'CYCLIC: floor not attainable by this partition'}"
)
