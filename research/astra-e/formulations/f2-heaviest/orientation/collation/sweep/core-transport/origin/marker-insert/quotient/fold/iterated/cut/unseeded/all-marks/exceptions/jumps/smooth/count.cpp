#include <algorithm>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <vector>
using U=uint32_t;
U word(std::istream& f){
    unsigned char b[4];f.read(reinterpret_cast<char*>(b),4);assert(f);
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
int main(int argc,char** argv){
    assert(argc==2);std::ifstream f(argv[1],std::ios::binary);
    char magic[8];f.read(magic,8);assert(std::string(magic,8)=="PREDCLK1");
    auto n=word(f),primary=word(f),contexts=word(f),count=word(f);
    std::vector<U> next(contexts);
    for(auto& v:next){f.get();v=word(f);}
    std::vector<bool> branch(contexts,false),exception(n,false);
    std::vector<U> restart(n);
    for(U i=0;i<count;i++){
        auto t=word(f);f.get();auto c=word(f);
        assert(t<n&&c<contexts&&!exception[t]);exception[t]=true;restart[t]=c;
    }
    assert(f.peek()==EOF);
    std::vector<U> state(n);U s=primary;
    for(U t=0;t<n;t++){
        state[t]=s;
        if(exception[t]){branch[s]=true;s=restart[t];}else s=next[s];
    }
    assert(s==primary);
    U smooth_rows=0,groups=0,longest=0,smooth8=0,modal8=0,all_smooth_modal8=0;
    for(U t=0;t<n;){
        if(branch[state[t]]){++t;continue;}
        U begin=t;while(t<n&&!branch[state[t]])t++;
        smooth_rows+=t-begin;groups++;longest=std::max(longest,t-begin);
        smooth8+=8*((t-begin)/8);
    }
    for(U t=0;t<n;){
        auto begin=t;while(t<n&&!exception[t])++t;
        for(U j=begin;j+8<=t;j+=8){
            modal8++;
            bool clean=true;
            for(U p=j;p<j+8;p++)clean&=!branch[state[p]];
            all_smooth_modal8+=clean;
        }
        if(t<n)t++;
    }
    std::cout<<n<<','<<contexts<<','<<std::count(branch.begin(),branch.end(),true)<<','
             <<n-smooth_rows<<','<<groups<<','<<longest<<','<<smooth8<<','<<modal8<<','<<all_smooth_modal8
             <<','<<n-7*modal8<<"\n";
}
