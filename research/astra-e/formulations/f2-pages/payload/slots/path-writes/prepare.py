"""Keep Vec's natural growth and compare per-wave slack proofs with a write cursor."""
from hashlib import sha256
from pathlib import Path
here=Path(__file__).resolve().parent
s=(here.parent/'candidate.rs').read_text()
assert sha256(s.encode()).hexdigest()=='84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06'
s=s.replace('../../../f2c/second.rs','../../../../f2c/second.rs').replace('pub type Candidate = Dispatch<2>;', 'pub type Candidate = Dispatch<2,2,3>;')
s=s.replace('const SCHEDULE:u8=1>', 'const SCHEDULE:u8=2,const PATH:u8=3>').replace('impl<const MODE:u8,const SCHEDULE:u8>', 'impl<const MODE:u8,const SCHEDULE:u8,const PATH:u8>').replace('Dispatch<MODE,SCHEDULE>','Dispatch<MODE,SCHEDULE,PATH>')
s=s.replace('        let mut job = [0usize; LANES];','''        let mut cursor:[*mut u8;LANES]=[std::ptr::null_mut();LANES];
        let mut limit:[*mut u8;LANES]=[std::ptr::null_mut();LANES];
        let mut budget=0usize;
        let mut wave=0u32;
        let mut spare=false;
        macro_rules! append {
            ($lane:expr,$byte:expr) => {{
                let lane=$lane;
                let byte=$byte;
                if PATH==3 {
                    let path=&mut paths[lane];
                    if cursor[lane]==limit[lane] {
                        // SAFETY: the cursor advances by one initialized byte
                        // and equals allocation end. Every byte is initialized.
                        unsafe { path.set_len(path.capacity()); }
                        path.push(byte);
                        let base=path.as_mut_ptr();
                        // SAFETY: push initialized the new length within capacity.
                        cursor[lane]=unsafe {base.add(path.len())};
                        limit[lane]=unsafe {base.add(path.capacity())};
                    } else {
                        #[cfg(path_guard_check)]
                        assert!((cursor[lane] as usize) >= path.as_ptr() as usize
                            && (cursor[lane] as usize) < limit[lane] as usize
                            && limit[lane] as usize == path.as_ptr() as usize+path.capacity());
                        // SAFETY: the live cursor is inside its current allocation;
                        // a new pointer is obtained on any reallocating push.
                        unsafe {cursor[lane].write(byte); cursor[lane]=cursor[lane].add(1);}
                    }
                } else if spare {
                    let path=&mut paths[lane];
                    let len=path.len();
                    #[cfg(path_guard_check)]
                    assert!(len<path.capacity());
                    // SAFETY: the wave bound reserves one remaining byte for
                    // each advancing job, including after a reset to one seed.
                    unsafe {path.as_mut_ptr().add(len).write(byte); path.set_len(len+1);}
                } else {
                    paths[lane].push(byte);
                }
            }};
        }
        let mut job = [0usize; LANES];''')
s=s.replace('                paths[lane].push(n.first);','''                paths[lane].push(n.first);
                if PATH==3 {
                    let p=&mut paths[lane];
                    let base=p.as_mut_ptr();
                    // SAFETY: the first byte is initialized and capacity covers it.
                    cursor[lane]=unsafe {base.add(1)};
                    limit[lane]=unsafe {base.add(p.capacity())};
                }''')
anchor='''            #[cfg(test)]
            {
                self.histogram[live.count_ones() as usize] += 1;
            }'''
assert s.count(anchor)==1
s=s.replace(anchor,anchor+'''
            if PATH==1 {
                if wave%8==0 {
                    spare=paths.iter().all(|p|p.capacity().saturating_sub(p.len().max(1))>=8);
                }
                wave+=1;
            } else if PATH==2 {
                if budget==0 {
                    budget=paths.iter().map(|p|p.capacity().saturating_sub(p.len().max(1))).min().unwrap();
                }
                spare=budget!=0;
                budget=budget.saturating_sub(1);
            }''')
assert s.count('paths[lane].push(r.byte());')==5
s=s.replace('paths[lane].push(r.byte());','append!(lane,r.byte());')
s=s.replace('                self.edges.extend_from_slice(&paths[lane]);','''                if PATH==3 {
                    let p=&mut paths[lane];
                    // SAFETY: reset and every grow set cursor to base+len;
                    // each in-place advance initialized exactly one further byte.
                    unsafe {
                        let len=cursor[lane].offset_from_unsigned(p.as_ptr());
                        debug_assert!(len<=p.capacity());
                        p.set_len(len);
                    }
                }
                self.edges.extend_from_slice(&paths[lane]);''')
(here/'family.rs').write_text(s)
