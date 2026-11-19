
#[derive(Default)]
struct DisplacementPages {
    header: [u32; 10],
    row_dir: Vec<u32>,
    row_pool: Vec<u8>,
    code_dir: Vec<u32>,
    code_pool: Vec<u8>,
}

#[derive(Clone, Copy, Default)]
struct DisplacementModel {
    kind: u8,
    width: u32,
    exceptions: usize,
    slope: i32,
    base: i32,
    bytes: usize,
}

#[inline]
fn width32(x: u32) -> u32 { u32::BITS - x.leading_zeros() }
#[inline]
fn packed_length(n: usize, w: u32) -> usize { (n*w as usize).div_ceil(8) }

fn pack_iter(pool: &mut Vec<u8>, values: impl Iterator<Item = u32>, w: u32) {
    let mut pending = 0u64;
    let mut count = 0u32;
    for value in values {
        pending |= u64::from(value) << count;
        count += w;
        while count >= 8 {
            pool.push(pending as u8);
            pending >>= 8;
            count -= 8;
        }
    }
    if count > 0 {pool.push(pending as u8);}
}

#[inline]
fn packed_at(pool: &[u8], start: usize, i: usize, w: u32) -> u32 {
    if w==0{return 0;}
    let bit = i*w as usize;
    let at = start+bit/8;
    let x = u64::from_le_bytes(pool[at..at+8].try_into().unwrap());
    ((x >> (bit%8)) & ((1u64 << w)-1)) as u32
}

#[inline]
fn get_payload_u32(pool: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(pool[at..at+4].try_into().unwrap())
}

fn choose_displacement(rows: &[u32], rb: u32) -> DisplacementModel {
    let q = rows.len();
    let mut best = DisplacementModel {bytes: 1+packed_length(q,rb), ..Default::default()};
    let mut slopes = [0,1,0,0];
    if q>1 {
        let mut diffs = [0i32;31];
        for (i,pair) in rows.windows(2).enumerate() {diffs[i]=pair[1] as i32-pair[0] as i32;}
        diffs[..q-1].sort_unstable();
        slopes[2] = ((rows[q-1] as i32-rows[0] as i32) as f64/(q-1) as f64).round() as i32;
        slopes[3] = diffs[(q-1)/2];
    }
    slopes.sort_unstable();
    for (si,&slope) in slopes.iter().enumerate() {
        if si>0 && slope==slopes[si-1] {continue;}
        let kind = if slope==0 {1} else if slope==1 {2} else {3};
        let mut ds=[0i32;32];
        for (i,&v) in rows.iter().enumerate(){ds[i]=v as i32-slope*i as i32;}
        ds[..q].sort_unstable();
        for k in 0..=8.min(q/4) {
            let overhead=6+usize::from(kind==3)*4+if k==0{0}else{q.div_ceil(8)}+packed_length(k,rb);
            for a in 0..=k {
                let w=width32((ds[a+q-k-1]-ds[a]) as u32);
                let bytes=overhead+packed_length(q-k,w);
                if bytes<best.bytes {best=DisplacementModel{kind,width:w,exceptions:k,slope,base:ds[a],bytes};}
            }
        }
    }
    best
}

fn encode_displacement(pool: &mut Vec<u8>, ys: &[u32], rb:u32, model:DisplacementModel) {
    let q=ys.len();
    if model.kind==0 {
        pool.push(0);pack_iter(pool,ys.iter().copied(),rb);return;
    }
    let (w,k)=(model.width,model.exceptions);
    let mut mask=0u32;
    let mut diffs=[0u32;32];
    for (i,&y) in ys.iter().enumerate(){
        let d=y as i32-model.base-model.slope*i as i32;
        diffs[i]=d as u32;
        if d<0 || d as u32 >= (1u32<<w) {mask|=1u32<<i;}
    }
    // Values tied at the ends of an interval can reduce the compulsory outlier count.
    let mut at=0;
    while (mask.count_ones() as usize)<k {mask|=1u32<<at;at+=1;}
    pool.push(model.kind | if k>0{4}else{0});
    pool.push(w as u8);
    pool.extend_from_slice(&model.base.to_le_bytes());
    if model.kind==3{pool.extend_from_slice(&model.slope.to_le_bytes());}
    if k!=0{pool.extend_from_slice(&mask.to_le_bytes()[..q.div_ceil(8)]);}
    pack_iter(pool,(0..q).filter(|&i|mask>>i&1==0).map(|i|diffs[i]),w);
    if k!=0{pack_iter(pool,(0..q).filter(|&i|mask>>i&1==1).map(|i|ys[i]),rb);}
}

