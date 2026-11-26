from pathlib import Path
import sys,subprocess
d=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
work.mkdir(parents=True,exist_ok=True)
subprocess.run([sys.executable,str(d/"prepare.py")],check=True)
s=(d/"open.rs").read_text().replace("../../../../../docs",str(d.parents[4]/"docs")).replace('"nav.rs"',f'"{d/"nav.rs"}"')
s=s.replace("    fbase:Bases", "    pub times:[u64;3], fbase:Bases")
s=s.replace("        let (s,mark,bits)=self.construct", "        let t0=crate::cpu(); self.times=[0;3];\n        let (s,mark,bits)=self.construct")
s=s.replace("        self.reason=0;", "        let t1=crate::cpu(); self.times[0]=t1-t0;\n        self.reason=0;")
s=s.replace("        self.owners.clear();","        let t2=crate::cpu();self.times[1]=t2-t1;\n        self.owners.clear();")
s=s.replace("        let mut row = mark;", "        self.times[2]=crate::cpu()-t2;\n        let mut row = mark;")
(work/"phase_open.rs").write_text(s)
s=(d.parent/"measure.rs").read_text().replace('"section.rs"',f'"{d.parent/"section.rs"}"').replace('"open.rs"','"phase_open.rs"').replace('"../../../../docs',f'"{d.parents[4]}/docs')
s=s.replace("&hazard, &mut out)", "&mut out, k)")
s=s.replace("                    assert_eq!(got, want);", "                    assert_eq!(got, want);\n                    sparse.verify_coordinates(&hazard.events,&hazard.danger);")
a=s.index("        if !hazard.build(caps, 1024)")
b=s.index("        old.run::<",a)
s=s[:a]+"        let indexed=hazard.build(caps,1024);\n"+s[b:]
s=s.replace('        verified += n;', '''        if indexed {sparse.verify_coordinates(&hazard.events,&hazard.danger);}
        else {assert_eq!(sparse.counts().4,1);}
        verified+=n;''')
a=s.index("        println!(\n            \"{index},{n},{},{},{},{},{},true")
b=s.index("        if !selected",a)
s=s[:a]+'''        let (events,danger,steps,owners,reason)=sparse.counts();
        println!("{index},{n},{events},{danger},{steps},{owners},{reason},{},{}",sparse.capacities(),out.len());
'''+s[b:]
s=s.replace("stream,rows,events,danger,seal,index_capacity,decoded,took","stream,rows,events,danger,rewinds,owners,fallback_reason,nav_capacity,decoded")
a=s.index("                if arm == 0 {\n                    assert!(hazard.build")
b=s.index("                let t1 = cpu();",a)
s=s[:a]+s[b:]
s=s.replace('construction_walk_ns,unrle_ns,crc_ns','construction_walk_ns,unrle_ns,crc_ns,constructor_ns,navigation_ns,owner_ns')
s=s.replace('                    "{index},{round},{},{wall},{},{},{},{},{}",','                    "{index},{round},{},{wall},{},{},{},{},{},{},{},{}",')
s=s.replace('                    t4 - t3\n','                    t4 - t3,\n                    if arm==0{sparse.times[0]}else{0},if arm==0{sparse.times[1]}else{0},if arm==0{sparse.times[2]}else{0}\n')
(work/"main.rs").write_text(s)
