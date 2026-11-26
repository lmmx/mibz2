"""Replay the uninstrumented constructor's exact wires with external dense clock oracles and independent bzip2."""
import bz2,csv,hashlib,importlib.util,subprocess,sys,tempfile
from pathlib import Path
here=Path(__file__).resolve().parent
powers=here.parents[3]/'powers'
sys.path.insert(0,str(powers))
spec=importlib.util.spec_from_file_location('gate_bzip2',powers/'real.py')
oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
native,writer,data,corpus=map(lambda p:Path(p).resolve(),sys.argv[1:])
records=list(csv.DictReader((here/'blocks.csv').open()))
refs={}
with tempfile.TemporaryDirectory() as tmp:
    tmp=Path(tmp)
    (tmp/'sample.bz2').symlink_to(corpus)
    subprocess.run(['bzip2recover','sample.bz2'],cwd=tmp,check=True,capture_output=True)
    out=csv.writer(sys.stdout,lineterminator='\n')
    out.writerow(('index','k','wire_agrees','pre_rle_sha256','decoded_sha256','block_crc','bytes_after_rle1'))
    for r in records:
        idx=int(r['index']);k=r['k'];src=data/'raw'/(str(idx)+'.txt')
        subprocess.run([native,'dump',src,tmp,k],check=True)
        model=tmp/(k+'.ctx')
        assert hashlib.sha256(model.read_bytes()).hexdigest()==r['sha256']
        raw=subprocess.check_output([writer,src,model])
        if idx not in refs:
            known,crc=oracle.recovered(tmp,idx);actual=oracle.rle1(raw)
            assert actual==known and oracle.crc32_bzip2(actual)==crc
            refs[idx]=(raw,hashlib.sha256(actual).hexdigest(),f'{crc:08x}',len(actual))
        first,digest,crc,n=refs[idx];assert first==raw
        out.writerow([idx,k,1,hashlib.sha256(raw).hexdigest(),digest,crc,n])
