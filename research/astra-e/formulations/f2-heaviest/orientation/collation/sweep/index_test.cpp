#include "substring.hpp"
#include <iostream>
#include <random>

int main() {
    std::mt19937 random(0x5e17);
    int64_t queries=0;
    int tapes=0;
    for(int size:{1,31,32,33,63,64,65,127,128,129,255,256,257}) {
        for(int pattern=0;pattern<6;++pattern){
            std::vector<uint8_t> text(size);
            for(int i=0;i<size;i++){
                switch(pattern){
                    case 0: text[i]=0;break;
                    case 1: text[i]=255;break;
                    case 2: text[i]=(i%7==2?255:0);break;
                    case 3: text[i]=((i/32)%3==0?i%4:0);break;
                    case 4: text[i]=random()%3;break;
                    case 5: text[i]=random()%256;break;
                }
            }
            ExactSubstrings index(text);++tapes;
            for(int i=0;i<size;i++)for(int j=0;j<size;j++){
                int same=0,cap=size-std::max(i,j);
                while(same<cap && text[i+same]==text[j+same])++same;
                assert(index.common(i,j,cap)==same);
                assert(index.common(i,j,cap/2)==std::min(same,cap/2));
                bool lower=std::lexicographical_compare(text.begin()+i,text.end(),text.begin()+j,text.end());
                assert(index.ordered(i,j)==lower);++queries;
            }
        }
    }
    std::cout<<tapes<<" periodic, constant and random byte tapes matched "<<queries<<" suffix-pair orderings and "<<2*queries<<" capped LCP queries\n";
}
