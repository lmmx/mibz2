from pathlib import Path
from hashlib import sha256
import subprocess,csv,sys,json,io
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parent/'powers'))
from real import rle1,crc32_bzip2
out=Path(sys.argv[1]).resolve();cases=Path(sys.argv[2]).resolve();dst=Path(sys.argv[3]).resolve();dst.mkdir(exist_ok=True,parents=True)
ref=[json.loads(x) for x in (here.parent/'powers/gated/python.jsonl').read_text().splitlines()][1:]
coverage=[];table=[]
for row in ref:
    tag=row['source_tape'].split('.')[0];folder=out/tag;folder.mkdir(exist_ok=True)
    file=cases/tag/'input.txt'
    q=subprocess.check_output([out/'fused','check',file,folder],text=True)
    for v in csv.DictReader(io.StringIO(q)):
        table.append({'block':tag,**v})
    y=(folder/'marked.bin').read_bytes();plain=rle1(y)
    crc=f'{crc32_bzip2(plain):08x}';digest=sha256(y).hexdigest();assert crc==row['block_crc'] and digest==row['original_inverse_sha256']
    assert sha256(plain).hexdigest()==row['decoded_sha256'] and len(plain)==row['decoded_bytes']
    coverage.append(dict(block=tag,index=row['original_block_index'],inverse_bytes=len(y),decoded_bytes=len(plain),crc=crc,
                         inverse_sha256=digest,decoded_sha256=row['decoded_sha256'],input_sha256=sha256(file.read_bytes()).hexdigest()))
with (dst/'trajectory.csv').open('w') as f:
    w=csv.DictWriter(f,list(table[0]));w.writeheader();w.writerows(table)
(dst/'gate.json').write_text(json.dumps(coverage,indent=2)+'\n')
for a in [0,1,2,3,4,6]:
    with (dst/f'rss-{a}.csv').open('w') as f:subprocess.run([out/'fused','memory',cases,str(a)],stdout=f,check=True)
files=[out/n for n in ['fused','fusion.cpp','stream-fused.hpp','xor/stream.hpp','xor/route.cpp','direct_cut.cpp','native.cpp','interval.hpp']]
(dst/'manifest.json').write_text(json.dumps({'source_sha256':{str(f.relative_to(out)):sha256(f.read_bytes()).hexdigest() for f in files},'donor':'0dd79ba81bb5474d60f113e3de82c027f2351d50','stream':'37c3bfd','g++':subprocess.check_output(['g++','--version'],text=True).splitlines()[0],'flags':'-std=c++20 -O3 -g0 -Wall -Wextra -Wno-return-type'},indent=2)+'\n')
