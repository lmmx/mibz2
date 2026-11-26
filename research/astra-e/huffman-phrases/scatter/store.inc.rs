                    // Slots beyond count will be overwritten or excluded from the final length.
                    // This macro is called only with four or more group positions available.
                    const MASKS:[u64;4]=[
                        511,511|(511<<16),511|(63<<16)|(63<<32),511|(15<<16)|(15<<32)|(15<<48)
                    ];
                    let mask=MASKS[(e>>8 & 3) as usize];
                    let packed=core::arch::x86_64::_pdep_u64((e>>FIRST)as u64,mask);
                    let dst=out[n..n+4].as_mut_ptr().cast::<u64>();
                    unsafe { std::ptr::write_unaligned(dst,packed); }