impl DisplacementPages {
    fn encode(&mut self,edges:&[u32],extra:&[u32],rb:u32,n:usize,mark:u32) {
        let m=edges.len();
        let mask=(1u32<<rb)-1;
        let edge_code=|i:usize| if extra.is_empty(){edges[i]>>rb}else{extra[i]};
        let cb=width32((0..m).map(edge_code).max().unwrap());
        self.row_dir.clear();
        self.row_pool.clear();
        self.code_dir.clear();
        self.code_pool.clear();
        for batch in edges.chunks(32) {
            self.row_dir.push(self.row_pool.len() as u32);
            let mut values=[0u32;32];
            for (o,&edge) in values.iter_mut().zip(batch) {*o=edge&mask;}
            let ys=&values[..batch.len()];
            encode_displacement(&mut self.row_pool,ys,rb,choose_displacement(ys,rb));
        }
        let row_paged=self.row_dir.len()*4+self.row_pool.len()<packed_length(m,rb);
        if !row_paged {
            self.row_dir.clear();self.row_pool.clear();
            pack_iter(&mut self.row_pool,edges.iter().map(|e|e&mask),rb);
        }
        for start in (0..m).step_by(16) {
            let q=16.min(m-start);
            let mut unique=[0u32;16];let mut indices=[0u32;16];let mut nu=0;
            for (i,v) in indices[..q].iter_mut().enumerate(){
                let code=edge_code(start+i);
                let j=unique[..nu].iter().position(|x|*x==code).unwrap_or(nu);
                if j==nu {unique[nu]=code;nu+=1;}
                *v=j as u32;
            }
            if nu==1 {self.code_dir.push(unique[0]);continue;}
            self.code_dir.push(0x80000000|self.code_pool.len() as u32);
            let iw=width32(nu as u32-1);
            if packed_length(nu,cb)+packed_length(q,iw)<packed_length(q,cb) {
                self.code_pool.push(nu as u8);
                pack_iter(&mut self.code_pool,unique[..nu].iter().copied(),cb);
                pack_iter(&mut self.code_pool,indices[..q].iter().copied(),iw);
            }else{
                self.code_pool.push(0);
                pack_iter(&mut self.code_pool,(start..start+q).map(edge_code),cb);
            }
        }
        let code_paged=self.code_dir.len()*4+self.code_pool.len()<packed_length(m,cb);
        if !code_paged {
            self.code_dir.clear();self.code_pool.clear();
            pack_iter(&mut self.code_pool,(0..m).map(edge_code),cb);
        }
        self.header=[m as u32,mark,n as u32,rb,cb,32,row_paged as u32,code_paged as u32,
            (self.row_dir.len()*4+self.row_pool.len()) as u32,(self.code_dir.len()*4+self.code_pool.len()) as u32];
        // An unaligned u64 extraction can touch seven bytes beyond the last payload byte.
        // A zero-width field at the very end reads no data.
        self.row_pool.extend_from_slice(&[0;7]);
        self.code_pool.extend_from_slice(&[0;7]);
    }

    #[inline]
    fn next(&self,index:usize)->u32 {
        let rb=self.header[3];
        if self.header[6]==0{return packed_at(&self.row_pool,0,index,rb);}
        let p=index/32;let i=index%32;
        let q=32.min(self.header[0] as usize-p*32);
        let mut at=self.row_dir[p] as usize;
        let tag=self.row_pool[at];at+=1;
        if tag==0{return packed_at(&self.row_pool,at,i,rb);}
        let kind=tag&3;let w=u32::from(self.row_pool[at]);at+=1;
        let base=get_payload_u32(&self.row_pool,at) as i32;at+=4;
        let slope=if kind==3 {let s=get_payload_u32(&self.row_pool,at) as i32;at+=4;s}else{i32::from(kind==2)};
        let mut j=i;
        if tag&4 !=0 {
            let mask=get_payload_u32(&self.row_pool,at)&(u32::MAX>>(32-q));
            at+=q.div_ceil(8);
            let before=(mask&((1u32<<i)-1)).count_ones() as usize;
            if mask>>i&1 !=0 {
                return packed_at(&self.row_pool,at+packed_length(q-mask.count_ones() as usize,w),before,rb);
            }
            j-=before;
        }
        let d=if w==0{0}else{packed_at(&self.row_pool,at,j,w)};
        (base+slope*i as i32+d as i32) as u32
    }

    #[inline]
    fn code(&self,index:usize)->u32 {
        let cb=self.header[4];
        if self.header[7]==0 {return if cb==0{0}else{packed_at(&self.code_pool,0,index,cb)};}
        let h=self.code_dir[index/16];
        if h>>31==0{return h;}
        let at=(h&0x7fffffff) as usize;
        let nu=self.code_pool[at] as usize;
        if nu==0{return packed_at(&self.code_pool,at+1,index%16,cb);}
        let i=packed_at(&self.code_pool,at+1+packed_length(nu,cb),index%16,width32(nu as u32-1));
        packed_at(&self.code_pool,at+1,i as usize,cb)
    }

    fn emit(&self,spelling:&[u8],offsets:&[u32],out:&mut Vec<u8>){
        let n=self.header[2] as usize;
        let mark=self.header[1] as usize;
        out.clear();out.reserve(n);
        let mut row=mark;
        let mut next=self.next(row) as usize;
        loop {
            let code=self.code(row);row=next;next=self.next(row) as usize;
            append_code(code,spelling,offsets,out);
            if row==mark {break;}
        }
        repeat(n,out);
    }
}

impl Kernel {
    pub fn displacement_entry(&self,row:usize,bits:u32)->(u32,u32) {
        if self.edges.is_empty() {
            (self.packed_pages.next(row),self.packed_pages.code(row))
        } else {
            let e=self.edges[row];
            (e&((1u32<<bits)-1),if self.extra.is_empty(){e>>bits}else{self.extra[row]})
        }
    }

    pub fn displacement_storage(&self)->[usize;4] {
        let p=&self.packed_pages;
        [40+4*(self.edges.len()+self.extra.len()+p.row_dir.len()+p.code_dir.len())
             +p.row_pool.len()+p.code_pool.len(),
         std::mem::size_of::<DisplacementPages>()
             +4*(self.edges.capacity()+self.extra.capacity()+p.row_dir.capacity()+p.code_dir.capacity())
             +p.row_pool.capacity()+p.code_pool.capacity(),
         self.spelling.len()+4*self.offsets.len(),
         self.spelling.capacity()+4*self.offsets.capacity()]
    }
}
