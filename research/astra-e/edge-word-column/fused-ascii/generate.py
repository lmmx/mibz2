"""Generate the exact change against the i9-measured edge; never patch shipping sources."""
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
REV = 'badb2aec3624bd65606c429b2e6f79e9d9d0050f'
ASTRA = Path('research/astra-e')
KERNEL = ASTRA / 'narrow-short/constructor_ahead.rs'
EXPECTED = 'c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'


def frozen(out):
    out.mkdir(parents=True, exist_ok=True)
    paths = ['src', 'docs/journal/2026-11-18-open-return']
    paths += [str(ASTRA / d) for d in ('edge-budget', 'narrow-short', 'narrow-endpoint',
                                       'short-words', 'lookahead')]
    p = subprocess.Popen(['git', '-C', ROOT, 'archive', REV, *paths], stdout=subprocess.PIPE)
    subprocess.run(['tar', '-x', '-C', out], stdin=p.stdout, check=True)
    assert p.wait() == 0
    subprocess.run([sys.executable, out/ASTRA/'edge-budget/prepare.py'], check=True)
    assert hashlib.sha256((out/KERNEL).read_bytes()).hexdigest() == EXPECTED


def modify(source, fused):
    def replace(a, b):
        nonlocal source
        assert source.count(a) == 1, a
        source = source.replace(a, b)
    fields = '    pair_first: u32,\n    pair_allowed: bool,\n'
    if fused:
        fields += '    pair_dir: PairDirectory,\n'
    replace('    edges: Vec<u32>,', fields+'    edges: Vec<u32>,')
    active = '''        self.pair_first = 0;
        let first = 256 + stats.words as u32;
        if INDUCE && QUEUE && self.pair_allowed && limit == 16 && !wide && first + 16383 <= (u32::MAX >> bits) {
            self.pair_first = first;
'''
    if fused:
        active += '''            self.pair_dir.write(&self.bands, &cursors, &self.queues, m, mark, bits, first, &mut self.edges);
            return (stats, mark, bits);
'''
    a = '        stats.table_bytes = 4 * (self.edges.len() + self.extra.len());'
    replace(a, a+'\n'+active+'        }')
    if not fused:
        a = '        (stats, mark, bits)\n    }'
        replace(a, '''        if self.pair_first != 0 {contract::<false>(&mut self.edges, mark, bits, self.pair_first);}
'''+a)
        source += (ROOT/ASTRA/'pairs/contract.rs').read_text()
    else:
        source += (HERE/'queue.rs').read_text()
    a = '        let (stats, mark, bits) = self.construct::<INDUCE, QUEUE>(records,'
    replace(a, '        self.pair_allowed = !SEGMENTS;\n'+a)
    a = '        if SEGMENTS {\n            self.segments.emit('
    replace(a, '''        if self.pair_first != 0 {
            emit_pair(&self.edges, &self.spelling, &self.offsets, n, mark, bits, self.pair_first, out);
        } else if SEGMENTS {
            self.segments.emit(''')
    a = '        let (_, mark, bits) = self.construct::<INDUCE, QUEUE>(runs.records(),'
    replace(a, '        self.pair_allowed = true;\n'+a)
    begin=source.index('        let start = std::time::Instant::now();',source.index('    pub fn price<'))
    end=source.index('        (built, start.elapsed()',begin)
    chunk=source[begin:end]
    source=source[:begin]+chunk.replace('        emit(','''        if self.pair_first != 0 {
            emit_pair(&self.edges, &self.spelling, &self.offsets, n, mark, bits, self.pair_first, out);
        } else { emit(''').rstrip()+'}\n'+source[end:]
    return source+(HERE/'emit.rs').read_text()


def materialize():
    target = HERE/'generated'
    target.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as scratch:
        depot=Path(scratch)
        frozen(depot)
        src=modify((depot/KERNEL).read_text(), True)
        src=src.replace('../short-words/endpoint.rs','../../../short-words/endpoint.rs')
        src=src.replace('../../../docs/journal/', '../../../../../docs/journal/')
        (target/'kernel.rs').write_text(src)
        shutil.copy(depot/ASTRA/'narrow-short/narrow.rs',target/'narrow.rs')
        shutil.copy(depot/ASTRA/'edge-budget/adapter.inc.rs',target/'adapter.inc.rs')


if __name__=='__main__':
    materialize()
