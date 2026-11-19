"""Power normalization followed by a provably shrinking stable delimiter cut."""
from collections import Counter
import hashlib
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parent.parent))
from cut import Fold, check_step, choose, geometry, overlay, seed, split
from grammar import Node, output
from normalize import normalize


def solve(raw,literals,mark,skip,horizon,detail=None,check=True):
    words=[Node(literal=s) for s in literals]
    raw,words,anchor=seed(raw,words,mark)
    stats=Counter(rounds=0,normalized_rounds=0,old_rows=0,old_runs=0,old_ids=0,
                  initial_rows=sum(n for _,n in raw),normal_written_rows=0,normal_pieces=0,
                  lookups=0,normal_types=0,normal_runs=0,powers=0,omitted_fixed=0,
                  macro_cut_input_rows=0,cut_added_aliases=0,joins=0,cut_refined_runs=0,
                  cut_refined_ids=0,cut_target_stream_runs=0,max_power=1,max_norm_scatter=0,
                  scanned_fragments=0,checked_cut_partners=0)
    while True:
        freq,bounds=geometry(raw,len(words))
        source=bounds[anchor]
        assert freq[anchor]==1
        cursor=0
        for k,n in raw:
            if k==anchor:
                assert n==1
                break
            cursor+=n
        if source==cursor:
            break
        old_m=bounds[-1]
        slices,edges=overlay(raw,bounds)
        self_mass=sum(targets.get(k,0) for k,targets in enumerate(edges))
        stats.update(old_rows=old_m,old_ids=len(words),old_runs=len(raw),scanned_fragments=len(slices))
        if self_mass:
            raw,words,anchor,c=normalize(raw,words,anchor,bounds,slices,check)
            stats.update(normalized_rounds=1,normal_written_rows=c['macro_rows'],
                         normal_pieces=c['affine_pieces'],lookups=c['point_lookups'],
                         normal_types=c['macro_types'],normal_runs=c['macro_runs'],
                         powers=c['power_nodes'],omitted_fixed=c['omitted_fixed_rows'])
            stats['max_power']=max(stats['max_power'],c['largest_power'])
            stats['max_norm_scatter']=max(stats['max_norm_scatter'],c['macro_rows'])
            freq,bounds=geometry(raw,len(words))
            source=bounds[anchor]
            slices,edges=overlay(raw,bounds)
            stats['scanned_fragments']+=len(slices)
            assert all(k not in targets for k,targets in enumerate(edges))
        else:
            c={}
        macro_m=bounds[-1]
        assert macro_m==old_m-self_mass
        target=choose(edges,anchor)
        refined,rwords,chosen,a=split(slices,edges,words,target,anchor)
        rf,rb=geometry(refined,len(rwords))
        f=Fold(refined,dict(enumerate(rwords)),list(range(len(rwords))),dict(enumerate(rf)),protected=a,chosen=chosen)
        assert rb[a]==source and f.m*4<=macro_m*3 and a in f.ids
        if check:
            checked,newsource=check_step(raw,bounds,words,refined,rb,rwords,f,source,target)
            assert f.bases[f.ids[a]]==newsource
            stats['checked_cut_partners']+=checked
        if detail is not None:
            detail(dict(depth=stats['rounds'],rows=old_m,self_mass=self_mass,macro_rows=macro_m,
                        normalized=int(bool(c)),affine_pieces=c.get('affine_pieces',0),
                        source_fragments=c.get('start_fragments',0),macro_types=len(words),
                        macro_runs=len(raw),lookups=c.get('point_lookups',0),powers=c.get('power_nodes',0),
                        largest=c.get('largest_power',1),fixed_omitted=c.get('omitted_fixed_rows',0),
                        refined_ids=len(rwords),refined_runs=len(refined),joins=len(chosen),after=f.m))
        stats.update(macro_cut_input_rows=macro_m,joins=len(chosen),
                     cut_added_aliases=len(rwords)-len(words),cut_refined_runs=len(refined),
                     cut_refined_ids=len(rwords),cut_target_stream_runs=f.slices,rounds=1)
        raw,words,anchor=f.root,f.weights,f.ids[a]
    result,counts=output(words[anchor],skip,horizon)
    stats.update(counts)
    stats.update(terminal_rows=bounds[-1],lap=words[anchor].n,output_bytes=len(result))
    assert stats['old_rows']<=4*stats['initial_rows'] and stats['lookups']<=stats['old_rows']
    return result,stats


def signature(data):
    return hashlib.sha256(data).hexdigest()
