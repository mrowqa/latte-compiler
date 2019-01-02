#!/bin/sh

clang++ -fno-builtin -O3 -S -emit-llvm -o lib/runtime.{ll,cpp} && \
llvm-as lib/runtime.ll && \
rm -f lib/runtime.o

