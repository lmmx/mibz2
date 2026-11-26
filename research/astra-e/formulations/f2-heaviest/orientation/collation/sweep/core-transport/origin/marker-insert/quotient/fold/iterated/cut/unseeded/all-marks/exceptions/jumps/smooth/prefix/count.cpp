#include "../native/smooth.hpp"
#include <filesystem>
#include <iostream>
#include <unordered_map>

using W=uint64_t;
template<class T> std::vector<T> file(const std::filesystem::path& name) {
    std::ifstream in(name,std::ios::binary|std::ios::ate);assert(in);
    size_t n=in.tellg();assert(n%sizeof(T)==0);
    std::vector<T> out(n/sizeof(T));in.seekg(0);
    in.read(reinterpret_cast<char*>(out.data()),n);assert(in);return out;
}
struct Word { std::array<Byte,17> bytes{}; U needed=0; };
struct Counts {
    U k;
    std::unordered_map<U,Word> words;
    W calls=0,total=0,short_calls=0,short_bytes=0,leading=0,old_copies=0,old_short_copies=0;
    explicit Counts(U depth):k(depth){}
    void visit(U key,U dest,U length,U phase,U copies,const std::vector<U>& psi,
               const std::vector<Byte>& first,const std::vector<Byte>& gold){
        ++calls;total+=length;leading+=std::min(k,length);old_copies+=copies;
        auto [it,inserted]=words.try_emplace(key);
        if(inserted){
            U r=dest;
            for(U i=0;i<k;i++){it->second.bytes[i]=first[r];r=psi[r];}
        }
        U n=gold.size();
        for(U i=0;i<std::min(k,length);i++){
            U j=n-phase+i;if(j>=n)j-=n;
            assert(it->second.bytes[i]==gold[j]);
        }
        if(length<=k){
            ++short_calls;short_bytes+=length;old_short_copies+=copies;
            it->second.needed=std::max(it->second.needed,length);
        }
    }
    void print(const char* name,U block,U depth)const{
        U used=0,bytes=0;for(const auto& [_,w]:words)if(w.needed){++used;bytes+=w.needed;}
        std::cout<<block<<','<<depth<<','<<name<<','<<calls<<','<<total<<','<<short_calls<<','<<short_bytes
                 <<','<<leading<<','<<words.size()<<','<<used<<','<<bytes<<','<<old_copies<<','<<old_short_copies<<'\n';
    }
};
struct Boundary { U t,key; };
int main(int argc,char** argv){
    assert(argc==5);std::filesystem::path dir=argv[1];std::string block=argv[2];
    U nblock=std::stoul(block),mark=std::stoul(argv[3]),k=std::stoul(argv[4]);
    assert(k<=16);
    auto caps=file<uint16_t>(dir/("block-"+block+".caps"));auto gold=file<Byte>(dir/("block-"+block+".raw"));
    U n=gold.size();auto raw=smooth::decode(caps.data(),caps.size(),n,mark);
    Charge price;auto model=construct(raw,k,price);std::vector<U> remap;
    auto index=smooth::build(raw,model.states,&remap);
    ::Geometry bases(raw);std::array<U,256> counts;std::copy_n(bases.base.begin(),256,counts.begin());
    std::vector<Byte> last(n),first(n);
    std::vector<U> lf(n),psi(n),row(n),cls(n);
    U x=0;
    for(auto r:raw.runs)for(U j=0;j<r.width;j++,x++){
        last[x]=r.symbol;U y=counts[r.symbol]++;first[y]=r.symbol;lf[x]=y;psi[y]=x;
    }
    U c=0,start=0;
    for(const auto& bucket:model.states.end)for(U e:bucket){
        std::fill(cls.begin()+start,cls.begin()+e,c++);start=e;
    }
    std::vector<Boundary> branches,exceptions;
    x=mark;
    for(U t=0;t<n;t++){
        assert(t==0||x!=mark);assert(gold[n-t-1]==last[x]);row[t]=x;U s=cls[x];
        if(remap[s]&smooth::branch_tag)branches.push_back({t,s});
        if(last[x]!=model.modal[s])exceptions.push_back({t+1==n?0:t+1,(s<<8)|last[x]});
        x=lf[x];
    }
    assert(x==mark&&!branches.empty()&&!exceptions.empty());
    std::sort(exceptions.begin(),exceptions.end(),[](Boundary a,Boundary b){return a.t<b.t;});
    auto copies=[&](U src,U len){
        U segments=0,id=remap[cls[row[src]]];
        while(len){
            assert(!(id&(smooth::branch_tag|smooth::cycle_tag)));
            const auto& v=index.paths[id];U span=std::min(len,id-v.head+1);
            len-=span;id=v.above;++segments;
        }
        return segments;
    };
    Counts smooths(k),returns(k),sparse(k+1);
    // This census treats both edge families as cyclic. The true marked seam
    // can cut one return, which the caller must phase separately.
    for(size_t i=0;i<branches.size();i++){
        auto from=branches[i],to=branches[(i+1)%branches.size()];
        U d=to.t>from.t?to.t-from.t:n-from.t+to.t;
        U s=(from.t+1)%n, pieces=copies(s,d-1);
        if(d>1)smooths.visit(to.key,row[to.t],d-1,to.t,pieces,psi,first,gold);
        returns.visit(to.key,row[to.t],d,to.t,pieces+1,psi,first,gold);
    }
    for(size_t i=0;i<exceptions.size();i++){
        auto from=exceptions[i],to=exceptions[(i+1)%exceptions.size()];
        U d=to.t>from.t?to.t-from.t:n-from.t+to.t;
        sparse.visit(to.key,row[to.t],d,to.t,0,psi,first,gold);
    }
    assert(returns.total==n&&sparse.total==n&&returns.calls==branches.size()&&sparse.calls==exceptions.size());
    smooths.print("smooth",nblock,k);returns.print("branch_return",nblock,k);sparse.print("sparse_return",nblock,k);
}
