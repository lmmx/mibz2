// Untimed high-water-capacity census while completed second state remains live.
impl Second {
    pub(super) fn capacity_bytes(&self)->usize {
        use super::space;
        space(&self.runs)+space(&self.count)+space(&self.base)+space(&self.epoch)+space(&self.child)
            +space(&self.page)+space(&self.masks)+space(&self.nodes)+space(&self.edges)+space(&self.selectors)
            +space(&self.words)+space(&self.dictionary)+space(&self.merged)+space(&self.links)
    }
}
