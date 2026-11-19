"""Build the exact precounted native worker with the scorer's external-module layout."""
from pathlib import Path
from tempfile import TemporaryDirectory
import re,subprocess,sys
d=Path(__file__).resolve().parent
with TemporaryDirectory() as directory:
    temp=Path(directory)
    subprocess.run([sys.executable,str(d.parent/"prepare.py"),directory],check=True)
    (d/"gate.inc.rs").write_bytes((temp/"gate.rs").read_bytes())
    s=(temp/"kernel.rs").read_text()
    begin=s.index('impl Kernel {\n    pub(super) fn fuse<const TILE')
    end=s.index('impl Kernel {\n    pub(super) fn fuse_edge<const LAZY',begin)
    s=s[:begin]+'#[cfg(target_arch = "x86_64")]\n'+s[end:]
    s=s.replace("crate::front::Bits","super::front::Bits").replace("crate::pre::front_avx2","super::pre::front_avx2")
    (d/"kernel.rs").write_text(s)
    s=(temp/"pre.rs").read_text().replace("use crate::front::Bits;","use super::front::Bits;")
    (d/"symbols.rs").write_text(s.replace('include!("cap_count.rs");','include!("../../cap_count.rs");'))
ahead=d.parents[2]/"narrow-short"
s=(ahead/"adapter.inc.rs").read_text().replace("if r.records().len() * 20 >= n * 3 {","if r.records().len() * 20 >= n * 3 && (n < 65536 || !fits(r.records(),p)) {").replace("    open: kernel::Kernel,", "    open: kernel::Kernel,\n    front: front::Bits,")
a=s.index("    fn invert_runs(")
s=s[:a]+'''    fn invert_symbol_runs(&mut self,b:&mibz2::block::Symbols,k:Kernels,out:&mut Vec<u8>)->bool {
        #[cfg(target_arch="x86_64")]
        if b.syms.len()>=256 && b.max_len<1<<24 && k!=Kernels::scalar()
            && is_x86_feature_detected!("avx2") && is_x86_feature_detected!("bmi1")
            && is_x86_feature_detected!("bmi2") && is_x86_feature_detected!("lzcnt")
            && is_x86_feature_detected!("popcnt") {
            return self.open.fuse_edge::<true>(b,&mut self.front,out).is_some();
        }
        let _=(b,k,out);
        false
    }
'''+s[a:]
(d/"adapter.inc.rs").write_text(s)
