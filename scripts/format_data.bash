#!/bin/bash

# Check if bin directory exists
if [ ! -d "bin" ]; then
    echo "Error: bin directory not found"
    exit 1
fi
rm -f bin/format_data.txt
touch bin/format_data.txt

for file in bin/res/results_*.txt; do
    echo "------------------------------------------------------------" >> bin/format_data.txt
    echo "FILE: $file" >> bin/format_data.txt
    echo "------------------------------------------------------------" >> bin/format_data.txt
    
    # grep -A 100 "PERFORMANCE SUMMARY" "$file" >> bin/format_data.txt
    grep -A 20 "Design:" "$file" >> bin/format_data.txt
    
    echo -e "\n" >> bin/format_data.txt
done