"""Use the unchanged stock scalar/Lazy/Open binary and verify its actual DSO before timing."""
from pathlib import Path
import os,csv,subprocess,sys,hashlib,json
here=Path(__file__).resolve().parent
src,exe,corpus,out=map(lambda x:Path(x).resolve(),sys.argv[1:5])
mode=sys.argv[5];out.mkdir(parents=True,exist_ok=True)
digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
expected=json.loads((here.parent/'powers/xor/capped/build.json').read_text())
build=json.loads((exe.parent/'build.json').read_text())
for k in ['rustc','donor','src_tree','source_sha256','rlib_sha256','rust_flags']:
    assert build[k]==expected[k],k
assert digest(exe)==build['exe_sha256']
if mode=='build':
    s=(src/'raw.cpp').read_text()
    assert s.count('int main(int argc,char** argv)')==1
    (src/'raw-lib.inc').write_text(s.replace('int main(int argc,char** argv)','int raw_geometry_main(int argc,char** argv)'))
    so=out/'libgrammar.so'
    subprocess.run(['g++','-O3','-std=c++20','-g0','-Wno-return-type','-fPIC','-shared','-I',src,here/'caps.cpp','-o',so],check=True)
    (out/'manifest.json').write_text(json.dumps({'rust_executable':exe.name,'rust_exe_sha256':digest(exe),'inherited_rust_build':build,'matches_archived_executable':digest(exe)==expected['exe_sha256'],'cxx_so_sha256':digest(so),'bridge_sha256':digest(here/'caps.cpp'),'raw_inc_sha256':digest(src/'raw-lib.inc'),'corpus_sha256':digest(corpus)},indent=2)+'\n')
else:
    assert mode=='price'
    env={**os.environ,'LD_LIBRARY_PATH':str(out),'FUSED_STAMPS':str(out/'stamps.csv')}
    trace=subprocess.check_output(['ldd',exe],env=env,text=True)
    assert f'libgrammar.so => {out}/libgrammar.so' in trace
    (out/'ldd.txt').write_text(trace)
    subprocess.run([exe,corpus,out,'24'],env=env,check=True)
    rows=list(csv.DictReader((out/'times.csv').open()));stamps=list(csv.DictReader((out/'stamps.csv').open()))
    assert len(rows)==1440 and len(stamps)==750
    donors=['stream','streamgaps','dense'];labels=sys.argv[6:] or ['compact_stream','joint','original'];assert len(labels)==3
    i=30;seen={}
    for q in rows:
        if q['arm'] in donors:
            j=donors.index(q['arm']);p=stamps[i];i+=1
            assert int(p['arm'])==j and all(p[k]==q[k] for k in ['rows','mark','cap_records'])
            q.update(arm=labels[j],first_paid_ns=p['first_paid_ns'],cut_only_ns=p['cut_only_ns'])
            assert 0<int(q['cut_only_ns'])<=int(q['first_paid_ns'])<=int(q['thread_ns'])
        else:q.update(first_paid_ns='',cut_only_ns='')
    assert i==750
    source=json.loads((here.parent/'powers/xor/capped/receipt.json').read_text())
    assert digest(corpus)==source['corpus_sha256']
    for v in source['inputs']:
        assert digest(out/f"block-{v['block']}.raw")==v['raw_sha256']
        assert digest(out/f"block-{v['block']}.caps")==v['caps_le_sha256']
    (out/'inputs.json').write_text(json.dumps(source['inputs'],indent=2)+'\n')
    with (out/'capped.csv').open('w') as f:
        w=csv.DictWriter(f,list(rows[0]));w.writeheader();w.writerows(rows)
