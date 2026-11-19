static std::vector<uint16_t> originals;
static u original_mark;
static std::ofstream modal_csv;
struct RawCaps{
    u index=0,original_pos=0,left=0,byte=0,records=0,runs=0,kept=0;
    bool ensure(){
        while(!left&&index<originals.size()){
            u c=originals[index]>>8,w=0;
            do{w+=originals[index]&255;index++;records++;}while(index<originals.size()&&(originals[index]>>8)==c);
            bool primary=original_pos<=original_mark&&original_mark<original_pos+w;
            left=w<=16?w:u(primary);byte=c;original_pos+=w;runs++;kept+=left!=0;
        }return left!=0;
    }
};
struct Exception{u lo,hi,tag;};
struct ModalBin{u byte;std::vector<Exception> slices;u si=0,pos=0;};
void override_header(){modal_csv<<"block,m,variable_sources,source_rows,old_cap_records,old_maximal_runs,original_retained_fragments,staged_source_claims,per_class_override_runs,per_band_override_runs,per_queue_override_runs,staged_outer_claims,staged_inner_claims,nondefault_rows,nondefault_queue_slices,nonempty_exception_bins,exception_q_reads,exception_heap_comparisons,sorted_exception_fragments,stored_exception_intervals,modal_inner_claims,default_gap_claims,output_runs\n";}
void override_check(const RootInput& a,const V& base,const V& uniform,const V& widths,const std::vector<std::vector<Bin>>& bins,
    const std::vector<std::vector<Span>>& ops,const std::vector<std::vector<Span>>& outer,const std::vector<std::vector<Span>>& ancestry,
    const V& phys,const std::vector<Span>& published,u block,bool print){
    const u K=base.size()-1,B=a.bands.size();V mode(K,NO);
    U mix=0,pull_mass=0,parts=0,perclass=0,perband=0,perqueue=0,outer_claims=0,inner_claims=0;
    U exceptional=0,nslices=0,active=0,exparts=0,intervals=0,modal_claims=0,gaps=0;
    MergeCost cost;
    // The joined alias used into this destination and its local carry both
    // have the exact positive destination mass as their final source width.
    for(u j=0;j<K;j++)if(uniform[j]==PULL){
        mix++;pull_mass+=base[j+1]-base[j];
        for(auto& b:bins[j])for(auto r:b.values)
            if(mode[j]==NO||widths[r.label]>widths[mode[j]]||(widths[r.label]==widths[mode[j]]&&r.label<mode[j]))mode[j]=r.label;
        assert(mode[j]!=NO);
    }
    std::vector<std::vector<Span>> ov(K),byband(B),byqueue(256);
    std::array<Cur,256> ac{};RawCaps first;
    for(u h:phys){
        for(auto op:ops[h]){u left=op.width;
            while(left){assert(first.ensure());u c=first.byte;auto& cur=ac[c];
                u take=std::min({left,first.left,cur.left(ancestry[c])});
                if(op.label!=NO&&(op.label&PULL)){
                    u j=op.label&~PULL,tag=cur.id(ancestry[c]);assert(tag!=NO);
                    append(ov[j],tag,take);append(byband[h],tag,take);
                }left-=take;first.left-=take;cur.take(ancestry[c],take);parts++;
            }
        }perband+=byband[h].size();
    }assert(!first.ensure());
    for(auto& v:ov)perclass+=v.size();
    for(u h=0;h<B;h++)for(auto r:byband[h])append(byqueue[a.bands[h].byte],r.label,r.width);
    for(auto& v:byqueue)perqueue+=v.size();
    std::vector<std::vector<Exception>> ex(K);
    for(u j=0;j<K;j++)if(mode[j]!=NO){
        std::vector<ModalBin> work;
        for(auto& b:bins[j]){u off=b.lo;ModalBin bin{b.c,{},0,0};
            for(auto r:b.values){if(r.label!=mode[j]){bin.slices.push_back({off,off+r.width,r.label});exceptional+=r.width;nslices++;}off+=r.width;}
            assert(off==b.hi);if(!bin.slices.empty())work.push_back(std::move(bin));
        }
        active+=work.size();
        std::priority_queue<Elem,std::vector<Elem>,Cmp> heap(Cmp{&cost});
        for(u i=0;i<work.size();i++){auto& b=work[i];b.pos=b.slices[0].lo;heap.push({a.q[b.byte][b.pos],i});cost.reads++;}
        while(!heap.empty()){
            auto e=heap.top();heap.pop();auto& b=work[e.idx];auto slice=b.slices[b.si];
            u start=e.pos,stop=start+1;b.pos++;
            while(b.pos<slice.hi){
                u next=a.q[b.byte][b.pos];cost.reads++;
                if(next!=stop){heap.push({next,e.idx});break;}
                b.pos++;stop++;
            }
            if(b.pos==slice.hi&&++b.si<b.slices.size()){
                b.pos=b.slices[b.si].lo;heap.push({a.q[b.byte][b.pos],e.idx});cost.reads++;
            }
            auto& out=ex[j];if(!out.empty()&&out.back().hi==start&&out.back().tag==slice.tag)out.back().hi=stop;
            else out.push_back({start,stop,slice.tag});exparts++;
        }
        intervals+=ex[j].size();
        u n=0;for(auto r:ex[j]){assert(r.lo>=base[j]&&r.hi<=base[j+1]&&r.lo<r.hi&&r.tag!=mode[j]);n+=r.hi-r.lo;}
        assert(n==base[j+1]-base[j]-widths[mode[j]]);
    }
    auto expand=[&](bool sparse){
        std::array<Cur,256> tagpos{};std::vector<Cur> ovpos(K);V x=base,expos(K);
        RawCaps raw;std::vector<Span> out;u pos=0;
        while(raw.ensure()){
            u c=raw.byte;auto& cur=tagpos[c];u w=std::min(raw.left,cur.left(outer[c])),tag=cur.id(outer[c]);
            outer_claims+=!sparse;
            if(tag!=NO&&(tag&PULL)){u j=tag&~PULL,left=w;
                while(left){
                    u take=left,val;
                    if(!sparse){auto& p=ovpos[j];take=std::min(take,p.left(ov[j]));val=p.id(ov[j]);p.take(ov[j],take);inner_claims++;}
                    else{const auto& stream=ex[j];auto& z=expos[j];
                        if(z==stream.size()||x[j]<stream[z].lo){val=mode[j];if(z<stream.size())take=std::min(take,stream[z].lo-x[j]);gaps++;}
                        else{auto r=stream[z];assert(r.lo<=x[j]&&x[j]<r.hi);val=r.tag;take=std::min(take,r.hi-x[j]);z+=x[j]+take==r.hi;}
                        x[j]+=take;modal_claims++;
                    }
                    assert(take);append(out,val,take);left-=take;
                }
            }else if(tag!=NO)append(out,tag,w);
            raw.left-=w;cur.take(outer[c],w);pos+=w;
        }
        assert(pos==a.m&&raw.records==first.records&&out.size()==published.size());
        for(u i=0;i<out.size();i++)assert(out[i].width==published[i].width&&out[i].label==published[i].label);
    };
    expand(false);expand(true);
    if(print)modal_csv<<block<<','<<a.m<<','<<mix<<','<<pull_mass<<','<<first.records<<','<<first.runs<<','<<first.kept<<','<<parts<<','<<perclass<<','<<perband<<','<<perqueue<<','
        <<outer_claims<<','<<inner_claims<<','<<exceptional<<','<<nslices<<','<<active<<','<<cost.reads<<','<<cost.comparisons<<','<<exparts<<','<<intervals<<','<<modal_claims<<','<<gaps<<','<<published.size()<<'\n';
}
