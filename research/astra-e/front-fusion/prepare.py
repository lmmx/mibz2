"""Pin the AVX2 front and the narrow/ahead inverse. Emit standalone research modules."""
from pathlib import Path
from hashlib import sha256
import os,re,sys,subprocess
d=Path(__file__).resolve().parent
scratch=Path(sys.argv[1]).resolve();scratch.mkdir(parents=True,exist_ok=True)
root=d.parents[2];ref="80d8409359214b4e5abfd4430213921e82e9cd30"
source=subprocess.check_output(["git","show",ref+":src/runs.rs"],cwd=root).decode()
assert sha256(source.encode()).hexdigest()=="eb04f43a03435d112948f25d1fa1d467f2ad048e9830c4891f236f61ee70bb58"
ahead=d.parent/"narrow-short"
subprocess.run([sys.executable,str(ahead/"prepare.py")],check=True)
base=(ahead/"constructor_ahead.rs").read_text()
assert sha256(base.encode()).hexdigest()=="c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f"
base=re.sub(r'#\[path\s*=\s*"([^"]+)"\]',lambda m:'#[path="'+str((ahead/m[1]).resolve())+'"]',base)
# Use the shipping split/queues after the front has supplied all first-pass quantities.
a=base.index("    fn construct<const")
b=base.index("        assert!(m > 0",a)
e=base.index("    pub fn run<const",b)
special=base[b:e]
special=special.replace("if !INDUCE {","if false {").replace("if INDUCE {","if true {").replace("if QUEUE {","if true {")
# The unqueued branch is statically unreachable and needs the original records. Remove it.
j=special.index("        } else {\n            let mut stream = starts(thin);")
end=special.index("        (stats, mark, bits)",j)
special=special[:j]+"        }\n"+special[end:]
special='''    fn complete(&mut self,p:Parts)->(Counts,u32,u32) {
        let Parts{all,thin,thick,m,mark,pos:n,..}=p;
        let mut stats=Counts{n:n as usize,..Counts::default()};
'''+special
base=base[:e]+special+base[e:]
(scratch/"kernel.rs").write_text(base+(d/"geometry.rs").read_text())
source=source[source.index("/// `src/mtf.rs`'s first level"):source.index("/// The block's LF table from its records")]
source=source.replace("unsafe fn front_avx2(b: &Symbols, f: &mut Runs)", "pub(super) unsafe fn front_avx2(b: &Symbols, f: &mut Bits, mut visit: impl FnMut(u8,u32)->bool)")
start=source.index("    // a record for each literal")
end=source.index("    let litp = ",start)
source=source[:start]+"    let mut row_count=0usize;\n"+source[end:]
a=source.index("            let recs = total.div_ceil");b=source.index("        }};",a)
source=source[:a]+'''            if total!=0 {
                if total>b.max_len as u64 || !visit(c as u8,total as u32) {return None;}
                row_count+=total as usize;
            }
'''+source[b:]
a=source.index("                // SAFETY: the literal's own record");b=source.index("            }\n",a)
source=source[:a]+'''                if !visit(c as u8,total as u32){return None;}
                row_count+=total as usize;
'''+source[b:]
a=source.index("    // (every `mtf` check on the length");source=source[:a]+"    (row_count<=b.max_len).then_some(row_count)\n}\n"
(scratch/"front.rs").write_text("use mibz2::block::Symbols;\nconst CAP:u32=127;\n#[derive(Default)] pub(super) struct Bits{pub(super)lit:Vec<u64>,pub(super)one:Vec<u64>}\n"+source)
(scratch/"main.rs").write_bytes((d/"main.rs").read_bytes())

(scratch/"adversarial.rs").write_bytes((d/"adversarial.rs").read_bytes())
