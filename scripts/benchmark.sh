#!/usr/bin/env bash

echo "Benchmarking cache hit time:"
hyperfine --warmup 3 "csep 'file containing typescript'"


echo "Benchmarking cache build time for fastembed:"
hyperfine --warmup 3 "csep cache -c; csep --client fastembed 'file containing typescript'"

echo "Benchmarking cache build time for ollama:"
hyperfine --warmup 3 "csep cache -c; csep --client ollama 'file containing typescript'"
