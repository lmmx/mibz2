#define main census_main
#include "../../census.cpp"
#undef main
#include <map>
#include <memory>
#include <queue>
#include <functional>
struct Frag{u i,j,w,a=NO;};
struct RootInput{
    u m=0,mark=0;
    std::vector<Band> bands;
    V names; // One exact word ID per physical band in queue order.
    std::vector<V> q=std::vector<V>(256); // Provided existing constructor queues.
};
struct Cur{
    size_t at=0;u used=0;
    u id(const std::vector<Span>& a)const{return a[at].label;}
    u left(const std::vector<Span>& a)const{return a[at].width-used;}
    void take(const std::vector<Span>& a,u w){assert(w&&w<=left(a));used+=w;if(used==a[at].width){used=0;at++;}}
};
struct Bin{u c,lo,hi;std::vector<Span> values;Cur pos;};
struct MergeCost{U reads=0,comparisons=0,runs=0;};
struct Elem{u pos,idx;};
struct Cmp{MergeCost* cost;bool operator()(Elem a,Elem b)const{cost->comparisons++;return a.pos>b.pos;}};
// Sorted disjoint Q slices cover every rank in [lo,hi). After popping p the next
// competing low byte starts at s: the winning list covers exactly [p,s).
struct LowOrder{
    const std::vector<V>& q;std::vector<Bin>& b;u end;
    std::priority_queue<Elem,std::vector<Elem>,Cmp> heap;V cur;MergeCost& cost;
    u ix=NO,left=0;
    LowOrder(const std::vector<V>& qq,std::vector<Bin>& bb,u end_,MergeCost& cc):
        q(qq),b(bb),end(end_),heap(Cmp{&cc}),cur(bb.size()),cost(cc){
        for(u i=0;i<b.size();i++){cur[i]=b[i].lo;heap.push({q[b[i].c][cur[i]],i});cost.reads++;}
    }
    void ensure(){
        if(left)return;
        assert(!heap.empty());auto top=heap.top();heap.pop();ix=top.idx;
        u limit=heap.empty()?end:heap.top().pos;assert(limit>top.pos);
        left=limit-top.pos;cur[ix]+=left;assert(cur[ix]<=b[ix].hi);cost.runs++;
        if(cur[ix]<b[ix].hi){heap.push({q[b[ix].c][cur[ix]],ix});cost.reads++;}
    }
};
using Names=std::vector<std::pair<u,u>>;
constexpr u PULL=1u<<31;
struct Counts{
    U lookup=0,rank_compare=0,frags=0,supp=0,livebins=0,aruns=0,outerops=0;
    U srcops=0,p_src=0,p_ancestry=0,mid_runs=0,second_runs=0,p_dest=0,pulled=0,p_inner=0,p_const=0;
    U root_pieces=0;
    MergeCost outer,inner,stage,finish;
};
void compare_stock(const std::vector<Span>& old,const V& words,const V& color,u mark,
    const std::vector<Span>& result,const Names& names,u next_mark);
