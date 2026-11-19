#include "core.hpp"
#include <fstream>
#include <iostream>
#include <map>
#include <numeric>
#include <set>
#include <time.h>
using namespace core;
using Clock=std::chrono::steady_clock;
double cpu(){timespec t{};clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);return t.tv_sec+t.tv_nsec*1e-9;}
struct Oracle{
    std::vector<int> f,b;
    std::string byte;
    explicit Oracle(const std::vector<InputRun>& raw){
        std::string L;for(auto [c,w]:raw)L.append(w,char(c));int n=L.size();
        f.resize(n);b.resize(n);std::iota(f.begin(),f.end(),0);
        std::stable_sort(f.begin(),f.end(),[&](int x,int y){return Byte(L[x])<Byte(L[y]);});
        for(int i=0;i<n;i++){byte.push_back(L[f[i]]);b[f[i]]=i;}
    }
    std::string marked(int p)const{std::string s;for(size_t k=0;k<f.size();k++){s.push_back(byte[p]);p=f[p];}return s;}
    std::pair<int,int> mark(int p,Range u)const{int x=p,k=0;while(x<u.lo || x>=u.hi){x=b[x];++k;if(x==p)return {-1,0};}return {x-u.lo,k};}
    int shortest(Range u)const{int ans=f.size();for(int x=u.lo;x<u.hi;x++){int y=x,n=0;do{++n;y=f[y];}while(y<u.lo || y>=u.hi);ans=std::min(ans,n);}return ans;}
    void check(const Plan& plan,Object& obj)const{
        auto [a,b]=plan.u;int h=plan.h;std::map<std::string,std::pair<int,int>> base;
        std::map<std::string,int> core_ids;std::vector<int> expected(b-a,-1),core_dest(b-a,-1);
        std::vector<std::pair<int,int>> origin(f.size(),{-1,-1});
        for(auto t:plan.target)for(int y=t.lo;y<t.hi;y++){
            assert(origin[y].first==-1);origin[y]={t.origin+y-t.lo,t.prefix};
        }
        for(int x=a;x<b;x++){
            int y=x,z=x;std::string prefix,full;
            for(int j=0;j<h;j++){prefix.push_back(byte[z]);z=f[z];}
            assert(origin[z].first==x && plan.prefixes[origin[z].second]==prefix);
            do{full.push_back(byte[y]);y=f[y];}while(y<a || y>=b);
            assert(int(full.size())>=h && full.substr(0,h)==prefix);
            auto [i,fresh]=base.try_emplace(full,std::pair{x-a,x-a});(void)fresh;
            assert(i->second.second==x-a);++i->second.second;
            expected[y-a]=i->second.first;
            auto [k,created]=core_ids.try_emplace(full.substr(h),core_ids.size());(void)created;
            core_dest[y-a]=k->second;
        }
        assert(obj.words.size()==base.size() && obj.cores.size()==core_ids.size());
        std::vector<int> ids;
        for(const auto& s:obj.cores){auto it=core_ids.find(s);assert(it!=core_ids.end());ids.push_back(it->second);}
        for(const auto& w:obj.words){auto it=base.find(obj.text(w));assert(it!=base.end());
            assert(w.base==it->second.first && w.base+w.width==it->second.second);}
        int dest=0,prev=-1;
        for(auto [id,width]:obj.incoming){assert(id!=prev);prev=id;
            for(int j=0;j<width;j++){
                assert(expected[dest]==obj.words[id].base && core_dest[dest]==ids[obj.words[id].core]);++dest;
            }}
        assert(dest==b-a);
    }
};
struct Receipt{Stats s;double setup=0,build=0,emit=0,total=0,wall=0;std::string out;};
Receipt run(const std::vector<InputRun>& raw,Range u,int h,int p,const Oracle* check=nullptr,bool all_marks=false){
    auto wall=Clock::now();double t=cpu();Receipt r;
    {
        Counts counts(raw);Plan plan(raw,counts,u,h,r.s);
        if(check)assert(plan.ok==(check->shortest(u)>=h));
        if(!plan.ok){r.setup=r.total=cpu()-t;r.wall=std::chrono::duration<double>(Clock::now()-wall).count();return r;}
        Index ix(raw,counts,plan,r.s);double begin=cpu();r.setup=begin-t;
        Object o(plan,ix,r.s);double built=cpu();r.build=built-begin;
        r.out=o.expand(p,counts.n);r.emit=cpu()-built;
        if(check){
            check->check(plan,o);
            assert(r.out==check->marked(p) && o.mark(p)==check->mark(p,u));
            if(all_marks)for(int i=0;i<counts.n;i++)assert(o.mark(i)==check->mark(i,u) && o.expand(i,counts.n)==check->marked(i));
        }
    }
    r.total=cpu()-t;r.wall=std::chrono::duration<double>(Clock::now()-wall).count();return r;
}
void small(){
    N cases=0,valid=0,rejected=0,marks=0,zeros=0,mixed=0,permuted=0;std::string witness;
    for(int alphabet:{2,3})for(int n=1;n<=(alphabet==2?8:5);n++){
        int limit=1;for(int j=0;j<n;j++)limit*=alphabet;
        for(int i=0;i<limit;i++){
            int seed=i;std::vector<InputRun> raw;bool third=false;
            for(int j=0;j<n;j++){int c=seed%alphabet;seed/=alphabet;third|=c==2;
                if(raw.empty() || raw.back().byte!=c)raw.push_back({c,0});
                ++raw.back().width;
            }
            if(alphabet==3 && !third)continue;
            Oracle oracle(raw);
            for(int a=0;a<n;a++)for(int b=a+1;b<=n;b++)for(int h=0;h<=std::min(6,n+1);h++){
                auto r=run(raw,{a,b},h,0,&oracle,true);++cases;
                if(!r.s.accepted){++rejected;continue;}
                ++valid;marks+=n;zeros+=r.s.root_zeros>0;mixed+=r.s.split_core_types>0;
                if(r.s.prefix_types>1){++permuted;if(r.s.split_core_types && witness.empty()){
                    for(auto v:raw)witness.append(v.width,char('0'+v.byte));
                    witness+=" U=["+std::to_string(a)+","+std::to_string(b)+") h="+std::to_string(h);
                }}
            }
        }
    }
    std::cout<<"cases "<<cases<<" admitted "<<valid<<" rejected "<<rejected<<" marked_outputs "<<marks
             <<" empty_core_objects "<<zeros<<" core_shared_by_prefixes "<<mixed<<" varying_prefix_objects "<<permuted
             <<" first_shared_core_example "<<witness<<'\n';
}
void print(const Receipt& r,int idx,int h,int round,int pos){
    auto v=r.s;
    std::cout<<idx<<','<<h<<','<<round<<','<<pos<<','<<v.accepted<<','<<v.first_hit<<','<<v.mass<<','<<v.prefix_types<<','<<v.tag_bands<<','<<v.tag_components
      <<','<<v.forward_seeks<<','<<v.forward_scans<<','<<v.formed_bands<<','<<v.formed_prefix_nodes<<','<<v.sort_comparisons<<','<<v.formation_slots
      <<','<<v.lf_pieces<<','<<v.lookup<<','<<v.scans<<','<<v.page_increments<<','<<v.gapped<<','<<v.holes<<','<<v.states<<','<<v.selector_entries
      <<','<<v.terminal_pieces<<','<<v.split_core_types<<','<<v.extra_word_lookups<<','<<v.root_zeros<<','<<v.root_zero_pieces
      <<','<<v.full_types<<','<<v.core_types<<','<<v.root_runs<<','<<v.core_runs<<','<<v.full_bytes<<','<<v.core_bytes<<','<<v.prefix_bytes
      <<','<<v.source_scatter<<','<<v.merged_inputs<<','<<v.branch_aliases<<','<<v.mark_steps<<','<<v.output_words
      <<','<<r.setup<<','<<r.build<<','<<r.emit<<','<<r.total<<','<<r.wall<<'\n';
}
int main(int argc,char**argv){
    if(argc==1){small();return 0;}assert(argc==4);bool check=std::string(argv[1])=="check";int rounds=std::stoi(argv[3]);
    std::cout<<"ordinal,h,round,position,accepted,first_hit,mass,prefix_types,final_tags,components,forward_seeks,forward_pieces,all_formed_tags,prefix_nodes,sort_comparisons,mask_input_slots,lf_table_records,lf_seeks,lf_pieces,page_increments,gapped_states,global_holes,live_states,selector_entries,terminal_pieces,cores_with_multiple_prefixes,extra_prefix_lookups,zero_rows,zero_fragments,full_words,core_words,full_token_runs,core_token_runs,logical_full_bytes,core_bytes,prefix_bytes,source_scatter,merge_entries,branch_aliases,mark_steps,output_words,setup_cpu_s,build_cpu_s,emit_cpu_s,cpu_s,wall_s\n";
    std::ifstream in(argv[2]);int idx,n,p,a,b,m;
    while(in>>idx>>n>>p>>a>>b>>m){
        std::vector<InputRun> raw(m);for(auto& r:raw)in>>r.byte>>r.width;Oracle oracle(raw);std::string gold=oracle.marked(p);
        std::vector<int> arms={0,3,4,6,8,10};if(check){arms.push_back(11);arms.push_back(12);}
        int narm=arms.size();
        for(int rep=check?0:-1;rep<rounds;rep++)for(int pos=0;pos<narm;pos++){
            int i=(pos+rep+1)%narm;if(rep>=0 && (rep/narm)%2)i=narm-i-1;
            int h=arms[i];auto r=run(raw,{a,b},h,p,check?&oracle:nullptr);
            if(r.s.accepted)assert(r.out==gold);
            if(rep>=0)print(r,idx,h,rep,pos);
        }
        std::cerr<<"checked "<<idx<<'\n';
    }
}
