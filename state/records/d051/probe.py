"""Independent finite permutation check of Prepared's adjacency/termination premise.
Not Rust execution, SIMD/Miri, or reproduction of the reported long fuzz.
"""
import itertools, json

cases=segments=0
for n in range(1,7):
    for psi in itertools.permutations(range(n)):
        lf=[0]*n
        for i,j in enumerate(psi): lf[j]=i
        for mask in range(1,1<<n):
            requested=[i for i in range(n) if mask>>i&1]
            for r0 in requested:
                # Independently express setup's simultaneous deletion rule as a set.
                dropped=set()
                for s in requested:
                    if lf[s] in requested and lf[s]!=s:
                        dropped.add(s if lf[s]==r0 else lf[s])
                starts=set(requested)-dropped
                assert r0 in starts
                assert all(lf[s]==s or lf[s] not in starts for s in starts)
                marks={lf[s]:s for s in starts if lf[s]!=s}
                pieces={}
                for s in starts:
                    # Pair lanes emit two rows at take; subsequent steps end at a start
                    # or at its preceding LF row (one byte emitted on that path).
                    rows=[s,psi[s]]
                    p=psi[psi[s]]
                    for _ in range(n+1):
                        if p in starts:
                            successor=p
                            break
                        if p in marks:
                            rows.append(p)
                            successor=marks[p]
                            break
                        rows.extend([p,psi[p]])
                        p=psi[psi[p]]
                    else: raise AssertionError('nonterminating pair lane')
                    # Single-row definition, checked independently of flags.
                    expected=[]
                    r=s
                    for _ in rows:
                        expected.append(r); r=psi[r]
                    assert rows==expected and r==successor
                    pieces[s]=(rows,successor)
                    segments+=1
                out=[]; s=r0
                while len(out)<n:
                    rows,s=pieces[s]
                    out.extend(rows[:n-len(out)])
                expected=[]; p=r0
                for _ in range(n): expected.append(p); p=psi[p]
                assert out==expected
                cases+=1

# The 13-byte end-cell guard witness: unchanged guard sends it to scalar.
n=q_end=13
q=0; length=13
guard=n-q>=8 and length<=n-q-8
mutant=n-q>=8 and length<=n-q
assert not guard and mutant
assert 8+8>13
print(json.dumps({'scope':'permutation/flag finite model, not Rust',
 'max_n':6,'prepared_cases':cases,'lane_segments':segments,
 'failures':0,'guard_witness':{'n':13,'cell_length':13,'baseline':'scalar',
 'weakened_guard_second_read':[8,16],'input_end':13}},indent=2))
