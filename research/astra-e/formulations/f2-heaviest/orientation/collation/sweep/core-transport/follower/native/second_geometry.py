"""Build a separate untimed observer for materialized sizes in the actual second induction."""
from pathlib import Path
import subprocess
import sys
from generate import main as materialize, once

out,deps=map(Path,sys.argv[1:])
materialize(out)
p=out/"second.rs"
p.write_text(p.read_text()+"""
impl Second{
    pub(super) fn census(&self)->[usize;7]{
        [self.edges.len(),self.nodes.len(),self.words.len(),self.dictionary.len(),self.selectors.len(),
         self.merged.len(),self.links.len()]
    }
}
""")
for module in ("control","shared","projected","follower"):
    p=out/f"{module}.rs"
    p.write_text(p.read_text()+"""
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn information(&self)->[usize;7]{self.second.census()}
}
""")
p=out/"main.rs"
s=p.read_text()
s=once(s,'if mode=="price"{println!("stream,rep,arm,','if mode=="geometry"{println!("stream,arm,second_lookups,second_nodes,second_words,second_dict_bytes,second_selectors,second_merge_pieces,second_links");}\n    else if mode=="price"{println!("stream,rep,arm,')
s=once(s,'if arm<16 && mode!="price"{continue;}','if arm<16 && mode!="price" && mode!="geometry"{continue;}')
s=once(s,"            if arm>=16{","""            if mode=="geometry"{
                assert!(r.second);
                let s=if arm== -1{sub.named.information()}else if arm<16{sub.original.information()}else{sub.physical.information()};
                println!("{},{},{},{},{},{},{},{},{}",x.index,name(arm),s[0],s[1],s[2],s[3],s[4],s[5],s[6]);
                continue;
            }
            if arm>=16{""")
p.write_text(s)
subprocess.run(["rustc","--edition=2021","-A","unexpected_cfgs","-A","unused_imports","-C","opt-level=3",
                "-C","codegen-units=1","-C","debug-assertions=yes","--cfg","clipped_profile",str(p),
                "--extern",f"mibz2={deps}/libmibz2.rlib","-L",f"dependency={deps}","-o",str(out/"geometry")],check=True)
