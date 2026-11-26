"""Use the frozen four-arm complete inverse-through-CRC driver."""
from pathlib import Path
import sys
d=Path(__file__).resolve().parent;base=d.parents[1]/"combined"
src=(base/"prepare_stage.py").read_text().replace('d.parent/"bundled/open.rs"','d.parents[1]/"combined/open.rs"')
src=src.replace('repo=d.parents[4]', 'repo=d.parents[5]').replace('ahead=d.parents[2]', 'ahead=d.parents[3]')
src=src.replace("if i>=2:", "if i==2:").replace('if i in (1,3)', 'if i in (1,2,3)')
src=src.replace('    formula=', '''    if i==3:
        fields.extend(["corrected_caps","original"])
        extras+="+self.reader.bytes()"
    formula=''')
src=src.replace('(d/"stage.rs")','(d.parents[1]/"combined/stage.rs")')
src=src.replace('sources[d/"stage.rs"]=', 'sources[d.parents[1]/"combined/stage.rs"]=')
exec(compile(src,__file__,"exec"),{"__name__":"__main__","__file__":str(d/"prepare_stage.py")})
scratch=Path(sys.argv[1]).resolve()
s=(scratch/"main.rs").read_text().replace('["opend", "ahead", "bundle", "combo"]','["opend", "ahead", "combo", "update"]')
s=s.replace("output_capacity,retained_bytes,crc","output_capacity,retained_bytes,crc,reason,events,ranks,explicit,holes")
s=s.replace('                    let total = kernel + scratch + output;', '''                    let total = kernel + scratch + output;
                    let (reason,events,ranks,nodes,holes)=if arm==3 {d.observations()} else {(0,0,0,0,0)};''')
s=s.replace('{scratch},{output},{total},{crc}', '{scratch},{output},{total},{crc},{reason},{events},{ranks},{nodes},{holes}')
(scratch/"main.rs").write_text(s)
s=(base/"adversarial.rs").read_text().replace('c.counts().4 != 0','c.observations().0 != 0')
(scratch/"adversarial.rs").write_text(s)
