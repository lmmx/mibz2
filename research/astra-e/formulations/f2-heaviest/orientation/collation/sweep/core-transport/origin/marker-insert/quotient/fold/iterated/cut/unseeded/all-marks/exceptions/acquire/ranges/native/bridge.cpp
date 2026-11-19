#include "ranges.hpp"
#include <cstring>
#include <fstream>
#include <stdexcept>
namespace {
using namespace ranges;
U read32(std::istream& f) {
    uint8_t b[4];f.read(reinterpret_cast<char*>(b),4);if(!f)throw std::runtime_error("short context file");
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
Input* load(const char* path,const char* wire) {
    Input tmp;std::ifstream txt(path),f(wire,std::ios::binary);U b,w;
    if(!(txt>>tmp.mark))throw std::runtime_error("missing primary");
    while(txt>>b>>w){if(b>=256||!w)throw std::runtime_error("bad run");tmp.runs.push_back({b,w});tmp.n+=w;}
    char sig[8];f.read(sig,8);
    if(!f||std::memcmp(sig,"CTXMOD01",8)||read32(f)!=tmp.n||read32(f)!=tmp.mark)
        throw std::runtime_error("wrong context model");
    read32(f);U K=read32(f);for(U i=0;i<3;++i)read32(f);
    tmp.start.reserve(K+1);tmp.modal.reserve(K);tmp.start.push_back(0);
    for(U i=0;i<K;++i){tmp.start.push_back(read32(f));read32(f);int c=f.get();
        if(c<0||tmp.start.back()<=tmp.start[i])throw std::runtime_error("bad context");
        tmp.modal.push_back(uint8_t(c));}
    if(tmp.start.back()!=tmp.n || tmp.mark>=tmp.n || tmp.n>=1u<<30)throw std::runtime_error("invalid rank bounds");
    return new Input(std::move(tmp));
}
}
extern "C" void* range_input(const char* text,const char* ctx){return load(text,ctx);}
extern "C" void range_input_free(void* input){delete static_cast<ranges::Input*>(input);}
extern "C" void* range_state(){return new ranges::Solver;}
extern "C" void range_state_free(void* state){delete static_cast<ranges::Solver*>(state);}
extern "C" const uint32_t* range_construct(void* state,void* input,uint32_t* count) {
    const auto& out=static_cast<ranges::Solver*>(state)->run(*static_cast<ranges::Input*>(input));
    *count=out.size();return reinterpret_cast<const uint32_t*>(out.data());
}
