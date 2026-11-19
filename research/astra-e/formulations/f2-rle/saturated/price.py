from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
subprocess.run([sys.executable,str(d/"prepare.py")],check=True)
s=(d.parent/"measure.rs").read_text().replace('"section.rs"',f'"{d.parent/"diagonal/section.rs"}"').replace('"open.rs"',f'"{d/"open.rs"}"').replace('"../../../../docs',f'"{d.parents[4]}/docs')
s=s.replace("&hazard, &mut out)", "&hazard, &mut out, k)")
s=s.replace("decoded,took", "decoded,took,steps,owners,worked").replace(",,,,,,false",",,,,,,false,,,")
s=s.replace('        println!(\n            "{index},{n},{},{},{},{},{},true",','''        let (steps,owners,worked)=sparse.pricing();
        println!(
            "{index},{n},{},{},{},{},{},true,{steps},{owners},{worked}",''')
Path(sys.argv[1]).write_text(s)
