"""Use stage2's fixed sample and both clocks; change only the arm roster."""
from pathlib import Path

here = Path(__file__).resolve().parent
source = (here.parent / 'stage2/experiment.rs').read_text()
a = source.index('#[allow(dead_code)]')
b = source.index('const ORDINALS:', a)
modules = {
    'open_return': '../../../docs/journal/2026-11-18-open-return/open_return.rs',
    'short_words': '../short-words/candidate.rs', 'endpoint16': '../narrow-endpoint/candidate.rs',
    'combined': 'candidate.rs', 'ahead': 'ahead.rs',
}
source = source[:a] + ''.join(f'#[allow(dead_code)]\n#[path = "{p}"]\nmod {name};\n' for name, p in modules.items()) + source[b:]
a = source.index('    let mut arms = [')
b = source.index('    let count = arms.len();', a)
arms = {'lazy': 'bwt::LazyCells', 'eager': 'bwt::Cells', 'opend16': 'open_return::Open<16, true, true>',
        **{name: name + '::Candidate' for name in modules if name != 'open_return'}}
source = source[:a] + '    let mut arms = [\n' + ''.join(f'        arm::<{typ}>("{name}", kernels),\n' for name, typ in arms.items()) + '    ];\n' + source[b:]
(here / 'stage.rs').write_text(source)
