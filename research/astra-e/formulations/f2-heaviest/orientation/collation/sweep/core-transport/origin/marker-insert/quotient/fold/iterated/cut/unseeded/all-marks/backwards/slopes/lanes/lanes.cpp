// Input is the oracle's normalized cyclic coordinates in original physical rank order.
#include <algorithm>
#include <cassert>
#include <cstdint>
#include <iostream>
#include <vector>
int main() {
    uint32_t n;
    while (std::cin.read(reinterpret_cast<char*>(&n), sizeof(n))) {
        assert(n>0 && n<=900000);
        std::vector<uint32_t> coord(n);
        std::cin.read(reinterpret_cast<char*>(coord.data()), sizeof(uint32_t)*n);
        assert(std::cin);
        for (uint32_t s=1;s<=256;++s) {
            if(s>64 && s!=128 && s!=256) continue;
            uint32_t spans=0,single=0,implied=0,maximum=0;
            auto difference=[&](uint32_t a,uint32_t b){
                return coord[b]>=coord[a]?coord[b]-coord[a]:n-coord[a]+coord[b];
            };
            for(uint32_t lane=0;lane<std::min(n,s);++lane){
                uint32_t x=lane;
                while(x<n){
                    uint32_t y=x+s, length=1;
                    if(y<n){
                        uint32_t slope=difference(x,y);
                        length=2;
                        y+=s;
                        while(y<n && difference(y-s,y)==slope){++length;y+=s;}
                    }
                    ++spans;
                    single+=length==1;
                    if(length>2) implied+=length-2;
                    maximum=std::max(maximum,length);
                    x=y;
                }
            }
            assert(n==2*spans-single+implied);
            std::cout<<s<<" "<<spans<<" "<<single<<" "<<implied<<" "<<maximum<<"\n";
        }
    }
}
