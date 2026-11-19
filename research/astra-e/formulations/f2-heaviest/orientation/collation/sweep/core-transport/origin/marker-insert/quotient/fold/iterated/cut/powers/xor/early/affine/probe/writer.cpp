#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <vector>
using U=uint32_t;
struct Owner {U source,image,x,y,first,end,lower,upper,word_color;};
struct Index {
    U n,rows,mark,source;
    std::vector<Owner> owners;
    std::vector<U> image,istart,lo,hi,closed,zero,mass;
    std::vector<uint8_t> out;
    template<typename T> static void read(std::ifstream& f,std::vector<T>& v,U n) {
        v.resize(n);f.read(reinterpret_cast<char*>(v.data()),v.size()*sizeof(T));assert(f);
    }
    explicit Index(const char* path) {
        static_assert(sizeof(Owner)==36);std::ifstream f(path,std::ios::binary);
        std::array<U,8> h;f.read(reinterpret_cast<char*>(h.data()),sizeof(h));assert(f&&h[0]==0x41335049);
        n=h[1];rows=h[2];mark=h[3];source=h[4];read(f,owners,h[5]);read(f,image,h[6]);read(f,istart,h[6]);
        read(f,lo,h[7]);read(f,hi,h[7]);read(f,closed,h[7]);read(f,zero,h[7]);read(f,mass,h[7]+1);
        assert(f.peek()==std::ifstream::traits_type::eof());out.resize(n+1);
    }
    const uint8_t* emit() {
        U row=mark,src=source,q=0;
        while(q<n) {
            const auto& prev=owners[src];
            U i=prev.lower;
            if(i+1<prev.upper)i=std::upper_bound(istart.begin()+i+1,istart.begin()+prev.upper,row)-istart.begin()-1;
            U e=image[i];const auto& v=owners[e];U off=row-v.image;
            out[q++]=uint8_t(v.word_color);src=e;
            if(v.first==UINT32_MAX){row=v.source+off;continue;}
            if(v.word_color&256) {
                U middle=v.x+off;
                U j=std::upper_bound(lo.begin()+v.first,lo.begin()+v.end,middle)-lo.begin();
                if(j>v.first&&middle<hi[j-1]) {
                    src=zero[j-1];const auto& partner=owners[src];
                    out[q++]=uint8_t(partner.word_color);
                    row=partner.source+middle-partner.y;
                }else row=v.source+off-(mass[j]-mass[v.first]);
            }else {
                U key=v.y+off-mass[v.first];
                U j=std::upper_bound(closed.begin()+v.first,closed.begin()+v.end,key)-closed.begin();
                row=v.source+off+mass[j]-mass[v.first];
            }
        }
        return out.data();
    }
};
extern "C" Index* index_load(const char* path){return new Index(path);}
extern "C" void index_drop(Index* p){delete p;}
extern "C" const uint8_t* index_emit(Index* p,U n){assert(n==p->n);return p->emit();}
