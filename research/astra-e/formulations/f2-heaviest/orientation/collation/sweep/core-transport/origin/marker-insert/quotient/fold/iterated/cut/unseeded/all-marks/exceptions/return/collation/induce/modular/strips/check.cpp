#include "strips.hpp"
#include "../modular.hpp"
#include <fstream>
#include <iostream>
#include <string>
using namespace strips;
struct Raw {std::vector<unsigned char> last;std::vector<U> lf;Exchange initial;
    explicit Raw(std::vector<unsigned char> in):last(std::move(in)),lf(last.size()),initial(last.size()){
        std::array<U,256> hist{},cur{};for(auto b:last)++hist[b];
        std::partial_sum(hist.begin(),hist.end()-1,cur.begin()+1);
        for(U x=0;x<last.size();){U y=x+1;
            while(y<last.size()&&last[y]==last[x])++y;
            auto b=last[x];initial.b.push_back({x,cur[b],y-x,b});
            for(;x<y;++x)lf[x]=cur[b]++;
        }
    }
};
std::vector<unsigned char> unpack(U id,const std::vector<Join>& grammar){
    std::vector<U> stack{id};std::vector<unsigned char> out;
    while(!stack.empty()){U a=stack.back();stack.pop_back();
        if(a<256)out.push_back(a);else{auto r=grammar[a-256];stack.push_back(r.right);stack.push_back(r.left);}}
    return out;
}
void audit(const Raw& p,const Exchange& result,const std::vector<Join>& grammar,U first,Q stride,const std::vector<unsigned char>* actual) {
    std::vector<unsigned char> prev;
    for(auto b:result.b){
        auto bytes=unpack(b.rule,grammar);assert(!bytes.empty()&&prev!=bytes);
        for(U i=0;i<b.w;++i){U x=first+stride*(b.x+i),steps=0;
            for(auto a:bytes){assert(a==p.last[x]);x=p.lf[x];++steps;
                if(steps!=bytes.size())assert(x%stride!=first);}
            assert(x==first+stride*(b.y+i));}
        prev.swap(bytes);
    }
    if(result.n==1&&actual){
        std::reverse(prev.begin(),prev.end());
        assert(actual->size()==p.last.size());
        for(U i=0;i<actual->size();++i)assert((*actual)[i]==prev[i%prev.size()]);
    }
}
void test(const Raw& p,U mark,U q,bool small,const std::vector<unsigned char>* output,U block) {
    auto e=p.initial;std::vector<Join> grammar;U current=mark,layer=0;Q stride=1;
    std::vector<induction::Run> runs;
    for(auto b:e.b)runs.push_back({b.x,b.x+b.w,b.y,b.rule});
    auto surgery=modular::initial(runs);modular::Tower surgery_tree;
    if(e.n==1&&output){assert((*output)[0]==p.last[0]);}
    while(e.n>1){
        Tally t;
        if(small){
            auto compare=e;std::vector<Join> other=grammar;Tally v;
            auto b=step(compare,q,current,true,other,v);auto next=step(e,q,current,false,grammar,t);
            assert(t.reverse_states==v.reverse_states&&t.hit_paths==v.hit_paths&&b.b.size()==next.b.size());
            for(U i=0;i<b.b.size();++i){auto x=b.b[i],y=next.b[i];assert(x.x==y.x&&x.y==y.y&&x.w==y.w&&unpack(x.rule,other)==unpack(y.rule,grammar));}
            e=std::move(next);
        }else e=step(e,q,current,false,grammar,t);
        surgery=modular::step(surgery,q,t.n,current,surgery_tree);
        auto v=surgery_tree.layers.back();assert(v.final_bands==t.canonical&&!v.partial);
        U start=0;
        std::vector<U> surgery_dest(surgery.arcs.size());
        for(U i:surgery.bottom){surgery_dest[i]=start;start+=surgery.arcs[i].width;}
        assert(start==e.n&&surgery.top.size()==e.b.size());
        for(U j=0;j<e.b.size();++j){U i=surgery.top[j];
            assert(surgery.arcs[i].width==e.b[j].w&&surgery_dest[i]==e.b[j].y);}
        stride*=q;current/=q;U phase=mark%stride;
        audit(p,e,grammar,phase,stride,output);
        if(!small)std::cout<<block<<','<<q<<','<<layer<<','<<t.n<<','<<t.kept<<','<<t.old<<','<<t.sinks<<','<<t.sink_ranges
            <<','<<t.packets<<','<<t.reverse_states<<','<<t.reverse_queries<<','<<t.reverse_compares<<','<<t.split<<','<<t.cut_sort
            <<','<<t.cut_ranges<<','<<t.hit_paths<<','<<t.empty_paths<<','<<t.canonical<<','<<t.rep_steps<<','<<t.forward_compares
            <<','<<t.joins<<','<<t.image_sort<<','<<grammar.size()<<','<<v.pieces<<','<<v.strict<<','<<v.equal<<','<<v.self<<','<<surgery_tree.grammar.size()<<'\n';
        ++layer;
    }
}
U read32(std::istream& f){std::array<unsigned char,4>b;f.read((char*)b.data(),4);assert(f);return U(b[0])|(U(b[1])<<8)|(U(b[2])<<16)|(U(b[3])<<24);}
int main(int argc,char** argv) {
    if(argc==1){Q models=0,cases=0;
        for(U alphabet:{2u,3u})for(U n=1,ways=alphabet;n<=(alphabet==2?9u:6u);++n,ways*=alphabet){
            for(U code=0;code<ways;++code){std::vector<unsigned char> last(n);U v=code;for(auto& b:last){b=v%alphabet;v/=alphabet;}
                Raw p(last);++models;
                for(U mark=0;mark<n;++mark)for(U q:{2u,3u,4u,5u,8u,16u}){test(p,mark,q,true,nullptr,0);++cases;}
            }
        }
        U sharp=0;
        for(U q=2;q<=64;++q){U n=q*q+3*q+1;std::vector<unsigned char> v(n,1);std::fill(v.end()-q-1,v.end(),0);Raw p(v);
            for(U phase=0;phase<q;++phase){auto e=p.initial;Tally t;std::vector<Join> rule;auto out=step(e,q,phase,false,rule,t);
                assert(t.sinks==q&&out.b.size()==q+1);++sharp;}}
        std::cout<<"models="<<models<<" marked_radix_cases="<<cases<<" sharp_two_run_cases="<<sharp<<'\n';return 0;
    }
    assert(argc==4);std::ifstream f(argv[1],std::ios::binary),g(argv[2],std::ios::binary);char magic[8];f.read(magic,8);
    assert(std::string(magic,8)=="CLKMASK1");U n=read32(f),mark=read32(f);std::vector<unsigned char> l(n),out(n);
    f.read((char*)l.data(),n);g.read((char*)out.data(),n);assert(f&&g&&g.peek()==EOF);
    Raw p(std::move(l));for(U q:{2u,4u,8u,16u})test(p,mark,q,false,&out,std::stoul(argv[3]));
}
