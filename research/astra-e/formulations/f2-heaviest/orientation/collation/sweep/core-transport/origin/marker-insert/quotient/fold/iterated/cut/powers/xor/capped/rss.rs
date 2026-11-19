// Prepend the frozen joint driver's imports and FFI declarations (rss.py).
fn peak_kib()->usize {
    let status=std::fs::read_to_string("/proc/self/status").unwrap();
    let line=status.lines().find(|s|s.starts_with("VmHWM:")).unwrap();
    line.split_whitespace().nth(1).unwrap().parse().unwrap()
}
fn main() {
    let args:Vec<_>=std::env::args().collect();
    let arm=ARMS.iter().position(|&s|s==args[2]).unwrap();let k=Kernels::detect();
    let mut r=runs::Runs::default();
    let data=std::fs::read(&args[1]).unwrap();
    let mut scanner=Scanner::new(SliceSource(&data));
    let Event::Block{symbols,..}=scanner.next(Vec::new()).unwrap() else {panic!()};
    assert!(runs::takes_runs(&symbols));let p=symbols.orig_ptr;
    let n=runs::front(&symbols,&mut r,k).unwrap();drop(symbols);drop(scanner);drop(data);
    let before=peak_kib();let mut out=Vec::new();let mut state=std::ptr::null_mut();
    let mut scalar=candidate::Candidate::default();let mut lazy=bwt::LazyCells::default();
    let mut opend=arms::OpenGated::default();
    let got=if arm<3 {
        state=unsafe{grammar_new()};let caps=r.records();
        let ptr=unsafe{grammar_run(state,caps.as_ptr(),caps.len() as u32,n as u32,p,arm as u32)};
        unsafe{std::slice::from_raw_parts(ptr,n)}
    } else {
        assert!(match arm {3=>scalar.invert_runs(&r,n,p,&mut out,k),4=>lazy.invert_runs(&r,n,p,&mut out,k),_=>opend.invert_runs(&r,n,p,&mut out,k)});
        out.as_slice()
    };
    let peak=peak_kib();
    assert!(got==std::fs::read(&args[3]).unwrap().as_slice());
    if !state.is_null(){unsafe{grammar_delete(state)};}
    println!("{p},{n},{},{before},{peak}",r.records().len());
}
