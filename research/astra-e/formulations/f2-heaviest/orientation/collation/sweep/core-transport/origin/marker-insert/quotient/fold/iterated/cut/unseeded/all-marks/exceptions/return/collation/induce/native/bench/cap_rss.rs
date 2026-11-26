fn hwm() -> usize {
    std::fs::read_to_string("/proc/self/status").unwrap().lines()
        .find_map(|line| line.strip_prefix("VmHWM:")).unwrap()
        .split_whitespace().next().unwrap().parse().unwrap()
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let arm: usize = args[1].parse().unwrap();
    let source = std::fs::File::open(&args[2]).unwrap();
    let records = source.metadata().unwrap().len() as usize/2;
    let r = runs::Runs::census_original_caps(source, records);
    let n: usize = args[3].parse().unwrap();
    let p: u32 = args[4].parse().unwrap();
    assert_eq!(r.byte_counts().iter().sum::<u32>(), n as u32);
    let k = Kernels::detect();
    let before = hwm();
    let mut out = Vec::new();
    let mut scalar=candidate::Candidate::default();
    let mut lazy=bwt::LazyCells::default();
    let mut opend=arms::OpenGated::default();
    let mut state=std::ptr::null_mut();
    let got = if arm<3 {
        state=unsafe{grammar_new()};
        let ptr=unsafe{grammar_run(state,r.records().as_ptr(),records as u32,n as u32,p,arm as u32)};
        unsafe{std::slice::from_raw_parts(ptr,n)}
    } else {
        assert!(match arm {
            3 => scalar.invert_runs(&r,n,p,&mut out,k),
            4 => lazy.invert_runs(&r,n,p,&mut out,k),
            5 => opend.invert_runs(&r,n,p,&mut out,k),
            _ => panic!()
        });
        &out
    };
    let post=hwm();
    assert_eq!(got,std::fs::read(&args[5]).unwrap());
    if !state.is_null() {unsafe{grammar_delete(state)};}
    println!("{before},{post}");
}
