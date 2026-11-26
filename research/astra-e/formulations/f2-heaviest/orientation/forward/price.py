"""Position-balanced, four-arm native crossover of select pages and a section-row directory."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import subprocess

from census import Column, Direct, heavy_set, normalize, paths, read_input

HEADER=("round","position","arm","cpu_ms","wall_ms","setup_ms","discover_ms","order_ms","dictionary_ms","emit_ms")

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary",type=Path);p.add_argument("tapes",type=Path);p.add_argument("out",type=Path)
    p.add_argument("--rounds",type=int,default=16)
    a=p.parse_args();assert a.rounds>0 and a.rounds%8==0
    binary=a.binary.resolve()
    with (a.out/"price.csv").open("w") as raw, (a.out/"indexed.csv").open("w") as stats:
        writer=csv.writer(raw,lineterminator="\n");writer.writerow(("ordinal","section",*HEADER))
        summary=None
        for tape in paths(a.tapes):
            ordinal=int(tape.name.split(".")[0])
            runs,mark=read_input(tape);col=Column(runs);parent=col.interval(b'"')
            frequent,_,_=heavy_set(col,parent,1)
            for name,sections in (("quote",[parent]),("frequent",frequent)):
                header=f"{col.rows} {mark} {len(runs)} {len(sections)}\n"
                data=(header+"".join(f"{x} {y}\n" for x,y in [*runs,*sections])).encode()
                info=json.loads(subprocess.check_output([binary,"INDEXED_BANDS"],input=data))
                direct=Direct(col,sections)
                bands=[(x,y,w,bytes.fromhex(word),color) for x,y,w,word,color in info.pop("bands")]
                assert normalize(bands)==normalize(direct.bands)
                out=bytes.fromhex(info.pop("expanded_hex"))
                assert out==direct.expand(mark)
                row=dict(ordinal=ordinal,section=name,output_sha256=hashlib.sha256(out).hexdigest(),**info)
                if summary is None:
                    summary=csv.DictWriter(stats,fieldnames=row,lineterminator="\n");summary.writeheader()
                summary.writerow(row);stats.flush()
                lines=subprocess.check_output([binary,"PRICE",str(a.rounds)],input=data).decode().splitlines()
                assert len(lines)==4*a.rounds
                positions={i:[] for i in range(4)}
                for line in lines:
                    fields=line.split(",")
                    assert len(fields)==len(HEADER)
                    rnd,slot,arm=map(int,fields[:3])
                    assert arm==(3-((slot+rnd+1)%4) if rnd//4%2 else (slot+rnd+1)%4)
                    positions[arm].append(slot)
                    writer.writerow((ordinal,name,*fields))
                assert all(p.count(slot)==a.rounds//4 for p in positions.values() for slot in range(4))
                raw.flush()
                print(ordinal,name,flush=True)

if __name__=="__main__":
    main()
