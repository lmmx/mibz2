from array import array
import csv
import io
from pathlib import Path
import subprocess
import sys
import tempfile
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'backwards'))
from encode import real,serialize_parts
import hashlib,json

def run(exe,tapes,wires,depth_file,export_dir=None):
    digest={r['index']:r['wire_sha'] for r in map(json.loads,(Path(__file__).resolve().parents[2]/'backwards'/'blocks.jsonl').read_text().splitlines())}
    out=csv.writer(sys.stdout,lineterminator='\n')
    out.writerow(('index','k','n','states','edges','exceptions','coordinate_visits','longest_default','chunk4','tail4','chunk8','tail8','chunk16','tail16'))
    with tempfile.TemporaryDirectory() as td,depth_file.open('x') as depth_output:
        wf=csv.writer(depth_output,lineterminator='\n');wf.writerow(('index','k','depth','rows','needed'))
        for tape in sorted(tapes.glob('*.irec.xz')):
            meta,runs,mark,raw,words=real.source(tape)
            idx=meta['index'];wire=wires/f'{idx}.pos'
            if not wire.is_file():
                wire=Path(td)/f'{idx}.pos';assert serialize_parts(raw,words,mark,wire) is not None
            assert hashlib.sha256(wire.read_bytes()).hexdigest()==digest[idx]
            last=b''.join(bytes([k])*count for k,count in runs)
            leg=Path(td)/'live'
            assert sys.byteorder=='little'
            args=[exe,wire,leg]
            if export_dir:
                export_dir.mkdir(parents=True,exist_ok=True)
                args.append(export_dir/str(idx))
            s=subprocess.check_output(args,input=array('I',(len(last),mark)).tobytes()+last)
            for row in csv.reader(io.StringIO(s.decode())):out.writerow((idx,*row))
            with leg.open() as f:
                rd=csv.reader(f);next(rd);wf.writerows((idx,*r) for r in rd)

if __name__=='__main__': run(*map(Path,sys.argv[1:]))
