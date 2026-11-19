#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <numeric>
#include <string>
#include <vector>
using U=uint32_t;
struct Rates {int a,b;};
constexpr std::array<Rates,5> rates={{{1,4},{1,2},{2,3},{1,1},{2,1}}};
struct Best {int score=0;U seams=0;};
struct Summary {
    uint64_t primitive=0;
    std::array<uint64_t,5> strictwins{},weakwins{},isolated{};
    std::array<std::string,5> examples;
};
unsigned step(U bit){return __builtin_ctz(bit);}
void examine(const std::vector<int>& last,int sigma,Summary& out) {
    int n=last.size();std::vector<int> freq(sigma),base(sigma),next(sigma),lf(n),psi(n),f(n);
    for(int a:last)++freq[a];if(*std::min_element(freq.begin(),freq.end())==0)return;
    std::partial_sum(freq.begin(),freq.end()-1,base.begin()+1);
    next=base;
    for(int x=0;x<n;++x){int y=next[last[x]]++;lf[x]=y;psi[y]=x;f[y]=last[x];}
    int y=lf[0],d=1;while(y){y=lf[y];++d;}if(d!=n)return;
    ++out.primitive;
    U original=0;std::vector<int> pos;for(int t=1;t<n;++t) {
        if(f[t]!=f[t-1])original|=1u<<t;else pos.push_back(t);
    }
    int h=pos.size();U limit=1u<<h;
    std::vector<U> support(h);U rooted=0,self=0;
    for(int i=0;i<h;++i) {
        int t=pos[i],lo=psi[t-1],hi=psi[t];assert(lo<hi);
        U cuts=(1u<<(hi+1))-(1u<<(lo+1));if(cuts&original)rooted|=1u<<i;
        for(int j=0;j<h;++j)if(cuts&(1u<<pos[j]))support[i]|=1u<<j;
        if(support[i]&(1u<<i))self|=1u<<i;
    }
    std::vector<std::vector<int>> modal(n,std::vector<int>(n+1));
    for(int lo=0;lo<n;++lo){std::vector<int> count(sigma);int top=0;
        for(int hi=lo+1;hi<=n;++hi){top=std::max(top,++count[last[hi-1]]);modal[lo][hi]=top;}}
    std::vector<bool> weak(limit),strict(limit);weak[0]=strict[0]=true;
    std::vector<U> physical(limit);
    std::array<Best,5> any{},single{},prior{},no_seed{};
    int initial=0;
    for(U set=0;set<limit;++set) {
        if(set){U bit=set&-set;physical[set]=physical[set^bit]|(1u<<pos[step(bit)]);}
        bool neutral=true;
        for(U todo=set;todo;todo&=todo-1){
            U bit=todo&-todo;int i=step(bit);
            if(!(rooted&bit)&&!(support[i]&set)){neutral=false;break;}
        }
        if(!neutral)continue;
        for(U todo=set;todo;todo&=todo-1){
            U bit=todo&-todo;int i=step(bit);
            bool already=(rooted&bit)||(support[i]&(set^bit));
            strict[set]=strict[set]||(strict[set^bit]&&already);
            weak[set]=weak[set]||(weak[set^bit]&&(already||(self&bit)));
        }
        int lo=0,mass=0;
        for(U parts=original|physical[set];parts;parts&=parts-1){
            int cut=step(parts);mass+=modal[lo][cut];lo=cut;
        }
        mass+=modal[lo][n];if(!set)initial=mass;
        for(unsigned j=0;j<rates.size();++j){
            int score=rates[j].b*(mass-initial)-rates[j].a*__builtin_popcount(set);
            auto update=[&](Best& b){if(score>b.score)b={score,physical[set]};};
            update(any[j]);if(weak[set])update(single[j]);if(strict[set])update(prior[j]);
            if(!(set&(rooted|self)))update(no_seed[j]);
        }
    }
    std::string L;for(int a:last)L+=char('0'+a);
    for(unsigned j=0;j<rates.size();++j){
        out.strictwins[j]+=any[j].score>prior[j].score;
        out.weakwins[j]+=any[j].score>single[j].score;
        out.isolated[j]+=no_seed[j].score>single[j].score;
        if(any[j].score>single[j].score&&out.examples[j].empty()){
            out.examples[j]=L+","+std::to_string(original)+","+std::to_string(initial)+","+
                std::to_string(any[j].score)+","+std::to_string(any[j].seams)+","+
                std::to_string(single[j].score)+","+std::to_string(single[j].seams)+","+
                std::to_string(prior[j].score)+","+std::to_string(no_seed[j].score)+","+
                std::to_string(no_seed[j].seams);
        }
    }
}
int main(int argc,char** argv){
    assert(argc==4);int sigma=std::atoi(argv[1]),start=std::atoi(argv[2]),stop=std::atoi(argv[3]);
    std::cout<<"alphabet,n,lambda_num,lambda_den,primitive,better_than_strict,better_than_weak,no_seed_beats_weak,first_L,F_mask,base_mass,any_gain_num,any_mask,weak_gain_num,weak_mask,strict_gain_num,isolated_gain_num,isolated_mask\n";
    for(int n=start;n<=stop;++n){
        Summary out;uint64_t end=1;for(int j=0;j<n;++j)end*=sigma;
        for(uint64_t code=0;code<end;++code){auto v=code;std::vector<int> s(n);
            for(int i=n;i-->0;){s[i]=v%sigma;v/=sigma;}examine(s,sigma,out);}
        for(U j=0;j<rates.size();++j)std::cout<<sigma<<','<<n<<','<<rates[j].a<<','<<rates[j].b<<','<<out.primitive<<','
            <<out.strictwins[j]<<','<<out.weakwins[j]<<','<<out.isolated[j]<<','
            <<(out.examples[j].empty()?",,,,,,,,,":out.examples[j])<<'\n';
        std::cout.flush();
    }
}
