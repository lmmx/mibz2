struct Fragment {
    U width,source,other;  // other changes from a byte-domain index to a class ID.
};
static_assert(sizeof(Fragment)==12);
std::vector<U> image_order(const std::vector<Band>& bands,Tally& t) {
    std::vector<U> order(bands.size()),scratch(bands.size());
    std::iota(order.begin(),order.end(),0);
    U top=0;for(const auto& b:bands)top=std::max(top,b.dst);
    const U bits=std::bit_width(top);
    for(U i=0;i<bits;i+=10){
        const U width=std::min(10u,bits-i),count=1u<<width,mask=count-1;
        std::array<U,1024> offset;std::fill_n(offset.begin(),count,0u);
        for(U b:order)++offset[(bands[b].dst>>i)&mask];
        U next=0;for(U j=0;j<count;++j){U freq=offset[j];offset[j]=next;next+=freq;}
        for(U b:order)scratch[offset[(bands[b].dst>>i)&mask]++]=b;
        order.swap(scratch);++t.passes;t.sort_items+=bands.size();t.radix_bins+=count;
    }
    return order;
}

Column publish(const std::vector<Band>& bands,const Descriptors& ds,Phase phase,Tally& t) {
    Column out;out.phase=phase;
    if(bands.empty()){assert(phase.origin==none);return out;}
    t.domains=ds.full.size();t.bytes=ds.byte.size();t.bands=bands.size();
    t.bound=t.bands+t.domains+t.bytes-2;
    auto order=image_order(bands,t);
    std::vector<U> slot(bands.size()+1);U di=0,image_end=0;
    for(U b:order) {
        U start=bands[b].dst;assert(start==image_end);image_end+=bands[b].len;
        while(ds.byte[di].start+ds.byte[di].width<=start)++di;
        slot[b]=di;
    }
    // Stage triples in original source order for one ordered replay at the end.
    std::vector<Fragment> q;
    U si=0;
    for(U b=0;b<bands.size();++b) {
        const auto band=bands[b];di=slot[b];slot[b]=q.size();
        for(U h=0;h<band.len;) {
            U x=band.src+h,y=band.dst+h;
            if(ds.full[si].start+ds.full[si].width==x)++si;
            if(ds.byte[di].start+ds.byte[di].width==y)++di;
            const auto& s=ds.full[si];const auto& d=ds.byte[di];
            U w=std::min({band.len-h,s.start+s.width-x,d.start+d.width-y});
            assert(w&&s.start<=x&&d.start<=y);q.push_back({w,si,di});
            if(phase.origin!=none&&y<=phase.origin&&phase.origin-y<w)
                out.phase.origin=ds.shifted(phase.origin,d);
            h+=w;
        }
    }
    slot.back()=q.size();t.pieces=q.size();assert(t.pieces<=t.bound);
    t.fragment_capacity=q.capacity();assert(t.fragment_capacity>=t.pieces);

    std::array<U,256> cursor=ds.base,prior;prior.fill(none);
    std::array<std::vector<U>,256> by_byte;
    for(U b:order) {
        auto band=bands[b];U y=band.dst;
        for(U z=slot[b];z<slot[b+1];++z) {
            auto& frag=q[z];const auto& s=ds.full[frag.source];const auto& d=ds.byte[frag.other];
            Key key{band.dist,s.context,d.b,s.b};
            assert(ds.shifted(y,d)==cursor[d.b]);y+=frag.width;
            U& id=prior[d.b];bool fresh=id==none;
            if(!fresh){++t.key_comparisons;fresh=out.classes[id].key!=key;}
            if(fresh) {
                id=out.classes.size();out.classes.push_back({key,s.post,cursor[d.b],0});by_byte[d.b].push_back(id);
            }
            auto& c=out.classes[id];assert(c.post==s.post);
            c.width+=frag.width;cursor[d.b]+=frag.width;frag.other=id;
        }
        assert(y==band.dst+band.len);
    }
    for(auto& ids:by_byte)out.order.insert(out.order.end(),ids.begin(),ids.end());
    U x=0;for(U i:out.order){assert(out.classes[i].base==x);x+=out.classes[i].width;}
    t.events=x;t.classes=out.classes.size();

    std::array<std::vector<Run>,256> incoming;
    for(auto f:q)append(incoming[ds.full[f.source].b],f.other,f.width);
    for(const auto& bin:incoming)for(auto r:bin)append(out.column,r.id,r.width);
    t.runs=out.column.size();
    out.select.resize(out.classes.size());std::vector<U> before(out.classes.size());x=0;
    for(auto [id,w]:out.column){out.select[id].push_back({before[id],x});before[id]+=w;x+=w;}
    for(U i=0;i<out.classes.size();++i)assert(before[i]==out.classes[i].width);
    return out;
}
