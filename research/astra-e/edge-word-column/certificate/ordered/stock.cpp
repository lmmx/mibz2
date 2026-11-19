#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type"
#define main existing_route_main
#include "../../../formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated/cut/powers/xor/route.cpp"
#undef main
#pragma GCC diagnostic pop
struct Span{uint32_t label,width;};
void compare_stock(const std::vector<Span>& old,const std::vector<uint32_t>& words,const std::vector<uint32_t>& color,
 uint32_t mark,const std::vector<Span>& expected,const std::vector<std::pair<uint32_t,uint32_t>>& names,uint32_t next_mark){
    Root in;in.mark=mark;in.bytes=words;for(auto r:old)in.r.push_back({r.label,r.width});
    auto b=bases(in);std::vector<Slice> pieces;U at=0,i=0;
    for(auto r:old){while(r.width){U w=std::min(r.width,b[i+1]-at);pieces.push_back({r.label,i,w,0});
        at+=w;r.width-=w;if(at==b[i+1])i++;}}
    U leaf=*std::max_element(words.begin(),words.end())+1;
    std::vector<Node> gram(leaf);for(U i=0;i<leaf;i++)gram[i]={1,i,0,Node::Literal};
    auto out=publish_compact<true>(in,gram,pieces,color);
    assert(out.mark==next_mark&&out.r.size()==expected.size()&&out.bytes.size()==names.size());
    for(U i=0;i<out.r.size();i++)assert(out.r[i].k==expected[i].label&&out.r[i].n==expected[i].width);
    for(U i=0;i<out.bytes.size();i++){U z=out.bytes[i];auto [l,r]=names[i];
        if(r==UINT32_MAX)assert(z==l);else assert(gram[z].kind==Node::Join&&gram[z].l==l&&gram[z].r==r);}
}
