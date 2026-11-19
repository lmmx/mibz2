mod splice;
use splice::*;
use std::path::{Path,PathBuf};

struct Reader {bytes:Vec<u8>,p:usize}
impl Reader {
    fn open(path:&Path,magic:&[u8;8])->Self {
        let bytes=std::fs::read(path).unwrap();assert_eq!(&bytes[..8],magic);
        Self{bytes,p:8}
    }
    fn u(&mut self)->u32 {
        let old=self.p;self.p+=4;u32::from_le_bytes(self.bytes[old..self.p].try_into().unwrap())
    }
    fn slice(&mut self,n:u32)->&[u8] {let old=self.p;self.p+=n as usize;&self.bytes[old..self.p]}
}
struct Case {name:String,j:Obj,arena:Vec<u8>,jmark:u32,jskip:u32,holes:Vec<u32>,n:u32,
             oracle_path:PathBuf,expected:Vec<u8>,original_dictionary:usize}
impl Case {
    fn new(path:&Path)->Self{
        let mut r=Reader::open(path,b"JHSPv1\0\0");
        let(_ord,_policy,_h,n,m,jmark,jskip,k,runs,e)=(r.u(),r.u(),r.u(),r.u(),r.u(),r.u(),r.u(),r.u(),r.u(),r.u());
        let mut j=Obj{m,..Obj::default()};let mut arena=Vec::new();
        for _ in 0..k {let w=r.u();let off=arena.len() as u32;arena.extend_from_slice(r.slice(w));j.words.push(Piece{id:off,width:w});}
        let mut counts=vec![0;k as usize];
        for _ in 0..runs {let (id,width)=(r.u(),r.u());j.runs.push(Piece{id,width});counts[id as usize]+=width;}
        let mut cursor=0;
        for count in counts {assert!(count>0);j.bases.push(cursor);cursor+=count;}
        assert_eq!(cursor,m);
        let holes=(0..e).map(|_|r.u()).collect();
        assert_eq!(r.p,r.bytes.len());
        arena.shrink_to_fit();
        let original_dictionary=arena.len();let name=path.file_stem().unwrap().to_str().unwrap().to_string();
        Self{name,j,arena,jmark,jskip,holes,n,oracle_path:path.with_extension("oracle"),expected:vec![],original_dictionary}
    }
}
#[repr(C)] struct ObjectView {root:*const Piece,r:usize,words:*const Piece,bases:*const u32,k:usize,
                             dict:*const u8,bytes:usize,mark:u32,skip:u32,tokens:u32,n:u32}
