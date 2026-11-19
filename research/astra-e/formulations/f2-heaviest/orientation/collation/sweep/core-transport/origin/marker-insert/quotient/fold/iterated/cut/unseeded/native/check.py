"""Compare native outputs to libbzip2 recovery, the Python oracle, and the two seeded histories."""
import csv
import hashlib
import json
from pathlib import Path
import subprocess
import sys

here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parent.parent/'powers'))
from real import source, rle1, crc32_bzip2, recovered

binary,inputs,corpus,scratch,results=map(lambda x:Path(x).resolve(),sys.argv[1:])
scratch.mkdir(exist_ok=False)
results.mkdir(exist_ok=True)
small=json.loads(subprocess.check_output([binary,'small']))
(results/'tiny.json').write_text(json.dumps(small,indent=2)+'\n')
recover=scratch/'recovered'
recover.mkdir()
(recover/'sample.bz2').symlink_to(corpus)
subprocess.run(['bzip2recover','sample.bz2'],cwd=recover,check=True,capture_output=True)
expected={d['index']:d for d in map(json.loads,(here.parent/'real.jsonl').read_text().splitlines())}
py_passes={}
for r in csv.DictReader((here.parent/'passes.csv').open()):py_passes.setdefault(int(r['index']),[]).append(r)
trajectories=[]
blocks=[]
def h(data):return hashlib.sha256(data).hexdigest()
for tape in sorted(inputs.glob('*.irec.xz')):
    m,runs,mark,_,_=source(tape)
    idx=m['index']
    tag=tape.name.split('.')[0]
    dest=scratch/tag
    dest.mkdir()
    raw=dest/'input.txt'
    raw.write_text(str(mark)+'\n'+''.join(f'{k} {n}\n' for k,n in runs))
    plain,authenticated_crc=recovered(recover,idx)
    raw_output=subprocess.check_output([binary,raw,'check',dest],text=True).splitlines()
    data=dict(tag=tag,index=idx,raw_input_sha256=h(raw.read_bytes()),tape_sha256=h(tape.read_bytes()),
              decoded_sha256=h(plain),crc32=f'{authenticated_crc:08x}',arms={})
    last=None
    for line in raw_output:
        name,*numbers=line.split(',')
        skip,lap,terminal,moves,queries,hops,powers=map(int,numbers)
        out=(dest/(name+'.bin')).read_bytes()
        final=rle1(out)
        assert final==plain and crc32_bzip2(final)==authenticated_crc
        assert h(out)==expected[idx]['pre_rle_sha']
        steps=list(csv.DictReader((dest/(name+'.csv')).open()))
        if name=='unseeded':
            py=expected[idx]
            assert (skip,lap,terminal,moves,powers)==(py['final_skip'],py['lap'],py['terminal_multiplicity'],
                py['moved_marks'],py.get('normalizations',0))
            for v,s in zip(py_passes[idx],steps,strict=True):
                for old,new in dict(depth='depth',old_rows='rows',old_runs='runs',refined_ids='refined',
                                    after='after',published_runs='out_runs',flips='flip',moved_marks='moved').items():
                    assert int(v[old])==int(s[new]),(idx,old,v[old],s[new])
        if name=='split-ordinal':
            for a,b in zip(last,steps,strict=True):
                assert all(a[k]==b[k] for k in a if k!='mark')
        last=steps
        trajectories.extend(dict(tag=tag,arm=name,**s) for s in steps)
        data['arms'][name]=dict(steps=len(steps),skip=skip,lap=lap,terminal_multiplicity=terminal,
                                ordinary_moves=moves,power_runs=powers,power_queries=queries,power_hops=hops)
        data['pre_rle_sha256']=h(out)
    blocks.append(data)
with (results/'trace.csv').open('w',newline='') as out:
    w=csv.DictWriter(out,fieldnames=trajectories[0],lineterminator='\n');w.writeheader();w.writerows(trajectories)
(results/'real.json').write_text(json.dumps(dict(corpus_sha256=h(corpus.read_bytes()),blocks=blocks),indent=2)+'\n')
