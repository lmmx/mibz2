"""Insert the verified scanner only into a detached experiment clone, preserving all stock APIs."""
from pathlib import Path
import argparse
import hashlib
import subprocess

PIN="d0a42980e896211906234c7679b38760ae44a46e"
PACKED="df6ad1d33b301fa8b6b593cff6a5e965cd9e9dcbf30edb5eabb57238869ef171"
def sha(f):return hashlib.sha256(f.read_bytes()).hexdigest()
def git(tree,*args):return subprocess.check_output(["git","-C",tree,*args],text=True).strip()

def install(tree,scanner):
    assert git(tree,"rev-parse","HEAD")==PIN and git(tree,"rev-parse","--abbrev-ref","HEAD")=="HEAD"
    assert sha(scanner)==PACKED
    file=tree/"src/block.rs"
    assert sha(file)=="c0e0257373c71a8526612e77561f286a73dd912ec637b09618e610dd90b61407"
    original=file.read_text()
    generated=scanner.read_text()
    a=generated.index("struct Huff {")
    b=generated.index("pub fn read_item(")
    c=generated.index('#[target_feature(enable="bmi2")]',b)
    kernel=generated[a:b]+generated[c:].replace("unsafe fn read_bmi2(","pub(super) unsafe fn read_bmi2(",1)
    # Keep the shared stock Item and Symbols types. The bounded scanner's ordinary
    # header parser refers to the identical upstream constants and error/reader types.
    module='''
#[cfg(target_arch="x86_64")]
mod phrase {
    use super::{Bits, Reg, ErrorKind, Item, Symbols, BLOCK_MAGIC, END_MAGIC,
                MAX_ALPHA, MAX_CODE_LEN, MAX_GROUPS, MAX_SELECTORS, MAX_SYMS, GROUP};
'''+kernel+'\n}\n'
    start=original.index("pub fn read_item(")
    body=original.index("    macro_rules! fail",start)
    sig=original[start:body]
    wrapper=sig+'''    #[cfg(target_arch="x86_64")]
    if std::is_x86_feature_detected!("bmi2") {
        // SAFETY: this CPU supports the BMI2 scatter used inside this private scanner.
        return unsafe { phrase::read_bmi2(br,level,syms) };
    }
    read_stock(br,level,syms)
}
'''
    file.write_text(original[:start]+wrapper+sig.replace("pub fn read_item(","fn read_stock(")+original[body:]+module)
    print(f"frozen scanner {sha(scanner)}\nintegrated block.rs {sha(file)}")

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("checkout",type=lambda s:Path(s).resolve())
    p.add_argument("scanner",type=lambda s:Path(s).resolve())
    a=p.parse_args()
    install(a.checkout,a.scanner)
