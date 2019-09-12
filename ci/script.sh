# This script takes care of testing your crate

set -ex

cargo test --target $TARGET
cargo build --target $TARGET --release
