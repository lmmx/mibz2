#[derive(Clone,Copy,Default)]
struct Partner { end:u32, delta:u32, code:u32 }

#[derive(Default)]
struct PairDirectory { starts:Vec<u64>, prefix:Vec<u32>, partner:Vec<Partner> }

impl PairDirectory {
    fn index(&self,x:u32)->usize {
        let page=x as usize/64;
        (self.prefix[page]+(self.starts[page]&((1u64<<(x%64))-1)).count_ones()) as usize
    }

    fn containing(&self,x:u32)->usize {
        let page=x as usize/64;
        let mask=u64::MAX>>(63-(x%64));
        (self.prefix[page]+(self.starts[page]&mask).count_ones()-1) as usize
    }

    fn write(&mut self, bands:&[Band], cursors:&[usize], queues:&[Vec<u32>], m:u32,
             mark:u32, bits:u32, first:u32, edges:&mut [u32]) {
        let pages=(m as usize).div_ceil(64);
        self.starts.clear();self.starts.resize(pages,0);self.prefix.resize(pages,0);
        self.partner.resize(bands.len(),Partner::default());
        let mut k=0u32;
        for b in bands {
            let d=k.wrapping_add(b.delta);
            self.starts[d as usize/64]|=1u64<<(d%64);k=b.end;
        }
        let mut count=0;
        for (p,&mask) in self.prefix.iter_mut().zip(&self.starts){*p=count;count+=mask.count_ones();}
        debug_assert_eq!(count as usize,bands.len());
        k=0;
        for a in 0..256 {
            let end=cursors.get(a+1).copied().unwrap_or(bands.len());let mut local=0u32;
            for b in &bands[cursors[a]..end] {
                let d=k.wrapping_add(b.delta);let w=b.end-k;let i=self.index(d);
                self.partner[i]=Partner {end:d+w,delta:local.wrapping_sub(d),code:b.code.min(128)};
                local+=w;k=b.end;
            }
        }
        for j in (1..self.partner.len()).rev(){
            let after=self.partner[j];let pred=&mut self.partner[j-1];
            if pred.code==after.code&&(pred.code==128||pred.delta==after.delta){pred.end=after.end;}
        }
        k=0;
        for a in 0..256 {
            let end=cursors.get(a+1).copied().unwrap_or(bands.len());let mut local=0;
            for b in &bands[cursors[a]..end]{
                let width=(b.end-k) as usize;let d=k.wrapping_add(b.delta) as usize;
                let src=&queues[a][local..local+width];let dst=&mut edges[d..d+width];let packed=b.code<<bits;
                if b.code>=128 {
                    for (target,&y) in dst.iter_mut().zip(src){*target=packed|y;}
                }else{
                    let mut at=0;
                    while at<width {
                        let partner=self.partner[self.containing(src[at])];
                        if partner.code<128 {
                            let pair=(first+b.code*128+partner.code)<<bits;
                            let second=&queues[partner.code as usize];
                            loop {
                                let y=src[at];
                                dst[at]=if y==mark{packed|y}else{pair|second[y.wrapping_add(partner.delta) as usize]};
                                at+=1;if at==width||src[at]>=partner.end{break;}
                            }
                        }else{
                            loop {dst[at]=packed|src[at];at+=1;if at==width||src[at]>=partner.end{break;}}
                        }
                    }
                }
                local+=width;k=b.end;
            }
        }
    }
}
