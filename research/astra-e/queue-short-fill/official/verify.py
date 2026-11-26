"""Replay the archived gate verdict, source provenance and scorer projection."""
import hashlib
import json
from pathlib import Path
import subprocess

base=Path(__file__).resolve().parent
checkout=base.parents[3]
snapshot=json.loads((base/'hashes.json').read_text())
report=json.loads((base/'score/summary.json').read_text())
info=json.loads((base/'binary-info.json').read_text())
assert report['info']==info and info['cpus']==1
assert report['options']['no_timing'] and report['options']['no_proxy'] and not report['options']['quick']
assert report['gate']=={'verdict':'PASS','cases':338,'failed':[],'exit':0}
lines=[json.loads(s) for s in (base/'score/check.jsonl').read_text().splitlines()]
assert lines[-1]=={'arm':'candidate','verdict':'PASS','failures':0}
cases=lines[:-1]
assert len(cases)==338 and all(x['pass'] and x['failure'] is None for x in cases)
assert any(x['case'].startswith('wikidata ') and x['bytes']==209700196 for x in cases)
assert any('600 mutated inputs' in x['case'] for x in cases)
assert 'scorer v1 at '+report['scorer']['commit'] in (base/'stdout.txt').read_text()
def git(*args):
    return subprocess.check_output(['git','-C',checkout,*args])
entry='research/astra-e/queue-short-fill/both1.rs'
assert hashlib.sha256(git('show',snapshot['candidate_revision']+':'+entry)).hexdigest()==snapshot['entry_sha256']
saved={x.split(maxsplit=3)[3]:x.split()[1] for x in (base/'scorer-blobs.txt').read_text().splitlines()}
original={x.split(maxsplit=3)[3]:x.split()[2] for x in git('ls-tree','-r',snapshot['refreshed_origin_master'],'--','bench/moonshot').decode().splitlines()}
assert saved==original and len(saved)==snapshot['scorer_checkout_verified_files']
assert git('log','-1','--format=%H',snapshot['refreshed_origin_master'],'--','bench/moonshot').decode().strip()==snapshot['scorer_path_revision']
print(f"{len(cases)} full cases passed; 209700196 Wikidata bytes and 600 mutated inputs verified")
print(f"{len(saved)} exact scorer blobs at path revision {snapshot['scorer_path_revision']}")
print(f"printed scorer v1 at {report['scorer']['commit']}; candidate {snapshot['candidate_revision']}")
print(f"binary SHA-256 {snapshot['binary_sha256']}")
