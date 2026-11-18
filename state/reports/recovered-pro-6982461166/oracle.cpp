#include <algorithm>
#include <cassert>
#include <fstream>
#include <iostream>
#include <numeric>
#include <string>
#include <vector>

using Vec = std::vector<int>;
using Bytes = std::string;

Vec rotations(const Vec& s) {
    int n = s.size();
    Vec sa(n), rank(n), shifted(n), next(n), counts(std::max(n,257));
    for (int c : s) ++counts[c];
    for (int i=1; i<257; ++i) counts[i] += counts[i-1];
    for (int i=n-1; i>=0; --i) sa[--counts[s[i]]] = i;
    int classes = 0, previous = -1;
    for (int i : sa) {
        if (s[i] != previous) ++classes, previous = s[i];
        rank[i] = classes-1;
    }
    for (int step=1; step<n && classes<n; step*=2) {
        std::fill(counts.begin(), counts.begin()+classes, 0);
        for (int j=0; j<n; ++j) {
            int i = sa[j]-step;
            shifted[j] = i<0 ? i+n : i;
            ++counts[rank[shifted[j]]];
        }
        for (int c=1; c<classes; ++c) counts[c] += counts[c-1];
        for (int j=n-1; j>=0; --j) sa[--counts[rank[shifted[j]]]] = shifted[j];
        next[sa[0]]=0;
        classes=1;
        for (int j=1; j<n; ++j) {
            int a=sa[j-1], b=sa[j];
            if (rank[a]!=rank[b] || rank[(a+step)%n]!=rank[(b+step)%n]) ++classes;
            next[b]=classes-1;
        }
        rank.swap(next);
    }
    return sa;
}

struct MinTree {
    int size=1;
    static constexpr int inf=1<<30;
    Vec a;
    explicit MinTree(const Vec& xs) {
        while (size < int(xs.size())) size *= 2;
        a.assign(2*size, inf);
        std::copy(xs.begin(), xs.end(), a.begin()+size);
        for (int j=size-1; j>0; --j) a[j]=std::min(a[2*j],a[2*j+1]);
    }
    int minimum(int l,int r) const {
        int out=inf;
        for (l+=size,r+=size; l<r; l/=2,r/=2) {
            if (l&1) out=std::min(out,a[l++]);
            if (r&1) out=std::min(out,a[--r]);
        }
        return out;
    }
    int below(int j,int lo,int hi,int l,int r,int value,bool right) const {
        if (r<=lo || hi<=l || a[j]>=value) return -1;
        if (hi-lo==1) return lo;
        const int m=(lo+hi)/2;
        const int k=right?below(2*j+1,m,hi,l,r,value,right):below(2*j,lo,m,l,r,value,right);
        if (k>=0) return k;
        return right?below(2*j,lo,m,l,r,value,right):below(2*j+1,m,hi,l,r,value,right);
    }
    int below(int l,int r,int value,bool right) const {
        return below(1,0,size,l,r,value,right);
    }
};

void parse(const char* label, const Bytes& seed, const Bytes& target) {
    Bytes xs=seed+target;
    Vec symbols;
    for (unsigned char c:xs) symbols.push_back(int(c)+1);
    symbols.push_back(0);
    Vec sa=rotations(symbols);
    int n=symbols.size(), d=seed.size(), end=xs.size();
    Vec rank(n),lcp(n);
    for(int r=0;r<n;++r) rank[sa[r]]=r;
    for(int i=0,h=0;i<n;++i) {
        int r=rank[i];
        if (!r) {h=0;continue;}
        int j=sa[r-1];
        while (i+h<n && j+h<n && symbols[i+h]==symbols[j+h]) ++h;
        lcp[r]=h;
        if(h) --h;
    }
    MinTree common(lcp), position(sa);
    Bytes replay=seed;
    replay.reserve(end);
    int ops=0,single=0,longest=0,seedops=0,seedbytes=0,outputops=0,outputbytes=0,crossops=0,crossbytes=0;
    for(int p=d;p<end;) {
        const int rank_p=rank[p];
        auto source=[&](int k) {
            int l=common.below(1,rank_p+1,k,true);
            int r=common.below(rank_p+1,n,k,false);
            return position.minimum(l<0?0:l,r<0?n:r);
        };
        int lo=0,hi=std::min(p,end-p)+1;
        while (hi-lo>1) {
            int m=(lo+hi)/2;
            if (source(m)<=p-m) lo=m;
            else hi=m;
        }
        assert(lo);
        int j=source(lo);
        assert(j+lo<=p && replay.size()==size_t(p));
        const auto phrase=replay.substr(j,lo);
        assert(xs.compare(p,lo,phrase)==0);
        replay+=phrase;
        ++ops;
        single += lo==1;
        longest=std::max(longest,lo);
        if(j+lo<=d) {++seedops;seedbytes+=lo;}
        else if(j>=d) {++outputops;outputbytes+=lo;}
        else {++crossops;crossbytes+=lo;}
        p+=lo;
    }
    assert(replay==xs);
    std::cout<<label<<","<<d<<","<<ops<<","<<single<<","<<longest<<","
             <<seedops<<","<<seedbytes<<","<<outputops<<","<<outputbytes<<","<<crossops<<","<<crossbytes<<"\n";
}

int main() {
    std::ifstream file("frozen.pre",std::ios::binary);
    Bytes target((std::istreambuf_iterator<char>(file)),{});
    assert(target.size()==899931);
    Vec bytes;
    for(unsigned char c:target) bytes.push_back(c);
    auto rows=rotations(bytes);
    int n=rows.size(),primary=-1;
    Bytes column,rank_seed,text_seed,asc_seed;
    Vec heads;
    for(int i=0;i<n;++i) {
        int j=(rows[i]+n-1)%n;
        if(!rows[i]) primary=i;
        if(i==0 || target[j]!=column.back()) {
            heads.push_back(j);
            rank_seed.push_back(target[j]);
        }
        column.push_back(target[j]);
    }
    std::ofstream("frozen.L",std::ios::binary).write(column.data(),column.size());
    assert(primary==264045 && heads.size()==51184);
    std::sort(heads.begin(),heads.end());
    for(int j:heads) text_seed.push_back(target[j]);
    for(int i=0;i<256;++i) asc_seed.push_back(char(i));
    std::cout<<"seed,seed_len,copies,singletons,longest,seed_ops,seed_bytes,output_ops,output_bytes,cross_ops,cross_bytes\n";
    parse("run heads in text order",text_seed,target);
    parse("run heads in rank order",rank_seed,target);
    parse("all ascending byte values",asc_seed,target);
}
