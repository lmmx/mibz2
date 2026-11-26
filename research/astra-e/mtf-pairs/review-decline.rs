// An input which passes the outer run gate and rejects the exact edge budget after a large scan.
pub use mibz2::{bwt, runs, Kernels};
use mibz2::{block::Symbols, bwt::InverseBwt};
#[path="stock.front.rs"] mod stock;
#[path="wide.front.rs"] mod wide;
#[path="candidate.rs"] mod integrated;
include!("gate.rs");

struct Runs {
    runs: Vec<u16>,
    counts: [u32; 256],
    per: [u32; 256],
    lit: Vec<u64>,
    one: Vec<u64>,
}
impl Runs {
    fn new() -> Self {
        Self { runs: Vec::new(), counts: [0;256], per: [0;256], lit: Vec::new(), one: Vec::new() }
    }
    fn vector_bytes(&self) -> usize {
        self.runs.capacity()*2 + (self.lit.capacity()+self.one.capacity())*8
    }
}

fn main() {
    const PAIRS: usize = 140000;
    let k=Kernels::detect();
    assert!(k.avx2() && is_x86_feature_detected!("bmi2") && is_x86_feature_detected!("popcnt") && is_x86_feature_detected!("lzcnt"));
    let mut syms=[2u16,0].repeat(PAIRS);
    syms.push(3);
    let mut alphabet=[0u8;256];
    alphabet[1]=1;
    let mut s=Symbols { seq_to_unseq:alphabet,syms,crc:0,orig_ptr:0,eob:3,max_len:900000 };
    assert!(runs::takes_runs(&s));
    let (mut private,mut retry)=(Runs::new(),Runs::new());
    let n=unsafe { wide::front_avx2(&s,&mut private) }.unwrap();
    assert_eq!(n,2*PAIRS);
    assert_eq!(unsafe { stock::front_avx2(&s,&mut retry) },Some(n));
    assert_eq!(private.runs,retry.runs);
    assert_eq!(private.counts,retry.counts);
    assert_eq!(private.per,retry.per);
    assert_eq!(private.runs.len(),PAIRS);
    assert!(private.runs.iter().enumerate().all(|(i,r)|*r==([0x102,2][i%2])));
    assert!(private.counts.iter().enumerate().all(|(c,&count)|count==if c<2{PAIRS as u32}else{0}));
    assert!(private.per.iter().enumerate().all(|(c,&count)|count==if c<2{(PAIRS/2) as u32}else{0}));
    for tape in [&private.runs,&retry.runs] {
        assert!(tape.len()*20>=n*3 && n>=65536);
        assert!(!fits(tape,s.orig_ptr));
    }
    let mut candidate=integrated::Candidate::default();
    let mut untouched=vec![0xcau8,0xfe];
    assert!(!candidate.invert_symbol_runs(&s,k,&mut untouched));
    assert_eq!(untouched,[0xca,0xfe]);

    let column=[1u8,1,0,0].repeat(PAIRS/2);
    let mut cursors=[0,PAIRS];
    let mut next=vec![0usize;n];
    for (i,&b) in column.iter().enumerate() {
        let dest=&mut cursors[b as usize];
        next[*dest]=i;
        *dest+=1;
    }
    let mut row=s.orig_ptr as usize;
    let raw:Vec<u8>=(0..n).map(|_|{row=next[row];column[row]}).collect();
    let mut expected=Vec::new();
    mibz2::rle::unrle_scalar(&raw,&mut expected,0).unwrap();
    s.crc=mibz2::crc::crc32(&expected,k);
    let mut got=vec![0xa5];
    let mut scratch=mibz2::Scratch::<integrated::Candidate>::default();
    assert_eq!(mibz2::decode_block(&s,k,&mut scratch,&mut got),Ok(s.crc));
    assert_eq!(got[0],0xa5);
    assert_eq!(got[1..],expected);
    std::fs::write(std::env::args().nth(1).expect("raw output path"),&expected).unwrap();
    println!("symbols={} n={} records={} retain_bytes={} scans=2",s.syms.len(),n,private.runs.len(),4*n);
    println!("fresh private_vec_bytes={} retry_vec_bytes={} private_Runs_headers_and_counters={} word_capacity={} records_capacity={}",
        private.vector_bytes(),retry.vector_bytes(),std::mem::size_of::<Runs>(),private.lit.capacity(),private.runs.capacity());
    println!("final_bytes={} crc={:08x} complete_integrated_pass=true",expected.len(),s.crc);
}
