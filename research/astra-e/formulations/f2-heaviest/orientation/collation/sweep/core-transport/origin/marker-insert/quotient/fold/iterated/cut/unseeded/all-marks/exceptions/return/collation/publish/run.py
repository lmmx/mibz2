"""Build from the cited native RLE and literal-word implementations, audit each full object."""
import csv
import hashlib
import json
from pathlib import Path
import subprocess as sp
import sys
here=Path(__file__).resolve().parent
work=Path(sys.argv[1]).resolve()
work.mkdir(exist_ok=True,parents=True)
stage=sys.argv[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
if stage=='build':
    # Retain the complete upstream overlay and band constructor. Populate its geometry
    # from ordered RLE, context exception ranges and primary in our C++ adapter.
    code=(here.parents[1]/'induce.cpp').read_text().split('struct Clock {',1)[0]
    start=code.index('    explicit Physical(')
    end=code.index('\n};',start)
    code=code[:start]+'    explicit Physical(U size):n(size),true_mark(false){}'+code[end:]
    (work/'bands.hpp').write_text(code)
    # Expose complete literal oracle objects from root's independent implementation.
    orig=(here.parent/'ordinary.cpp').read_text().split('int main(',1)[0]
    orig=orig.replace('Metrics evaluate(','''std::vector<U> expected_base,expected_l,expected_psi;
std::vector<std::string> expected_words;
Metrics evaluate(''')
    if '    return m;\n}' not in orig:raise ValueError('literal oracle interface')
    orig=orig.replace('    return m;\n}', '''    expected_base=first;expected_l=encoded;expected_psi=reduced;expected_words.clear();
    for(U start:first)expected_words.push_back(ret[start].bytes);
    return m;
}''')
    (work/'literal.hpp').write_text(orig)
    args=['g++','-std=c++20','-O3','-Wall','-Wextra','-Werror','-I',work,here/'publish.cpp','-o',work/'check']
    sp.run(args,check=True)
    manifest={k:sha(p)for k,p in {'bands_donor':here.parents[1]/'induce.cpp',
              'contexts':here.parents[2]/'contexts/contexts.hpp','literal_donor':here.parent/'ordinary.cpp',
              'source':here/'publish.cpp','driver':here/'run.py','bands_header':work/'bands.hpp',
              'literal_header':work/'literal.hpp','binary':work/'check'}.items()}
    manifest.update(compiler=sp.check_output(['g++','--version'],text=True).splitlines()[0],flags=args[1:6])
    (work/'build.json').write_text(json.dumps(manifest,indent=2)+'\n')
elif stage=='small':
    (work/'small.csv').write_bytes(sp.check_output([work/'check']))
elif stage=='real':
    raw,wires,ref=map(Path,sys.argv[3:6])
    known=json.loads((here.parents[1]/'inputs.json').read_text())
    earlier={int(x['index']):x for x in csv.DictReader((here.parents[2]/'contexts/inputs.csv').open())}
    references={x['block']:x for x in json.loads((here.parents[1]/'emitter/inputs.json').read_text())}
    objects=work/'objects';objects.mkdir(exist_ok=True)
    hashes=[]
    with (work/'blocks.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow('block k n primary states physical_payload_ranges events anchored_bands true_bands source_ranges dest_byte_ranges intersection_pieces piece_bound classes incoming_runs dest_lookup_comparisons prior_induce_packets primary_is_exception shifted_source skip verified_bytes key_comparisons base_sort_comparisons'.split())
        for q in known:
            i,k=q['index'],q['k'];source=raw/f'{i}.txt';mask=wires/f'{i}.mask{k}'
            assert sha(mask)==q['mask'] and sha(source)==earlier[i]['input_sha256']
            want=ref/f'block-{i}.raw';assert sha(want)==references[i]['raw_sha256']
            obj=objects/f'{i}-{k}.col'
            hashes.append(dict(block=i,k=k,original_rle_sha256=sha(source),mask_sha256=q['mask']))
            fields=sp.check_output([work/'check',source,str(k),mask,obj,want],text=True).strip().split(',')
            w.writerow([i,k,*fields]);f.flush()
            hashes[-1].update(column_sha256=sha(obj),column_bytes=obj.stat().st_size,marked_sha256=sha(want),libbzip2_sha256=references[i]['libbzip2_plain_sha256'])
    (work/'inputs.json').write_text(json.dumps(hashes,indent=2)+'\n')
else: raise ValueError(stage)
