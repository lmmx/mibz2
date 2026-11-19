"""Dense first-return oracle for the global terminal-boundary claim."""
from collections import Counter, defaultdict
from itertools import groupby, product
import json


def runs(values):
    result=[]
    for value,sub in groupby(values):
        width=sum(1 for _ in sub)
        lo=result[-1][1] if result else 0
        result.append((lo,lo+width,value))
    return result


def tag_cap_records(segments,cap,low,high):
    """Reference adapter on counted raw runs; no dense LF destinations are consulted."""
    freq=Counter()
    for lo,hi,c in segments:freq[c]+=hi-lo
    cursor={}
    total=0
    for c in sorted(freq):
        cursor[c]=total
        total+=freq[c]
    output=[]
    physical_refinements=0
    for lo,hi,c in segments:
        while lo<hi:
            length=min(cap,hi-lo)
            a,b=cursor[c],cursor[c]+length
            cuts=[a,*[e for e in (low,high) if a<e<b],b]
            physical_refinements+=len(cuts)-2
            for x,y in zip(cuts,cuts[1:]):
                value=(c,low<=x<high)
                if output and output[-1][2]==value:
                    output[-1]=(output[-1][0],lo+y-a,value)
                else:
                    output.append((lo+x-a,lo+y-a,value))
            cursor[c]=b;lo+=length
    return output,physical_refinements


def audit(last,lo,hi,totals,witness):
    n=len(last); width=hi-lo
    psi=sorted(range(n),key=last.__getitem__); lf=[None]*n
    for a,b in enumerate(psi):lf[b]=a
    f=[last[x] for x in psi]
    segments=runs(last)
    run_of={x:i for i,(b,e,_) in enumerate(segments) for x in range(b,e)}
    inside=lambda x:lo<=x<hi
    global_cuts={}
    for e in (lo,hi):
        if 0<e<n and f[e-1]==f[e]:
            x,y=psi[e-1],psi[e]
            global_cuts[e]=(x,y)
    run_cuts={e:y for e,(x,y) in global_cuts.items() if x+1==y}
    partial_runs={run_of[y] for y in run_cuts.values()}
    tag=[(c,inside(lf[x])) for x,c in enumerate(last)]
    tagged=runs(tag)
    assert len(tagged)==len(segments)+len(run_cuts)
    assert {x for x in range(1,n) if last[x-1]==last[x] and tag[x-1]!=tag[x]}==set(run_cuts.values())
    for cap in (1,2,3,127):
        normalized, splits=tag_cap_records(segments,cap,lo,hi)
        assert normalized==tagged
        assert splits==sum((y-segments[run_of[y]][0])%cap!=0 for y in run_cuts.values())

    # This routes individual original destination rows back to U. It does not
    # march the interval trie or use the clipped implementation's selectors.
    parent_by_row={}; branch=defaultdict(list); incoming=[]
    for dest in range(lo,hi):
        x=dest
        # Store backward steps as a path; order matters but orientation is just a name.
        path=()
        while True:
            assert x not in parent_by_row
            parent_by_row[x]=path
            c=last[x];y=lf[x]
            branch[path,c].append((x,y,inside(y)))
            path=path+(c,)
            if inside(y):
                incoming.append(tuple(reversed(path)))
                break
            x=y
    by_parent=defaultdict(list)
    for x,path in parent_by_row.items():by_parent[path].append(x)
    for path,rows in by_parent.items():
        assert max(rows)-min(rows)+1==len(rows)
    by_boundary={}; mixed_aggregate=[]; piece_mixed=0
    for (path,c), entries in branch.items():
        entries.sort()
        rows=[x for x,y,t in entries]
        target=[y for x,y,t in entries]
        assert target==list(range(target[0],target[-1]+1)) and len(target)<=width
        assert all(tag[x][1]==t for x,y,t in entries)
        # Restrict each physical input run to this parent interval.
        for _,piece in groupby(entries,key=lambda e:run_of[e[0]]):
            piece_mixed+=len({t for x,y,t in piece})==2
        if any(t for x,y,t in entries) and not all(t for x,y,t in entries):
            inner=[e for e in (lo,hi) if target[0]<e<=target[-1]]
            assert len(inner)==1
            e=inner[0]
            assert e not in by_boundary
            by_boundary[e]=(path,c)
            mixed_aggregate.append({"edge":e,"parent":list(path),"byte":c,"inputs":rows,
                                    "arrivals":target})
    expected={e:(parent_by_row[x],last[x]) for e,(x,y) in global_cuts.items()
              if x in parent_by_row and y in parent_by_row and parent_by_row[x]==parent_by_row[y]}
    assert expected==by_boundary
    assert piece_mixed==len(set(by_boundary)&set(run_cuts))
    root_runs=len(runs(incoming))
    assert root_runs<=len(tagged)
    name=lambda seq: "".join(map(str,seq))
    def preserve(key):
        if key not in witness:
            witness[key]=dict(last=name(last),section=[lo,hi],byte_runs=len(segments),
                         physical_interior_cuts=run_cuts,full_input_runs_split=len(partial_runs),
                         mixed_parent_run_pieces=piece_mixed,mixed_branches=mixed_aggregate,
                         terminal_tagged_runs=len(tagged),incoming_token_runs=root_runs,
                         destination_word_labels=[name(w) for w in incoming])
    if len(by_boundary)==2:preserve("two_mixed")
    if by_boundary and piece_mixed==0:preserve("mixed_aggregate_only")
    if run_cuts and not by_boundary:preserve("retagged_but_no_mixed_branch")
    if len(run_cuts)==2 and len(partial_runs)==1:preserve("two_cuts_inside_one_full_run")
    if len(by_boundary)==2 and len(run_cuts)==2 and len(partial_runs)==1:
        preserve("two_affine_cuts_in_one_full_run_seen_in_different_branches")
    if root_runs==len(segments)+2:preserve("two_extra_root_runs")
    if len(by_boundary)==2 and len(run_cuts)==0:
        preserve("two_mixed_aggregates_without_input_split")
    if width==1:
        assert not by_boundary and root_runs==1
        totals["singletons"]+=1
    totals["sections"]+=1
    totals[f"mixed_branches_{len(by_boundary)}"]+=1
    totals[f"physical_cuts_{len(run_cuts)}"]+=1
    totals[f"mixed_parent_run_pieces_{piece_mixed}"]+=1
    totals["max_F_symbols_touched"]=max(
        totals["max_F_symbols_touched"],len(set(f[lo:hi])))


if __name__=="__main__":
    totals=Counter();witness={}
    for alphabet,limit in ((2,9),(3,7)):
        for n in range(1,limit+1):
            for last in product(range(alphabet),repeat=n):
                if alphabet==3 and 2 not in last:continue
                for lo in range(n):
                    for hi in range(lo+1,n+1):
                        audit(last,lo,hi,totals,witness)
    # Real cap boundaries and a broad section crossing the entire byte alphabet.
    wide=Counter()
    for last,sections in [
        ([0]*387,[(127,254),(126,255),(121,128),(130,131)]),
        ([1]*150+[0]*301+[1]*109,[(128,255),(129,257),(298,464),(60,497)]),
        (list(reversed(range(256))),[(0,256),(1,255),(4,245),(17,18)])]:
        for a,b in sections:audit(last,a,b,wide,witness)
    print(json.dumps(dict(exhaustive=totals,wide=wide,first_witnesses=witness),indent=2,sort_keys=True))
