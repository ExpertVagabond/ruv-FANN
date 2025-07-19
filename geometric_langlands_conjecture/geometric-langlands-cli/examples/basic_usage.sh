#!/bin/bash
# Basic usage examples for Langlands CLI

echo "=== Geometric Langlands CLI Examples ==="
echo

echo "1. Computing Langlands correspondence for GL(3)..."
langlands compute correspondence --input "GL(3)" --output gl3_correspondence.json --parallel
echo

echo "2. Visualizing Hecke eigenvalues..."
langlands visual hecke-eigenvalues --resolution 1024x768 --output hecke_plot.png
echo

echo "3. Training neural network on correspondence patterns..."
langlands train --dataset synthetic_data.json --architecture langlands_v1 --epochs 50 --save-model trained_model.bin
echo

echo "4. Verifying Ramanujan conjecture..."
langlands verify ramanujan --depth standard --proof
echo

echo "5. Starting interactive REPL session..."
echo "Type 'quit' to exit the REPL"
langlands repl --auto-save --history examples_history.txt