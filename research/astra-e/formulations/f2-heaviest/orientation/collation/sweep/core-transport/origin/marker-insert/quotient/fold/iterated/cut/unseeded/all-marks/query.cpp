// The preserved wire is little-endian u32/i32. This verifier reads the explicit bytes.
#include <algorithm>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <numeric>
#include <vector>
struct Piece {uint32_t start;int32_t delta;uint32_t add;};
uint32_t u32(std::ifstream& f){
    unsigned char v[4];f.read(reinterpret_cast<char*>(v),4);assert(f.good());
    return uint32_t(v[0])|(uint32_t(v[1])<<8)|(uint32_t(v[2])<<16)|(uint32_t(v[3])<<24);
}
int main(int argc,char** argv){
    assert(argc==2);std::ifstream f(argv[1],std::ios::binary);
    char magic[8];f.read(magic,8);assert(std::string(magic,8)=="ATLS0001");
    uint32_t n=u32(f),mark=u32(f),runs=u32(f),types=u32(f);
    std::vector<uint32_t> last,b(types+1),p(n);
    for(uint32_t i=0;i<runs;++i){auto k=u32(f),w=u32(f);assert(k<types);b[k+1]+=w;last.insert(last.end(),w,k);}
    assert(last.size()==n);std::partial_sum(b.begin(),b.end(),b.begin());
    for(uint32_t j=0;j<n;++j)p[b[last[j]]++]=j;
    std::vector<std::vector<Piece>> levels;
    size_t number=0;
    while(auto count=u32(f)){
        std::vector<Piece> v;v.reserve(count);
        for(uint32_t i=0;i<count;++i){uint32_t start=u32(f);int32_t d=int32_t(u32(f));uint32_t a=u32(f);v.push_back({start,d,a});}
        assert(v[0].start==0);levels.push_back(std::move(v));number+=count;
    }
    assert(f.peek()==std::ifstream::traits_type::eof());
    uint64_t visits=0,comparisons=0;std::vector<uint32_t> positions(n);
    for(uint32_t x=0;x<n;++x){
        int64_t y=x,off=0;
        for(auto& level:levels){
            auto it=std::upper_bound(level.begin(),level.end(),y,[&](auto a,auto& e){++comparisons;return a<e.start;});
            assert(it!=level.begin());--it;y+=it->delta;off+=it->add;
            assert(0<=y&&y<n&&off<n);++visits;
        }
        assert(y==0);positions[x]=off;
    }
    std::vector<bool> seen(n,false);uint32_t x=mark;
    for(uint32_t i=0;i<n;++i){
        assert(!seen[x]&&positions[x]==(positions[mark]+i)%n);
        seen[x]=true;x=p[x];
    }
    assert(x==mark);
    std::cout<<"rows="<<n<<" levels="<<levels.size()<<" primary_offset="<<positions[mark]
             <<" pieces="<<number<<" descriptor_bytes="<<12*number<<" visits="<<visits
             <<" binary_comparisons="<<comparisons<<" all_cyclic_offsets_verified=1\n";
}
