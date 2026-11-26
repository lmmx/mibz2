#pragma once
#include "../../../contexts/contexts.hpp"
#include <cstring>

// The two tags fit inside the 24-bit state field at bzip2's maximum block size.
namespace smooth {
constexpr U branch_tag=1u<<23, cycle_tag=1u<<22, no_state=~U(0);
struct Position { U x,state; };
struct Pure { U delta,length,head,above; };
struct Branch { U first,end; };
struct Slice { U end,delta,edge; };
struct Cycle { U first,length,delta; };
struct Index {
    U n=0;
    Position primary{};
    std::vector<Pure> paths;
    std::vector<Byte> words,cycle_words;
    std::vector<Branch> branches;
    std::vector<Slice> slices;
    std::vector<Cycle> cycles;
    size_t bytes() const {
        return paths.capacity()*sizeof(Pure)+words.capacity()+cycle_words.capacity()
            +branches.capacity()*sizeof(Branch)+slices.capacity()*sizeof(Slice)+cycles.capacity()*sizeof(Cycle);
    }
};
inline Input decode(const uint16_t* caps,U records,U n,U mark){
    Input raw;raw.n=n;raw.primary=mark;raw.runs.reserve(records);
    U sum=0;
    for(U i=0;i<records;++i){
        U width=caps[i]&255;Byte byte=caps[i]>>8;assert(width&&width<=127);
        if(!raw.runs.empty()&&raw.runs.back().symbol==byte)raw.runs.back().width+=width;
        else raw.runs.push_back({width,byte});
        sum+=width;
    }
    assert(sum==n&&n<=900000&&mark<n);return raw;
}
struct Temp {
    U width=0, delta=0, next=no_state, heavy=no_state;
    U total=0, length=0, final=no_state;
    Byte byte=0, status=0;
};
inline Index build(const Input& raw,const Partition& geometry,std::vector<U>* mapping=nullptr){
    Index idx;idx.n=raw.n;idx.primary.x=raw.primary;
    std::vector<U> ends;ends.reserve(geometry.size);
    for(const auto& bucket:geometry.end)ends.insert(ends.end(),bucket.begin(),bucket.end());
    assert(!ends.empty()&&ends.back()==raw.n);
    ::Geometry bases(raw);std::array<U,256> cursor,owner{};
    std::copy_n(bases.base.begin(),256,cursor.begin());
    for(Byte a:bases.active)owner[a]=std::upper_bound(ends.begin(),ends.end(),cursor[a])-ends.begin();
    std::vector<Temp> states(ends.size());
    Cursor scan{0,raw.runs.front().width};
    for(U c=0,begin=0;c<ends.size();begin=ends[c++]){
        U end=ends[c],head=idx.slices.size();auto& st=states[c];st.width=end-begin;
        intersect(raw,scan,begin,end,[&](Byte a,U x,U y){
            U& s=owner[a];U at=cursor[a];while(at>=ends[s])++s;
            assert(at+y-x<=ends[s]);
            idx.slices.push_back({y,at-x,(s<<8)|a});cursor[a]+=y-x;
        });
        if(idx.slices.size()==head+1){
            auto tr=idx.slices.back();idx.slices.pop_back();
            st.next=tr.edge>>8;st.delta=tr.delta;st.byte=Byte(tr.edge);
        }else{
            st.final=branch_tag|U(idx.branches.size());st.status=2;
            idx.branches.push_back({head,U(idx.slices.size())});
        }
    }
    // Resolve the functional forest; any remaining directed cycle is a closed
    // cycle of equal-width intervals because each LF image has source width.
    std::vector<U> stack;
    for(U c=0;c<states.size();++c){
        if(states[c].status)continue;
        U s=c;stack.clear();
        while(!states[s].status){
            states[s].status=1;stack.push_back(s);s=states[s].next;
        }
        if(states[s].status==1){
            assert(s==c);U first=idx.cycles.size(),len=stack.size(),sum=0;
            for(U i=0;i<len;i++){
                auto& t=states[stack[i]];assert(t.width==states[c].width);
                t.final=cycle_tag|(first+i);t.status=2;sum+=t.delta;
                idx.cycles.push_back({first,len,t.delta});
                idx.cycle_words.push_back(states[stack[len-1-i]].byte);
            }
            assert(sum==0);continue;
        }
        for(auto it=stack.rbegin();it!=stack.rend();++it){
            auto& t=states[*it];auto& parent=states[t.next];
            assert(parent.final==no_state||!(parent.final&cycle_tag));
            t.length=1+parent.length;t.total=t.delta+parent.total;t.status=2;
        }
    }
    U pure_count=0;
    for(U c=0;c<states.size();++c){
        const auto& t=states[c];if(t.final!=no_state)continue;
        ++pure_count;auto& p=states[t.next];
        if(p.final==no_state&&(p.heavy==no_state||t.width>states[p.heavy].width))p.heavy=c;
    }
    idx.words.reserve(pure_count);idx.paths.reserve(pure_count);
    // Allocate states in head-to-leaf order on each frequent-child chain.
    // Renumbering makes a short prefix's final state an integer subtraction.
    for(U c=0;c<states.size();++c){
        const auto& t=states[c];
        if(!t.length||states[t.next].heavy==c)continue;
        U head=idx.paths.size();
        for(U at=c;at!=no_state;at=states[at].heavy){
            auto& v=states[at];v.final=idx.paths.size();idx.words.push_back(v.byte);
            idx.paths.push_back({v.total,v.length,head,t.next});
        }
    }
    assert(idx.paths.size()==pure_count);
    for(auto& v:idx.paths)v.above=states[v.above].final;
    for(auto& e:idx.slices)e.edge=(states[e.edge>>8].final<<8)|Byte(e.edge);
    U primary_old=std::upper_bound(ends.begin(),ends.end(),raw.primary)-ends.begin();
    idx.primary.state=states[primary_old].final;
    if(mapping){
        mapping->clear();mapping->reserve(states.size());
        for(const auto& v:states)mapping->push_back(v.final);
    }
    return idx;
}
inline Index prepare(const uint16_t* caps,U records,U n,U mark,U depth){
    Input raw=decode(caps,records,n,mark);
    Charge cost;auto model=construct(raw,depth,cost);
    return build(raw,model.states);
}
inline Byte branch(const Index& idx,Position& p){
    auto range=idx.branches[p.state^branch_tag];
    const auto* it=idx.slices.data()+range.first;
    if(range.end-range.first<=4){while(it->end<=p.x)++it;}
    else {
        auto end=idx.slices.data()+range.end;
        it=std::upper_bound(it,end,p.x,[](U y,const Slice& s){return y<s.end;});
    }
    p.x+=it->delta;p.state=it->edge>>8;return Byte(it->edge);
}
inline Byte unit(const Index& idx,Position& p){
    if(p.state&branch_tag)return branch(idx,p);
    if(p.state&cycle_tag){
        U i=p.state^cycle_tag;auto t=idx.cycles[i];
        Byte b=idx.cycle_words[t.first+t.length-1-(i-t.first)];
        p.x+=t.delta;p.state=(i+1==t.first+t.length?t.first:i+1)|cycle_tag;return b;
    }
    const auto& v=idx.paths[p.state];Byte b=idx.words[p.state];
    p.x+=v.delta;p.state=p.state==v.head?v.above:p.state-1;
    if(!(p.state&branch_tag))p.x-=idx.paths[p.state].delta;
    return b;
}
inline void copy(Byte* dst,const Byte* src,U n){
    if(n>=4&&n<=8){std::memcpy(dst,src,4);std::memcpy(dst+n-4,src+n-4,4);}
    else if(n>=2&&n<4){std::memcpy(dst,src,2);std::memcpy(dst+n-2,src+n-2,2);}
    else if(n==1)*dst=*src;
    else std::memcpy(dst,src,n);
}
// Copy one complete smooth stretch (or its bounded prefix) backwards into
// [end-taken,end); update p to the exact physical row after those bytes.
inline U group(const Index& idx,Position& p,Byte* end,U limit){
    assert(limit>0);
    if(p.state&branch_tag){end[-1]=branch(idx,p);return 1;}
    if(p.state&cycle_tag){end[-1]=unit(idx,p);return 1;}
    const auto& first=idx.paths[p.state];U state=p.state,remaining=std::min(first.length,limit);
    U consumed=remaining;p.x+=first.delta;
    do {
        const auto& v=idx.paths[state];U available=state-v.head+1;
        U take=std::min(available,remaining);
        end-=take;copy(end,idx.words.data()+state+1-take,take);remaining-=take;
        state=take==available?v.above:state-take;
    }while(remaining);
    p.state=state;if(!(state&branch_tag))p.x-=idx.paths[state].delta;
    return consumed;
}
inline void repeat(Byte* out,U have,U n){
    while(have<n){U count=std::min(have,n-have);std::memcpy(out+have,out,count);have+=count;}
}
inline U emit(const Index& idx,Byte* output){
    auto p=idx.primary;
    if(p.state&cycle_tag){
        U i=p.state^cycle_tag;auto cyc=idx.cycles[i];
        U offset=i==cyc.first?0:cyc.length-(i-cyc.first);
        const Byte* word=idx.cycle_words.data()+cyc.first;
        std::memcpy(output,word+offset,cyc.length-offset);
        if(offset)std::memcpy(output+cyc.length-offset,word,offset);
        repeat(output,cyc.length,idx.n);return 0;
    }
    U left=idx.n;
    while(left)left-=group(idx,p,output+left,left);
    if(p.x==idx.primary.x)return 0;
    U period=0;p=idx.primary;
    do{unit(idx,p);++period;assert(period<=idx.n);}while(p.x!=idx.primary.x);
    std::memmove(output,output+idx.n-period,period);repeat(output,period,idx.n);return period;
}
} // namespace smooth
