// Literal arrays are the sequential reference; the treatment has two intrusive circular lists.
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <vector>
using U=uint32_t;
using Q=uint64_t;
using Pair=std::array<U,2>;
struct Words {
    std::vector<std::array<U,3>> chronological;
    std::vector<Pair> rope;
    U join(U a,U b) {
        U x=0;
        if(!a||!b) x=a|b;
        else if(a<257 && b<257)x=(1u<<30)|(a-1)|((b-1)<<8);
        else if(a<257 && b>>30==1)x=(2u<<30)|(a-1)|((b&65535)<<8);
        else if(a>>30==1 && b<257)x=(2u<<30)|(a&65535)|((b-1)<<16);
        else {x=(3u<<30)|rope.size();rope.push_back({a,b});}
        chronological.push_back({a,b,x});
        return x;
    }
    bool operator==(const Words&)const=default;
};
struct Seed {std::vector<U> w,h;std::array<std::vector<U>,2> o; Words words;};
void remove(std::vector<U>& v,U x) {auto p=std::find(v.begin(),v.end(),x);assert(p!=v.end());v.erase(p);}
struct Plain {
    Seed z;
    U single() {
        auto&[w,h,o,words]=z;U x=o[0].back(),y=o[1].back(),amount=std::min(w[x],w[y]);
        if(x==y) {o[0].pop_back();o[1].pop_back();return amount;}
        bool top_wins=w[x]>w[y];U side=top_wins?1:0;
        U win=top_wins?x:y,lose=top_wins?y:x;
        h[lose]=words.join(h[x],h[y]);
        auto& losing=o[side];losing.pop_back();
        auto it=std::find(losing.begin(),losing.end(),win);assert(it!=losing.end());
        losing.insert(it+1,lose);
        if(w[x]==w[y]) {remove(o[0],y);remove(o[1],y);}
        else w[win]-=amount;
        return amount;
    }
};
struct Totals {
    Q inputs=0,horizons=0,phases=0,primitive=0,self=0,equal=0,top=0,bottom=0;
    Q multi=0,identity=0,identity_multi=0,limit_cut=0,rule_count=0,repeated_full_turns=0;
    U longest=0,longest_repeated=0;
} total;
struct Linked {
    std::vector<Pair> prev,next;
    std::vector<U> w,h;Words words;
    std::array<std::vector<U>,2> orders()const{
        std::array<std::vector<U>,2> out;
        for(U s:{0u,1u}){
            U p=0;
            for(U v=next[0][s];v;v=next[v][s]){
                assert(v<w.size()&&out[s].size()<w.size()&&prev[v][s]==p);
                out[s].push_back(v);p=v;
            }
            assert(prev[0][s]==p);
        }
        return out;
    }
    explicit Linked(const Seed& z):prev(z.w.size()),next(z.w.size()),w(z.w),h(z.h),words(z.words){
        for(U s:{0u,1u}){
            U p=0;
            for(U v:z.o[s]){link(p,v,s);p=v;}
            link(p,0,s);
        }
    }
    void link(U a,U b,U s){next[a][s]=b;prev[b][s]=a;}
    void cut(U v,U s){link(prev[v][s],next[v][s],s);}
    void insert(U winner,U loser,U s){U u=next[winner][s];link(winner,loser,s);link(loser,u,s);}
    struct Step {U width,count,win,identity;};
    Step phase(U budget) {
        U x=prev[0][0],y=prev[0][1];assert(x&&y&&budget>=std::min(w[x],w[y]));
        if(x==y){++total.self;cut(x,0);cut(x,1);return {w[x],1,0,0};}
        if(w[x]==w[y]){
            ++total.equal;h[x]=words.join(h[x],h[y]);
            cut(x,0);insert(y,x,0);cut(y,0);cut(y,1);
            return {w[x],1,0,0};
        }
        U side=w[x]>w[y]?1:0, win=side?x:y, tail=side?y:x;
        U lo=0, sum=0, count=0;
        for(U at=tail;at!=win;at=prev[at][side]){
            if(w[at]>=w[win]-sum)break;
            if(w[at]>budget-sum){++total.limit_cut;break;}
            // Encounter order follows the old tail towards the pivot.
            h[at]=side?words.join(h[win],h[at]):words.join(h[at],h[win]);
            sum+=w[at];lo=at;++count;
        }
        assert(count&&lo);
        U before=prev[lo][side],following=next[win][side];
        bool identity=before==win;
        if(!identity){link(before,0,side);link(win,lo,side);link(tail,following,side);}
        w[win]-=sum;
        total.top+=side;total.bottom+=!side;total.multi+=count>1;
        total.identity+=identity;total.identity_multi+=identity&&count>1;
        total.longest=std::max(total.longest,count);
        return {sum,count,win,identity};
    }
};
void check(const Seed& z) {
    ++total.inputs;
    const U mass=std::accumulate(z.w.begin(),z.w.end(),0u);
    Plain discover{z};std::vector<U> stops{0};
    while(stops.back()<mass)stops.push_back(stops.back()+discover.single());
    assert(stops.back()==mass);
    for(U budget:stops) {
        Plain scalar{z};Linked linked(z);U used=0,last=0,consecutive=0;
        ++total.horizons;
        while(used<budget){
            auto step=linked.phase(budget-used);
            U amount=0;
            for(U i=0;i<step.count;++i)amount+=scalar.single();
            assert(amount==step.width&&amount<=budget-used);used+=amount;
            assert(linked.orders()==scalar.z.o && linked.h==scalar.z.h && linked.w==scalar.z.w);
            assert(linked.words==scalar.z.words);
            ++total.phases;total.primitive+=step.count;
            if(step.identity){
                consecutive=step.win==last?consecutive+1:1;
                total.repeated_full_turns+=consecutive>=2;
                total.longest_repeated=std::max(total.longest_repeated,consecutive);last=step.win;
            }else{last=consecutive=0;}
        }
        assert(used==budget);
        total.rule_count+=linked.words.rope.size()-z.words.rope.size();
    }
}
void example(const std::vector<U>& widths,const std::vector<U>& bottom,U case_id){
    U n=widths.size();
    Seed z;z.w={0};z.w.insert(z.w.end(),widths.begin(),widths.end());
    z.o[0].resize(n);std::iota(z.o[0].begin(),z.o[0].end(),1);z.o[1]=bottom;z.h.assign(n+1,0);
    // Starting handles include a genuine pre-existing concat as well as empty/aliased words.
    z.words.rope.push_back({(2u<<30)|0x00fedcba,230});
    for(U layout=0;layout<3;++layout){
        for(U v=1;v<=n;++v){
            if(layout==0)z.h[v]=v;
            else if(layout==1)z.h[v]=((v+case_id)%3)?0:256;
            else z.h[v]=std::array<U,6>{0,256,3,(1u<<30)|0xff00,(2u<<30)|0xabcdef,3u<<30}[(case_id+v)%6];
        }
        check(z);
    }
}
int main(){
    for(U n=1;n<=5;++n){
        U classes=n<5?4:2, configs=1;for(U i=0;i<n;++i)configs*=classes;
        std::vector<U> bottom(n);std::iota(bottom.begin(),bottom.end(),1);
        do{for(U code=0;code<configs;++code){
            U v=code;std::vector<U> w(n);for(U& q:w){q=1+v%classes;v/=classes;}
            example(w,bottom,code);
        }}while(std::next_permutation(bottom.begin(),bottom.end()));
    }
    example({1,2,3,98},{4,3,2,1},7); // multiple complete turns of a top winner
    example({98,1,2,3},{2,3,4,1},2); // mirrored bottom winner
    std::vector<U> many(24,1), reverse(24);
    std::iota(reverse.rbegin(),reverse.rend(),1);
    many.back()=99;
    example(many,reverse,4); // 23 distinct joins in one full turn, then a partial rotation
    std::rotate(reverse.begin(),reverse.begin()+3,reverse.end());
    example(many,reverse,0); // nonempty prefix and proper suffix splice
    assert(total.longest>=23);
    assert(total.identity_multi&&total.repeated_full_turns&&total.equal&&total.self&&total.limit_cut);
    std::cout<<"input_layouts,legal_horizons,group_phases,primitive_steps,self_cuts,equalities,top_groups,bottom_groups,multiloser_groups,whole_turns,whole_multiloser_turns,budget_stops,rule_appends_across_horizons,repeated_turns,longest_group,longest_repeated_turns\n"
        <<total.inputs<<','<<total.horizons<<','<<total.phases<<','<<total.primitive<<','<<total.self<<','<<total.equal<<','<<total.top<<','<<total.bottom<<','<<total.multi<<','<<total.identity<<','<<total.identity_multi<<','<<total.limit_cut<<','<<total.rule_count<<','<<total.repeated_full_turns<<','<<total.longest<<','<<total.longest_repeated<<'\n';
}
