#define SHIFT_PANEL_LIBRARY
#include "main.cpp"

void test(){
    N cases=0,marks=0,gaps=0,early=0;
    for(int n=1;n<=8;n++)for(int b=0;b<1<<n;b++){
        std::vector<InputRun> runs;
        for(int j=0;j<n;j++){int c=(b>>j)&1;if(runs.empty() || runs.back().byte!=c)runs.push_back({c,0});++runs.back().width;}
        Oracle o(runs);
        for(int a=0;a<n;a++)for(int e=a+1;e<=n;e++)for(int h=0;h<=4;h++){
            auto r=execute<true>(runs,{a,e},h,true,0,&o,true);if(r.stats.h<0)break;
            ++cases;marks+=n;gaps+=r.stats.skipped_holes;early+=r.stats.fused_early_rows>0;
        }
    }
    std::cout<<"hull_objects "<<cases<<" marked_outputs "<<marks<<" static_holes_skipped "<<gaps<<" early_return_objects "<<early<<'\n';
}
int main(int argc,char**argv){
    if(argc==1){test();return 0;}
    assert(argc==4);std::string mode=argv[1];bool checked=mode=="check";int laps=std::stoi(argv[3]);
    std::ifstream in(argv[2]);int id,n,mark,a,b,m;
    std::cout<<"ordinal,round,pos,h,hull,loaded_forbidden_components,gapped_nodes,skipped_holes,skipped_ranks,live_packets,translations,selector_runs,setup_cpu_s,build_cpu_s,emit_cpu_s,cpu_s,wall_s\n";
    while(in>>id>>n>>mark>>a>>b>>m){
        std::vector<InputRun> runs(m);for(auto& x:runs)in>>x.byte>>x.width;
        Oracle o(runs);auto gold=o.marked(mark);
        for(int lap=checked?0:-1;lap<laps;lap++)for(int slot=0;slot<4;slot++){
            int arm=(slot+lap+1)%4;if(lap>=0 && (lap/4)%2)arm=3-arm;
            bool hull=arm>=2;int h=arm%2?3:0;double outer=cpu();auto wall=Clock::now();
            auto r=hull?execute<true>(runs,{a,b},h,true,mark,checked?&o:nullptr):
                        execute<false>(runs,{a,b},h,true,mark,checked?&o:nullptr);
            double elapsed=cpu()-outer;
            double duration=std::chrono::duration<double>(Clock::now()-wall).count();
            assert(r.bytes==gold); // Outside outer timing including constructor destruction.
            auto s=r.stats;if(lap>=0)std::cout<<id<<','<<lap<<','<<slot<<','<<h<<','<<hull<<','<<s.global_ranges
              <<','<<s.gapped_hulls<<','<<s.skipped_holes<<','<<s.skipped_rows<<','<<s.live_packets<<','<<s.translated_pieces
              <<','<<s.selectors<<','<<r.prepare<<','<<r.build<<','<<r.emit<<','<<elapsed<<','<<duration<<'\n';
        }
        std::cerr<<"verified "<<id<<'\n';
    }
}
