#[link(name="libsais",kind="static")]
unsafe extern "C"{fn libsais_int(t:*mut i32,sa:*mut i32,n:i32,k:i32,fs:i32)->i32;}
#[derive(Default)]
struct SparseSort {
    pairs: Vec<(u32,u32)>, tmp:Vec<(u32,u32)>, alphabet:Vec<i32>, sa:Vec<i32>,
}
impl SparseSort {
    fn sorted(&mut self,seq:&[usize],rank:&[usize],gap:&[usize],new:&[u8])->&[i32] {
        self.pairs.clear();let n=seq.len();
        for (i,&j) in seq.iter().enumerate(){self.pairs.push((((2*gap[j]+usize::from(rank[j]!=usize::MAX))as u32)*256+u32::from(new[j]),i as u32));}
        self.tmp.resize(n,(0,0));
        for shift in [0,10,20] {
            let mut offset=[0usize;1024];
            for &(key,_) in &self.pairs{offset[((key>>shift)&1023)as usize]+=1;}
            let mut tally=0;for at in &mut offset{let s=*at;*at=tally;tally+=s;}
            for &pair in &self.pairs{let at=&mut offset[((pair.0>>shift)&1023)as usize];self.tmp[*at]=pair;*at+=1;}
            std::mem::swap(&mut self.pairs,&mut self.tmp);
        }
        let mut num=0;let mut prev=None;
        self.alphabet.resize(n,0);self.sa.resize(n,0);
        for &(key,position) in &self.pairs {
            if Some(key)!=prev {prev=Some(key);num+=1;}
            self.alphabet[position as usize]=num-1;
        }
        // The supplied seq is a concatenation of exceptional chains followed by their distinct certified terminals.
        if n>0 {
            let err=unsafe {libsais_int(self.alphabet.as_mut_ptr(),self.sa.as_mut_ptr(),n as i32,num,0)};
            assert_eq!(err,0);
        }
        &self.sa
    }
}
#[repr(C)]
struct TimeSpec { sec:i64, ns:i64 }
unsafe extern "C" {fn clock_gettime(id:i32, t:*mut TimeSpec)->i32;}
fn cpu()->u64 {let mut t=TimeSpec{sec:0,ns:0};assert_eq!(unsafe {clock_gettime(3,&mut t)},0);t.sec as u64*1_000_000_000+t.ns as u64}
