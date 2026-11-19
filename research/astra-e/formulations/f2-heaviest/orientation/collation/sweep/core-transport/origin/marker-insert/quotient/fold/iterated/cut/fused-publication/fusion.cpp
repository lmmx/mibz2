#define main archived_compact_main
#include "xor/route.cpp"
#undef main

struct Meter {
    U rows=0, K=0, R=0, H=0, pairs=0, selected=0, live_ids=0, target_count=0;
    U outer=0, side=0, side_rows=0, selected_queries=0, published=0, old_map_runs=0;
    U cursors_bytes=0, old_slices_bytes=0, graph_bytes=0;
};
static std::vector<Meter> audit;
struct Pos {U run=0,used=0;};
struct Adj {U target,mass;};

// Construct necessarily source-ordered output IDs directly. Every OUT subclass
// survives and each IN class has at most one revised ID. The old singleton
// mark is OUT, so its only neighboring class uses the first rank of its bucket.
template<bool Track,class C>
Root dictionary(const Root& in,C& c,const std::vector<U>& base,std::vector<Node>& nodes,
                std::vector<U>& first,std::vector<U>& del,Meter& m) {
    const U K=in.bytes.size();first.resize(K+1);del.assign(K,0);
    for(U k=0;k<K;++k) if(!c.color[k])for(auto e:c.graph[k]) {
        if(c.color[e.target]){del[e.target]+=e.mass;if constexpr(Track)++m.selected;}
    }
    for(U k=0;k<K;++k) {
        first[k+1]=first[k]+(c.color[k]?U(base[k+1]-base[k]>del[k]):U(c.graph[k].size()));
        if constexpr(Track){m.pairs+=c.graph[k].size();m.target_count+=del[k]!=0;}
    }
    Root out;out.bytes.reserve(first.back());
    for(U k=0;k<K;++k) {
        if(c.color[k]) {if(first[k]<first[k+1])out.bytes.push_back(in.bytes[k]);continue;}
        for(auto e:c.graph[k]) {
            U h=in.bytes[k];
            if(c.color[e.target]){
                U b=in.bytes[e.target],a=h;h=nodes.size();nodes.push_back({nodes[a].n+nodes[b].n,a,b,Node::Join});
            }
            out.bytes.push_back(h);
        }
    }
    assert(out.bytes.size()==first.back() && !c.color[in.mark] && c.graph[in.mark].size()==1);
    out.mark=first[in.mark];if constexpr(Track){m.live_ids=first.back();m.K=K;m.R=in.r.size();m.rows=base.back();}
    return out;
}

template<bool Track> struct SliceCut:Cut{
    using Cut::Cut;
    Root apply(const Root& in,std::vector<Node>& nodes) {
        Meter m;std::vector<U> first,del;
        Root out=dictionary<Track>(in,*this,b,nodes,first,del,m);
        const U K=graph.size();
        std::vector<Pos> cursor(K);
        for(U k=0;k<K;++k)if(del[k]){
            auto it=std::lower_bound(parts.begin(),parts.end(),k,[](const Slice& a,U j){return a.target<j;});
            cursor[k].run=it-parts.begin();
        }
        if constexpr(Track){
            m.H=parts.size();m.old_slices_bytes=parts.capacity()*sizeof(Slice);
            m.cursors_bytes=cursor.capacity()*sizeof(Pos);
            for(auto& g:graph)m.graph_bytes+=g.capacity()*sizeof(Edge);
        }
        out.r.reserve(in.r.size());
        for(auto s:parts) {
            U k=s.source;
            if constexpr(Track)++m.outer;
            if(!color[k]) {
                if(!color[s.target])append(out.r,first[k]+s.edge,s.width);
            }else if(!del[k]){
                append(out.r,first[k],s.width);
            }else{
                Pos& p=cursor[k];U left=s.width;
                while(left){
                    auto u=parts[p.run];assert(u.target==k);
                    U w=std::min(left,u.width-p.used);
                    U label=color[u.source]?first[k]:first[u.source]+u.edge;
                    append(out.r,label,w);left-=w;p.used+=w;
                    if constexpr(Track){++m.side;m.side_rows+=w;m.selected_queries+=!color[u.source];}
                    if(p.used==u.width){++p.run;p.used=0;}
                }
            }
        }
        if constexpr(Track){m.published=out.r.size();audit.push_back(m);}
        return out;
    }
};

