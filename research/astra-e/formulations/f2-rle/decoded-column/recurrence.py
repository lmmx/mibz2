from pathlib import Path
import sys
d=Path(__file__).resolve().parent
s=(d/"probe.rs").read_text()
a=s.index("    let v = vulnerable.iter()")
b=s.index("\n}",a)
s=s[:a]+"    recurrence(&text,&corrected,&row,&mapped,&vulnerable,&groups,&new_rank,index);"+s[b:]
a=s.index('    println!(\n        "stream,') if '    println!(\n        "stream,' in s else s.index('    println!("stream,')
b=s.index(";",a)
s=s[:a]+'    println!("stream,raw_rows,counts,exceptional,interfaces,comparisons,byte_pairs,anchor_exits,longest_comparison,doubling_rounds")'+s[b:]
prefix=(d/"merge.inc.rs").read_text().split("// Search monotonically",1)[0]
Path(sys.argv[1]).write_text(prefix+(d/"recurrence.inc.rs").read_text()+s)
