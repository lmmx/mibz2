"""Five-way retained-object stage panel on unchanged sections and independent reference bytes."""
import csv
import sys
import tempfile
import subprocess
import argparse
from pathlib import Path

D=Path(__file__).resolve().parent
sys.path.insert(0,str(D.parent))
from real import paths,read_input,dense_audit,check_frozen

HEAD='ordinal,section,rep,arm,n,tokens,bands,words,runs,dictionary_bytes,mark,skip,used_second,triple_probes,sort_comparisons,assembly_peak_bytes,assembly_s,second_s,total_s'
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary",type=Path)
    parser.add_argument("tapes",type=Path)
    parser.add_argument("csv",type=Path)
    parser.add_argument("--reps",type=int,default=20)
    a=parser.parse_args()
    manifest={int(r['ordinal']):r for r in csv.DictReader((D.parent/'inputs.csv').open())}
    a.csv.write_text(HEAD+'\n')
    with tempfile.TemporaryDirectory() as tmp:
        buf,raw=Path(tmp,'runs.txt'),Path(tmp,'marked')
        for file in paths(a.tapes):
            ordinal=int(file.name.split('.')[0]);m=manifest[ordinal]
            runs,p=read_input(file);col,out,_=dense_audit(runs,p);check_frozen(ordinal,col,out,p)
            raw.write_bytes(out)
            with buf.open('w') as f:
                print(len(col),p,len(runs),2,file=f)
                f.writelines(f'{b} {w}\n' for b,w in runs)
                for s in ('quote','frequent'):print(s,1,m[s+'_lo'],m[s+'_hi'],file=f)
            subprocess.run([a.binary.resolve(),buf,raw,str(ordinal),str(a.reps),a.csv],check=True)
            print(f'verified block {ordinal:04}',file=sys.stderr,flush=True)
if __name__=='__main__':main()
