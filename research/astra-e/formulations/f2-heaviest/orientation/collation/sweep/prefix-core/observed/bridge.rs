// Appended inside the pinned native Compact module, where its object fields are visible.
#[repr(C)] pub struct WordView {
    base:u32,prefix_len:u32,core_len:u32,prefix:*const u8,core:*const u8,
}
#[repr(C)] pub struct RunView {id:i32,width:i32}
#[repr(C)] pub struct Output {ptr:*const u8,len:usize,second:u32,capacity:usize}
#[derive(Default)] pub struct Handle {native:Dispatch<2,2>,out:Vec<u8>}
#[no_mangle] pub extern "C" fn core_handle()->*mut Handle{Box::into_raw(Box::new(Handle::default()))}
#[no_mangle] pub unsafe extern "C" fn core_drop(h:*mut Handle){drop(unsafe{Box::from_raw(h)});}
#[no_mangle] pub unsafe extern "C" fn core_materialize(h:&mut Handle,words:*const WordView,k:usize,root:*const RunView,nr:usize){
    let c=&mut h.native;let words=unsafe{std::slice::from_raw_parts(words,k)};
    let root=unsafe{std::slice::from_raw_parts(root,nr)};
    c.words.clear();c.dict.clear();c.nodes.clear();c.merged.clear();
    c.nodes.reserve(k);c.words.reserve(k);c.merged.reserve(nr);
    for (i,w) in words.iter().enumerate(){
        let offset=c.dict.len() as u32;
        let a=unsafe{std::slice::from_raw_parts(w.prefix,w.prefix_len as usize)};
        let b=unsafe{std::slice::from_raw_parts(w.core,w.core_len as usize)};
        c.dict.extend_from_slice(a);c.dict.extend_from_slice(b);
        c.words.push(Piece{id:offset,width:w.prefix_len+w.core_len});
        let mut n=Node::new(0,NONE,0);n.base=w.base;n.word=i as u32;c.nodes.push(n);
    }
    c.nodes[0].out=0;
    c.merged.extend(root.iter().map(|v|Piece{id:v.id as u32,width:v.width as u32}));
}
fn output(h:&Handle,second:bool)->Output{Output{ptr:h.out.as_ptr(),len:h.out.len(),second:second.into(),
                                                capacity:h.out.capacity()+h.native.whole_capacity()}}
#[no_mangle] pub extern "C" fn core_finish(h:&mut Handle,mark:u32,skip:u32,m:u32,n:u32)->Output{
    let c=&mut h.native;
    let second=c.second.invert(&c.merged,&c.nodes,&c.words,&c.dict,mark,skip,m,n,&mut h.out);
    if !second && !c.expand(mark,skip,m,n,&mut h.out){
        let mut links=vec![(0u32,0u32);m as usize];
        let mut cursor:Vec<_>=c.nodes.iter().map(|v|v.base).collect();let mut dest=0;
        for p in &c.merged{
            let x=&mut cursor[p.id as usize];
            for l in &mut links[*x as usize..(*x+p.width) as usize]{*l=(p.id,dest);dest+=1;}
            *x+=p.width;
        }
        let(mut x,mut off)=(mark,skip);h.out.clear();
        while h.out.len()<n as usize{
            let (id,z)=links[x as usize];let w=c.words[id as usize];let pos=w.id+off;
            let take=(w.width-off).min(n-h.out.len() as u32);
            h.out.extend_from_slice(&c.dict[pos as usize..(pos+take) as usize]);x=z;off=0;
        }
    }
    output(h,second)
}
#[no_mangle] pub unsafe extern "C" fn core_compact(h:&mut Handle,caps:*const u16,ncaps:usize,freq:*const u32,
                                                  n:u32,p:u32,a:u32,b:u32,symbol:u8)->Output{
    let raw=unsafe{std::slice::from_raw_parts(caps,ncaps)};let counts=unsafe{&*freq.cast::<[u32;256]>()};
    let (mark,m)=h.native.first::<true>(raw,counts,n,p,(a,b),symbol).expect("valid one-byte starting bucket");
    let second=h.native.finish(mark,m,n,p,&mut h.out);output(h,second)
}
