"""Replay the frozen three-target projection panel and authenticate the same-cap donors."""
import csv,hashlib,json,os
from pathlib import Path
import subprocess,sys,tempfile
HERE=Path(__file__).resolve().parent
REPO=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip())
ref,wires,out=map(lambda v:Path(v).resolve(),sys.argv[1:4])
mode=sys.argv[4];out.mkdir(exist_ok=True,parents=True)
donors=next(REPO.glob('research/astra-e/**/powers/xor/capped/receipt.json')).parent
blocks=json.loads((donors/'receipt.json').read_text())['inputs']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return json.loads(p.read_text())
def write(p,x):p.write_text(json.dumps(x,indent=2)+'\n')
base=read(ref/'build.json')
for k in ['rustc','donor','src_tree','source_sha256','rlib_sha256','rust_flags']:
    assert base[k]==read(donors/'build.json')[k]
assert sha(ref/'panel')==base['exe_sha256']
corpus=REPO/'corpora/wikidata.bz2'
assert sha(corpus)=='82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915'
if mode=='build':
    with (out/'training.json').open('w') as f:
        subprocess.run([sys.executable,HERE/'train.py',out,wires/'1.mask8'],stdout=f,check=True)
    assert read(out/'training.json')['dictionaries_sha']=='7f500cbec1ce3af655014a93dd485468eed2870f10b0786b2b21db4a2e16a39f'
    flags=['-std=c++20','-O3','-mbmi2','-mpopcnt','-fPIC','-shared','-Wall','-Wextra']
    subprocess.run(['g++',*flags,'-I',out,'-I',out/'jumps/native',HERE/'worker.cpp','-o',out/'libgrammar.so'],check=True)
else:
    env=os.environ|{'LD_LIBRARY_PATH':str(out)}
    binary=ref/('rss' if mode=='rss' else 'panel')
    assert 'libgrammar.so => '+str(out/'libgrammar.so') in subprocess.check_output(['ldd',binary],env=env,text=True)
    if mode=='rss':
        old=['stream','streamgaps','dense','scalar','lazy','opend']
        with tempfile.TemporaryDirectory() as td,(out/'rss.csv').open('x') as fh:
            tmp=Path(td);(tmp/'sample.bz2').symlink_to(corpus)
            subprocess.run(['bzip2recover','sample.bz2'],cwd=tmp,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
            w=csv.writer(fh,lineterminator='\n');w.writerow(('repeat','index','arm','primary','n','caps','pre_hwm_kib','peak_kib'))
            for rep in range(3):
                for b in blocks:
                    idx=b['block']
                    for slot in range(6):
                        a=(idx+slot+rep)%6
                        vals=list(map(int,subprocess.check_output([binary,tmp/f'rec{idx+1:05d}sample.bz2',old[a],ref/f'block-{idx}.raw'],env=env,text=True).split(',')))
                        assert vals[:3]==[b[k] for k in ('mark','rows','records')]
                        w.writerow((rep,idx,old[a],*vals))
    elif mode in ('gate','paid'):
        target=out/mode;target.mkdir(exist_ok=True)
        if mode=='paid':
            stamp=target/'stamps.csv';assert not stamp.exists();env['FROZEN_JUMP_LOG']=str(stamp)
        subprocess.run([binary,corpus,target,'0' if mode=='gate' else '24'],env=env,check=True)
        for b in blocks:
            assert sha(target/f"block-{b['block']}.raw")==b['raw_sha256']
            assert sha(target/f"block-{b['block']}.caps")==b['caps_le_sha256']
    elif mode=='manifest':
        write(out/'manifest.json',dict(corpus_sha=sha(corpus),blocks=blocks,cpp_sha=sha(HERE/'worker.cpp'),so_sha=sha(out/'libgrammar.so'),compiler=subprocess.check_output(['g++','--version'],text=True).splitlines()[0],rust=base,training=read(out/'training.json'),sources=['de650248','8e3e6732','0eb27930'],affinity=sorted(os.sched_getaffinity(0)),cpu_quota=Path('/sys/fs/cgroup/cpu.max').read_text().strip(),host_note='Native CPU4 pinned. Independent CPU0 and CPU2 research measurements could overlap. No full-host wall claim.'))
    else:raise ValueError(mode)
