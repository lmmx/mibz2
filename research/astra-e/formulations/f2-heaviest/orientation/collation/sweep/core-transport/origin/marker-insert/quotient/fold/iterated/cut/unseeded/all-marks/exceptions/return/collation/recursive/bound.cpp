#define main recursive_test_cli
#include "recursive.cpp"
#undef main
#include <random>
std::pair<std::vector<U>,U> bwt(const std::vector<U>& text){
    U n=text.size();std::vector<U> order(n);std::iota(order.begin(),order.end(),0);
    std::stable_sort(order.begin(),order.end(),[&](U a,U b){
        for(U i=0;i<n;i++)if(text[(a+i)%n]!=text[(b+i)%n])return text[(a+i)%n]<text[(b+i)%n];
        return a<b;
    });
    U primary=0;std::vector<U> l;
    for(U x:order){if(!x)primary=l.size();l.push_back(text[(x+n-1)%n]);}
    return {l,primary};
}
int main(){
    U max_stages=0,three_plus=0,examples=0;
    for(U alphabet=2;alphabet<=16;alphabet++){
        std::vector<U> next(alphabet),trail{0},circuit;
        while(!trail.empty()){
            U v=trail.back();
            if(next[v]<alphabet)trail.push_back(next[v]++);
            else{circuit.push_back(v);trail.pop_back();}
        }
        circuit.pop_back();std::reverse(circuit.begin(),circuit.end());
        auto [l,mark]=bwt(circuit);std::vector<Stage> trace;
        auto out=decode(l,mark,1,1,0,trace);
        assert(l.size()==alphabet*alphabet&&trace[0].events==l.size()-alphabet);
        for(U i=0;i<circuit.size();i++)assert(out[i%out.size()]==circuit[i]);
    }
    std::mt19937 rng(436711);
    for(U trial=0;trial<1200;trial++){
        U n=9+rng()%230,sigma=2+rng()%7;std::vector<U> text(n);
        for(auto& a:text)a=rng()%sigma;
        // Keep the integer alphabet dense on short accidental omissions.
        auto keys=text;std::sort(keys.begin(),keys.end());keys.erase(std::unique(keys.begin(),keys.end()),keys.end());
        for(auto& a:text)a=std::lower_bound(keys.begin(),keys.end(),a)-keys.begin();
        auto [l,mark]=bwt(text);
        for(U next:{1,2}){
            std::vector<Stage> trace;auto out=decode(l,mark,1,next,0,trace);
            for(U i=0;i<n;i++)assert(out[i%out.size()]==text[i]);
            max_stages=std::max(max_stages,U(trace.size()));three_plus+=trace.size()>=3;examples++;
        }
    }
    std::cout<<"sharp_complete_graphs=15 marked_random_panels="<<examples<<" at_least_three_stages="
             <<three_plus<<" maximum_stages="<<max_stages<<"\n";
}
