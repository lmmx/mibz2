const ARMS:[&str;6]=["stock","stock twin","joined","joined twin","sliced","sliced twin"];
#[derive(Default)]
struct Buffers {raw:Vec<u8>,dst:Vec<u8>}
fn call(t:&Table,w:&mut Buffers,k:Kernels,arm:usize)->Result<u32,mibz2::ErrorKind>{
    w.dst.clear();
    let mut audit=slices::Audit::default();
    match arm/2 {
        0=>{multi::emit::<4,false>(t,&mut w.raw);mibz2::rle::unrle(&w.raw,&mut w.dst,k)?;},
        1=>expanded::expand::<4,false,false>(t,&mut w.raw,&mut w.dst,k,&mut audit)?,
        2=>expanded::expand::<4,true,false>(t,&mut w.raw,&mut w.dst,k,&mut audit)?,
        _=>unreachable!(),
    }
    Ok(mibz2::crc::crc32(&w.dst,k))
}
fn fixture() {
    let k=Kernels::detect();
    let mut cases=0;
    let mut errors=0;
    let mut certified=0;
    let mut bridged=0;
    let mut invalid_crc=0;
    for n in 4..=15 {
        for mask in 0..1u32<<n {
            let data:Vec<u8>=(0..n).map(|i|if mask&(1<<i)==0{0}else{19}).collect();
            let mut state=mask.wrapping_mul(1664525).wrapping_add(1013904223);
            for j in 0..4 {
                let a=1+state as usize%(n-3);
                state=state.rotate_left(7).wrapping_add(17);
                let b=a+1+state as usize%(n-a-2);
                let c=b+1+(state as usize>>9)%(n-b-1);
                let raw=[&data[..a],&data[a..b],&data[b..c],&data[c..]];
                let mut spans=[slices::Span::default();4];let mut physical=Vec::new();
                for i in [2,0,3,1] {
                    spans[i]=slices::Span{start:physical.len(),len:raw[i].len()};physical.extend(raw[i]);
                }
                let mut want=vec![42,255];let expected=mibz2::rle::unrle(&data,&mut want,k);
                let stored=mibz2::crc::crc32(&want[2..],k) ^ (j==0) as u32;
                let mut out=vec![42,255];let mut audit=slices::Audit::default();
                let observed=slices::parse::<4,true>(&physical,&spans,&mut out,k,&mut audit);
                if observed.is_none() {assert_eq!(out,[42,255]);}
                let observed=observed.unwrap_or_else(||mibz2::rle::unrle(&data,&mut out,k));
                assert_eq!(observed,expected);
                if observed.is_err() {errors+=1;}
                if observed.is_ok() && stored!=mibz2::crc::crc32(&out[2..],k){invalid_crc+=1;}
                if observed.is_err() || stored!=mibz2::crc::crc32(&out[2..],k){out.truncate(2);want.truncate(2);}
                assert_eq!(out,want);
                cases+=1;certified+=audit.accepted as usize;bridged+=(audit.accepted && audit.joined>0)as usize;
            }
        }
    }
    assert!(errors>0 && invalid_crc>0 && bridged>0 && certified<cases);
    println!("{cases} exact sliced/fallback differential cases; {certified} certified, {bridged} bridged, {errors} RunUnfinished, {invalid_crc} deliberately rejected CRCs, nonempty output rollback matched");
}
fn main(){
    let args=std::env::args().collect::<Vec<_>>();if args[1]=="synthetic"{return fixture();}
    let gate=args[1]=="gate";assert!(gate||args[1]=="price");
    let data=std::fs::read(Path::new(&args[2])).unwrap();
    let oracle=std::fs::read(Path::new(&args[3])).unwrap();
    let k=Kernels::detect();let mut scanner=Scanner::new(SliceSource(&data));
    let mut builder=original::Kernel::default();let mut runs=runs::Runs::default();
    let mut base=mibz2::Scratch::<Segmented>::default();let mut gold=Vec::new();let mut offset=0;
    let mut buffers:[Buffers;6]=std::array::from_fn(|_|Buffers::default());let mut samples=Vec::new();
    let(mut admissions,mut bytes)=(0,0);
    if gate{println!("block,n,expanded,crc,accepted,direct,joined,bridge_bytes,avoided,reason,base_raw,base_dst,sliced_raw,sliced_dst");}
    while let Event::Block{index,symbols,..}=scanner.next(Vec::new()).unwrap() {
        gold.clear();
        assert_eq!(mibz2::decode_block(&symbols,k,&mut base,&mut gold).unwrap(),symbols.crc);
        let expected=offset..offset+gold.len();offset=expected.end;assert_eq!(gold,oracle[expected.clone()]);
        if (!gate && !IDS.contains(&index)) || !runs::takes_runs(&symbols) {continue;}
        let n=runs::front(&symbols,&mut runs,k).unwrap();
        if runs.records().len()*20>=3*n && (n<65536 || !fits(runs.records(),symbols.orig_ptr)) {continue;}
        let t=builder.make(runs.records(),n,symbols.orig_ptr);
        if gate {
            admissions+=1;bytes+=n;
            for (a,b) in buffers.iter_mut().enumerate(){
                assert_eq!(call(&t,b,k,a).unwrap(),symbols.crc);assert_eq!(b.dst,gold);
            }
            let mut audit=slices::Audit::default();let b=&mut buffers[4];b.dst.clear();
            expanded::expand::<4,true,true>(&t,&mut b.raw,&mut b.dst,k,&mut audit).unwrap();
            assert_eq!(b.dst,gold);
            println!("{index},{n},{},{},{},{},{},{},{},{},{},{},{},{}",gold.len(),symbols.crc,audit.accepted,
                     audit.direct,audit.joined,audit.bridge_bytes,audit.avoided,audit.fallback,buffers[0].raw.capacity(),
                     buffers[0].dst.capacity(),buffers[4].raw.capacity(),buffers[4].dst.capacity());
        } else {samples.push((index,t,symbols.crc,expected));}
    }
    assert_eq!(offset,oracle.len());
    if gate {assert_eq!((admissions,bytes),(233,209697662));
        eprintln!("{admissions} real admission outputs and block CRCs checked, {offset} bytes compared with libbzip2 over all 431 blocks");
        return;
    }
    assert_eq!(samples.len(),14);println!("round,block,position,arm,cpu_ns");
    for r in 0usize..=36 {
        for off in 0..samples.len(){
            let j=(r*5+if r%2==0{off}else{samples.len()-off-1})%samples.len();
            let(id,t,crc,expected)=&samples[j];
            for pos in 0..6 {
                let shift=(r.saturating_sub(1)+j)%6;
                let arm=(shift+if (r.saturating_sub(1)/6+j)%2==0{pos}else{6-pos})%6;
                let start=cpu();let checked=call(t,&mut buffers[arm],k,arm);let time=cpu()-start;
                assert_eq!(checked,Ok(*crc));assert_eq!(buffers[arm].dst,oracle[expected.clone()]);
                if r>0{println!("{r},{id},{pos},{},{time}",ARMS[arm]);}
            }
        }
    }
}
