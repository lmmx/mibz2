"""Compose the verified width guard, immediate words and optional one-edge lookahead."""
from hashlib import sha256
from pathlib import Path
import runpy

here = Path(__file__).resolve().parent
standalone = here.parent / 'narrow-endpoint'
runpy.run_path(str(standalone / 'prepare.py'))
short = (here.parent / 'short-words/endpoint.rs').read_text()
if sha256(short.encode()).hexdigest() != '09380d30e328d27056735776812e55677a69c21933dc8b56412da75b87ee2d4d':
    raise ValueError('short-word endpoint source changed')


def span(text, start, end):
    a = text.index(start)
    b = text.index(end, a)
    return text[a:b]


# Node IDs and word handles occupy disjoint fields. Transfer only word operations.
narrow = (standalone / 'narrow.rs').read_text()
for start, end in [('    fn len(&self', '    fn gaps('), ('    fn append(', '\n}\n\nimpl Kernel')]:
    narrow = narrow.replace(span(narrow, start, end), span(short, start, end))
(here / 'narrow.rs').write_text(narrow)
# Preserve tagged spellings in the overflow path as well as the narrow layout.
constructor = (standalone / 'constructor.rs').read_text().replace(
    '../../../docs/journal/2026-11-18-open-return/endpoint.rs', '../short-words/endpoint.rs')
(here / 'constructor.rs').write_text(constructor)

# The lookahead arm fetches the valid successor before spelling the current edge.
# The writer receives the same bits, spelling dictionary and mark.
ahead = (here.parent / 'lookahead/writer.rs').read_text()
emit = ahead[ahead.index('fn emit('):]
old_emit = span(constructor, 'fn emit(', '#[inline]\nfn append_code(')
(here / 'constructor_ahead.rs').write_text(constructor.replace(old_emit, emit + '\n'))

# Both public entries implement the same full gated worker contract.
entry = (standalone / 'candidate.rs').read_text()
(here / 'adapter.inc.rs').write_text(entry[entry.index('use crate::'):])
