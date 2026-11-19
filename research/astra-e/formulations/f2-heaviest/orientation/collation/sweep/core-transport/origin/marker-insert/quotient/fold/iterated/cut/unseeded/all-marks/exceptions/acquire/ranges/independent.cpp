// Serialize the independent original-run membership-packet constructor's canonical bands.
#define main reference_program_main
#include "../../return/induce.cpp"
#undef main
int main(int argc,char** argv) {
    assert(argc==2);std::ifstream f(argv[1],std::ios::binary);char sig[8];f.read(sig,8);
    assert(!std::memcmp(sig,"CLKMASK1",8));U n=read32(f),mark=read32(f);
    Input in{std::vector<uint8_t>(n),std::vector<uint8_t>(n),mark};
    f.read(reinterpret_cast<char*>(in.l.data()),n);f.read(reinterpret_cast<char*>(in.mask.data()),n);
    assert(f&&f.peek()==EOF);Cost c;Physical p(in);Map map(p,true,c);auto z=induce(p,map,c);
    for(auto r:z)for(U x:{r.src,r.y,r.dt}) {
        for(U shift:{0u,8u,16u,24u})std::cout.put(char((x>>shift)&255));
    }
}
