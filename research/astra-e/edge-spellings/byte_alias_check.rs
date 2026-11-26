// Independently supply the compactor's arrays. Actual endpoint output in words.bin
// provides the band order, number of visits, and full spelling for every label.
use std::collections::HashSet;

struct Band { code: u32 }
struct Counts { words: usize }
#[derive(Default)]
struct Kernel { offsets: Vec<u32>, spelling: Vec<u8>, edges: Vec<u32>, bands: Vec<Band> }
include!("byte_alias.inc.rs");

fn check(words: &[Vec<u8>], m: usize) -> usize {
    let mut k = Kernel { offsets: vec![0], edges: vec![1234; m], ..Kernel::default() };
    for (i,w) in words.iter().enumerate() {
        k.spelling.extend_from_slice(w);
        k.offsets.push(k.spelling.len() as u32);
        k.bands.push(Band { code: (i % 256) as u32 });
        k.bands.push(Band { code: 256+i as u32 });
    }
    let caps = (k.edges.capacity(), k.spelling.capacity(), k.offsets.capacity());
    let mut counts = Counts { words: words.len() };
    k.alias_spellings(&mut counts);
    assert_eq!(caps, (k.edges.capacity(), k.spelling.capacity(), k.offsets.capacity()));
    assert_eq!(counts.words+1, k.offsets.len());
    assert_eq!(k.spelling.len() as u32,*k.offsets.last().unwrap());
    assert!(k.offsets.windows(2).all(|x|x[0]<x[1]));
    for (i,w) in words.iter().enumerate() {
        assert_eq!(k.bands[2*i].code, (i%256) as u32);
        let owner = (k.bands[2*i+1].code-256) as usize;
        let actual=&k.spelling[k.offsets[owner] as usize..k.offsets[owner+1] as usize];
        assert_eq!(actual,w);
    }
    counts.words
}

fn nat(f: &mut &[u8])->usize {
    let (bytes,rest)=f.split_at(4);
    *f=rest;
    u32::from_le_bytes(bytes.try_into().unwrap()) as usize
}
fn main() {
    assert_eq!(check(&[],1),0);
    assert_eq!(check(&[vec![1;2]],1),1);
    let mut collided = Vec::new();
    for i in 0..48 {
        let mut w=vec![5;24];
        w[6]=i;
        collided.push(w);
    }
    let keys=collided.iter().map(|w| spelling_hash(w)).collect::<HashSet<_>>();
    assert_eq!(keys.len(),1);
    collided.extend_from_within(..);
    assert_eq!(check(&collided,512),64);
    assert_eq!(check(&collided,96),96);
    let memory=std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    let mut data=memory.as_slice();
    let (mut blocks,mut all,mut retained,mut unique)=(0,0,0,0);
    while !data.is_empty() {
        let (_block,_stream,_n,m,_edge,count) = (
            nat(&mut data),nat(&mut data),nat(&mut data),nat(&mut data),nat(&mut data),nat(&mut data)
        );
        let mut words=Vec::with_capacity(count);
        for _ in 0..count {
            let (_handle,_visits,n)=(nat(&mut data),nat(&mut data),nat(&mut data));
            let (word,rest)=data.split_at(n);
            words.push(word.to_vec());
            data=rest;
        }
        let distinct=words.iter().collect::<HashSet<_>>().len();
        let used=check(&words,m);
        assert!(used>=distinct);
        blocks+=1;all+=count;retained+=used;unique+=distinct;
    }
    println!("Empty, singleton, 48 fully colliding distinct words with duplicate replay, 32-probe exhaustion, and original allocation-bound fallback passed.");
    println!("{blocks} real dictionaries, {all} original codes, {retained} compacted codes, {unique} exact byte-distinct codes, {} missed aliases.",retained-unique);
}
