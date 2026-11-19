"""Use the same balanced driver as the combined study, with compiled edits as fourth arm."""
from pathlib import Path
import sys
d=Path(__file__).resolve().parent
base=d.parent/"combined"
src=(base/"prepare_stage.py").read_text().replace('d.parent/"bundled/open.rs"', 'd.parent/"combined/open.rs"')
src=src.replace("if i in (1,3)","if i in (1,2,3)").replace('    extras="+self.narrow', '''    if i==3: fields.append("edits")
    extras="+self.narrow''')
src=src.replace('(d/"stage.rs")','(d.parent/"combined/stage.rs")')
exec(compile(src, __file__,"exec"), dict(__name__="__main__",__file__=str(d/"prepare_stage.py")))
scratch=Path(sys.argv[1]).resolve()
s=(scratch/"main.rs").read_text().replace('["opend", "ahead", "bundle", "combo"]','["opend", "ahead", "combo", "patch"]')
s=s.replace("output_capacity,retained_bytes,crc","output_capacity,retained_bytes,crc,reason,events,rewinds,patches,patched_spelling_bytes")
s=s.replace('                    let total = kernel + scratch + output;', '''                    let total = kernel + scratch + output;
                    let (reason,events,rewinds,patches,spelling)=if arm==3 {d.observations()} else {(0,0,0,0,0)};''')
s=s.replace('{scratch},{output},{total},{crc}', '{scratch},{output},{total},{crc},{reason},{events},{rewinds},{patches},{spelling}')
(scratch/"main.rs").write_text(s)
adv=(base/"adversarial.rs").read_text().replace('c.counts().4 != 0','c.observations().0 != 0')
(scratch/"adversarial.rs").write_text(adv)
