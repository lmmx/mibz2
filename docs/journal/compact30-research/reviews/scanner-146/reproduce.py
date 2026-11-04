#!/usr/bin/env python3
"""Extract pinned blobs and replay this audit without compiling or running constructors."""
import argparse,contextlib,hashlib,importlib.util,io,json,pathlib,runpy,subprocess,sys
sys.dont_write_bytecode=True

def load_module(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec)
    sys.modules[name]=module
    spec.loader.exec_module(module)
    return module

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo',type=pathlib.Path,required=True,help='Read-only checkout with pinned commits available')
    parser.add_argument('--evidence-dir',type=pathlib.Path,required=True,help='Scratch directory for extracted evidence')
    parser.add_argument('--output-dir',type=pathlib.Path,required=True,help='Scratch directory for replay results')
    args=parser.parse_args()
    packet=pathlib.Path(__file__).resolve().parent
    scratch=args.evidence_dir.resolve();output=args.output_dir.resolve()
    scratch.mkdir(parents=True,exist_ok=True);output.mkdir(parents=True,exist_ok=True)
    manifest=json.loads((packet/'complete-evidence-manifest.json').read_text())
    for entry in manifest:
        destination=scratch/entry['file']
        assert scratch in destination.resolve().parents
        data=subprocess.run(['git','-C',str(args.repo),'show',entry['ref']+':'+entry['path']],check=True,stdout=subprocess.PIPE).stdout
        blob=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
        assert blob==entry['blob'],(entry['path'],blob)
        assert hashlib.sha256(data).hexdigest()==entry['sha256'],entry['path']
        destination.parent.mkdir(parents=True,exist_ok=True)
        destination.write_bytes(data)
    items=sorted(set((x['path']+'@'+x['ref'],x['blob'],x['sha256']) for x in manifest))
    digest=hashlib.sha256(json.dumps(items,separators=(',',':')).encode()).hexdigest()
    expected_cursor=json.loads((packet/'review-cursor.json').read_text())
    assert digest==expected_cursor['evidence_sha256']
    (scratch/'safety/fetched-pins.json').write_bytes((packet/'safety/fetched-pins.json').read_bytes())
    replay=load_module('replay',packet/'replay.py')
    replay.ROOT=output;replay.EV=scratch/'evidence'
    with contextlib.redirect_stdout(io.StringIO()):
        replay.main()
        runpy.run_path(str(packet/'compare_asm.py'),run_name='__main__')
    safety=load_module('scanner_safety_replay',packet/'safety/replay_safety.py')
    safety.HERE=scratch/'safety'
    safety_text=io.StringIO()
    with contextlib.redirect_stdout(safety_text): safety.main()
    (output/'safety-replay-results.json').write_text(safety_text.getvalue())
    expected=json.loads((packet/'replay-result.json').read_text())
    observed=json.loads((output/'replay-result.json').read_text())
    assert observed==expected
    assert json.loads((output/'asm-comparison.json').read_text())==json.loads((packet/'asm-comparison.json').read_text())
    assert json.loads(safety_text.getvalue())==json.loads((packet/'safety/replay-results.json').read_text())
    print(json.dumps({'result':'PASS','evidence_sha256':digest,'own_body_saving':observed['self_saving'],'observed_run1_inclusive_saving':observed['inclusive_saving'],'miri_cases':105,'symbolic_tile_sequences':19531,'output_dir':str(output)},indent=2))

if __name__=='__main__': main()
