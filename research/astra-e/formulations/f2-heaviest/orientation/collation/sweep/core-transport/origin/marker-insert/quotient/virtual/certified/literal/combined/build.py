from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
direct=HERE.parents[2]/'direct'
out=Path(sys.argv[1]);deps=sys.argv[2]
subprocess.run([sys.executable,'-B',str(HERE.parent/'build.py'),str(out),deps],check=True)
def one(s,a,b):
    assert s.count(a)==1,a
    return s.replace(a,b)
p=out/'second.rs';s=p.read_text()
s=one(s,'    pub(super) supplied_child:','    pub(super) use_child:bool,\n    pub(super) supplied_child:')
s=one(s,'if marker==d{f.order.push(child);}','if self.use_child && marker==d{f.order.push(child);}')
a=s.index('    fn forest_index(');b=s.index('        let f=&mut self.forest;',a)
c=s.index('\n    }\n}',b)
body=s[b:c]
body=one(body,'for r in root {\n            let n=nodes[r.id as usize];let c=n.word as usize;','for (word,width) in input {\n            let c=word as usize;')
body=body.replace('n.word','word').replace('r.width','width')
a=s.index('        if self.stream {self.discover::<true>')
c=s.index('\nimpl Second{\n    pub(super) fn capacity',a)
tail=s[a:c]
tail=one(tail,'if self.stream {self.discover::<true>(d,first_words,bytes);} else {self.discover::<false>(d,first_words,bytes);}','self.discover::<true>(d,first_words,bytes);')
mix=one((direct/'direct.rs').read_text(),'self.direct_index(make(),actual,words,bytes,row,skip,m+extra)','self.known_index(make(),actual,words,bytes,row,skip,m+extra)')
p.write_text(s+'''\nimpl Second {
    fn known_index(&mut self,input:impl Iterator<Item=(u32,u32)>,actual:usize,words:&[Piece],buf:&[u8],p:u32,mut skip:u32,m:u32)->Option<(u32,u32,u32)>{
        let k=words.len();let d=actual as u32;if self.count[actual]==m{return None;}
'''+body+'\n    }\n'+'''
    fn virtual_finish(&mut self,d:u32,mut row:u32,mut begin:u32,first_words:&[Piece],bytes:&[u8],n:u32,out:&mut Vec<u8>)->bool{
        let t1=crate::now();let token_count=self.count[d as usize];
'''+tail+mix)
p=out/'control.rs';s=(direct/'control.rs').read_text()
s=one(s,'let mut z=NONE;let mut any=false;','let mut z=NONE;let mut brace=NONE;let mut any=false;')
s=one(s,'else {self.links[i]=1;any=true;}','else {self.links[i]=1;any=true;if w.width==3 && self.dict[w.id as usize]==b\'}\'{brace=i as u32;}}')
s=one(s,'        let src=&self.merged','        if brace!=NONE {self.second.supplied_child=Some((z,brace));}\n        let src=&self.merged')
p.write_text(p.read_text()+s+'''\nimpl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE>{
    pub fn set_child(&mut self,on:bool){self.second.use_child=on;}
}\n''')
(out/'virtual_column.rs').write_text((direct/'iter.rs').read_text())
p=out/'main.rs';s=p.read_text()
s=one(s,'self.fast.set_root_stream(inspect!=0 || route>=6);self.fast.set_forest(inspect!=0 || route>=12);','let direct=route>=24;self.fast.set_root_stream(true);self.fast.set_forest(true);self.fast.set_child(route>=36 || (!direct && route>=12));')
s=one(s,'        let mut counts=[0;10];let mut publish=0;','''        if direct {
            let h=[4,8,12][route-3];
            let (mark,m,counts,second,obj)=self.fast.direct(mark,m,x.n,x.p,h,&mut self.out,inspect==2);
            return ResultRow{counts,tokens:m,mark,obj,second,winner:if second && inspect!=0{self.fast.chosen_base()}else{u32::MAX},
                first:self.fast.phases,inner:if second{self.fast.second_phases()}else{[0;4]},publish:0,total:now()-t,
                info:if second{self.fast.information()}else{[0;7]},cap:self.fast.whole_capacity()+self.out.capacity()};
        }
        let mut counts=[0;10];let mut publish=0;''')
s=one(s,'[(3,4),(4,8),(5,12)]','[(39,4),(40,8),(41,12)]')
s=one(s,'[15,17].into_iter()','[39,41].into_iter()')
a=s.index('fn price(path:&str)');b=s.index('fn rss(',a)
s=s[:a]+(HERE/'rotor.rs').read_text()+s[b:]
s=one(s,'let code=[0,12,3,15,5,17][arm];let name=["named","named_forest","quot4","quot4_forest","quot12","quot12_forest"][arm];','let code=ROSTER[arm];let name=POLICY[arm];')
p.write_text(s)
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-A','unused_mut','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',str(p),'--extern','mibz2='+deps+'/libmibz2.rlib','-L','dependency='+deps,'-o',str(out/'combined')],check=True)
