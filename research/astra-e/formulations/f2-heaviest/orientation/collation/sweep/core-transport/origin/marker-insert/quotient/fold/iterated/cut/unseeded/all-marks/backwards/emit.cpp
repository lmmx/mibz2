// Reverse stable survivor expansion in one offsets buffer, then scatter the original F bytes.
#include <algorithm>
#include <cassert>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iostream>
#include <vector>
struct Piece {uint32_t x; int32_t delta; uint32_t add;};
struct Level {uint32_t old, next;std::vector<Piece> rec;};
uint32_t u32(std::ifstream& f){
    unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f.good());
    return uint32_t(b[0])|(uint32_t(b[1])<<8)|(uint32_t(b[2])<<16)|(uint32_t(b[3])<<24);
}
int main(int argc,char** argv){
    assert(argc==3||argc==4);std::ifstream f(argv[1],std::ios::binary);
    char magic[8];f.read(magic,8);assert(std::string(magic,8)=="POSONLY1");
    auto n=u32(f),mark=u32(f),k=u32(f),h=u32(f);
    std::vector<uint32_t> bounds(1);std::vector<uint8_t> bytes;
    for(uint32_t i=0;i<k;++i){auto count=u32(f);bounds.push_back(bounds.back()+count);bytes.push_back(uint8_t(f.get()));}
    assert(bounds.back()==n&&n&&mark<n);
    const bool audit=argc==4;
    std::vector<uint32_t> positions(audit?n:0);
    if(!h){
        assert(n==1&&k==1&&f.peek()==std::ifstream::traits_type::eof());
        std::ofstream out(argv[2],std::ios::binary);out.put(char(bytes[0]));assert(out.good());
        if(audit){std::ofstream fpos(argv[3],std::ios::binary);fpos.write("\0\0\0\0",4);assert(fpos.good());}
        return 0;
    }
    std::vector<Level> levels;uint64_t count=0,read_words=0,written_words=0,retained_moves=0,deleted_words=0;
    for(uint32_t i=0;i<h;++i){
        Level v{u32(f),u32(f),{}};auto p=u32(f);v.rec.reserve(p+1);
        for(uint32_t j=0;j<p;++j){uint32_t x=u32(f);int32_t d=int32_t(u32(f));uint32_t a=u32(f);v.rec.push_back({x,d,a});}
        assert(v.rec.front().x==0&&v.old==(i?levels.back().next:n));
        v.rec.push_back({v.old,0,0});levels.push_back(std::move(v));count+=p;
    }
    assert(levels.back().next==1&&f.peek()==std::ifstream::traits_type::eof());
    std::vector<uint32_t> offset(levels[0].next);offset[0]=0;
    for(size_t z=levels.size();z-->1;){
        const auto& level=levels[z];auto& r=level.rec;
        for(size_t i=r.size()-1;i-->0;){
            auto [x,shift,add]=r[i];auto width=r[i+1].x-x;
            if(add||!shift)continue;
            auto src=int64_t(x)+shift;
            assert(shift<0&&src>=0&&src+width<=level.next);
            std::memmove(offset.data()+x,offset.data()+src,width*sizeof(uint32_t));
            retained_moves+=width;read_words+=width;written_words+=width;
        }
        for(size_t i=0;i+1<r.size();++i){
            auto [x,shift,add]=r[i];auto end=r[i+1].x;
            if(!add)continue;
            assert(int64_t(x)+shift>=0&&int64_t(end)+shift<=level.old);
            for(auto j=x;j<end;++j)offset[j]=offset[int64_t(j)+shift]+add;
            read_words+=end-x;written_words+=end-x;deleted_words+=end-x;
        }
    }
    auto& r=levels[0].rec;
    auto it=std::upper_bound(r.begin(),r.end(),mark,[](auto x,const Piece& p){return x<p.x;});--it;
    auto origin=offset[int64_t(mark)+it->delta]+it->add;
    std::vector<uint8_t> output(n);uint32_t owner=0;
    for(size_t i=0;i+1<r.size();++i){
        auto [x,shift,add]=r[i];auto end=r[i+1].x;
        while(x<end){
            while(x>=bounds[owner+1])++owner;
            auto stop=std::min(end,bounds[owner+1]);uint8_t v=bytes[owner];
            while(x<stop){
                auto pos=offset[int64_t(x)+shift]+add;assert(pos<n);
                auto at=pos>=origin?pos-origin:n-origin+pos;
                output[at]=v;if(audit)positions[x]=at;++x;
            }
        }
    }
    read_words+=n+1;
    std::ofstream out(argv[2],std::ios::binary);out.write(reinterpret_cast<const char*>(output.data()),n);assert(out.good());
    if(audit){
        std::ofstream fpos(argv[3],std::ios::binary);
        for(auto v:positions){
            const uint8_t b[]={uint8_t(v),uint8_t(v>>8),uint8_t(v>>16),uint8_t(v>>24)};
            fpos.write(reinterpret_cast<const char*>(b),4);
        }
        assert(fpos.good());
    }
    std::cout<<"rows="<<n<<" levels="<<h<<" descriptor_bytes="<<12*count
             <<" offset_array_bytes="<<4*offset.size()<<" returned_bytes="<<n
             <<" survivor_words_moved="<<retained_moves<<" deleted_words_set="<<deleted_words
             <<" offset_words_read="<<read_words<<" offset_words_written="<<written_words
             <<" scattered_bytes="<<n<<" primary_offset="<<origin<<"\n";
}
