#define FORWARD_LIBRARY
#include "forward.cpp"
#include "direct_native.hpp"

int main(int argc,char** argv) {
    int laps=argc==2 ? std::stoi(argv[1]) : 16;
    assert(laps>0 && laps%4==0);
    int n,mark,r,c;
    while(std::cin>>n>>mark>>r>>c) {
        std::vector<InputRun> runs(r);
        for(auto& v:runs) std::cin>>v.byte>>v.width;
        std::vector<Interval> sections(c);
        for(auto& v:sections) std::cin>>v.lo>>v.hi;
        std::string expected;
        for(int lap=-1;lap<laps;++lap) {
            for(int slot=0;slot<2;++slot) {
                // AB BA BA AB, reversing both temporal position and neighboring direction.
                int id=slot^(lap>=0 && (lap%4==1 || lap%4==2));
                std::string out;
                timespec before,after;
                clock_gettime(CLOCK_THREAD_CPUTIME_ID,&before);
                auto start=Clock::now();
                double setup,discover,order,dict,emit;
                if(id==0) {
                    direct_native::Stats st;
                    out=direct_native::decode(n,mark,runs,sections,&st);
                    setup=st.setup_ms;discover=st.discover_ms;order=st.order_ms;dict=st.dict_ms;emit=st.emit_ms;
                } else {
                    Forward f(n,runs,sections,true,true);
                    f.discover();
                    bool found=f.expand(mark,out);
                    assert(found);
                    setup=f.setup_ms;discover=f.discover_ms;order=f.order_ms;dict=f.dict_ms;emit=f.emit_ms;
                }
                double wall=std::chrono::duration<double,std::milli>(Clock::now()-start).count();
                clock_gettime(CLOCK_THREAD_CPUTIME_ID,&after);
                double cpu=(after.tv_sec-before.tv_sec)*1000.+(after.tv_nsec-before.tv_nsec)*1e-6;
                assert(out.size()==size_t(n));
                if(lap==-1 && slot==0) expected=out;
                else assert(out==expected);
                if(lap>=0) std::cout<<lap<<","<<slot<<","<<id<<","<<cpu<<","<<wall<<","<<setup<<","
                                    <<discover<<","<<order<<","<<dict<<","<<emit<<"\n";
            }
        }
    }
}
