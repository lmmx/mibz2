// Append within the frozen Compact/preload module to access its normal raw-input path.
#[repr(C)]
pub struct Owned { data: *mut u8, len: usize, cap: usize }
#[no_mangle]
pub unsafe extern "C" fn compact_inverse(caps:*const u16,caplen:usize,counts:*const u32,n:u32,p:u32)->Owned {
    let caps=unsafe { std::slice::from_raw_parts(caps,caplen) };
    let freq=unsafe { &*counts.cast::<[u32;256]>() };
    let mut c=Dispatch::<2,2>::default();
    let mut out=Vec::new();
    assert!(c.attempt_with::<true>(caps,n as usize,p,&mut out,Some(freq)));
    let h=Owned{data:out.as_mut_ptr(),len:out.len(),cap:out.capacity()};
    std::mem::forget(out);
    h
}
#[no_mangle]
pub unsafe extern "C" fn compact_release(h:Owned){
    drop(unsafe { Vec::from_raw_parts(h.data,h.len,h.cap) });
}
