#define POWER_PHASE_LIBRARY
#include "native.cpp"
#include <cstring>
#include <map>

struct Node { U n,l,r; enum Kind {Literal,Join,Power} kind; };
struct Edge { U target,mass,alias; };
struct Slice { U source,target,width,edge; };
struct Clock { uint64_t graph=0,apply=0,powers=0,emit=0; };
struct Pass { U old_m,old_runs,old_types,self,ingress,after,new_runs,new_types; bool normalized; };

struct Cut {
    U m=0,self=0,ingress=0;
    bool done=false;
    std::vector<U> b;
    std::vector<std::vector<Edge>> graph;
    std::vector<Slice> parts;
    std::vector<int8_t> color;
    Cut(const Root& in):b(bases(in)),graph(in.bytes.size()),color(in.bytes.size(),-1) {
        m=b.back();U i=0,y=0;
        // An initial protected fixed point needs no graph or normalization.
        for(auto [k,n]:in.r) {
            if(k==in.mark) {assert(n==1); done=(y==b[k]);break;}
            y+=n;
        }
        if(done) return;
        y=0;
        for(auto [k,n]:in.r) while(n) {
            U width=std::min(n,b[i+1]-y);
            auto& g=graph[k];
            if(g.empty()||g.back().target!=i) g.push_back({i,0,0});
            g.back().mass+=width;
            parts.push_back({k,i,width,U(g.size()-1)});
            if(k==i) self+=width;
            y+=width;n-=width;
            if(y==b[i+1])++i;
        }
        std::vector<std::vector<std::pair<U,U>>> incoming(graph.size());
        for(U k=0;k<graph.size();++k) for(auto e:graph[k]) if(k!=e.target)
            incoming[e.target].push_back({k,e.mass});
        color[in.mark]=0;
        for(U k=0;k<graph.size();++k) if(k!=in.mark) {
            int gain=0;
            auto vote=[&](U t,U mass) {
                if(color[t]>=0) gain+=color[t]?-int(mass):int(mass);
            };
            for(auto e:graph[k])vote(e.target,e.mass);
            for(auto [j,n]:incoming[k])vote(j,n);
            color[k]=gain>0;
        }
        for(U k=0;k<graph.size();++k) if(!color[k])
            for(auto e:graph[k]) if(color[e.target])ingress+=e.mass;
    }
    Root apply(const Root& in,std::vector<Node>& grammar) {
        const U no=UINT32_MAX;
        std::vector<U> first(graph.size()),rem,word,into,target_of;
        for(U k=0;k<graph.size();++k) {
            first[k]=rem.size();
            if(color[k]) {
                rem.push_back(b[k+1]-b[k]);word.push_back(in.bytes[k]);
                into.push_back(no);target_of.push_back(k);
            } else for(auto& e:graph[k]) {
                e.alias=rem.size();rem.push_back(e.mass);word.push_back(in.bytes[k]);
                into.push_back(color[e.target]?e.target:no);target_of.push_back(no);
            }
        }
        std::vector<bool> targeted(graph.size(),false);
        for(U a=0;a<rem.size();++a) if(into[a]!=no) {
            targeted[into[a]]=true;
            rem[first[into[a]]]-=rem[a];
        }
        Root out;std::vector<U> ids(rem.size(),no);
        for(U a=0;a<rem.size();++a) if(rem[a]) {
            ids[a]=out.bytes.size();U w=word[a];
            if(into[a]!=no) {
                U r=in.bytes[into[a]],p=grammar.size();
                grammar.push_back({grammar[w].n+grammar[r].n,w,r,Node::Join});w=p;
            }
            out.bytes.push_back(w);
        }
        out.mark=ids[first[in.mark]];
        std::vector<std::vector<Run>> maps(graph.size());
        auto alias=[&](const Slice& s) {return color[s.source]?first[s.source]:graph[s.source][s.edge].alias;};
        for(auto s:parts) if(targeted[s.target]) {
            U a=alias(s);append(maps[s.target],into[a]!=no?a:first[s.target],s.width);
        }
        std::vector<U> cursor(graph.size()),used(graph.size());
        for(auto s:parts) {
            U a=alias(s);
            if(into[a]!=no) continue;
            U j=target_of[a];
            if(j==no||!targeted[j]) {append(out.r,ids[a],s.width);continue;}
            U n=s.width;
            while(n) {
                auto t=maps[j][cursor[j]];U w=std::min(n,t.n-used[j]);
                assert(ids[t.k]!=no);append(out.r,ids[t.k],w);used[j]+=w;n-=w;
                if(used[j]==t.n){++cursor[j];used[j]=0;}
            }
        }
        return out;
    }
};

