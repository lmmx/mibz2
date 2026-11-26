from pathlib import Path
import re,subprocess,sys
h=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(h.parent/'harness.py'),str(out)],check=True)
price=(h.parents[3]/'f2c/price_impl.rs').read_text().replace('self.prepare(r.records(),n as u32)','self.prepare_counted(r.records(),n as u32,r.byte_counts())')
for mod,file,width in [('control',h.parent/'candidate.rs',False),('widths',h/'candidate.rs',True)]:
    s=file.read_text()
    def resolve(m):return m[0].replace(m[1],str((file.parent/m[1]).resolve()))
    s=re.sub(r'(?:#\[path = "|include!\(")([^"]+)"',resolve,s)
    impl='impl<const MODE:u8,const SCHEDULE:u8,const LANES:usize> Dispatch<MODE,SCHEDULE,LANES> {' if width else 'impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {'
    paths='self.lane_paths.0' if width else 'self.lane_paths'
    s+='\n'+price.replace('impl Candidate {',impl)
    s+='\n'+impl+' pub fn path_bytes(&self)->usize { '+paths+'.iter().map(Vec::capacity).sum()} }'
    if width:
        s+='\n#[cfg(slot_counts)] '+impl+'''
            pub fn shape(&self)->(u64,u64,u64) {
                let mut waves=0;let mut queries=0;let mut full=0;
                for a in 1..=LANES {
                    for &count in &self.wave_counts[a] {
                        waves+=count;queries+=a as u64*count;
                        full+=u64::from(a==LANES)*count;
                    }
                } (waves,queries,full)
            }
        }
'''
    (out/(mod+'.rs')).write_text(s)
s=(out/'focus.rs').read_text().replace('mod slots;', 'mod widths;')
a=s.index('    let mut old=');b=s.index('    let mut tmp=',a)
s=s[:a]+'''    let mut old=control::Dispatch::<2,2>::default();
    let mut l4=widths::Dispatch::<2,2,4>::default();
    let mut l8=widths::Dispatch::<2,2,8>::default();
    let mut l12=widths::Dispatch::<2,2,12>::default();
    let mut l16=widths::Dispatch::<2,2,16>::default();
    let mut l24=widths::Dispatch::<2,2,24>::default();
'''+s[b:]
a=s.index('    if full {println!(');b=s.index('\n',a)
s=s[:a]+'''    if full {println!("block,rows,arm,path_capacity_bytes,waves,queries,full_waves");}'''+s[b:]
a=s.index('            macro_rules! check');b=s.index('            continue;',a)
s=s[:a]+'''            assert!(old.invert_runs(&r,n,symbols.orig_ptr,&mut tmp,kn)); assert_eq!(tmp,want);
            macro_rules! check { ($arm:ident,$name:expr) => {{
                assert!($arm.invert_runs(&r,n,symbols.orig_ptr,&mut tmp,kn)); assert_eq!(tmp,want);
                #[cfg(slot_counts)] {
                    let (waves,queries,full)=$arm.shape();
                    println!("{index},{n},{},{},{waves},{queries},{full}",$name,$arm.path_bytes());
                }
            }}; }
            check!(l4,4);check!(l8,8);check!(l12,12);check!(l16,16);check!(l24,24);
'''+s[b:]
s=s.replace('let mut outputs: [Vec<u8>;4]','let mut outputs: [Vec<u8>;6]')
a=s.index('    let perms:');b=s.index('    println!("block,round',a)
s=s[:a]+'''    let perms: [[usize;6];12]=std::array::from_fn(|r| std::array::from_fn(|j| {
        let base=[0usize,1,5,2,4,3]; (base[if r<6 {j} else {5-j}] + r%6)%6
    }));
'''+s[b:]
s=s.replace('pass in 0..=64','pass in 0..=60').replace('perms[(pass+bi)%8]', 'perms[(pass+bi)%12]')
a=s.index('                    0=>old.measure');b=s.index('\n                };',a)
s=s[:a]+'''                    0=>old.measure(&b.r,b.n,b.mark,out,true),
                    1=>l4.measure(&b.r,b.n,b.mark,out,true),
                    2=>l8.measure(&b.r,b.n,b.mark,out,true),
                    3=>l12.measure(&b.r,b.n,b.mark,out,true),
                    4=>l16.measure(&b.r,b.n,b.mark,out,true),
                    _=>l24.measure(&b.r,b.n,b.mark,out,true),'''+s[b:]
s=s.replace('["mask","fixed","preload","bulk"]','["frozen8","width4","width8","width12","width16","width24"]')
(out/'focus.rs').write_text(s)
