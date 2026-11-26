// Separate validator. Dense location is used exclusively to supply the hypothetical ordered clocks.
#include "contexts.hpp"
#include "oracle.hpp"
#include <iostream>
U word(std::istream& f){U v=0;for(U j=0;j<4;++j){int c=f.get();assert(c>=0);v|=U(c)<<(8*j);}return v;}
struct Event{U at,post;Byte byte;};
int main(int argc,char** argv) {
    assert(argc==3);
    auto raw=read_input(argv[1]);Dense ref(raw);
    std::ifstream wire(argv[2],std::ios::binary);char magic[8];wire.read(magic,8);
    assert(std::string(magic,8)=="CTXMOD01"&&word(wire)==raw.n&&word(wire)==raw.primary);
    U k=word(wire),C=word(wire),D=word(wire),events=word(wire),primary_state=word(wire);assert(k&&C);
    std::vector<U> ends(C),post(C);std::vector<Byte> def(C);
    for(U s=0;s<C;++s){ends[s]=word(wire);post[s]=word(wire);def[s]=wire.get();assert(post[s]<C);}
    assert(ends.back()==raw.n&&std::upper_bound(ends.begin(),ends.end(),raw.primary)-ends.begin()==primary_state);
    std::vector<U> exception_id(raw.n,UINT32_MAX),target(D);std::vector<Byte> actual(D);U covered=0;
    for(U i=0;i<D;++i){
        U b=word(wire),e=word(wire);target[i]=word(wire);actual[i]=wire.get();
        assert(b<e&&e<=raw.n&&target[i]<C);covered+=e-b;
        for(U x=b;x<e;++x){assert(exception_id[x]==UINT32_MAX);exception_id[x]=i;}
    }
    assert(covered==events&&wire.peek()==EOF);
    U row=raw.primary;
    for(U i=0;i<raw.n;++i)row=ref.psi[row];
    U cursor=std::upper_bound(ends.begin(),ends.end(),row)-ends.begin();std::vector<Event> ordered;
    for(U j=0;j<raw.n;++j){
        U rec=exception_id[row];
        if(rec!=UINT32_MAX)ordered.push_back({j,target[rec],actual[rec]});
        row=ref.lf[row];
    }
    assert(row==raw.primary);
    // Starting state, defaults and ordered exceptions are the entire writer input.
    std::vector<Byte> bytes(raw.n);U i=0;
    for(U t=0;t<raw.n;++t){
        if(i<ordered.size()&&ordered[i].at==t){auto e=ordered[i++];bytes[raw.n-1-t]=e.byte;cursor=e.post;}
        else {bytes[raw.n-1-t]=def[cursor];cursor=post[cursor];}
    }
    assert(cursor==primary_state&&i==ordered.size());
    std::cout.write(reinterpret_cast<const char*>(bytes.data()),bytes.size());
}
