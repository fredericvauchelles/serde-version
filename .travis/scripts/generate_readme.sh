#!/bin/bash

TARGET=README.md

cat README.md.tpl > $TARGET
sed -n 's/\/\/!/\/\/!/p' serde-version/src/lib.rs >> $TARGET
sed -i 's/\/\/!\s//g' $TARGET
sed -i 's/```dont_compile/```rust/g' $TARGET
sed -i 's/```compile_fail/```rust/g' $TARGET