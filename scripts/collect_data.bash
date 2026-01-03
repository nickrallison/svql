#!/bin/bash

export RUST_LOG=error
mkdir -p bin/res/

CONFIGS=(
    "cv32e"
    "soc_intercon"
    "soc_periph"
    "udma"
    "tile"
    "e203"
)

QUERIES=(
    "Cwe1234"
    "Cwe1271"
    "Cwe1280"
)

for cfg in "${CONFIGS[@]}"; do
    JSON_FILE="scripts/collect_data_${cfg}.json"
    
    if [ ! -f "$JSON_FILE" ]; then
        echo "Skipping $cfg: JSON not found"
        continue
    fi

    for q in "${QUERIES[@]}"; do
        echo "Processing: Design=$cfg | Query=$q"

        # Parallel Run
        cargo run --bin collect_data --release \
            --features svql_subgraph/rayon \
            --features svql_query/parallel \
            -- --config "$JSON_FILE" --query "$q" --format pretty > "bin/res/results_${cfg}_${q}_par.txt"

        # Single-Threaded Run
        cargo run --bin collect_data --release \
            -- --config "$JSON_FILE" --query "$q" --format pretty > "bin/res/results_${cfg}_${q}_st.txt"
    done
done

echo "Done. Results in ./bin/"