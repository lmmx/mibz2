from pathlib import Path
import sys
d=Path(__file__).resolve().parent
s=(d.parent/"probe.rs").read_text()
a=s.index("    let mut vulnerable = ")
b=s.index("\n}",a)
s=s[:a]+'    audit_update(&text,&row,&groups,&new_rank,index);'+s[b:]
a=s.index('    println!(\n        "stream,') if '    println!(\n        "stream,' in s else s.index('    println!("stream,')
b=s.index(";",a)
s=s[:a]+'    println!("stream,raw_rows,count_edits,atomic_edits,moved_rows,longest_chain,sum_absolute_move_distance")'+s[b:]
Path(sys.argv[1]).write_text((d/"pages.rs").read_text()+(d/"audit.inc.rs").read_text()+s)
