#!/bin/bash

echo "Compiling process-minion-out..."
make configure

echo "Setting up directories..."
mkdir -p minion-files
mkdir -p minion-out
mkdir -p database

for ((i = 0; i <= $(($1-$2)); i++))
do
    echo "[$i non-identity idempotents]"
    echo "Writing minion file..."
    rm -f minion-files/category$1-$2-$i.minion
    python3 generate-minion-file-split.py $1 $2 $i > minion-files/category$1-$2-$i.minion
    echo "Running minion..."
    rm -f minion-out/out$1-$2-$i.txt
    minion -findallsols -noprintsols -solsout minion-out/out$1-$2-$i.txt minion-files/category$1-$2-$i.minion > /dev/null 2>&1
    echo "Churning through the results..."
    rm -f database/cats$1-$2-$i.txt
    ./bin/process-minion-out minion-out/out$1-$2-$i.txt database/cats$1-$2-$i.txt $1 $2 $i
    echo "Deleting output file to save disk space..."
    rm -f minion-out/out$1-$2-$i.txt
done
echo "Merging results..."
rm -f database/cats$1-$2.txt
cat database/cats$1-$2-*.txt > database/cats$1-$2.txt
echo "Sorting output file..."
sort -o database/cats$1-$2.txt database/cats$1-$2.txt
echo "Deleting temporary files..."
rm -f database/cats$1-$2-*.txt
echo "Counting..."
wc -l < database/cats$1-$2.txt