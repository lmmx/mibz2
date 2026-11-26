#[path="probe.rs"] mod probe;
fn main() {
    // Branch a three-way copy off each exact initial capacity and occupancy.
    let mut cases=0;
    let mut grows=0;
    for cap in 0..=96 {
        for used in 0..=cap {
            for width in 0..=17 {
                let mut a=Vec::with_capacity(cap);
                let mut b=Vec::with_capacity(cap);
                let mut c=Vec::with_capacity(cap);
                for i in 0..used {a.push(i as u32);b.push(i as u32);c.push(i as u32);}
                let old=a.capacity();
                let m=899_980;
                probe::extend_rows(&mut a,m,width);
                probe::first_single(&mut b,m,width);
                probe::first_four(&mut c,m,width);
                assert_eq!(a,b);assert_eq!(a,c);
                assert_eq!(a.capacity(),b.capacity());assert_eq!(a.capacity(),c.capacity());
                grows+=usize::from(a.capacity()>old);
                cases+=1;
            }
        }
    }
    println!("{cases} states crossed initial capacity 0..96, every occupancy, and widths 0..17; {grows} forced growth. All contents and resulting capacities agreed on Rust 1.99.1.");
}
