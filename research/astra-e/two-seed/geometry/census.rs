use std::fs::File;
use std::io::{BufRead,BufReader,Read,Seek,SeekFrom};

fn read32(f:&mut impl Read)->u32{let mut v=[0;4];f.read_exact(&mut v).unwrap();u32::from_le_bytes(v)}
fn bytes(f:&mut impl Read,n:usize)->Vec<u8>{let mut data=vec![0;n];f.read_exact(&mut data).unwrap();data}
fn repeat(out:&mut Vec<u8>,n:usize){
    assert!(!out.is_empty() && out.len()<=n);
    while out.len()<n {let copy=out.len().min(n-out.len());out.extend_from_within(..copy);}
}
struct Table {edges:Vec<u32>,bits:u32,dict:Vec<Vec<u8>>}
impl Table {
    fn next(&self,e:u32)->usize {(e&((1<<self.bits)-1))as usize}
    fn code(&self,e:u32)->u32{e>>self.bits}
    fn length(&self,c:u32)->usize{if c<256{1}else{self.dict[(c-256)as usize].len()}}
    fn emit(&self,c:u32,out:&mut Vec<u8>){
        if c<256{out.push(c as u8);}else{out.extend_from_slice(&self.dict[(c-256)as usize]);}
    }
    fn arc(&self,start:usize,other:usize)->(Vec<usize>,bool){
        let mut order=Vec::new();let mut r=start;
        loop{
            order.push(r);r=self.next(self.edges[r]);
            if r==other {return(order,true);}
            if r==start {return(order,false);}
            assert!(order.len()<self.edges.len());
        }
    }
    fn payload(&self,order:&[usize],out:&mut Vec<u8>)->(usize,usize){
        out.clear();
        let(mut words,mut spelled)=(0,0);
        for &i in order {
            let c=self.code(self.edges[i]);self.emit(c,out);
            if c>=256{words+=1;spelled+=self.length(c);}
        }
        (words,spelled)
    }
}
#[derive(Default,Debug)]
struct Measured {
    relation:&'static str,primary_hops:usize,secondary_hops:usize,paired:usize,ideal:usize,critical:usize,
    primary_words:usize,secondary_words:usize,primary_word_bytes:usize,secondary_word_bytes:usize,
    primary_bytes:usize,secondary_bytes:usize,joined:usize,discarded:usize,repeat:usize,
    paired_both_words:usize,paired_min_bytes:usize,same_line:usize,same_page:usize,
    fresh_cap:usize,reuse_cap:usize
}
fn measure(t:&Table,mark:usize,seed:Option<usize>,n:usize,
           canonical:&[usize],want:&[u8],reused:&mut Vec<u8>)->Measured{
    let rows=t.edges.len();let valid=seed.is_some_and(|s|s<rows && s!=mark);
    let (mut first,mut second);
    let mut z=Measured::default();
    if valid {
        let s=seed.unwrap();
        let (a,ac)=t.arc(mark,s);let (b,bc)=t.arc(s,mark);
        assert_eq!(ac,bc);first=a;second=b;
        if ac {
            z.relation="same";z.critical=first.len().max(second.len());
            assert_eq!(canonical.len(),first.len()+second.len());
            assert_eq!(canonical[..first.len()],first);
            assert_eq!(canonical[first.len()..],second);
        }else{
            z.relation="foreign";
            // The first closure proves the seeds are disjoint. Once the primary closes,
            // further secondary reads can be cancelled, even if its cycle is longer.
            second.truncate(first.len());
            z.critical=first.len();
            assert_eq!(canonical,first);
        }
    }else{
        z.relation=if rows==1{"single"}else{"invalid_or_equal"};
        first=canonical.to_vec();second=Vec::new();z.critical=first.len();
    }
    z.primary_hops=first.len();z.secondary_hops=second.len();
    z.ideal=canonical.len()/2;z.paired=first.len().min(second.len());
    for (&a,&b) in first.iter().zip(&second){
        let (ca,cb)=(t.code(t.edges[a]),t.code(t.edges[b]));
        z.paired_both_words+=usize::from(ca>=256 && cb>=256);
        z.paired_min_bytes+=t.length(ca).min(t.length(cb));
        z.same_line+=usize::from(a/16==b/16);
        z.same_page+=usize::from(a/1024==b/1024);
    }
    let mut out=Vec::with_capacity(n);
    (z.primary_words,z.primary_word_bytes)=t.payload(&first,&mut out);z.primary_bytes=out.len();
    (z.secondary_words,z.secondary_word_bytes)=t.payload(&second,reused);z.secondary_bytes=reused.len();
    let mut fresh=Vec::new();t.payload(&second,&mut fresh);assert_eq!(fresh,*reused);
    z.fresh_cap=fresh.capacity();z.reuse_cap=reused.capacity();
    if z.relation=="same"{out.extend_from_slice(reused);z.joined=reused.len();}
    if z.relation=="foreign"{z.discarded=reused.len();}
    z.repeat=n-out.len();repeat(&mut out,n);assert_eq!(out,want);
    // Do not let incidental per-function temporaries become part of a worker-state estimate.
    first.clear();
    z
}
fn un_rle(coded:&[u8])->Vec<u8>{
    let mut out=Vec::new();let mut at=0;let(mut prev,mut count)=(256,0);
    while at<coded.len(){
        let v=coded[at];at+=1;out.push(v);
        if prev==v as usize{count+=1;}else{prev=v as usize;count=1;}
        if count==4{let extra=coded[at]as usize;at+=1;out.resize(out.len()+extra,v);prev=256;count=0;}
    }
    out
}
fn oracle(t:&Table,mark:usize,n:usize)->(Vec<usize>,Vec<u8>){
    let (cycle,_)=t.arc(mark,mark);
    let mut want=Vec::new();t.payload(&cycle,&mut want);repeat(&mut want,n);
    (cycle,want)
}
fn seeds(mark:usize,m:usize)->[Option<usize>;4]{
    [if m<=1{None}else{Some((mark+(m/2).max(1))%m)},Some(mark^1),Some(0),Some(m/2)]
}
fn small(){
    fn permute(a:&mut [usize],i:usize,visit:&mut impl FnMut(&[usize])){
        if i==a.len(){visit(a);return;}
        for j in i..a.len(){a.swap(i,j);permute(a,i+1,visit);a.swap(i,j);}
    }
    let(mut maps,mut cases,mut foreign,mut collision,mut periodic,mut truncated)=(0u64,0,0,0,0,0);
    for m in 1..=7{
        let mut perm:Vec<_>=(0..m).collect();
        permute(&mut perm,0,&mut |next|{
            maps+=1;let bits=(usize::BITS-(m-1).leading_zeros()).max(1);
            let codes:Vec<_>=(0..m).map(|r|[b'x' as u32,256,257,b'y' as u32][r%4]).collect();
            let t=Table{edges:next.iter().zip(&codes).map(|(&k,&c)|k as u32|(c<<bits)).collect(),
                        bits,dict:vec![b"pq".to_vec(),b"1234567".to_vec()]};
            let mut buffer=Vec::new();
            for a in 0..m{
                let (cy,_)=t.arc(a,a);let emit=cy.iter().map(|&r|t.length(codes[r])).sum::<usize>();
                for n in [emit,2*emit,2*emit+1]{
                    let (cy,original)=oracle(&t,a,n);
                    for s in 0..=m{
                        let z=measure(&t,a,Some(s),n,&cy,&original,&mut buffer);
                        cases+=1;foreign+=u64::from(z.relation=="foreign");
                        collision+=u64::from(z.relation=="invalid_or_equal" || z.relation=="single");
                        periodic+=u64::from(z.repeat!=0);
                        truncated+=u64::from(z.repeat!=0 && n%emit!=0);
                    }
                }
            }
        });
    }
    println!("{maps} permutations, {cases} seed/mark/budget cases, {foreign} foreign-cycle pairs, {collision} equal/out-of-range/singleton cases, {periodic} repeated and {truncated} partial-cycle truncations");
}
fn main(){
    let args:Vec<_>=std::env::args().collect();
    if args[1]=="--small"{small();return;}
    let mut edges=BufReader::new(File::open(&args[1]).unwrap());
    let mut words=BufReader::new(File::open(&args[2]).unwrap());
    let routes=std::fs::read_to_string(&args[3]).unwrap();let mut spans=Vec::new();let mut offset=0;
    for line in routes.lines().skip(1){
        let f:Vec<_>=line.split(',').collect();assert_eq!(f[0].parse::<usize>().unwrap(),spans.len());
        let size=f[5].parse::<usize>().unwrap();spans.push((offset,size));offset+=size;
    }
    let mut lib=File::open(&args[4]).unwrap();assert_eq!(offset,lib.metadata().unwrap().len()as usize);
    let mut reusable:[Vec<u8>;4]=Default::default();
    let(mut blocks,mut total,mut final_bytes,mut rows)=(0,0,0,0);
    println!("block,stream,n,m,mark,method,seed,relation,primary_hops,secondary_hops,paired_rounds,ideal_paired_rounds,critical_rounds,primary_words,secondary_words,primary_spelling_bytes,secondary_spelling_bytes,primary_bytes,secondary_bytes,join_copy_bytes,discarded_bytes,repeat_bytes,both_nonliteral_rounds,paired_shorter_payload_sum,same_edge_line_rounds,same_edge_page_rounds,fresh_secondary_capacity,persistent_secondary_capacity");
    while !edges.fill_buf().unwrap().is_empty(){
        let h:[u32;8]=std::array::from_fn(|_|read32(&mut edges));
        let id=h[0] as usize;let stream=h[1];let n=h[2]as usize;let m=h[3]as usize;let mark=h[4]as usize;
        let links=(0..m).map(|_|read32(&mut edges)).collect();
        for _ in 0..h[7]{let _:[u32;4]=std::array::from_fn(|_|read32(&mut edges));}
        let g:[u32;6]=std::array::from_fn(|_|read32(&mut words));
        assert_eq!(g,[h[0],h[1],h[2],h[3],4*h[3],h[6]]);
        let mut freq=Vec::new();let mut dict=Vec::new();
        for _ in 0..g[5]{
            let _handle=read32(&mut words);freq.push(read32(&mut words));let len=read32(&mut words);
            dict.push(bytes(&mut words,len as usize));
        }
        let t=Table{edges:links,bits:h[5],dict};
        let(cycle,want)=oracle(&t,mark,n);let mut counts=vec![0u32;freq.len()];
        for &i in &cycle{let c=t.code(t.edges[i]);if c>=256{counts[c as usize-256]+=1;}}
        assert_eq!(counts,freq);
        let checked=un_rle(&want);let (at,size)=spans[id];assert_eq!(checked.len(),size);
        lib.seek(SeekFrom::Start(at as u64)).unwrap();assert_eq!(bytes(&mut lib,size),checked);
        for (j,seed) in seeds(mark,m).into_iter().enumerate(){
            let z=measure(&t,mark,seed,n,&cycle,&want,&mut reusable[j]);
            let method=["half_shift","xor1","zero","midpoint"][j];
            println!("{id},{stream},{n},{m},{mark},{method},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                     seed.map_or(-1,|s|s as i64),z.relation,z.primary_hops,z.secondary_hops,z.paired,z.ideal,z.critical,
                     z.primary_words,z.secondary_words,z.primary_word_bytes,z.secondary_word_bytes,z.primary_bytes,
                     z.secondary_bytes,z.joined,z.discarded,z.repeat,z.paired_both_words,z.paired_min_bytes,z.same_line,
                     z.same_page,z.fresh_cap,z.reuse_cap);
        }
        blocks+=1;total+=n;final_bytes+=size;rows+=m;
    }
    assert!(words.fill_buf().unwrap().is_empty());
    assert_eq!((blocks,total,final_bytes,rows),(233,209697662,209691304,26567581));
    eprintln!("{blocks} admissions, {total} coded and {final_bytes} libbzip2 output bytes, {rows} original packed edges; all four marked schedules and every word frequency checked");
}
