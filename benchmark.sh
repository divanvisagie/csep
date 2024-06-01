#!/usr/bin/env bash

hyperfine --warmup 3 "csep 'file containing typescript'"
