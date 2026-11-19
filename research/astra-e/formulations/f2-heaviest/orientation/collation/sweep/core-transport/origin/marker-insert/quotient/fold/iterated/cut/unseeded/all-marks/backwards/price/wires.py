"""Regenerate exactly the immutable source and reference SHA-256 of ten POSONLY1 roots."""
from pathlib import Path
import sys,hashlib,subprocess,json,tarfile,io,time
here=Path(__file__).resolve().parent
repo=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
out=Path(sys.argv[1]).resolve();out.mkdir(parents=True,exist_ok=True)
rev='6a80e8a9';ref='3a1d3819';here0=here.parent.relative_to(repo)
files=subprocess.check_output(['git','-C',repo,'ls-tree','-r','--name-only',rev,'research/astra-e'],text=True).splitlines()
py=[f for f in files if f.endswith('.py')]
archive=subprocess.check_output(['git','-C',repo,'archive',rev,'--',*py])
with tarfile.open(fileobj=io.BytesIO(archive)) as t:t.extractall(out/'source',filter='data')
sys.path.insert(0,str(out/'source'/here0));sys.dont_write_bytecode=True
from encode import serialize
rows=[json.loads(x) for x in subprocess.check_output(['git','-C',repo,'show',f'{ref}:{here0}/blocks.jsonl'],text=True).splitlines()]
for q in rows:
    tape=repo/'research/astra-e/dense/target/wikidata'/q['source'];dst=out/f'{q["index"]}.pos'
    assert hashlib.sha256(tape.read_bytes()).hexdigest()==q['source_sha']
    t=time.process_time_ns()
    if not dst.exists():serialize(tape,dst)
    dt=time.process_time_ns()-t
    assert dst.stat().st_size==q['wire_bytes'] and hashlib.sha256(dst.read_bytes()).hexdigest()==q['wire_sha']
    print(json.dumps(dict(block=q['index'],file=dst.name,wire_bytes=q['wire_bytes'],wire_sha=q['wire_sha'],mark_bytes=q['rows'],
                          constructor_wall_excluded=True,regenerate_or_check_process_ns=dt)),flush=True)
