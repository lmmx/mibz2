#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <vector>
using U=uint32_t;
using Key=uint64_t;
struct State {U begin,end;Key key;};
struct Run {U begin,end,base;uint8_t a;};
struct Counts {uint64_t fragments=0,skipped_runs=0,context_edges=0,overlay_steps=0,patterns=0;};
struct Slice {U n=0,base=0;};
U read32(std::istream& f){
    unsigned char v[4];f.read(reinterpret_cast<char*>(v),4);assert(f);
    return U(v[0])|(U(v[1])<<8)|(U(v[2])<<16)|(U(v[3])<<24);
}
struct Input {
    U n,primary;std::vector<uint8_t> last,f;
    std::vector<U> lf,psi;
    std::vector<Run> runs;
    Input(std::vector<uint8_t> l,U p):n(l.size()),primary(p),last(std::move(l)),f(n),lf(n),psi(n){
        std::array<U,256> count{},base{};
        for(auto c:last)count[c]++;
        for(U a=1;a<256;a++)base[a]=base[a-1]+count[a-1];
        for(U x=0;x<n;){
            auto a=last[x];U y=x+1;
            while(y<n&&last[y]==a)++y;
            runs.push_back({x,y,base[a],a});
            for(;x<y;x++){lf[x]=base[a]++;psi[lf[x]]=x;f[lf[x]]=a;}
        }
    }
};
// This constructor reads only the original run table and its cumulative base.
template<typename F>
void histograms(const std::vector<Run>& runs,const std::vector<State>& states,Counts& account,F accept){
    std::array<Slice,256> sums{};
    std::array<uint8_t,256> touched{};
    size_t ri=0;
    for(auto st:states){
        while(runs[ri].end<=st.begin){ri++;account.skipped_runs++;}
        U used=0;
        for(U i=ri;runs[i].begin<st.end;i++){
            auto r=runs[i];
            U x=std::max(st.begin,r.begin),y=std::min(st.end,r.end);
            auto& cell=sums[r.a];
            if(!cell.n){touched[used++]=r.a;cell.base=r.base+x-r.begin;}
            cell.n+=y-x;account.fragments++;
            if(y==st.end){ri=i;break;}
        }
        account.context_edges+=used;
        accept(st,sums,touched,used);
        for(U j=0;j<used;j++)sums[touched[j]].n=0;
    }
}
std::vector<State> overlay(const std::vector<State>& old,const std::vector<State>& add,Counts& ac){
    std::vector<State> out;
    size_t i=0,j=0;U x=0;
    while(i<old.size()){
        while(j<add.size()&&add[j].end<=x)++j;
        Key id=old[i].key;U end=old[i].end;
        if(j<add.size()){
            if(add[j].begin<=x){id=add[j].key;end=std::min(end,add[j].end);}
            else end=std::min(end,add[j].begin);
        }
        assert(end>x);
        if(!out.empty()&&out.back().key==id)out.back().end=end;
        else out.push_back({x,end,id});
        ++ac.overlay_steps;x=end;
        if(x==old[i].end)i++;
    }
    return out;
}
std::vector<State> select(const Input& in,U depth,U threshold,Counts& ac){
    std::vector<State> active{{0,in.n,0}},owner=active;
    for(U k=1;k<=depth && !active.empty();k++){
        std::array<std::vector<State>,256> buckets;
        histograms(in.runs,active,ac,[&](State,const auto& sum,const auto& seen,U used){
            for(U i=0;i<used;i++){
                auto a=seen[i];auto [n,start]=sum[a];
                if(n>=threshold)buckets[a].push_back({start,start+n,(Key(k)<<32)|start});
            }
        });
        active.clear();
        for(auto& group:buckets)active.insert(active.end(),group.begin(),group.end());
        ac.patterns+=active.size();
        owner=overlay(owner,active,ac);
    }
    return owner;
}
std::vector<Key> oracle(const Input& in,U depth,U threshold){
    std::vector<U> tip(in.n),prefix(in.n),new_prefix(in.n);
    std::vector<Key> longest(in.n);
    for(U x=0;x<in.n;x++)tip[x]=x;
    for(U k=1;k<=depth;k++){
        for(U x=0;x<in.n;){
            U y=x+1;
            while(y<in.n&&prefix[y]==prefix[x]&&in.f[tip[y]]==in.f[tip[x]])++y;
            for(U j=x;j<y;j++){
                if(y-x>=threshold)longest[j]=(Key(k)<<32)|x;
                new_prefix[j]=x;
            }
            x=y;
        }
        prefix.swap(new_prefix);
        for(auto& x:tip)x=in.psi[x];
    }
    return longest;
}
void trial(const Input& in,U k,U t,bool output){
    Counts ac;
    auto comps=select(in,k,t,ac);
    auto truth=oracle(in,k,t);
    std::vector<U> owner(in.n);
    std::vector<uint8_t> prediction;
    std::vector<Key> keys;
    for(U s=0;s<comps.size();s++){
        auto c=comps[s];keys.push_back(c.key);
        for(U x=c.begin;x<c.end;x++){assert(truth[x]==c.key);owner[x]=s;}
    }
    std::sort(keys.begin(),keys.end());
    size_t logical=std::unique(keys.begin(),keys.end())-keys.begin();
    U events=0;
    Counts finish;
    histograms(in.runs,comps,finish,[&](State st,const auto& sum,const auto& seen,U used){
        uint8_t modal=seen[0];
        for(U i=0;i<used;i++){
            auto a=seen[i];auto c=sum[a];
            assert(owner[c.base]==owner[c.base+c.n-1]);
            if(c.n>sum[modal].n || (c.n==sum[modal].n&&a<modal))modal=a;
        }
        prediction.push_back(modal);events+=st.end-st.begin-sum[modal].n;
    });
    U x=in.primary,remaining=0,chunks=0,actual=0;
    for(U i=0;i<in.n;i++){
        if(in.last[x]==prediction[owner[x]])remaining++;
        else{chunks+=remaining/8;remaining=0;actual++;}
        x=in.lf[x];
    }
    chunks+=remaining/8;
    if(output){
        assert(x==in.primary&&events==actual);
        std::cout<<k<<','<<t<<','<<in.n<<','<<ac.patterns<<','<<logical<<','<<comps.size()<<','
                 <<events<<','<<chunks<<','<<ac.fragments<<','<<ac.skipped_runs<<','<<ac.overlay_steps<<','
                 <<ac.context_edges<<','<<finish.fragments<<','<<finish.context_edges<<"\n";
    }
}
int main(int argc,char** argv){
    if(argc==1){
        U columns=0,panels=0;
        for(U n=1,ways=3;n<=7;n++,ways*=3)for(U code=0;code<ways;code++){
            std::vector<uint8_t> l(n);U q=code;
            for(auto& a:l){a=q%3;q/=3;}
            Input in(l,0);
            for(U k:{1,2,3,4})for(U t:{1,2,3,4}){trial(in,k,t,false);panels++;}
            columns++;
        }
        std::cout<<"columns="<<columns<<" panels="<<panels<<"\n";return 0;
    }
    std::ifstream f(argv[1],std::ios::binary);
    char magic[8];f.read(magic,8);assert(std::string(magic,8)=="CLKMASK1");
    U n=read32(f),p=read32(f);
    std::vector<uint8_t> l(n);f.read(reinterpret_cast<char*>(l.data()),n);assert(f);
    Input in(l,p); // trailing test mask is deliberately unused
    for(U k:{8,16})for(U threshold:{1,2,4,8,16,32,64})trial(in,k,threshold,true);
}