struct Stats { Clock clocks; std::vector<Pass> passes; U nodes=0; };
std::vector<uint8_t> grammar_output(const std::vector<Node>& nodes,U root,U horizon,U skip) {
    std::vector<uint8_t> out;out.reserve(horizon);
    std::vector<int> cache(nodes.size(),-1);
    struct Frame {U id,begin;bool close;};
    std::vector<Frame> stack{{root,0,false}};
    auto duplicate=[&](U pos,U n) {
        while(n) {
            U size=out.size(),w=std::min(n,size-pos);
            assert(w);out.resize(size+w);std::memcpy(out.data()+size,out.data()+pos,w);n-=w;
        }
    };
    while(!stack.empty()) {
        auto f=stack.back();stack.pop_back();
        auto n=nodes[f.id];
        if(f.close) {
            if(n.kind==Node::Power) duplicate(f.begin,n.n-(out.size()-f.begin));
            cache[f.id]=f.begin;
        } else if(n.kind==Node::Literal) out.push_back(uint8_t(n.l));
        else if(cache[f.id]>=0) duplicate(U(cache[f.id]),n.n);
        else {
            stack.push_back({f.id,U(out.size()),true});
            if(n.kind==Node::Join)stack.push_back({n.r,0,false});
            stack.push_back({n.l,0,false});
        }
    }
    assert(out.size()==nodes[root].n&&horizon>=out.size()&&skip<out.size());
    std::rotate(out.begin(),out.begin()+skip,out.end());
    duplicate(0,horizon-out.size());
    return out;
}
template<class Builder> std::vector<uint8_t> decode_with(const Root& original,Stats& stats) {
    U horizon=0;for(auto r:original.r)horizon+=r.n;
    auto in=seed(original);
    std::vector<Node> nodes;
    for(auto b:original.bytes)nodes.push_back({1,b,0,Node::Literal});
    // Seeded source aliases share their original byte literal node.
    std::vector<U> index(256);
    for(U i=0;i<original.bytes.size();++i)index[original.bytes[i]]=i;
    for(U& v:in.bytes) v=index[v];
    for(;;) {
        uint64_t t=cpu();Builder cut(in);stats.clocks.graph+=cpu()-t;
        if(cut.done)break;
        Pass pass{cut.m,U(in.r.size()),U(in.bytes.size()),cut.self,cut.ingress,0,0,0,false};
        if(4*cut.ingress<cut.m) {
            t=cpu();auto v=interval_seeded(in);
            Root macro;macro.r=std::move(v.r);macro.mark=v.mark;
            std::map<std::pair<U,U>,U> power;
            for(auto sym:v.words) {
                U node=sym.base;
                if(sym.n>1) {
                    auto key=std::make_pair(node,sym.n);
                    auto [it,fresh]=power.emplace(key,U(nodes.size()));
                    if(fresh) nodes.push_back({nodes[node].n*sym.n,node,sym.n,Node::Power});
                    node=it->second;
                }
                macro.bytes.push_back(node);
            }
            in=std::move(macro);stats.clocks.powers+=cpu()-t;
            t=cpu();cut=Builder(in);stats.clocks.graph+=cpu()-t;
            assert(!cut.done&&cut.self==0);pass.normalized=true;
        }
        assert(4*cut.ingress>=cut.m);
        t=cpu();in=cut.apply(in,nodes);stats.clocks.apply+=cpu()-t;
        pass.after=cut.m-cut.ingress;pass.new_runs=in.r.size();pass.new_types=in.bytes.size();
        stats.passes.push_back(pass);
    }
    stats.nodes=nodes.size();
    auto t=cpu();auto out=grammar_output(nodes,in.bytes[in.mark],horizon,0);
    stats.clocks.emit+=cpu()-t;return out;
}
std::vector<uint8_t> decode(const Root& original,Stats& stats) { return decode_with<Cut>(original,stats); }
#ifndef POWER_SOLVER_LIBRARY
int main(int argc, char** argv) {
    assert(argc>=4); auto in=load(argv[1]);std::string mode=argv[2];
    if(mode=="check") {
        Stats st;auto bytes=decode(in,st);assert(bytes==direct(in));
        std::filesystem::path dir=argv[3];
        std::ofstream out(dir/"cut.bin",std::ios::binary);
        out.write(reinterpret_cast<const char*>(bytes.data()),bytes.size());
        std::ofstream csv(dir/"cut-depths.csv");
        csv<<"depth,old_rows,old_runs,old_ids,self_mass,old_ingress,after,new_runs,new_types,normalized\n";
        U i=0;for(auto p:st.passes)csv<<i++<<','<<p.old_m<<','<<p.old_runs<<','<<p.old_types<<','<<p.self<<','<<p.ingress<<','<<p.after<<','<<p.new_runs<<','<<p.new_types<<','<<p.normalized<<'\n';
    } else {
        int reps=std::stoi(argv[3]);
        std::cout<<"round,arm,thread_ns,graph_ns,cut_ns,norm_ns,emit_ns,bytes,nodes,peak_kib\n";
        for(int i=0;i<reps;++i) for(int slot=0;slot<(mode=="panel"?2:1);++slot) {
            auto arm=mode=="panel"?((i+slot)%2?"direct":"gated"):mode;
            Stats stats;U n=0;auto start=cpu();
            {
                auto out=arm=="direct"?direct(in):decode(in,stats);n=out.size();
                asm volatile("" : : "g"(out.data()) : "memory");
            }
            auto ns=cpu()-start;
            std::cout<<i<<','<<arm<<','<<ns<<','<<stats.clocks.graph<<','<<stats.clocks.apply<<','<<stats.clocks.powers<<','<<stats.clocks.emit<<','<<n<<','<<stats.nodes<<','<<resident_peak()<<'\n';
        }
    }
}

#endif
