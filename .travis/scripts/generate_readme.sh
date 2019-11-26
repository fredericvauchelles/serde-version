#!/bin/bash

cat README.md.tpl > README.md
sed -n 's/\/\/!/\/\/!/p' serde-version/src/lib.rs >> README.md
# sed -i 's/\/\/!\s#.*//g' README.md
sed -i 's/\/\/!\s//g' README.md
sed -i 's/```dont_compile/```rust/g' README.md
sed -i 's/```compile_fail/```rust/g' README.md