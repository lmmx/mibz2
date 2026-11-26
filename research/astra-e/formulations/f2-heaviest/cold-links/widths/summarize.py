import csv
from pathlib import Path
HERE=Path(__file__).resolve().parent

def records(path):
    with path.open() as f:return list(csv.DictReader(f))
real=records(HERE/"real.csv")
hot={(int(r["stream"]),int(r["threshold"])):r for r in records(HERE/"hot-waves.csv")}
marks={(int(r["stream"]),int(r["threshold"])):r for r in records(HERE/"forward-marks.csv")}
assert len(real)==len(hot)==len(marks)==40
for r in real:
    key=int(r["stream"]),int(r["threshold"])
    assert int(r["rows"])==int(marks[key]["output_bytes"])
    assert int(r["first_hot_queries"])==int(hot[key]["hot_queries"])
    assert int(r["tail_row_queries"])==int(r["backward_row_queries"])
    assert int(r["tail_table"])<=int(r["source_runs"])+2*int(r["cold_groups"])
    assert int(r["barrier_rows"])==int(r["second_rows"])+int(r["cold_rows"])
    for f in ("ordinary_waves","hot_waves","hot_critical"):r[f]=hot[key][f]
    for f in ("initial_queries","cold_queries","macro_visits","cold_macro_visits"):r[f]=marks[key][f]
print("all comparisons use the same nine canonical admissions; columns are width thresholds 1,2,4,8")
rows=[r for r in real if int(r["stream"])!=41]
for f in ("first_original_queries","ordinary_waves","source_runs","cold_groups","mixed_groups","fully_affine_groups","cold_rows",
          "hidden_word_types","first_hot_queries","hot_waves","eager_saved_queries","backward_row_queries",
          "elided_distinct_dict_bytes","weighted_cold_bytes","barrier_rows","regularity_queries","partial_fm_intersections",
          "hot_dict","dangling_known_bytes","tail_table","tail_entrances","tail_lookups","tail_pieces","tail_rank_waves8",
          "tail_piece_waves8","tail_ideal_prefix_queries","tail_terminal_spans","tail_spell_bytes_by_span",
          "full_lookups","full_pieces","full_spell_bytes_by_span","initial_queries","cold_queries","macro_visits"):
    print(f,*[sum(int(r[f]) for r in rows if int(r["threshold"])==k) for k in (1,2,4,8)])
for key,fs in (("hot_plus_Psi_probes",("first_hot_queries","tail_lookups")),
               ("hot_plus_Psi_rank_waves",("hot_waves","tail_rank_waves8")),
               ("first_plus_second_probes",("first_hot_queries","tail_lookups","regularity_queries"))):
    print(key,*[sum(sum(int(r[f]) for f in fs) for r in rows if int(r["threshold"])==k) for k in (1,2,4,8)])
