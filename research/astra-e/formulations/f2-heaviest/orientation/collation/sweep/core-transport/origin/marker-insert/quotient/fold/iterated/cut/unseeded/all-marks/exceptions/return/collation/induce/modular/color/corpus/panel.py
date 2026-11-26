"""Price exact physical geometry of all four cuts on independently frozen caps."""
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys

HERE=Path(__file__).resolve().parent
CUT=next(p for p in HERE.parents if (p/'matching/first-cuts.csv').is_file())
sys.path.insert(0,str(CUT/'matching'))
sys.path.insert(0,str(CUT))
import certify

def type_masks(source, mark):
    raw=source.read_bytes()
    present=sorted(set(raw[1::2])); ordinal={a:i for i,a in enumerate(present)}
    runs=[]
    for width,a in zip(raw[::2],raw[1::2],strict=True):
        key=ordinal[a]
        if runs and runs[-1][0]==key: runs[-1]=(key,runs[-1][1]+width)
        else: runs.append((key,width))
    seeded,words,anchor=certify.seed(runs,present,mark)
    bounds=certify.geometry(seeded,len(words))[1]
    slices,edges=certify.overlay(seeded,bounds)
    greedy=certify.choose(edges,anchor)
    matching,_=certify.matching_cut(edges,anchor)
    mask=bytearray()
    for incoming,source,width in slices:
        a=source in greedy and incoming not in greedy
        b=source in matching and incoming not in matching
        mask.extend(bytes([a+2*b])*width)
    return mask

caps,out=map(Path,sys.argv[1:]); out.mkdir(parents=True,exist_ok=True)
donor=json.loads((CUT/'powers/xor/capped/receipt.json').read_text())
prior={int(r['index']):r for r in csv.DictReader((CUT/'matching/first-cuts.csv').open())}
fields='block arm n primary normalized_primary maximal_runs run_image_tiles kept removed kept_spans removed_spans src_atoms image_atoms src_image_atoms consecutive_removed affine_bands affine_word_bands word_runs unique_words unique_spelling_bytes single two max_return spelled_bytes tile_piece_live_bytes map_graph_capacity_bytes'.split()
exe=out/'census'; subprocess.run(['g++','-std=c++20','-Wall','-Wextra','-O2',HERE/'census.cpp','-o',exe],check=True)
with (out/'blocks.csv').open('x') as table:
    w=csv.writer(table,lineterminator='\n');w.writerow(fields)
    for b in donor['inputs']:
        i=b['block']; stem=f'block-{i}'
        source=caps/(stem+'.caps'); raw=caps/(stem+'.raw')
        for path,field in [(source,'caps_le_sha256'),(raw,'raw_sha256')]:
            assert hashlib.sha256(path.read_bytes()).hexdigest()==b[field]
        mask=out/'type.mask';mask.write_bytes(type_masks(source,b['mark']))
        rows=list(csv.DictReader(io.StringIO(subprocess.check_output([exe,source,raw,str(b['mark']),str(i),mask],text=True)),fieldnames=fields))
        assert len(rows)==4
        for r in rows:
            assert int(r['n'])==b['rows'] and int(r['maximal_runs'])==b['maximal_runs']
            if r['arm'].startswith('type'):
                key='ingress_old' if r['arm']=='type-greedy' else 'ingress_matching'
                assert int(r['removed'])==int(prior[i][key])
            w.writerow(r.values())
        table.flush()
    mask.unlink()
