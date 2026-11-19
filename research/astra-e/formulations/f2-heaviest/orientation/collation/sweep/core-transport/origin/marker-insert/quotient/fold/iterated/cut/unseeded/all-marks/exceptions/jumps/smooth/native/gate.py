"""Check immutable Rust donor outputs and all physical transitions in the native terminal."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
RECEIPT = next(p / 'powers/xor/capped/receipt.json' for p in HERE.parents
               if (p / 'powers/xor/capped/receipt.json').is_file())
COLUMNS = ('block,depth,inputs,marked_outputs,output_bytes,physical_rows_checked,smooth_rows,'
           'branch_rows,all_smooth_marked_laps,nondividing_marked_laps,'
           'nonconstant_all_smooth_nondividing_laps,prepared_vector_bytes')

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('caps',type=Path)
    parser.add_argument('work',type=Path)
    args=parser.parse_args()
    args.work.mkdir(parents=True,exist_ok=True)
    exe=args.work/'gate'
    subprocess.run(['g++','-std=c++20','-O2','-g','-Wall','-Wextra',HERE/'gate.cpp','-o',exe], check=True)
    actual={'gate_source_sha256':digest(HERE/'gate.cpp'),'terminal_sha256':digest(HERE/'smooth.hpp'),
            'contexts_sha256':digest(HERE.parents[2]/'contexts/contexts.hpp'),
            'exe_sha256':digest(exe),'donor_receipt_sha256':digest(RECEIPT),'inputs':[],
            'compiler':subprocess.check_output(['g++','--version'],text=True).splitlines()[0]}
    with (args.work/'finite.csv').open('x') as f:
        f.write(COLUMNS+'\n')
        f.write(subprocess.check_output([exe],text=True))
    with (args.work/'real.csv').open('x') as f:
        f.write(COLUMNS+'\n')
        for case in json.loads(RECEIPT.read_text())['inputs']:
            i,mark=case['block'],case['mark']
            caps=args.caps/f'block-{i}.caps'
            gold=args.caps/f'block-{i}.raw'
            assert digest(caps)==case['caps_le_sha256'] and digest(gold)==case['raw_sha256']
            assert caps.stat().st_size==2*case['records'] and gold.stat().st_size==case['rows']
            actual['inputs'].append({k:case[k] for k in ('block','mark','rows','records','caps_le_sha256',
                                    'raw_sha256','decoded_sha256','block_crc')})
            for k in (4,8,16):
                row=subprocess.check_output([exe,args.caps,str(i),str(mark),str(k)],text=True)
                cols=list(csv.reader(io.StringIO(row)))
                assert len(cols)==1 and len(cols[0])==len(COLUMNS.split(','))
                vals=list(map(int,cols[0]))
                assert vals[:6]==[i,k,1,1,case['rows'],case['rows']]
                assert vals[6]+vals[7]==case['rows']
                f.write(row)
                f.flush()
    (args.work/'gate-manifest.json').write_text(json.dumps(actual,indent=2)+'\n')

if __name__=='__main__': main()
