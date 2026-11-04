#!/usr/bin/env python3
"""Materialize only manifest-listed immutable blobs from a local Git checkout.

No branch checkout, fetch, source execution, build, benchmark, or profile capture.
The checkout must already contain the manifest's pinned commit objects.
"""
import argparse, hashlib, json, pathlib, subprocess

ap=argparse.ArgumentParser()
ap.add_argument('--repo',type=pathlib.Path,required=True)
ap.add_argument('--output',type=pathlib.Path,default=pathlib.Path(__file__).parent/'evidence')
args=ap.parse_args()
manifest=json.loads((pathlib.Path(__file__).parent/'manifest.json').read_text())
for item in manifest:
    commit=item['commit']; path=item['path']
    data=subprocess.check_output(['git','-C',str(args.repo),'show',f'{commit}:{path}'])
    sha=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
    assert sha==item['blob'],(commit,path,sha,item['blob'])
    dest=args.output/item['local'].removeprefix('evidence/')
    dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
print(f'Verified {len(manifest)} pinned blobs.')
