from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
out=Path(sys.argv[1]);deps=sys.argv[2]
subprocess.run([sys.executable,'-B',str(HERE.parent/'sparse-native'/'build.py'),str(out),deps],check=True)
def one(s,a,b):
    assert s.count(a)==1,a
    return s.replace(a,b)
p=out/'second.rs';s=p.read_text()
s=one(s,'    pub(super) use_forest:bool,','    pub(super) use_forest:bool,\n    pub(super) supplied_child:Option<(u32,u32)>,')
s=one(s,'Self::reset(&mut self.forest.first,k,m);Self::reset(&mut self.forest.last,k,0);','')
s=one(s,'self.forest.first[c]=self.forest.first[c].min(at);','')
s=one(s,'at+=r.width;self.forest.last[c]=at;','')
s=one(s,'        let mut at=0;\n        for r in root {\n            let n=nodes[r.id as usize];let c=n.word as usize;','        for r in root {\n            let n=nodes[r.id as usize];let c=n.word as usize;')
a=s.index('        let b=self.base[d as usize];let z=b+self.count[d as usize];');b=s.index('        let root_pos=',a)
s=s[:a]+'''        let b=self.base[d as usize];
        f.order.clear();
        if let Some((marker,child))=self.supplied_child{if marker==d{f.order.push(child);}}
'''+s[b:]
s=one(s,'        Self::reset(&mut f.slot,k,NONE);','')
s=one(s,'let slot=j as u32;f.slot[w as usize]=slot;','let slot=j as u32;')
s=one(s,'if f.slot[c]==NONE {','if !f.bfs.contains(&(c as u32)) {')
s=s.replace('self.forest.slot[k]','self.fixed_slot(k as u32)').replace('self.forest.slot[label]','self.fixed_slot(label as u32)').replace('self.forest.slot[r.word as usize]','self.fixed_slot(r.word)').replace('self.forest.slot[next.unwrap() as usize]','self.fixed_slot(next.unwrap())')
s+='''\nimpl Second {
    #[inline] fn fixed_slot(&self,w:u32)->u32{
        if self.forest.bfs[0]==w{0}else if self.forest.bfs.get(1)==Some(&w){1}else{NONE}
    }
}\n'''
p.write_text(s)
p=out/'control.rs';s=p.read_text()
a='pub fn first(&mut self,caps:&[u16],freq:&[u32;256],n:u32,p:u32 )->Option<(Option<(u32,u32)>,u32)>{'
s=one(s,a,a+'\n        self.second.supplied_child=None;')
s=one(s,'let (mut zid,mut types)=(NONE,0);','let (mut zid,mut types,mut brace)=(NONE,0,NONE);')
s=one(s,'else {self.links[i]=1;types+=1;}','else {self.links[i]=1;types+=1;if w.width==3 && self.dict[w.id as usize]==b\'}\'{brace=i as u32;}}')
a='if types==0{return (old,m,[1,0,0,0,0,0,len,len,0,0]);}'
s=one(s,a,a+'\n        if brace!=NONE {self.second.supplied_child=Some((zid,brace));}')
p.write_text(s)
p=out/'main.rs';s=p.read_text()
s=one(s,'"metrics"=>forest_metrics(&args[2]),','"held"=>held(&args[2]),"metrics"=>forest_metrics(&args[2]),')
p.write_text(s+(HERE/'held.rs').read_text())
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-A','unused_mut','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',str(p),'--extern','mibz2='+deps+'/libmibz2.rlib','-L','dependency='+deps,'-o',str(out/'literal')],check=True)
