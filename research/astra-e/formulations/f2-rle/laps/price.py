from pathlib import Path
import sys,subprocess
d=Path(__file__).resolve().parent
base=d.parent/"compiled"
work=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(d/"prepare.py")],check=True)
subprocess.run([sys.executable,str(base/"price.py"),str(work)],check=True)
def links(text):
    return text.replace("../../../../../docs",str(d.parents[4]/"docs")).replace('"../compiled/nav.rs"',f'"{base/"nav.rs"}"').replace('"nav.rs"',f'"{base/"nav.rs"}"').replace('"walk.rs"',f'"{d/"walk.rs"}"')
(work/"lap.rs").write_text(links((d/"open.rs").read_text()))
(work/"plain.rs").write_text(links((base/"open.rs").read_text()))
p=work/"main.rs"
s=p.read_text().replace('"phase_open.rs"','"lap.rs"')
s=s.replace("mod sparse;","mod sparse;\n#[path=\"plain.rs\"]mod plain;")
s=s.replace("let mut sparse = sparse::Kernel::default();","let mut sparse=sparse::Kernel::default();\n    let mut plain=plain::Kernel::default();")
s=s.replace('0..=20','0..=21').replace('0..2 {','0..3 {').replace('(round + offset) % 2','(round+offset)%3')
s=s.replace('["sparse", "opend"][arm]', '["laps", "opend","compiled"][arm]')
a=s.index('                let t1 = cpu();')
b=s.index('                let t2 = cpu();',a)
s=s[:a]+'''                let t1=cpu();
                if arm==0{sparse.expanded(caps,n,p,&mut out,k).unwrap();}
                else if arm==1{old.run::<true,true,false>(caps,n,p,16,&mut raw);}
                else{plain.expanded(caps,n,p,&mut out,k).unwrap();}
'''+s[b:]
s=s.replace('                if arm != 0 {','                if arm == 1 {')
s=s.replace('if arm==0{sparse.times[0]}else{0},if arm==0{sparse.times[1]}else{0},if arm==0{sparse.times[2]}else{0}', '0,0,0')
(work/"main.rs").write_text(s)
