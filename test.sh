#!/bin/sh
cargo r examples/test.c
./a.out
echo $?
