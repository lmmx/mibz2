#include "color.hpp"
#include <fstream>
#include <iostream>
#include <random>
using namespace lf_color;
struct Source{
    std::vector<U> last,psi,lf,levels;
    std::vector<induction::Run> runs;
    explicit Source(std::vector<U> bytes):last(std::move(bytes)),psi(last.size()),lf(psi),levels(psi){
        std::iota(psi.begin(),psi.end(),0);std::stable_sort(psi.begin(),psi.end(),[&](U a,U b){return last[a]<last[b];});
        for(U x=0;x<last.size();x++)lf[psi[x]]=x;
        for(U x=0;x<last.size();){
            U y=x+1;while(y<last.size()&&last[y]==last[x])y++;
            runs.push_back({x,y,lf[x],last[x]});x=y;
        }
        for(U x=0;x<last.size();x++)levels[x]=level(x,lf[x]);
    }
};
struct Audit{W columns=0,maps=0,rows=0,outputs=0,short_laps=0;};
void verify(const Source& a,const Graph& g,const Phases& p,U mark,Audit& audit){
    Map map(g,p,mark);U n=g.n;
    std::vector<U> c(n),survivors;
    for(U x=0;x<n;x++)c[x]=a.levels[x]==no_level?0:((x>>a.levels[x])&1)^p[a.levels[x]];
    U flip=1^c[mark];
    for(U x=0;x<n;x++){
        bool removed=(c[x]^flip)==0&&(c[a.lf[x]]^flip)==1;
        assert(map.rank(x)==survivors.size()&&map.removed(x)==removed);
        if(!removed)survivors.push_back(x);
    }
    assert(survivors.size()==map.kept&&map.rank(n)==map.kept&&!map.removed(mark));
    for(U i=0;i<survivors.size();i++){
        U x=survivors[i];assert(map.select(i)==x);
        auto edge=map.next(i);U y=a.lf[x];
        assert(edge.first==a.last[x]);
        if(map.removed(y)){assert(edge.length==2&&edge.second==a.last[y]);y=a.lf[y];}
        else assert(edge.length==1);
        assert(survivors[edge.target]==y);++audit.rows;
    }
    U m=map.rank(mark),i=m;std::vector<U> lap;
    do{
        auto e=map.next(i);lap.push_back(e.first);if(e.length==2)lap.push_back(e.second);i=e.target;
        assert(lap.size()<=n);
    }while(i!=m);
    std::reverse(lap.begin(),lap.end());U r=mark;
    for(U j=0;j<n;j++){r=a.psi[r];assert(lap[j%lap.size()]==a.last[r]);}
    audit.maps++;audit.outputs+=n;audit.short_laps+=n%lap.size()!=0;
}
void check(Source a,Audit& s){
    auto g=graph(a.runs);Phases greedy=choose(g),best=exact(g);
    std::array<std::array<Pair,32>,32> oracle{};
    for(U x=0;x<g.n;x++){
        U y=a.lf[x],j=a.levels[x],l=a.levels[y];
        if(j==no_level){assert(y==x);continue;}
        auto& e=oracle[std::min(j,l)][std::max(j,l)];++e.mass;
        e.different+=((x>>j)^(y>>l))&1;
    }
    W mass=0;for(U i=0;i<32;i++)for(auto [j,v]:g.edges[i]){
        assert(oracle[j][i].mass==v.mass&&oracle[j][i].different==v.different);mass+=v.mass;
    }
    assert(mass==g.n-g.fixed&&score(g,best)>=score(g,greedy));
    assert(2*score(g,greedy)>=g.n-g.fixed+g.equal);
    std::vector<U> active;for(U i=0;i<32;i++)if(g.active[i])active.push_back(i);
    W exhaustive=0;Phases other{};
    for(U z=0;z<(1u<<active.size());z++){
        for(U k=0;k<active.size();k++)other[active[k]]=(z>>k)&1;
        exhaustive=std::max(exhaustive,score(g,other));
        if(g.n<=8)for(U m=0;m<g.n;m++)verify(a,g,other,m,s);
    }
    assert(exhaustive==score(g,best));
    if(g.n>8)for(U m:{0u,g.n/2,g.n-1})verify(a,g,greedy,m,s);
    ++s.columns;
}
int main(){
    Audit s;
    for(U hi=1;hi<=8;hi++)for(U lo=0;lo<hi;lo++)for(U a:{0u,1u})for(U b:{0u,1u}){
        Mask m{lo,hi,a,b};W count=0;
        for(U x=0;x<1537;x++){
            assert(m.kept(x)==count);
            if(((x>>lo)&1)!=a||((x>>hi)&1)!=b)assert(m.select(count++)==x);
        }
    }
    for(U alphabet:{2u,3u})for(U n=1,ways=alphabet;n<=(alphabet==2?8:6);n++,ways*=alphabet){
        for(U code=0;code<ways;code++){
            U x=code;std::vector<U> bytes(n);for(auto& b:bytes){b=x%alphabet;x/=alphabet;}
            check(Source(std::move(bytes)),s);
        }
    }
    std::mt19937 rng(12851);
    for(U i=0;i<150;i++){std::vector<U> a(9+rng()%128);for(auto& b:a)b=rng()%(i%11+1);check(Source(std::move(a)),s);}
    Source sharp({1,0,1,0});auto g=graph(sharp.runs);
    assert(g.fixed==0&&g.equal==0&&score(g,exact(g))==2);
    std::cout<<"columns "<<s.columns<<" maps "<<s.maps<<" retained_returns "<<s.rows
             <<" marked_forward_bytes "<<s.outputs<<" nondividing_maps "<<s.short_laps<<'\n';
    std::cout<<"two_run_family,m,n,graph_tiles,kept,unit_slope_word_bands\n";
    for(U m=4;m<=32768;m*=2){
        std::vector<U> v(2*m+1);std::fill_n(v.begin(),m,1);
        Source a(std::move(v));auto table=graph(a.runs);auto p=exact(table);Map map(table,p,0);
        assert(score(table,p)==m+2&&map.kept==3*m/2);
        U bands=0;std::array<U,4> prev{};
        for(U i=0;i<map.kept;i++){
            auto q=map.next(i);std::array<U,4> key{q.target-i,q.length,q.first,q.second};
            if(i==0||prev!=key)++bands;
            prev=key;
        }
        for(U x=0;x<table.n;x++)assert(map.removed(x)==(x==m-1||(x>=m&&x<2*m&&x%2==0)));
        assert(bands==map.kept);std::cout<<"two_run_family,"<<m<<','<<table.n<<','<<table.tiles.size()<<','<<map.kept<<','<<bands<<'\n';
    }
}
