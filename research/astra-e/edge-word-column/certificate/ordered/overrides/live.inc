static std::ofstream live_csv;
struct LiveBucket {
    const std::vector<V>& q;std::vector<ModalBin> bins;
    std::priority_queue<Elem,std::vector<Elem>,Cmp> heap;
    MergeCost& cost;Exception current{};bool ready=false,done=false;
    LiveBucket(const std::vector<V>& qq,std::vector<ModalBin>&& source,MergeCost& c):
        q(qq),bins(std::move(source)),heap(Cmp{&c}),cost(c){
        for(u k=0;k<bins.size();k++){auto& b=bins[k];b.pos=b.slices[0].lo;
            heap.push({q[b.byte][b.pos],k});cost.reads++;}
    }
    void ensure(u x){
        if(done||(ready&&x<current.hi))return;
        if(heap.empty()){done=true;ready=false;return;}
        auto head=heap.top();heap.pop();auto& b=bins[head.idx];auto s=b.slices[b.si];
        u hi=head.pos+1;++b.pos;
        while(b.pos<s.hi){
            u y=q[b.byte][b.pos];cost.reads++;
            if(y!=hi){heap.push({y,head.idx});break;}
            hi++;b.pos++;
        }
        if(b.pos==s.hi&&++b.si<b.slices.size()){
            b.pos=b.slices[b.si].lo;heap.push({q[b.byte][b.pos],head.idx});cost.reads++;
        }
        assert(!ready||current.hi<=head.pos);cost.runs++;current={head.pos,hi,s.tag};ready=true;
    }
};
void live_check(const RootInput& a,const V& base,const V& color,const V& surv,const V& uniform,const V& widths,
    const std::vector<Frag>& fs,const V& first,const V& stop,const V& start,
    const std::vector<std::vector<Span>>& outer,const std::vector<Span>& expected,u block,bool print){
    u K=base.size()-1;V mode(K,NO);std::vector<std::vector<ModalBin>> slices(K);
    auto tag=[&](const Frag& f){return color[f.i]?surv[f.j]:f.a;};
    for(auto f:fs)if(uniform[f.j]==PULL){
        u t=tag(f),old=mode[f.j];if(old==NO||widths[t]>widths[old]||(widths[t]==widths[old]&&t<old))mode[f.j]=t;
    }
    U mass=0,ranges=0,dirs=0;
    for(u h=0;h<a.bands.size();h++){
        u c=a.bands[h].byte,pos=start[h];
        for(u it=first[h];it<stop[h];it++){
            auto f=fs[it];if(mode[f.j]!=NO&&tag(f)!=mode[f.j]){
                u t=tag(f);auto& bins=slices[f.j];
                if(bins.empty()||bins.back().byte!=c){bins.push_back({c,{},0,0});dirs++;}
                auto& v=bins.back().slices;
                if(!v.empty()&&v.back().hi==pos&&v.back().tag==t)v.back().hi+=f.w;
                else{v.push_back({pos,pos+f.w,t});ranges++;}
                mass+=f.w;
            }
            pos+=f.w;
        }
    }
    std::vector<std::unique_ptr<LiveBucket>> streams(K);
    V x=base;MergeCost cost;RawCaps raw;std::array<Cur,256> cursors{};
    U outer_claims=0,inner_claims=0,default_claims=0,peak_buckets=0,peak_heads=0;
    u open=0,heads=0,pos=0,run_width=0,run_label=NO,emitted=0;
    // The external consumer can consume each complete run immediately. Here it
    // compares against the previous exact answer. One pending run is sufficient.
    auto close=[&](){if(run_width){assert(emitted<expected.size()&&expected[emitted].label==run_label&&expected[emitted].width==run_width);emitted++;run_width=0;}};
    auto emit=[&](u id,u w){if(run_width&&id!=run_label)close();run_label=id;run_width+=w;};
    while(raw.ensure()){
        u c=raw.byte;auto& cur=cursors[c];u w=std::min(raw.left,cur.left(outer[c])),op=cur.id(outer[c]);outer_claims++;
        if(op!=NO&&(op&PULL)){
            u j=op&~PULL,left=w;
            if(!streams[j]){
                assert(x[j]==base[j]);streams[j]=std::make_unique<LiveBucket>(a.q,std::move(slices[j]),cost);
                open++;heads+=streams[j]->heap.size();peak_buckets=std::max<U>(peak_buckets,open);peak_heads=std::max<U>(peak_heads,heads);
            }
            auto& stream=*streams[j];
            while(left){
                u initial=stream.heap.size();stream.ensure(x[j]);heads-=initial-stream.heap.size();
                u take=left,label;
                if(!stream.ready||x[j]<stream.current.lo){label=mode[j];if(stream.ready)take=std::min(take,stream.current.lo-x[j]);default_claims++;}
                else{assert(x[j]<stream.current.hi);label=stream.current.tag;take=std::min(take,stream.current.hi-x[j]);}
                assert(take);emit(label,take);left-=take;x[j]+=take;inner_claims++;
            }
            if(x[j]==base[j+1]){assert(stream.heap.empty());streams[j].reset();open--;}
        }else if(op!=NO)emit(op,w);
        raw.left-=w;cur.take(outer[c],w);pos+=w;
    }
    close();assert(pos==a.m&&emitted==expected.size()&&open==0&&heads==0&&cost.reads==mass);
    if(print)live_csv<<block<<','<<a.m<<','<<mass<<','<<ranges<<','<<dirs<<','<<cost.reads<<','<<cost.comparisons<<','<<cost.runs<<','<<outer_claims<<','<<inner_claims<<','
        <<default_claims<<','<<peak_buckets<<','<<peak_heads<<','<<raw.records<<','<<emitted<<'\n';
}
