"""Check original-byte native output/CRC, stage trajectory and a mandatory power fallback."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess
from real import source,rle1,crc32_bzip2


def h(x):return hashlib.sha256(x).hexdigest()


def validate(args):
    table=[json.loads(s) for s in args.accepted.read_text().splitlines()][1:]
    passes={}
    for r in csv.DictReader(args.passes.open()):
        passes.setdefault(r['tape'],[]).append(r)
    data=[]
    args.scratch.mkdir()
    for row in table:
        tape=args.tapes/row['source_tape']
        tag=tape.name.split('.')[0]
        folder=args.scratch/tag;folder.mkdir()
        _,runs,mark,_,_=source(tape)
        raw=folder/'input.txt'
        raw.write_text(str(mark)+'\n'+''.join(f'{k} {n}\n' for k,n in runs))
        subprocess.run([args.binary,raw,'check',folder],check=True)
        out=(folder/'cut.bin').read_bytes()
        plain=rle1(out)
        assert h(out)==row['original_inverse_sha256'] and h(plain)==row['decoded_sha256']
        crc=f'{crc32_bzip2(plain):08x}'
        assert crc==row['block_crc']
        native=list(csv.DictReader((folder/'cut-depths.csv').open()))
        expected=passes[tag]
        assert len(native)==len(expected)
        for a,b in zip(native,expected):
            for x,y in [('depth','depth'),('old_rows','rows'),('self_mass','self_mass'),
                        ('after','after'),('normalized','normalized')]:
                assert int(a[x])==int(b[y])
            if a['normalized']=='0':
                assert int(a['old_ingress'])==int(b['rows'])-int(b['after'])
        data.append(dict(tape=tag,index=row['original_block_index'],rounds=len(native),
                         powers=sum(int(s['normalized']) for s in native),
                         inverse_sha256=h(out),decoded_bytes=len(plain),crc=crc))
    for n in (17,128,16384):
        folder=args.scratch/f'pure{n}';folder.mkdir()
        raw=folder/'input.txt'
        raw.write_text(f'{n}\n98 1\n97 {n}\n')
        subprocess.run([args.binary,raw,'check',folder],check=True)
        assert (folder/'cut.bin').read_bytes()==b'b'+b'a'*n
        trace=list(csv.DictReader((folder/'cut-depths.csv').open()))
        assert len(trace)==1 and trace[0]['normalized']=='1'
    print(json.dumps(dict(binary_sha256=h(args.binary.read_bytes()),ten=data,
                          mandatory_fallback_sizes=[17,128,16384]),indent=2))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('binary','tapes','accepted','passes','scratch'):p.add_argument(name,type=Path)
    validate(p.parse_args())
