"""Move the terminal-byte specialization onto methods so one worker can reuse one scratch."""
def materialize(s):
    s=s.replace('const MODE:u8,const SCHEDULE:u8=1,const ONLY:bool=true','const MODE:u8,const SCHEDULE:u8=1')
    s=s.replace('const MODE:u8,const SCHEDULE:u8,const ONLY:bool','const MODE:u8,const SCHEDULE:u8')
    s=s.replace('Dispatch<MODE,SCHEDULE,ONLY>','Dispatch<MODE,SCHEDULE>')
    for method in ['prepare','prepare_body','prepare_counted','prepare_pop','transfer_mark','build','build_bmi','build_pop','build_body','inverse_merge','first']:
        original=f'fn {method}('
        assert s.count(original)==1,method
        s=s.replace(original,f'fn {method}<const ONLY:bool>(').replace(f'self.{method}(',f'self.{method}::<ONLY>(')
    return s
