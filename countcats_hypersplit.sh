#!/bin/bash

echo "Configuring process-minion-out..."
make -s configure

echo "Setting up directories..."
mkdir -p minion-files
mkdir -p minion-out
mkdir -p database

for ((i = 0; i <= $(($1-$2)); i++))
do
    for ((j = 0; j <= $(($1-$2-$i)); j++))
    do
        for ((k = 0; 2*k <= $(($1-$2-$i-$j)); k++))
        do
            echo "[$i $j $k]"
            if ! [ -f minion-files/category$1-$2-$i-$j-$k.minion ]
            then
                echo "Writing minion file..."
                python3 generate-minion-file-hypersplit.py $1 $2 $i $j $k > minion-files/category$1-$2-$i-$j-$k.minion
            fi
            if ! [ -f minion-out/out$1-$2-$i-$j-$k.txt ]
            then
                echo "Running minion..."
                minion -findallsols -noprintsols -solsout minion-out/out$1-$2-$i-$j-$k.txt minion-files/category$1-$2-$i-$j-$k.minion > /dev/null 2>&1
            fi
            if ! [ -f database/cats$1-$2-$i-$j-$k.txt ]
            then
                echo "Churning through the results..."
                ./bin/process-minion-out minion-out/out$1-$2-$i-$j-$k.txt database/cats$1-$2-$i-$j-$k.txt $1 $2 $i $j $k
                echo "Deleting output file to save disk space..."
                rm -f minion-out/out$1-$2-$i-$j-$k.txt
            fi
        done
    done
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