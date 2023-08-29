#!/bin/bash

echo "Compiling process-minion-out..."
make configure

echo "Setting up directories..."
mkdir -p minion-files
mkdir -p minion-out
mkdir -p database

echo "Writing minion file..."
rm -f minion-files/category$1-$2.minion
python3 generate-minion-file.py $1 $2 > minion-files/category$1-$2.minion
echo "Running minion..."
rm -f minion-out/out$1-$2.txt
minion -findallsols -noprintsols -solsout minion-out/out$1-$2.txt minion-files/category$1-$2.minion > /dev/null 2>&1
echo "Churning through the results..."
rm -f database/cats$1-$2.txt
./bin/process-minion-out minion-out/out$1-$2.txt database/cats$1-$2.txt $1 $2