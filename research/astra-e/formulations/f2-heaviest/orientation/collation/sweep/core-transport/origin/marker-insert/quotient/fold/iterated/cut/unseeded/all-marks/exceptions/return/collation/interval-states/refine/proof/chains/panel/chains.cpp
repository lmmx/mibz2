// Reverse physical gap chains. Only the original L column and original primary are consumed.
#include <algorithm>
#include <array>
#include <cassert>
#include <bit>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <map>
#include <numeric>
#include <vector>
using U=uint32_t;
using Q=uint64_t;
int main(int argc,char** argv) {
    assert(argc==4 || argc==5);
    std::ifstream f(argv[1],std::ios::binary);assert(f);
    U z,p=std::stoul(argv[2]),id=std::stoul(argv[3]),R=0;
    std::vector<unsigned char> last;
    for(int lo,hi;(lo=f.get())!=EOF;) {
        hi=f.get();assert(hi!=EOF && lo>0 && lo<=127);
        R+=last.empty()||last.back()!=hi;last.insert(last.end(),lo,hi);
    }
    U n=last.size();std::array<U,257> c{};
    for(U a:last)++c[a+1];
    U sigma=std::count_if(c.begin(),c.end(),[](U i){return i!=0;});
    std::partial_sum(c.begin(),c.end(),c.begin());
    std::vector<U> lf(n),psi(n),succ(n),pred(n),support_lo(n);
    auto cursor=c;
    for(U x=0;x<n;++x){z=cursor[last[x]]++;lf[x]=z;psi[z]=x;}
    std::vector<U> run_id(n);
    for(U i=1;i<n;++i)run_id[i]=run_id[i-1]+(last[i]!=last[i-1]);
    U lap=0;
    for(U x=p;;){x=lf[x];++lap;if(x==p)break;assert(lap<n);}
    assert(lap==n);
    std::vector<bool> fixed(n,false),forced(n,false);
    for(U a=0;a<256;++a) if(c[a]&&c[a]<n && c[a]!=c[a+1])fixed[c[a]]=true;
    Q arcs=0,flexible=0,grounded=0,support_width=0;U widest=0;
    std::map<U,U> width_hist;
    for(U i=1;i<n;++i){
        if(fixed[i]){++grounded;continue;}
        U a=psi[i-1],b=psi[i];assert(last[a]==last[b] && a<b);
        if(b==a+1){succ[i]=b;assert(!pred[b]);pred[b]=i;++arcs;}
        else {++flexible;assert(!support_lo[b]);support_lo[b]=a+1;++width_hist[b-a];
              widest=std::max(widest,b-a);support_width+=b-a;}
    }
    // Optional single terminals survive with only F seams when their own support contains
    // a permanently selected F seam or the terminal itself. Any subset can be retained.
    std::vector<U> fc(n);
    for(U x=1;x<n;++x)fc[x]=fc[x-1]+fixed[x];
    U f_safe_endpoints=sigma;
    for(U gap=1;gap<n;++gap)if(!fixed[gap]){
        U lo=psi[gap-1]+1,hi=psi[gap];
        f_safe_endpoints+=fc[hi]>fc[lo-1];
    }
    U f_stab=0,self_stab=0,optional=0;
    for(U x=1;x<n;++x)if(!fixed[x]&&!succ[x]){
        U lo=psi[x-1]+1,hi=psi[x];
        bool by_f=fc[hi]>fc[lo-1],by_self=lo<=x&&x<=hi;
        f_stab+=by_f;self_stab+=by_self;optional+=by_f||by_self;
    }
    // Independent forward calculation at physical original L row boundaries.
    for(U x=1;x<n;++x){
        assert((pred[x]!=0)==(last[x-1]==last[x]));
        if(pred[x])assert(pred[x]==lf[x] && lf[x]==lf[x-1]+1);
    }
    assert(arcs==n-R && grounded==sigma-1 && flexible==R-sigma);
    for(U i=1;i<n;++i)forced[i]=!succ[i];
    // Select one support for every flexible endpoint. Cover by terminal if present,
    // otherwise by greedy rightmost point. Distinct Ψ rows allow right-sorted linear scan.
    std::vector<U> at=[&]{std::vector<U> v(n);for(U i=1;i<n;++i)v[i]=v[i-1]+forced[i];return v;}();
    U last_added=0,extra=0,already=0;
    for(U right=1;right<n;++right) if(U lo=support_lo[right]){
        if(at[right]>at[lo-1]){++already;continue;}
        if(last_added<lo){forced[right]=true;last_added=right;++extra;}
    }
    std::vector<U> witness(n);
    for(U i=1;i<n;++i)witness[i]=witness[i-1]+forced[i];
    for(U r=1;r<n;++r)if(support_lo[r])assert(witness[r]>witness[support_lo[r]-1]);
    Q vertices=0,paths=0,lower_exp=0,upper_exp=0,all_terminal_rows=0,fully_forced=0;
    U longest=0;double upper_log=0,lower_log=0,fonly_log=0;
    std::map<std::array<U,3>,U> hist;
    Q bursts=0,one_edge=0,burst_edges=0,arithmetic_saved=0;U max_streak=0;
    for(U x=1;x<n;++x)if(!pred[x]){
        U a=x,len=0,limit=0,terminal,streak=0,old_run=n,origin=x,delta=0;
        auto close_burst=[&](){
            if(!streak)return;
            // Unsigned arithmetic is defined even for negative physical translations.
            assert(origin+streak*delta==a);
            ++bursts;burst_edges+=streak;one_edge+=streak==1;arithmetic_saved+=streak-1;
            max_streak=std::max(max_streak,streak);
        };
        for(;;){
            ++len;assert(len<n);
            if(!limit && forced[a])limit=len;
            if(!succ[a]){terminal=a;break;}
            U target=succ[a],run=run_id[target];
            assert(target && run_id[target-1]==run);
            if(run!=old_run){close_burst();old_run=run;streak=0;origin=a;delta=target-a;}
            assert(target-a==delta);++streak;a=target;
        }
        close_burst();
        assert(limit&&limit<=len);U upper=len+!fixed[terminal];
        upper_log+=std::log2(upper);lower_log+=std::log2(limit);
        upper_exp+=std::bit_width(upper-1u);lower_exp+=std::bit_width(limit)-1u;
        if(fixed[terminal])fonly_log+=std::log2(len);
        fully_forced+=limit==1;longest=std::max(longest,len);
        ++paths;vertices+=len;all_terminal_rows+=len-limit+1;
        ++hist[{fixed[terminal],len,limit}];
    }
    assert(paths==R-1 && vertices==n-1 && burst_edges==arcs && bursts+arithmetic_saved==arcs);
    if(argc==5){
        std::ofstream detail(argv[4],std::ios::app);assert(detail);
        detail<<id<<','<<n<<','<<R<<','<<paths<<','<<arcs<<','<<bursts<<','<<one_edge<<','<<max_streak<<','<<arithmetic_saved<<'\n';
    }
    std::cout.precision(16);
    std::cout<<id<<','<<n<<','<<R<<','<<sigma<<','<<lap<<','<<arcs<<','<<paths<<','<<grounded<<','<<flexible<<','<<longest<<','<<support_width<<','<<widest<<','<<already<<','<<extra<<','<<all_terminal_rows<<','<<fully_forced<<','<<fonly_log<<','<<lower_log<<','<<upper_log<<','<<lower_exp<<','<<upper_exp<<','<<f_stab<<','<<self_stab<<','<<optional<<','<<f_safe_endpoints<<'\n';
    for(auto [k,v]:hist)std::cerr<<"path,"<<id<<','<<k[0]<<','<<k[1]<<','<<k[2]<<','<<v<<'\n';
    // Width bins preserve the number of binary bits needed to address a witness within a support.
    std::array<U,32> bins{};for(auto [w,v]:width_hist)bins[std::bit_width(w)-1]+=v;
    for(U j=0;j<32;++j)if(bins[j])std::cerr<<"width,"<<id<<','<<j<<','<<bins[j]<<'\n';
}