#[repr(C)] struct Owned {data:*mut u8,len:usize,cap:usize}
unsafe extern "C" {fn consume(obj:&ObjectView,second:*mut u32)->Owned;fn release(o:Owned);}
impl Owned {
    fn as_slice(&self)->&[u8]{unsafe{std::slice::from_raw_parts(self.data,self.len)}}
    fn finish(self){unsafe{release(self);}}
}
fn native(obj:&Obj,arena:&[u8],mark:u32,skip:u32,n:u32)->(Owned,bool){
    assert!(mark<obj.m);
    let view=ObjectView{root:obj.runs.as_ptr(),r:obj.runs.len(),words:obj.words.as_ptr(),bases:obj.bases.as_ptr(),k:obj.words.len(),
                        dict:arena.as_ptr(),bytes:arena.len(),mark,skip,tokens:obj.m,n};
    let mut used=0;let out=unsafe{consume(&view,&mut used)};(out,used!=0)
}
fn choose<'a>(j:&'a Obj,p:&'a Patch,mark:u32,skip:u32)->(&'a Obj,u32,u32,bool) {
    match p.mark(mark,skip) {
        Mark::New(at,off)=>(p.obj.as_ref().unwrap_or(j),at,off,false),
        Mark::Old(at,off)=>(j,at,off,true)
    }
}
fn validate(c:&mut Case,report:bool) {
    let mut h=Reader::open(&c.oracle_path,b"JHORv1\0\0");
    let (hm,mark,skip,n)=(h.u(),h.u(),h.u(),h.u());
    assert_eq!(n,c.n);
    let before=c.arena.as_ptr();let cap0=c.arena.capacity();
    let p=splice(&c.j,&mut c.arena,&c.holes);
    let shifted=c.arena.as_ptr()!=before;
    let obj=p.obj.as_ref().unwrap_or(&c.j);assert_eq!(obj.m,hm);
    let mut next=vec![(0,0);hm as usize];
    let mut write=obj.bases.clone();let mut y=0;
    for run in &obj.runs {
        let a=run.id as usize;
        for _ in 0..run.width {next[write[a] as usize]=(y,run.id);write[a]+=1;y+=1;}
    }
    assert_eq!(y,hm);
    for (actual,id) in next {
        let (want,w)=(h.u(),h.u());assert_eq!(actual,want);
        let range=obj.words[id as usize];
        assert_eq!(w,range.width);assert_eq!(&c.arena[range.id as usize..(range.id+w)as usize],h.slice(w));
    }
    let expected=h.slice(n);
    if c.jmark==u32::MAX {assert_eq!(mark,u32::MAX);c.expected=expected.to_vec();c.arena.truncate(c.original_dictionary);return;}
    let (run,at,off,fallback)=choose(&c.j,&p,c.jmark,c.jskip);
    if fallback{assert_eq!(mark,u32::MAX);}else{assert_eq!((at,off),(mark,skip));}
    let (old,old_second)=native(&c.j,&c.arena,c.jmark,c.jskip,n);
    assert_eq!(old.as_slice(),expected);old.finish();
    let (new,new_second)=native(run,&c.arena,at,off,n);
    assert_eq!(new.as_slice(),expected);new.finish();c.expected=expected.to_vec();
    if report{
        println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            c.name,c.j.m,hm,c.holes.len(),p.changes,c.j.words.len(),obj.words.len(),c.j.runs.len(),obj.runs.len(),
            p.visits,p.segments,p.compound_bytes,c.original_dictionary,cap0,c.arena.capacity(),usize::from(shifted),
            c.jmark,at,off,usize::from(fallback),usize::from(old_second),usize::from(new_second));
    }
    assert_eq!(h.p,h.bytes.len());
    c.arena.truncate(c.original_dictionary);
}
#[repr(C)] struct Ts {s:i64,n:i64}
unsafe extern "C" {fn clock_gettime(id:i32,t:*mut Ts)->i32;}
fn clock()->u64 {let mut t=Ts{s:0,n:0};assert_eq!(unsafe{clock_gettime(3,&mut t)},0);t.s as u64*1_000_000_000+t.n as u64}
fn laps(cases:&mut[Case]){
    println!("round,case,position,arm,splice_ns,consume_ns,complete_ns,second");
    for round in 0..=24usize {
        for i in 0..cases.len(){
            let idx=(if round%2==0{i}else{cases.len()-1-i}+round*3)%cases.len();
            let c=&mut cases[idx];
            let lap=round.saturating_sub(1);
            for pos in 0..4{
                let arm=(lap+idx+if (lap/4+idx)%2==0{pos}else{4-pos})%4;
                c.arena.truncate(c.original_dictionary);
                let begin=clock();
                let p=(arm>=2).then(||splice(&c.j,&mut c.arena,&c.holes));
                let (run,at,off,_)=if arm==2{choose(&c.j,p.as_ref().unwrap(),c.jmark,c.jskip)}else{(&c.j,c.jmark,c.jskip,false)};
                let mid=clock();
                let (out,used)=native(run,&c.arena,at,off,c.n);
                drop(p);
                let end=clock();
                assert_eq!(out.as_slice(),c.expected);out.finish();
                if round>0 {println!("{round},{},{pos},{arm},{},{},{},{}",c.name,mid-begin,end-mid,end-begin,usize::from(used));}
            }
        }
    }
}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    let dir=Path::new(&args[2]);
    let mut paths:Vec<_>=std::fs::read_dir(dir).unwrap().flatten().map(|x|x.path()).filter(|p|
        p.extension().is_some_and(|v|v=="jh") && (args.get(3).is_some_and(|v|v=="all") ||
          p.file_name().unwrap().to_string_lossy().contains("-frequent-h8.") || p.file_name().unwrap().to_string_lossy().contains("-quote-h12."))).collect();
    paths.sort();
    let mut cases=Vec::new();
    let timed=args[1]=="price";
    if !timed{println!("case,m_j,m_h,holes,q,k_j,k_h,r_j,r_h,run_fragments,source_segments,compound_bytes,original_dictionary,old_capacity,new_capacity,arena_moved,jmark,new_mark,new_skip,old_j_fallback,j_used_second,h_used_second");}
    for path in paths {let mut c=Case::new(&path);validate(&mut c,!timed);if timed && c.jmark!=u32::MAX {cases.push(c);}}
    if timed{assert_eq!(cases.len(),20);laps(&mut cases);}
}
