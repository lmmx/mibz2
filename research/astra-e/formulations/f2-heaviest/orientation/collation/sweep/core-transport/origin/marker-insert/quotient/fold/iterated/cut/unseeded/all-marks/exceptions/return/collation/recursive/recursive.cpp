#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <numeric>
#include <vector>
using U=uint32_t;
struct Perm {
    std::vector<U> lf,psi,f;
    U types;
    explicit Perm(const std::vector<U>& l):lf(l.size()),psi(l.size()),f(l.size()){
        types=1+*std::max_element(l.begin(),l.end());
        std::vector<U> count(types+1);
        for(U a:l)++count[a+1];
        std::partial_sum(count.begin(),count.end(),count.begin());
        for(U i=0;i<l.size();i++){auto d=count[l[i]]++;lf[i]=d;psi[d]=i;f[d]=l[i];}
    }
};
struct Rule{U len,first,poststate;};
struct Stage{U depth,k,n,types,contexts,events,classes,runs,phase,passed,lap,expanded,terminal;};
std::vector<U> decode(const std::vector<U>& l,U mark,U k,U next_k,U depth,std::vector<Stage>& trace){
    const U n=l.size();Perm p(l);
    std::vector<U> tips(n),id(n),next_id(n),frequency(p.types),touched;
    std::iota(tips.begin(),tips.end(),0);
    for(U h=0;h<k;h++){
        U s=0;
        for(U i=0;i<n;i++){
            if(i&&(id[i]!=id[i-1]||p.f[tips[i]]!=p.f[tips[i-1]]))++s;
            next_id[i]=s;
        }
        id.swap(next_id);for(auto& x:tips)x=p.psi[x];
    }
    std::vector<U> modal,transition,rank(n,n),rows;
    std::vector<bool> selected(n,false);
    U saved=0;
    for(U i=0;i<n;){
        U end=i;while(end<n&&id[i]==id[end])++end;
        touched.clear();
        for(U j=i;j<end;j++){if(!frequency[l[j]]++)touched.push_back(l[j]);}
        U mode=touched[0];
        for(U a:touched)if(frequency[a]>frequency[mode]||(frequency[a]==frequency[mode]&&a<mode))mode=a;
        saved+=frequency[mode];U post=n;
        for(U j=i;j<end;j++){
            if(l[j]!=mode)selected[p.lf[j]]=true;
            else if(post==n)post=id[p.lf[j]];else assert(post==id[p.lf[j]]);
        }
        modal.push_back(mode);transition.push_back(post);
        for(U a:touched)frequency[a]=0;
        i=end;
    }
    for(U i=0;i<n;i++)if(selected[i]){rank[i]=rows.size();rows.push_back(i);}
    assert(saved>=p.types && uint64_t(saved)*p.types>=n && saved+rows.size()==n);
    U phase=0,boundary=mark;
    while(!selected[boundary]){
        boundary=p.psi[boundary];phase++;
        if(boundary==mark)break;
    }
    const U num=rows.size();
    size_t log=trace.size();trace.push_back({depth,k,n,p.types,U(modal.size()),num,0,0,phase,0,0,0,0});
    std::vector<U> out;
    if(!selected[boundary]){
        U s=id[mark];
        do{out.push_back(modal[s]);s=transition[s];assert(out.size()<=modal.size());}while(s!=id[mark]);
        std::reverse(out.begin(),out.end());
        trace[log].terminal=1;
    }else{
        std::vector<Rule> rules;
        std::vector<U> L(num),succ(num),cls(num);
        std::array<U,4> before{};
        for(U i=0;i<num;i++){
            U x=rows[i],y=x,d=0;
            do{y=p.psi[y];++d;}while(rank[y]==n);
            std::array<U,4> key={d,p.f[x],p.f[y],id[p.psi[y]]};
            if(!i||key!=before)rules.push_back({d,p.f[x],id[y]});
            before=key;
            cls[i]=rules.size()-1;succ[i]=rank[y];L[succ[i]]=cls[i];trace[log].passed+=d;
        }
        {
            Perm reduced(L);assert(reduced.psi==succ);
            for(U i=0;i<num;i++)assert(reduced.f[i]==cls[i]);
        }
        trace[log].classes=rules.size();
        for(U i=0;i<num;i++)trace[log].runs+=!i||L[i]!=L[i-1];
        auto token_cycle=decode(L,rank[boundary],next_k,next_k,depth+1,trace);
        for(U c:token_cycle){
            auto rule=rules[c];U begin=out.size(),s=rule.poststate;
            assert(uint64_t(begin)+rule.len<=n);
            out.resize(begin+rule.len);out[begin]=rule.first;
            for(U j=rule.len-1;j;j--){out[begin+j]=modal[s];s=transition[s];}
        }
        trace[log].expanded=out.size();
        std::rotate(out.begin(),out.end()-(phase%out.size()),out.end());
    }
    trace[log].lap=out.size();
    // Independently check two marked physical laps. The returned semantic period
    // may divide a longer orbit with indistinguishable rotations.
    U x=mark,t=0;
    do{assert(out[t%out.size()]==p.f[x]);x=p.psi[x];++t;}while(x!=mark);
    assert(t%out.size()==0);
    for(U i=0;i<t;i++){assert(out[i%out.size()]==p.f[x]);x=p.psi[x];}
    return out;
}
U word(std::istream& f){
    unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f);
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
int main(int argc,char** argv){
    if(argc==1){
        uint64_t cases=0,stages=0,reduced=0,short_cycle=0;U max_depth=0;
        for(U n=1,ways=3;n<=7;n++,ways*=3)for(U code=0;code<ways;code++){
            U q=code;std::vector<U> l(n);for(auto& a:l){a=q%3;q/=3;}
            // Compact the used alphabet, as in a true ordinary column.
            auto keys=l;std::sort(keys.begin(),keys.end());keys.erase(std::unique(keys.begin(),keys.end()),keys.end());
            for(auto& a:l)a=std::lower_bound(keys.begin(),keys.end(),a)-keys.begin();
            for(U mark=0;mark<n;mark++)for(U k:{1,2})for(U next:{1,2}){
                std::vector<Stage> trace;
                auto out=decode(l,mark,k,next,0,trace);Perm p(l);U x=mark;
                for(U i=0;i<n;i++){assert(out[i%out.size()]==p.f[x]);x=p.psi[x];}
                cases++;stages+=trace.size();reduced+=trace.size()>1;short_cycle+=out.size()<n;
                max_depth=std::max(max_depth,U(trace.size()));
            }
        }
        std::cout<<"marked_configurations="<<cases<<" stages="<<stages<<" selected_marked_orbits="<<reduced
                 <<" short_semantic_cycles="<<short_cycle<<" maximum_levels="<<max_depth<<"\n";
    }else{
        assert(argc==4);std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);
        assert(std::string(magic,8)=="CLKMASK1");U n=word(f),mark=word(f);
        std::vector<U> l(n);for(U& a:l)a=static_cast<unsigned char>(f.get());
        // Physical symbols are sparse byte values; relabel the used alphabet in stable byte order.
        std::vector<bool> used(256);for(U a:l)used[a]=true;
        std::vector<U> label(256);U number=0;for(U a=0;a<256;a++)if(used[a])label[a]=number++;
        for(auto& a:l)a=label[a];
        std::vector<Stage> trace;auto out=decode(l,mark,std::stoul(argv[2]),std::stoul(argv[3]),0,trace);
        for(auto r:trace)std::cout<<r.depth<<','<<r.k<<','<<r.n<<','<<r.types<<','<<r.contexts<<','<<r.events<<','<<r.classes<<','<<r.runs<<','<<r.phase<<','<<r.passed<<','<<r.lap<<','<<r.expanded<<','<<r.terminal<<"\n";
    }
    return 0;
}
