"""Create the sliced-RLE terminal in a copied, validated four-forward constructor."""
from pathlib import Path
here=Path(__file__).resolve().parent
def generate():
    s=(here.parent/'writer.rs').read_text()
    a='pub fn emit<const K:usize,const AUDIT:bool>(t:&Table,out:&mut Vec<u8>)->Work {'
    assert s.count(a)==1
    s=s.replace(a,'''pub fn expand<const K:usize,const SLICED:bool,const AUDIT:bool>(
        t:&Table,out:&mut Vec<u8>,dst:&mut Vec<u8>,k:mibz2::Kernels,audit:&mut crate::slices::Audit
    )->Result<(),mibz2::ErrorKind> {''')
    s=s.replace('return work;', 'if AUDIT {audit.fallback="stock";}\n        return mibz2::rle::unrle(out,dst,k);')
    needle='    let mut pos=0;\n    for l in &lanes {'
    assert s.count(needle)==1
    s=s.replace(needle,'''    if SLICED {
        let mut parts=[crate::slices::Span::default();K];
        let mut selected=0;
        let mut n=0;
        let mut count=0;
        for p in &mut parts {
            let l=&lanes[selected];
            *p=crate::slices::Span{start:l.start,len:l.len};
            n+=l.len;
            selected=l.landing;count+=1;
            if selected==0 {break;}
        }
        if n==t.n && count==K {
            // The positive lengths sum to n, all K physical reservations are
            // disjoint in these n cells, and every length fits its allocation.
            // Therefore every cell in this extent has been initialized.
            let data=unsafe{std::slice::from_raw_parts(storage.as_ptr().cast::<u8>(),t.n)};
            if let Some(result)=crate::slices::parse::<K,AUDIT>(data,&parts,dst,k,audit) {
                if AUDIT {
                    let mut order:[usize;K]=std::array::from_fn(|i|i);
                    for (i,p) in parts.iter().enumerate() {
                        let j=i+order[i..].iter().position(|&v|lanes[v].start==p.start).unwrap();
                        if i!=j {
                            audit.avoided+=order[i..=j].iter().map(|&v|lanes[v].len).sum::<usize>();
                            order[i..=j].rotate_right(1);
                        }
                    }
                }
                return result;
            }
        } else if AUDIT {audit.fallback="period";}
    }
'''+needle)
    needle='    writer::finish(t,out,pos);\n    work'
    assert s.count(needle)==1
    return s.replace(needle,'    writer::finish(t,out,pos);\n    mibz2::rle::unrle(out,dst,k)')