template<bool Track> struct RawCut{
    U m=0,self=0,ingress=0,h=0;bool done=false;
    std::vector<U> b;
    std::vector<std::vector<Adj>> graph;
    std::vector<int8_t> color;
    std::vector<Pos> start;
    RawCut(const Root& in):b(bases(in)),graph(in.bytes.size()),color(in.bytes.size(),-1),start(in.bytes.size()) {
        m=b.back();U pos=0;
        for(auto [k,n]:in.r){if(k==in.mark){assert(n==1);done=(pos==b[k]);break;}pos+=n;}
        if(done)return;
        U j=0;pos=0;
        for(U r=0;r<in.r.size();++r){
            auto [k,width]=in.r[r];U left=width;
            while(left){
                if(pos==b[j])start[j]={r,width-left};
                U n=std::min(left,b[j+1]-pos);auto& edges=graph[k];
                if(edges.empty()||edges.back().target!=j)edges.push_back({j,0});
                edges.back().mass+=n;self+=(k==j?n:0);
                pos+=n;left-=n;if(pos==b[j+1])++j;
                if constexpr(Track)++h;
            }
        }
        std::vector<std::vector<std::pair<U,U>>> incoming(graph.size());
        for(U k=0;k<graph.size();++k)for(auto e:graph[k])if(e.target!=k)incoming[e.target].push_back({k,e.mass});
        color[in.mark]=0;
        for(U k=0;k<graph.size();++k)if(k!=in.mark){
            int gain=0;
            auto vote=[&](U j,U f){if(color[j]>=0)gain+=color[j]?-int(f):int(f);};
            for(auto e:graph[k])vote(e.target,e.mass);
            for(auto [j,n]:incoming[k])vote(j,n);
            color[k]=gain>0;
        }
        for(U k=0;k<graph.size();++k)if(!color[k])for(auto e:graph[k])if(color[e.target])ingress+=e.mass;
    }
    Root apply(const Root& in,std::vector<Node>& nodes){
        Meter mt;std::vector<U> first,del,ord(graph.size());
        Root out=dictionary<Track>(in,*this,b,nodes,first,del,mt);out.r.reserve(in.r.size());
        if constexpr(Track){
            mt.H=h;mt.cursors_bytes=start.capacity()*sizeof(Pos)+ord.capacity()*sizeof(U);
            for(auto& g:graph)mt.graph_bytes+=g.capacity()*sizeof(Adj);
        }
        U pos=0,j=0;
        for(auto [k,width]:in.r){
            if(!color[k]){
                U left=width;U& e=ord[k];
                while(left){
                    while(pos>=b[j+1])++j;
                    U n=std::min(left,b[j+1]-pos);while(graph[k][e].target<j)++e;
                    assert(graph[k][e].target==j);
                    if(!color[j])append(out.r,first[k]+e,n);
                    pos+=n;left-=n;if constexpr(Track)++mt.outer;
                }
            }else{
                if constexpr(Track)++mt.outer;
                pos+=width;
                if(!del[k]){append(out.r,first[k],width);continue;}
                Pos& p=start[k];
                while(width){
                    auto u=in.r[p.run];U w=std::min(width,u.n-p.used);
                    U label=first[k];
                    if(!color[u.k]){
                        const auto& list=graph[u.k];
                        auto it=std::lower_bound(list.begin(),list.end(),k,[](Adj v,U k){return v.target<k;});
                        assert(it!=list.end()&&it->target==k);label=first[u.k]+U(it-list.begin());
                        if constexpr(Track)++mt.selected_queries;
                    }
                    append(out.r,label,w);
                    if constexpr(Track){++mt.side;mt.side_rows+=w;}
                    p.used+=w;width-=w;if(p.used==u.n){++p.run;p.used=0;}
                }
            }
        }
        if constexpr(Track){mt.published=out.r.size();audit.push_back(mt);}
        return out;
    }
};
#include "stream-fused.hpp"

