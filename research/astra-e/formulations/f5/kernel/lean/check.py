"""Exercise the one-byte hint against the independently grouped-run checker."""
from pathlib import Path
import sys

here = Path(__file__).resolve().parent
base = here.parent
source = (base / "check_actions.rs").read_text()
state = (base / "actions.rs").read_text().split("// After the first input,", 1)[0]
source = source.replace('include!("actions.rs");', state+'\n'+(here / "book.rs").read_text())
source = source.replace("for budget in [0, word.len() / 2, 32_768]", "for budget in [0]")
start=source.index("            assert!(book.expanded.len() <= budget);")
end=source.index("            for (last, count)",start)
source=source[:start]+'            stats[1 + usize::from(book.hints[1] == 0)] += 1;\n'+source[end:]
source=source.replace("stats.iter().all", "stats[..3].iter().all")
a=source.index('        println!(')
b=source.index("    }",a)
source=source[:a]+'''        println!("{} exact representative, incoming-state and prefix checks, {} transparent hints and {} opaque hints",stats[0],stats[1],stats[2]);
'''+source[b:]
Path(sys.argv[1]).write_text(source)
