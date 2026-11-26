"""Build a rank/select navigator from the already constructed Open section."""
from pathlib import Path
from hashlib import sha256
d=Path(__file__).resolve().parent
base=d.parent
journal=d.parents[4]/"docs/journal/2026-11-18-open-return"
raw=(journal/"open_return.rs").read_text()
assert sha256(raw.encode()).hexdigest()=="b02a1d2ed7490a0dbef8f5ecf464153d75d2cd71b42993807a3a713bc18bbfc1"
s=raw[:raw.index("    pub fn run<const INDUCE:")]+"}\n"+raw[raw.index("fn emit("):raw.index("fn records_of(")]
s=s.replace("use crate::bwt::{InverseBwt, Segmented};","").replace("use crate::{runs::Runs, Kernels};","")
# Arrays of 256 lack Default on the pinned toolchain. Keep the existing constructor's
# Default derivation through one small, explicit fixed-size wrapper.
s=s.replace("#[derive(Default)]\npub struct Kernel",'''struct Bases([u32;256]);
impl Default for Bases {fn default()->Self{Self([0;256])}}
impl std::ops::Deref for Bases {type Target=[u32;256];fn deref(&self)->&[u32;256]{&self.0}}
#[derive(Default)]
pub struct Kernel''')
s=s.replace("    segments: segments::Segments,",'''    fbase:Bases,thin_base:Bases,thick_base:Bases,
    source_width:Vec<u32>,image_width:Vec<u32>,events:Vec<u32>,danger:Vec<u32>,owners:Vec<u32>,labels:Vec<u32>,raw:Vec<u8>,steps:usize,worked:bool,reason:u8,''')
s=s.replace("        let mut write = starts(thick);",'''        let mut write = starts(thick);
        self.fbase.0=first;self.thin_base.0=starts(thin);self.thick_base.0=write;
        self.source_width.resize(self.strips.len()+1,0);
        self.image_width.resize(self.strips.len()+1,0);
        let mut image_mass=starts(std::array::from_fn(|c|all[c]-thin[c]));
        let mut seen_width=0;
        self.source_width[self.strips.len()]=n-m;
        self.image_width[self.strips.len()]=n-m;''')
s=s.replace("            s.to += first[a];",'''            self.source_width[i]=seen_width;
            self.image_width[write[a] as usize]=image_mass[a];
            seen_width+=s.width;image_mass[a]+=s.width;
            s.to += first[a];''')
w=(base/"saturated/writer.rs").read_text()
w=w.replace("        h: &super::section::Hazards,\n","")
i=w.index("        let (s, mark, bits)")
j=w.index("        self.owners.clear();", i)
w=w[:i]+'''        let (s,mark,bits)=self.construct::<true,true>(caps,n as u32,p,16);
        self.reason=0;
        self.labels.clear();self.steps=0;self.worked=false;
        if !self.hazards(n as u32){
            self.reason=1;
            emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,&mut self.raw);
            return mibz2::rle::unrle(&self.raw,out,k);
        }
'''+w[j:]
w=w.replace("for &start in &h.danger", "for &start in &self.danger").replace("if self.steps > 4096 {","if self.steps > 4096 {self.reason=2;")
safety='''        if self.extra.is_empty() && !self.owners.is_empty() && first+self.owners.len()as u32-1 > u32::MAX>>bits {
            self.reason=3;self.worked=false;
            emit(&self.edges,&self.extra,&self.spelling,&self.offsets,n,mark,bits,&mut self.raw);
            return mibz2::rle::unrle(&self.raw,out,k);
        }
'''
w=w.replace("        let mask =",safety+"        let mask =")
w+='''impl Kernel{
    pub fn counts(&self)->(usize,usize,usize,usize,u8){(self.events.len(),self.danger.len(),self.steps,self.labels.len(),self.reason)}
    pub fn verify_coordinates(&self,events:&[u32],danger:&[u32]){assert_eq!(self.events,events);assert_eq!(self.danger,danger);}
}
'''
s+='''#[path="../../../../../docs/journal/2026-11-18-open-return/endpoint.rs"] mod endpoint;
include!("nav.rs");
'''+w
(d/"open.rs").write_text(s)
other=(base/"candidate.rs").read_text().split("mod section;",1)[1]
other=other.replace("    hazards: section::Hazards,\n","").replace("            hazards: section::Hazards::new(),\n","")
other=other.replace("        _: Kernels,","        k: Kernels,").replace(" || !self.hazards.build(caps, 1024)","").replace("&self.hazards, out)","out,k)")
(d/"adapter.inc.rs").write_text(other)
