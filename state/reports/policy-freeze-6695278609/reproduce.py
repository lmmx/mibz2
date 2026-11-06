"""Review-only boundary checks: no decoder builds, corpus runs or timing measurements."""
import json
import subprocess
import tempfile
from pathlib import Path

recipe = Path(__file__).parent / 'inputs/bench.just'
line = next(x.strip() for x in recipe.read_text().splitlines() if x.strip().startswith('measured='))
with tempfile.TemporaryDirectory() as td:
    p = Path(td)
    def git(*args):
        return subprocess.check_output(['git', '-C', td, *args], text=True).strip()
    git('init', '-q')
    (p/'Cargo.toml').write_text('[package]\nname="fixture"\nversion="0.1.0"\nedition="2021"\n')
    (p/'src').mkdir()
    (p/'src/lib.rs').write_text('pub fn f() {}\n')
    git('add', '.')
    git('-c','user.name=Louis Maddox','-c','user.email=lmmx@users.noreply.github.com','commit','-qm','fixture')
    (p/'build.rs').write_text('fn main() { println!("cargo:rustc-cfg=review_fixture"); }\n')
    checked = subprocess.check_output(['bash','-c',line+'\ngit status --porcelain --untracked-files=all -- "${measured[@]}"'],cwd=td,text=True).strip()
    result={'untracked_root_build_script': {'tracked_only_status':git('status','--porcelain','--untracked-files=no'),'measured_status':checked,'full_status':git('status','--porcelain','--untracked-files=all')}}
    assert not checked and not result['untracked_root_build_script']['tracked_only_status']
    assert 'build.rs' in result['untracked_root_build_script']['full_status']
    frozen=git('rev-parse','HEAD')
    git('add','build.rs')
    git('-c','user.name=Louis Maddox','-c','user.email=lmmx@users.noreply.github.com','commit','-qm','additional build input')
    changed=subprocess.run(['bash','-c',line+'\ngit diff --quiet "$1" HEAD -- "${measured[@]}"','fixture',frozen],cwd=td).returncode
    result['clean_later_head_build_script']={'full_status':git('status','--porcelain','--untracked-files=all'),'measured_diff_exit':changed,'all_diff':git('diff','--name-only',frozen,'HEAD')}
    assert changed==0 and result['clean_later_head_build_script']['all_diff']=='build.rs'
    (p/'rust-toolchain.toml').write_text('[toolchain]\nchannel="stable"\n')
    checked=subprocess.check_output(['bash','-c',line+'\ngit status --porcelain --untracked-files=all -- "${measured[@]}"'],cwd=td,text=True).strip()
    result['untracked_toolchain_file']={'measured_status':checked,'full_status':git('status','--porcelain','--untracked-files=all')}
    assert not checked

# Source-level state-transition replay of Census's Vacant-only insertion, not a live
# /proc test: this runtime exposes no /proc/self/task, so that boundary cannot run here.
observations=[{12:'mibz2-0'}, {12:'mibz2-coop-1'}, {12:'mibz2-coop-1'}]
seen={}
for tasks in observations:
    for tid,name in tasks.items():
        if tid not in seen: seen[tid]=name
result['first_name_cache_model']={'observations':observations,'cached':seen,'helper_count_cached':sum(n.startswith('mibz2-coop') for n in seen.values()),'helper_count_final_observation':sum(n.startswith('mibz2-coop') for n in observations[-1].values())}
assert result['first_name_cache_model']['helper_count_cached']==0
assert result['first_name_cache_model']['helper_count_final_observation']==1
refresh={}
for tasks in observations: refresh.update(tasks)
result['refresh_model_helper_count']=sum(n.startswith('mibz2-coop') for n in refresh.values())
assert result['refresh_model_helper_count']==1
print(json.dumps(result,indent=2))
