#!/bin/bash

# ./extract_id_label.sh src/EDAM_1.25.csv > src/EDAM_1.25.id_label.csv

original_file=$1

awk -F',' 'NR==1 || $1 ~ /format_/' "$original_file" |  cut -d',' -f1,2 