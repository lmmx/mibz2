from pathlib import Path
import hashlib, re, subprocess, sys

here = Path(__file__).resolve().parent
root = Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'], text=True).strip())
out = Path(sys.argv[1]).resolve()
deps = Path(sys.argv[2]).resolve()
out.mkdir(parents=True, exist_ok=True)
subprocess.run([sys.executable,'-B',here.parents[1]/'native/generate.py',out],check=True)
control = out/'control.rs'
assert hashlib.sha256(control.read_bytes()).hexdigest()=='01663fb4fa08e977bf775b925565e7f71a1faf3f822f02d9122d1278fc7529ab'
assert hashlib.sha256((out/'second.rs').read_bytes()).hexdigest()=='523bee3c587e17bda12437918e2b5f45951df64a0435820fe045b05683fda52a'
control.write_text(control.read_text()+'\n'+(here/'grammar.rs').read_text())
main=out/'main.rs'
s=main.read_text().split('struct Origins {')[0]
# The frozen joint constructor on the 228 paid admissions, with no alteration.
kernel='research/astra-e/front-fusion/cap-counter/kernel.rs'
joint='50b8ecb8946a498601e3fd894ed84a2687df6cc8'
base=out/'joint'
def materialize(rel):
    dest=base/rel
    if dest.exists():return
    dest.parent.mkdir(parents=True,exist_ok=True)
    raw=subprocess.check_output(['git','-C',root,'show',f'{joint}:{rel}']).decode()
    dest.write_text(raw)
    for path in re.findall(r'#\[path = "([^"]+)"\]',raw):
        from os.path import normpath
        materialize(normpath(str(Path(rel).parent/path)))
materialize(kernel)
s+='mod joint { const SCALAR_CAP:bool=true;const RECOVER:bool=true; #[path="'+str(base/kernel)+'"] pub mod kernel; }\n'
main.write_text(s+(here/'main.rs').read_text())
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports',
                '-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',main,
                '--extern',f'mibz2={deps}/libmibz2.rlib','-L',f'dependency={deps}','-o',out/'grammar'],check=True)
