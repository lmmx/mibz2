use mibz2::{
    runs,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
fn main() {
    let input = std::fs::read(std::env::args_os().nth(1).unwrap()).unwrap();
    let mut scanner = Scanner::new(SliceSource(&input));
    let mut reuse = Vec::new();
    let mut records = runs::Runs::default();
    println!(
        "block,first_bit,end_bit,eob,max_symbol,syms,rows,takes_runs,would_narrow,stock_capacity"
    );
    while let Event::Block {
        index,
        bit,
        end,
        symbols,
        ..
    } = scanner.next(reuse).unwrap()
    {
        let n = runs::front(&symbols, &mut records, Kernels::detect()).unwrap();
        let max = symbols.syms.iter().copied().max().unwrap();
        assert_eq!(Some(&symbols.eob), symbols.syms.last());
        assert_eq!(max, symbols.eob);
        println!(
            "{index},{bit},{end},{},{max},{},{n},{},{},{}",
            symbols.eob,
            symbols.syms.len(),
            runs::takes_runs(&symbols),
            symbols.eob <= 255,
            symbols.syms.capacity()
        );
        reuse = symbols.syms;
    }
}
