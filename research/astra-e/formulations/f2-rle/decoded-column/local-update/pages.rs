struct Column {
    pages: Vec<Vec<usize>>,
    which: Vec<usize>,
    count: Vec<[u32; 256]>,
    histogram: [u32; 256],
    byte: Vec<u8>,
    pred: Vec<usize>,
    succ: Vec<usize>,
    n: usize,
    moves: usize,
    longest: usize,
    operations: usize,
    displacement: usize,
}
impl Column {
    fn new(text: &[u8], old_order: Vec<usize>) -> Self {
        let n=text.len();
        let pages:Vec<_>=old_order.chunks(512).map(|a|a.to_vec()).collect();
        let mut which=vec![0;n];let mut count=vec![[0;256];pages.len()];let mut histogram=[0;256];
        let pred:Vec<_>=(0..n).map(|i|(i+n-1)%n).collect();
        let succ:Vec<_>=(0..n).map(|i|(i+1)%n).collect();
        for (p,pg) in pages.iter().enumerate(){for &i in pg{which[i]=p;count[p][text[pred[i]] as usize]+=1;histogram[text[i]as usize]+=1;}}
        Self{pages,which,count,histogram,byte:text.to_vec(),pred,succ,n,moves:0,longest:0,operations:0,displacement:0}
    }
    fn offset(&self,id:usize)->(usize,usize) {
        let p=self.which[id];(p,self.pages[p].iter().position(|&x|x==id).unwrap())
    }
    fn position(&self,id:usize)->usize{
        let (p,i)=self.offset(id);self.pages[..p].iter().map(Vec::len).sum::<usize>()+i
    }
    fn rank(&self,c:u8,id:usize)->usize{
        let (p,k)=self.offset(id);
        self.count[..p].iter().map(|v|v[c as usize]as usize).sum::<usize>()+
        self.pages[p][..k].iter().filter(|&&x|self.byte[self.pred[x]]==c).count()
    }
    fn base(&self,c:u8)->usize{self.histogram[..c as usize].iter().map(|&v|v as usize).sum()}
    fn remove(&mut self,id:usize){
        let (p,k)=self.offset(id);self.pages[p].remove(k);self.count[p][self.byte[self.pred[id]]as usize]-=1;self.n-=1;
    }
    fn place(&mut self,id:usize,mut at:usize){
        assert!(at<=self.n);
        let mut p=0;
        while p+1<self.pages.len()&&at>self.pages[p].len(){at-=self.pages[p].len();p+=1;}
        self.pages[p].insert(at,id);self.which[id]=p;self.count[p][self.byte[self.pred[id]]as usize]+=1; self.n+=1;
    }
    fn change_l(&mut self,id:usize,old:u8,new:u8){
        let p=self.which[id];self.count[p][old as usize]-=1;self.count[p][new as usize]+=1;
    }
    fn fix(&mut self,mut x:usize)->bool{
        self.operations+=1;
        for depth in 0..=self.n{
            let c=self.byte[x];let target=self.base(c)+self.rank(c,self.succ[x]);let old=self.position(x);
            if old==target {self.longest=self.longest.max(depth);return true;}
            self.moves+=1;self.displacement+=old.abs_diff(target);
            self.remove(x);self.place(x,target);x=self.pred[x];
        }
        false
    }
    fn replace(&mut self,x:usize,b:u8)->bool{
        let old=self.byte[x];if old==b{return true;}
        self.change_l(self.succ[x],old,b);self.byte[x]=b;self.histogram[old as usize]-=1;self.histogram[b as usize]+=1;
        self.fix(x)
    }
    fn delete(&mut self,y:usize)->bool{
        assert!(self.n>1);let a=self.pred[y];let j=self.succ[y];
        self.remove(y);self.histogram[self.byte[y]as usize]-=1;self.change_l(j,self.byte[y],self.byte[a]);
        self.succ[a]=j;self.pred[j]=a;
        self.fix(a)
    }
    fn insert_before(&mut self,j:usize,b:u8)->bool{
        let a=self.pred[j];let q=self.base(b)-usize::from(self.byte[a]<b)+self.rank(b,j);
        let pos=q+usize::from(self.position(a)<=q);
        let y=self.byte.len();self.which.push(0);self.byte.push(b);self.pred.push(a);self.succ.push(j);
        self.change_l(j,self.byte[a],b);self.pred[j]=y;self.succ[a]=y;
        self.place(y,pos);self.histogram[b as usize]+=1;
        self.fix(a)
    }
}
