"""Blind precision for a single predeclared pooled wall test using the frozen edge receipt only."""
import csv
import hashlib
import json
import math
import tarfile
from pathlib import Path

import numpy as np
from scipy.stats import chi2, nct, t

HERE=Path(__file__).resolve().parent
ARCHIVE=HERE.parent/"evidence/edge-i9-683c22e.tar.gz"
DIGEST="3a10607c7d4ff026f4f3a7241a6d72dff10f590c07fd9b19eab9a7c6fbed2559"
SEED=683022
TRIALS=100000
EFFECTS=(0,.0044,.008,.02)


def save_csv(path,records):
    with path.open("w") as stream:
        writer=csv.DictWriter(stream,fieldnames=records[0],lineterminator="\n")
        writer.writeheader();writer.writerows(records)


def historical():
    assert hashlib.sha256(ARCHIVE.read_bytes()).hexdigest()==DIGEST
    with tarfile.open(ARCHIVE) as archive:
        def content(name):
            return archive.extractfile("run-20261119T022117Z/"+name).read()
        data=[json.loads(line) for line in content("runs.jsonl").splitlines()]
        assert json.loads(content("roster.json"))==[[r["round"],r["workers"],r["arm"]] for r in data]
        assert content("mode.txt").strip()==b"measure"
    d={(r["round"],r["workers"],r["arm"]):r for r in data}
    assert len(d)==len(data)==256 and all(r["out_bytes"]==838800784 for r in data)
    rows=[]
    for j in range(16):
        row={"round":j}
        for w in range(7,11):
            ratio=d[j,w,"candidate"]["wall_s"]/d[j,w,"predecessor"]["wall_s"]
            row[f"w{w}_log_ratio"]=math.log(ratio)
        row["four_w_mean"]=sum(row[f"w{w}_log_ratio"] for w in range(7,11))/4
        rows.append(row)
    save_csv(HERE/"rounds.csv",rows)
    return np.array([[r[f"w{w}_log_ratio"] for w in range(7,11)] for r in rows])


def directional_power(n, effect, sd):
    # Success is the preregistered direction at a two-sided 95% interval.
    crit=t.ppf(.975,n-1)
    return float(nct.cdf(-crit,n-1,math.log1p(-effect)*math.sqrt(n)/sd))


def resample(centered,n,design,rng):
    if design=="independent_complete_rounds":
        idx=rng.integers(0,16,size=(2000,n))
    elif design=="balanced_by_eight_rotor_phases":
        # Alternate past measurements with the same rotor phase. Each complete
        # round is sampled as a unit; each phase has exactly n/8 future visits.
        idx=np.tile(np.arange(8),n//8)+8*rng.integers(0,2,size=(2000,n))
    else:
        # Length-four circular blocks allow adjacent historical rounds to covary.
        # This sensitivity does not assert a particular temporal noise law.
        start=rng.integers(0,16,size=(2000,n//4,1))
        idx=((start+np.arange(4))%16).reshape(2000,n)
    return centered[idx]


def main():
    x=historical(); y=x.mean(axis=1); mu=y.mean(); sd=y.std(ddof=1)
    cov=np.cov(x,rowvar=False,ddof=1)
    assert math.isclose(float(cov.sum()/16),sd*sd)
    lo=sd*math.sqrt(15/chi2.ppf(.975,15))
    hi=sd*math.sqrt(15/chi2.ppf(.025,15))
    # Center the aggregate once. The named effects are hypothetical common log
    # shifts; historical candidate/predecessor mean gain is removed.
    center=y-mu; rng=np.random.default_rng(SEED)
    analytic=[]; empirical=[]
    for n in [32,48]:
        ci=t.ppf(.975,n-1)
        for e in EFFECTS:
            analytic.append(dict(rounds=n,wall_reduction=e,
                tcrit_975=ci,assumed_round_sd=sd,
                upper_multiplicative_halfwidth=math.expm1(ci*sd/math.sqrt(n)),
                probability_upper_95_below_one=directional_power(n,e,sd),
                probability_at_upper_sd=directional_power(n,e,hi),
                probability_at_lower_sd=directional_power(n,e,lo),
                probability_upper_onesided95_below_one=float(nct.cdf(-t.ppf(.95,n-1),n-1,
                                                    math.log1p(-e)*math.sqrt(n)/sd))))
        for design in ["independent_complete_rounds","balanced_by_eight_rotor_phases","circular_four_round_blocks"]:
            found=dict.fromkeys(EFFECTS,0)
            for _ in range(TRIALS//2000):
                sample=resample(center,n,design,rng)
                upper=sample.mean(axis=1)+ci*sample.std(axis=1,ddof=1)/math.sqrt(n)
                for effect in EFFECTS:found[effect]+=int(np.count_nonzero(upper+math.log1p(-effect)<0))
            for effect,num in found.items():
                p=num/TRIALS
                empirical.append(dict(rounds=n,design=design,wall_reduction=effect,trials=TRIALS,
                                      fraction_upper_95_below_one=p,mc_standard_error=math.sqrt(p*(1-p)/TRIALS)))
    maxn=min(n for n in range(3,401) if directional_power(n,.0044,sd)>=.8)
    info=dict(source_archive_sha256=DIGEST,measure="log(edge wall / combined predecessor wall)",
              aggregate="mean of four W-specific paired logs at each fixed round",
              n_historical_rounds=len(y),seed=SEED,trials=TRIALS,workers=[7,8,9,10],
              historical_center=mu,round_log_sd=sd,
              hypothetical_if_w_were_independent_sd=math.sqrt(np.trace(cov))/4,
              sd_normal_iid_95=[lo,hi],cross_worker_covariance=cov.tolist(),
              cross_worker_correlation=np.corrcoef(x,rowvar=False).tolist(),
              consecutive_round_correlation=np.corrcoef(y[:-1],y[1:])[0,1],
              illustrative_n_for_80pct_at_0044=maxn,
              historical_rotor_phase_log_means=[float(y[i::8].mean()) for i in range(8)])
    (HERE/"dispersion.json").write_text(json.dumps(info,indent=2)+"\n")
    save_csv(HERE/"analytic.csv",analytic);save_csv(HERE/"resampling.csv",empirical)


if __name__=="__main__":
    main()
