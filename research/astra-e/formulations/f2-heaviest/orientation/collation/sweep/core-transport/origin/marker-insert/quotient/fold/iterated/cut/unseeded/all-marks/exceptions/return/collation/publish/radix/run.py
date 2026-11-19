import csv
import hashlib
import json
from pathlib import Path
import subprocess as sp
import sys

here=Path(__file__).resolve().parent
parent=here.parent
work=Path(sys.argv[1]).resolve()
stage=sys.argv[2]
sha=lambda x:hashlib.sha256(x.read_bytes()).hexdigest()
if stage=='build':
    sp.run([sys.executable,parent/'run.py',work,'build'],check=True)
    pinned=json.loads((parent/'build.json').read_text())
    assert sha(parent/'publish.cpp')==pinned['source']
    code=(parent/'publish.cpp').read_text()
    split=code.index('Column publish(');tail=code.index('void spell(',split)
    code=code[:split]+(here/'kernel.inc').read_text()+'\n'+code[tail:]
    old='Q lookup=0,key_comparisons=0,base_comparisons=0;'
    assert code.count(old)==1
    code=code.replace(old,old+' Q passes=0,sort_items=0,radix_bins=0,fragment_capacity=0;')
    old="<<t.base_comparisons<<'\\n';"
    assert code.count(old)==1
    code=code.replace(old, "<<t.base_comparisons<<','<<t.passes<<','<<t.sort_items<<','<<t.radix_bins<<','<<t.fragment_capacity<<'\\n';")
    (work/'radix.cpp').write_text(code)
    flags=['-std=c++20','-O3','-Wall','-Wextra','-Werror']
    sp.run(['g++',*flags,'-I',work,'-I',parent,work/'radix.cpp','-o',work/'radix'],check=True)
    out={k:sha(f)for k,f in {'generated':work/'radix.cpp','kernel':here/'kernel.inc','parent':parent/'publish.cpp',
                              'executable':work/'radix','driver':here/'run.py'}.items()}
    (work/'radix-build.json').write_text(json.dumps(dict(sha256=out,flags=flags),indent=2)+'\n')
elif stage=='small':
    out=sp.check_output([work/'radix']);assert out==(parent/'small.csv').read_bytes();(work/'radix-small.csv').write_bytes(out)
elif stage=='real':
    raw,wires,ref=map(Path,sys.argv[3:6])
    known=json.loads((parent/'inputs.json').read_text())
    source=list(csv.DictReader((parent/'blocks.csv').open()))
    fields=list(source[0])+['radix_passes','radix_item_scatters','radix_histogram_bins','fragment_capacity']
    tally={}
    with (work/'radix.csv').open('w') as f:
        dest=csv.DictWriter(f,fields,lineterminator='\n');dest.writeheader()
        for pinned,q in zip(known,source,strict=True):
            i,k=pinned['block'],pinned['k'];assert i==int(q['block']) and k==int(q['k'])
            cap=raw/f'{i}.txt';mask=wires/f'{i}.mask{k}';expected=ref/f'block-{i}.raw'
            assert sha(cap)==pinned['original_rle_sha256'] and sha(mask)==pinned['mask_sha256'] and sha(expected)==pinned['marked_sha256']
            wire=work/f'radix-{i}-{k}.col'
            rows=sp.check_output([work/'radix',cap,str(k),mask,wire,expected],text=True).strip().split(',')
            assert sha(wire)==pinned['column_sha256']
            r=dict(zip(fields,map(int,[i,k,*rows]),strict=True))
            for key,v in q.items():
                if key not in ('dest_lookup_comparisons','key_comparisons','base_sort_comparisons'):assert int(v)==r[key]
            assert r['dest_lookup_comparisons']==r['base_sort_comparisons']==0
            assert 0<r['key_comparisons']<r['intersection_pieces']
            assert r['radix_item_scatters']==r['true_bands']*r['radix_passes']
            dest.writerow(r);f.flush()
            t=tally.setdefault(k,{})
            for key in ['true_bands','intersection_pieces','key_comparisons','radix_item_scatters','radix_histogram_bins','fragment_capacity']:
                t[key]=t.get(key,0)+r[key]
            t['logical_fragment_bytes']=t['intersection_pieces']*12
            t['allocated_fragment_bytes']=t['fragment_capacity']*12
            t['max_block_allocated_fragment_bytes']=max(t.get('max_block_allocated_fragment_bytes',0),12*r['fragment_capacity'])
    (work/'radix-summary.json').write_text(json.dumps(tally,indent=2)+'\n')
else:raise ValueError(stage)
