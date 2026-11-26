struct Solver{
    child:std::process::Child,
    input:Option<std::process::ChildStdin>,
    output:std::io::BufReader<std::process::ChildStdout>,
    frame:Vec<u8>,
}
fn u32le(buf:&[u8])->u32{u32::from_le_bytes(buf.try_into().unwrap())}
impl Solver{
    fn new(script:&str)->Self{
        use std::process::{Command,Stdio};
        let mut child=Command::new("python3").args(["-B",script])
            .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let input=child.stdin.take();let output=std::io::BufReader::new(child.stdout.take().unwrap());
        Self{child,input,output,frame:Vec::new()}
    }
    fn pick(&mut self,f:&control::Feature)->(Vec<u32>,u64,u32,u32){
        use std::io::Write;
        self.frame.clear();
        let mut add=|v:u32|self.frame.extend_from_slice(&v.to_le_bytes());
        add(f.types.len() as u32);add(f.support.len() as u32);
        for &(b,m,g,w) in &f.types{add(b);add(m);add(g);add(w);}
        for e in &f.support{add(e.len() as u32);for &k in e{add(k);}}
        let tx=self.input.as_mut().unwrap();
        tx.write_all(&(self.frame.len() as u32).to_le_bytes()).unwrap();
        tx.write_all(&self.frame).unwrap();tx.flush().unwrap();
        let mut h=[0;20];self.output.read_exact(&mut h).unwrap();
        let n=u32le(&h[..4]) as usize;
        let cpu=u64::from_le_bytes(h[4..12].try_into().unwrap());
        let (q,w)=(u32le(&h[12..16]),u32le(&h[16..20]));
        self.frame.resize(4*n,0);self.output.read_exact(&mut self.frame).unwrap();
        (self.frame.chunks_exact(4).map(u32le).collect(),cpu,q,w)
    }
    fn close(mut self){drop(self.input.take());assert!(self.child.wait().unwrap().success());}
}
fn fresh(inputs:&[Input],server:&str){
    use std::time::Instant;
    let mut worker=Engine::default();let mut named=Engine::default();
    let expected=inputs.iter().map(|x|Truth::new(x).output(x.p)).collect::<Vec<_>>();
    let mut solver=Solver::new(server);
    println!("rep,stream,position,arm,first_ns,eligible_ns,index_ns,geometry_ns,types_ns,transfer_wait_ns,apply_ns,second_ns,caller_cpu_ns,solver_cpu_ns,elapsed_ns,saved_q,appended");
    for rep in 0..9{
        for step in 0..inputs.len(){
            let i=(3*rep+step)%inputs.len();let x=&inputs[i];
            for pos in 0..2{
                let arm=(pos+rep+i)%2;let start=Instant::now();let t=now();
                if arm==0{
                    let r=named.run(x,0,false);
                    let caller=now()-t;let wall=start.elapsed().as_nanos();assert_eq!(named.out,expected[i]);
                    if rep!=0{println!("{},{},{pos},named,{},0,0,0,0,0,0,{},{caller},0,{wall},0,0",rep-1,x.index,r.first.iter().sum::<u64>(),r.second.iter().sum::<u64>());}
                }else{
                    let (mark,m)=worker.fast.first(&x.caps,&x.freq,x.n,x.p).unwrap();
                    let root=now()-t;let f=worker.fast.census(m);
                    let comm=Instant::now();let (mask,child,q,w)=solver.pick(&f);let wait=comm.elapsed().as_nanos();
                    let apply=now();worker.fast.set_policy(mask);
                    let (p,out_m,s,_)=worker.fast.fold_one(mark,m,true);let ap=now()-apply;
                    let finish=now();let entered=worker.fast.finish(p,out_m,x.n,x.p,&mut worker.out);let second=now()-finish;
                    let caller=now()-t;let wall=start.elapsed().as_nanos();
                    assert!(entered);assert_eq!(s[7],w);assert_eq!(f.sizes[0]-worker.fast.information()[0],q as usize);
                    assert_eq!(worker.out,expected[i]);
                    if rep!=0{println!("{},{},{pos},fresh,{root},{},{},{},{},{wait},{ap},{second},{caller},{child},{wall},{q},{w}",
                        rep-1,x.index,f.cpu[0],f.cpu[1],f.cpu[2],f.cpu[3]);}
                }
            }
        }
    }
    solver.close();
}
