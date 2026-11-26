"""Independent dense and deletion/renewal oracles for arbitrary holes in an ordered token BWT."""
from collections import Counter
from itertools import groupby, product
import json


def runs(column):
    return sum(1 for _ in groupby(column))


def ordinary(column):
    # Stable sorting the incoming symbols gives hosts of the increasing successor queues.
    successor=sorted(range(len(column)), key=column.__getitem__)
    source=[column[t] for t in successor]
    return successor,source


def marked(partner,words,mark,offset,bound):
    out=bytearray(words[mark][offset:])
    i=partner[mark]
    while len(out)<bound:
        out.extend(words[i])
        i=partner[i]
    return bytes(out[:bound])


def induced_marked(partner,old_words,H,order,new_partner,new_words,mark,offset,bound):
    prefix=bytearray()
    i=mark
    if offset:
        prefix.extend(old_words[i][offset:])
        i=partner[i]
    # A mark in a discarded cycle can circulate here until the byte budget is filled.
    while i not in H and len(prefix)<bound:
        prefix.extend(old_words[i]);i=partner[i]
    if len(prefix)>=bound:
        return bytes(prefix[:bound])
    return bytes(prefix)+marked(new_partner,new_words,order[i],0,bound-len(prefix))


def bucket_ids(words,partner):  # coarsest contiguous, equal-spelling, stable partition
    ids=[]
    for i,word in enumerate(words):
        boundary=i==0 or word!=words[i-1] or partner[i-1]>=partner[i]
        ids.append((ids[-1] if i else -1)+boundary)
    return ids


def scatter(source,partner):
    result=[None]*len(partner)
    for i,to in enumerate(partner):
        assert result[to] is None
        result[to]=source[i]
    return result


def verify(column,holes,spells,marks=True):
    n=len(column); X=set(holes);H=set(range(n))-X
    section=sorted(H);order={v:i for i,v in enumerate(section)}
    partner,source=ordinary(column)
    old_words=[spells[c] for c in source]
    # Dense independent first returns to the surviving physical ranks.
    to=[];words=[];changed=set()
    for i in section:
        v=partner[i]
        word=bytearray(old_words[i])
        if v in X:changed.add(i)
        while v in X:
            word.extend(old_words[v]);v=partner[v]
        words.append(bytes(word));to.append(order[v])
    assert sorted(to)==list(range(len(section)))
    # Determine the landing slots independently, by *one* forward step out of each hole.
    landings={partner[x] for x in X if partner[x] in H}
    assert {section[to[order[i]]] for i in changed}==landings
    k=len(changed)
    assert len(landings)==k<=len(X)
    # Safe ordered names preserve each unchanged interval and distinguish every splice.
    ids=[]
    for j,i in enumerate(section):
        prev=section[j-1] if j else None
        new=j==0 or i in changed or prev in changed or source[prev]!=source[i]
        ids.append((ids[-1] if j else -1)+new)
    safe=scatter(ids,to)
    minimum_ids=bucket_ids(words,to)
    minimum=scatter(minimum_ids,to)
    assert ordinary(safe)[0]==ordinary(minimum)[0]==to
    fine_to_coarse=dict(zip(ids,minimum_ids))
    assert [fine_to_coarse[x] for x in ids]==minimum_ids

    unaffected={i for i in H if i not in changed}
    by_id={ids[order[i]]:source[i] for i in unaffected}
    represented=set(by_id.values())
    old_count=len(set(column))
    u=len(by_id)-len(represented)
    assert 0<=u<=k
    untouched=[d for d in range(n) if d not in X|landings]
    kept_original=[column[d] for d in untouched]
    kept_renamed=[safe[order[d]] for d in untouched]
    assert [by_id[a] for a in kept_renamed]==kept_original
    # Unchanged occurrences of each original type follow its source order even after
    # arbitrary destination deletions. Its ordered renamed IDs therefore never reverse.
    for c in represented:
        seq=[a for a in kept_renamed if by_id[a]==c]
        assert sorted(seq)==seq
    assert runs(kept_original)<=runs(column)
    assert runs(kept_renamed)<=runs(kept_original)+u
    stated=[None if d in landings else safe[order[d]] for d in section]
    landing_groups=split_gaps=0
    for i,val in enumerate(stated):
        if val is not None or (i and stated[i-1] is None):
            continue
        landing_groups+=1
        j=i+1
        while j<len(stated) and stated[j] is None:
            j+=1
        split_gaps+=i>0 and j<len(stated) and stated[i-1]==stated[j]
    assert runs(safe)==runs(kept_renamed)+k+split_gaps
    assert split_gaps<=landing_groups<=k
    hole_label_intersections=sum(len({column[i] for i in group}) for inside,group in groupby(range(n),key=lambda i:i in X) if inside)
    assert u<=hole_label_intersections
    assert runs(safe)<=runs(kept_renamed)+2*k
    assert len(set(ids))==len(represented)+u+k<=old_count+2*k
    assert runs(minimum)<=runs(safe)<=runs(column)+u+2*k
    assert len(set(minimum_ids))<=len(set(ids))
    # q can include entire removed cycles. Those do not enter a splice.
    deleted_cycles=0
    seen=set()
    for x in X:
        if x in seen:continue
        trace=set();v=x
        while v in X and v not in seen and v not in trace:
            trace.add(v);v=partner[v]
        deleted_cycles+=v in trace
        seen.update(trace)
    if marks:
        bound=2*sum(map(len,old_words))+1
        for mark,w in enumerate(old_words):
            for offset in range(len(w)):
                assert induced_marked(partner,old_words,H,order,to,words,mark,offset,bound)==marked(partner,old_words,mark,offset,bound)
    return dict(KJ=old_count,RJ=runs(column),q=len(X),k=k,u=u,surviving_old_labels=len(represented),
                KH=len(set(ids)),RH=runs(safe),Kmin=len(set(minimum_ids)),Rmin=runs(minimum),
                removed_cycles=deleted_cycles,mark_offsets=sum(map(len,old_words)) if marks else 0)


