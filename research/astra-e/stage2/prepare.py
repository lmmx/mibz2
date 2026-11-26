"""Derive the second fixed-sample stage executable from the first frozen example."""
import subprocess
from pathlib import Path

base = "0f25dee737432feafb03b8b14dc10afbf5c141ad"
source = subprocess.check_output(["git", "show", base + ":examples/astra_e.rs"], text=True)
start = source.index("#[allow(dead_code)]")
end = source.index("const ORDINALS", start)
modules = {
    "open_return": "../../../docs/journal/2026-11-18-open-return/open_return.rs",
    "lookahead": "../lookahead/candidate.rs",
    "dictionary": "../dictionary/candidate.rs",
    "compact_f": "../open/compact/candidate.rs",
    "short_words": "../short-words/candidate.rs",
}
source = source[:start] + "".join(f'#[allow(dead_code)]\n#[path = "{path}"]\nmod {name};\n'
                                   for name, path in modules.items()) + source[end:]
start = source.index("    let mut arms = [")
end = source.index("    let count =", start)
arms = {
    "lazy": "bwt::LazyCells",
    "eager": "bwt::Cells",
    "opend16": "open_return::Open<16, true, true>",
    **{name: name + "::Candidate" for name in modules if name != "open_return"},
}
source = source[:start] + "    let mut arms = [\n" + "".join(f'        arm::<{ty}>("{name}", kernels),\n'
    for name, ty in arms.items()) + "    ];\n" + source[end:]
helpers = Path(__file__).with_name("clock.inc.rs").read_text()
source = source.replace("fn main() {", helpers + "\nfn main() {")
source = source.replace('    let count = arms.len();', '''    if env::args().nth(3).as_deref() == Some("probe") {
        let mut probe = arm::<short_words::ShortWords<true>>("probe", kernels);
        for block in &blocks {
            assert!((probe.run)(block, &mut probe.out) && probe.out == block.reference);
        }
        return;
    }
    let count = arms.len();''')
source = source.replace("let begin = (pass > 0).then(Instant::now);", '''let cpu_begin = (pass > 0).then(cpu_ns);
                let begin = (pass > 0).then(Instant::now);''')
source = source.replace('let elapsed = begin.map(|start| start.elapsed().as_nanos());', '''let elapsed = begin.map(|start| start.elapsed().as_nanos());
                let cpu = cpu_begin.map(|start| cpu_ns() - start);''')
source = source.replace("records,arm,elapsed_ns", "records,arm,elapsed_ns,thread_cpu_ns")
source = source.replace('"{},{},{},{},{},{},{},{}"', '"{},{},{},{},{},{},{},{},{}"')
source = source.replace('                        elapsed\n', '                        elapsed, cpu.unwrap()\n')
assert 'cpu.unwrap()' in source
Path(__file__).with_name("experiment.rs").write_text(source)
