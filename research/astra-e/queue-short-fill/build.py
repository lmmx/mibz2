from pathlib import Path
import argparse
import subprocess
from prepare import prepare,HERE

def build(folder,lib,mode):
    folder=Path(folder).resolve()
    dest=prepare(folder,meter=mode=='meter')
    arms=[]
    for pred in ['edge','both']:
        for k in [0,1,4]:
            nm=f'{pred}{k}'
            mod=(HERE/f'{nm}.rs').read_text().replace('target/generated/',str(dest/'generated')+'/')
            (dest/f'{nm}.rs').write_text(mod)
            arms.append(f'mod {nm};')
    source='#![allow(dead_code)]\n'+'\n'.join(arms)+'\n'+(HERE/'panel.rs').read_text()
    file=dest/'main.rs'
    file.write_text(source)
    lib=Path(lib).resolve()
    subprocess.run(['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1','-C','debuginfo=1',
                    file,'--extern',f'mibz2={lib}','-L',f'dependency={lib.parent}','-o',dest/'panel'],check=True)
if __name__=='__main__':
    a=argparse.ArgumentParser()
    a.add_argument('folder');a.add_argument('lib');a.add_argument('mode',choices=['original','meter'])
    v=a.parse_args();build(v.folder,v.lib,v.mode)
