#[repr(C)]#[derive(Clone,Copy,Default)]
struct Geo{rows:u32,runs:u32,types:u32,mark_class:u32,mark_row:u32,max_weight:u32,grammar_count:u32,dict_bytes:u32,weighted_bytes:u32}
extern "C" {
    fn prefix_chain(c:*const u16,k:u32,n:u32,p:u32,t:*mut u64,g:*mut Geo,reference:*const u8)->u32;
    fn prefix_finish(s:*mut c_void,c:*const u16,k:u32,n:u32,p:u32,d:u32)->*const u8;
}
fn main() {
    let args:Vec<_>=std::env::args().collect();
    let dir=Path::new(&args[2]);let mode=&args[3];let k=Kernels::detect();
    let mut scalar=candidate::Candidate::default();let mut lazy=bwt::LazyCells::default();
    let mut out=Vec::new();let data=std::fs::read(&args[1]).unwrap();
    let mut sc=Scanner::new(SliceSource(&data));let mut blocks=Vec::new();
    while let Event::Block{index,symbols,..}=sc.next(Vec::new()).unwrap() {
        let id=index as usize;if !IDS.contains(&id){continue}
        assert!(runs::takes_runs(&symbols));let mut r=runs::Runs::default();
        let n=runs::front(&symbols,&mut r,k).unwrap();let mut want=Vec::new();let p=symbols.orig_ptr;
        assert!(lazy.invert_runs(&r,n,p,&mut want,k));blocks.push(Block{id,n,p,r,want});
    }
    assert_eq!(blocks.len(),10);
    let mut f=std::io::BufWriter::new(std::fs::File::create(dir.join(format!("{mode}.csv"))).unwrap());
    let mut t=[0u64;64];let mut g=[Geo::default();64];
    if mode=="geometry" {
        writeln!(f,"block,depth,rows,runs,types,mark_class,mark_row,max_weight,grammar_nodes,dictionary_bytes,weighted_bytes").unwrap();
        for b in &blocks {
            let c=b.r.records();let got=unsafe{prefix_chain(c.as_ptr(),c.len() as u32,b.n as u32,b.p,t.as_mut_ptr(),g.as_mut_ptr(),b.want.as_ptr())};
            for(d,v)in g.iter().enumerate().take(got as usize){
                writeln!(f,"{},{d},{},{},{},{},{},{},{},{},{}",b.id,v.rows,v.runs,v.types,v.mark_class,v.mark_row,v.max_weight,v.grammar_count,v.dict_bytes,v.weighted_bytes).unwrap();
            }
        }
        return;
    }
    let finish:Vec<u32>=args[4..].iter().map(|v|v.parse().unwrap()).collect();
    let states:Vec<_>=finish.iter().map(|_|unsafe{grammar_new()}).collect();
    let arms=if mode=="prefix" {2} else {assert_eq!(mode,"residual");1+states.len()};
    writeln!(f,"round,block,slot,arm,depth,thread_ns").unwrap();
    for r in 0..=24 {
        for i in 0..blocks.len() {
            let b=&blocks[(r+i)%blocks.len()];let c=b.r.records();
            for slot in 0..arms {
                let arm=(r+b.id+if (r/arms)%2==0{slot}else{arms-slot-1})%arms;
                if arm==0{
                    let start=cpu();assert!(scalar.invert_runs(&b.r,b.n,b.p,&mut out,k));let dt=cpu()-start;
                    assert!(out==b.want);if r>0{writeln!(f,"{r},{},{slot},scalar,0,{dt}",b.id).unwrap();}
                }else if mode=="prefix"{
                    let d=unsafe{prefix_chain(c.as_ptr(),c.len() as u32,b.n as u32,b.p,t.as_mut_ptr(),g.as_mut_ptr(),std::ptr::null())};
                    if r>0{for(depth,elapsed)in t.iter().enumerate().take(d as usize){writeln!(f,"{r},{},{slot},prefix,{depth},{elapsed}",b.id).unwrap();}}
                }else{
                    let d=finish[arm-1];let start=cpu();let ptr=unsafe{prefix_finish(states[arm-1],c.as_ptr(),c.len() as u32,b.n as u32,b.p,d)};let dt=cpu()-start;
                    assert!(unsafe{std::slice::from_raw_parts(ptr,b.n)}==b.want);
                    if r>0{writeln!(f,"{r},{},{slot},residual,{d},{dt}",b.id).unwrap();}
                }
            }
        }
    }
    for s in states{unsafe{grammar_delete(s)};}
}
