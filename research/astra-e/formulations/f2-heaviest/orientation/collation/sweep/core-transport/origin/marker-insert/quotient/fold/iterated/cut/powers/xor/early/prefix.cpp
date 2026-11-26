#include "../capped/bridge.cpp"

struct Prefix {
    Root state;
    std::vector<Node> grammar;
    U horizon,rows;
    Prefix(const uint16_t* caps,U count,U n,U mark):horizon(n),rows(n) {
        auto original=from_caps(caps,count,n,mark);
        state=seed(original);
        std::array<U,256> ids{};
        for(auto k:original.bytes) {
            ids[k]=grammar.size();grammar.push_back({1,k,0,Node::Literal});
        }
        for(auto& k:state.bytes)k=ids[k];
    }
    bool advance() {
        StreamCut<true> cut(state);
        if(cut.done)return false;
        if(4*cut.ingress<cut.m) {
            auto normalized=interval_seeded(state);
            Root macro;macro.r=std::move(normalized.r);macro.mark=normalized.mark;
            std::map<std::pair<U,U>,U> powers;
            for(auto sym:normalized.words) {
                U node=sym.base;
                if(sym.n>1) {
                    auto [it,fresh]=powers.emplace(std::make_pair(node,sym.n),U(grammar.size()));
                    if(fresh)grammar.push_back({grammar[node].n*sym.n,node,sym.n,Node::Power});
                    node=it->second;
                }
                macro.bytes.push_back(node);
            }
            state=std::move(macro);cut=StreamCut<true>(state);
            assert(!cut.done && cut.self==0);
        }
        assert(4*cut.ingress>=cut.m);
        rows=cut.m-cut.ingress;state=cut.apply(state,grammar);
        return true;
    }
};
// Each surviving type is materialized once. At depth d its total dictionary
// length is at most the weighted row sum because each type has positive mass.
void spell(const std::vector<Node>& grammar,U node,std::vector<uint8_t>& dst) {
    auto v=grammar[node];
    if(v.kind==Node::Literal){dst.push_back(v.l);return;}
    if(v.kind==Node::Join){spell(grammar,v.l,dst);spell(grammar,v.r,dst);return;}
    auto offset=dst.size();spell(grammar,v.l,dst);
    for(U i=1;i<v.r;++i) {
        auto size=dst.size();U n=grammar[v.l].n;
        dst.resize(size+n);std::memcpy(dst.data()+size,dst.data()+offset,n);
    }
}
struct Geometry { U rows,runs,types,mark_class,mark_row,max_weight,grammar_count,dict_bytes,weighted_bytes; };
Geometry describe(const Prefix& p) {
    auto& s=p.state;auto b=bases(s);U sum=0,w=0,weighted=0;
    for(U k=0;k<s.bytes.size();++k) {
        U len=p.grammar[s.bytes[k]].n;sum+=len;w=std::max(w,len);weighted+=(b[k+1]-b[k])*len;
    }
    return {p.rows,U(s.r.size()),U(s.bytes.size()),s.mark,b[s.mark],w,U(p.grammar.size()),sum,weighted};
}
template<typename Link> std::vector<uint8_t> residual(const Prefix& p) {
    auto& s=p.state;
    auto b=bases(s),cursor=b;
    std::vector<Link> arc(p.rows);U dst=0;
    constexpr U bits=20;assert(p.rows<1<<bits);constexpr U mask=(1<<bits)-1;
    for(auto r:s.r) {
        auto j=cursor[r.k];
        for(U i=0;i<r.n;++i)arc[j++]=(Link(r.k)<<bits)|Link(dst++);
        cursor[r.k]=j;
    }
    std::vector<U> offset{0};offset.reserve(s.bytes.size()+1);
    for(auto node:s.bytes)offset.push_back(offset.back()+p.grammar[node].n);
    std::vector<uint8_t> pool;pool.reserve(offset.back());
    for(auto node:s.bytes)spell(p.grammar,node,pool);
    std::vector<uint8_t> out(p.horizon);U row=b[s.mark],at=0;
    while(at<p.horizon) {
        auto a=arc[row];U k=U(a>>bits);row=U(a)&mask;
        U w=std::min(offset[k+1]-offset[k],p.horizon-at);
        std::memcpy(out.data()+at,pool.data()+offset[k],w);at+=w;
    }
    return out;
}
std::vector<uint8_t> residual(const Prefix& p) {
    return p.state.bytes.size()<=4096?residual<uint32_t>(p):residual<uint64_t>(p);
}
extern "C" U prefix_chain(const uint16_t* caps,U count,U n,U mark,uint64_t* stamp,Geometry* geo,const uint8_t* reference) {
    const uint64_t start=cpu();
    Prefix p(caps,count,n,mark);
    U depth=0;
    for(;;) {
        assert(depth<64);stamp[depth]=cpu()-start;
        if(reference) {
            geo[depth]=describe(p);
            auto out=residual(p);assert(std::memcmp(out.data(),reference,n)==0);
        }
        ++depth;if(!p.advance())break;
    }
    return depth;
}
extern "C" const uint8_t* prefix_finish(Output* state,const uint16_t* caps,U count,U n,U mark,U d) {
    Prefix p(caps,count,n,mark);
    for(U t=0;t<d;++t)if(!p.advance())break;
    state->data=residual(p);return state->data.data();
}
