"""Build/load checked numerical terminal into the frozen real cap scalar/Lazy/Open binary."""
from pathlib import Path
import csv,json,hashlib,subprocess,sys,os
here=Path(__file__).resolve().parent
repo=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
cut=here.parents[3]; donors=cut/'powers/xor/capped'
ref=Path(sys.argv[1]).resolve();wires=Path(sys.argv[2]).resolve();out=Path(sys.argv[3]).resolve();mode=sys.argv[4]
out.mkdir(exist_ok=True,parents=True);exe=ref/'panel';corpus=repo/'corpora/wikidata.bz2'
load=lambda x:json.loads(x.read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=load(donors/'receipt.json')['inputs'];baseline=load(donors/'build.json');actual=load(ref/'build.json')
for k in ['rustc','donor','src_tree','source_sha256','rlib_sha256','rust_flags']:assert actual[k]==baseline[k]
assert sha(exe)==actual['exe_sha256']
if mode=='build':
    (out/'ids.inc').write_text('unsigned lookup(unsigned n,unsigned p,unsigned k){\n'+''.join(
        f"if(n=={q['rows']}u&&p=={q['mark']}u&&k=={q['records']}u)return {q['block']}u;\n" for q in rows)+'std::abort();}\n')
    cmd=['g++','-std=c++20','-O3','-g0','-fPIC','-shared','-Wall','-Wextra','-I',out,here/'terminal.cpp','-o',out/'libgrammar.so']
    subprocess.run(cmd,check=True)
    manifest=dict(source_sha256=sha(here/'terminal.cpp'),ids_sha256=sha(out/'ids.inc'),so_sha256=sha(out/'libgrammar.so'),gxx=subprocess.check_output(['g++','--version'],text=True).splitlines()[0],rust_executable_sha256=sha(exe),rust_build=actual,flags=cmd[1:8],wire='6a80e8a9',gate='3a1d3819')
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
else:
    assert mode in ('gate','price','rss')
    frozen=[json.loads(s) for s in subprocess.check_output(['git','-C',repo,'show',f'3a1d3819:{here.parent.relative_to(repo)}/blocks.jsonl'],text=True).splitlines()]
    for q,v in zip(frozen,rows,strict=True):
        assert q['index']==v['block'] and sha(wires/f"{v['block']}.pos")==q['wire_sha']
        assert sha(ref/f"block-{v['block']}.raw")==q['pre_rle_sha']==v['raw_sha256']
    env={**os.environ,'LD_LIBRARY_PATH':str(out),'POSONLY_WIRES':str(wires),'POSONLY_REFS':str(ref)}
    assert f'libgrammar.so => {out}/libgrammar.so' in subprocess.check_output(['ldd',exe],env=env,text=True)
    if mode=='rss':
        src=Path(ref/'rss')
        assert f'libgrammar.so => {out}/libgrammar.so' in subprocess.check_output(['ldd',src],env=env,text=True)
        import tempfile
        labels=['free','warm','parse','scalar','lazy','opend'];old=['stream','streamgaps','dense','scalar','lazy','opend']
        with tempfile.TemporaryDirectory(dir=out) as tmp,(out/'rss.csv').open('w') as f:
            base=Path(tmp);(base/'sample.bz2').symlink_to(corpus)
            subprocess.run(['bzip2recover','sample.bz2'],cwd=base,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
            w=csv.writer(f);w.writerow(['round','block','arm','mark','rows','caps','pre_hwm_kib','post_hwm_kib'])
            for r in range(3):
                for q in rows:
                    b=q['block']
                    for h in range(6):
                        j=(r+b+h)%6
                        x=list(map(int,subprocess.check_output([src,base/f'rec{b+1:05d}sample.bz2',old[j],ref/f'block-{b}.raw'],text=True,env=env).strip().split(',')))
                        assert x[:3]==[q['mark'],q['rows'],q['records']];w.writerow([r,b,labels[j],*x])
        (out/'rss-manifest.json').write_text(json.dumps({'rust_exe_sha256':sha(src),'so_sha256':sha(out/'libgrammar.so')},indent=2)+'\n')
    else:
        env['POSONLY_STAMPS']=str(out/'stamps.csv')
        subprocess.run([exe,corpus,out,'24' if mode=='price' else '0'],env=env,check=True)
        for v in rows:
            assert sha(out/f"block-{v['block']}.raw")==v['raw_sha256'] and sha(out/f"block-{v['block']}.caps")==v['caps_le_sha256']
        q=list(csv.DictReader((out/'stamps.csv').open()))
        if mode=='gate':
            assert len(q)==30 and all(int(x['free_tree_load_ns'])>0 for x in q)
            (out/'gate.json').write_text(json.dumps({'round':0,'matched_outputs':60,'authenticated_inputs':rows,'so':sha(out/'libgrammar.so')},indent=2)+'\n')
        else:
            z=list(csv.DictReader((out/'times.csv').open()));assert len(z)==1440 and len(q)==750
            x=iter(q[30:]);new=['free','warm','parse'];old=['stream','streamgaps','dense']
            for row in z:
                if row['arm'] in old:
                    j=old.index(row['arm']);t=next(x);assert int(t['arm'])==j and row['block']==t['block'] and int(t['free_tree_load_ns'])==0
                    row['arm']=new[j];row.update({k:v for k,v in t.items() if k not in ['block','arm']})
                    inside=sum(int(t[k]) for k in ['parse_ns','provision_ns','reverse_ns','scatter_ns','rehearsal_ns'])
                    assert inside<=int(row['thread_ns'])
            assert not list(x)
            cols=list(dict.fromkeys(k for row in z for k in row))
            with (out/'paired.csv').open('w') as f:
                wr=csv.DictWriter(f,cols);wr.writeheader();wr.writerows(z)
