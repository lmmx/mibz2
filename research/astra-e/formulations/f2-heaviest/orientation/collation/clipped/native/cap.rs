pub(super) fn bytes<T>(v:&Vec<T>)->usize{v.capacity()*std::mem::size_of::<T>()}
impl<PARAMS> TYPE{
    pub fn whole_capacity(&self)->usize{
        bytes(&self.runs)+bytes(&self.starts)+bytes(&self.page_first)+bytes(&self.nodes)+bytes(&self.selectors)
            +bytes(&self.edges)+bytes(&self.words)+bytes(&self.dict)+bytes(&self.merged)+bytes(&self.links)
            +self.lane_paths.iter().map(bytes).sum::<usize>()+self.second.capacity()+EXTRA
    }
}