bool equal_nodes(const std::vector<Node>& a,const std::vector<Node>& b){
    return a.size()==b.size()&&std::equal(a.begin(),a.end(),b.begin(),[](Node u,Node v){
        return u.kind==v.kind&&u.n==v.n&&u.l==v.l&&u.r==v.r;
    });
}
bool equal_root(const Root&a,const Root&b){return a.mark==b.mark&&a.bytes==b.bytes&&a.r==b.r;}
struct Check:GraphCompact{
    using GraphCompact::GraphCompact;
    Root apply(const Root& in,std::vector<Node>& nodes) {
        SliceCut<true> s(in);RawCut<true> r(in);StreamCut<false> z(in);StreamFused<true> f(in);
        assert(!s.done&&!r.done&&s.ingress==ingress&&r.ingress==ingress&&s.self==self&&r.self==self);
        assert(s.color==color&&r.color==color&&z.color==color&&f.color==color&&f.ingress==ingress);
        auto ns=nodes,nr=nodes,nz=nodes,nf=nodes;
        auto a=s.apply(in,ns),b=r.apply(in,nr),d=z.apply(in,nz),e=f.apply(in,nf);
        const U K=color.size(),none=UINT32_MAX;
        std::vector<bool> selected(K,false);for(U k=0;k<K;++k)if(!color[k])
            for(auto e:graph[k]){if(color[e.target])selected[e.target]=true;}
        U count=0,prior_class=none,prior_label=none;
        for(auto x:parts)if(selected[x.target]){
            U label=color[x.source]?K:x.source;
            // For this fixed target, each OUT predecessor has just one alias.
            if(prior_class!=x.target||prior_label!=label)++count;
            prior_label=label;prior_class=x.target;
        }
        for(U i=0;i<3;++i)audit[audit.size()-1-i].old_map_runs=count;
        auto expected=GraphCompact::apply(in,nodes);
        assert(equal_root(expected,a)&&equal_root(expected,b)&&equal_nodes(ns,nodes)&&equal_nodes(nr,nodes)&&equal_root(d,expected)&&equal_root(e,expected)&&equal_nodes(nz,nodes)&&equal_nodes(nf,nodes));
        return expected;
    }
};

