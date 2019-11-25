#!/bin/bash

# Expected environment variables:
# - CARGO_TOKEN: token for cargo login

TOOLCHAIN=$1

if [ -z "$CARGO_TOKEN" ]; then
  echo "ERROR: Missing environment variable CARGO_TOKEN";
  exit 1;
fi

echo "INFO: Login with cargo";
cargo login "$CARGO_TOKEN" || exit 1;

echo "INFO: Publish serde-version-derive";
cd serde_version_derive;
cargo +$TOOLCHAIN publish --allow-dirty || exit 1;

echo "wait crates.io registry to update";
sleep 10

echo "INFO: update registry with an install command";
cargo +$TOOLCHAIN install lazy_static;

echo "INFO: Publish serde-version";
cd ../serde-version;
cargo +$TOOLCHAIN publish --allow-dirty || exit 1;