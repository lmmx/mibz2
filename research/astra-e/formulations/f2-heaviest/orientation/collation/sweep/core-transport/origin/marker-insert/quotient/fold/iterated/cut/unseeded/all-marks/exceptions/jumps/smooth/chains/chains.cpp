#include <algorithm>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <vector>
using U=uint32_t;
using W=uint64_t;
U word(std::istream& f){
    unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f);
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
int main(int argc,char** argv){
    assert(argc==2);std::ifstream f(argv[1],std::ios::binary);
    char magic[8];f.read(magic,8);assert(std::string(magic,8)=="PREDCLK1");
    auto n=word(f),primary=word(f),k=word(f),events=word(f);
    std::vector<U> next(k),post(n,k),state(n),mass(k),heavy(k,k);
    std::vector<bool> branch(k,false);
    for(auto& v:next){f.get();v=word(f);}
    for(U j=0;j<events;j++){
        U t=word(f);f.get();post[t]=word(f);assert(post[t]<k);
    }
    assert(f.peek()==EOF);U s=primary;
    for(U t=0;t<n;t++){
        state[t]=s;mass[s]++;
        if(post[t]<k){branch[s]=true;s=post[t];}else s=next[s];
    }
    assert(s==primary);
    std::vector<U> incoming(k);
    for(U v=0;v<k;v++)if(!branch[v]&&!branch[next[v]]){
        U u=next[v];incoming[u]+=mass[v];
        if(heavy[u]==k||mass[v]>mass[heavy[u]])heavy[u]=v;
    }
    U light_cost=0;
    for(U u=0;u<k;u++){
        assert(incoming[u]<=mass[u]);
        if(heavy[u]!=k)light_cost+=incoming[u]-mass[heavy[u]];
    }
    U starts=0,light=0,longest=0,num=0;
    for(U t=0;t<n;){
        if(branch[state[t]]){t++;continue;}
        U b=t,segments=1;
        while(++t<n&&!branch[state[t]]){
            if(heavy[state[t]]!=state[t-1]){++segments;++light;}
        }
        starts++;num+=segments;longest=std::max(longest,segments);
        assert(segments<=U(32-__builtin_clz(n/mass[state[b]])));
    }
    U seam_adjust=!branch[state.back()]&&!branch[state.front()]&&heavy[state.front()]!=state.back();
    assert(light+seam_adjust==light_cost);
    std::vector<U> depth(k,UINT32_MAX),trail;
    W naive=0;U longest_path=0;
    for(U v=0;v<k;v++)if(branch[v])depth[v]=0;
    for(U v=0;v<k;v++)if(depth[v]==UINT32_MAX){
        trail.clear();U u=v;
        while(depth[u]==UINT32_MAX){depth[u]=UINT32_MAX-1;trail.push_back(u);u=next[u];}
        // On the real primitive inputs there are true exceptions on the single orbit.
        assert(depth[u]<UINT32_MAX-1);
        U d=depth[u];while(!trail.empty()){
            depth[trail.back()]=++d;trail.pop_back();
        }
    }
    for(U v=0;v<k;v++)if(!branch[v]){naive+=depth[v];longest_path=std::max(longest_path,depth[v]);}
    std::cout<<n<<','<<k<<','<<std::count(branch.begin(),branch.end(),false)<<','<<starts<<','<<light_cost
             <<','<<seam_adjust<<','<<num<<','<<longest<<','<<naive<<','<<longest_path<<"\n";
}
