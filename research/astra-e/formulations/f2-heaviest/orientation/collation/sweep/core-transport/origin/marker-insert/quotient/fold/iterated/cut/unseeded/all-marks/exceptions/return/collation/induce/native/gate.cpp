#include "worker.hpp"
#include <filesystem>
#include <fstream>
#include <iostream>
#include <random>
using namespace induced_worker;

template<class T> std::vector<T> file(const std::filesystem::path& path){
    std::ifstream f(path,std::ios::binary|std::ios::ate);assert(f);
    size_t bytes=f.tellg();assert(bytes%sizeof(T)==0);std::vector<T> result(bytes/sizeof(T));f.seekg(0);
    f.read(reinterpret_cast<char*>(result.data()),bytes);assert(f);return result;
}
struct Totals{uint64_t columns=0,marks=0,calls=0,nondividing=0,short_laps=0,bytes=0;};
void check(const std::vector<uint16_t>& caps,const std::vector<U>& marks,Totals& totals,
           const std::vector<Byte>* gold=nullptr,U block=0){
    std::vector<Byte> last;
    for(auto c:caps)last.insert(last.end(),c&255,Byte(c>>8));
    U n=last.size();std::vector<U> psi(n);std::iota(psi.begin(),psi.end(),0);
    std::stable_sort(psi.begin(),psi.end(),[&](U a,U b){return last[a]<last[b];});
    std::vector<Byte> first(n);std::array<U,256> count{};
    for(U i=0;i<n;i++){first[i]=last[psi[i]];++count[first[i]];}
    std::vector<U> active;
    for(U a=0;a<256;a++)if(count[a])active.push_back(a);
    std::stable_sort(active.begin(),active.end(),[&](U a,U b){return count[a]<count[b];});
    std::array<bool,256> selected{};
    for(U j=0;j<std::min(size_t(16),active.size());j++)selected[active[j]]=true;
    std::array<std::vector<bool>,3> flags;
    for(auto& a:flags)a.resize(n);
    for(U begin=0;begin<n;){
        U end=begin+1;while(end<n&&last[end]==last[begin])++end;
        for(U x=begin;x<end;x++){flags[0][x]=x<std::max(U(1),n/22);flags[1][x]=end-begin<=4;flags[2][x]=selected[first[x]];}
        begin=end;
    }
    Worker decoder;++totals.columns;
    for(U p:marks){
        U row=p,lap=0;
        std::vector<Byte> want(n);
        for(U j=0;j<n;j++){
            want[j]=first[row];row=psi[row];
            if(!lap&&row==p)lap=j+1;
        }
        assert(lap&&lap<=n);if(gold)assert(want==*gold);
        ++totals.marks;totals.nondividing+=n%lap!=0;totals.short_laps+=lap<n;
        for(U arm=0;arm<3;arm++){
            decoder.run(caps.data(),caps.size(),n,p,arm);
            assert(decoder.output==want&&decoder.last.lap==lap);
            U begin=0,kept=0;
            for(auto span:decoder.spans){
                assert(begin<span.end&&span.end<=n);
                for(U x=begin;x<span.end;x++){assert(span.retained==(x==p||flags[arm][x]));kept+=span.retained;}
                begin=span.end;
            }
            assert(begin==n&&kept==decoder.last.kept);++totals.calls;totals.bytes+=n;
            if(gold){
                U q=0,orbit=0;for(U i=0;i<p;i++)q+=flags[arm][i];
                U start=q;do{++orbit;q=decoder.forward[q].next;}while(q!=start);
                U visited=std::count_if(decoder.memo.begin(),decoder.memo.end(),[](auto m){return m.length!=0;});
                auto z=decoder.last;std::cout<<block<<','<<arm<<','<<n<<','<<p<<','<<kept<<','<<z.selected_spans
                <<','<<z.records<<','<<z.joins<<','<<z.lap<<','<<z.vector_bytes<<','<<z.live_with_result<<','<<orbit<<','<<visited<<'\n';}
        }
    }
}
std::vector<uint16_t> encode(const std::vector<Byte>& last){
    std::vector<uint16_t> caps;
    for(Byte b:last){
        if(!caps.empty()&&(caps.back()>>8)==b&&(caps.back()&255)<127)++caps.back();
        else caps.push_back((U(b)<<8)|1);
    }
    return caps;
}
int main(int argc,char** argv){
    Totals s;
    if(argc==1){
        for(U alphabet:{2u,3u}){
            U ways=1;
            for(U n=1;n<=(alphabet==2?8:6);n++){
                ways*=alphabet;std::vector<U> marks(n);std::iota(marks.begin(),marks.end(),0);
                for(U code=0;code<ways;code++){
                    U x=code;std::vector<Byte> l(n);
                    for(auto& b:l){b=x%alphabet;x/=alphabet;}
                    check(encode(l),marks,s);
                }
            }
        }
        std::mt19937 gen(6610917);
        for(U j=0;j<100;j++){
            U n=std::array<U,10>{21,22,23,125,127,128,129,254,255,2049}[j%10];
            std::vector<Byte> l(n);U a=j%7;
            for(U i=0;i<n;i++){
                if(a==0)l[i]=255;
                if(a==1)l[i]=(i/(1+j%17))%37;
                if(a==2)l[i]=i%251;
                if(a==3)l[i]=i==n-1?128:0;
                if(a==4)l[i]=gen()%57;
                if(a==5)l[i]=i<127?3:2;
                if(a==6)l[i]=(i%27)<(i/5)%27?0:1;
            }
            check(encode(l),{0,1,std::max(U(1),n/22)-1,n/22,n/2,n-1},s);
        }
        assert(s.nondividing&&s.short_laps);
        std::cout<<"columns "<<s.columns<<" marks "<<s.marks<<" three-arm_calls "<<s.calls<<" bytes "<<s.bytes
                 <<" nondividing_marks "<<s.nondividing<<" short_lap_marks "<<s.short_laps<<'\n';
    }else{
        assert(argc==4);std::filesystem::path dir=argv[1];std::string id=argv[2];U mark=std::stoul(argv[3]);
        auto caps=file<uint16_t>(dir/("block-"+id+".caps"));auto gold=file<Byte>(dir/("block-"+id+".raw"));
        check(caps,{mark},s,&gold,std::stoul(id));
    }
}
