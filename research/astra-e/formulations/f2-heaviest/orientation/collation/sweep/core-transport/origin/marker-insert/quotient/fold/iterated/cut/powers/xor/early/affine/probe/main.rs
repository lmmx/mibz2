extern "C" {
    fn index_load(path:*const i8)->*mut c_void;
    fn index_drop(state:*mut c_void);
    fn index_emit(state:*mut c_void,n:u32)->*const u8;
}
fn main(){
    let args:Vec<_>=std::env::args().collect();let dir=Path::new(&args[2]);
    let warm=args.get(3).is_some_and(|v|v=="warm");
    let data=std::fs::read(&args[1]).unwrap();let mut sc=Scanner::new(SliceSource(&data));
    let mut scalar=candidate::Candidate::default();let mut lazy=bwt::LazyCells::default();
    let k=Kernels::detect();let mut blocks=Vec::new();let mut indices=Vec::new();let mut out=Vec::new();
    while let Event::Block{index,symbols,..}=sc.next(Vec::new()).unwrap(){
        let id=index as usize;if !IDS.contains(&id){continue}
        assert!(runs::takes_runs(&symbols));let mut r=runs::Runs::default();let n=runs::front(&symbols,&mut r,k).unwrap();
        let p=symbols.orig_ptr;let mut want=Vec::new();assert!(lazy.invert_runs(&r,n,p,&mut want,k));
        let path=std::ffi::CString::new(dir.join(format!("{id}.idx")).to_str().unwrap()).unwrap();
        indices.push(unsafe{index_load(path.as_ptr())});blocks.push(Block{id,n,p,r,want});
    }
    assert_eq!(blocks.len(),10);let mut f=std::io::BufWriter::new(std::fs::File::create(dir.join(if warm{"warm.csv"}else{"times.csv"})).unwrap());
    writeln!(f,"round,block,slot,arm,thread_ns").unwrap();
    for r in 0..=24 {for j in 0..10{let i=(r+j)%10;let b=&blocks[i];for slot in 0..2{
        let arm=(r+i+slot)%2;
        if warm {
            if arm==0 {
                let ptr=unsafe{index_emit(indices[i],b.n as u32)};
                assert!(unsafe{std::slice::from_raw_parts(ptr,b.n)}==b.want);
            }else{assert!(scalar.invert_runs(&b.r,b.n,b.p,&mut out,k));assert!(out==b.want);}
        }
        let t=cpu();
        let got=if arm==0{
            let ptr=unsafe{index_emit(indices[i],b.n as u32)};
            unsafe{std::slice::from_raw_parts(ptr,b.n)}
        }else{
            assert!(scalar.invert_runs(&b.r,b.n,b.p,&mut out,k));out.as_slice()
        };
        let dt=cpu()-t;assert!(got==b.want);if r>0{writeln!(f,"{r},{},{slot},{},{dt}",b.id,if arm==0{"implicit"}else{"scalar"}).unwrap();}
    }}}
    for v in indices{unsafe{index_drop(v)}}
}
