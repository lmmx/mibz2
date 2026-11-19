"""Generate isolated Candidate modules from the workstation-measured scalar-cap source."""
from pathlib import Path
import hashlib
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PIN = 'd0a42980e896211906234c7679b38760ae44a46e'


def prepare(dest):
    dest = Path(dest).resolve()
    dest.mkdir(parents=True, exist_ok=True)
    ref = dest / 'frozen'
    ref.mkdir(exist_ok=True)
    astra = Path('research/astra-e')
    paths = [astra / name for name in ('front-fusion/cap-batch', 'narrow-short',
             'narrow-endpoint', 'short-words', 'lookahead', 'edge-budget')]
    p = subprocess.Popen(['git', '-C', ROOT, 'archive', PIN, 'docs/journal/2026-11-18-open-return', *paths],
                         stdout=subprocess.PIPE)
    subprocess.run(['tar', '-x', '-C', ref], stdin=p.stdout, check=True)
    assert p.wait() == 0
    base = dest / 'scalar-cap'
    subprocess.run([sys.executable, '-B', ref / astra / 'front-fusion/cap-batch/prepare.py', base],
                   cwd=ROOT, check=True)
    kernel = (base / 'scalar.rs').read_text()
    narrow_path = ref / astra / 'narrow-short/narrow.rs'
    source = narrow_path.read_text()
    assert hashlib.sha256(source.encode()).hexdigest() == '252301a541ee67617a447dc1755ab39d337ab9975e43816345a2fa6296d58224'
    start, end = source.index('    fn eliminate('), source.index('    fn append(')
    body = (HERE / 'eliminate.inc.rs').read_text()
    for arm in ('control', 'batched', 'count'):
        sub = dest / arm
        sub.mkdir(exist_ok=True)
        candidate = kernel
        if arm != 'control':
            impl = body
            if arm == 'count':
                impl = impl.replace('// OBSERVE_STRICT', 'crate::observe_batch(SIDE, count, first == after_winner, width);')
                impl = impl.replace('// OBSERVE_SELF', 'crate::observe_batch(3, 1, false, w);')
                impl = impl.replace('// OBSERVE_EQUAL', 'crate::observe_batch(2, 1, false, w);')
            new = sub / 'narrow.rs'
            new.write_text(source[:start] + impl + source[end:])
            assert candidate.count(str(narrow_path)) == 1
            candidate = candidate.replace(str(narrow_path), str(new))
        (sub / 'kernel.rs').write_text(candidate)
        entry = (ref / astra / 'front-fusion/cap-batch/worker/candidate.rs').read_text()
        entry = entry.replace('mod kernel;', 'pub mod kernel;')
        entry = entry.replace('path="kernel.rs"', f'path="{sub / "kernel.rs"}"')
        for name in ('adapter.inc.rs', 'gate.rs'):
            entry = entry.replace(f'include!("{name}")', f'include!("{base / name}")')
        (dest / f'{arm}.rs').write_text(entry)

if __name__ == '__main__':
    prepare(sys.argv[1])