#ifdef EDGE_MODAL_OVERRIDES
#include "overrides/modal.inc"
#endif
void process_order(const RootInput& a,const V& truth,const Weights& words,const std::vector<uint8_t>& oracle,
       u block,int forced,bool print){
    const u m=a.m,B=a.bands.size();V physical(B),start(B),base,weight;
    std::iota(physical.begin(),physical.end(),0);std::sort(physical.begin(),physical.end(),[&](u l,u r){return a.bands[l].start<a.bands[r].start;});
    u at=0,prev=NO;
    for(u h:physical){const auto& b=a.bands[h];assert(b.start==at);
        if(a.names[h]!=prev){base.push_back(at);weight.push_back(a.names[h]);}
        at+=b.width;prev=a.names[h];
    }
    assert(at==m);
    for(u p:{a.mark,a.mark+1})if(p<m){
        auto it=std::lower_bound(base.begin(),base.end(),p);if(it==base.end()||*it!=p){
            u j=it-base.begin();base.insert(it,p);u id=weight[j-1];weight.insert(weight.begin()+j,id);
        }
    }
    base.push_back(m);u K=weight.size();
    Counts cnt;
    auto klass=[&](u y){u lo=0,hi=K;while(hi-lo>1){u j=(hi+lo)/2;cnt.rank_compare++;if(base[j]<=y)lo=j;else hi=j;}return lo;};
    u seed=klass(a.mark);assert(base[seed+1]-base[seed]==1);
    std::array<u,256> endq{};for(u h=0;h<B;h++){
        u c=a.bands[h].byte;start[h]=endq[c];endq[c]+=a.bands[h].width;
    }
    u bins=1;while(bins<K)bins*=2;V dif(bins),first(B),stop(B);
    std::vector<Frag> fs;
    for(u h:physical){auto b=a.bands[h];first[h]=fs.size();u x=b.start;
        const auto& q=a.q[b.byte];
        auto readq=[&](u i){cnt.lookup++;return q[i];};
        while(x<b.start+b.width){u i=klass(x),z=std::min(b.start+b.width,base[i+1]);u l=start[h]+x-b.start,t=l+z-x;
            u last=readq(t-1);
            while(l<t){u j=klass(readq(l)),split=t;
                if(last>=base[j+1]){u good=l,bad=t-1;
                    for(u step=1;step<bad-l;step*=2){u p=l+step;if(readq(p)>=base[j+1]){bad=p;break;}good=p;}
                    while(bad-good>1){u p=(bad+good)/2;if(readq(p)<base[j+1])good=p;else bad=p;}split=bad;
                }
                dif[i^j]+=split-l;fs.push_back({i,j,split-l});l=split;
            }x=z;
        }stop[h]=fs.size();
    }
    u v=0;for(u w=1;w<bins;w*=2){int s=0;for(u j=w;j<2*w;j++)s+=__builtin_parity(v&j)?-int(dif[j]):int(dif[j]);if(s>0)v+=w;}
    V color(K),deleted(K),surv(K,NO),uniform(K,NO);for(u i=0;i<K;i++)color[i]=forced<0?__builtin_parity(v&(i^seed)):((u(forced)>>i)&1u);
    color[seed]=0;u removed=0;
    for(auto f:fs)if(!color[f.i]&&color[f.j]){deleted[f.j]+=f.w;removed+=f.w;}
    Names names;V widths;u last_i=NO,last_j=NO,current=NO,next_mark=NO;
    for(auto& f:fs){
        if(color[f.i]){if(last_i!=f.i&&base[f.i+1]-base[f.i]>deleted[f.i]){
            surv[f.i]=names.size();names.push_back({weight[f.i],NO});widths.push_back(base[f.i+1]-base[f.i]-deleted[f.i]);
        }f.a=surv[f.i];}
        else{u group=color[f.j]?f.j:NO;if(f.i!=last_i||last_j!=group){
                current=names.size();names.push_back({weight[f.i],group==NO?NO:weight[group]});widths.push_back(0);}
            f.a=current;widths[f.a]+=f.w;last_j=group;if(f.i==seed)next_mark=f.a;}
        last_i=f.i;
    }
    assert(next_mark<names.size()&&widths[next_mark]==1);
    auto ancestor=[&](const Frag& f){assert(color[f.j]);u tag=color[f.i]?surv[f.j]:f.a;assert(tag!=NO);return tag;};
    for(auto f:fs)if(color[f.j]){u tag=ancestor(f);auto& z=uniform[f.j];if(z==NO)z=tag;else if(z!=tag)z=PULL;}
    auto directive=[&](const Frag& f){
        if(!color[f.i])return color[f.j]?NO:f.a;
        return uniform[f.i]==PULL?(PULL|f.i):uniform[f.i];
    };
    std::vector<std::vector<Span>> ops(B),outer(256),ancestry(256),mid(B),final(256);
    std::vector<std::vector<Bin>> branches(K);
    for(u h=0;h<B;h++){
        u c=a.bands[h].byte,pos=start[h];
        for(u i=first[h];i<stop[h];i++){
            auto f=fs[i];u tag=directive(f);append(ops[h],tag,f.w);append(outer[c],tag,f.w);
            u anc=NO;
            if(color[f.j]&&uniform[f.j]==PULL){
                anc=ancestor(f);auto& dirs=branches[f.j];
                if(dirs.empty()||dirs.back().c!=c){dirs.push_back({c,pos,pos,{},{}});}
                auto& d=dirs.back();assert(d.hi==pos);d.hi+=f.w;append(d.values,anc,f.w);
            }
            append(ancestry[c],anc,f.w);pos+=f.w;
        }cnt.srcops+=ops[h].size();
    }
    for(u c=0;c<256;c++)cnt.outerops+=outer[c].size();
    std::vector<Bin> all;for(u c=0;c<256;c++)if(!a.q[c].empty())all.push_back({c,0,(u)a.q[c].size(),{},{}});
    std::vector<Span> published,other;
    std::vector<std::unique_ptr<LowOrder>> pulls(K);
    for(u i=0;i<K;i++)if(!branches[i].empty()){
        cnt.livebins++;cnt.supp+=branches[i].size();for(auto& bin:branches[i])cnt.aruns+=bin.values.size();
        pulls[i]=std::make_unique<LowOrder>(a.q,branches[i],base[i+1],cnt.inner);
    }else if(color[i])cnt.p_const+=base[i+1]-base[i];
    auto emit=[&](std::vector<Span>& out,u tag,u w){if(tag!=NO)append(out,tag,w);};
    // Direct ordered iterator A. next actions combine queues, sparse bins and a
    // single previous output run. Storing its RLE here is only for the audit.
    LowOrder out_order(a.q,all,m,cnt.outer);std::array<Cur,256> outpos{};
    at=0;while(at<m){out_order.ensure();u c=all[out_order.ix].c;auto& cu=outpos[c];
        u tag=cu.id(outer[c]),w=std::min(out_order.left,cu.left(outer[c]));cnt.p_dest++;
        if(tag!=NO&&(tag&PULL)){u j=tag&~PULL;auto& branch=*pulls[j];u left=w;cnt.pulled+=w;
            while(left){branch.ensure();auto& dir=branches[j][branch.ix];u take=std::min({left,branch.left,dir.pos.left(dir.values)});
                emit(published,dir.pos.id(dir.values),take);dir.pos.take(dir.values,take);left-=take;branch.left-=take;cnt.p_inner++;}
        }else emit(published,tag,w);
        cu.take(outer[c],w);out_order.left-=w;at+=w;
    }
    // Direct schedule B, source-order annotation and one destination-order replay.
    LowOrder src(a.q,all,m,cnt.stage);std::array<Cur,256> acur{};
    for(u h:physical){for(auto step:ops[h]){u left=step.width;
        while(left){src.ensure();u c=all[src.ix].c;auto& cu=acur[c];u w=std::min({src.left,left,cu.left(ancestry[c])});
            u tag=step.label;if(tag!=NO&&(tag&PULL)){tag=cu.id(ancestry[c]);assert(tag!=NO);}
            append(mid[h],tag,w);cu.take(ancestry[c],w);src.left-=w;left-=w;cnt.p_src++;
        }
    }cnt.mid_runs+=mid[h].size();}
    for(u h=0;h<B;h++)for(auto r:mid[h])append(final[a.bands[h].byte],r.label,r.width);
    for(u c=0;c<256;c++)cnt.second_runs+=final[c].size();
    LowOrder replay(a.q,all,m,cnt.finish);std::array<Cur,256> fcur{};
    at=0;while(at<m){replay.ensure();u c=all[replay.ix].c;auto& cu=fcur[c];
        u w=std::min(replay.left,cu.left(final[c]));emit(other,cu.id(final[c]),w);cu.take(final[c],w);replay.left-=w;at+=w;cnt.p_ancestry++;}
    assert(other.size()==published.size());for(u i=0;i<other.size();i++)assert(other[i].label==published[i].label&&other[i].width==published[i].width);
#ifdef EDGE_MODAL_OVERRIDES
    override_check(a,base,uniform,widths,branches,ops,outer,ancestry,physical,published,block,print);
#endif
    // Verification only. The truth permutation, old incoming RLE, new dense
    // ranks and output vectors below are not used by either publication schedule.
    V inc(m);for(u x=0;x<m;x++){auto it=std::upper_bound(base.begin(),base.end(),x);inc[truth[x]]=it-base.begin()-1;}
    std::vector<Span> old;for(u z:inc)append(old,z,1);
    compare_stock(old,weight,color,seed,published,names,next_mark);
    V newbase(names.size()+1);std::partial_sum(widths.begin(),widths.end(),newbase.begin()+1);
    assert(newbase.back()==m-removed);
    V ptr=newbase,nx(m-removed),type(m-removed);at=0;
    for(auto r:published){for(u z=0;z<r.width;z++){u x=ptr[r.label]++;nx[x]=at++;type[x]=r.label;}}
    assert(at==m-removed);
    size_t bytes=0;u row=newbase[next_mark];while(bytes<oracle.size()){auto [l,r]=names[type[row]];
        for(u w:{l,r})if(w!=NO)for(uint8_t c:words.value[w])if(bytes<oracle.size())assert(oracle[bytes++]==c);
        row=nx[row];
    }
    if(print)std::cout<<block<<','<<m<<','<<B<<','<<K<<','<<fs.size()<<','<<cnt.lookup<<','<<cnt.rank_compare<<','<<removed<<','<<names.size()<<','<<published.size()<<','
      <<cnt.livebins<<','<<cnt.supp<<','<<cnt.aruns<<','<<cnt.p_const<<','<<cnt.pulled<<','<<cnt.outerops<<','<<cnt.p_dest<<','<<cnt.p_inner<<','
      <<cnt.outer.runs<<','<<cnt.outer.reads<<','<<cnt.outer.comparisons<<','<<cnt.inner.runs<<','<<cnt.inner.reads<<','<<cnt.inner.comparisons<<','
      <<cnt.srcops<<','<<cnt.p_src<<','<<cnt.mid_runs<<','<<cnt.second_runs<<','<<cnt.p_ancestry<<'\n';
}
void small(){
    U cases=0,short_laps=0,nondiv=0;
    for(u letters:{2u,3u})for(u m=1;m<=(letters==2?6u:5u);m++){
        u limit=1;for(u j=0;j<m;j++)limit*=letters;
        for(u pattern=0;pattern<limit;pattern++){
            V l(m),next(m),owner(m),pred(m);u v=pattern;for(auto& c:l){c=v%letters;v/=letters;}
            std::iota(next.begin(),next.end(),0);std::stable_sort(next.begin(),next.end(),[&](u x,u y){return l[x]<l[y];});
            Weights ww;V w(letters);for(u c=0;c<letters;c++){
                std::string t(1,char('a'+c));if(c%2)t+="p";t+=char((pattern%2==0)?1:1+c%2);w[c]=ww.intern(t);
            }
            RootInput inp;inp.m=m;for(u x=0;x<m;x++){owner[x]=w[l[next[x]]];pred[next[x]]=x;}
            for(u y=0;y<m;y++){u x=pred[y];inp.q[(uint8_t)ww.value[owner[x]].back()].push_back(y);}
            for(u c=0;c<256;c++)for(u y:inp.q[c]){
                u x=pred[y];if(!inp.bands.empty()&&inp.bands.back().byte==c&&inp.names.back()==owner[x]
                    &&inp.bands.back().start+inp.bands.back().width==x)inp.bands.back().width++;
                else{inp.bands.push_back({x,1,0,c});inp.names.push_back(owner[x]);}
            }
            u horizon=0;for(u w:owner)horizon+=ww.value[w].size();
            for(u p=0;p<m;p++){
                inp.mark=p;
#ifdef EDGE_MODAL_OVERRIDES
                originals.clear();original_mark=p;
                for(u y=0;y<m;y++){
                    auto c=(uint8_t)ww.value[owner[pred[y]]].back();
                    if(!originals.empty()&&(originals.back()>>8)==c)originals.back()++;
                    else originals.push_back((u(c)<<8)|1);
                }
#endif
                std::vector<uint8_t> cycle,oracle;u x=p;
                do{for(uint8_t z:ww.value[owner[x]])cycle.push_back(z);x=next[x];}while(x!=p);
                for(u j=0;j<horizon;j++)oracle.push_back(cycle[j%cycle.size()]);
                u classes=1;for(u j=1;j<m;j++)classes+=(owner[j]!=owner[j-1]||j==p||j==p+1);
                for(u mask=0;mask<(1u<<classes);mask++){
                    process_order(inp,next,ww,oracle,0,mask,false);cases++;
                    short_laps+=cycle.size()<horizon;nondiv+=horizon%cycle.size()!=0;
                }
            }
        }
    }
    std::cout<<cases<<" marked weighted columns/color assignments including "<<short_laps<<" shorter marked laps and "<<nondiv
             <<" nondivisor horizons: both ordered schedules and complete donor RLE, alias, mark, spelling matched.\n";
}
int main(int argc,char** argv){
    if(argc==2&&std::string(argv[1])=="small"){small();return 0;}
#ifdef EDGE_MODAL_OVERRIDES
    assert(argc==6);std::ifstream cap(argv[4],std::ios::binary);modal_csv.open(argv[5]);override_header();
#else
    assert(argc==4);
#endif
    std::ifstream edge(argv[1],std::ios::binary),word(argv[2],std::ios::binary),thin(argv[3],std::ios::binary);
    std::cout<<"block,m,bands,seeded_classes,cert_frags,cert_queue_reads,cert_rank_comparisons,deleted,aliases,output_runs,mixed_buckets,occupied_mixed_bins,stored_ancestry_runs,constant_inside_mass,pulled_mass,outer_queue_ops,outer_claims,inner_claims,outer_low_runs,outer_heap_reads,outer_heap_comparisons,inner_low_runs,inner_heap_reads,inner_heap_comparisons,source_band_ops,source_composition_claims,band_final_tags,queue_final_tags,replay_claims\n";
    u n=0;while(true){Block b;if(!next_block(b,edge,word,thin))break;
#ifdef EDGE_MODAL_OVERRIDES
        assert(get(cap)==b.index&&get(cap)==b.n);original_mark=get(cap);u ncap=get(cap);originals.resize(ncap);read(cap,originals.data(),2*ncap);
#endif
        RootInput in;in.m=b.m;in.mark=b.mark;in.bands=b.band;
        for(auto r:b.band)in.names.push_back(b.weight[r.code]);
        for(u y=0;y<b.m;y++)in.q[b.thin[y]].push_back(y);
        process_order(in,b.next,b.words,b.reference,b.index,-1,true);n++;
    }assert(n==233);std::cerr<<n<<" original edge roots; both ordered methods matched each donor RLE/alias/mark and all 209697662 marked coded bytes.\n";
}
