#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <limits>
#include <numeric>
#include <string>
#include <vector>

using U=uint32_t;
using Byte=uint8_t;
struct Run { U width; Byte symbol; };
struct Input {
    std::vector<Run> runs;
    U primary=0,n=0;
};
struct Geometry {
    std::array<U,257> base{};
    std::vector<Byte> active;
    explicit Geometry(const Input& in) {
        for(auto [n,a]:in.runs)base[U(a)+1]+=n;
        for(U a=0;a<256;++a)if(base[a+1])active.push_back(Byte(a));
        std::partial_sum(base.begin(),base.end(),base.begin());
        assert(base.back()==in.n);
    }
};
// Exclusive physical endpoints of prefix intervals, grouped by the initial byte.
struct Partition {
    std::array<std::vector<U>,256> end;
    U size=0;
    void add(Byte c,U right){end[c].push_back(right);++size;}
    size_t bytes(const Geometry& g) const {
        size_t amount=0;
        for(auto c:g.active)amount+=end[c].capacity()*sizeof(U);
        return amount;
    }
};
struct Cursor {U run,right;};
template<class Visit> void intersect(const Input& raw,Cursor& walk,U x,U stop,Visit visit) {
    while(x<stop) {
        auto r=raw.runs[walk.run];U y=std::min(stop,walk.right);
        visit(r.symbol,x,y);x=y;
        if(x==walk.right && ++walk.run<raw.runs.size())walk.right+=raw.runs[walk.run].width;
    }
}
struct Exceptions {
    std::vector<U> begin,end,next;
    std::vector<Byte> symbol;
    void append(U x,U y,Byte a,U post) {
        if(!end.empty()&&x==end.back()&&a==symbol.back()&&post==next.back()) {end.back()=y;return;}
        begin.push_back(x);end.push_back(y);symbol.push_back(a);next.push_back(post);
    }
    size_t bytes() const {return 4*(begin.capacity()+end.capacity()+next.capacity())+symbol.capacity();}
};
struct Model {
    Partition states;
    std::vector<Byte> modal;
    std::vector<U> next;
    Exceptions exceptions;
    U primary_state=0,edges=0,exception_rows=0;
};
struct Layer {
    U depth=0,states=0,intersections=0,edges=0,modal_revisit=0,pure=0;
    U exception_slices=0,post_advances=0;
    size_t initial_bytes=0,final_bytes=0;
};
struct Charge {
    std::vector<Layer> layers;
    size_t peak_dynamic=0;
};
template<bool Final,bool Account>
Partition step(const Input& raw,const Geometry& geom,const Partition& old,U depth,Model& model,Charge& cost) {
    Partition built;
    // Alphabet initialization is O(256); each context clears only its touched counts.
    std::array<U,256> hist{},after{},first_id{},state_cursor{},post{};
    std::array<Byte,256> keys;
    Layer work;work.depth=depth;work.states=old.size;
    if constexpr(Account)work.initial_bytes=old.bytes(geom);
    U ids=0;
    for(Byte a:geom.active) {
        first_id[a]=ids;ids+=old.end[a].size();after[a]=geom.base[a];
    }
    if constexpr(Final){model.modal.reserve(ids);model.next.reserve(ids);}
    Cursor scan{0,raw.runs.front().width};U x=0,state=0;
    for(Byte lead:geom.active)for(U right:old.end[lead]) {
        Cursor replay=scan;U distinct=0;
        intersect(raw,scan,x,right,[&](Byte a,U s,U e){
            if(!hist[a])keys[distinct++]=a;
            hist[a]+=e-s;
            if constexpr(Account)++work.intersections;
        });
        assert(distinct);
        U best=0;Byte mode=0;
        for(U i=0;i<distinct;++i){
            Byte a=keys[i];U width=hist[a];hist[a]=0;
            U end=after[a]+width;
            if constexpr(Final) {
                auto j=state_cursor[a];
                assert(j<old.end[a].size()&&end<=old.end[a][j]);
                post[a]=first_id[a]+j;
                if(end==old.end[a][j]) {
                    ++state_cursor[a];
                    if constexpr(Account)++work.post_advances;
                }
                if(width>best||(width==best&&a<mode)){mode=a;best=width;}
            } else {built.add(a,end);}
            after[a]=end;
        }
        if constexpr(Account){work.edges+=distinct;work.pure+=distinct==1;}
        if constexpr(Final) {
            model.edges+=distinct;model.modal.push_back(mode);model.next.push_back(post[mode]);
            if(x<=raw.primary&&raw.primary<right)model.primary_state=state;
            model.exception_rows+=right-x-best;
            if(distinct>1)intersect(raw,replay,x,right,[&](Byte a,U s,U e){
                if constexpr(Account)++work.modal_revisit;
                if(a!=mode) {
                    if constexpr(Account)++work.exception_slices;
                    model.exceptions.append(s,e,a,post[a]);
                }
            });
        }
        ++state;x=right;
    }
    assert(x==raw.n&&scan.run==raw.runs.size());
    for(Byte a:geom.active)assert(after[a]==geom.base[U(a)+1]);
    if constexpr(Account) {
        work.final_bytes=Final?model.modal.capacity()+4*model.next.capacity()+model.exceptions.bytes():built.bytes(geom);
        cost.peak_dynamic=std::max(cost.peak_dynamic,work.initial_bytes+work.final_bytes);
        cost.layers.push_back(work);
    }
    return built;
}
template<bool Account=false> Model construct(const Input& raw,U k,Charge& cost) {
    assert(k&&raw.n&&raw.primary<raw.n);
    Geometry g(raw);Partition root;
    for(Byte a:g.active)root.add(a,g.base[U(a)+1]);
    Model model;
    for(U depth=1;depth<k;++depth) {
        auto finer=step<false,Account>(raw,g,root,depth,model,cost);
        bool stable=finer.size==root.size;
        root=std::move(finer);
        if(stable)break;
    }
    step<true,Account>(raw,g,root,k,model,cost);
    model.states=std::move(root);
    if constexpr(Account)cost.peak_dynamic+=g.active.capacity();
    return model;
}
inline Input read_input(const char* path) {
    std::ifstream f(path);Input in;U a,n;
    if(!(f>>in.primary))std::abort();
    while(f>>a>>n){
        assert(a<256&&n&&in.n<=std::numeric_limits<U>::max()-n);
        if(!in.runs.empty()&&in.runs.back().symbol==a)in.runs.back().width+=n;
        else in.runs.push_back({n,Byte(a)});
        in.n+=n;
    }
    assert(in.primary<in.n);return in;
}
