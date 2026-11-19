#define CORE_STAGE_LIBRARY
#include "../main.cpp"
#include <cstring>
extern "C" {
struct Handle;
struct WordView{uint32_t base,prefix_len,core_len;const char* prefix;const char* core;};
struct Output{const char* ptr;size_t len;uint32_t second;size_t capacity;};
Handle* core_handle();
void core_drop(Handle*);
void core_materialize(Handle*,const WordView*,size_t,const Item*,size_t);
Output core_finish(Handle*,unsigned,unsigned,unsigned,unsigned);
Output core_compact(Handle*,const uint16_t*,size_t,const uint32_t*,unsigned,unsigned,unsigned,unsigned,Byte);
}
static_assert(sizeof(Item)==8);
struct Paid{Stats s;double setup=0,discover=0,handoff=0,writer=0,total=0,wall=0;Output out{};std::string fallback;};
template<bool Local>
Paid construct(Handle* consumer,const std::vector<InputRun>& raw,Range u,int h,int p,const Oracle* check){
    auto wall=Clock::now();double start=cpu();Paid r;
    {
        Counts count(raw);PlanT<Local> plan(raw,count,u,h,r.s);
        if(!plan.ok){r.total=cpu()-start;return r;}
        Index index(raw,count,plan,r.s);double ready=cpu();r.setup=ready-start;
        ObjectT<Local> object(plan,index,r.s);auto [ordinal,skip]=object.mark(p);double built=cpu();r.discover=built-ready;
        if(ordinal<0){r.fallback=object.expand(p,count.n);r.writer=cpu()-built;}
        else {
            std::vector<WordView> records;records.reserve(object.words.size());
            for(auto w:object.words){
                const auto& a=plan.prefixes[w.prefix];const auto& b=object.cores[w.core];
                records.push_back({unsigned(w.base),unsigned(a.size()),unsigned(b.size()),a.data(),b.data()});
            }
            core_materialize(consumer,records.data(),records.size(),object.incoming.data(),object.incoming.size());
            auto handed=cpu();r.handoff=handed-built;
            r.out=core_finish(consumer,ordinal,skip,r.s.mass,count.n);r.writer=cpu()-handed;
        }
        if(check){check->check(plan,object);assert(check->mark(p,u)==std::make_pair(ordinal,skip));}
    }
    r.total=cpu()-start;r.wall=std::chrono::duration<double>(Clock::now()-wall).count();
    return r;
}
bool equal(const Paid& r,const std::string& expected){
    if(!r.fallback.empty())return r.fallback==expected;
    return r.out.len==expected.size() && !std::memcmp(r.out.ptr,expected.data(),expected.size());
}
void small_handoff(){
    N attempts=0,valid=0,second=0,disjoint=0;
    auto h=core_handle();
    for(int n=1;n<=6;n++)for(int seed=0;seed<(1<<n);seed++){
        std::vector<InputRun> raw;
        for(int j=0;j<n;j++){int c=(seed>>j)&1;
            if(raw.empty() || raw.back().byte!=c)raw.push_back({c,0});
            ++raw.back().width;
        }
        Oracle gold(raw);
        for(int a=0;a<n;a++)for(int b=a+1;b<=n;b++)for(int k=0;k<=n;k++)
            for(int p=0;p<n;p++){++attempts;auto r=construct<true>(h,raw,{a,b},k,p,&gold);
                if(!r.s.accepted)break;
                assert(equal(r,gold.marked(p)));++valid;second+=r.out.second;disjoint+=!r.fallback.empty();}
    }
    core_drop(h);
    std::cout<<"handed_admitted_marks "<<valid<<" attempts_including_first_rejection "<<attempts<<" actual_second "<<second<<" disjoint_fallback "<<disjoint<<'\n';
}
void row(int ordinal,int lap,int pos,int h,int layout,const Paid& r){
    auto s=r.s;
    std::cout<<ordinal<<','<<lap<<','<<pos<<','<<h<<','<<layout<<','<<s.accepted<<','<<s.sort_comparisons<<','<<s.formation_slots
      <<','<<s.gapped<<','<<s.holes<<','<<s.posted_holes<<','<<s.link_reads<<','<<s.link_writes<<','<<s.gap_capacity_bytes
      <<','<<s.lookup<<','<<s.scans<<','<<s.selector_entries<<','<<s.full_bytes<<','<<s.full_types<<','<<s.root_runs<<','<<s.mark_steps
      <<','<<r.out.second<<','<<r.out.capacity<<','<<r.setup<<','<<r.discover<<','<<r.handoff<<','<<r.writer<<','<<r.total<<','<<r.wall<<'\n';
}

#ifndef OBSERVED_PANEL_LIBRARY
int main(int argc,char** argv){
    if(argc==1){small<true>();small_handoff();return 0;}
    assert(argc==4);bool check=std::string(argv[1])=="check";int laps=std::stoi(argv[3]);
    std::cout<<"ordinal,round,position,h,layout,accepted,sort_comparisons,history_slots,gapped_nodes,holes,posted_holes,link_reads,link_writes,gap_capacity_bytes,lf_seeks,lf_pieces,selector_runs,logical_dict_bytes,words,root_runs,mark_steps,second,compact_capacity_bytes,setup_cpu_s,discovery_mark_cpu_s,handoff_cpu_s,writer_cpu_s,cpu_s,wall_s\n";
    std::ifstream f(argv[2]);int idx,n,p,a,b,m;
    while(f>>idx>>n>>p>>a>>b>>m){
        std::vector<InputRun> raw(m);std::vector<uint16_t> caps;std::array<uint32_t,256> freq{};
        for(auto& r:raw){f>>r.byte>>r.width;freq[r.byte]+=r.width;
            for(int w=r.width;w>0;w-=127)caps.push_back(uint16_t(r.byte*256+std::min(w,127)));}
        std::array<uint32_t,257> cumulative{};std::partial_sum(freq.begin(),freq.end(),cumulative.begin()+1);
        int byte=std::upper_bound(cumulative.begin(),cumulative.end(),a)-cumulative.begin()-1;
        assert(cumulative[byte+1]>=unsigned(b));Oracle oracle(raw);auto expected=oracle.marked(p);
        std::vector<std::pair<int,int>> arms={{-1,2}};
        for(int depth:(check?std::vector<int>{0,3,4,6,8,10,11,12}:std::vector<int>{0,3,8,10}))
            for(int j=0;j<2;j++)arms.emplace_back(depth,j);
        int size=arms.size();std::vector<Handle*> contexts(size);
        for(auto& c:contexts)c=core_handle();
        for(int lap=check?0:-1;lap<laps;lap++)for(int pos=0;pos<size;pos++){
            int arm=(pos+lap+1)%size;if(lap>=0 && lap/size%2)arm=size-1-arm;
            auto [h,layout]=arms[arm];Paid r;
            if(layout==2){
                auto wall=Clock::now();double t=cpu();r.out=core_compact(contexts[arm],caps.data(),caps.size(),freq.data(),n,p,a,b,byte);
                r.total=cpu()-t;r.wall=std::chrono::duration<double>(Clock::now()-wall).count();r.s.accepted=1;
            }else if(layout==1)r=construct<true>(contexts[arm],raw,{a,b},h,p,check?&oracle:nullptr);
            else r=construct<false>(contexts[arm],raw,{a,b},h,p,check?&oracle:nullptr);
            if(r.s.accepted)assert(equal(r,expected));
            if(lap>=0)row(idx,lap,pos,h,layout,r);
        }
        for(auto c:contexts)core_drop(c);
        std::cerr<<"verified "<<idx<<'\n';
    }
}

#endif
