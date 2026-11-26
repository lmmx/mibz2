from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
p=Path(sys.argv[1])
subprocess.run([sys.executable,str(d/"recurrence.py"),str(p)],check=True)
s=p.read_text()
a=s.index("    let v=exceptional.len();")
b=s.index("let mut output=0;",a)
s=s[:a]+'''    let v=exceptional.len();
    let mut seq=Vec::new();
    for &a in &anchors {
        let mut j=(a+1)%m;if rank[j]!=usize::MAX{continue;}
        while rank[j]==usize::MAX {seq.push(j);j=(j+1)%m;}
        seq.push(j);
    }
    assert_eq!(seq.len(),v+endpoints.len());
    let mut sorter=SparseSort::default();let mut out=Vec::new();
    for round in 0..=32 {
       for pos in 0..2 {
           let sorting=(round+pos)%2==0;
           let now=cpu();
           if sorting { std::hint::black_box(sorter.sorted(&seq,&rank,&gap,new)); }
           else { out.clear();mibz2::rle::unrle(raw,&mut out,Kernels::detect()).unwrap();std::hint::black_box(&out); }
           let nanos=cpu()-now;
           if round>0 {println!("{index},{round},{pos},{},{nanos},{v},{}",if sorting {"sais"}else{"rle"},endpoints.len());}
       }
    }
    let sorted=sorter.sorted(&seq,&rank,&gap,new);
    let mut ordered=sorted.iter().map(|&i|seq[i as usize]).filter(|&j|rank[j]==usize::MAX).peekable();
    '''+s[b:]
a=s.index('    println!("{index},{n},{},{v},')
b=s.index("\n}",a)
s=s[:a]+s[b:]
s=s.replace("stream,raw_rows,counts,exceptional,interfaces,comparisons,byte_pairs,anchor_exits,longest_comparison,doubling_rounds","stream,round,position,arm,cpu_ns,exceptional,interfaces")
p.write_text((d/"linear.inc.rs").read_text()+s)
