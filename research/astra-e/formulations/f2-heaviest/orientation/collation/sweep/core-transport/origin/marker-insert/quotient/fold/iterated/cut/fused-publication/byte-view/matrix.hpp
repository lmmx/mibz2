// A byte cap stream split at the physical image of the singleton mark.
// Source IDs, original bucket boundaries, and aliases are stable ranks.
struct ByteMatrix {
    enum class Route {fixed,normalize,cut};
    struct Ptr{U record,used;};
    struct Piece{U id,len;};
    const uint16_t* caps;U count,n,primary,marked_byte=0,center=0,z=0,zr=0,zu=0,K=0,self=0,ingress=0;
    std::array<U,256> freq{},literal{},lead{};
    std::vector<U> base,old_node,del,first,cell;
    std::vector<Ptr> cursors;
    std::vector<int8_t> col;
    std::vector<Node> grammar;
    explicit ByteMatrix(const uint16_t* p,U num,U rows,U mark):caps(p),count(num),n(rows),primary(mark){
        for(U i=0;i<num;++i)freq[p[i]>>8]+=p[i]&255;
        U pos=0,rank=0;bool found=false;
        base.push_back(0);
        for(U a=0;a<256;++a)if(freq[a]){
            literal[a]=rank++;grammar.push_back({1,a,0,Node::Literal});lead[a]=K;
            auto put=[&](U len){if(len){base.push_back(base.back()+len);old_node.push_back(literal[a]);++K;}};
            if(pos<=mark&&mark<pos+freq[a]){
                marked_byte=a;found=true;put(mark-pos);center=K;put(1);put(pos+freq[a]-mark-1);
            }else put(freq[a]);
            pos+=freq[a];
        }
        assert(found&&base.back()==rows&&center<K&&K<=258);
        U h=mark;for(U a=0;a<marked_byte;++a)h-=freq[a];
        pos=0;
        for(U r=0;r<num;++r){
            auto v=p[r];U w=v&255;
            if(U(v>>8)==marked_byte){
                if(h<w){zr=r;zu=h;z=pos+h;return;}
                h-=w;
            }
            pos+=w;
        }
        assert(false);
    }
    Piece read(Ptr p)const{
        auto q=caps[p.record];U a=q>>8,len=(q&255)-p.used;
        if(a!=marked_byte)return{lead[a],len};
        if(p.record<zr)return{center-1,len};
        if(p.record>zr)return{center+1,len};
        if(p.used<zu)return{center-1,zu-p.used};
        if(p.used==zu)return{center,1};
        return{center+1,len};
    }
    void advance(Ptr& p,U len)const{
        p.used+=len;
        if(p.used==U(caps[p.record]&255)){++p.record;p.used=0;}
    }
    template<class F> void seeded_pieces(F f)const{
        Ptr p{};U x=0;
        while(p.record<count){auto v=read(p);f(p,v,x);advance(p,v.len);x+=v.len;}
        assert(x==n);
    }
    Route choose(){
        if(z==primary)return Route::fixed;
        cell.assign(K*K,0);cursors.resize(K);
        U j=0;
        seeded_pieces([&](Ptr p,Piece v,U pos){
            for(U left=v.len;left;){
                if(pos==base[j])cursors[j]=p;
                U width=std::min(left,base[j+1]-pos);cell[v.id*K+j]+=width;
                left-=width;pos+=width;
                if(left)p.used+=width;
                if(pos==base[j+1])++j;
            }
        });
        assert(j==K);
        for(U k=0;k<K;++k)self+=cell[k*K+k];
        col.assign(K,-1);col[center]=0;
        for(U k=0;k<K;++k)if(k!=center){
            int gain=0;
            for(U j=0;j<K;++j)if(col[j]>=0){
                int w=int(cell[k*K+j]+cell[j*K+k]);gain+=col[j]?-w:w;
            }
            col[k]=gain>0;
        }
        del.assign(K,0);
        for(U k=0;k<K;++k)if(!col[k])for(U j=0;j<K;++j)if(col[j]){
            U w=cell[k*K+j];del[j]+=w;ingress+=w;
        }
        return 4*ingress<n?Route::normalize:Route::cut;
    }
    Root publish(){
        Root out;out.r.reserve(count);first.resize(K);
        for(U k=0;k<K;++k){
            first[k]=out.bytes.size();U a=old_node[k];
            if(col[k]){
                if(base[k+1]-base[k]>del[k])out.bytes.push_back(a);
                continue;
            }
            for(U j=0;j<K;++j)if(cell[k*K+j]){
                U id=out.bytes.size(),h=a;
                if(col[j]){
                    U b=old_node[j];h=grammar.size();grammar.push_back({2,a,b,Node::Join});
                }
                out.bytes.push_back(h);cell[k*K+j]=id;
            }
        }
        out.mark=first[center];
        U j=0;
        seeded_pieces([&](Ptr,Piece v,U pos){
            U k=v.id,left=v.len;
            if(!col[k]){
                while(left){
                    while(pos>=base[j+1])++j;
                    U len=std::min(left,base[j+1]-pos);
                    if(!col[j])append(out.r,cell[k*K+j],len);
                    left-=len;pos+=len;
                }
            }else if(!del[k])append(out.r,first[k],left);
            else{
                Ptr& p=cursors[k];
                while(left){
                    auto u=read(p);U len=std::min(left,u.len);
                    auto id=col[u.id]?first[k]:cell[u.id*K+k];
                    append(out.r,id,len);advance(p,len);left-=len;
                }
            }
        });
        return out;
    }
};
