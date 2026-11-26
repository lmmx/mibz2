#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iostream>
#include <numeric>
#include <vector>
using U=uint32_t;
struct Piece {U x; int32_t d; U h;};
struct Level {U old,next; std::vector<U> image,add;};
U read32(std::istream& in){
    unsigned char b[4];in.read(reinterpret_cast<char*>(b),4);assert(in);
    return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);
}
std::vector<Level> maps(const char* path,U n,U primary){
    std::ifstream f(path,std::ios::binary);char magic[8];f.read(magic,8);
    assert(!std::memcmp(magic,"POSONLY1",8)&&read32(f)==n&&read32(f)==primary);
    U k=read32(f),levels=read32(f);f.ignore(5*k);
    std::vector<Level> out;
    for(U l=0;l<levels;l++){
        U old=read32(f),next=read32(f),count=read32(f);
        std::vector<Piece> parts;
        for(U i=0;i<count;i++)parts.push_back({read32(f),int32_t(read32(f)),read32(f)});
        parts.push_back({old,0,0});
        Level lv{old,next,std::vector<U>(old,UINT32_MAX),std::vector<U>(old)};
        for(U i=0;i<count;i++){
            auto [x,d,h]=parts[i];
            if(!h || l==0)for(U j=x;j<parts[i+1].x;j++)lv.image[j]=j+d;
        }
        for(U i=0;i<count;i++){
            auto [x,d,h]=parts[i];
            if(!h)continue;
            for(U j=x;j<parts[i+1].x;j++){
                if(l>0) lv.image[j]=lv.image[j+d];
                lv.add[j]=h;
            }
        }
        assert(*std::max_element(lv.image.begin(),lv.image.end())<next);
        out.push_back(std::move(lv));
    }
    assert(out.back().next==1&&f.peek()==EOF);return out;
}
struct Event {U clock,row,next;uint8_t symbol;};
struct Model {uint8_t symbol;U next;};
int main(int argc,char** argv){
    assert(argc==3||argc==4);
    U n=read32(std::cin),primary=read32(std::cin);
    std::vector<uint8_t> last(n),original(n),first(n);
    std::cin.read(reinterpret_cast<char*>(last.data()),n);assert(std::cin&&primary<n);
    std::vector<U> lf(n),psi(n),positions(n,n);
    std::array<U,256> freq{},cursor{};
    for(auto c:last)++freq[c];
    std::partial_sum(freq.begin(),freq.end()-1,cursor.begin()+1);
    for(U x=0;x<n;x++){U y=cursor[last[x]]++;lf[x]=y;psi[y]=x;first[y]=last[x];}
    U row=primary;
    for(U t=0;t<n;t++){
        assert(positions[row]==n);positions[row]=t;original[t]=first[row];row=psi[row];
    }
    assert(row==primary&&n>=16);
    auto layers=maps(argv[1],n,primary);
    std::vector<uint8_t> circular=original;
    circular.insert(circular.end(),original.begin(),original.begin()+16);
    std::ofstream work(argv[2]);assert(work);
    work<<"k,depth,rows,needed\n";
    for(U k:{1,2,3,4,6,8,12,16}){
        std::vector<U> state(n),starts{0};
        for(U x=1;x<n;x++){
            if(std::memcmp(&circular[positions[x]],&circular[positions[x-1]],k))
                starts.push_back(x);
            state[x]=starts.size()-1;
        }
        starts.push_back(n);
        std::vector<Model> model;
        U edge_count=0;
        for(U s=0;s+1<starts.size();s++){
            auto begin=starts[s],end=starts[s+1];
            std::array<U,256> counts{};
            for(U x=begin;x<end;x++)++counts[last[x]];
            edge_count+=std::count_if(counts.begin(),counts.end(),[](U x){return x>0;});
            uint8_t m=std::max_element(counts.begin(),counts.end())-counts.begin();
            auto anchor=std::find(last.begin()+begin,last.begin()+end,m)-last.begin();
            U next=state[lf[anchor]];
            for(U x=begin;x<end;x++)if(last[x]==m)assert(state[lf[x]]==next);
            model.push_back({m,next});
        }
        std::vector<Event> exceptions;
        std::vector<U> required{primary};
        row=primary;
        for(U t=0;t<n;t++){
            if(last[row]!=model[state[row]].symbol){
                exceptions.push_back({t,row,state[lf[row]],last[row]});required.push_back(row);
            }
            row=lf[row];
        }
        if(argc==4 && (k==4||k==8||k==16)){
            auto le=[](std::ofstream& stream,U v){
                unsigned char b[]={uint8_t(v),uint8_t(v>>8),uint8_t(v>>16),uint8_t(v>>24)};
                stream.write(reinterpret_cast<char*>(b),4);
            };
            auto prefix=std::string(argv[3]);
            std::ofstream out(prefix+".pred"+std::to_string(k),std::ios::binary);
            out.write("PREDCLK1",8);le(out,n);le(out,state[primary]);
            le(out,model.size());le(out,exceptions.size());
            for(auto m:model){out.put(m.symbol);le(out,m.next);}
            for(auto e:exceptions){le(out,e.clock);out.put(e.symbol);le(out,e.next);}
            assert(out.good());
            std::ofstream flags(prefix+".mask"+std::to_string(k),std::ios::binary);
            flags.write("CLKMASK1",8);le(flags,n);le(flags,primary);
            flags.write(reinterpret_cast<const char*>(last.data()),n);
            for(U x=0;x<n;x++)flags.put(last[x]!=model[state[x]].symbol);
            assert(flags.good());
        }
        // Give only the exception rows and the primary to the coordinate locator.
        // Test the clock of each independently even when nearby requests merge.
        std::vector<std::vector<U>> live;
        uint64_t visits=0;
        std::sort(required.begin(),required.end());
        required.erase(std::unique(required.begin(),required.end()),required.end());
        for(U l=0;l<layers.size();l++){
            work<<k<<','<<l<<','<<layers[l].old<<','<<required.size()<<"\n";
            visits+=required.size();live.push_back(required);
            for(auto& x:required)x=layers[l].image[x];
            std::sort(required.begin(),required.end());
            required.erase(std::unique(required.begin(),required.end()),required.end());
        }
        assert(required.size()==1&&required[0]==0);
        std::vector<U> back(1,0);
        for(size_t l=layers.size();l-->0;){
            std::vector<U> old(layers[l].old,UINT32_MAX);
            for(U x:live[l]){
                U v=back[layers[l].image[x]];
                assert(v<n);old[x]=v+layers[l].add[x];assert(old[x]<n);
            }
            back.swap(old);
        }
        U origin=back[primary];
        for(auto e:exceptions)assert((origin+n-back[e.row])%n==e.clock);
        // Once exception clocks and post-exception states are supplied, real output
        // needs no LF array or source ranks. Exercise every byte of this path.
        U s=state[primary],event=0;std::vector<uint8_t> replay(n);
        for(U t=0;t<n;t++){
            uint8_t c;
            if(event<exceptions.size()&&t==exceptions[event].clock){
                c=exceptions[event].symbol;s=exceptions[event++].next;
            }else{c=model[s].symbol;s=model[s].next;}
            replay[n-1-t]=c;
        }
        assert(event==exceptions.size()&&s==state[primary]&&replay==original);
        std::cout<<k<<','<<n<<','<<model.size()<<','<<edge_count<<','<<exceptions.size()<<','<<visits;
        U begin=0,max_gap=0;std::array<uint64_t,3> chunks{},tails{};
        for(size_t i=0;i<=exceptions.size();i++){
            U end=i==exceptions.size()?n:exceptions[i].clock;
            U gap=end-begin;max_gap=std::max(max_gap,gap);
            for(U j=0;j<3;j++){U len=4u<<j;chunks[j]+=gap/len;tails[j]+=gap%len;}
            begin=end+1;
        }
        std::cout<<','<<max_gap;
        for(U j=0;j<3;j++)std::cout<<','<<chunks[j]<<','<<tails[j];
        std::cout<<'\n';
    }
}
