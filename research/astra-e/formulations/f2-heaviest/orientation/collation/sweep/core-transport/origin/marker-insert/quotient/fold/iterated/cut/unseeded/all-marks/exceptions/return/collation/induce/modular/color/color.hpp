#pragma once
#include "../../induce.hpp"
#include <bit>
#include <cstdint>
namespace lf_color {
using induction::U;
using W=uint64_t;
using Phases=std::array<uint8_t,32>;
constexpr U no_level=32;
struct Tile{U x,y,width,j,l,byte;};
struct Pair{W mass=0,different=0;};
struct Bin{U lower;Pair value;};
struct Graph{
    // edges[high] stores only observed pairs, including its diagonal.
    std::array<std::vector<Bin>,32> edges;
    std::array<bool,32> active{};
    std::vector<Tile> tiles;
    U n=0,fixed=0,equal=0;
};
inline U level(U x,U y){return x==y?no_level:U(std::countr_zero(x>y?x-y:y-x));}
inline W ones(W n,U bit){
    W h=W(1)<<bit,p=2*h;
    return n/p*h+(n%p>h?n%p-h:0);
}
inline W joint(W n,U lo,U a,U hi,U b){
    // Count z in [0,n) with the two given bits. Every high half-period
    // consists of whole low periods, including the start of its second half.
    W h=W(1)<<hi,rem=n%(2*h),at=b?h:0;
    W t=rem>at?std::min(h,rem-at):0;
    return n/(2*h)*(h/2)+(a?ones(t,lo):t-ones(t,lo));
}
inline void add(Graph& g,Tile t){
    g.tiles.push_back(t);
    if(t.j==no_level){assert(t.l==no_level);g.fixed+=t.width;return;}
    assert(t.l!=no_level);
    U lo=std::min(t.j,t.l),hi=std::max(t.j,t.l);
    g.active[lo]=true;g.active[hi]=true;
    if(lo==hi){g.equal+=t.width;g.edges[hi].push_back({lo,{t.width,t.width}});return;}
    U z=t.j<t.l?t.y:t.x;
    auto diff=[&](W n){return joint(n,lo,0,hi,1)+joint(n,lo,1,hi,0);};
    W unequal=diff(W(z)+t.width)-diff(z);
    g.edges[hi].push_back({lo,{t.width,t.j<t.l?t.width-unequal:unequal}});
}
// On an original LF, images run monotonically through each F-byte bucket.
// Initializing all bucket cursors costs one pass through the source endpoints.
inline Graph graph(const std::vector<induction::Run>& runs){
    Graph g;assert(!runs.empty());g.n=runs.back().end;
    std::array<U,256> mass{},at{};
    for(auto r:runs)mass[r.byte]+=r.end-r.begin;
    U base=0,cursor=0;
    for(U a=0;a<256;a++){
        if(mass[a]){while(runs[cursor].end<=base)++cursor;at[a]=cursor;}
        base+=mass[a];
    }
    for(auto r:runs){
        U s=level(r.begin,r.base);
        U& q=at[r.byte];
        for(U x=r.begin;x<r.end;){
            U y=r.base+(x-r.begin);while(runs[q].end<=y)++q;
            auto target=runs[q];assert(target.begin<=y);
            U w=std::min(r.end-x,target.end-y);
            add(g,{x,y,w,s,level(target.begin,target.base),r.byte});x+=w;
        }
    }
    // Bucket on the higher level during the sweep. Coalesce equal lower
    // levels by resetting only the touched bins, retaining O(R+w) work.
    std::array<Pair,32> scratch{};std::vector<U> touched;
    for(auto& row:g.edges){
        auto parts=std::move(row);touched.clear();
        for(auto b:parts){
            auto& sum=scratch[b.lower];if(!sum.mass)touched.push_back(b.lower);
            sum.mass+=b.value.mass;sum.different+=b.value.different;
        }
        for(U lo:touched){row.push_back({lo,scratch[lo]});scratch[lo]={};}
    }
    assert(g.tiles.size()<=2*runs.size()-1);return g;
}
inline W score(const Graph& g,const Phases& phase){
    W total=0;
    for(U i=0;i<32;i++)for(auto [j,e]:g.edges[i]){
        total+=phase[i]==phase[j]?e.different:e.mass-e.different;
    }
    return total;
}
inline Phases choose(const Graph& g){
    Phases p{};
    for(U i=0;i<32;i++)if(g.active[i]){
        W equal=0,flip=0;
        for(auto [j,e]:g.edges[i])if(j!=i){
            equal+=p[j]?e.mass-e.different:e.different;
            flip+=p[j]?e.different:e.mass-e.different;
        }
        p[i]=flip>equal;
    }
    return p;
}
inline Phases exact(const Graph& g){
    // Offline audit only. Gray-code enumeration with the first level fixed.
    Phases p{},best{};std::vector<U> ids;
    for(U i=0;i<32;i++)if(g.active[i])ids.push_back(i);
    if(ids.empty())return best;
    assert(ids.size()<=20);
    std::array<std::vector<std::pair<U,int64_t>>,32> adjacent;
    for(U a:ids)for(auto [b,e]:g.edges[a])if(a!=b){
        int64_t bias=int64_t(e.mass)-2*int64_t(e.different);
        adjacent[a].push_back({b,bias});adjacent[b].push_back({a,bias});
    }
    int64_t current=score(g,p),peak=current;
    for(U step=1;step<(U(1)<<(ids.size()-1));step++){
        U a=ids[1+std::countr_zero(step)];
        for(auto [b,gain]:adjacent[a]){
            current+=p[a]==p[b]?gain:-gain;
        }
        p[a]^=1;
        if(current>peak){peak=current;best=p;}
    }
    assert(W(peak)==score(g,best));return best;
}
struct Mask {
    U lo=no_level,hi=no_level,a=0,b=0;
    W holes(W n) const{
        if(lo==no_level)return 0;
        if(hi!=no_level)return joint(n,lo,a,hi,b);
        return a?ones(n,lo):n-ones(n,lo);
    }
    W kept(W n)const{return n-holes(n);}
    static W insert(W rank,U bit,U val){
        W m=(W(1)<<bit)-1;return (rank&m)|((rank&~m)<<1)|(W(val)<<bit);
    }
    W select(W ord)const{
        if(lo==no_level)return ord;
        if(hi==no_level)return insert(ord,lo,a^1);
        W h=W(1)<<hi,quota=h+h/2,period=ord/quota,k=ord%quota;
        W first=b?h:h/2;U half=k>=first;if(half)k-=first;
        return period*2*h+half*h+(half==b?insert(k,lo,a^1):k);
    }
};
struct Piece{Mask mask;U base,end;};
struct Map{
    const Graph& g;
    std::vector<Piece> data;
    U kept=0;
    size_t source(U x)const{
        return std::upper_bound(g.tiles.begin(),g.tiles.end(),x,
            [](U v,const Tile& t){return v<t.x+t.width;})-g.tiles.begin();
    }
    Map(const Graph& graph,const Phases& p,U mark):g(graph){
        assert(mark<g.n);
        auto tmark=g.tiles[source(mark)];
        U original=tmark.j==no_level?0:((mark>>tmark.j)&1)^p[tmark.j];
        U flip=1^original;
        data.reserve(g.tiles.size());
        for(auto t:g.tiles){
            U base=t.x;Mask m;
            if(t.j!=no_level){
                U pj=p[t.j]^flip,pl=p[t.l]^flip;
                if(t.j==t.l)m={t.j,no_level,pj,0};
                else if(t.j<t.l){base=t.y;m={t.j,t.l,pj^1,pl^1};}
                else m={t.l,t.j,pl^1,pj};
            }
            kept+=m.kept(W(base)+t.width)-m.kept(base);
            data.push_back({m,base,kept});
        }
        assert(2*(g.n-kept)==score(g,p));
    }
    U rank(U x)const{
        if(x==g.n)return kept;
        auto i=source(x);auto t=g.tiles[i];auto d=data[i];
        return (i?data[i-1].end:0)+d.mask.kept(W(d.base)+x-t.x)-d.mask.kept(d.base);
    }
    bool removed(U x)const{
        auto i=source(x);auto t=g.tiles[i];auto d=data[i];
        W z=W(d.base)+x-t.x;
        if(d.mask.lo==no_level)return false;
        return ((z>>d.mask.lo)&1)==d.mask.a&&(d.mask.hi==no_level||((z>>d.mask.hi)&1)==d.mask.b);
    }
    U select(U ord)const{
        assert(ord<kept);
        auto i=std::upper_bound(data.begin(),data.end(),ord,
            [](U v,const Piece& d){return v<d.end;})-data.begin();
        auto d=data[i];U local=ord-(i?data[i-1].end:0);
        return g.tiles[i].x+d.mask.select(d.mask.kept(d.base)+local)-d.base;
    }
    struct Return{U target,length,first,second;};
    Return next(U ord)const{
        U x=select(ord);auto t=g.tiles[source(x)];U y=t.y+x-t.x;
        if(!removed(y))return {rank(y),1,t.byte,0};
        auto t2=g.tiles[source(y)];U z=t2.y+y-t2.x;assert(!removed(z));
        return {rank(z),2,t.byte,t2.byte};
    }
};
}