static const char* arms[]={"compact","slices","raw","direct","stream_a","stream_b","joint_a","joint_b"};
std::vector<uint8_t> run_arm(const Root& in,U arm,Stats& s) {
    if(arm==0)return decode_with<GraphCompact>(in,s);
    if(arm==1)return decode_with<SliceCut<false>>(in,s);
    if(arm==2)return decode_with<RawCut<false>>(in,s);
    if(arm>=4&&arm<=5)return decode_with<StreamCut<false>>(in,s);
    if(arm>=6)return decode_with<StreamFused<false>>(in,s);
    return direct(in);
}
std::vector<std::filesystem::path> getpaths(std::filesystem::path dir){
    std::vector<std::filesystem::path> ps;
    for(auto& e:std::filesystem::directory_iterator(dir))
        if(auto tag=e.path().filename().string(); tag.size()==4 && tag.find_first_not_of("0123456789")==std::string::npos && std::filesystem::exists(e.path()/"input.txt"))ps.push_back(e.path()/"input.txt");
    std::sort(ps.begin(),ps.end());assert(ps.size()==10);return ps;
}
void small(){
    uint64_t cases=0,passes=0,normalized=0;
    for(U n=1;n<=9;++n){
        U al=n<=6?3:2,num=1;for(U h=0;h<n;++h)num*=al;
        for(U h=0;h<num;++h){
            Root in;std::vector<U>a(n),freq(al),rank(al);U v=h;
            for(U& k:a){k=v%al;v/=al;++freq[k];}
            for(U k=0;k<al;++k)if(freq[k]){
                rank[k]=in.bytes.size();in.bytes.push_back(11+31*k);
            }
            for(U k:a)append(in.r,rank[k],1);
            for(U p=0;p<n;++p){in.mark=p;Stats s;audit.clear();auto y=decode_with<Check>(in,s);
                assert(y==direct(in));++cases;passes+=s.passes.size();
                for(auto row:s.passes)normalized+=row.normalized;
            }
        }
    }
    for(U n: {17U,128U,16384U}){
        Root a{{{1,1},{0,n}},{97,98},n};Stats s;auto y=decode_with<Check>(a,s);
        assert(y==direct(a)&&s.passes.size()==1&&s.passes.front().normalized);
    }
    std::cout<<"native_marks,matched_passes,normalized,required_normalizations\n"<<cases<<','<<passes<<','<<normalized<<",3\n";
}
void check(const char* path,const char* folder){
    auto in=load(path);auto want=direct(in);Stats s;
    audit.clear();auto y=decode_with<Check>(in,s);assert(y==want);
    for(U a=0;a<8;++a){Stats t;assert(run_arm(in,a,t)==y);}
    std::ofstream b(std::filesystem::path(folder)/"marked.bin",std::ios::binary);
    b.write((const char*)y.data(),y.size());
    std::cout<<"depth,arm,rows,K,R,H,pairs,selected,ids,targets,outer,side,side_rows,side_aliases,out_runs,donor_ancestry,cursor_bytes,slice_bytes,adjacency_bytes,normalized\n";
    assert(audit.size()==3*s.passes.size());
    for(U i=0;i<audit.size();++i){auto q=audit[i];std::cout<<i/3<<','<<(i%3==2?"stream_joint":arms[1+i%3])<<','<<q.rows<<','<<q.K<<','<<q.R<<','<<q.H<<','<<q.pairs<<','<<q.selected<<','<<q.live_ids<<','<<q.target_count<<','<<q.outer<<','<<q.side<<','<<q.side_rows<<','<<q.selected_queries<<','<<q.published<<','<<q.old_map_runs<<','<<q.cursors_bytes<<','<<q.old_slices_bytes<<','<<q.graph_bytes<<','<<s.passes[i/3].normalized<<'\n';}
}
void memory(const char* dir,U a){
    std::cout<<"block,arm,output_bytes,fnv64,process_hwm_kib\n";
    for(auto f:getpaths(dir)){
        auto in=load(f.c_str());Stats s;auto y=run_arm(in,a,s);
        uint64_t h=14695981039346656037ull;for(auto v:y){h^=v;h*=1099511628211ull;}
        std::cout<<f.parent_path().filename().string()<<','<<arms[a]<<','<<y.size()<<','<<h<<','<<resident_peak()<<'\n';
    }
}
void panel(const char* dir,U base){
    auto files=getpaths(dir);std::vector<Root> cases;std::vector<std::vector<uint8_t>> refs;
    for(auto f:files){cases.push_back(load(f.c_str()));refs.push_back(direct(cases.back()));}
    for(U i=0;i<10;++i)for(U a=0;a<4;++a){Stats s;assert(run_arm(cases[i],base+a,s)==refs[i]);}
    std::cout<<"cluster,round,block_position,block,position,arm,total_ns,graph_ns,publish_ns,power_ns,emit_ns,nodes\n";
    constexpr U schedule[]={0,1,3,2};
    for(U r=0;r<24;++r){U c=r/4;
        for(U step=0;step<10;++step){U i=(3*r+(c%2?10-step:step))%10;
            for(U p=0;p<4;++p){U j=schedule[p],a=base+(r%4+i+c+(c%2?4-j:j))%4;Stats s;
                auto t=cpu();auto y=run_arm(cases[i],a,s);auto end=cpu();assert(y==refs[i]);
                std::cout<<c<<','<<r<<','<<step<<','<<files[i].parent_path().filename().string()<<','<<p<<','<<arms[a]<<','<<end-t<<','<<s.clocks.graph<<','<<s.clocks.apply<<','<<s.clocks.powers<<','<<s.clocks.emit<<','<<s.nodes<<'\n';
            }
        }
    }
}
int main(int argc,char**argv){
    if(argc==1)small();
    else if(std::string(argv[1])=="check"){assert(argc==4);check(argv[2],argv[3]);}
    else if(std::string(argv[1])=="memory"){assert(argc==4);memory(argv[2],std::stoul(argv[3]));}
    else{assert(argc==3);auto m=std::string(argv[1]);assert(m=="price"||m=="stream");panel(argv[2],m=="stream"?4:0);}
}
