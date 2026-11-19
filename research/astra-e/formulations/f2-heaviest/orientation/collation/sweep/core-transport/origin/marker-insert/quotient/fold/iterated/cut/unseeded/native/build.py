"""Specialize the frozen donor at three publication events without editing the donor."""
from pathlib import Path
import subprocess
import sys

here = Path(__file__).resolve().parent
powers = here.parent.parent / 'powers'
build = Path(sys.argv[1]).resolve()
build.mkdir(parents=True,exist_ok=True)

def donor(name):
    root = Path(subprocess.check_output(['git','-C',here,'rev-parse','--show-toplevel'],text=True).strip())
    old = subprocess.check_output(['git','-C',root,'show',
        '37c3bfd5e0f27161b38bb3d3d08d442c692778a6:'+str((powers/name).relative_to(root))])
    return old.decode()

def replace(s,old,new):
    assert s.count(old)==1, old
    return s.replace(old,new)

# The exact fragment construction and alias ordering remain in the donor.
for dep in ('native.cpp','direct_cut.cpp','interval.hpp'):
    (build/dep).write_text(donor(dep))
interval = donor('interval.hpp')
interval = interval[interval.index('Result interval_seeded'):interval.index('Result interval(const')]
interval = replace(interval,'Result interval_seeded(const Root& in)',
    'template<class Observer> Result observed_interval(const Root& in, Observer& observer)')
interval = replace(interval,'    Result out;','    observer.power_start(in,b,raw);\n    Result out;')
interval = replace(interval,'            if(k!=owner) {','            if(k!=owner) {\n                observer.power_range(y,n);')
(build/'observed-interval.hpp').write_text(interval)

compact = donor('xor/route.cpp')
compact = compact[compact.index('template<bool Gaps,class Colors>'):compact.index('template<bool Exact,')]
compact = replace(compact,'template<bool Gaps,class Colors> Root publish_compact',
                  'template<bool Gaps,class Colors,class Observer> Root observed_compact')
compact = replace(compact,'const Colors& color)', 'const Colors& color,Observer& observer)')
compact = replace(compact,'for(auto& s:parts) {','auto classify = [&](Slice& s) {')
compact = replace(compact,'        }\n        for(U j=0;j<K;++j)if(deleted[j])rem[first[j]]-=deleted[j];',
'''        };
        const U local = observer.classify(parts,deleted,classify);
        U before = observer.discount(parts,into,deleted,first,rem);
''')
compact = replace(compact,'out.mark=id[first[in.mark]];',
'''out.mark=observer.finish(in,grammar,parts,color,first,id,into,local,before);''')
(build/'observed-compact.hpp').write_text(compact)
subprocess.run(['g++','-std=c++20','-O3','-g0','-Wall','-Wextra','-I',str(build),
                str(here/'run.cpp'),'-o',str(build/'run')],check=True)
