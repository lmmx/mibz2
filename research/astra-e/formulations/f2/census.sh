#!/usr/bin/env bash
# Input directory contains capped tapes and stream_index,rows,primary,records lines.
set -euo pipefail
dir=${1:?input directory}
exe=${2:?factor executable}
echo 'stream,rows,caps,runs,delimiter_byte_values,token_rows,nodes,covered_rows,selector_intersections,selectors,run_searches,words,dictionary_bytes,aliases,merge_pieces,subtree_runs,peak_live_output_runs,token_runs,mark_rewind,node_bytes,run_bytes'
while IFS=, read -r idx n primary records; do
    for delim in '"' ',' '",' '":,{}[]'; do
        "$exe" "$dir/$idx.recs" "$idx" "$primary" "$delim"
    done
done < "$dir/inputs.csv"
