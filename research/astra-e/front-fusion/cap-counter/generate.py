"""Recreate the four counter/first-cap arms from the historical edge and scalar sources."""
from hashlib import sha256
from pathlib import Path
import runpy
import subprocess
import sys
from tempfile import TemporaryDirectory

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
EDGE='badb2aec3624bd65606c429b2e6f79e9d9d0050f'
SCALAR='d0a42980e896211906234c7679b38760ae44a46e'
COUNTER='af0664a4cb7a534c8b7a35a519c85efd9fa09793'
R='research/astra-e/'
J='docs/journal/2026-11-18-open-return/'


def source(pin,path):
    return subprocess.check_output(['git','-C',ROOT,'show',pin+':'+path])


def replace_once(s,old,new):
    assert s.count(old)==1,old
    return s.replace(old,new)


def main(out):
    out.mkdir(exist_ok=True,parents=True)
    with TemporaryDirectory() as work:
        tmp=Path(work)
        for path in [J+'open_return.rs',J+'endpoint.rs',R+'narrow-endpoint/prepare.py',R+'narrow-endpoint/candidate.rs',
                     R+'narrow-short/prepare.py',R+'short-words/endpoint.rs',R+'lookahead/writer.rs',R+'edge-budget/prepare.py']:
            p=tmp/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(source(EDGE,path))
        subprocess.run([sys.executable,'-B',tmp/R/'edge-budget/prepare.py'],check=True)
        ahead=tmp/R/'narrow-short'
        kernel=(ahead/'constructor_ahead.rs').read_text()
        assert sha256(kernel.encode()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
        transform_source=source(COUNTER,R+'narrow-short/cost/queues.py')
        p=tmp/'queues.py';p.write_bytes(transform_source)
        counter=runpy.run_path(str(p))['transform'](kernel)
        # Every counter-changing operation in this section is guarded by a compile-time worker switch.
        start=counter.index('        let mut last = None;')
        stop=counter.index('        assert!(m > 0 && all.iter()',start)
        guarded=counter[start:stop]
        guarded=replace_once(guarded,'if !QUEUE { thin[a]','if !(QUEUE && super::RECOVER) { thin[a]')
        guarded=replace_once(guarded,'to: if QUEUE {','to: if QUEUE && super::RECOVER {')
        guarded=replace_once(guarded,'if !QUEUE || !keep','if !(QUEUE && super::RECOVER) || !keep')
        guarded=replace_once(guarded,'        if QUEUE {\n            // The ordered','        if QUEUE && super::RECOVER {\n            // The ordered')
        kernel=counter[:start]+guarded+counter[stop:]
        scalar_src=source(SCALAR,R+'front-fusion/cap-batch/prepare.py').decode()
        assert 'addition = "        if width == 127 {' in scalar_src
        a=kernel.index('        while i < records.len() &&')
        b=kernel.index('        let end',a)
        kernel=kernel[:a]+'        if !super::SCALAR_CAP || width == 127 {\n'+kernel[a:b]+'        }\n'+kernel[b:]
        kernel=kernel.replace('"../short-words/endpoint.rs"','"../../short-words/endpoint.rs"').replace('"../../../docs/','"../../../../docs/')
        (out/'kernel.rs').write_text(kernel)
        (out/'narrow.rs').write_bytes((ahead/'narrow.rs').read_bytes())
        (out/'adapter.inc.rs').write_bytes((tmp/R/'edge-budget/adapter.inc.rs').read_bytes())
        assert sha256((out/'adapter.inc.rs').read_bytes()).hexdigest()=='2dde2d590cb4684e79b9b5a910c5ebd16ebf4ed87077eaa614980c43fd029104'
        for path,expected in [(R+'edge-budget/gate.rs','23d48115c627f3e0a8233cbbdc63a7e0b2fee04de1be3852140d2aabd6964212'),(R+'short-words/endpoint.rs','09380d30e328d27056735776812e55677a69c21933dc8b56412da75b87ee2d4d'),(J+'segments.rs','6b7e1c516893ba585183a12202b433da450e605a2f122473266d42de112a5fd7')]:
            assert sha256((ROOT/path).read_bytes()).hexdigest()==expected
        for name,s,r in [('edge',False,False),('counter_edge',False,True),('scalar_edge',True,False),('both',True,True)]:
            (out/(name+'.rs')).write_text(f'''//! Frozen scalar first-cap and retained-queue counter factorial arm.
const SCALAR_CAP: bool = {str(s).lower()};
const RECOVER: bool = {str(r).lower()};
#[allow(dead_code)]
#[path = "kernel.rs"]
pub mod kernel;
include!("../../edge-budget/gate.rs");
include!("adapter.inc.rs");
''')


if __name__=='__main__':
    main(Path(sys.argv[1]) if len(sys.argv)>1 else HERE)
