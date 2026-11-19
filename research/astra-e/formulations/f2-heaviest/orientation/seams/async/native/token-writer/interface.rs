// Appended inside the frozen Compact module. Piece has its matching C repr.
#[repr(C)]
pub struct Owned { data:*mut u8, len:usize, cap:usize }
fn own(mut x:Vec<u8>)->Owned{
    let o=Owned{data:x.as_mut_ptr(),len:x.len(),cap:x.capacity()};
    std::mem::forget(x);o
}
#[no_mangle]
pub unsafe extern "C" fn release(o:Owned){drop(unsafe{Vec::from_raw_parts(o.data,o.len,o.cap)});}
struct Inputs<'a> {caps:&'a [u16],freq:&'a [u32;256]}
unsafe fn raw<'a>(caps:*const u16,count:usize,freq:*const u32)->Inputs<'a>{
    Inputs{caps:unsafe{std::slice::from_raw_parts(caps,count)},freq:unsafe{&*freq.cast::<[u32;256]>()}}
}
#[no_mangle]
pub unsafe extern "C" fn full(caps:*const u16,count:usize,freq:*const u32,n:u32,p:u32)->Owned{
    let i=unsafe{raw(caps,count,freq)};let mut out=Vec::new();
    assert!(Dispatch::<2,2>::default().attempt_with::<true>(i.caps,n as usize,p,&mut out,Some(i.freq)));
    own(out)
}
#[repr(C)] pub struct Object {
    root:*const Piece, r:usize, words:*const Piece, bases:*const u32, k:usize,
    dict:*const u8, bytes:usize, mark:u32, skip:u32, tokens:u32, n:u32,
}
struct View<'a>{ root:&'a [Piece], words:&'a [Piece],bases:&'a[u32],dict:&'a[u8] }
impl Object{
    unsafe fn view(&self)->View<'_>{
        View{root:unsafe{std::slice::from_raw_parts(self.root,self.r)},
             words:unsafe{std::slice::from_raw_parts(self.words,self.k)},
             bases:unsafe{std::slice::from_raw_parts(self.bases,self.k)},
             dict:unsafe{std::slice::from_raw_parts(self.dict,self.bytes)}}
    }
}
#[no_mangle]
pub unsafe extern "C" fn consume(o:&Object,used_second:&mut u32)->Owned{
    let v=unsafe{o.view()};
    let nodes:Vec<Node>=v.bases.iter().enumerate().map(|(i,&base)|{
        let mut node=Node::new(0,NONE,0);node.word=i as u32;node.base=base;node
    }).collect();
    let mut out=Vec::new();
    *used_second=u32::from(second::Second::default().invert(v.root,&nodes,v.words,v.dict,o.mark,o.skip,o.tokens,o.n,&mut out));
    if *used_second==0 {
        let mut links=vec![(0u32,0u32);o.tokens as usize];
        let mut next=v.bases.to_vec();
        let mut dest=0;
        for p in v.root{
            let pos=&mut next[p.id as usize];
            for slot in &mut links[*pos as usize..(*pos+p.width) as usize]{
                *slot=(p.id,dest);dest+=1;
            }
            *pos+=p.width;
        }
        let (mut row,mut skip)=(o.mark,o.skip);
        out.reserve(o.n as usize);
        while out.len()<o.n as usize{
            let (word,to)=links[row as usize];let w=v.words[word as usize];
            let take=(w.width-skip).min(o.n-out.len() as u32);
            out.extend_from_slice(&v.dict[(w.id+skip) as usize..(w.id+skip+take) as usize]);
            skip=0;row=to;
        }
    }
    own(out)
}
// Verify the complete marked interchange up to ID renaming by source base.
#[no_mangle]
pub unsafe extern "C" fn compare(o:&Object,caps:*const u16,count:usize,freq:*const u32,p:u32){
    let v=unsafe{o.view()};let i=unsafe{raw(caps,count,freq)};
    let mut c=Dispatch::<2,2>::default();
    let (lo,m)=c.prepare_counted(i.caps,o.n,i.freq).unwrap();
    assert_eq!(m,o.tokens);
    assert_eq!(c.transfer_mark(p,lo),Some((o.mark,o.skip)));
    c.build(lo,m);c.inverse_merge();
    assert_eq!(v.words.len(),c.words.len());
    let mut id_for_base=std::collections::HashMap::new();
    for (id,&base) in v.bases.iter().enumerate(){assert!(id_for_base.insert(base,id).is_none());}
    for node in &c.nodes{
        if node.word==NONE{continue;}
        let w=c.words[node.word as usize];let peer=v.words[id_for_base[&node.base]];
        assert_eq!(&c.dict[w.id as usize..(w.id+w.width) as usize],
                   &v.dict[peer.id as usize..(peer.id+peer.width) as usize]);
    }
    let seq=&c.merged[c.nodes[0].out as usize..];
    assert_eq!(v.root.len(),seq.len());
    for (original,actual) in seq.iter().zip(v.root){
        assert_eq!(id_for_base[&c.nodes[original.id as usize].base],actual.id as usize);
        assert_eq!(original.width,actual.width);
    }
}
