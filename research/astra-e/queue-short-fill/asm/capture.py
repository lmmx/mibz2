"""Archive full inlined constructors and the original emitted short/fallback paths."""
from pathlib import Path
import hashlib,lzma,re,subprocess,sys

here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
old=Path(sys.argv[2]).resolve()
binary=out/'worker'
art=Path(sys.argv[3]);art.mkdir(parents=True,exist_ok=True)

def symbols(exe):
    return subprocess.check_output(['nm','-S','-C',exe],text=True)
def dis(exe,start,end,lines=False):
    return subprocess.check_output(['objdump','-dCl' if lines else '-dC','--no-show-raw-insn',
                      '--start-address='+hex(start),'--stop-address='+hex(end),exe],text=True)
def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()
def section(full,ranges):
    result=[]
    for l in full.splitlines():
        m=re.match(r'^\s+([0-9a-f]+):',l)
        if m and any(a<=int(m[1],16)<b for a,b in ranges):
            result.append(l.split(' <')[0].rstrip())
    return '\n'.join(result)+'\n'

matches=re.findall(r'^(\w+) (\w+) t <asm_worker::(\w+)::kernel::Kernel>::construct::<true, true>$',symbols(binary),re.M)
assert len(matches)==6
data=[]
excerpt=[]
manifest=[]
# Offsets below refer to the six-arm binary from the toolchain and source in manifest.
# Rebuilds with different machine code still get their complete source-mapped disassembly.
cuts={
 'edge_range':[(0x22acb,0x22b15),(0x22b80,0x22ba2)],
 'edge_single':[(0x2aaa2,0x2aad6),(0x2ab40,0x2ab8c)],
 'edge_four':[(0x32cc1,0x32d08),(0x32ec1,0x32ef1),(0x32b08,0x32b25),(0x32e80,0x32ebe)],
 'both_range':[(0x1e9f8,0x1ea46),(0x1eab0,0x1eacd)],
 'both_single':[(0x26a14,0x26a5d)],
 'both_four':[(0x2ea3a,0x2ea8a),(0x2ec61,0x2ec8a),(0x2ec9d,0x2ecd4)],
}
for lo,sz,arm in sorted(matches,key=lambda x:x[2]):
    begin=int(lo,16);end=begin+int(sz,16)
    full=dis(binary,begin,end,True)
    data.append(f'\n# {arm}\n'+full)
    excerpt.append(f'\n# {arm}\n'+section(full,cuts[arm]))
    manifest.append(f'{arm} start={lo} size={int(sz,16)} kernel_sha256={digest(out/arm/"kernel.rs")}')
m=re.search(r'^(\w+) (\w+) t <moonshot::candidate::kernel::Kernel>::construct::<true, true>$',symbols(old),re.M)
assert m
origin=dis(old,int(m[1],16),int(m[1],16)+int(m[2],16),True)
data.append('\n# actual scorertype edge\n'+origin)
excerpt.append('\n# actual scorer edge regular keep and marked singleton\n'+section(origin,[(0x3707b,0x370c1),(0x37130,0x37142),(0x37201,0x37240)]))
(art/'constructors.txt.xz').write_bytes(lzma.compress(''.join(data).encode(),preset=6))
(art/'sites.txt').write_text(''.join(excerpt))
manifest+=['worker '+digest(binary),'original '+digest(old),'support library '+digest(out/'libmibz2.rlib'),
           'actual scorer info '+subprocess.check_output([old,'info'],text=True).strip()]
for f in ['worker.py','capture.py','probe.rs','probe.s','capacity.rs']:
    manifest.append(f+' '+digest(here/f))
manifest.append('compressed full disassembly '+digest(art/'constructors.txt.xz'))
manifest.append(subprocess.check_output(['rustc','-vV'],text=True).strip())
manifest.append(subprocess.check_output(['objdump','-v'],text=True).splitlines()[0])
(art/'manifest.txt').write_text('\n'.join(manifest)+'\n')
