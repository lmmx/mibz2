# Route A model, rows per configuration; see model.py for the calibration (EPYC 9V74).
F=3.6947; L2H=14/F; LM=28.4; M=19.0; BW=32*F; IPCa=4.0; IPCc=6.0
N=900_000
def row(name, ir, t_acc, t_miss, q_l1, other, lines, smt):
    occ=(t_acc-t_miss)*L2H + t_miss*LM + q_l1*L2H + other*LM
    k=2 if smt else 1
    issue=ir/((IPCc/2 if smt else IPCa)*F); cap=occ/M*k; bw=lines*64/BW*k
    print(f"{name:44s} {'SMT  ' if smt else 'alone'} issue {issue:5.2f} cap {cap:5.2f} bw {bw:5.2f} -> {max(issue,cap,bw):5.2f} ns/B  (occ {occ:4.1f} ns, Ir {ir:4.1f})")
# master: callgrind LL=1MB (alone) and LL=512KB (SMT: the sibling's walk holds half the L2)
row("master room/32/L1/512", 10.65, 1.0, 0.729, 0, 0.060, 0.789, False)
row("master room/32/L1/512", 10.65, 1.0, 0.866, 0, 0.065, 0.93, True)
# partitioned k=3 (S=112.5k, mean segment 8), P=7 (alone, 1MB) / P=14 (SMT, 512KB)
# T misses/B, queue L1 misses/B, other LL misses/B from callgrind (part-k3-*); Ir: walk2 per-path model 31 + flag 6.5 + assemble 12.3
row("partitioned k=3 prototype (as built)", 49.8, 1.0, 0.464, 0.443, 0.668, 2.0, False)
row("partitioned k=3 prototype (as built)", 49.8, 1.0, 0.492, 0.46, 0.70, 2.1, True)
# idealised: starts flagged in the build (free), assembly a parallel second-level walk (2 misses/segment, 16 Ir/segment)
row("partitioned k=3 idealised", 35.0, 1.0, 0.464, 0.443, 0.455, 1.7, False)
row("partitioned k=3 idealised", 35.0, 1.0, 0.492, 0.46, 0.48, 1.8, True)
# lower bound: no segment costs at all and a 22-instruction step
row("partitioned k=3 bound (no segment cost, 22 Ir)", 22.0, 1.0, 0.464, 0.443, 0.0, 1.06, False)
row("partitioned k=3 bound (no segment cost, 22 Ir)", 22.0, 1.0, 0.492, 0.46, 0.0, 1.1, True)
# break-even instruction budget against master
for smt,t in ((False,1.16),(True,2.31)):
    print("break-even Ir/B", "SMT" if smt else "alone", round(t*(IPCc/2 if smt else IPCa)*F,1))
