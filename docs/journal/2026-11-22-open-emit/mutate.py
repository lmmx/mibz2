"""Compile the actual emit tests against each original and single-site mutant."""
from pathlib import Path
import subprocess, sys, tempfile
a,b=map(Path,sys.argv[1:])
master=(a/'src/open.rs').read_text()
parts=['const INLINE: usize = 16;','const LONG: u8 = u8::MAX;']
for start,end in [('fn inline_words(', '/// The walk over'),('fn emit_reference(', '#[cfg(test)]\nfn records_of')]:
    parts.append(master.split(start,1)[1].split(end,1)[0])
    parts[-1]=start+parts[-1]
for root,name,before,after in [
    (a,'two_ended','if front == back {\n        let middle','if false {\n        let middle'),
    (b,'relay','out[ea..ea + keep].copy_from_slice(&saved[t][..keep]);','let _ = keep;'),
]:
    source=(root/f'src/open/{name}.rs').read_text().replace('//!','//')
    assert source.count(before)==1
    with tempfile.TemporaryDirectory() as d:
        for label,body in [('original',source),('mutant',source.replace(before,after))]:
            p=Path(d)/'test.rs'
            p.write_text('mod open {\n'+'\n'.join(parts)+'\nmod '+name+' {\n'+body+'\n}}')
            binary=Path(d)/'tests'
            subprocess.run(['rustc','--edition','2021','--test','-O',p,'-o',binary],check=True)
            r=subprocess.run([binary,'segment_boundaries_and_meetings'],capture_output=True,text=True)
            print(name,label,r.returncode,r.stdout)
            assert (r.returncode==0)==(label=='original')
