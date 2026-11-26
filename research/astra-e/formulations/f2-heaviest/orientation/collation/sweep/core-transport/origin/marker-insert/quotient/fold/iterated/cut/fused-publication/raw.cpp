// Source-space cut coloring without destination slices. At steady state the
// new word directory holds one original neighbor ID for each outside alias.
#include "fusion-without-main.inc"

template<bool Track> struct RawStream {
    U m=0,self=0,ingress=0,fragments=0;
    bool done=false;
    std::vector<U> b,deleted;
    std::vector<Pos> cursor;
    std::vector<int8_t> color;
    std::vector<std::vector<U>> targets;
    template<class F> void bucket(const Root& in,U k,F f) const {
        Pos p=cursor[k];U left=b[k+1]-b[k];
        while(left){
            auto r=in.r[p.run];U len=std::min(left,r.n-p.used);
            f(r.k,len);left-=len;++p.run;p.used=0;
        }
    }
    explicit RawStream(const Root& in):b(bases(in)),deleted(in.bytes.size()),cursor(in.bytes.size()),
                                       color(in.bytes.size(),-1),targets(in.bytes.size()){
        m=b.back();U pos=0;
        for(auto [k,w]:in.r){if(k==in.mark){assert(w==1);done=(pos==b[k]);break;}pos+=w;}
        if(done)return;
        pos=0;U j=0;
        for(U r=0;r<in.r.size();++r){auto [k,n]=in.r[r];
            while(n){
                if(pos==b[j])cursor[j]={r,in.r[r].n-n};
                U len=std::min(n,b[j+1]-pos);
                if(k==j)self+=len;
                if constexpr(Track)++fragments;
                n-=len;pos+=len;if(pos==b[j+1])++j;
            }
        }
        {
            std::vector<int> pending(color.size());
            auto decide=[&](U k,bool force){
                int vote=pending[k];
                bucket(in,k,[&](U u,U w){if(color[u]>=0)vote+=color[u]?-int(w):int(w);});
                color[k]=!force&&vote>0;
                int sign=color[k]?-1:1;
                bucket(in,k,[&](U u,U w){if(color[u]<0)pending[u]+=sign*int(w);});
            };
            decide(in.mark,true);
            for(U k=0;k<color.size();++k)if(k!=in.mark)decide(k,false);
        }
        for(U k=0;k<color.size();++k)bucket(in,k,[&](U u,U w){
            if(color[u])return;
            auto& v=targets[u];if(v.empty()||v.back()!=k)v.push_back(k);
            if(color[k]){deleted[k]+=w;ingress+=w;}
        });
    }
    Root apply(const Root& in,std::vector<Node>& grammar){
        auto K=color.size();std::vector<U> first(K+1),ord(K);
        for(U k=0;k<K;++k)
            first[k+1]=first[k]+(color[k]?U(b[k+1]-b[k]>deleted[k]):U(targets[k].size()));
        Root out;out.mark=first[in.mark];out.bytes.reserve(first.back());out.r.reserve(in.r.size());
        Meter q;
        for(U k=0;k<K;++k){
            if(color[k]){if(first[k]!=first[k+1])out.bytes.push_back(in.bytes[k]);continue;}
            for(U j:targets[k]){
                U v=in.bytes[k];
                if(color[j]){
                    U a=v,r=in.bytes[j];v=grammar.size();grammar.push_back({grammar[a].n+grammar[r].n,a,r,Node::Join});
                    if constexpr(Track)++q.selected;
                }
                out.bytes.push_back(v);
            }
        }
        assert(out.bytes.size()==first.back()&&targets[in.mark].size()==1);
        if constexpr(Track){
            q.K=K;q.rows=m;q.R=in.r.size();q.H=fragments;q.live_ids=first.back();
            q.target_count=std::count_if(deleted.begin(),deleted.end(),[](U v){return v!=0;});
            q.cursors_bytes=cursor.capacity()*sizeof(Pos)+ord.capacity()*sizeof(U);
            q.graph_bytes=targets.capacity()*sizeof(std::vector<U>);
            for(auto& v:targets){q.pairs+=v.size();q.graph_bytes+=v.capacity()*sizeof(U);}
        }
        U pos=0,j=0;
        for(auto [k,width]:in.r){
            if(!color[k]){
                auto& v=targets[k];U& rank=ord[k];
                while(width){
                    while(pos>=b[j+1])++j;
                    U len=std::min(width,b[j+1]-pos);
                    while(v[rank]<j)++rank;
                    assert(v[rank]==j);
                    if(!color[j])append(out.r,first[k]+rank,len);
                    width-=len;pos+=len;if constexpr(Track)++q.outer;
                }
            }else{
                if constexpr(Track)++q.outer;
                pos+=width;
                if(!deleted[k]){append(out.r,first[k],width);continue;}
                auto& p=cursor[k];
                while(width){
                    auto r=in.r[p.run];U len=std::min(width,r.n-p.used),id=first[k];
                    if(!color[r.k]){
                        const auto& v=targets[r.k];auto it=std::lower_bound(v.begin(),v.end(),k);
                        assert(it!=v.end()&&*it==k);id=first[r.k]+U(it-v.begin());
                        if constexpr(Track)++q.selected_queries;
                    }
                    append(out.r,id,len);p.used+=len;width-=len;
                    if(p.used==r.n){++p.run;p.used=0;}
                    if constexpr(Track){++q.side;q.side_rows+=len;}
                }
            }
        }
        if constexpr(Track){q.published=out.r.size();audit.push_back(q);}
        return out;
    }
};
struct RawCheck:StreamCut<false>{
    using StreamCut<false>::StreamCut;
    Root apply(const Root& in,std::vector<Node>& grammar){
        RawStream<true> candidate(in);
        assert(candidate.done==done&&candidate.color==color&&candidate.self==self&&candidate.ingress==ingress);
        auto ng=grammar;auto a=candidate.apply(in,ng), b=StreamCut<false>::apply(in,grammar);
        assert(equal_root(a,b)&&equal_nodes(ng,grammar));return b;
    }
};
constexpr const char* raw_arms[]={"compact_stream","joint","raw_graph","original"};
std::vector<uint8_t> raw_run(const Root& in,U arm,Stats& s){
    switch(arm){
        case 0:return decode_with<StreamCut<false>>(in,s);
        case 1:return decode_with<StreamFused<false>>(in,s);
        case 2:return decode_with<RawCut<false>>(in,s);
        default:assert(arm==3);return decode_with<RawStream<false>>(in,s);
    }
}
void raw_gate(const char* dir,bool test){
    for(auto f:getpaths(dir)){
        auto in=load(f.c_str());Stats s;audit.clear();auto y=decode_with<RawCheck>(in,s);
        assert(y==direct(in));for(U a=0;a<4;++a){Stats t;assert(raw_run(in,a,t)==y);}
        assert(audit.size()==s.passes.size());
        for(U i=0;i<audit.size();++i){auto q=audit[i];
            std::cout<<f.parent_path().filename().string()<<','<<i<<','<<q.rows<<','<<q.K<<','<<q.R<<','<<q.H<<','<<q.pairs<<','<<q.cursors_bytes<<','<<q.graph_bytes<<','<<q.side_rows<<','<<q.published<<'\n';
        }
    }
    if(test){
        uint64_t marked=0,passes=0,powers=0;
        for(U n=1;n<=9;++n){U alphabet=n<=6?3:2,count=1;
            for(U j=0;j<n;++j)count*=alphabet;
            for(U h=0;h<count;++h){
                std::vector<U> col(n),freq(alphabet),rank(alphabet);U v=h;Root in;
                for(auto& k:col){k=v%alphabet;v/=alphabet;++freq[k];}
                for(U k=0;k<alphabet;++k)if(freq[k]){rank[k]=in.bytes.size();in.bytes.push_back(11+31*k);}
                for(U k:col)append(in.r,rank[k],1);
                for(U mark=0;mark<n;++mark){
                    in.mark=mark;Stats s;audit.clear();auto y=decode_with<RawCheck>(in,s);assert(y==direct(in));
                    ++marked;passes+=s.passes.size();for(auto z:s.passes)powers+=z.normalized;
                }
            }
        }
        for(U n:{17U,128U,16384U}){Root in{{{1,1},{0,n}},{97,98},n};Stats s;
            assert(decode_with<RawCheck>(in,s)==direct(in)&&s.passes.size()==1&&s.passes[0].normalized);
        }
        std::cerr<<"native_marks,matched_passes,normalized,required_normalizations\n"<<marked<<','<<passes<<','<<powers<<",3\n";
    }
}
void raw_memory(const char* dir,U arm){
    std::cout<<"block,arm,output_bytes,fnv64,process_hwm_kib\n";
    for(auto f:getpaths(dir)){auto in=load(f.c_str());Stats s;auto y=raw_run(in,arm,s);
        uint64_t h=14695981039346656037ull;for(auto v:y){h^=v;h*=1099511628211ull;}
        std::cout<<f.parent_path().filename().string()<<','<<raw_arms[arm]<<','<<y.size()<<','<<h<<','<<resident_peak()<<'\n';
    }
}
void raw_panel(const char* dir){
    auto files=getpaths(dir);std::vector<Root> cases;std::vector<std::vector<uint8_t>> refs;
    for(auto f:files){cases.push_back(load(f.c_str()));refs.push_back(direct(cases.back()));}
    for(U i=0;i<10;++i)for(U a=0;a<4;++a){Stats s;assert(raw_run(cases[i],a,s)==refs[i]);}
    std::cout<<"cluster,round,block_position,block,position,arm,total_ns,graph_ns,publish_ns,power_ns,emit_ns,nodes\n";
    constexpr U schedule[]={0,1,3,2};
    for(U r=0;r<16;++r){U c=r/4;
        for(U st=0;st<10;++st){U i=(3*r+(c%2?10-st:st))%10;
            for(U p=0;p<4;++p){U j=schedule[p],a=(r%4+i+c+(c%2?4-j:j))%4;Stats s;
                auto t=cpu();auto y=raw_run(cases[i],a,s);auto end=cpu();assert(y==refs[i]);
                std::cout<<c<<','<<r<<','<<st<<','<<files[i].parent_path().filename().string()<<','<<p<<','<<raw_arms[a]<<','<<end-t<<','<<s.clocks.graph<<','<<s.clocks.apply<<','<<s.clocks.powers<<','<<s.clocks.emit<<','<<s.nodes<<'\n';}
        }
    }
}
int main(int argc,char** argv){assert(argc>=3);std::string mode=argv[1];
    if(mode=="gate"){
        std::cout<<"block,depth,rows,K,R,H,distinct_outside_pairs,cursor_bytes,directory_bytes,side_rows,output_runs\n";
        raw_gate(argv[2],true);
    }else if(mode=="memory"){assert(argc==4);raw_memory(argv[2],std::stoul(argv[3]));}
    else{assert(mode=="price"&&argc==3);raw_panel(argv[2]);}
}
