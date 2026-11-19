// Enumerate exact first-return graphs for every cut, keeping only cuts with 4q >= m.
#include <algorithm>
#include <array>
#include <cassert>
#include <iostream>
#include <numeric>
#include <string>
using A=std::array<int,160>;
struct Table {
    int n,k,r=1,loop=0; bool primitive;
    A last{},first{},p{};
    Table(int n_,int k_,const A& a):n(n_),k(k_),last(a) {
        std::iota(p.begin(),p.begin()+n,0);
        std::stable_sort(p.begin(),p.begin()+n,[&](int i,int j){return last[i]<last[j];});
        for(int i=0;i<n;++i){first[i]=last[p[i]];r+=i>0&&last[i]!=last[i-1];}
        for(int i=0;i<n;++i) loop+=first[i]==first[p[i]];
        int t=1;
        for(int j=p[0];j!=0;j=p[j]) ++t;
        primitive=t==n;
    }
    int after(int mask,int& q) {
        A tags{},keep{},dead{},incoming{};
        incoming.fill(-1);
        for(int i=0;i<n;++i) {
            int x=first[i],y=first[p[i]];
            bool left=!(mask>>x&1),right=mask>>y&1;
            tags[i]=x*(k+1)+(left?1+y:0);
            if(left&&right)dead[p[i]]=1;
        }
        q=std::accumulate(dead.begin(),dead.begin()+n,0);
        int z=0;
        for(int i=0;i<n;++i)if(!dead[i])keep[i]=z++;
        for(int i=0;i<n;++i)if(!dead[i]){
            int d=p[i];if(dead[d])d=p[d];
            assert(!dead[d]&&incoming[keep[d]]<0);incoming[keep[d]]=tags[i];
        }
        int runs=1;
        for(int i=1;i<z;++i)runs+=incoming[i]!=incoming[i-1];
        return runs;
    }
    std::string str()const {
        std::string s;for(int i=0;i<n;++i)s+=char('0'+last[i]);return s;
    }
};
struct Results {
    long long columns=0,cuts=0,prescribed_loses=0,any_loses=0;
    int largest=0;std::string witness,first_forced,first_prescribed;
    void run(Table& t) {
        int best=999,full=(1<<t.k)-1;
        for(int mask=1;mask<full;++mask)if(mask&1){
            int q,q2,a=t.after(mask,q),b=t.after(full^mask,q2);assert(q==q2);
            if(4*q<t.n)continue;
            ++cuts;int low=std::min(a,b);best=std::min(best,low);
            if(low>t.r){
                ++prescribed_loses;
                if(first_prescribed.empty())first_prescribed=t.str()+"/"+std::to_string(mask)+"/"+std::to_string(q)+"/"+std::to_string(a)+"/"+std::to_string(b);
            }
        }
        if(best==999)return;
        ++columns;
        if(best>t.r){
            ++any_loses;
            if(first_forced.empty())first_forced=t.str()+"/"+std::to_string(t.r)+"/"+std::to_string(best);
            if(best-t.r>largest){largest=best-t.r;witness=t.str();}
        }
    }
};
int main(){
    Results stats[4];A last{};
    for(int n=2;n<=9;++n)for(int k=2;k<=std::min(4,n);++k){
        int limit=1;for(int i=0;i<n;++i)limit*=k;
        for(int code=0;code<limit;++code){
            int z=code,used=0;
            for(int i=n-1;i>=0;--i){last[i]=z%k;used|=1<<last[i];z/=k;}
            if(used!=(1<<k)-1)continue;
            Table t(n,k,last);
            stats[0].run(t);
            if(t.primitive)stats[1].run(t);
            if(!t.loop)stats[2].run(t);
            if(t.primitive&&!t.loop)stats[3].run(t);
        }
    }
    std::cout<<"class,productive_columns,productive_bipartitions,prescribed_growth,forced_growth_columns,largest_forced_increase,witness_largest,first_forced,first_prescribed\n";
    const char* names[]={"all","primitive","zero_self","primitive_zero_self"};
    for(int i=0;i<4;++i){auto& s=stats[i];
        std::cout<<names[i]<<','<<s.columns<<','<<s.cuts<<','<<s.prescribed_loses<<','<<s.any_loses<<','<<s.largest<<','<<s.witness<<','<<s.first_forced<<','<<s.first_prescribed<<'\n';}
    return 0;
}