def cyclic_bwt(text):
    return tuple(text[(s-1)%len(text)] for s in sorted(range(len(text)),key=lambda s:text[s:]+text[:s]))


def sharp(q):
    m=2*q+1
    col=(1,)*m+(0,)*m
    # This is a genuine BWT of a single cyclic text with tied rotations.
    assert cyclic_bwt((0,1)*m)==col
    holes=list(range(m+1,2*m,2))
    z=verify(col,holes,[b'a',b'b'],marks=q<=7)
    assert z['Kmin']==2+2*q and z['Rmin']==2+3*q
    return z


def primitive_family(q):
    m=2*q+1
    n=2*m+1
    col=(1,)*m+(0,)*(m+1)
    p,source=ordinary(col)
    assert p==[(i+m)%n for i in range(n)]
    seq=[];i=0
    while not seq or i!=0:
        seq.append(source[i]);i=p[i]
    assert len(seq)==n and cyclic_bwt(tuple(seq))==col
    z=verify(col,[m+2*j for j in range(1,q+1)],[b'a',b'b'],marks=q<=7)
    assert z['Kmin']==2+2*q and z['Rmin']==2+3*q
    return z


def focused_witnesses():
    # Independently scatter a tiny cycle in which the changed source and landing differ.
    col=(1,0,0);T,F=ordinary(col);H=(0,1);i=1
    assert T==[1,2,0] and T[i]==2 and T[T[i]]==0
    emitted=[bytes([97+F[0]]),bytes([97+F[1],97+F[2]])]
    incoming=scatter(emitted,[1,0])
    assert incoming==[b'ab',b'a']
    # Both ordinary BWT inputs restrict to the same retained a token, but the
    # deleted mark must decode distinct old bytes all the way to the block length.
    tp,_=ordinary((0,1));assert tp==[0,1]
    a=marked(tp,[b'a',b'b'],1,0,9);b=marked(tp,[b'a',b'c'],1,0,9)
    assert a!=b and a==b'b'*9 and b==b'c'*9
    many=verify(tuple([0,1,2]*3),[3,4,5],[b'a',b'b',b'c'])
    assert many['u']==2 and many['removed_cycles']==1
    # A payload dictionary may store identical bytes under different ordered IDs.
    alias_checks=0
    for col in product((0,1,2),repeat=4):
        for mask in range(16):
            verify(col,[i for i in range(4) if mask>>i&1],[b'a',b'a',b'aa'])
            alias_checks+=1
    return dict(changed_source_vs_landing=dict(B='baa',holes=[2],source=1,landing=0,
                                                incoming=[w.decode() for w in incoming]),
                deleted_only_pair=dict(retained='a',first_deleted_output=a.decode(),second_deleted_output=b.decode()),
                one_hole_interval=dict(column='abcabcabc',holes=[3,4,5],result=many),
                equal_payload_id_instances=alias_checks)


def main():
    stats=Counter()
    witness=None
    for alphabet,max_n in [(2,7),(3,5)]:
        for n in range(1,max_n+1):
            for col in product(range(alphabet),repeat=n):
                if alphabet==3 and len(set(col))!=3:continue
                # Unequal and overlapping old byte spellings test cuts inside tokens too.
                spell=[b'a',b'ab',b'ba']
                for mask in range(1<<n):
                    z=verify(col,[j for j in range(n) if mask>>j&1],spell)
                    stats['instances']+=1
                    stats['mark_offsets']+=z['mark_offsets']
                    stats['nonempty_splices']+=z['k']>0
                    stats['cases_with_removed_cycles']+=z['removed_cycles']>0
                    stats['only_hole_cycles_deleted']+=z['q']>0 and z['k']==0
                    stats['empty_result']+=z['q']==n
                    stats['sharply_increases_both']+=z['k']>0 and z['Kmin']==z['KJ']+2*z['k'] and z['Rmin']==z['RJ']+3*z['k']
                    if witness is None and z['q']==1 and z['Kmin']==z['KJ']+2 and z['Rmin']==z['RJ']+3:
                        p,_=ordinary(col);i=p[0];cycle=1
                        while i!=0:cycle+=1;i=p[i]
                        if cycle==n:witness=dict(column=list(col),holes=[j for j in range(n) if mask>>j&1],result=z)
    # Deterministically look beyond the small exhaustive panel for a single-cycle equality.
    for n in range(6,15):
        if witness is not None:break
        for col in product(range(2),repeat=n):
            p,_=ordinary(col);i=p[0];cycle=1
            while i!=0:cycle+=1;i=p[i]
            if cycle!=n:continue
            for x in range(n):
                z=verify(col,[x],[b'a',b'b'],marks=False)
                if z['Kmin']==z['KJ']+2 and z['Rmin']==z['RJ']+3:
                    witness=dict(column=list(col),holes=[x],result=z)
                    break
            if witness is not None:break
    assert witness is not None
    print(json.dumps(dict(exhaustive=stats,primitive_witness=witness,
                          sharp_family={q:sharp(q) for q in (1,2,3,7,31,127)},
                          primitive_family={q:primitive_family(q) for q in (1,2,3,7,31,127)}),indent=2))


if __name__=='__main__':
    import sys
    if sys.argv[1:]==['--witnesses']:
        print(json.dumps(focused_witnesses(),indent=2))
    else:
        main()
