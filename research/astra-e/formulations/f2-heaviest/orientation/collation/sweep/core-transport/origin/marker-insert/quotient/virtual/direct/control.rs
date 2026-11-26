#[path="virtual_column.rs"] mod virtual_column;
impl<const MODE:u8,const SCHEDULE:u8> Dispatch<MODE,SCHEDULE> {
    pub fn direct(&mut self,mark:Option<(u32,u32)>,m:u32,n:u32,p:u32,h:u32,out:&mut Vec<u8>,inspect:bool)
        ->(Option<(u32,u32)>,u32,[u32;10],bool,Option<crate::Obj>){
        self.links.clear();self.links.resize(self.words.len(),0);
        let mut z=NONE;let mut any=false;
        for (i,w) in self.words.iter().enumerate(){
            if w.width>=2 && w.width<=h+1 && self.dict[(w.id+w.width-2) as usize]==b',' {
                if w.width==2 {z=i as u32;} else {self.links[i]=1;any=true;}
            }
        }
        if z==NONE || !any {
            let (q,m,c)=self.publish_quotient(mark,m,h);
            let o=inspect.then(||self.object());
            let t=self.finish(q,m,n,p,out);
            return (q,m,c,t,o);
        }
        let src=&self.merged[self.nodes[0].out as usize..];
        let result=self.second.mixed(src,&self.nodes,&mut self.words,&self.dict,&self.links,z,mark,m,n,out,inspect);
        if !result.3 {self.finish(None,m,n,p,out);}
        result
    }
}
