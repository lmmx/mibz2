"""Generate isolated variants from pinned Compact/preload and Second sources."""
import hashlib
from pathlib import Path
import sys
from shared import materialize
D=Path(__file__).resolve().parent
F=D.parents[4]
def checked(path,digest):
    b=path.read_bytes()
    assert hashlib.sha256(b).hexdigest()==digest,path
    return b.decode()

def once(s,a,b):
    assert s.count(a)==1,(a,s.count(a))
    return s.replace(a,b)

def main(out):
    out.mkdir(exist_ok=True,parents=True)
    base=checked(F/'f2-pages/payload/slots/candidate.rs','84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06')
    second=checked(F/'f2c/second.rs','877dd1870572c4c5237cbdd1362b8d7124c4393abdcbcebb5194e1fadcffc992')
    (out/'second.rs').write_text(second+(D/'second-cap.rs').read_text())
    base=once(base,'../../../f2c/second.rs','second.rs')
    api=(D/'api.rs').read_text()+(D/'cap.rs').read_text()
    base=once(base,'    fallback: Segmented,','    #[cfg(clipped_profile)] pub phases:[u64;3],\n    fallback: Segmented,')
    ctrl=base+'\n'+api.replace('PARAMS','const MODE:u8,const SCHEDULE:u8').replace('TYPE','Dispatch<MODE,SCHEDULE>').replace('START_MARK','self.prepare_counted(caps,n,freq)?').replace('AFTER_MERGE','').replace('SECTION_SIGNATURE','').replace('EXTRA','0')
    (out/'control.rs').write_text(ctrl)
    # Retain the original eight-lane preload loop, popcount index, merge and writer.
    s=base[:base.index('    #[cfg(test)]\n    fn attempt<const FORCE')]
    s=s.replace('const MODE:u8,const SCHEDULE:u8=1','const MODE:u8,const SCHEDULE:u8=1,const ONLY:bool=true')
    s=s.replace('impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>', 'impl<const MODE:u8,const SCHEDULE:u8,const ONLY:bool> Dispatch<MODE,SCHEDULE,ONLY>')
    s=once(s,'const NONE: u32 = u32::MAX;','const NONE: u32 = u32::MAX;\nconst LIVE:u32=1<<31;')
    s=once(s,'self.position_byte<(hi<<8) || self.byte()==QUOTE','self.position_byte<((hi<<8)|LIVE)')
    s=once(s,'fn new(pos:u32, byte:u8, delta:u32)->Self {','fn terminal(self)->bool {self.position_byte & LIVE == 0}\n    #[inline]\n    fn new(pos:u32, byte:u8, delta:u32, terminal:bool)->Self {')
    s=once(s,'position_byte:(pos<<8) | u32::from(byte)','position_byte:(pos<<8) | u32::from(byte) | if terminal {0} else {LIVE}')
    s=once(s,'self.position_byte >> 8','(self.position_byte & !LIVE) >> 8')
    s=once(s,'    fallback: Segmented,','    section:(u32,u32),\n    symbol:u8,\n    bases:Vec<u32>,\n    #[cfg(clipped_counts)] pub mixed:u32,\n    #[cfg(clipped_counts)] pub interior_splits:u32,')
    a=s.index('        let tokens = counts[QUOTE as usize];');b=s.index('        let mut preceding = 0u32;',a)
    s=s[:a]+(D/'prepare.rs').read_text()+s[b:]
    s=once(s,'Some((quote, tokens))','Some((self.section.0,self.section.1-self.section.0))')
    a=s.index('        let (mut x, mut skip) = (p, 0);');b=s.index('\n    }',a)
    s=s[:a]+'''        let (mut x,mut skip)=(p,0);
        if !ONLY && p>=quote && p<self.section.1{return Some((p-quote,0));}
        loop {
            let r=self.runs[self.locate(x)];
            x=r.map(x);
            if r.terminal(){return Some((x-quote,skip+u32::from(!ONLY)));}
            skip+=1;
            if x==p{return None;}
        }'''+s[b:]
    s=once(s,'let mut root = Node::new(quote, NONE, QUOTE);','''self.bases.clear();
        #[cfg(clipped_counts)] {self.mixed=0;}
        let mut root = Node::new(quote, NONE, self.symbol);''')
    s=once(s,'let mut active = [0usize; 256];','let mut active = [0usize; 256];\n        let mut terminal_epoch=[NONE;256];\n        let mut terminal_word=[0u32;256];')
    s=once(s,'paths[lane].push(n.first);','if ONLY || job[lane]!=0{paths[lane].push(n.first);}')
    a=s.index('                    let id = if r.byte() == QUOTE {');b=s.index('                        if epoch[c] != i as u32 {',a)
    s=s[:a]+(D/'terminal.rs').read_text()+s[b:]
    s=once(s,'                            epoch[c] = i as u32;', '''                            #[cfg(clipped_counts)] {self.mixed+=u32::from(terminal_epoch[c]==i as u32);}
                            epoch[c] = i as u32;''')
    s=once(s,'if id == NONE {\n                    Self::emit(&mut self.merged, start, i as u32, width);','''if (ONLY && id==NONE) || (!ONLY && id&LIVE!=0) {
                    Self::emit(&mut self.merged,start,if ONLY {i as u32} else {!id},width);''')
    s+='\n}\n'
    s+=api.replace('PARAMS','const MODE:u8,const SCHEDULE:u8,const ONLY:bool').replace('TYPE','Dispatch<MODE,SCHEDULE,ONLY>').replace('START_MARK','{self.section=sec;self.symbol=first;self.prepare_counted(caps,n,freq)?}').replace('AFTER_MERGE',(D/'bases.rs').read_text()).replace('SECTION_SIGNATURE',',sec:(u32,u32), first:u8').replace('EXTRA','bytes(&self.bases)')
    (out/'clip.rs').write_text(s)
    fast=once(s,(D/'prepare.rs').read_text(),(D/'setup-fast.rs').read_text())
    fast=once(fast,'    fn prepare(&mut self,',(D/'flush.rs').read_text()+'    fn prepare(&mut self,')
    (out/'fast.rs').write_text(fast)
    (out/'shared.rs').write_text(materialize(fast))
    (out/'memory.rs').write_text((D/'shared-main.rs').read_text())
    (out/'main.rs').write_text((D/'main.rs').read_text())
if __name__=='__main__':main(Path(sys.argv[1]))
