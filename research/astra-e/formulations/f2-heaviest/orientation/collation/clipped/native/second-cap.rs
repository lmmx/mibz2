impl Second{
    pub(super) fn capacity(&self)->usize{
        use super::bytes;
        bytes(&self.runs)+bytes(&self.count)+bytes(&self.base)+bytes(&self.epoch)+bytes(&self.child)
         +bytes(&self.page)+bytes(&self.masks)+bytes(&self.nodes)+bytes(&self.edges)+bytes(&self.selectors)
         +bytes(&self.words)+bytes(&self.dictionary)+bytes(&self.merged)+bytes(&self.links)
    }
}
