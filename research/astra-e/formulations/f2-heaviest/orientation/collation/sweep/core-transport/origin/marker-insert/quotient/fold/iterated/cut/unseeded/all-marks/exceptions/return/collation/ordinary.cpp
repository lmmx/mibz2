#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>
using U=uint32_t;
using Byte=uint8_t;
U word(std::istream& f){
    unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f);
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
struct Return {U target,len;std::string bytes,pattern;};
struct Metrics {U events=0,classes=0,runs=0,descent=0,covered=0,marks=0,uncovered_marks=0;
    U bad0=0,bad_short=0,min_h=0,max_hop=0;uint64_t spelling=0;};
Metrics evaluate(const std::vector<Byte>& last,U k,const std::vector<Byte>* given,U mark,bool allmarks){
    U n=last.size();
    std::array<U,257> base{};
    for(auto a:last)++base[U(a)+1];
    for(U i=0;i<256;i++)base[i+1]+=base[i];
    auto cursor=base;
    std::vector<U> lf(n),psi(n),tip(n),id(n),next_id(n);
    std::vector<Byte> F(n);
    for(U x=0;x<n;x++){lf[x]=cursor[last[x]]++;psi[lf[x]]=x;F[lf[x]]=last[x];tip[x]=x;}
    for(U j=0;j<k;j++){
        U owner=0;
        for(U x=0;x<n;x++){
            if(x&&(id[x]!=id[x-1]||F[tip[x]]!=F[tip[x-1]]))owner++;
            next_id[x]=owner;
        }
        id.swap(next_id);for(auto& x:tip)x=psi[x];
    }
    std::vector<U> default_next;std::vector<Byte> modes;
    std::vector<U> exception;
    for(U x=0;x<n;){
        U y=x+1;while(y<n&&id[x]==id[y])y++;
        std::array<U,256> count{};
        for(U j=x;j<y;j++)count[last[j]]++;
        Byte b=std::max_element(count.begin(),count.end())-count.begin();
        modes.push_back(b);
        U s=n;
        for(U j=x;j<y;j++)if(last[j]==b){if(s==n)s=id[lf[j]];else assert(s==id[lf[j]]);}
        else exception.push_back(j);
        default_next.push_back(s);
        x=y;
    }
    if(given){U count=0;for(U x=0;x<n;x++){bool flag=last[x]!=modes[id[x]];assert(flag==bool((*given)[x]));count+=flag;}assert(count==exception.size());}
    // The shift is a stable counting sort on *only* original event rows.
    std::stable_sort(exception.begin(),exception.end(),[&](U a,U b){return last[a]<last[b];});
    std::vector<U> row,rank(n,n);
    for(U x:exception){
        U y=lf[x];assert(row.empty()||row.back()<y);
        rank[y]=row.size();row.push_back(y);
    }
    Metrics m;m.events=row.size();
    std::vector<Return> ret;
    for(U r:row){
        std::string W,P;
        U x=r;
        do{W+=char(F[x]);x=psi[x];}while(rank[x]==n);
        U t=x;for(U j=0;j<=k;j++){P+=char(F[t]);t=psi[t];}
        U state=id[x];std::string rev;
        for(U j=1;j<W.size();j++){rev+=char(modes[state]);state=default_next[state];}
        std::reverse(rev.begin(),rev.end());
        assert(std::string(1,char(F[r]))+rev==W);
        ret.push_back({rank[x],U(W.size()),std::move(W),std::move(P)});
        m.covered+=ret.back().len;
    }
    std::vector<U> cls,first,encoded;
    bool found=false;
    for(U h=0;h<=k+1;h++){
        std::unordered_map<std::string,U> known;
        U old=n;bool contiguous=true;
        std::vector<U> classes,starts,prev_dest;
        for(U i=0;i<ret.size();i++){
            auto& r=ret[i];
            std::string key=r.bytes+r.pattern.substr(0,h);
            auto [it,ins]=known.try_emplace(key,known.size());U c=it->second;
            if(ins){starts.push_back(i);prev_dest.push_back(r.target);}
            else{if(old!=c)contiguous=false;assert(prev_dest[c]<r.target);prev_dest[c]=r.target;}
            classes.push_back(c);old=c;
        }
        if(!contiguous){
            if(!h)m.bad0=1;
            if(h==k)m.bad_short=1;
        }else if(!found){m.min_h=h;found=true;}
        if(h==k+1){
            assert(contiguous&&found);cls.swap(classes);first.swap(starts);m.classes=known.size();
            for(auto r:first)m.spelling+=ret[r].len;
        }
    }
    // A triple determines W without a byte dictionary, while P certifies the
    // boundary at its end. A unique (W,P) in turn determines exactly the triple.
    std::unordered_map<std::string,U> triple;
    encoded.resize(row.size());
    for(U i=0;i<ret.size();i++){
        auto& r=ret[i];
        std::string key=r.pattern+std::string(1,r.bytes[0])+std::to_string(r.len);
        auto [it,ins]=triple.try_emplace(key,cls[i]);assert(ins||it->second==cls[i]);
        encoded[r.target]=cls[i];
        m.descent+=i&&ret[i-1].target>r.target;
    }
    assert(triple.size()==m.classes);
    // Reconstruct sparse Psi entirely by stable alphabet counts of its L column.
    auto ptr=first;std::vector<U> reduced(row.size());
    for(U j=0;j<row.size();j++){
        U c=encoded[j];reduced[ptr[c]++]=j;m.runs+=j==0||encoded[j]!=encoded[j-1];
    }
    for(U i=0;i<row.size();i++)assert(reduced[i]==ret[i].target);
    for(U p=allmarks?0:mark;p<(allmarks?n:mark+1);p++){
        U x=p,steps=0;std::string out;
        while(rank[x]==n&&steps<n){out+=char(F[x]);x=psi[x];steps++;}
        m.max_hop=std::max(m.max_hop,steps);
        if(steps==n)m.uncovered_marks++;
        else{
            U j=rank[x];
            while(out.size()<n){auto& w=ret[first[cls[j]]].bytes;out.append(w,0,std::min(size_t(n)-out.size(),w.size()));j=reduced[j];}
        }
        x=p;for(char b:out){assert(Byte(b)==F[x]);x=psi[x];}assert(out.size()==n);m.marks++;
    }
    return m;
}
int main(int argc,char** argv){
    if(argc==1){
        uint64_t models=0,marks=0,none=0,h0=0,hk=0;
        for(U n=1,ways=3;n<=8;n++,ways*=3)for(U code=0;code<ways;code++){
            std::vector<Byte> last(n);U q=code;for(auto& x:last){x=q%3;q/=3;}
            for(U k:{1,2,3}){
                auto m=evaluate(last,k,nullptr,0,true);
                if(m.bad0&&!h0){std::cout<<"bare-witness k="<<k<<" last=";for(auto b:last)std::cout<<U(b);std::cout<<"\n";}
                if(m.bad_short&&!hk){std::cout<<"k-lookahead-witness k="<<k<<" last=";for(auto b:last)std::cout<<U(b);std::cout<<"\n";}
                models++;marks+=m.marks;none+=m.uncovered_marks;h0+=m.bad0;hk+=m.bad_short;
            }
        }
        std::cout<<"models="<<models<<" marks="<<marks<<" no_sections="<<none<<" bare_failures="<<h0<<" k_failures="<<hk<<"\n";
    }else{
        assert(argc==3);std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);assert(std::string(magic,8)=="CLKMASK1");
        U n=word(f),p=word(f),k=std::stoul(argv[2]);std::vector<Byte> last(n),flags(n);
        f.read(reinterpret_cast<char*>(last.data()),n);f.read(reinterpret_cast<char*>(flags.data()),n);assert(f);
        auto m=evaluate(last,k,&flags,p,false);
        std::cout<<n<<','<<m.events<<','<<m.classes<<','<<m.runs<<','<<m.descent<<','<<m.spelling<<','<<m.covered
                 <<','<<m.min_h<<','<<m.bad0<<','<<m.bad_short<<','<<m.max_hop<<"\n";
    }
}
