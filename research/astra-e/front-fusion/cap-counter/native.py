"""Build the unchanged and separately instrumented four post-Huffman worker paths."""
from hashlib import sha256
from pathlib import Path
import argparse
import re
import subprocess

HERE=Path(__file__).resolve().parent
ARMS=['edge','counter_edge','scalar_edge','both']


def native_source(dest,mode):
    dest.mkdir(parents=True,exist_ok=True)
    s=(HERE/'kernel.rs').read_text()
    s=re.sub(r'#\[path = "([^"]+)"\]',lambda m:'#[path = "'+str((HERE/m[1]).resolve())+'"]',s)
    if mode=='meter':
        a='        let mut last = None;'
        b='        assert!(m > 0 && all.iter()'
        assert s.count(a)==s.count(b)==1
        s=s.replace(a,'        let started = crate::cpu();\n'+a)
        s=s.replace(b,'        crate::PHASE.with(|p| p.set(crate::cpu()-started));\n'+b)
    if mode=='adverse':
        a='        assert!(m > 0 && all.iter()'
        assert s.count(a)==1
        s=s.replace(a,'''        crate::take_scan(&all,&thin,m,mark, self.strips.iter().map(|s|(s.from,s.to,s.width,s.byte)).collect());
'''+a)
    (dest/'kernel.rs').write_text(s)
    modules=[]
    for name in ARMS:
        s=(HERE/(name+'.rs')).read_text()
        s=re.sub(r'include!\("([^"]+)"\);',lambda m:'include!("'+str((HERE/m[1]).resolve())+'");',s)
        (dest/(name+'.rs')).write_text(s)
        modules.append(f'mod {name};')
    (dest/'arms.inc.rs').write_text('\n'.join(modules)+'\n')


def main():
    args=argparse.ArgumentParser()
    args.add_argument('output',type=Path)
    args.add_argument('deps',type=Path)
    args.add_argument('modes',nargs='+',choices=['original','meter','adverse'])
    opt=args.parse_args()
    for mode in opt.modes:
        dest=opt.output.resolve()/mode
        native_source(dest,mode)
        prog='adverse' if mode=='adverse' else 'panel'
        (dest/'main.rs').write_bytes((HERE/(prog+'.rs')).read_bytes())
        flags=['rustc','--edition=2021','-C','opt-level=3','-C','codegen-units=1','-C','debuginfo=1',
               '--extern','mibz2='+str(opt.deps.resolve()/'libmibz2.rlib'),'-L','dependency='+str(opt.deps.resolve()),'-o',str(dest/prog),str(dest/'main.rs')]
        subprocess.run(flags,check=True)
        print(mode,sha256((dest/prog).read_bytes()).hexdigest())


if __name__=='__main__':
    main()
