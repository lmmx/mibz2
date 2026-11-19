#[derive(Clone,Copy)]
struct Near {row:u32, byte:u8, kind:u8, neighbors:u8}
#[derive(Default)]
struct Selector { near:Vec<Near>, moved:Vec<u32> }

impl Selector {
    fn plan<const POLICY:u8>(&mut self,records:&[u16],n:usize,mark:u32)->(u32,u32,usize) {
        self.near.clear();
        self.moved.clear();
        let mut total=[0u32;256];
        let mut omitted=[0u32;256];
        let mut prev_image=[false;256];
        let mut pend_image=[usize::MAX;256];
        let mut prev=false;
        let mut pend=usize::MAX;
        let (mut row,mut pos,mut retained,mut strips)=(0u32,0,0u32,0u32);
        let mut available=[0usize;2];
        while pos<records.len() {
            let c=records[pos]>>8;
            let mut width=0u32;
            while pos<records.len() && records[pos]>>8==c {
                width+=u32::from(records[pos]&255);
                pos+=1;
            }
            let a=usize::from(c);let thick=width>16;
            let contains=(row..row+width).contains(&mark);
            if !thick {retained+=width;}
            else if contains {
                retained+=1;strips+=u32::from(row!=mark)+u32::from(row+width!=mark+1);
            } else {strips+=1;}
            if POLICY==4 {
                total[a]+=width;
                if thick {omitted[a]+=width;}
            }
            if POLICY==2 || POLICY==3 {
                if pend!=usize::MAX {self.near[pend].neighbors+=u8::from(thick);}
                if pend_image[a]!=usize::MAX {self.near[pend_image[a]].neighbors+=u8::from(thick);}
            }
            let selected=(width==16 || width==17) && !contains;
            if selected {
                let idx=self.near.len();
                self.near.push(Near{row,byte:c as u8,kind:(width-16) as u8,neighbors:u8::from(prev)+u8::from(prev_image[a])});
                available[(width-16) as usize]+=1;
                pend=idx;pend_image[a]=idx;
            } else {pend=usize::MAX;pend_image[a]=usize::MAX;}
            prev=thick;prev_image[a]=thick;
            row+=width;
        }
        assert_eq!(row as usize,n);
        let bits=(32-(retained-1).leading_zeros()).max(1);
        let mut q=128.min(available[0]).min(available[1]).min(((1u32<<bits)-retained) as usize);
        if records.len()*20>=n*3 {
            let per=if 255+3*strips+256<=u32::MAX>>bits {4}else{8};
            q=q.min(((1u32<<20)/per).saturating_sub(retained) as usize);
        }
        let mut bucket=[0u8;256];
        if POLICY==4 {for i in 0..256 {if total[i]>0 {bucket[i]=(8*omitted[i]/total[i]) as u8;}}}
        let priority=|item:&Near| ->usize {
            if POLICY==1 {return 0}
            let s=if POLICY==4 {bucket[item.byte as usize]} else {item.neighbors};
            let top=if POLICY==4 {8} else {4};
            if (item.kind==0) != (POLICY==3) {(top-s) as usize} else {s as usize}
        };
        let mut counts=[[0usize;9];2];
        for z in &self.near {counts[z.kind as usize][priority(z)]+=1;}
        for row in &mut counts {
            let mut left=q;
            for amount in row {let take=(*amount).min(left);*amount=take;left-=take;}
            assert_eq!(left,0);
        }
        for z in &self.near {
            let k=&mut counts[z.kind as usize][priority(z)];
            if *k>0 {*k-=1;self.moved.push(z.row);}
        }
        assert_eq!(self.moved.len(),2*q);
        (retained,strips,q)
    }
}
