# This script takes care of testing your crate

set -ex

carto test --target $TARGET
cargo build --target $TARGET --release
