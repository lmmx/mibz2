#define XOR_ROUTE_LIBRARY
#include "../route.cpp"
#include <array>

Root from_caps(const uint16_t* caps, unsigned count, unsigned n, unsigned mark) {
    Root in;in.mark=mark;in.r.reserve(count);
    std::array<U,256> freq{},ids{};
    U rows=0;
    for(U i=0;i<count;++i) {
        U k=caps[i]>>8,width=caps[i]&255;
        assert(width);rows+=width;freq[k]+=width;append(in.r,k,width);
    }
    assert(rows==n && mark<n);
    for(U k=0;k<256;++k) if(freq[k]) {
        ids[k]=in.bytes.size();in.bytes.push_back(k);
    }
    for(auto& run:in.r)run.k=ids[run.k];
    return in;
}

struct Output { std::vector<uint8_t> data; };
extern "C" Output* grammar_new() { return new Output; }
extern "C" void grammar_delete(Output* p) { delete p; }
extern "C" const uint8_t* grammar_run(Output* state,const uint16_t* caps,U count,U n,U mark,U arm) {
    auto in=from_caps(caps,count,n,mark);
    Stats stats;
    if(arm==0)state->data=decode_with<StreamCut<false>>(in,stats);
    else if(arm==1)state->data=decode_with<StreamCut<true>>(in,stats);
    else {assert(arm==2);state->data=direct(in);}
    assert(state->data.size()==n);return state->data.data();
}
