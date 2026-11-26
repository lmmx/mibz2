import hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
here=Path(__file__).resolve().parent
repo=Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
frozen='d0a42980e896211906234c7679b38760ae44a46e'
native='990450a2'
out,rlib,prepared=map(lambda x:Path(x).resolve(),sys.argv[1:])
out.mkdir(exist_ok=True,parents=True)
sha=lambda p:hashlib.file_digest(p.open('rb'),'sha256').hexdigest()
def read(rev,p):
    return subprocess.check_output(['git','-C',repo,'show',rev+':'+str(p)])
capped=next(repo.glob('research/astra-e/**/cut/powers/xor/capped/build.json'))
config=json.loads(capped.read_text())
assert config['donor']==frozen and sha(rlib)==config['rlib_sha256']
rustc=os.environ.get('RUSTC') or shutil.which('rustc') or '/workspace/.mibz2-tools/rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu/bin/rustc'
assert subprocess.check_output([rustc,'-Vv'],text=True).strip()==config['rustc']
source=out/'scalar';source.mkdir(exist_ok=True)
assert subprocess.check_output(['git','-C',prepared,'rev-parse','HEAD'],text=True).strip()==frozen
for label in ('candidate','kernel.rs','gate.rs','adapter.inc.rs'):
    name='candidate.rs' if label=='candidate' else label
    p=source/name;p.write_bytes((prepared/'research/astra-e/front-fusion/cap-batch/worker'/name).read_bytes())
    assert sha(p)==config['source_sha256'][label]
cpp=out/'constructor';cpp.mkdir(exist_ok=True)
for name in ('bridge.cpp','ranges.hpp','rope.hpp','sweep.hpp'):
    original=(here.parent/name).relative_to(repo)
    (cpp/name).write_bytes(read(native,original))
flags=['-O3','-DNDEBUG','-std=c++20','-fPIC','-shared']
so=out/'libranges.so'
subprocess.run(['g++',*flags,cpp/'bridge.cpp','-o',so],check=True)
main=out/'main.rs';main.write_text('#[path="scalar/candidate.rs"] pub mod candidate;\n'+(here/'driver.rs').read_text())
exe=out/'panel'
subprocess.run([rustc,*config['rust_flags'],main,'-L',f'dependency={rlib.parent}','--extern',f'mibz2={rlib}',
                '-L',f'native={out}','-l','dylib=ranges','-C','link-arg=-Wl,-rpath,$ORIGIN','-o',exe],check=True)
assert f'libranges.so => {so}' in subprocess.check_output(['ldd',exe],text=True)
out.joinpath('build.json').write_text(json.dumps(dict(rust_donor=frozen,native_source=native,rustc=config['rustc'],
    rlib_sha256=sha(rlib),rust_flags=config['rust_flags'],gxx=subprocess.check_output(['g++','--version'],text=True).splitlines()[0],
    cpp_flags=flags,sources={str(p.relative_to(out)):sha(p) for p in [main,*sorted(cpp.iterdir()),*sorted(source.iterdir())]},
    executable=sha(exe),shared=sha(so),processor=next(x.split(':')[1].strip() for x in Path('/proc/cpuinfo').read_text().splitlines() if x.startswith('model name'))),indent=2)+'\n')
