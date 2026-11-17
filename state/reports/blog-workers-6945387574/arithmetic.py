import json, math
fits = [(1.5,.300,.0248),(1.75,.370,.0129),(2,.422,.0068),(2.5,.494,.0020)]
rows=[]
for exponent,a,b in fits:
    c=a+b*7**exponent
    optimum=(a/(b*(exponent-1)))**(1/exponent)
    rows.append(dict(exponent=exponent,c7=c,contention_share=(c-a)/c,conditional_speedup=c/a,continuous_optimum=optimum,share_at_optimum=1/exponent))
a,b=.422,.0068
out=dict(fits=rows,quadratic_wall_proxy={str(w):(a+b*w*w)/w for w in [6,7,8,9,10]},wall8_over7=((a+b*64)/8)/((a+b*49)/7),cycles_saved_per_miss=(22.98-17.32)/(.01964-.01002),relative_L2_increase=.712/.689-1,relative_LLC_decrease=1-.01002/.01964,wall_reduction_at_ratio_0885=1-.885,throughput_gain_at_ratio_0885=1/.885-1,buffer_ratio=5.97/16.81)
assert out['wall8_over7']<1
assert abs(rows[2]['contention_share']-.4412076271)<1e-8
print(json.dumps(out,indent=2))
