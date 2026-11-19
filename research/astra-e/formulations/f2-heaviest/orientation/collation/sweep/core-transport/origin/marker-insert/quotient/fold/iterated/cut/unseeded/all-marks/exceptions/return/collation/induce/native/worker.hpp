#pragma once
#include "../induce.hpp"
#include <cstring>
#include <ctime>
#include <cstdlib>

namespace induced_worker {
using U = induction::U;
using Byte = uint8_t;
struct Charge {
    uint64_t caps_ns=0,selector_ns=0,induce_ns=0,map_ns=0,emit_ns=0;
    U kept=0,records=0,joins=0,lap=0,selected_spans=0;
    size_t vector_bytes=0,live_with_result=0;
};
inline uint64_t now(){
    timespec t{};if(clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t)!=0)std::abort();
    return uint64_t(t.tv_sec)*1000000000+t.tv_nsec;
}
struct Edge{U next,rule;};
struct Occurrence{U start,length;};
struct Worker {
    std::vector<Byte> output;
    Charge last;
    std::vector<induction::Run> runs;
    std::vector<induction::Span> spans;
    std::vector<Edge> forward;
    std::vector<Occurrence> memo;
    std::vector<U> stack;
    std::array<U,256> counts{};

    void decode(const uint16_t* caps,U size,U n){
        counts.fill(0);runs.clear();runs.reserve(size);U x=0;
        for(U i=0;i<size;i++){
            U width=caps[i]&255,byte=caps[i]>>8;
            assert(width&&width<=127&&width<=n-x);
            if(!runs.empty()&&runs.back().byte==byte)runs.back().end+=width;
            else runs.push_back({x,x+width,0,byte});
            counts[byte]+=width;x+=width;
        }
        assert(x==n);
        std::array<U,256> next;U sum=0;
        for(U a=0;a<256;a++){next[a]=sum;sum+=counts[a];}
        for(auto& run:runs){run.base=next[run.byte];next[run.byte]+=run.end-run.begin;}
    }

    U select(U n,U mark,U arm){
        spans.clear();U begin=0,kept=0,primary=induction::none;
        auto append=[&](U right,bool hit){
            if(right==begin)return;
            assert(begin<right&&right<=n);
            if(hit&&begin<=mark&&mark<right)primary=kept+mark-begin;
            if(hit)kept+=right-begin;
            if(!spans.empty()&&spans.back().retained==hit)spans.back().end=right;
            else spans.push_back({right,hit});
            begin=right;
        };
        auto add=[&](U right,bool hit){
            if(!hit&&begin<=mark&&mark<right){append(mark,false);append(mark+1,true);}
            append(right,hit);
        };
        if(arm==0){
            add(std::max(U(1),n/22),true);add(n,false);
        }else if(arm==1){
            for(const auto& r:runs)add(r.end,r.end-r.begin<=4);
        }else{
            assert(arm==2);
            std::array<std::pair<U,U>,256> present;U size=0;
            for(U a=0;a<256;a++)if(counts[a])present[size++]={counts[a],a};
            std::sort(present.begin(),present.begin()+size);
            std::array<bool,256> rare{};
            for(U i=0;i<std::min(U(16),size);i++)rare[present[i].second]=true;
            for(U a=0;a<256;a++)if(counts[a])add(begin+counts[a],rare[a]);
        }
        assert(begin==n&&primary<kept);last.kept=kept;return primary;
    }

    // Emit the reverse of an original backward return-rule spelling. A close
    // frame publishes its completed span; future uses copy only finalized text.
    void spell(const std::vector<induction::Join>& grammar,U rule,U& written){
        constexpr U close=1u<<31;
        stack.clear();stack.push_back(rule);
        while(!stack.empty()){
            U id=stack.back();stack.pop_back();
            if(id<256){assert(written<output.size());output[written++]=Byte(id);continue;}
            if(id&close){
                auto& seen=memo[(id^close)-256];
                assert(seen.length==0&&seen.start<written);seen.length=written-seen.start;
                continue;
            }
            auto& seen=memo[id-256];
            if(seen.length){
                assert(seen.start+seen.length<=written&&seen.length<=output.size()-written);
                std::memcpy(output.data()+written,output.data()+seen.start,seen.length);
                written+=seen.length;
            }else{
                auto v=grammar[id-256];
                assert(v.left<id&&v.right<id);
                seen.start=written;
                stack.push_back(id|close);
                stack.push_back(v.left);
                stack.push_back(v.right);
            }
        }
    }
    void run(const uint16_t* caps,U records,U n,U mark,U arm){
        assert(n>0&&n<=900000&&mark<n&&arm<3);last={};
        auto a=now();decode(caps,records,n);auto b=now();
        U primary=select(n,mark,arm);auto c=now();
        auto result=induction::induce(runs,spans);auto d=now();
        assert(result.kept==last.kept&&result.grammar.size()<(1u<<31)-256);
        forward.resize(result.kept);
        for(auto r:result.records){
            assert(r.target<=result.kept-r.width&&r.source<=result.kept-r.width);
            for(U j=0;j<r.width;j++)forward[r.target+j]={r.source+j,r.rule};
        }
        memo.assign(result.grammar.size(),{});auto e=now();
        output.resize(n);U x=primary,written=0;
        do{auto edge=forward[x];spell(result.grammar,edge.rule,written);x=edge.next;}while(x!=primary);
        assert(written>0);last.lap=written;
        while(written<n){
            U size=std::min(written,n-written);
            std::memcpy(output.data()+written,output.data(),size);written+=size;
        }
        auto f=now();
        last.caps_ns=b-a;last.selector_ns=c-b;last.induce_ns=d-c;last.map_ns=e-d;last.emit_ns=f-e;
        last.records=result.records.size();last.joins=result.grammar.size();last.selected_spans=spans.size();
        last.vector_bytes=output.capacity()+runs.capacity()*sizeof(induction::Run)+spans.capacity()*sizeof(induction::Span)
            +forward.capacity()*sizeof(Edge)+memo.capacity()*sizeof(Occurrence)+stack.capacity()*sizeof(U);
        last.live_with_result=last.vector_bytes+result.records.capacity()*sizeof(induction::Record)
            +result.grammar.capacity()*sizeof(induction::Join);
    }
};
}
