// Present only in the separately built, untimed structural checker.
impl Kernel {
    pub fn bench_state(
        &mut self, records:&[u16], n:usize, p:u32, limit:u32
    ) -> ([usize;15],u32,u32,&[u32],&[u32],&[u8],&[u32],Vec<(u32,u32,u32)>) {
        let (s,mark,bits)=self.construct::<true,true>(records,n as u32,p,limit);
        let counts=[s.open_nodes,s.steps,s.rules,s.n,s.runs,s.retained,s.thick,s.bands,s.words,s.copies,
                    s.traced,s.thick_area,s.unit_work,s.longest,s.table_bytes];
        (counts,mark,bits,&self.edges,&self.extra,&self.spelling,&self.offsets,
            self.bands.iter().map(|b|(b.end,b.delta,b.code)).collect())
    }
}
