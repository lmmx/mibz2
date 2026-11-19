//! Expose the exact run-encoded token last column before F2b allocates token links.
use super::*;
use mibz2::stream::{Scanner,SliceSource,Event};
use std::{env,fs::File,io::Write,path::Path};
#[test]
#[ignore]
fn export() {
    let bytes=std::fs::read(env::var("F2_CORPUS").unwrap()).unwrap();
    let target=env::var("F2_EXPORT").unwrap();
    let mut scanner=Scanner::new(SliceSource(&bytes));
    let mut r=Runs::default();
    let mut first=Candidate::default();
    let mut reference=Segmented::default();
    let mut want=Vec::new();let mut ordinal=0;
    while let Ok(Event::Block{index,symbols,..})=scanner.next(Vec::new()) {
        let k=Kernels::detect();let n=crate::runs::front(&symbols,&mut r,k).unwrap();
        if n<65536{continue}
        if ordinal%24==0 {
            let (quote,m)=first.prepare(r.records(),n as u32).unwrap();
            let (p,skip)=first.transfer_mark(symbols.orig_ptr,quote).unwrap();
            first.build(quote,m);first.inverse_merge();
            let col=&first.merged[first.nodes[0].out as usize..];
            let mut bases=vec![0;first.words.len()];
            for node in &first.nodes {if node.word!=NONE {bases[node.word as usize]=node.base;}}
            let file=File::create(Path::new(&target).join(format!("{index}.token"))).unwrap();
            let mut out=std::io::BufWriter::new(file);
            for v in [n as u32,p,skip,m,first.words.len() as u32,col.len() as u32] {out.write_all(&v.to_le_bytes()).unwrap();}
            for (i,word) in first.words.iter().enumerate() {
                out.write_all(&bases[i].to_le_bytes()).unwrap();
                out.write_all(&word.width.to_le_bytes()).unwrap();
                out.write_all(&first.dict[word.id as usize..(word.id+word.width) as usize]).unwrap();
            }
            for piece in col {
                out.write_all(&first.nodes[piece.id as usize].word.to_le_bytes()).unwrap();
                out.write_all(&piece.width.to_le_bytes()).unwrap();
            }
            assert!(reference.invert_runs(&r,n,symbols.orig_ptr,&mut want,k));
            std::fs::write(Path::new(&target).join(format!("{index}.expected")), &want).unwrap();
            println!("{ordinal},{index},{n},{p},{skip},{m},{},{}",first.words.len(),col.len());
        }
        ordinal+=1;
    }
    assert_eq!(ordinal,233);
}
