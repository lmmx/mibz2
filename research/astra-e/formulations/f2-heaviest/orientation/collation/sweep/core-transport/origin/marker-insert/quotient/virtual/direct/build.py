"""Build the dual-cursor root on the tested native auxiliary source."""
from pathlib import Path
import subprocess
import sys
HERE=Path(__file__).resolve().parent
p=Path(sys.argv[1]);deps=sys.argv[2]
subprocess.run([sys.executable,'-B',str(HERE.parent/'native'/'build.py'),str(p),deps],check=True)
def one(s,a,b):
    assert s.count(a)==1,a
    return s.replace(a,b)
control=p/'control.rs'
control.write_text(control.read_text()+(HERE/'control.rs').read_text())
(p/'virtual_column.rs').write_text((HERE/'iter.rs').read_text())

sec=p/'second.rs'
s=sec.read_text()
# Start from the tested complement-index builder after census and winner identification.
body=s[s.index('        let (b,s)=(self.base[pivot],self.count[pivot]);'):]
assert body.endswith('    }\n}\n')
body=body[:-len('    }\n}\n')]
body=one(body,'for piece in root {\n            let node=&nodes[piece.id as usize];\n            let k=node.word as usize;\n            let next=at+piece.width;', 'for (word,width) in input {\n            let k=word as usize;\n            let next=at+width;')
body=body.replace('node.base','self.base[k]').replace('piece.width','width').replace('node.word','word')
body=one(body,'if at<b {self.runs.push(TokenRun{start:at,lf,word:word});}','if at<b {self.outer_append(at,lf,word);}')
body=one(body,'self.runs.push(TokenRun{start:lo-s,lf:lf+lo-at,word:word});','self.outer_append(lo-s,lf+lo-at,word);')
tail=s[s.index('        if self.stream {self.discover::<true>'):s.index('\nimpl Second{\n    pub(super) fn capacity')]
tail=one(tail,'if self.stream {self.discover::<true>(d,first_words,bytes);} else {self.discover::<false>(d,first_words,bytes);}','self.discover::<true>(d,first_words,bytes);')
sec.write_text(s+'\nimpl Second {\n'+'''
    fn outer_append(&mut self,start:u32,lf:u32,word:u32) {
        if let Some(prev)=self.runs.last(){
            if prev.word==word && prev.lf+start-prev.start==lf{return;}
        }
        self.runs.push(TokenRun{start,lf,word});
    }
    fn direct_index(&mut self,input:impl Iterator<Item=(u32,u32)>,pivot:usize,words:&[Piece],bytes:&[u8],p:u32,mut skip:u32,m:u32)->Option<(u32,u32,u32)>{
        let alphabet=words.len();
'''+body+'\n    }\n'+'''
    fn virtual_finish(&mut self,d:u32,mut row:u32,mut begin:u32,first_words:&[Piece],bytes:&[u8],n:u32,out:&mut Vec<u8>)->bool{
        let t1=crate::now();
        let token_count=self.count[d as usize];
'''+tail+(HERE/'direct.rs').read_text())
main=p/'main.rs'
s=main.read_text()
s=one(s,'self.fast.set_root_stream(inspect!=0 || route>=6);','let direct=route>=12;\n        self.fast.set_root_stream(inspect!=0 || route>=6);')
old='        let mut counts=[0;10];let mut publish=0;'
s=one(s,old,'''        if direct {
            let h=[4,8,12][route-3];
            let (mark,m,counts,second,obj)=self.fast.direct(mark,m,x.n,x.p,h,&mut self.out,inspect==2);
            return ResultRow{counts,tokens:m,mark,obj,second,winner:if second && inspect!=0{self.fast.chosen_base()}else{u32::MAX},
                first:self.fast.phases,inner:if second{self.fast.second_phases()}else{[0;4]},publish:0,total:now()-t,
                info:if second{self.fast.information()}else{[0;7]},cap:self.fast.whole_capacity()+self.out.capacity()};
        }
'''+old)
s=one(s,'[(3,4),(4,8),(5,12)]','[(15,4),(16,8),(17,12)]')
s=one(s,'if arm==1{0}else{arm},if arm>=3','if arm==1{0}else if arm>=3{arm+12}else{arm},if arm>=3')
s=one(s,'let codes=[0,6,3,9,4,10];','let codes=[0,3,9,15,4,16];')
s=one(s,'let names=["named","named_stream","quot4","quot4_stream","quot8","quot8_stream"]','let names=["named","quot4","quot4_stream","direct4","quot8","direct8"]')
s=one(s,'let code=[0,6,3,9,4,10][arm];','let code=[0,3,9,15,4,16][arm];')
s=one(s,'let name=["named","named_stream","quot4","quot4_stream","quot8","quot8_stream"][arm];','let name=["named","quot4","quot4_stream","direct4","quot8","direct8"][arm];')
main.write_text(s)
subprocess.run(['rustc','--edition=2021','-A','dead_code','-A','unexpected_cfgs','-A','unused_imports','-A','unused_mut','-C','opt-level=3','-C','codegen-units=1','--cfg','clipped_profile',str(main),'--extern','mibz2='+deps+'/libmibz2.rlib','-L','dependency='+deps,'-o',str(p/'direct')],check=True)
