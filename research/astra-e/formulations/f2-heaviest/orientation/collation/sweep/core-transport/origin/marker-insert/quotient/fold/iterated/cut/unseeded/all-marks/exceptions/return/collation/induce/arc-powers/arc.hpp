#pragma once
#include "../modular/modular.hpp"
namespace arc_cut {
using namespace modular;
// Arc IDs in each exchange are in increasing source order.
struct Node {U left,right,copies,length;};
struct Grammar{
    std::vector<Node> nodes;
    U len(U id)const{return id<256?1:nodes[id-256].length;}
    U repeat(U id,U count){
        if(count==1)return id;
        U out=256+nodes.size();nodes.push_back({id,none,count,len(id)*count});return out;
    }
    U join(U a,U b){
        U out=256+nodes.size();nodes.push_back({a,b,1,len(a)+len(b)});return out;
    }
    void emit(U root,std::vector<unsigned char>& out,U horizon)const{
        struct Work{U id,copies;};
        std::vector<Work> stack{{root,1}};
        while(!stack.empty()){
            auto [id,copies]=stack.back();stack.pop_back();
            if(id<256){out.insert(out.end(),copies,static_cast<unsigned char>(id));continue;}
            if(copies>1)stack.push_back({id,copies-1});
            auto n=nodes[id-256];
            if(n.right==none)stack.push_back({n.left,n.copies});
            else{stack.push_back({n.left,1});stack.push_back({n.right,1});}
        }
        U lap=out.size();assert(lap&&lap<=horizon);
        out.resize(horizon);
        for(U x=lap;x<horizon;){U w=std::min(x,horizon-x);std::copy_n(out.data(),w,out.data()+x);x+=w;}
    }
};
struct Layer {U n=0,b=0,dead=0,powered=0,power_rows=0,power_arcs=0,diagonal=0,pairs=0,cut=0,triples=0,final_arcs=0,radix_visits=0;};
inline std::vector<U> bases(const Exchange& e){
    U y=0;std::vector<U> dest(e.arcs.size());for(U i:e.bottom){dest[i]=y;y+=e.arcs[i].width;}return dest;
}
inline Exchange isolate(const Exchange& e,U mark){
    Exchange r;std::vector<U> start(e.arcs.size()),count(e.arcs.size());U x=0;
    for(U i=0;i<e.arcs.size();i++){
        Arc a=e.arcs[i];start[i]=r.arcs.size();
        if(x<=mark&&mark<x+a.width){
            if(mark>x)r.arcs.push_back({mark-x,a.rule});
            r.arcs.push_back({1,a.rule});
            if(mark+1<x+a.width)r.arcs.push_back({x+a.width-mark-1,a.rule});
        }else r.arcs.push_back(a);
        count[i]=r.arcs.size()-start[i];x+=a.width;
    }
    for(U i:e.bottom)for(U j=0;j<count[i];j++)r.bottom.push_back(start[i]+j);
    return r;
}
inline Exchange powers(const Exchange& e,U& mark,Grammar& g,Layer& l,U& terminal){
    auto dest=bases(e);U x=0,kept=0,newmark=none;
    Exchange next;
    struct Group{U first=0,count=0;bool reverse=false;};
    std::vector<Group> order(e.arcs.size());
    for(U i=0;i<e.arcs.size();++i){
        auto a=e.arcs[i];U w=a.width;
        int64_t d=int64_t(dest[i])-x;
        if(d==0){
            if(mark==x){assert(w==1);terminal=a.rule;return {};}
            l.dead+=w;x+=w;continue;
        }
        if(mark==x){assert(w==1);newmark=kept;}
        U delta=U(d>0?d:-d),width=std::min(w,delta);
        U offset=d>0?0:w-width,previous=0;
        auto& group=order[i];group.first=next.arcs.size();
        for(U k=0;k<width;){
            U t=offset+k;
            U h=d>0?1+(w-1-t)/delta:1+t/delta;
            U available=d>0?w-(h-1)*delta-t:U(uint64_t(h)*delta-t);
            U step=std::min(width-k,available);assert(step);
            U y=U(int64_t(x+t)+int64_t(h)*d);
            assert(dest[i]<=y&&y+step<=dest[i]+w);
            group.reverse|=group.count&&y<previous;previous=y;++group.count;
            next.arcs.push_back({step,g.repeat(a.rule,h)});k+=step;
        }
        assert(group.count<=2);kept+=width;l.powered+=w-width;x+=w;
    }
    for(U i:e.bottom){
        auto v=order[i];for(U j=0;j<v.count;++j)
            next.bottom.push_back(v.first+(v.reverse?v.count-1-j:j));
    }
    assert(newmark!=none);mark=newmark;l.power_rows=kept;l.power_arcs=next.arcs.size();return next;
}
struct Edge{U source,middle,width,i,j,joined=none;};
struct Fragment{U source,target,width,rule;};
template<class F> std::vector<U> radix(U count,U maximum,F key,U& visits){
    std::vector<U> v(count),out(count);std::iota(v.begin(),v.end(),0);
    for(U shift=0;shift<32&&(maximum>>shift);shift+=8){
        std::array<U,256> buckets{};
        for(U i:v)++buckets[(key(i)>>shift)&255];
        U sum=0;for(auto& c:buckets){U a=c;c=sum;sum+=a;}
        for(U i:v)out[buckets[(key(i)>>shift)&255]++]=i;
        visits+=count;v.swap(out);
    }
    return v;
}
inline U color_mask(const std::vector<U>& hist){
    U mask=0;
    for(U bit=1;bit<hist.size();bit*=2){
        uint64_t same=0,opposite=0;
        for(U v=bit;v<2*bit;++v){
            if(__builtin_parity(v&mask))opposite+=hist[v];else same+=hist[v];
        }
        if(same>opposite)mask|=bit;
    }
    return mask;
}
inline Exchange cut(const Exchange& e,U& mark,Grammar& g,Layer& l){
    U m=l.power_rows,B=e.arcs.size(),domain=1;while(domain<B)domain*=2;
    auto dest=bases(e);std::vector<U> src(B),hist(domain);
    U x=0,mark_id=none;for(U i=0;i<B;++i){src[i]=x;if(x==mark)mark_id=i;x+=e.arcs[i].width;}
    assert(mark_id!=none&&e.arcs[mark_id].width==1);
    std::vector<Edge> edges;
    U y=0,a=0,j=0;
    while(y<m){
        U i=e.bottom[a],end1=dest[i]+e.arcs[i].width,end2=src[j]+e.arcs[j].width;
        U stop=std::min(end1,end2);edges.push_back({src[i]+y-dest[i],y,stop-y,i,j});
        hist[i^j]+=stop-y;y=stop;if(y==end1)++a;if(y==end2)++j;
    }
    l.pairs=edges.size();l.diagonal=hist[0];assert(!l.diagonal);
    U mask=color_mask(hist);
    bool flip=!__builtin_parity(mark_id&mask);
    auto side=[&](U i){return bool(__builtin_parity(i&mask))^flip;};
    auto removed=[&](const Edge& v){return !side(v.i)&&side(v.j);};
    auto by_source=radix(edges.size(),m-1,[&](U i){return edges[i].source;},l.radix_visits);
    std::vector<Fragment> frags;
    U t=0,h=0;x=0;
    while(x<m){
        auto& in=edges[t];auto& out=edges[by_source[h]];
        U right_in=in.middle+in.width,right_out=out.source+out.width;
        U stop=std::min(right_in,right_out),w=stop-x;++l.triples;
        if(!removed(in)){
            U target=x,word=e.arcs[in.i].rule;
            if(removed(out)){
                target=out.middle+x-out.source;
                if(in.joined==none)in.joined=g.join(word,e.arcs[in.j].rule);
                word=in.joined;
            }
            frags.push_back({in.source+x-in.middle,target,w,word});
        }else l.cut+=w;
        x=stop;if(x==right_in)++t;if(x==right_out)++h;
    }
    assert(4*uint64_t(l.cut)>=m&&l.cut<m);
    auto order=radix(frags.size(),m-1,[&](U i){return frags[i].source;},l.radix_visits);
    std::vector<Fragment> strips;U newmark=none,pos=0;
    for(U z:order){
        auto v=frags[z];
        if(v.source==mark){assert(v.width==1);newmark=pos;}
        pos+=v.width;
        if(!strips.empty()&&v.source!=mark&&strips.back().source!=mark){
            auto& b=strips.back();
            if(b.rule==v.rule&&b.source+b.width==v.source&&b.target+b.width==v.target){b.width+=v.width;continue;}
        }
        strips.push_back(v);
    }
    assert(newmark!=none);mark=newmark;Exchange next;
    for(auto v:strips)next.arcs.push_back({v.width,v.rule});
    next.bottom=radix(strips.size(),m-1,[&](U i){return strips[i].target;},l.radix_visits);
    l.final_arcs=next.arcs.size();return next;
}
inline U solve(Exchange e,U p,Grammar& g,std::vector<Layer>& rows){
    e=isolate(e,p);U n=0;for(auto v:e.arcs)n+=v.width;
    for(;;){
        Layer l;l.n=n;l.b=e.arcs.size();U terminal=none;
        auto power=powers(e,p,g,l,terminal);
        if(terminal!=none){rows.push_back(l);return terminal;}
        e=cut(power,p,g,l);assert(l.n>=l.power_rows&&l.power_rows>l.cut);
        n=l.power_rows-l.cut;rows.push_back(l);
    }
}
}
